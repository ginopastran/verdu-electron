mod comun;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use andexmarket::app::{App, Global, Ruta, Servicios};
use andexmarket::pantallas::carrito::id_buscador;
use andexmarket::pantallas::carrito::productos::ProductoDisponible;
use andexmarket::ui::v2::toast;
use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use hardware::balanza::Balanza;
use hardware::config::Config;
use nucleo::api::Cliente;
use nucleo::carrito::Carritos;
use serde_json::{json, Value};

const TAMANOS: [(&str, [f32; 2]); 2] = [("1920", [1920.0, 1080.0]), ("1366", [1366.0, 768.0])];

const CATALOGO: [(&str, &str, f64, &str, &str); 24] = [
    ("Tomate redondo", "Kg", 2890.0, "101", "7790001000213"),
    ("Tomate perita", "Kg", 2590.0, "102", "7790001000220"),
    ("Papa negra", "Kg", 1150.0, "103", "7790001000046"),
    ("Batata", "Kg", 1380.0, "104", "7790001000053"),
    ("Cebolla", "Kg", 1300.0, "105", "7790001000084"),
    ("Zanahoria", "Kg", 1100.0, "106", "7790001000121"),
    ("Zapallo anco", "Kg", 1040.0, "107", "7790001000404"),
    ("Morrón rojo", "Kg", 4280.0, "108", "7790001000411"),
    ("Lechuga criolla", "Unidad", 950.0, "109", "7790001000305"),
    ("Acelga", "Unidad", 780.0, "110", "7790001000312"),
    ("Ajo", "Unidad", 480.0, "111", "7790001000091"),
    ("Rúcula", "Unidad", 1085.0, "112", "7790001000329"),
    ("Banana Ecuador", "Kg", 2400.0, "201", "7790001002019"),
    ("Manzana roja", "Kg", 3200.0, "202", "7790001002002"),
    ("Pera Williams", "Kg", 2940.0, "203", "7790001002033"),
    ("Naranja de jugo", "Kg", 1450.0, "204", "7790001002217"),
    ("Mandarina", "Kg", 1800.0, "205", "7790001002224"),
    ("Limón", "Kg", 2100.0, "206", "7790001002255"),
    ("Palta Hass", "Unidad", 1600.0, "207", "7790001002330"),
    ("Frutilla", "Kg", 7280.0, "208", "7790001002606"),
    ("Uva blanca", "Kg", 5240.0, "209", "7790001002705"),
    ("Kiwi", "Kg", 5670.0, "210", "7790001002507"),
    ("Huevos blancos x12", "Unidad", 3300.0, "301", "7790001003016"),
    ("Nuez mariposa 250 g", "Unidad", 5260.0, "302", "7790001003108"),
];

fn productos() -> Vec<ProductoDisponible> {
    CATALOGO
        .iter()
        .enumerate()
        .map(|(i, (nombre, unidad, precio, plu, codigo))| {
            ProductoDisponible::desde_backend(&json!({
                "id": i + 1, "nombre": nombre, "precio": precio, "tipoMedida": unidad, "costo": 0,
                "codigoBarras": codigo, "plu": plu, "ivaIncluido": true, "ivaPorcentaje": 10.5
            }))
        })
        .collect()
}

fn precio_lista(id: usize, factor: f64) -> f64 {
    (CATALOGO[id - 1].2 * factor / 10.0).round() * 10.0
}

#[derive(Clone, Copy)]
struct Opciones {
    afip: bool,
    facturacion: bool,
    controlada: bool,
    peso_manual: bool,
}

const POR_DEFECTO: Opciones = Opciones { afip: true, facturacion: true, controlada: false, peso_manual: true };

fn negocio(o: Opciones) -> Value {
    json!({
        "id": 4, "nombre": "Verdulería Don José", "sistemaPago": "redondeo", "descuentoEfectivo": 10,
        "mpEnabled": true, "afipHabilitado": o.afip, "facturacionHabilitada": o.facturacion,
        "cancelacionHabilitada": o.controlada, "ivaIncluidoEnPrecios": true, "popupListaPreciosEnabled": false,
        "listaPrecioPorDefectoId": null, "etiquetaPrecioBase": "Precio catálogo"
    })
}

fn global(o: Opciones) -> Global {
    let mut global = comun::global(Ruta::Carrito);
    global.negocio = Some(negocio(o));
    global.usuario = Some(json!({
        "id": 1, "nombre": "Lucía Fernández", "email": "lucia@donjose.com.ar", "role": "VENDEDOR",
        "permisos": { "pesoManualEnabled": o.peso_manual, "cierreDeCajaEnabled": true }
    }));
    global.peso = json!(1245);
    global.estado_balanza = json!({ "conectada": true });
    global
}

fn item(id: usize, cantidad: f64, lista: Option<(i64, &str, f64)>) -> Value {
    let (nombre, unidad, precio, _, _) = CATALOGO[id - 1];
    let precio = lista.map_or(precio, |l| precio_lista(id, l.2));
    json!({
        "id": id, "cartId": format!("{id}-semilla-{cantidad}"), "name": nombre, "quantity": cantidad, "unit": unidad,
        "pricePerUnit": precio, "subtotal": (precio * cantidad * 100.0).round() / 100.0, "costo": 0,
        "ivaIncluido": true, "ivaPorcentaje": 10.5,
        "listaPrecioId": lista.map(|l| l.0), "listaPrecioNombre": lista.map_or("Precio catálogo", |l| l.1)
    })
}

fn items_orden() -> Vec<Value> {
    vec![
        item(1, 1.245, None),
        item(3, 2.1, None),
        item(9, 2.0, None),
        item(13, 0.85, Some((1, "Mayorista", 0.88))),
        item(23, 1.0, None),
        item(19, 3.0, None),
    ]
}

fn carritos(pantallas: Vec<Vec<Value>>, activa: usize) -> Carritos {
    let screens: Vec<Value> = pantallas.into_iter().enumerate().map(|(id, items)| json!({ "id": id, "items": items })).collect();
    serde_json::from_value(json!({ "screens": screens, "activeScreen": activa })).unwrap()
}

fn con_items(o: Opciones) -> Global {
    let mut g = global(o);
    g.carritos = carritos(vec![items_orden()], 0);
    g
}

fn responder(ruta: &str) -> (u16, Value) {
    let fila = |id: usize, lista: i64, nombre: &str, factor: f64| {
        json!({ "listaPrecioId": lista, "precio": precio_lista(id, factor), "activa": true, "listaPrecio": { "nombre": nombre, "activa": true } })
    };
    let id = ruta.strip_prefix("/api/productos/").and_then(|r| r.strip_suffix("/listas-precios")).and_then(|n| n.parse::<usize>().ok());
    match id {
        Some(id) if (id - 1) % 3 != 2 => (200, json!({ "productos": [fila(id, 1, "Mayorista", 0.88), fila(id, 2, "Restaurantes", 0.93)] })),
        Some(_) => (200, json!({ "productos": [] })),
        None if ruta == "/api/ordenes" => (200, json!({ "id": 9022, "idReal": 9022, "estado": "COMPLETADA" })),
        None => (500, json!({ "error": "sin datos" })),
    }
}

fn api_falsa() -> Cliente {
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let puerto = escucha.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
            let mut lector = BufReader::new(conexion.try_clone().unwrap());
            let mut linea = String::new();
            let _ = lector.read_line(&mut linea);
            loop {
                let mut encabezado = String::new();
                if lector.read_line(&mut encabezado).unwrap_or(0) == 0 || encabezado.trim().is_empty() {
                    break;
                }
            }
            let ruta = linea.split_whitespace().nth(1).unwrap_or("/").to_string();
            let (estado, cuerpo) = responder(&ruta);
            let cuerpo = cuerpo.to_string();
            let mut salida = conexion;
            let _ = write!(salida, "HTTP/1.1 {estado} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}", cuerpo.len());
        }
    });
    Cliente::new(&format!("http://127.0.0.1:{puerto}"))
}

fn servicios() -> Servicios {
    let carpeta = std::env::temp_dir().join(format!("andexmarket-carrito-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&carpeta);
    let config = Config::cargar(carpeta.clone());
    config.set("impresoraWindows", json!("TP806L"));
    Servicios { config: Arc::new(config), balanza: Balanza::new(|_| {}), api: api_falsa(), carpeta }
}

fn arnes(global: Global, tam: [f32; 2], wgpu: bool) -> Harness<'static, App> {
    let mut app = App::de_prueba(global, servicios());
    app.pantallas().carrito.precargar_productos(productos());
    let constructor = Harness::builder().with_size(tam);
    let constructor = if wgpu { constructor.wgpu() } else { constructor };
    let mut harness = constructor.build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    harness.run_steps(3);
    harness
}

fn capturar(nombre: &str, global: impl Fn() -> Global, pasos: impl Fn(&mut Harness<'_, App>, [f32; 2])) {
    for (sufijo, tam) in TAMANOS {
        let mut harness = arnes(global(), tam, true);
        pasos(&mut harness, tam);
        harness.run_steps(6);
        let imagen = harness.render().expect("render");
        imagen.save(comun::carpeta_capturas().join(format!("{nombre}-{sufijo}.png"))).expect("guardar captura");
    }
}

fn escribir(harness: &mut Harness<'_, App>, texto: &str) {
    harness.event(Event::Text(texto.into()));
    harness.run_steps(2);
}

fn tecla(harness: &mut Harness<'_, App>, tecla: Key) {
    harness.key_press(tecla);
    harness.run_steps(2);
}

fn clic(harness: &mut Harness<'_, App>, pos: Pos2) {
    harness.event(Event::PointerMoved(pos));
    harness.run_steps(1);
    for pressed in [true, false] {
        harness.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
        harness.run_steps(1);
    }
    harness.run_steps(2);
}

fn esperar_red(harness: &mut Harness<'_, App>) {
    for _ in 0..40 {
        harness.step();
        std::thread::sleep(Duration::from_millis(15));
    }
}

fn abrir_producto(harness: &mut Harness<'_, App>, busqueda: &str) {
    escribir(harness, busqueda);
    tecla(harness, Key::Enter);
    esperar_red(harness);
}

fn etiqueta(harness: &mut Harness<'_, App>, texto: &str) {
    harness.get_by_label(texto).click();
    harness.run_steps(3);
}

fn nada(_: &mut Harness<'_, App>, _: [f32; 2]) {}

fn sin_manual() -> Opciones {
    Opciones { peso_manual: false, ..POR_DEFECTO }
}

#[test]
fn pos_orden_vacia() {
    capturar("11-pos-orden-vacia", || global(POR_DEFECTO), nada);
}

#[test]
fn pos_con_items() {
    capturar("12-pos-con-items", || con_items(POR_DEFECTO), nada);
}

#[test]
fn pos_buscador_resultados() {
    capturar("13-pos-buscador-resultados", || con_items(POR_DEFECTO), |h, _| escribir(h, "to"));
}

#[test]
fn pos_buscador_segundo_resultado() {
    capturar("14-pos-buscador-segundo-resultado", || con_items(POR_DEFECTO), |h, _| {
        escribir(h, "to");
        tecla(h, Key::ArrowDown);
    });
}

#[test]
fn pos_buscador_sin_resultados() {
    capturar("15-pos-buscador-sin-resultados", || con_items(POR_DEFECTO), |h, _| escribir(h, "xyz"));
}

#[test]
fn pos_menu_usuario() {
    capturar("16-pos-menu-usuario", || con_items(POR_DEFECTO), |h, tam| clic(h, Pos2::new(tam[0] - 100.0, 34.0)));
}

#[test]
fn pos_varias_pestanas() {
    capturar(
        "17-pos-varias-pestanas",
        || {
            let mut g = global(POR_DEFECTO);
            g.carritos = carritos(vec![Vec::new(), vec![item(1, 1.245, None), item(9, 2.0, None)], Vec::new()], 1);
            g
        },
        nada,
    );
}

#[test]
fn pos_sin_afip() {
    capturar("18-pos-sin-afip", || con_items(Opciones { afip: false, ..POR_DEFECTO }), nada);
}

#[test]
fn pos_sin_facturacion() {
    capturar("19-pos-sin-facturacion", || con_items(Opciones { facturacion: false, ..POR_DEFECTO }), nada);
}

#[test]
fn pos_sin_afip_sin_facturacion() {
    capturar("20-pos-sin-afip-sin-facturacion", || con_items(Opciones { afip: false, facturacion: false, ..POR_DEFECTO }), nada);
}

#[test]
fn pos_cancelacion_controlada() {
    capturar("21-pos-cancelacion-controlada", || con_items(Opciones { controlada: true, ..POR_DEFECTO }), nada);
}

#[test]
fn pos_responsive() {
    for (sufijo, tam) in [("1280", [1280.0, 720.0]), ("1200", [1200.0, 800.0]), ("1024", [1024.0, 768.0])] {
        let mut harness = arnes(con_items(POR_DEFECTO), tam, true);
        harness.run_steps(6);
        let imagen = harness.render().expect("render");
        imagen.save(comun::carpeta_capturas().join(format!("22-pos-responsive-{sufijo}.png"))).expect("guardar captura");
    }
}

#[test]
fn pos_buscador_muchos_resultados() {
    capturar("13b-pos-buscador-muchos-resultados", || con_items(POR_DEFECTO), |h, _| {
        escribir(h, "a");
        for _ in 0..9 {
            tecla(h, Key::ArrowDown);
        }
        assert!(buscador_enfocado(h));
    });
}

#[test]
fn modal_escanear_lista() {
    capturar(
        "55-modal-escanear-lista",
        || {
            let mut g = con_items(POR_DEFECTO);
            g.negocio.as_mut().unwrap()["popupListaPreciosEnabled"] = json!(true);
            g
        },
        |h, _| {
            escribir(h, "7790001000213");
            esperar_red(h);
        },
    );
}

#[test]
fn modal_cerrar_pestana() {
    capturar(
        "25-modal-cerrar-pestana",
        || {
            let mut g = global(POR_DEFECTO);
            g.carritos = carritos(vec![items_orden(), vec![item(1, 1.245, None), item(9, 2.0, None)]], 0);
            g
        },
        |h, _| clic(h, Pos2::new(X_CERRAR_ORDEN_2, 106.0)),
    );
}

const X_CERRAR_ORDEN_2: f32 = 251.0;

#[test]
fn modal_agregar_kg() {
    capturar("26-modal-agregar-kg", || con_items(POR_DEFECTO), |h, _| {
        abrir_producto(h, "redondo");
        etiqueta(h, "Ingresar peso manualmente");
    });
}

#[test]
fn modal_agregar_kg_desconectada() {
    capturar(
        "27-modal-agregar-kg-desconectada",
        || {
            let mut g = con_items(sin_manual());
            g.estado_balanza = json!({ "conectada": false });
            g
        },
        |h, _| abrir_producto(h, "redondo"),
    );
}

#[test]
fn modal_agregar_kg_manual() {
    capturar("28-modal-agregar-kg-manual", || con_items(POR_DEFECTO), |h, _| {
        abrir_producto(h, "redondo");
        escribir(h, "1250");
    });
}

#[test]
fn modal_agregar_kg_sin_peso_manual() {
    capturar("29-modal-agregar-kg-sin-peso-manual", || con_items(sin_manual()), |h, _| abrir_producto(h, "zanahoria"));
}

#[test]
fn modal_agregar_unidad() {
    capturar("30-modal-agregar-unidad", || con_items(POR_DEFECTO), |h, _| {
        abrir_producto(h, "lechuga");
        escribir(h, "2");
    });
}

#[test]
fn modal_agregar_unidad_lista() {
    capturar("31-modal-agregar-unidad-lista", || con_items(POR_DEFECTO), |h, _| {
        abrir_producto(h, "palta");
        escribir(h, "3");
        etiqueta(h, "Mayorista");
    });
}

#[test]
fn modal_agregar_error() {
    capturar("32-modal-agregar-error", || con_items(POR_DEFECTO), |h, tam| {
        abrir_producto(h, "lechuga");
        clic(h, Pos2::new(tam[0] / 2.0 + 115.0, tam[1] / 2.0 + 113.0));
    });
}

#[test]
fn toast_error_orden_vacia() {
    capturar("88-toast-error", || global(POR_DEFECTO), |h, _| tecla(h, Key::F2));
}

#[test]
fn agregar_y_quitar_items() {
    let mut h = arnes(global(POR_DEFECTO), [1200.0, 800.0], false);
    escribir(&mut h, "palta");
    tecla(&mut h, Key::Enter);
    esperar_red(&mut h);
    escribir(&mut h, "3");
    tecla(&mut h, Key::Enter);
    h.run_steps(3);
    let items = h.state_mut().global().carritos.items().to_vec();
    assert_eq!(items.len(), 1);
    assert_eq!((items[0].quantity, items[0].unit.as_str(), items[0].subtotal), (3.0, "Unidad", 4800.0));
    assert_eq!(items[0].lista_precio_nombre.as_deref(), Some("Precio catálogo"));
    escribir(&mut h, "0101000013552");
    for _ in 0..20 {
        h.step();
        std::thread::sleep(Duration::from_millis(10));
    }
    let items = h.state_mut().global().carritos.items().to_vec();
    assert_eq!(items.len(), 2);
    assert_eq!((items[1].id, items[1].quantity), (1, 1.355));
}

#[test]
fn enter_cierra_venta_cobrada() {
    let mut h = arnes(con_items(Opciones { afip: false, ..POR_DEFECTO }), [1366.0, 768.0], false);
    tecla(&mut h, Key::F3);
    tecla(&mut h, Key::Num2);
    for _ in 0..300 {
        if h.state_mut().pantallas().carrito.cobro.venta_cobrada() {
            break;
        }
        h.step();
        std::thread::sleep(Duration::from_millis(20));
    }
    h.run_steps(4);
    assert!(h.state_mut().pantallas().carrito.cobro.venta_cobrada());
    assert!(h.state_mut().global().carritos.items().is_empty());
    assert!(buscador_enfocado(&h));
    tecla(&mut h, Key::Enter);
    assert!(!h.state_mut().pantallas().carrito.cobro.venta_cobrada());
    h.run_steps(4);
    assert!(buscador_enfocado(&h));
}

fn api_recarga(funciona: bool) -> Cliente {
    let escucha = TcpListener::bind("127.0.0.1:0").unwrap();
    let puerto = escucha.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for conexion in escucha.incoming().flatten() {
            let mut lector = BufReader::new(conexion.try_clone().unwrap());
            let mut linea = String::new();
            let _ = lector.read_line(&mut linea);
            loop {
                let mut encabezado = String::new();
                if lector.read_line(&mut encabezado).unwrap_or(0) == 0 || encabezado.trim().is_empty() {
                    break;
                }
            }
            let ruta = linea.split_whitespace().nth(1).unwrap_or("/").to_string();
            let (estado, cuerpo): (u16, Vec<u8>) = match ruta.as_str() {
                _ if !funciona => (500, b"{}".to_vec()),
                "/api/business/logo-comprobante" => (200, include_bytes!("../../../assets/imagenes/andex-market-logo.png").to_vec()),
                r if r.starts_with("/api/business/4") => (200, json!({ "id": 4, "nombre": "Verdulería Recargada", "sistemaPago": "redondeo", "facturacionHabilitada": true }).to_string().into_bytes()),
                r if r.starts_with("/api/productos?") => (
                    200,
                    json!({ "productos": [{ "id": 99, "nombre": "Producto recargado", "precio": 100, "tipoMedida": "Unidad" }], "pagination": { "hasNext": false } }).to_string().into_bytes(),
                ),
                _ => (404, b"{}".to_vec()),
            };
            let mut salida = conexion;
            let _ = write!(salida, "HTTP/1.1 {estado} OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", cuerpo.len());
            let _ = salida.write_all(&cuerpo);
        }
    });
    Cliente::new(&format!("http://127.0.0.1:{puerto}"))
}

fn recargar(funciona: bool) -> (Harness<'static, App>, std::path::PathBuf) {
    let carpeta = std::env::temp_dir().join(format!("andexmarket-recarga-{funciona}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&carpeta);
    let config = Config::cargar(carpeta.clone());
    config.set("impresoraWindows", json!("Impresora inexistente de prueba"));
    let logo = config.ruta_logo_negocio();
    let servicios = Servicios { config: Arc::new(config), balanza: Balanza::new(|_| {}), api: api_recarga(funciona), carpeta };
    let mut app = App::de_prueba(con_items(POR_DEFECTO), servicios);
    app.pantallas().carrito.precargar_productos(productos());
    let mut h = Harness::builder().with_size([1366.0, 768.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    esperar_red(&mut h);
    let _ = std::fs::remove_file(&logo);
    h.key_press_modifiers(Modifiers::COMMAND, Key::R);
    h.run_steps(2);
    for _ in 0..200 {
        if !h.state_mut().pantallas().recargando() {
            break;
        }
        h.step();
        std::thread::sleep(Duration::from_millis(15));
    }
    assert!(!h.state_mut().pantallas().recargando());
    h.run_steps(2);
    (h, logo)
}

#[test]
fn ctrl_r_recarga_negocio_productos_y_logo() {
    let (mut h, logo) = recargar(true);
    assert_eq!(toast::titulo_actual(&h.ctx).as_deref(), Some("Datos actualizados"));
    let app = h.state_mut();
    assert_eq!(app.global().negocio.as_ref().unwrap()["nombre"], "Verdulería Recargada");
    assert_eq!(app.global().ruta, Ruta::Carrito);
    assert_eq!(app.global().usuario.as_ref().unwrap()["nombre"], "Lucía Fernández");
    assert_eq!(app.global().carritos.items().len(), items_orden().len());
    let nombres: Vec<String> = app.pantallas().carrito.productos().iter().map(|p| p.nombre.clone()).collect();
    assert_eq!(nombres, ["Producto recargado"]);
    assert!(std::fs::read(&logo).unwrap().starts_with(b"\x89PNG"));
}

#[test]
fn ctrl_r_con_error_conserva_los_datos() {
    let (mut h, logo) = recargar(false);
    assert_eq!(toast::titulo_actual(&h.ctx).as_deref(), Some("No se pudieron actualizar los datos"));
    let app = h.state_mut();
    assert_eq!(app.global().negocio.as_ref().unwrap()["nombre"], "Verdulería Don José");
    assert_eq!(app.pantallas().carrito.productos().len(), CATALOGO.len());
    assert_eq!(app.global().carritos.items().len(), items_orden().len());
    assert!(!logo.exists());
}

fn buscador_enfocado(harness: &Harness<'_, App>) -> bool {
    harness.ctx.memory(|m| m.has_focus(id_buscador()))
}

#[test]
fn borrar_pestana_y_cerrar_sesion() {
    let mut g = global(POR_DEFECTO);
    g.carritos = carritos(vec![items_orden(), vec![item(1, 1.245, None), item(9, 2.0, None)]], 0);
    let mut h = arnes(g, [1920.0, 1080.0], false);
    clic(&mut h, Pos2::new(X_CERRAR_ORDEN_2, 106.0));
    assert_eq!(h.state_mut().global().carritos.screens.len(), 2);
    tecla(&mut h, Key::Enter);
    assert_eq!(h.state_mut().global().carritos.screens.len(), 1);
    tecla(&mut h, Key::F4);
    assert_eq!(h.state_mut().global().ruta, Ruta::Carrito);
    h.run_steps(70);
    assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
}

#[test]
fn pestana_clickeada_enfoca_buscador() {
    let mut g = global(POR_DEFECTO);
    g.carritos = carritos(vec![items_orden(), vec![item(1, 1.245, None)]], 0);
    let mut h = arnes(g, [1920.0, 1080.0], false);
    clic(&mut h, Pos2::new(X_CERRAR_ORDEN_2 - 60.0, 106.0));
    assert_eq!(h.state_mut().global().carritos.active_screen, 1);
    assert!(buscador_enfocado(&h));
    escribir(&mut h, "ma");
    assert_eq!(h.state_mut().global().carritos.active_screen, 1);
    tecla(&mut h, Key::Escape);
    clic(&mut h, Pos2::new(60.0, 106.0));
    assert_eq!(h.state_mut().global().carritos.active_screen, 0);
    assert!(buscador_enfocado(&h));
}

#[test]
fn menu_usuario_abre_impresion_y_balanza() {
    let mut h = arnes(global(POR_DEFECTO), [1920.0, 1080.0], false);
    clic(&mut h, Pos2::new(1820.0, 34.0));
    clic(&mut h, Pos2::new(1760.0, Y_MENU_IMPRESION));
    assert!(h.state_mut().pantallas().carrito.impresion.abierto());
    tecla(&mut h, Key::Escape);
    assert!(!h.state_mut().pantallas().carrito.impresion.abierto());
    clic(&mut h, Pos2::new(1820.0, 34.0));
    clic(&mut h, Pos2::new(1760.0, Y_MENU_IMPRESION + 37.0));
    assert!(h.state_mut().pantallas().carrito.balanza.abierto());
}

const Y_MENU_IMPRESION: f32 = 140.0;
