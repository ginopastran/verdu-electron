mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use andexmarket::app::{Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::cobro::{AccionCobro, Cobro};
use andexmarket::ui;
use eframe::egui::{self, Key};
use egui_kittest::Harness;
use nucleo::api::Cliente;
use nucleo::carrito::{Carritos, Producto};
use serde_json::{json, Value};

type Registro = Arc<Mutex<Vec<(String, Value)>>>;

fn pedidos(registro: &Registro, ruta: &str) -> Vec<Value> {
    registro.lock().unwrap().iter().filter(|(r, _)| r == ruta).map(|(_, cuerpo)| cuerpo.clone()).collect()
}

struct Estado {
    global: Global,
    servicios: Servicios,
    cobro: Cobro,
    instalado: bool,
}

fn qr_falso() -> String {
    let n = 29i32;
    let mut semilla = 7.0f64;
    let mut azar = move || {
        semilla = (semilla * 1103515245.0 + 12345.0) % 2147483648.0;
        semilla / 2147483648.0
    };
    let finder = |x: i32, y: i32, i: i32, j: i32| -> Option<bool> {
        let (dx, dy) = (i - x, j - y);
        if !(0..=6).contains(&dx) || !(0..=6).contains(&dy) {
            return None;
        }
        Some(dx == 0 || dy == 0 || dx == 6 || dy == 6 || ((2..=4).contains(&dx) && (2..=4).contains(&dy)))
    };
    let mut rects = String::new();
    for i in 0..n {
        for j in 0..n {
            let f = finder(0, 0, i, j).or(finder(n - 7, 0, i, j)).or(finder(0, n - 7, i, j));
            let separador = (i <= 7 && j <= 7) || (i >= n - 8 && j <= 7) || (i <= 7 && j >= n - 8);
            let negro = f.unwrap_or_else(|| if separador { false } else { azar() > 0.5 });
            if negro {
                rects.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"1\" height=\"1\"/>", i + 2, j + 2));
            }
        }
    }
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {0} {0}\" shape-rendering=\"crispEdges\"><rect width=\"100%\" height=\"100%\" fill=\"#fff\"/><g fill=\"#000\">{1}</g></svg>",
        n + 4,
        rects
    );
    format!("data:image/svg+xml;base64,{}", base64(svg.as_bytes()))
}

fn base64(bytes: &[u8]) -> String {
    const TABLA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut salida = String::new();
    for bloque in bytes.chunks(3) {
        let b = [bloque[0], *bloque.get(1).unwrap_or(&0), *bloque.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= bloque.len() {
                salida.push(TABLA[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                salida.push('=');
            }
        }
    }
    salida
}

fn responder(metodo: &str, ruta: &str, cuerpo: &Value, acreditado: bool) -> (u16, Value) {
    match (metodo, ruta) {
        ("POST", "/api/mercadopago/generate-qr") => (
            200,
            json!({
                "id": 9023,
                "orderId": 9023,
                "qrData": "00020101021243650016COM.MERCADOLIBRE0201306364fake-qr-lahuerta-9023",
                "qrImageUrl": qr_falso(),
                "status": "PENDING",
                "monto": cuerpo["monto"],
                "isSplitPayment": cuerpo["isSplitPayment"] == true,
                "cashAmount": if cuerpo["cashAmount"].is_null() { json!(0) } else { cuerpo["cashAmount"].clone() },
            }),
        ),
        ("GET", r) if r.starts_with("/api/mercadopago/check-status") => (200, json!({ "status": if acreditado { "APPROVED" } else { "PENDING" } })),
        ("POST", "/api/ordenes") => (200, json!({ "id": 9022, "idReal": 9022, "estado": "COMPLETADA" })),
        ("POST", "/api/facturas/crear-afip") => (
            201,
            json!({
                "message": "Factura AFIP creada correctamente",
                "factura": { "id": 88, "numero": "0003-00001861", "numeroFactura": "1861", "puntoVenta": "0003", "tipoFactura": "B" },
                "afip": { "cae": "76411664625912", "vencimientoCae": "20261020" },
            }),
        ),
        _ => (404, json!({ "error": "No encontrado" })),
    }
}

fn servidor(registro: Registro, acreditado: Arc<AtomicBool>) -> String {
    let escucha = TcpListener::bind("127.0.0.1:0").expect("puerto");
    let puerto = escucha.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
            let mut lector = BufReader::new(conexion.try_clone().unwrap());
            let mut linea = String::new();
            if lector.read_line(&mut linea).is_err() {
                continue;
            }
            let partes: Vec<&str> = linea.split_whitespace().collect();
            let (metodo, ruta) = (partes.first().copied().unwrap_or(""), partes.get(1).copied().unwrap_or(""));
            let mut largo = 0usize;
            loop {
                let mut cabecera = String::new();
                if lector.read_line(&mut cabecera).is_err() || cabecera.trim().is_empty() {
                    break;
                }
                if let Some(valor) = cabecera.to_ascii_lowercase().strip_prefix("content-length:") {
                    largo = valor.trim().parse().unwrap_or(0);
                }
            }
            let mut cuerpo = vec![0; largo];
            let _ = lector.read_exact(&mut cuerpo);
            let cuerpo: Value = serde_json::from_slice(&cuerpo).unwrap_or(Value::Null);
            registro.lock().unwrap().push((ruta.to_string(), cuerpo.clone()));
            let (estado, respuesta) = responder(metodo, ruta, &cuerpo, acreditado.load(Ordering::SeqCst));
            let texto = respuesta.to_string();
            let mut conexion = conexion;
            let _ = write!(
                conexion,
                "HTTP/1.1 {estado} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}",
                texto.len()
            );
        }
    });
    format!("http://127.0.0.1:{puerto}")
}

fn producto(id: i64, nombre: &str, cantidad: f64, precio: f64, unidad: &str, iva: f64) -> Producto {
    Producto {
        id,
        cart_id: format!("{id}-seed-{cantidad}"),
        name: nombre.into(),
        quantity: cantidad,
        unit: unidad.into(),
        price_per_unit: precio,
        subtotal: (precio * cantidad * 100.0).round() / 100.0,
        costo: (precio * 0.62).round(),
        iva_incluido: Some(true),
        iva_porcentaje: Some(iva),
        lista_precio_id: None,
        lista_precio_nombre: Some("Precio catálogo".into()),
    }
}

fn carrito() -> Carritos {
    let mut carritos = Carritos::default();
    for item in [
        producto(1, "Tomate redondo", 1.245, 2890.0, "Kg", 10.5),
        producto(2, "Papa negra", 2.1, 1150.0, "Kg", 10.5),
        producto(3, "Lechuga criolla", 1.0, 1900.0, "Unidad", 10.5),
        producto(4, "Banana Ecuador", 0.85, 2110.0, "Kg", 10.5),
        producto(5, "Huevos blancos x12", 1.0, 3300.0, "Unidad", 21.0),
        producto(6, "Palta Hass", 2.0, 2400.0, "Unidad", 10.5),
    ] {
        carritos.agregar(item);
    }
    carritos
}

fn negocio(afip: bool) -> Value {
    json!({
        "id": 7,
        "nombre": "Verdulería Don José",
        "sistemaPago": if afip { "pago-exacto" } else { "redondeo" },
        "descuentoEfectivo": 10,
        "mpEnabled": true,
        "afipHabilitado": afip,
        "facturacionHabilitada": afip,
        "dobleImpresionEnabled": false,
        "cancelacionHabilitada": afip,
        "ivaIncluidoEnPrecios": true,
        "cuit": "30-71234567-8",
        "configuracionAfip": {
            "cuit": "30-71234567-8",
            "puntoVenta": 2,
            "condicionIva": "Responsable Inscripto",
            "razonSocial": "Don José S.R.L.",
            "tipoFactura": "B",
        },
    })
}

const TAMANOS: [[f32; 2]; 2] = [[1920.0, 1080.0], [1366.0, 768.0]];

struct Escena {
    h: Harness<'static, Estado>,
    registro: Registro,
    acreditado: Arc<AtomicBool>,
    ancho: u32,
}

fn harness(afip: bool, tam: [f32; 2]) -> Escena {
    let registro = Registro::default();
    let acreditado = Arc::new(AtomicBool::new(false));
    let mut global = comun::global(Ruta::Carrito);
    global.negocio = Some(negocio(afip));
    global.admin = Some(json!({ "id": 1, "nombre": "Martín Álvarez", "businessId": 7, "businessName": "Verdulería Don José" }));
    global.usuario = Some(json!({ "id": 11, "nombre": "Lucía Fernández", "email": "lucia@donjose.com.ar", "sucursalId": 1, "role": "VENDEDOR" }));
    global.carritos = carrito();
    let mut servicios = comun::servicios();
    servicios.api = Cliente::new(&servidor(registro.clone(), acreditado.clone()));
    servicios.config.set("impresoraWindows", json!("Impresora inexistente de prueba"));
    let estado = Estado { global, servicios, cobro: Cobro::default(), instalado: false };
    let mut h = Harness::builder()
        .with_size(tam)
        .with_step_dt(1.0 / 60.0)
        .wgpu()
        .build_ui_state(
            |ui, e: &mut Estado| {
                if !e.instalado {
                    ui::instalar(ui.ctx());
                    e.instalado = true;
                    ui.ctx().request_repaint();
                    return;
                }
                ui.painter().rect_filled(ui.ctx().content_rect(), 0, ui::marca::color::FONDO_APP);
                let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ui.ctx().clone() };
                e.cobro.mostrar(ui, &mut cx);
                ui::v2::toast::pintar(ui.ctx());
            },
            estado,
        );
    h.run_steps(3);
    Escena { h, registro, acreditado, ancho: tam[0] as u32 }
}

fn en_ambos(afip: bool, mut pasos: impl FnMut(&mut Escena)) {
    for tam in TAMANOS {
        let mut escena = harness(afip, tam);
        pasos(&mut escena);
    }
}

impl Escena {
    fn iniciar(&mut self, accion: AccionCobro) {
        let ctx = self.h.ctx.clone();
        let e = self.h.state_mut();
        let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ctx };
        e.cobro.iniciar(accion, &mut cx);
        self.h.run_steps(3);
    }

    fn tecla(&mut self, tecla: Key) {
        self.h.key_press(tecla);
        self.h.run_steps(3);
    }

    fn escribir(&mut self, texto: &str) {
        self.h.event(egui::Event::Text(texto.into()));
        self.h.run_steps(3);
    }

    fn clic(&mut self, x: f32, y: f32) {
        let pos = egui::Pos2::new(x, y);
        self.h.event(egui::Event::PointerMoved(pos));
        self.h.run_steps(1);
        let boton = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        self.h.event(boton(true));
        self.h.run_steps(1);
        self.h.event(boton(false));
        self.h.run_steps(3);
        self.h.event(egui::Event::PointerGone);
        self.h.run_steps(1);
    }

    fn esperar(&mut self, mut listo: impl FnMut(&mut Escena) -> bool) {
        for _ in 0..600 {
            if listo(self) {
                return;
            }
            self.h.run_steps(1);
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("tiempo de espera agotado");
    }

    fn pausa(&mut self, milisegundos: u64) {
        for _ in 0..milisegundos / 25 {
            self.h.run_steps(1);
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    fn pedidos(&self, ruta: &str) -> Vec<Value> {
        pedidos(&self.registro, ruta)
    }

    fn centro(&self) -> (f32, f32) {
        let c = self.h.ctx.content_rect().center();
        (c.x, c.y)
    }

    fn capturar(&mut self, nombre: &str) {
        self.h.run_steps(20);
        let imagen = self.h.render().expect("render");
        imagen.save(comun::carpeta_capturas().join(format!("{nombre}-{}.png", self.ancho))).expect("guardar captura");
    }
}

#[test]
fn metodo_de_pago() {
    en_ambos(true, |e| {
        e.iniciar(AccionCobro::PagarAfip);
        e.capturar("36-modal-metodo-pago-factura");
        e.tecla(Key::Escape);
        e.iniciar(AccionCobro::Pagar);
        e.capturar("37-modal-metodo-pago-sin-factura");
        e.tecla(Key::Escape);
        e.iniciar(AccionCobro::PagarAfipConDescuento);
        e.escribir("10");
        e.tecla(Key::Enter);
        e.capturar("38-modal-metodo-pago-con-descuento");
    });
    en_ambos(false, |e| {
        e.iniciar(AccionCobro::Pagar);
        e.h.key_press(Key::Num2);
        e.h.run_steps(2);
        e.capturar("39-modal-metodo-pago-procesando");
        e.esperar(|e| !e.h.state().cobro.ocupado());
        e.capturar("48-modal-venta-completada-imprimiendo");
        assert!(e.h.state().global.carritos.items().is_empty());
    });
}

#[test]
fn efectivo() {
    en_ambos(false, |e| {
        e.iniciar(AccionCobro::Pagar);
        e.tecla(Key::Num3);
        e.capturar("40-modal-efectivo");
        e.escribir("20000");
        e.capturar("41-modal-efectivo-vuelto");
        e.tecla(Key::Escape);
        e.iniciar(AccionCobro::Pagar);
        e.tecla(Key::Num3);
        e.tecla(Key::F5);
        e.capturar("42-modal-efectivo-descuento");
    });
}

#[test]
fn mixto() {
    en_ambos(false, |e| {
        e.iniciar(AccionCobro::Pagar);
        e.tecla(Key::Num4);
        e.tecla(Key::Enter);
        e.capturar("44-modal-mixto-error");
        e.escribir("5000");
        e.capturar("43-modal-mixto");
        e.tecla(Key::Enter);
        e.esperar(|e| !e.h.state().cobro.ocupado());
        e.capturar("49-modal-venta-completada-mixto");
        assert!(e.h.state().global.carritos.items().is_empty());
    });
}

#[test]
fn qr() {
    en_ambos(false, |e| {
        e.iniciar(AccionCobro::Pagar);
        e.tecla(Key::Num1);
        e.esperar(|e| !e.pedidos("/api/mercadopago/generate-qr").is_empty());
        e.pausa(600);
        e.capturar("45-modal-qr-esperando");
        e.acreditado.store(true, Ordering::SeqCst);
        e.pausa(3600);
        e.capturar("46-modal-qr-acreditado");
    });
}

#[test]
fn venta_completada_con_factura() {
    en_ambos(true, |e| {
        e.iniciar(AccionCobro::PagarAfip);
        e.tecla(Key::Num3);
        e.escribir("20000");
        e.tecla(Key::Enter);
        e.esperar(|e| !e.pedidos("/api/facturas/crear-afip").is_empty());
        e.esperar(|e| !e.h.state().cobro.ocupado());
        e.capturar("47-modal-venta-completada-factura");
        assert!(e.h.state().global.carritos.items().is_empty());
    });
}

#[test]
fn pago_mixto_qr_con_descuento() {
    let mut e = harness(false, TAMANOS[1]);
    e.iniciar(AccionCobro::PagarConDescuento);
    e.escribir("10");
    e.tecla(Key::Enter);
    e.tecla(Key::Num4);
    e.escribir("5000");
    let (x, y) = e.centro();
    e.clic(x + 112.0, y + 89.0);
    e.tecla(Key::Enter);
    e.esperar(|e| !e.pedidos("/api/mercadopago/generate-qr").is_empty());
    let cuerpo = &e.pedidos("/api/mercadopago/generate-qr")[0];
    let total = cuerpo["subtotalSinDescuento"].as_f64().unwrap() * 0.9;
    assert!((cuerpo["monto"].as_f64().unwrap() - total).abs() < 0.01, "{cuerpo}");
    assert_eq!(cuerpo["cashAmount"], 5000.0);
    assert_eq!(cuerpo["isSplitPayment"], true);
}

#[test]
fn qr_manual_con_clave_incorrecta_no_completa() {
    let mut e = harness(false, TAMANOS[1]);
    e.iniciar(AccionCobro::Pagar);
    e.tecla(Key::Num1);
    e.esperar(|e| !e.pedidos("/api/mercadopago/generate-qr").is_empty());
    e.pausa(300);
    e.tecla(Key::Enter);
    e.escribir("clave-que-no-es");
    e.tecla(Key::Enter);
    e.pausa(300);
    assert!(e.pedidos("/api/mercadopago/manual-complete").is_empty());
}

#[test]
fn afip_efectivo_manda_el_descuento_sin_aplicar_d() {
    let mut e = harness(true, TAMANOS[1]);
    e.h.state_mut().global.negocio.as_mut().unwrap()["sistemaPago"] = json!("redondeo");
    e.iniciar(AccionCobro::PagarAfipConDescuento);
    e.escribir("5");
    e.tecla(Key::Enter);
    e.tecla(Key::Num3);
    e.tecla(Key::Enter);
    e.esperar(|e| !e.pedidos("/api/facturas/crear-afip").is_empty());
    let cuerpo = &e.pedidos("/api/facturas/crear-afip")[0];
    assert_eq!(cuerpo["tieneDescuento"], true, "{cuerpo}");
    assert_eq!(cuerpo["tipoDescuento"], "porcentual");
    assert_eq!(cuerpo["valorDescuento"], 5.0);
    assert!(cuerpo["montoDescuento"].as_f64().unwrap() > 0.0);
    e.esperar(|e| !e.h.state().cobro.ocupado());
}

#[test]
fn descuento_con_tarjeta_vacia_el_carrito() {
    let mut e = harness(false, TAMANOS[1]);
    e.iniciar(AccionCobro::PagarConDescuento);
    e.escribir("10");
    e.tecla(Key::Enter);
    e.tecla(Key::Num2);
    e.esperar(|e| !e.h.state().cobro.ocupado());
    assert!(e.h.state().global.carritos.items().is_empty());
}

#[test]
fn cancelar_y_descuento() {
    en_ambos(false, |e| {
        e.iniciar(AccionCobro::Cancelar);
        e.capturar("23-modal-cancelar-orden");
        e.tecla(Key::Escape);
        e.iniciar(AccionCobro::PagarAfipConDescuento);
        e.capturar("33-modal-descuento");
        e.escribir("10");
        e.capturar("34-modal-descuento-porcentaje");
        e.tecla(Key::Escape);
        e.iniciar(AccionCobro::PagarConDescuento);
        let (x, y) = e.centro();
        e.clic(x + 100.0, y - 120.0);
        e.clic(x + 52.0, y - 72.0);
        e.capturar("35-modal-descuento-monto-fijo");
    });
    en_ambos(true, |e| {
        e.iniciar(AccionCobro::Cancelar);
        let (x, y) = e.centro();
        e.clic(x + 167.0, y + 150.0);
        e.pausa(300);
        e.escribir("El cliente cambió de opinión");
        e.capturar("24-modal-cancelar-controlada");
    });
}
