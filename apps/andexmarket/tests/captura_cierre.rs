mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use andexmarket::app::{App, Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::cierre::resultado::DialogoResultadoCierre;
use andexmarket::pantallas::cierre::DialogoCierre;
use andexmarket::pantallas::pedidos::DialogoPedidos;
use chrono::{FixedOffset, Local, Offset, Utc};
use egui_kittest::Harness;
use eframe::egui;
use nucleo::api::Cliente;
use serde_json::{json, Value};

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
        "email": "lucia@lahuerta.test",
        "sucursalId": 1,
        "role": "VENDEDOR",
        "permisos": { "pesoManualEnabled": false, "cierreDeCajaEnabled": true }
    }));
    global.admin = Some(json!({ "id": 1, "businessId": 7, "businessName": "Verdulería La Huerta" }));
    global.negocio = Some(json!({ "id": 7, "nombre": "Verdulería La Huerta", "facturacionHabilitada": false, "dobleImpresionEnabled": false }));
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

fn escena(servicios: Servicios) -> Harness<'static, Escena> {
    let estado = Escena {
        app: App::de_prueba(global(), servicios.clone()),
        servicios,
        cierre: DialogoCierre::default(),
        pedidos: DialogoPedidos::default(),
        resultado: DialogoResultadoCierre::default(),
        completados: 0,
    };
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(
        |ui, e: &mut Escena| {
            e.app.mostrar(ui);
            let mut cx = Contexto { global: e.app.global(), servicios: &e.servicios, egui: ui.ctx().clone() };
            if let Some(andexmarket::pantallas::cierre::ResultadoCierre::Completado) = e.cierre.mostrar(ui, &mut cx) {
                e.completados += 1;
            }
            e.pedidos.mostrar(ui, &mut cx);
            e.resultado.mostrar(ui, &mut cx);
        },
        estado,
    );
    harness.run_steps(2);
    harness
}

fn clic(harness: &mut Harness<'_, Escena>, x: f32, y: f32) {
    let pos = egui::pos2(x, y);
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

fn guardar(harness: &mut Harness<'_, Escena>, nombre: &str) {
    harness.run_steps(4);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}

fn sin_servidor() -> Servicios {
    servicios_con("http://127.0.0.1:9")
}

fn cierre_ok() -> Value {
    json!({
        "id": 311, "periodo": "mañana", "fechaInicio": "2026-10-10T00:00:00.000Z", "fechaCierre": "2026-10-10T12:37:00.619Z",
        "totalVentas": 186450, "cantidadVentas": 27,
        "ventasPorMetodo": { "efectivo": 84300, "tarjeta": 61250, "qr": 32400, "transferencia": 8500 },
        "ventasPorVendedor": [
            { "id": 11, "nombre": "Lucía Fernández", "email": "lucia@lahuerta.test", "totalVentas": 112300, "cantidadVentas": 16 },
            { "id": 12, "nombre": "Martín Gómez", "email": "martin@lahuerta.test", "totalVentas": 74150, "cantidadVentas": 11 }
        ]
    })
}

fn ordenes_de_hoy() -> Value {
    let hoy = Utc::now().with_timezone(&FixedOffset::west_opt(3 * 3600).unwrap()).date_naive();
    let fecha = |hora: &str| format!("{hoy}T{hora}.370Z");
    json!([
        { "id": 9021, "fecha": fecha("15:23:37"), "metodoPago": "efectivo", "total": 7650, "tieneDescuento": true, "tipoDescuento": "porcentual", "valorDescuento": 10, "montoDescuento": 850, "subtotalSinDescuento": 8500, "vendedor": { "nombre": "Lucía Fernández" }, "items": [{ "nombre": "Tomate perito", "cantidad": 1.25, "precioHistorico": 2400, "subtotal": 3000 }] },
        { "id": 9020, "fecha": fecha("15:00:37"), "metodoPago": "qr", "total": 12480, "tieneDescuento": false, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] },
        { "id": 9019, "fecha": fecha("14:30:37"), "metodoPago": "tarjeta", "total": 23900, "tieneDescuento": false, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] },
        { "id": 9018, "fecha": fecha("13:55:37"), "metodoPago": "split", "total": 15300, "tieneDescuento": false, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] },
        { "id": 9017, "fecha": fecha("13:20:37"), "metodoPago": "efectivo", "total": 4350, "tieneDescuento": false, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] },
        { "id": 9016, "fecha": fecha("12:33:37"), "metodoPago": "tarjeta", "total": 9800, "tieneDescuento": true, "tipoDescuento": "cantidad", "valorDescuento": 500, "montoDescuento": 500, "subtotalSinDescuento": 10300, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] },
        { "id": 8000, "fecha": "2020-01-01T12:00:00.000Z", "metodoPago": "efectivo", "total": 1, "vendedor": { "nombre": "Lucía Fernández" }, "items": [] }
    ])
}

fn listas(metodo: &str, ruta: &str) -> Option<(u16, Value)> {
    match (metodo, ruta) {
        ("GET", "/api/usuarios") => Some((200, json!([
            { "id": 11, "nombre": "Lucía Fernández", "rol": { "nombre": "VENDEDOR" }, "sucursalId": 1 },
            { "id": 1, "nombre": "Roberto Sánchez", "rol": { "nombre": "ADMIN" }, "sucursalId": 1 }
        ]))),
        ("GET", "/api/sucursales") => Some((200, json!([{ "id": 1, "nombre": "Casa central" }, { "id": 2, "nombre": "Sucursal Mercado" }]))),
        ("POST", "/api/cierres") => Some((200, cierre_ok())),
        _ => Some((404, json!({ "error": "no encontrado" }))),
    }
}

#[test]
fn cierre_caja() {
    let mut harness = escena(sin_servidor());
    harness.state_mut().cierre.abrir();
    guardar(&mut harness, "37-cierre-caja");
}

#[test]
fn cierre_caja_procesando() {
    let (base, pedidos) = servidor(|_, _| None);
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().cierre.abrir();
    harness.run_steps(2);
    clic(&mut harness, 496.0, 367.0);
    esperar(&mut harness, |_| true);
    assert!(harness.state().cierre.cerrando());
    for _ in 0..40 {
        if !pedidos.lock().unwrap().is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    guardar(&mut harness, "38-cierre-caja-procesando");
    let pedido = pedidos.lock().unwrap()[0].clone();
    assert_eq!(pedido.metodo, "POST");
    assert_eq!(pedido.ruta, "/api/cierres");
    assert!(pedido.encabezados.iter().any(|(c, v)| c == "content-type" && v == "application/json"));
    assert_eq!(pedido.cuerpo["vendedorId"], 11);
    assert_eq!(pedido.cuerpo["sucursalId"], 1);
    assert_eq!(pedido.cuerpo["periodo"], "mañana");
    let claves: Vec<&String> = pedido.cuerpo.as_object().unwrap().keys().collect();
    assert_eq!(claves, ["vendedorId", "sucursalId", "fechaInicio", "fechaCierre", "periodo"]);
    if Local::now().offset().fix().local_minus_utc() == -3 * 3600 {
        let hoy = Local::now().date_naive();
        assert_eq!(pedido.cuerpo["fechaInicio"], format!("{hoy}T00:00:00.000Z"));
        let cierre = pedido.cuerpo["fechaCierre"].as_str().unwrap().to_string();
        assert!(cierre.starts_with(&Local::now().naive_local().format("%Y-%m-%dT%H").to_string()[..13]));
    }
}

#[test]
fn cierre_caja_resultado() {
    let (base, pedidos) = servidor(listas);
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().cierre.abrir();
    harness.run_steps(2);
    clic(&mut harness, 496.0, 367.0);
    esperar(&mut harness, |e| e.completados == 1);
    assert!(!harness.state().cierre.abierto());
    guardar(&mut harness, "39-cierre-caja-resultado");
    assert_eq!(pedidos.lock().unwrap().len(), 1);
}

#[test]
fn cierre_manual() {
    let (base, pedidos) = servidor(listas);
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().cierre.abrir();
    harness.run_steps(2);
    clic(&mut harness, 703.0, 512.0);
    esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 2);
    harness.run_steps(4);
    std::thread::sleep(Duration::from_millis(100));
    guardar(&mut harness, "40-cierre-manual");
    let rutas: Vec<String> = pedidos.lock().unwrap().iter().map(|p| p.ruta.clone()).collect();
    assert_eq!(rutas, ["/api/usuarios", "/api/sucursales"]);
    clic(&mut harness, 741.0, 663.0);
    esperar(&mut harness, |_| pedidos.lock().unwrap().len() >= 3);
    let pedido = pedidos.lock().unwrap()[2].clone();
    assert_eq!(pedido.ruta, "/api/cierres");
    assert_eq!(pedido.cuerpo["periodo"], "custom");
    assert_eq!(pedido.cuerpo["vendedorId"], 11);
    assert_eq!(pedido.cuerpo["sucursalId"], 1);
    if Local::now().offset().fix().local_minus_utc() == -3 * 3600 {
        let hoy = Local::now().date_naive();
        assert_eq!(pedido.cuerpo["fechaInicio"], format!("{hoy}T00:00:00.000Z"));
        assert_eq!(pedido.cuerpo["fechaCierre"], format!("{hoy}T23:59:59.999Z"));
    }
    guardar(&mut harness, "40b-cierre-manual-creando");
    esperar(&mut harness, |e| !e.cierre.manual_abierto());
    assert!(harness.state().cierre.abierto());
    guardar(&mut harness, "40d-cierre-manual-creado");
}

#[test]
fn cierre_manual_calendario() {
    let (base, _) = servidor(listas);
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().cierre.abrir();
    harness.run_steps(2);
    clic(&mut harness, 703.0, 512.0);
    harness.run_steps(4);
    std::thread::sleep(Duration::from_millis(200));
    clic(&mut harness, 496.0, 389.0);
    guardar(&mut harness, "40c-cierre-manual-calendario");
    clic(&mut harness, 532.0, 560.0);
    clic(&mut harness, 600.0, 131.0);
    guardar(&mut harness, "40e-cierre-manual-varios-dias");
}

#[test]
fn ordenes_recientes() {
    let (base, pedidos) = servidor(|metodo, ruta| match (metodo, ruta) {
        ("GET", "/api/ordenes/vendedor/11") => Some((200, ordenes_de_hoy())),
        _ => Some((404, json!({}))),
    });
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().pedidos.abrir();
    harness.run_steps(2);
    esperar(&mut harness, |_| !pedidos.lock().unwrap().is_empty());
    std::thread::sleep(Duration::from_millis(150));
    harness.run_steps(4);
    guardar(&mut harness, "36-ordenes-recientes");
    let rutas: Vec<String> = pedidos.lock().unwrap().iter().map(|p| p.ruta.clone()).collect();
    assert_eq!(rutas, ["/api/ordenes/vendedor/11"]);
}

#[test]
fn ordenes_recientes_cargando_y_vacio() {
    let (base, _) = servidor(|_, _| None);
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().pedidos.abrir();
    harness.run_steps(3);
    guardar(&mut harness, "36b-ordenes-cargando");
    let (base, pedidos) = servidor(|_, _| Some((200, json!([]))));
    let mut harness = escena(servicios_con(&base));
    harness.state_mut().pedidos.abrir();
    harness.run_steps(2);
    esperar(&mut harness, |_| !pedidos.lock().unwrap().is_empty());
    std::thread::sleep(Duration::from_millis(150));
    guardar(&mut harness, "36c-ordenes-vacio");
}

#[test]
fn resultado_del_cierre() {
    let mut harness = escena(sin_servidor());
    harness.state_mut().resultado.abrir(cierre_ok());
    guardar(&mut harness, "39b-cierre-resultado-dialogo");
}
