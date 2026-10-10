mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use andexmarket::app::{App, Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::facturas::formulario::{FormularioFactura, ProductoInicial};
use andexmarket::ui;
use eframe::egui::{self, Event, Pos2, PointerButton};
use egui_kittest::Harness;
use nucleo::api::Cliente;
use serde_json::{json, Value};

fn mock() -> Value {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidatos = [base.join("../../docs/referencia/mock.json"), base.join("../../../verdu-rust/docs/referencia/mock.json")];
    let ruta = candidatos.iter().find(|r| r.exists()).expect("mock.json de las capturas de referencia");
    serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap()
}

fn ejemplo(mock: &Value, metodo: &str, ruta: &str, indice: usize) -> Value {
    mock["endpoints"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["metodo"] == metodo && e["ruta"] == ruta)
        .map(|e| e["ejemplos"][indice]["respuesta"].clone())
        .unwrap_or(Value::Null)
}

#[derive(Clone, Copy, Default)]
struct Opciones {
    facturas_vacias: bool,
    demora_facturas: u64,
    precios_sin_iva: bool,
}

const PDF_FALSO: &str = "%PDF-1.4 factura de prueba";

fn responder(mock: &Value, opciones: Opciones, metodo: &str, ruta: &str, enviado: &Value) -> (u16, Value, u64) {
    let (camino, consulta) = ruta.split_once('?').unwrap_or((ruta, ""));
    let partes: Vec<&str> = camino.trim_start_matches('/').split('/').collect();
    let mut datos = mock["datos"].clone();
    if opciones.precios_sin_iva {
        for producto in datos["productos"].as_array_mut().unwrap() {
            producto["ivaIncluido"] = json!(false);
        }
    }
    let datos = &datos;
    let buscar = |lista: &Value, id: &str| {
        lista.as_array().unwrap().iter().find(|x| x["id"].as_str().map(str::to_string).unwrap_or(x["id"].to_string()) == id).cloned()
    };
    match (metodo, partes.as_slice()) {
        ("GET", ["api", "facturas"]) => {
            let vacias = opciones.facturas_vacias || consulta.contains("search=");
            let respuesta = ejemplo(mock, "GET", "/api/facturas", usize::from(vacias));
            (200, respuesta, opciones.demora_facturas)
        }
        ("GET", ["api", "facturas", "generar-pdf"]) => (200, json!(PDF_FALSO), 0),
        ("PUT", ["api", "facturas", id]) => match buscar(&datos["facturas"], id) {
            Some(mut f) => {
                for (clave, valor) in enviado.as_object().unwrap() {
                    f[clave] = valor.clone();
                }
                (200, f, 0)
            }
            None => (404, json!({ "error": "Factura no encontrada" }), 0),
        },
        ("DELETE", ["api", "facturas", _]) => (200, json!({ "message": "Factura anulada correctamente" }), 0),
        ("GET", ["api", "facturas", id]) => match buscar(&datos["facturas"], id) {
            Some(f) => (200, f, 0),
            None => (404, json!({ "error": "Factura no encontrada" }), 0),
        },
        ("GET", ["api", "clientes"]) => (200, ejemplo(mock, "GET", "/api/clientes", 0), 0),
        ("GET", ["api", "productos"]) => (200, json!({ "productos": datos["productos"] }), 0),
        ("GET", ["api", "productos", id]) => match buscar(&datos["productos"], id) {
            Some(p) => (200, p, 0),
            None => (404, json!({}), 0),
        },
        ("GET", ["api", "listas-precios"]) => (200, json!({ "listasPrecios": datos["listasPrecios"] }), 0),
        ("GET", ["api", "listas-precios", lista, "productos"]) => {
            let precios: Vec<Value> = datos["preciosLista"][*lista]
                .as_object()
                .map(|m| m.iter().map(|(id, precio)| json!({ "productoId": id.parse::<i64>().unwrap(), "precio": precio })).collect())
                .unwrap_or_default();
            (200, Value::Array(precios), 0)
        }
        ("GET", ["api", "listas-precios", lista, "productos", producto]) => match &datos["preciosLista"][*lista][*producto] {
            Value::Null => (404, json!({}), 0),
            precio => (200, json!({ "precio": precio }), 0),
        },
        ("POST", ["api", "facturas"]) => (200, json!({ "id": "f200", "numero": "R-0002-00000149" }), 0),
        ("POST", ["api", "facturas", "agregar-pago"]) => (200, json!({ "success": true, "pago": {} }), 0),
        _ => (404, json!({ "error": "sin mock" }), 0),
    }
}

type Registro = Arc<Mutex<Vec<(String, String, Value)>>>;

fn servidor(opciones: Opciones) -> String {
    servidor_con_registro(opciones).0
}

fn servidor_con_registro(opciones: Opciones) -> (String, Registro) {
    let registro: Registro = Arc::default();
    let anotar = registro.clone();
    let mock = Arc::new(mock());
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", escucha.local_addr().unwrap());
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
            let mock = mock.clone();
            let anotar = anotar.clone();
            std::thread::spawn(move || {
                let mut lector = BufReader::new(conexion.try_clone().unwrap());
                let mut primera = String::new();
                if lector.read_line(&mut primera).is_err() {
                    return;
                }
                let mut largo = 0usize;
                loop {
                    let mut linea = String::new();
                    if lector.read_line(&mut linea).unwrap_or(0) == 0 || linea == "\r\n" {
                        break;
                    }
                    if let Some(valor) = linea.to_ascii_lowercase().strip_prefix("content-length:") {
                        largo = valor.trim().parse().unwrap_or(0);
                    }
                }
                let mut cuerpo = vec![0; largo];
                let _ = lector.read_exact(&mut cuerpo);
                let mut partes = primera.split_whitespace();
                let metodo = partes.next().unwrap_or_default().to_string();
                let ruta = partes.next().unwrap_or_default().to_string();
                let enviado: Value = serde_json::from_slice(&cuerpo).unwrap_or(Value::Null);
                let (estado, respuesta, demora) = responder(&mock, opciones, &metodo, &ruta, &enviado);
                anotar.lock().unwrap().push((metodo.clone(), ruta.clone(), enviado));
                std::thread::sleep(Duration::from_millis(demora));
                let (tipo, texto) = match respuesta.as_str() {
                    Some(pdf) if pdf.starts_with("%PDF") => ("application/pdf", pdf.to_string()),
                    _ => ("application/json", respuesta.to_string()),
                };
                let mut salida = conexion;
                let _ = write!(
                    salida,
                    "HTTP/1.1 {estado} OK\r\nContent-Type: {tipo}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}",
                    texto.len()
                );
            });
        }
    });
    (url, registro)
}

fn servicios(url: &str) -> Servicios {
    let mut servicios = comun::servicios();
    servicios.api = Cliente::new(url);
    servicios
}

fn global(ruta: Ruta) -> Global {
    let mut global = comun::global(ruta);
    global.negocio = Some(mock()["escenarios"]["afip"]["negocio"].clone());
    global
}

fn esperar<S>(harness: &mut Harness<'_, S>, milisegundos: u64) {
    for _ in 0..milisegundos / 25 {
        harness.step();
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn clic<S>(harness: &mut Harness<'_, S>, x: f32, y: f32) {
    let pos = Pos2::new(x, y);
    harness.event(Event::PointerMoved(pos));
    harness.step();
    harness.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Default::default() });
    harness.step();
    harness.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Default::default() });
    harness.step();
}

fn guardar<S>(harness: &mut Harness<'_, S>, nombre: &str) {
    harness.remove_cursor();
    harness.run_steps(4);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}

fn pleno(ui: &mut egui::Ui, contenido: impl FnOnce(&mut egui::Ui)) {
    let rect = ui.ctx().content_rect();
    let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    hijo.set_clip_rect(rect);
    egui::Frame::new().fill(ui::color::FONDO).show(&mut hijo, |ui| {
        ui.set_min_size(ui.available_size());
        contenido(ui);
    });
}

fn pantalla(nombre: &str, ruta: Ruta, opciones: Opciones, pasos: impl FnOnce(&mut Harness<'_, App>)) {
    let url = servidor(opciones);
    let app = App::de_prueba(global(ruta), servicios(&url));
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(|ui, app: &mut App| pleno(ui, |ui| app.mostrar(ui)), app);
    esperar(&mut harness, 600);
    pasos(&mut harness);
    guardar(&mut harness, nombre);
}

#[test]
fn facturas_cargando() {
    pantalla("61-facturas-cargando", Ruta::Facturas, Opciones { demora_facturas: 5000, ..Default::default() }, |_| {});
}

#[test]
fn facturas_lista() {
    pantalla("62-facturas-lista", Ruta::Facturas, Opciones::default(), |_| {});
}

#[test]
fn facturas_filtro_estado() {
    pantalla("63-facturas-filtro-estado", Ruta::Facturas, Opciones::default(), |h| {
        clic(h, 580.0, 129.0);
        esperar(h, 200);
    });
}

#[test]
fn historial_pagos() {
    pantalla("64-historial-pagos", Ruta::Facturas, Opciones::default(), |h| {
        clic(h, 129.0, 544.0);
        esperar(h, 200);
        h.remove_cursor();
    });
}

#[test]
fn agregar_pago() {
    pantalla("65-agregar-pago", Ruta::Facturas, Opciones::default(), |h| {
        clic(h, 286.0, 544.0);
        esperar(h, 200);
        h.remove_cursor();
    });
}

#[test]
fn factura_editar() {
    pantalla("66-factura-editar", Ruta::Facturas, Opciones::default(), |h| {
        clic(h, 230.0, 500.0);
        esperar(h, 200);
        h.remove_cursor();
    });
}

#[test]
fn factura_detalle() {
    pantalla("67-factura-detalle", Ruta::FacturaDetalle("f102".into()), Opciones::default(), |_| {});
}

#[test]
fn factura_detalle_remito() {
    pantalla("68-factura-detalle-remito", Ruta::FacturaDetalle("f101".into()), Opciones::default(), |_| {});
}

#[test]
fn facturas_vacio() {
    pantalla("69-facturas-vacio", Ruta::Facturas, Opciones { facturas_vacias: true, ..Default::default() }, |_| {});
}

#[test]
fn factura_form_nueva() {
    pantalla("70-factura-form-nueva", Ruta::Facturas, Opciones { facturas_vacias: true, ..Default::default() }, |h| {
        clic(h, 600.0, 317.0);
        esperar(h, 400);
        h.remove_cursor();
    });
}

struct Formulario {
    instalado: bool,
    global: Global,
    servicios: Servicios,
    formulario: FormularioFactura,
}

fn productos_carrito() -> Vec<ProductoInicial> {
    mock()["escenarios"]["carritoSembrado"]["screens"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| ProductoInicial {
            id: i["id"].as_i64().unwrap(),
            nombre: i["name"].as_str().unwrap().to_string(),
            precio: i["pricePerUnit"].as_f64().unwrap(),
            cantidad: i["quantity"].as_f64().unwrap(),
            subtotal: i["subtotal"].as_f64().unwrap(),
        })
        .collect()
}

fn formulario(nombre: &str, pasos: impl FnOnce(&mut Harness<'_, Formulario>)) {
    formulario_con(nombre, Opciones::default(), pasos);
}

fn formulario_con(nombre: &str, opciones: Opciones, pasos: impl FnOnce(&mut Harness<'_, Formulario>)) {
    let url = servidor(opciones);
    let mut formulario = FormularioFactura::default();
    formulario.abrir(productos_carrito());
    let mut global = global(Ruta::Carrito);
    if opciones.precios_sin_iva {
        global.negocio.as_mut().unwrap()["ivaIncluidoEnPrecios"] = json!(false);
    }
    let estado = Formulario { instalado: false, global, servicios: servicios(&url), formulario };
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(
        |ui, e: &mut Formulario| {
            if !e.instalado {
                ui::instalar(ui.ctx());
                e.instalado = true;
                ui.ctx().request_repaint();
                return;
            }
            pleno(ui, |ui| {
                let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ui.ctx().clone() };
                if e.formulario.abierto() {
                    e.formulario.mostrar(ui, &mut cx);
                }
            });
            ui::toast::pintar(ui.ctx());
        },
        estado,
    );
    esperar(&mut harness, 800);
    pasos(&mut harness);
    guardar(&mut harness, nombre);
}

#[test]
fn factura_form_desde_carrito() {
    formulario("56-factura-form-desde-carrito", |h| h.remove_cursor());
}

#[test]
fn factura_form_totales() {
    formulario("56b-factura-form-totales", |h| {
        h.event(Event::PointerMoved(Pos2::new(600.0, 400.0)));
        for _ in 0..20 {
            h.event(Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: egui::vec2(0.0, -200.0), phase: egui::TouchPhase::Move, modifiers: Default::default() });
            h.step();
        }
        esperar(h, 300);
        h.remove_cursor();
    });
}

#[test]
fn selector_cliente() {
    formulario("57-selector-cliente", |h| {
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        h.remove_cursor();
    });
}

#[test]
fn factura_form_con_cliente() {
    formulario("58-factura-form-con-cliente", |h| {
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        clic(h, 590.0, 310.0);
        esperar(h, 300);
        h.remove_cursor();
    });
}

#[test]
fn selector_producto() {
    formulario("59-selector-producto", |h| {
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        clic(h, 590.0, 310.0);
        esperar(h, 300);
        clic(h, 892.0, 637.0);
        esperar(h, 400);
        h.remove_cursor();
    });
}

#[test]
fn cantidad_producto() {
    formulario("60-cantidad-producto", |h| {
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        clic(h, 590.0, 310.0);
        esperar(h, 300);
        clic(h, 892.0, 637.0);
        esperar(h, 400);
        clic(h, 590.0, 302.0);
        esperar(h, 300);
        h.remove_cursor();
    });
}

#[test]
fn factura_form_remito_sin_iva() {
    formulario_con("56e-factura-form-remito-sin-iva", Opciones { precios_sin_iva: true, ..Default::default() }, |h| h.remove_cursor());
}

#[test]
fn factura_form_a_con_iva() {
    formulario_con("56f-factura-form-a-con-iva", Opciones { precios_sin_iva: true, ..Default::default() }, |h| {
        clic(h, 803.0, 155.0);
        esperar(h, 300);
        h.remove_cursor();
    });
}

fn bajar_formulario(h: &mut Harness<'_, Formulario>) {
    h.event(Event::PointerMoved(Pos2::new(600.0, 400.0)));
    for _ in 0..20 {
        h.event(Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: egui::vec2(0.0, -200.0), phase: egui::TouchPhase::Move, modifiers: Default::default() });
        h.step();
    }
    esperar(h, 300);
    h.remove_cursor();
}

#[test]
fn factura_form_remito_sin_iva_totales() {
    formulario_con("56g-factura-form-remito-sin-iva-totales", Opciones { precios_sin_iva: true, ..Default::default() }, bajar_formulario);
}

#[test]
fn factura_form_a_con_iva_totales() {
    formulario_con("56h-factura-form-a-con-iva-totales", Opciones { precios_sin_iva: true, ..Default::default() }, |h| {
        clic(h, 803.0, 155.0);
        esperar(h, 300);
        bajar_formulario(h);
    });
}

fn enviados(registro: &Registro, metodo: &str, ruta: &str) -> Vec<Value> {
    registro.lock().unwrap().iter().filter(|(m, r, _)| m == metodo && r == ruta).map(|(_, _, c)| c.clone()).collect()
}

fn formulario_registrado(pasos: impl FnOnce(&mut Harness<'_, Formulario>)) -> (Registro, bool) {
    let (url, registro) = servidor_con_registro(Opciones::default());
    let servicios = servicios(&url);
    servicios.config.set("impresoraWindows", json!("impresora-inexistente-de-prueba"));
    let mut formulario = FormularioFactura::default();
    formulario.abrir(productos_carrito());
    let estado = Formulario { instalado: false, global: global(Ruta::Carrito), servicios, formulario };
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(
        |ui, e: &mut Formulario| {
            if !e.instalado {
                ui::instalar(ui.ctx());
                e.instalado = true;
                ui.ctx().request_repaint();
                return;
            }
            pleno(ui, |ui| {
                let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ui.ctx().clone() };
                if e.formulario.abierto() {
                    e.formulario.mostrar(ui, &mut cx);
                }
            });
        },
        estado,
    );
    esperar(&mut harness, 800);
    pasos(&mut harness);
    let abierto = harness.state().formulario.abierto();
    (registro, abierto)
}

fn bajar_y_crear(h: &mut Harness<'_, Formulario>) {
    h.event(Event::PointerMoved(Pos2::new(600.0, 400.0)));
    for _ in 0..20 {
        h.event(Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: egui::vec2(0.0, -200.0), phase: egui::TouchPhase::Move, modifiers: Default::default() });
        h.step();
    }
    esperar(h, 300);
    clic(h, 870.0, 735.0);
    esperar(h, 1500);
}

#[test]
fn envio_de_remito_desde_carrito() {
    let (registro, abierto) = formulario_registrado(|h| {
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        clic(h, 590.0, 310.0);
        esperar(h, 300);
        bajar_y_crear(h);
    });
    let cuerpos = enviados(&registro, "POST", "/api/facturas");
    assert_eq!(cuerpos.len(), 1, "{:?}", registro.lock().unwrap());
    let cuerpo = &cuerpos[0];
    assert_eq!(cuerpo["clienteId"], "c1");
    assert_eq!(cuerpo["tipoFactura"], "remito");
    assert_eq!(cuerpo["pagoInicial"], 0.0);
    assert_eq!(cuerpo["total"], 22449.5);
    assert_eq!(cuerpo["impuestos"], 0.0);
    assert_eq!(cuerpo["esCuentaCorriente"], true);
    assert!(cuerpo.get("listaPrecioId").is_none());
    assert_eq!(cuerpo["detalles"].as_array().unwrap().len(), 7);
    assert!(!abierto);
}

#[test]
fn lista_de_precios_actualiza_los_precios() {
    let (registro, abierto) = formulario_registrado(|h| {
        clic(h, 440.0, 237.0);
        esperar(h, 200);
        clic(h, 440.0, 319.0);
        esperar(h, 800);
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        clic(h, 590.0, 310.0);
        esperar(h, 300);
        bajar_y_crear(h);
    });
    let cuerpos = enviados(&registro, "POST", "/api/facturas");
    assert_eq!(cuerpos.len(), 1, "{:?}", registro.lock().unwrap());
    let cuerpo = &cuerpos[0];
    assert_eq!(cuerpo["listaPrecioId"], 1);
    let precios: Vec<(i64, f64)> = cuerpo["detalles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["productoId"].as_i64().unwrap(), d["precioUnitario"].as_f64().unwrap()))
        .collect();
    assert_eq!(precios, vec![(1, 2100.0), (2, 780.0), (5, 2450.0), (21, 5900.0), (19, 1500.0), (12, 4500.0), (23, 800.0)]);
    assert!(!enviados(&registro, "GET", "/api/listas-precios/1/productos").is_empty());
    assert!(!abierto);
}

#[test]
fn agregar_pago_envia_el_pago_y_recarga() {
    let (url, registro) = servidor_con_registro(Opciones::default());
    let app = App::de_prueba(global(Ruta::Facturas), servicios(&url));
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(|ui, app: &mut App| pleno(ui, |ui| app.mostrar(ui)), app);
    esperar(&mut harness, 600);
    clic(&mut harness, 286.0, 544.0);
    esperar(&mut harness, 200);
    clic(&mut harness, 600.0, 287.0);
    harness.event(Event::Text("1000".into()));
    harness.step();
    harness.key_press(egui::Key::Enter);
    esperar(&mut harness, 800);
    let pagos = enviados(&registro, "POST", "/api/facturas/agregar-pago");
    assert_eq!(pagos.len(), 1, "{:?}", registro.lock().unwrap());
    assert_eq!(pagos[0]["facturaId"], "f101");
    assert_eq!(pagos[0]["monto"], 1000.0);
    assert_eq!(pagos[0]["metodoPago"], "efectivo");
    assert_eq!(pagos[0]["fecha"], nucleo::facturas::hoy_argentina().format("%Y-%m-%d").to_string());
    assert!(pagos[0].get("referencia").is_none());
    let listas = registro.lock().unwrap().iter().filter(|(m, r, _)| m == "GET" && r.starts_with("/api/facturas?")).count();
    assert!(listas >= 2, "la lista no se recargó");
}

fn lista_registrada(registro_url: (String, Registro)) -> (Harness<'static, App>, Registro) {
    let (url, registro) = registro_url;
    let app = App::de_prueba(global(Ruta::Facturas), servicios(&url));
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(|ui, app: &mut App| pleno(ui, |ui| app.mostrar(ui)), app);
    esperar(&mut harness, 600);
    (harness, registro)
}

fn pedidos_de_lista(registro: &Registro) -> Vec<String> {
    registro.lock().unwrap().iter().filter(|(m, r, _)| m == "GET" && r.starts_with("/api/facturas?")).map(|(_, r, _)| r.clone()).collect()
}

#[test]
fn guardar_en_el_modal_hace_put() {
    let (mut h, registro) = lista_registrada(servidor_con_registro(Opciones::default()));
    clic(&mut h, 230.0, 500.0);
    esperar(&mut h, 200);
    clic(&mut h, 806.0, 239.0);
    esperar(&mut h, 200);
    guardar(&mut h, "66b-factura-editar-estado");
    clic(&mut h, 806.0, 311.0);
    esperar(&mut h, 200);
    clic(&mut h, 954.0, 87.0);
    esperar(&mut h, 800);
    let cuerpos = enviados(&registro, "PUT", "/api/facturas/f101");
    assert_eq!(cuerpos, vec![json!({ "estado": "pagada" })], "{:?}", registro.lock().unwrap());
    assert!(pedidos_de_lista(&registro).len() >= 2, "la lista no se recargó");
    guardar(&mut h, "66c-factura-guardada");
}

const BOTON_ANULAR: (f32, f32) = (736.0, 455.0);

#[test]
fn eliminar_anula_con_confirmacion() {
    let (mut h, registro) = lista_registrada(servidor_con_registro(Opciones::default()));
    clic(&mut h, 346.0, 500.0);
    esperar(&mut h, 200);
    assert!(enviados(&registro, "DELETE", "/api/facturas/f101").is_empty());
    guardar(&mut h, "71-facturas-eliminar");
    h.key_press(egui::Key::Escape);
    esperar(&mut h, 200);
    clic(&mut h, 346.0, 500.0);
    esperar(&mut h, 200);
    clic(&mut h, BOTON_ANULAR.0, BOTON_ANULAR.1);
    esperar(&mut h, 800);
    let cuerpos = enviados(&registro, "DELETE", "/api/facturas/f101");
    assert_eq!(cuerpos, vec![json!({ "forceDelete": false })], "{:?}", registro.lock().unwrap());
    assert!(pedidos_de_lista(&registro).len() >= 2, "la lista no se recargó");
}

#[test]
fn limpiar_filtros_borra_la_busqueda() {
    let (mut h, registro) = lista_registrada(servidor_con_registro(Opciones::default()));
    clic(&mut h, 250.0, 129.0);
    h.event(Event::Text("zzz".into()));
    esperar(&mut h, 800);
    assert!(pedidos_de_lista(&registro).last().unwrap().contains("search=zzz"));
    clic(&mut h, 600.0, 317.0);
    esperar(&mut h, 800);
    let ultimo = pedidos_de_lista(&registro).last().unwrap().clone();
    assert!(!ultimo.contains("search="), "{ultimo}");
    guardar(&mut h, "69b-facturas-filtros-limpios");
}

#[test]
fn pdf_se_guarda_en_la_carpeta_indicada() {
    let (url, registro) = servidor_con_registro(Opciones::default());
    let carpeta = std::env::temp_dir().join(format!("andexmarket-pdf-{}", std::process::id()));
    let factura = json!({ "id": "f102", "tipoFactura": "A", "numero": "A-0002-00000013" });
    let ruta = nucleo::facturas::descargar_pdf(&Cliente::new(&url), &factura, &carpeta).unwrap();
    assert_eq!(ruta, carpeta.join("factura-A-A-0002-00000013.pdf"));
    assert_eq!(std::fs::read_to_string(&ruta).unwrap(), PDF_FALSO);
    assert!(registro.lock().unwrap().iter().any(|(m, r, _)| m == "GET" && r == "/api/facturas/generar-pdf?id=f102"));
    let _ = std::fs::remove_dir_all(carpeta);
}

fn productos_pedidos(registro: &Registro) -> usize {
    registro.lock().unwrap().iter().filter(|(m, r, _)| m == "GET" && r.starts_with("/api/productos/")).count()
}

#[test]
fn borrar_todas_las_filas_no_recarga_el_carrito() {
    let (registro, abierto) = formulario_registrado(|h| {
        for _ in 0..7 {
            clic(h, 925.0, 364.0);
            esperar(h, 100);
        }
        esperar(h, 800);
        guardar(h, "56c-factura-form-sin-filas");
    });
    assert_eq!(productos_pedidos(&registro), 7, "{:?}", registro.lock().unwrap());
    assert!(abierto);
}

#[test]
fn reabrir_con_lista_mantiene_desplegable_y_envio() {
    let (registro, abierto) = formulario_registrado(|h| {
        clic(h, 440.0, 237.0);
        esperar(h, 200);
        clic(h, 440.0, 319.0);
        esperar(h, 800);
        h.key_press(egui::Key::Escape);
        esperar(h, 200);
        assert!(!h.state().formulario.abierto());
        h.state_mut().formulario.abrir(productos_carrito());
        esperar(h, 1200);
        guardar(h, "56d-factura-form-reabierto-con-lista");
        clic(h, 440.0, 155.0);
        esperar(h, 400);
        clic(h, 590.0, 310.0);
        esperar(h, 300);
        bajar_y_crear(h);
    });
    let cuerpos = enviados(&registro, "POST", "/api/facturas");
    assert_eq!(cuerpos.len(), 1, "{:?}", registro.lock().unwrap());
    assert_eq!(cuerpos[0]["listaPrecioId"], 1);
    let precios: Vec<(i64, f64)> = cuerpos[0]["detalles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["productoId"].as_i64().unwrap(), d["precioUnitario"].as_f64().unwrap()))
        .collect();
    assert_eq!(precios, vec![(1, 2100.0), (2, 780.0), (5, 2450.0), (21, 5900.0), (19, 1500.0), (12, 4500.0), (23, 800.0)]);
    assert!(!abierto);
}

#[test]
fn detalle_ver_cliente_y_agregar_pago() {
    let (url, registro) = servidor_con_registro(Opciones::default());
    let app = App::de_prueba(global(Ruta::FacturaDetalle("f101".into())), servicios(&url));
    let mut h = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(|ui, app: &mut App| pleno(ui, |ui| app.mostrar(ui)), app);
    esperar(&mut h, 600);
    clic(&mut h, 980.0, 320.0);
    esperar(&mut h, 200);
    guardar(&mut h, "68b-factura-detalle-cliente");
    h.key_press(egui::Key::Escape);
    esperar(&mut h, 200);
    clic(&mut h, 1086.0, 52.0);
    esperar(&mut h, 200);
    guardar(&mut h, "68c-factura-detalle-agregar-pago");
    clic(&mut h, 600.0, 287.0);
    h.event(Event::Text("1500,50".into()));
    h.step();
    h.key_press(egui::Key::Enter);
    esperar(&mut h, 800);
    let pagos = enviados(&registro, "POST", "/api/facturas/agregar-pago");
    assert_eq!(pagos.len(), 1, "{:?}", registro.lock().unwrap());
    assert_eq!(pagos[0]["facturaId"], "f101");
    assert_eq!(pagos[0]["monto"], 1500.5);
    assert_eq!(enviados(&registro, "GET", "/api/facturas/f101").len(), 2, "el detalle no se recargó");
}
