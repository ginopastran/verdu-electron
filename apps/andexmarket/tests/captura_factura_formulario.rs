mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use andexmarket::app::{Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::facturas::formulario::{FormularioFactura, ProductoInicial};
use andexmarket::ui;
use eframe::egui::{Event, PointerButton, Pos2};
use egui_kittest::Harness;
use nucleo::api::Cliente;
use serde_json::{json, Value};

fn mock() -> Value {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidatos = [base.join("../../docs/referencia/mock.json"), base.join("../../../verdu-rust/docs/referencia/mock.json")];
    let ruta = candidatos.iter().find(|r| r.exists()).expect("mock.json de las capturas de referencia");
    serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap()
}

fn responder(mock: &Value, ruta: &str) -> (u16, Value) {
    let camino = ruta.split_once('?').map_or(ruta, |(c, _)| c);
    let partes: Vec<&str> = camino.trim_start_matches('/').split('/').collect();
    let datos = &mock["datos"];
    let ejemplo = |ruta: &str| {
        mock["endpoints"].as_array().unwrap().iter().find(|e| e["metodo"] == "GET" && e["ruta"] == ruta).map_or(Value::Null, |e| e["ejemplos"][0]["respuesta"].clone())
    };
    match partes.as_slice() {
        ["api", "clientes"] => (200, ejemplo("/api/clientes")),
        ["api", "productos"] => (200, json!({ "productos": datos["productos"] })),
        ["api", "productos", id] => match datos["productos"].as_array().unwrap().iter().find(|p| p["id"].to_string() == *id) {
            Some(p) => (200, p.clone()),
            None => (404, json!({})),
        },
        ["api", "listas-precios"] => (200, json!({ "listasPrecios": datos["listasPrecios"] })),
        _ => (404, json!({ "error": "sin mock" })),
    }
}

fn servidor() -> String {
    let mock = Arc::new(mock());
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", escucha.local_addr().unwrap());
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
            let mock = mock.clone();
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
                let ruta = primera.split_whitespace().nth(1).unwrap_or_default().to_string();
                let (estado, respuesta) = responder(&mock, &ruta);
                let texto = respuesta.to_string();
                let mut salida = conexion;
                let _ = write!(salida, "HTTP/1.1 {estado} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}", texto.len());
            });
        }
    });
    url
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

fn esperar(h: &mut Harness<'_, Formulario>, milisegundos: u64) {
    for _ in 0..milisegundos / 25 {
        h.step();
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn clic(h: &mut Harness<'_, Formulario>, x: f32, y: f32) {
    let pos = Pos2::new(x, y);
    h.event(Event::PointerMoved(pos));
    h.step();
    h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Default::default() });
    h.step();
    h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Default::default() });
    h.step();
}

fn centro(h: &Harness<'_, Formulario>) -> (f32, f32) {
    let c = h.ctx.content_rect().center();
    (c.x, c.y)
}

fn escena(nombre: &str, iniciales: bool, pasos: impl Fn(&mut Harness<'_, Formulario>)) {
    for tam in [[1920.0, 1080.0], [1366.0, 768.0]] {
        let url = servidor();
        let mut servicios = comun::servicios();
        servicios.api = Cliente::new(&url);
        let mut global = comun::global(Ruta::Carrito);
        global.negocio = Some(mock()["escenarios"]["afip"]["negocio"].clone());
        let mut formulario = FormularioFactura::default();
        formulario.abrir(if iniciales { productos_carrito() } else { Vec::new() });
        let estado = Formulario { instalado: false, global, servicios, formulario };
        let mut h = Harness::builder().with_size(tam).wgpu().build_ui_state(
            |ui, e: &mut Formulario| {
                if !e.instalado {
                    ui::instalar(ui.ctx());
                    e.instalado = true;
                    ui.ctx().request_repaint();
                    return;
                }
                ui.painter().rect_filled(ui.ctx().content_rect(), 0, ui::marca::color::FONDO_APP);
                let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ui.ctx().clone() };
                if e.formulario.abierto() {
                    e.formulario.mostrar(ui, &mut cx);
                }
                ui::v2::toast::pintar(ui.ctx());
            },
            estado,
        );
        esperar(&mut h, 800);
        pasos(&mut h);
        h.remove_cursor();
        h.run_steps(20);
        let imagen = h.render().expect("render");
        imagen.save(comun::carpeta_capturas().join(format!("{nombre}-{}.png", tam[0] as u32))).expect("guardar captura");
    }
}

#[test]
fn cuenta_corriente_desde_orden() {
    escena("65-modal-cuenta-corriente-desde-orden", true, |_| {});
}

#[test]
fn factura_nueva_clientes() {
    escena("66-modal-factura-nueva-clientes", false, |h| {
        let (x, y) = centro(h);
        clic(h, x - 150.0, y + CLIENTE_Y);
        esperar(h, 400);
    });
}

#[test]
fn factura_nueva_productos() {
    escena("67-modal-factura-nueva-productos", false, |h| {
        let (x, y) = centro(h);
        clic(h, x - 60.0, y + AGREGAR_Y);
        esperar(h, 400);
    });
}

#[test]
fn factura_error_sin_cliente() {
    escena("69-modal-factura-error", false, |h| {
        let (x, y) = centro(h);
        clic(h, x + 270.0, y + CREAR_Y);
        esperar(h, 300);
    });
}

const CLIENTE_Y: f32 = -198.0;
const AGREGAR_Y: f32 = 67.0;
const CREAR_Y: f32 = 288.0;
