mod comun;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use andexmarket::app::{App, Ruta};
use andexmarket::ui::v2;
use chrono::Duration as Dias;
use eframe::egui::{self, Event, PointerButton, Pos2};
use egui_kittest::Harness;
use nucleo::api::Cliente;
use serde_json::{json, Value};

const PRODUCTOS: [(&str, &str, f64, bool); 24] = [
    ("Tomate redondo", "Kg", 2890.0, true),
    ("Tomate perita", "Kg", 2590.0, true),
    ("Papa negra", "Kg", 1150.0, false),
    ("Batata", "Kg", 1380.0, true),
    ("Cebolla", "Kg", 1300.0, true),
    ("Zanahoria", "Kg", 1100.0, false),
    ("Zapallo anco", "Kg", 1040.0, true),
    ("Morrón rojo", "Kg", 4280.0, true),
    ("Lechuga criolla", "Unidad", 950.0, false),
    ("Acelga", "Unidad", 780.0, true),
    ("Ajo", "Unidad", 480.0, true),
    ("Rúcula", "Unidad", 1085.0, false),
    ("Banana Ecuador", "Kg", 2400.0, true),
    ("Manzana roja", "Kg", 3200.0, true),
    ("Pera Williams", "Kg", 2940.0, false),
    ("Naranja de jugo", "Kg", 1450.0, true),
    ("Mandarina", "Kg", 1800.0, true),
    ("Limón", "Kg", 2100.0, false),
    ("Palta Hass", "Unidad", 1600.0, true),
    ("Frutilla", "Kg", 7280.0, true),
    ("Uva blanca", "Kg", 5240.0, false),
    ("Kiwi", "Kg", 5670.0, true),
    ("Huevos blancos x12", "Unidad", 3300.0, true),
    ("Nuez mariposa 250 g", "Unidad", 5260.0, false),
];

const CLIENTES: [(&str, &str, &str); 5] = [
    ("Restaurante La Huerta", "30-71234567-8", "responsable_inscripto"),
    ("Hotel Aconcagua", "30-70987654-1", "responsable_inscripto"),
    ("Comedor San José", "30-71555222-4", "exento"),
    ("Bar El Patio", "20-28765432-9", "monotributo"),
    ("Rotisería Doña Ana", "27-30111222-5", "monotributo"),
];

type Pago = (i64, f64, &'static str, &'static str);
type Semilla = (&'static str, usize, i64, &'static [(usize, f64)], &'static [Pago]);

const FACTURAS: [Semilla; 9] = [
    ("remito", 1, -28, &[(1, 12.0), (3, 25.0), (5, 10.0)], &[(-20, 30000.0, "efectivo", ""), (-6, 20000.0, "transferencia", "TRF 88213")]),
    ("A", 2, -22, &[(13, 18.0), (14, 12.0), (19, 30.0), (23, 8.0)], &[(-15, 60000.0, "transferencia", "TRF 87541")]),
    ("remito", 3, -19, &[(3, 40.0), (6, 15.0), (7, 12.0)], &[]),
    ("A", 1, -14, &[(8, 6.0), (2, 20.0), (9, 24.0)], &[(-14, 25000.0, "efectivo", "")]),
    ("C", 4, -11, &[(16, 10.0), (18, 6.0), (17, 8.0)], &[(-11, 37000.0, "tarjeta", "Lote 0042")]),
    ("remito", 5, -8, &[(1, 8.0), (4, 10.0), (12, 10.0)], &[]),
    ("A", 2, -5, &[(20, 4.0), (21, 6.0), (22, 3.0)], &[(-2, 40000.0, "transferencia", "TRF 90012")]),
    ("remito", 1, -3, &[(10, 20.0), (11, 30.0), (5, 15.0)], &[]),
    ("C", 3, -1, &[(23, 10.0), (24, 4.0)], &[]),
];

fn precio(indice: usize) -> f64 {
    let (_, _, base, con_lista) = PRODUCTOS[indice - 1];
    if con_lista {
        (base * 0.88 / 10.0).round() * 10.0
    } else {
        base
    }
}

fn fecha(dias: i64, hora: u32) -> String {
    let dia = nucleo::facturas::hoy_argentina() + Dias::days(dias);
    format!("{}T{:02}:00:00.000Z", dia.format("%Y-%m-%d"), hora + 3)
}

fn facturas() -> Vec<Value> {
    let mut lista: Vec<Value> = FACTURAS
        .iter()
        .enumerate()
        .map(|(i, (tipo, cliente, dias, items, pagos))| {
            let (letra, secuencia) = match *tipo {
                "remito" => ("R", 114 + i),
                "A" => ("A", 212 + i),
                _ => ("C", 1838 + i),
            };
            let detalles: Vec<Value> = items
                .iter()
                .map(|(producto, cantidad)| {
                    let (nombre, medida, _, _) = PRODUCTOS[producto - 1];
                    let unitario = precio(*producto);
                    json!({ "productoId": producto, "cantidad": cantidad, "precioUnitario": unitario, "subtotal": (cantidad * unitario).round(), "producto": { "id": producto, "nombre": nombre, "tipoMedida": medida } })
                })
                .collect();
            let total: f64 = detalles.iter().map(|d| d["subtotal"].as_f64().unwrap()).sum();
            let pagado: f64 = pagos.iter().map(|p| p.1).sum();
            let pagos: Vec<Value> = pagos
                .iter()
                .enumerate()
                .map(|(j, (dias, monto, metodo, referencia))| {
                    let mut pago = json!({ "id": format!("{i}-{j}"), "monto": monto, "metodoPago": metodo, "fecha": fecha(*dias, 12) });
                    if !referencia.is_empty() {
                        pago["referencia"] = json!(referencia);
                    }
                    pago
                })
                .collect();
            let (nombre, cuit, condicion) = CLIENTES[cliente - 1];
            let iva = if *tipo == "remito" { 0.0 } else { (total - total / 1.21).round() };
            json!({
                "id": format!("f{i}"),
                "numero": format!("{letra} 0003-{secuencia:08}"),
                "tipoFactura": tipo,
                "puntoVenta": 3,
                "fecha": fecha(*dias, 10),
                "cliente": { "id": cliente, "razonSocial": nombre, "cuit": cuit, "condicionFiscal": condicion, "email": "compras@cliente.com.ar" },
                "detalles": detalles,
                "subtotal": total - iva,
                "impuestos": iva,
                "total": total,
                "pagos": pagos,
                "estado": if pagado >= total { "pagada" } else { "pendiente" },
                "observaciones": "",
            })
        })
        .collect();
    lista.reverse();
    lista
}

#[derive(Clone, Copy, Default)]
struct Opciones {
    copias: usize,
    demora: u64,
}

type Registro = Arc<Mutex<Vec<(String, String, Value)>>>;

fn consulta(ruta: &str, clave: &str) -> String {
    let (_, parametros) = ruta.split_once('?').unwrap_or((ruta, ""));
    parametros
        .split('&')
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| *k == clave)
        .map(|(_, v)| v.replace('+', " ").replace("%20", " ").to_lowercase())
        .unwrap_or_default()
}

fn lista_filtrada(opciones: Opciones, ruta: &str) -> Value {
    let mut todas = facturas();
    for copia in 0..opciones.copias {
        for (i, mut f) in facturas().into_iter().enumerate() {
            f["id"] = json!(format!("c{copia}-{i}"));
            todas.push(f);
        }
    }
    let busqueda = consulta(ruta, "search");
    let estado = consulta(ruta, "estado");
    let tipo = consulta(ruta, "tipoFactura");
    let filtradas: Vec<Value> = todas
        .into_iter()
        .filter(|f| f["tipoFactura"] != "B")
        .filter(|f| busqueda.is_empty() || f["cliente"]["razonSocial"].as_str().unwrap().to_lowercase().contains(&busqueda) || f["numero"].as_str().unwrap().to_lowercase().contains(&busqueda))
        .filter(|f| estado.is_empty() || f["estado"] == estado.as_str())
        .filter(|f| tipo.is_empty() || f["tipoFactura"].as_str().unwrap().to_lowercase() == tipo)
        .collect();
    let limite: usize = consulta(ruta, "limit").parse().unwrap_or(12);
    let pagina: usize = consulta(ruta, "page").parse().unwrap_or(1);
    let total = filtradas.len();
    let pagina_facturas: Vec<Value> = filtradas.into_iter().skip((pagina - 1) * limite).take(limite).collect();
    json!({ "facturas": pagina_facturas, "pagination": { "total": total, "page": pagina, "limit": limite, "totalPages": total.div_ceil(limite) } })
}

fn responder(opciones: Opciones, metodo: &str, ruta: &str, enviado: &Value) -> (u16, Value, u64) {
    let camino = ruta.split('?').next().unwrap_or(ruta);
    let partes: Vec<&str> = camino.trim_start_matches('/').split('/').collect();
    let buscar = |id: &str| facturas().into_iter().find(|f| f["id"] == id);
    match (metodo, partes.as_slice()) {
        ("GET", ["api", "facturas"]) => (200, lista_filtrada(opciones, ruta), opciones.demora),
        ("GET", ["api", "facturas", "generar-pdf"]) => (200, json!("%PDF-1.4"), 0),
        ("GET", ["api", "facturas", id]) => match buscar(id) {
            Some(f) => (200, f, opciones.demora),
            None => (404, json!({ "error": "Factura no encontrada" }), 0),
        },
        ("PUT", ["api", "facturas", id]) => match buscar(id) {
            Some(mut f) => {
                for (clave, valor) in enviado.as_object().unwrap() {
                    f[clave] = valor.clone();
                }
                (200, f, 0)
            }
            None => (404, json!({}), 0),
        },
        ("DELETE", ["api", "facturas", _]) => (200, json!({ "message": "Factura anulada correctamente" }), 0),
        ("POST", ["api", "facturas", "agregar-pago"]) => (200, json!({ "success": true }), 0),
        _ => (404, json!({ "error": "sin mock" }), 0),
    }
}

fn servidor(opciones: Opciones) -> (String, Registro) {
    let registro: Registro = Arc::default();
    let anotar = registro.clone();
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", escucha.local_addr().unwrap());
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
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
                let (estado, respuesta, demora) = responder(opciones, &metodo, &ruta, &enviado);
                anotar.lock().unwrap().push((metodo, ruta, enviado));
                std::thread::sleep(Duration::from_millis(demora));
                let cuerpo = respuesta.to_string();
                let mut salida = conexion;
                let _ = write!(
                    salida,
                    "HTTP/1.1 {estado} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
                    cuerpo.len()
                );
            });
        }
    });
    (url, registro)
}

fn armar(ruta: Ruta, opciones: Opciones, tam: [f32; 2]) -> (Harness<'static, App>, Registro) {
    let (url, registro) = servidor(opciones);
    let mut servicios = comun::servicios();
    servicios.api = Cliente::new(&url);
    let app = App::de_prueba(comun::global(ruta), servicios);
    let mut harness = Harness::builder()
        .with_size(tam)
        .wgpu()
        .build_ui_state(
            |ui, app: &mut App| {
                let rect = ui.ctx().content_rect();
                let mut pleno = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                pleno.set_clip_rect(rect);
                app.mostrar(&mut pleno);
                v2::toast::pintar(ui.ctx());
            },
            app,
        );
    esperar(&mut harness, 600);
    (harness, registro)
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

fn escribir<S>(harness: &mut Harness<'_, S>, texto: &str) {
    harness.event(Event::Text(texto.into()));
    harness.step();
}

fn guardar<S>(harness: &mut Harness<'_, S>, nombre: &str) {
    harness.remove_cursor();
    esperar(harness, 400);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}

const TAMANOS: [(&str, [f32; 2]); 2] = [("1920", [1920.0, 1080.0]), ("1366", [1366.0, 768.0])];

fn en_ambos(nombre: &str, ruta: Ruta, opciones: Opciones, pasos: impl Fn(&mut Harness<'_, App>, f32)) {
    for (sufijo, tam) in TAMANOS {
        let (mut h, _) = armar(ruta.clone(), opciones, tam);
        pasos(&mut h, tam[0]);
        guardar(&mut h, &format!("{nombre}-{sufijo}"));
    }
}

fn enviados(registro: &Registro, metodo: &str, ruta: &str) -> Vec<Value> {
    registro.lock().unwrap().iter().filter(|(m, r, _)| m == metodo && r == ruta).map(|(_, _, c)| c.clone()).collect()
}

fn listas_pedidas(registro: &Registro) -> Vec<String> {
    registro.lock().unwrap().iter().filter(|(m, r, _)| m == "GET" && r.starts_with("/api/facturas?")).map(|(_, r, _)| r.clone()).collect()
}

#[test]
fn cc_lista() {
    en_ambos("70-cc-lista", Ruta::Facturas, Opciones::default(), |_, _| {});
}

fn segun(ancho: f32, grande: f32, chico: f32) -> f32 {
    if ancho > 1500.0 {
        grande
    } else {
        chico
    }
}

fn rueda<S>(harness: &mut Harness<'_, S>, x: f32, y: f32, delta: f32) {
    harness.event(Event::PointerMoved(Pos2::new(x, y)));
    harness.step();
    harness.event(Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: egui::vec2(0.0, delta), phase: egui::TouchPhase::Move, modifiers: Default::default() });
    esperar(harness, 300);
}

const FILA_1: f32 = 374.0;

#[test]
fn cc_filtros() {
    en_ambos("71-cc-filtro-con-saldo", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 534.0, 420.0), 262.0);
        esperar(h, 600);
    });
    en_ambos("72-cc-busqueda", Ruta::Facturas, Opciones::default(), |h, _| {
        clic(h, 150.0, 262.0);
        escribir(h, "hotel");
        esperar(h, 900);
    });
    en_ambos("73-cc-dropdown-tipo", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 935.0, 820.0), 262.0);
        esperar(h, 300);
    });
    en_ambos("74-cc-filtro-tipo-remito", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 935.0, 820.0), 262.0);
        esperar(h, 300);
        clic(h, segun(ancho, 935.0, 820.0), 342.0);
        esperar(h, 600);
    });
    en_ambos("75-cc-vacio", Ruta::Facturas, Opciones::default(), |h, _| {
        clic(h, 150.0, 262.0);
        escribir(h, "zzz");
        esperar(h, 900);
    });
}

#[test]
fn cc_paginacion() {
    en_ambos("76-cc-paginacion-2", Ruta::Facturas, Opciones { copias: 1, ..Default::default() }, |h, ancho| {
        rueda(h, ancho / 2.0, 500.0, -3000.0);
        clic(h, segun(ancho, 1822.0, 1269.0), segun(ancho, 1012.0, 700.0));
        esperar(h, 600);
        rueda(h, ancho / 2.0, 500.0, 3000.0);
    });
}

#[test]
fn cc_detalle() {
    en_ambos("77-cc-detalle-parcial", Ruta::FacturaDetalle("f1".into()), Opciones::default(), |_, _| {});
    en_ambos("78-cc-detalle-sin-pagos", Ruta::FacturaDetalle("f2".into()), Opciones::default(), |_, _| {});
    en_ambos("79-cc-detalle-pagada", Ruta::FacturaDetalle("f4".into()), Opciones::default(), |_, _| {});
}

#[test]
fn cc_modales() {
    en_ambos("80-modal-agregar-pago", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 1709.0, 1155.0), FILA_1);
        esperar(h, 400);
    });
    en_ambos("83-modal-historial-pagos", Ruta::Facturas, Opciones::default(), |h, ancho| {
        rueda(h, ancho / 2.0, 500.0, -3000.0);
        clic(h, segun(ancho, 1761.0, 1207.0), segun(ancho, 870.0, 696.0));
        esperar(h, 400);
    });
    en_ambos("84-modal-historial-pagos-vacio", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 1761.0, 1207.0), FILA_1);
        esperar(h, 400);
    });
    en_ambos("85-modal-confirmar-eliminar-factura", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 1857.0, 1303.0), FILA_1);
        esperar(h, 400);
    });
}

fn en_modal(ancho: f32, x: f32, y: f32) -> (f32, f32) {
    if ancho > 1500.0 {
        (x + 277.0, y + 156.0)
    } else {
        (x, y)
    }
}

#[test]
fn cc_agregar_pago_variantes() {
    en_ambos("81-modal-agregar-pago-completo", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 1709.0, 1155.0), FILA_1);
        esperar(h, 400);
        let (x, y) = en_modal(ancho, 877.0, 314.0);
        clic(h, x, y);
        let (x, y) = en_modal(ancho, 836.0, 434.0);
        clic(h, x, y);
        let (x, y) = en_modal(ancho, 762.0, 513.0);
        clic(h, x, y);
        escribir(h, "TRF 90544");
        esperar(h, 200);
    });
    en_ambos("82-modal-agregar-pago-error", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 1709.0, 1155.0), FILA_1);
        esperar(h, 400);
        let (x, y) = en_modal(ancho, 680.0, 352.0);
        clic(h, x, y);
        escribir(h, "999999");
        h.key_press(egui::Key::Enter);
        esperar(h, 300);
    });
}

#[test]
fn cc_editar_factura() {
    en_ambos("68-cc-editar-factura", Ruta::Facturas, Opciones::default(), |h, ancho| {
        clic(h, segun(ancho, 1793.0, 1239.0), FILA_1);
        esperar(h, 400);
    });
}
