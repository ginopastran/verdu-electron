mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use andexmarket::app::{App, Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::cierre::resultado::DialogoResultadoCierre;
use andexmarket::pantallas::cierre::{DialogoCierre, ResultadoCierre};
use andexmarket::pantallas::pedidos::DialogoPedidos;
use andexmarket::ui::v2;
use chrono::Duration as Dias;
use eframe::egui;
use egui_kittest::Harness;
use nucleo::api::Cliente;
use nucleo::cierre::ahora_argentina;
use serde_json::{json, Value};

const TAMANOS: [(f32, f32); 2] = [(1920.0, 1080.0), (1366.0, 768.0)];

#[derive(Clone, Debug)]
struct Pedido {
    metodo: String,
    ruta: String,
    encabezados: Vec<(String, String)>,
    cuerpo: Value,
}

type Responder = dyn Fn(&str, &str) -> Option<(u16, Value)> + Send + Sync;

fn servidor(responder: impl Fn(&str, &str) -> Option<(u16, Value)> + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Pedido>>>) {
    let escucha = TcpListener::bind("127.0.0.1:0").expect("puerto libre");
    let base = format!("http://{}", escucha.local_addr().unwrap());
    let pedidos = Arc::new(Mutex::new(Vec::new()));
    let registro = pedidos.clone();
    let responder: Arc<Responder> = Arc::new(responder);
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
            let registro = registro.clone();
            let responder = responder.clone();
            std::thread::spawn(move || {
                let mut lector = BufReader::new(conexion.try_clone().unwrap());
                let mut linea = String::new();
                lector.read_line(&mut linea).unwrap();
                let partes: Vec<&str> = linea.split_whitespace().collect();
                let (metodo, ruta) = (partes[0].to_string(), partes[1].to_string());
                let mut encabezados = Vec::new();
                let mut largo = 0;
                loop {
                    let mut linea = String::new();
                    lector.read_line(&mut linea).unwrap();
                    let linea = linea.trim_end();
                    if linea.is_empty() {
                        break;
                    }
                    if let Some((clave, valor)) = linea.split_once(':') {
                        if clave.eq_ignore_ascii_case("content-length") {
                            largo = valor.trim().parse().unwrap_or(0);
                        }
                        encabezados.push((clave.to_ascii_lowercase(), valor.trim().to_string()));
                    }
                }
                let mut cuerpo = vec![0; largo];
                lector.read_exact(&mut cuerpo).unwrap();
                let cuerpo = serde_json::from_slice(&cuerpo).unwrap_or(Value::Null);
                if !ruta.starts_with("/api/productos") && !ruta.starts_with("/api/business/logo-comprobante") {
                    registro.lock().unwrap().push(Pedido { metodo: metodo.clone(), ruta: ruta.clone(), encabezados, cuerpo });
                }
                let Some((estado, respuesta)) = responder(&metodo, &ruta) else {
                    std::thread::sleep(Duration::from_secs(600));
                    return;
                };
                let texto = respuesta.to_string();
                let mut salida = conexion;
                let _ = write!(
                    salida,
                    "HTTP/1.1 {estado} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}",
                    texto.len()
                );
            });
        }
    });
    (base, pedidos)
}

fn servicios_con(base: &str) -> Servicios {
    let mut servicios = comun::servicios();
    servicios.api = Cliente::new(base);
    servicios.config.set("impresoraWindows", json!("AndexMarket-impresora-de-prueba-inexistente"));
    servicios
}

fn global() -> Global {
    let mut global = comun::global(Ruta::Carrito);
    global.usuario = Some(json!({
        "id": 11,
        "nombre": "Lucía Fernández",
        "email": "lucia@donjose.com.ar",
        "sucursalId": 1,
        "role": "VENDEDOR",
        "permisos": { "pesoManualEnabled": false, "cierreDeCajaEnabled": true }
    }));
    global.admin = Some(json!({ "id": 1, "businessId": 7, "businessName": "Verdulería Don José" }));
    global.negocio = Some(json!({ "id": 7, "nombre": "Verdulería Don José", "facturacionHabilitada": true, "dobleImpresionEnabled": false }));
    global
}

struct Escena {
    app: App,
    servicios: Servicios,
    cierre: DialogoCierre,
    pedidos: DialogoPedidos,
    resultado: DialogoResultadoCierre,
    completados: usize,
}

fn escena(servicios: Servicios, tam: (f32, f32)) -> Harness<'static, Escena> {
    let estado = Escena {
        app: App::de_prueba(global(), servicios.clone()),
        servicios,
        cierre: DialogoCierre::default(),
        pedidos: DialogoPedidos::default(),
        resultado: DialogoResultadoCierre::default(),
        completados: 0,
    };
    let mut harness = Harness::builder().with_size([tam.0, tam.1]).wgpu().build_ui_state(
        |ui, e: &mut Escena| {
            e.app.mostrar(ui);
            let mut cx = Contexto { global: e.app.global(), servicios: &e.servicios, egui: ui.ctx().clone() };
            if let Some(ResultadoCierre::Completado) = e.cierre.mostrar(ui, &mut cx) {
                e.completados += 1;
            }
            e.pedidos.mostrar(ui, &mut cx);
            e.resultado.mostrar(ui, &mut cx);
            v2::toast::pintar(ui.ctx());
        },
        estado,
    );
    harness.run_steps(2);
    harness
}

fn clic_centro(harness: &mut Harness<'_, Escena>, tam: (f32, f32), dx: f32, dy: f32) {
    let pos = egui::pos2(tam.0 / 2.0 + dx, tam.1 / 2.0 + dy);
    harness.hover_at(pos);
    harness.step();
    harness.drag_at(pos);
    harness.step();
    harness.drop_at(pos);
    harness.step();
}

fn esperar(harness: &mut Harness<'_, Escena>, listo: impl Fn(&Escena) -> bool) {
    for _ in 0..200 {
        harness.step();
        if listo(harness.state()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("la tarea no terminó a tiempo");
}

fn guardar(harness: &mut Harness<'_, Escena>, nombre: &str, tam: (f32, f32)) {
    harness.run_steps(4);
    std::thread::sleep(Duration::from_millis(250));
    harness.run_steps(2);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}-{}.png", tam.0 as u32))).expect("guardar captura");
}

fn sin_servidor() -> Servicios {
    servicios_con("http://127.0.0.1:9")
}

fn cierre_ok() -> Value {
    let hoy = ahora_argentina().date();
    json!({
        "id": 311, "periodo": "todo", "fechaInicio": format!("{hoy}T00:00:00.000Z"), "fechaCierre": format!("{hoy}T23:59:59.999Z"),
        "totalVentas": 80750, "cantidadVentas": 11,
        "ventasPorMetodo": { "efectivo": 19800, "tarjeta": 24060, "qr": 18490, "split": { "total": 18400, "cantidad": 1 } },
        "ventasPorVendedor": [
            { "id": 11, "nombre": "Lucía Fernández", "email": "lucia@donjose.com.ar", "totalVentas": 59560, "cantidadVentas": 7 },
            { "id": 12, "nombre": "Sofía Medina", "email": "sofia@donjose.com.ar", "totalVentas": 11380, "cantidadVentas": 2 },
            { "id": 13, "nombre": "Diego Romero", "email": "diego@donjose.com.ar", "totalVentas": 9810, "cantidadVentas": 2 }
        ]
    })
}

fn orden(id: i64, hora: &str, metodo: &str, total: f64) -> Value {
    let hoy = ahora_argentina().date();
    json!({ "id": id, "fecha": format!("{hoy}T{hora}:00.000Z"), "metodoPago": metodo, "total": total, "tieneDescuento": false, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] })
}

fn ordenes_de_hoy(cuantas: usize) -> (Value, Value) {
    let mut descuento = orden(9002, "09:47", "efectivo", 2150.0);
    descuento["tieneDescuento"] = json!(true);
    descuento["tipoDescuento"] = json!("porcentual");
    descuento["valorDescuento"] = json!(10);
    descuento["montoDescuento"] = json!(239);
    descuento["subtotalSinDescuento"] = json!(2389);
    let mut ordenes = vec![
        orden(9006, "13:40", "tarjeta", 6430.0),
        orden(9005, "12:36", "tarjeta", 9710.0),
        orden(9004, "10:22", "split", 18400.0),
        descuento,
        orden(9001, "08:14", "efectivo", 4380.0),
        orden(8000, "08:00", "efectivo", 1.0),
    ];
    ordenes[5]["fecha"] = json!("2020-01-01T12:00:00.000Z");
    let hoy = ahora_argentina().date();
    let mut facturas = vec![
        json!({ "id": 501, "fechaEmision": format!("{hoy}T11:58:00.000Z"), "formaPago": "qr", "total": 5840 }),
        json!({ "id": 500, "fechaEmision": format!("{hoy}T08:41:00.000Z"), "formaPago": "qr", "total": 12650 }),
    ];
    let mut extra = 0;
    while ordenes.len() - 1 + facturas.len() < cuantas {
        let hora = format!("{:02}:{:02}", 14 + extra / 6, (extra * 9) % 60);
        let metodo = ["efectivo", "tarjeta", "qr", "split"][extra % 4];
        ordenes.push(orden(9100 + extra as i64, &hora, metodo, 4300.0 + 1000.0 * extra as f64));
        extra += 1;
    }
    if cuantas < 7 {
        facturas.clear();
    }
    (json!(ordenes), json!(facturas))
}

fn servidor_ordenes(cuantas: usize) -> (String, Arc<Mutex<Vec<Pedido>>>) {
    let (ordenes, facturas) = ordenes_de_hoy(cuantas);
    servidor(move |metodo, ruta| match (metodo, ruta) {
        ("GET", "/api/ordenes/vendedor/11") => Some((200, ordenes.clone())),
        ("GET", "/api/facturas/vendedor/11") => Some((200, facturas.clone())),
        _ => Some((404, json!({}))),
    })
}

fn listas(metodo: &str, ruta: &str) -> Option<(u16, Value)> {
    match (metodo, ruta) {
        ("GET", "/api/usuarios") => Some((200, json!([
            { "id": 11, "nombre": "Lucía Fernández", "rol": { "nombre": "VENDEDOR" }, "sucursalId": 1 },
            { "id": 1, "nombre": "Roberto Sánchez", "rol": { "nombre": "ADMIN" }, "sucursalId": 1 }
        ]))),
        ("GET", "/api/sucursales") => Some((200, json!([{ "id": 1, "nombre": "Sucursal Centro" }, { "id": 2, "nombre": "Sucursal Mercado" }]))),
        ("POST", "/api/cierres") => Some((200, cierre_ok())),
        _ => Some((404, json!({ "error": "no encontrado" }))),
    }
}

#[test]
fn cierre_caja() {
    for tam in TAMANOS {
        let mut harness = escena(sin_servidor(), tam);
        harness.state_mut().cierre.abrir();
        guardar(&mut harness, "54-modal-cierre-caja", tam);
    }
}

#[test]
fn cierre_caja_procesando() {
    for tam in TAMANOS {
        let (base, pedidos) = servidor(|_, _| None);
        let mut harness = escena(servicios_con(&base), tam);
        harness.state_mut().cierre.abrir();
        harness.run_steps(2);
        harness.key_press(egui::Key::Num3);
        harness.run_steps(2);
        assert!(harness.state().cierre.cerrando());
        for _ in 0..40 {
            if !pedidos.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        guardar(&mut harness, "55-modal-cierre-caja-procesando", tam);
        let pedido = pedidos.lock().unwrap()[0].clone();
        assert_eq!(pedido.metodo, "POST");
        assert_eq!(pedido.ruta, "/api/cierres");
        assert!(pedido.encabezados.iter().any(|(c, v)| c == "content-type" && v == "application/json"));
        assert_eq!(pedido.cuerpo["vendedorId"], 11);
        assert_eq!(pedido.cuerpo["sucursalId"], 1);
        assert_eq!(pedido.cuerpo["periodo"], "todo");
        let claves: Vec<&String> = pedido.cuerpo.as_object().unwrap().keys().collect();
        assert_eq!(claves, ["vendedorId", "sucursalId", "fechaInicio", "fechaCierre", "periodo"]);
        let ahora = ahora_argentina();
        assert_eq!(pedido.cuerpo["fechaInicio"], format!("{}T00:00:00.000Z", ahora.date()));
    }
}

#[test]
fn cierre_caja_clic_en_manana() {
    let tam = TAMANOS[1];
    let (base, pedidos) = servidor(|_, _| None);
    let mut harness = escena(servicios_con(&base), tam);
    harness.state_mut().cierre.abrir();
    harness.run_steps(4);
    clic_centro(&mut harness, tam, -120.0, -46.0);
    assert!(harness.state().cierre.cerrando());
    for _ in 0..40 {
        if !pedidos.lock().unwrap().is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(pedidos.lock().unwrap()[0].cuerpo["periodo"], "mañana");
}

#[test]
fn cierre_caja_atajos() {
    let (base, pedidos) = servidor(listas);
    let mut harness = escena(servicios_con(&base), TAMANOS[1]);
    harness.state_mut().cierre.abrir();
    harness.run_steps(2);
    harness.key_press(egui::Key::Num4);
    harness.run_steps(2);
    assert!(harness.state().cierre.manual_abierto());
    harness.key_press(egui::Key::Num2);
    harness.run_steps(2);
    assert!(!harness.state().cierre.cerrando());
    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    assert!(!harness.state().cierre.manual_abierto());
    assert!(harness.state().cierre.abierto());
    harness.key_press(egui::Key::Num3);
    esperar(&mut harness, |e| e.completados == 1);
    let cierres: Vec<Pedido> = pedidos.lock().unwrap().iter().filter(|p| p.ruta == "/api/cierres").cloned().collect();
    assert_eq!(cierres.len(), 1);
    assert_eq!(cierres[0].cuerpo["periodo"], "todo");
}

#[test]
fn cierre_caja_resultado() {
    for tam in TAMANOS {
        let (base, pedidos) = servidor(listas);
        let mut harness = escena(servicios_con(&base), tam);
        harness.state_mut().cierre.abrir();
        harness.run_steps(2);
        harness.key_press(egui::Key::Num3);
        esperar(&mut harness, |e| e.completados == 1);
        assert!(!harness.state().cierre.cerrando());
        assert!(harness.state().cierre.abierto());
        guardar(&mut harness, "59-modal-resultado-cierre", tam);
        assert_eq!(pedidos.lock().unwrap().len(), 1);
    }
}

fn abrir_manual(tam: (f32, f32)) -> (Harness<'static, Escena>, Arc<Mutex<Vec<Pedido>>>) {
    let (base, pedidos) = servidor(listas);
    let mut harness = escena(servicios_con(&base), tam);
    harness.state_mut().cierre.abrir();
    harness.run_steps(2);
    harness.key_press(egui::Key::Num4);
    esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 2);
    std::thread::sleep(Duration::from_millis(100));
    harness.run_steps(4);
    (harness, pedidos)
}

#[test]
fn cierre_manual() {
    let hoy = ahora_argentina().date();
    for tam in TAMANOS {
        let (mut harness, pedidos) = abrir_manual(tam);
        harness.state_mut().cierre.fijar_manual(hoy, hoy, [0, 0, 23, 59]);
        guardar(&mut harness, "56-modal-cierre-manual", tam);
        let rutas: Vec<String> = pedidos.lock().unwrap().iter().map(|p| p.ruta.clone()).collect();
        assert_eq!(rutas, ["/api/usuarios", "/api/sucursales"]);
        harness.state_mut().cierre.fijar_manual(hoy - Dias::days(1), hoy, [8, 0, 23, 59]);
        guardar(&mut harness, "57-modal-cierre-manual-varios-dias", tam);
        harness.state_mut().cierre.fijar_manual(hoy, hoy, [18, 0, 9, 0]);
        guardar(&mut harness, "58-modal-cierre-manual-error", tam);
    }
}

#[test]
fn cierre_manual_crea_el_cierre() {
    let tam = TAMANOS[1];
    let hoy = ahora_argentina().date();
    let (mut harness, pedidos) = abrir_manual(tam);
    harness.state_mut().cierre.fijar_manual(hoy, hoy, [0, 0, 23, 59]);
    harness.run_steps(2);
    clic_centro(&mut harness, tam, 197.0, 154.0);
    esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 3);
    let pedido = pedidos.lock().unwrap()[2].clone();
    assert_eq!(pedido.ruta, "/api/cierres");
    assert_eq!(pedido.cuerpo["periodo"], "custom");
    assert_eq!(pedido.cuerpo["vendedorId"], 11);
    assert_eq!(pedido.cuerpo["sucursalId"], 1);
    assert_eq!(pedido.cuerpo["fechaInicio"], format!("{hoy}T00:00:00.000Z"));
    assert_eq!(pedido.cuerpo["fechaCierre"], format!("{hoy}T23:59:59.999Z"));
    esperar(&mut harness, |e| e.completados == 1);
    assert!(!harness.state().cierre.manual_abierto());
    assert!(harness.state().cierre.abierto());
}

#[test]
fn cierre_manual_calendario() {
    let tam = TAMANOS[1];
    let (mut harness, _) = abrir_manual(tam);
    clic_centro(&mut harness, tam, -205.0, 14.0);
    guardar(&mut harness, "90-cierre-manual-calendario", tam);
    clic_centro(&mut harness, tam, 216.0, 14.0);
    guardar(&mut harness, "91-cierre-manual-hora", tam);
}

#[test]
fn ordenes() {
    for tam in TAMANOS {
        let (base, pedidos) = servidor_ordenes(7);
        let mut harness = escena(servicios_con(&base), tam);
        harness.state_mut().pedidos.abrir();
        harness.run_steps(2);
        esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 2);
        guardar(&mut harness, "51-modal-ordenes", tam);
        let rutas: Vec<String> = pedidos.lock().unwrap().iter().map(|p| p.ruta.clone()).collect();
        assert_eq!(rutas, ["/api/ordenes/vendedor/11", "/api/facturas/vendedor/11"]);
    }
}

#[test]
fn ordenes_paginacion() {
    for tam in TAMANOS {
        let (base, pedidos) = servidor_ordenes(14);
        let mut harness = escena(servicios_con(&base), tam);
        harness.state_mut().pedidos.abrir();
        harness.run_steps(2);
        esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 2);
        guardar(&mut harness, "52-modal-ordenes-paginacion", tam);
    }
}

#[test]
fn ordenes_cargando_y_vacio() {
    for tam in TAMANOS {
        let (base, _) = servidor(|_, _| None);
        let mut harness = escena(servicios_con(&base), tam);
        harness.state_mut().pedidos.abrir();
        harness.run_steps(3);
        guardar(&mut harness, "50-modal-ordenes-cargando", tam);
        let (base, pedidos) = servidor(|_, _| Some((200, json!([]))));
        let mut harness = escena(servicios_con(&base), tam);
        harness.state_mut().pedidos.abrir();
        harness.run_steps(2);
        esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 2);
        guardar(&mut harness, "53-modal-ordenes-vacio", tam);
    }
}

#[test]
fn resultado_del_cierre_manual() {
    let tam = TAMANOS[1];
    let mut harness = escena(sin_servidor(), tam);
    let mut cierre = cierre_ok();
    cierre["periodo"] = json!("custom");
    harness.state_mut().resultado.abrir(cierre);
    guardar(&mut harness, "92-resultado-cierre-manual", tam);
}
