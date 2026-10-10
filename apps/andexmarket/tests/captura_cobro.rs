mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use andexmarket::app::{Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::cobro::{AccionCobro, Cobro};
use andexmarket::ui;
use eframe::egui::{self, Key, Pos2};
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

fn responder(metodo: &str, ruta: &str, cuerpo: &Value) -> (u16, Value) {
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
        ("GET", r) if r.starts_with("/api/mercadopago/check-status") => (200, json!({ "status": "PENDING" })),
        ("POST", "/api/ordenes") => (200, json!({ "id": 9022, "idReal": 9022, "estado": "COMPLETADA" })),
        ("POST", "/api/facturas/crear-afip") => (
            201,
            json!({
                "message": "Factura AFIP creada correctamente",
                "factura": { "id": 88, "numero": "0002-00000213", "numeroFactura": "213", "puntoVenta": "0002", "tipoFactura": "B" },
                "afip": { "cae": "75412369899123", "vencimientoCae": "20261020" },
            }),
        ),
        _ => (404, json!({ "error": "No encontrado" })),
    }
}

fn servidor(registro: Registro) -> String {
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
            let (estado, respuesta) = responder(metodo, ruta, &cuerpo);
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
        producto(1, "Tomate perito", 1.245, 2400.0, "Kg", 10.5),
        producto(2, "Papa negra", 2.5, 900.0, "Kg", 10.5),
        producto(5, "Manzana roja", 0.98, 2800.0, "Kg", 10.5),
        producto(21, "Huevos blancos maple x30", 1.0, 6500.0, "Unidad", 21.0),
        producto(19, "Palta Hass", 3.0, 1500.0, "Unidad", 10.5),
        producto(12, "Pimiento morrón rojo", 0.415, 4500.0, "Kg", 10.5),
        producto(23, "Perejil (atado)", 2.0, 800.0, "Unidad", 10.5),
    ] {
        carritos.agregar(item);
    }
    carritos
}

fn negocio(afip: bool) -> Value {
    json!({
        "id": 7,
        "nombre": "Verdulería La Huerta",
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
            "razonSocial": "La Huerta S.R.L.",
            "tipoFactura": "B",
        },
    })
}

fn harness(afip: bool, registro: Registro) -> Harness<'static, Estado> {
    let mut global = comun::global(Ruta::Carrito);
    global.negocio = Some(negocio(afip));
    global.admin = Some(json!({ "id": 1, "nombre": "Roberto Sánchez", "businessId": 7, "businessName": "Verdulería La Huerta" }));
    global.usuario = Some(json!({ "id": 11, "nombre": "Lucía Fernández", "email": "lucia@lahuerta.test", "sucursalId": 1, "role": "VENDEDOR" }));
    global.carritos = carrito();
    let mut servicios = comun::servicios();
    servicios.api = Cliente::new(&servidor(registro));
    servicios.config.set("impresoraWindows", json!("Impresora inexistente de prueba"));
    let estado = Estado { global, servicios, cobro: Cobro::default(), instalado: false };
    let mut h = Harness::builder()
        .with_size([1200.0, 800.0])
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
                let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ui.ctx().clone() };
                e.cobro.mostrar(ui, &mut cx);
                ui::toast::pintar(ui.ctx());
            },
            estado,
        );
    h.run_steps(3);
    h
}

fn iniciar(h: &mut Harness<'static, Estado>, accion: AccionCobro) {
    let ctx = h.ctx.clone();
    let e = h.state_mut();
    let mut cx = Contexto { global: &mut e.global, servicios: &e.servicios, egui: ctx };
    e.cobro.iniciar(accion, &mut cx);
    h.run_steps(3);
}

fn tecla(h: &mut Harness<'static, Estado>, tecla: Key) {
    h.key_press(tecla);
    h.run_steps(3);
}

fn escribir(h: &mut Harness<'static, Estado>, texto: &str) {
    h.event(egui::Event::Text(texto.into()));
    h.run_steps(3);
}

fn clic(h: &mut Harness<'static, Estado>, x: f32, y: f32) {
    let pos = Pos2::new(x, y);
    h.event(egui::Event::PointerMoved(pos));
    h.run_steps(1);
    let boton = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    h.event(boton(true));
    h.run_steps(1);
    h.event(boton(false));
    h.run_steps(3);
    h.event(egui::Event::PointerGone);
    h.run_steps(1);
}

fn esperar(h: &mut Harness<'static, Estado>, mut listo: impl FnMut(&mut Harness<'static, Estado>) -> bool) {
    for _ in 0..600 {
        if listo(h) {
            return;
        }
        h.run_steps(1);
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("tiempo de espera agotado");
}

fn pausa(h: &mut Harness<'static, Estado>, milisegundos: u64) {
    for _ in 0..milisegundos / 25 {
        h.run_steps(1);
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn capturar(h: &mut Harness<'static, Estado>, nombre: &str) {
    h.run_steps(2);
    let imagen = h.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}

#[test]
fn pagos_negocio_simple() {
    let registro = Registro::default();
    let mut h = harness(false, registro.clone());
    iniciar(&mut h, AccionCobro::Cancelar);
    capturar(&mut h, "24-cancelar-orden");
    tecla(&mut h, Key::Escape);
    iniciar(&mut h, AccionCobro::Pagar);
    capturar(&mut h, "25-pago-metodos");
    tecla(&mut h, Key::Num3);
    capturar(&mut h, "26-pago-efectivo-redondeo");
    tecla(&mut h, Key::F5);
    capturar(&mut h, "27-pago-efectivo-con-descuento");
    tecla(&mut h, Key::Escape);
    iniciar(&mut h, AccionCobro::Pagar);
    tecla(&mut h, Key::Num4);
    capturar(&mut h, "28-pago-dividido");
    escribir(&mut h, "10000");
    capturar(&mut h, "29-pago-dividido-con-monto");
    tecla(&mut h, Key::Escape);
    iniciar(&mut h, AccionCobro::Pagar);
    tecla(&mut h, Key::Num1);
    pausa(&mut h, 600);
    capturar(&mut h, "30-pago-qr-mercadopago");
    tecla(&mut h, Key::Enter);
    capturar(&mut h, "31-pago-qr-manual");
    escribir(&mut h, "clave-que-no-es");
    tecla(&mut h, Key::Enter);
    pausa(&mut h, 300);
    assert!(pedidos(&registro, "/api/mercadopago/manual-complete").is_empty());
}

#[test]
fn pago_mixto_qr_con_descuento() {
    let registro = Registro::default();
    let mut h = harness(false, registro.clone());
    iniciar(&mut h, AccionCobro::PagarConDescuento);
    escribir(&mut h, "10");
    tecla(&mut h, Key::Enter);
    tecla(&mut h, Key::Num4);
    escribir(&mut h, "5000");
    clic(&mut h, 716.0, 524.0);
    capturar(&mut h, "29b-pago-dividido-qr-con-descuento");
    tecla(&mut h, Key::Enter);
    esperar(&mut h, |_| !pedidos(&registro, "/api/mercadopago/generate-qr").is_empty());
    let cuerpo = &pedidos(&registro, "/api/mercadopago/generate-qr")[0];
    let total = cuerpo["subtotalSinDescuento"].as_f64().unwrap() * 0.9;
    assert!((cuerpo["monto"].as_f64().unwrap() - total).abs() < 0.01, "{cuerpo}");
    assert_eq!(cuerpo["cashAmount"], 5000.0);
    assert_eq!(cuerpo["isSplitPayment"], true);
}

#[test]
fn descuentos_negocio_simple() {
    let mut h = harness(false, Arc::default());
    iniciar(&mut h, AccionCobro::PagarConDescuento);
    capturar(&mut h, "32-descuento");
    escribir(&mut h, "10");
    capturar(&mut h, "33-descuento-con-valor");
    clic(&mut h, 703.0, 267.0);
    clic(&mut h, 600.0, 347.0);
    escribir(&mut h, "1500");
    capturar(&mut h, "33b-descuento-cantidad-fija");
    let mut h = harness(false, Arc::default());
    iniciar(&mut h, AccionCobro::PagarConDescuento);
    escribir(&mut h, "10");
    tecla(&mut h, Key::Enter);
    capturar(&mut h, "34-pago-con-descuento");
    tecla(&mut h, Key::Num2);
    esperar(&mut h, |h| !h.state().cobro.ocupado());
    capturar(&mut h, "35-toast-exito-orden");
    assert!(h.state().global.carritos.items().is_empty());
}

#[test]
fn pagos_afip() {
    let registro = Registro::default();
    let mut h = harness(true, registro.clone());
    iniciar(&mut h, AccionCobro::PagarAfip);
    capturar(&mut h, "45-pago-afip");
    tecla(&mut h, Key::Num3);
    capturar(&mut h, "46-pago-afip-efectivo-exacto");
    escribir(&mut h, "50000");
    capturar(&mut h, "47-pago-afip-efectivo-vuelto");
    tecla(&mut h, Key::Escape);
    iniciar(&mut h, AccionCobro::PagarAfip);
    tecla(&mut h, Key::Num4);
    capturar(&mut h, "48-pago-afip-dividido");
    escribir(&mut h, "15000");
    capturar(&mut h, "49-pago-afip-dividido-con-monto");
    tecla(&mut h, Key::Escape);
    iniciar(&mut h, AccionCobro::PagarAfip);
    tecla(&mut h, Key::Num1);
    pausa(&mut h, 400);
    capturar(&mut h, "50-pago-afip-qr");
    let mut h = harness(true, registro.clone());
    iniciar(&mut h, AccionCobro::Pagar);
    tecla(&mut h, Key::Num3);
    capturar(&mut h, "51-pago-efectivo-exacto");
    tecla(&mut h, Key::Escape);
    iniciar(&mut h, AccionCobro::PagarAfipConDescuento);
    escribir(&mut h, "5");
    tecla(&mut h, Key::Enter);
    capturar(&mut h, "52-pago-afip-con-descuento");
    h.key_press(Key::Num2);
    esperar(&mut h, |_| !pedidos(&registro, "/api/facturas/crear-afip").is_empty());
    pausa(&mut h, 300);
    capturar(&mut h, "53-pago-afip-procesando");
    esperar(&mut h, |h| !h.state().cobro.ocupado());
    capturar(&mut h, "53b-pago-afip-terminado");
    assert!(h.state().global.carritos.items().is_empty());
}

#[test]
fn afip_efectivo_manda_el_descuento_sin_aplicar_d() {
    let registro = Registro::default();
    let mut h = harness(true, registro.clone());
    h.state_mut().global.negocio.as_mut().unwrap()["sistemaPago"] = json!("redondeo");
    iniciar(&mut h, AccionCobro::PagarAfipConDescuento);
    escribir(&mut h, "5");
    tecla(&mut h, Key::Enter);
    tecla(&mut h, Key::Num3);
    capturar(&mut h, "52b-pago-afip-efectivo-con-descuento");
    tecla(&mut h, Key::Enter);
    esperar(&mut h, |_| !pedidos(&registro, "/api/facturas/crear-afip").is_empty());
    let cuerpo = &pedidos(&registro, "/api/facturas/crear-afip")[0];
    assert_eq!(cuerpo["tieneDescuento"], true, "{cuerpo}");
    assert_eq!(cuerpo["tipoDescuento"], "porcentual");
    assert_eq!(cuerpo["valorDescuento"], 5.0);
    assert!(cuerpo["montoDescuento"].as_f64().unwrap() > 0.0);
    esperar(&mut h, |h| !h.state().cobro.ocupado());
}

#[test]
fn cancelacion_controlada() {
    let mut h = harness(true, Arc::default());
    iniciar(&mut h, AccionCobro::Cancelar);
    clic(&mut h, 722.0, 440.0);
    capturar(&mut h, "54-cancelacion-motivo");
}
