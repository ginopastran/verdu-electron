mod comun;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use andexmarket::app::{App, Global, Ruta, Servicios};
use andexmarket::pantallas::carrito::id_buscador;
use andexmarket::pantallas::carrito::productos::ProductoDisponible;
use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use hardware::balanza::Balanza;
use hardware::config::Config;
use nucleo::api::Cliente;
use nucleo::carrito::Carritos;
use serde_json::{json, Value};

fn productos() -> Vec<ProductoDisponible> {
    let datos = json!([
        [1, "Tomate perito", 2400, "Kg", null, "001"],
        [2, "Papa negra", 900, "Kg", null, "002"],
        [3, "Cebolla", 1100, "Kg", null, "003"],
        [4, "Banana Ecuador", 2200, "Kg", null, "004"],
        [5, "Manzana roja", 2800, "Kg", null, "005"],
        [6, "Naranja de jugo", 1500, "Kg", null, "006"],
        [7, "Mandarina", 1600, "Kg", null, "007"],
        [8, "Zapallo anco", 1200, "Kg", null, "008"],
        [9, "Zanahoria", 1000, "Kg", null, "009"],
        [10, "Limón", 2000, "Kg", null, "010"],
        [11, "Lechuga criolla", 2500, "Kg", null, "011"],
        [12, "Pimiento morrón rojo", 4500, "Kg", null, "012"],
        [13, "Batata", 1300, "Kg", null, "013"],
        [14, "Pera Williams", 2600, "Kg", null, "014"],
        [15, "Uva blanca", 4800, "Kg", null, "015"],
        [16, "Berenjena", 2000, "Kg", null, "016"],
        [17, "Zucchini", 2200, "Kg", null, "017"],
        [18, "Kiwi", 4000, "Kg", null, "018"],
        [19, "Palta Hass", 1500, "Unidad", "7791234500019", null],
        [20, "Ananá", 3500, "Unidad", "7791234500026", null],
        [21, "Huevos blancos maple x30", 6500, "Unidad", "7791234500033", null],
        [22, "Ajo (cabeza)", 600, "Unidad", "7791234500040", null],
        [23, "Perejil (atado)", 800, "Unidad", "7791234500057", null],
        [24, "Choclo", 700, "Unidad", "7791234500064", null],
        [25, "Frutilla bandeja 500 g", 3000, "Unidad", "7791234500071", null],
        [26, "Acelga (atado)", 1200, "Unidad", "7791234500088", null]
    ]);
    datos
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            ProductoDisponible::desde_backend(&json!({
                "id": p[0], "nombre": p[1], "precio": p[2], "tipoMedida": p[3], "costo": 0,
                "codigoBarras": p[4], "plu": p[5], "ivaIncluido": true, "ivaPorcentaje": 10.5
            }))
        })
        .collect()
}

fn negocio_simple() -> Value {
    json!({
        "id": 4, "nombre": "Verdulería La Huerta", "sistemaPago": "redondeo", "descuentoEfectivo": 10,
        "mpEnabled": true, "afipHabilitado": false, "facturacionHabilitada": false,
        "cancelacionHabilitada": false, "ivaIncluidoEnPrecios": true, "popupListaPreciosEnabled": false,
        "listaPrecioPorDefectoId": null, "etiquetaPrecioBase": "Precio catálogo"
    })
}

fn negocio_afip() -> Value {
    let mut negocio = negocio_simple();
    for clave in ["afipHabilitado", "facturacionHabilitada", "cancelacionHabilitada", "popupListaPreciosEnabled"] {
        negocio[clave] = json!(true);
    }
    negocio["sistemaPago"] = json!("pago-exacto");
    negocio
}

fn global(negocio: Value, peso_manual: bool) -> Global {
    let mut global = comun::global(Ruta::Carrito);
    global.negocio = Some(negocio);
    global.usuario = Some(json!({
        "id": 11, "nombre": "Lucía Fernández", "email": "lucia@lahuerta.test", "role": "VENDEDOR",
        "permisos": { "pesoManualEnabled": peso_manual, "cierreDeCajaEnabled": true }
    }));
    global.peso = json!(1245);
    global
}

fn carritos_sembrados() -> Carritos {
    let item = |id: i64, nombre: &str, cantidad: f64, unidad: &str, precio: f64| {
        json!({
            "id": id, "cartId": format!("{id}-seed-{cantidad}"), "name": nombre, "quantity": cantidad, "unit": unidad,
            "pricePerUnit": precio, "subtotal": (precio * cantidad * 100.0).round() / 100.0, "costo": 0,
            "ivaIncluido": true, "ivaPorcentaje": 10.5, "listaPrecioId": null, "listaPrecioNombre": "Precio catálogo"
        })
    };
    serde_json::from_value(json!({
        "screens": [
            { "id": 0, "items": [
                item(1, "Tomate perito", 1.245, "Kg", 2400.0),
                item(2, "Papa negra", 2.5, "Kg", 900.0),
                item(5, "Manzana roja", 0.98, "Kg", 2800.0),
                item(21, "Huevos blancos maple x30", 1.0, "Unidad", 6500.0),
                item(19, "Palta Hass", 3.0, "Unidad", 1500.0),
                item(12, "Pimiento morrón rojo", 0.415, "Kg", 4500.0),
                item(23, "Perejil (atado)", 2.0, "Unidad", 800.0)
            ]},
            { "id": 1, "items": [
                item(4, "Banana Ecuador", 1.1, "Kg", 2200.0),
                item(25, "Frutilla bandeja 500 g", 1.0, "Unidad", 3000.0)
            ]}
        ],
        "activeScreen": 0
    }))
    .unwrap()
}

fn responder(ruta: &str) -> (u16, Value) {
    let lista = |id: i64, nombre: &str, precio: f64| json!({ "listaPrecioId": id, "precio": precio, "activa": true, "listaPrecio": { "nombre": nombre, "activa": true } });
    match ruta {
        "/api/productos/1/listas-precios" => (200, json!({ "productos": [lista(1, "Mayorista", 2100.0), lista(2, "Comercios", 2250.0)] })),
        "/api/productos/20/listas-precios" => (200, json!({ "productos": [lista(1, "Mayorista", 3100.0)] })),
        r if r.ends_with("/listas-precios") => (200, json!({ "productos": [] })),
        _ => (500, json!({ "error": "sin datos" })),
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
    Servicios { config: Arc::new(Config::cargar(carpeta.clone())), balanza: Balanza::new(|_| {}), api: api_falsa(), carpeta }
}

fn capturar(nombre: &str, global: Global, preparar: impl FnOnce(&mut App), pasos: impl FnOnce(&mut Harness<'_, App>)) {
    capturar_en(nombre, [1200.0, 800.0], global, preparar, pasos);
}

fn capturar_en(nombre: &str, tam: [f32; 2], global: Global, preparar: impl FnOnce(&mut App), pasos: impl FnOnce(&mut Harness<'_, App>)) {
    let mut app = App::de_prueba(global, servicios());
    app.pantallas().carrito.precargar_productos(productos());
    preparar(&mut app);
    let mut harness = Harness::builder().with_size(tam).wgpu().build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    harness.run_steps(3);
    pasos(&mut harness);
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
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

fn nada(_: &mut Harness<'_, App>) {}

#[test]
fn carrito_vacio() {
    capturar("14-carrito-vacio", global(negocio_simple(), false), |_| {}, nada);
}

#[test]
fn toast_carrito_vacio() {
    capturar("15-toast-error-carrito-vacio", global(negocio_simple(), false), |_| {}, |h| tecla(h, Key::F2));
}

#[test]
fn busqueda_resultados() {
    capturar("16-busqueda-resultados", global(negocio_simple(), false), |_| {}, |h| escribir(h, "ma"));
}

#[test]
fn busqueda_flechas() {
    capturar("17-busqueda-flechas", global(negocio_simple(), false), |_| {}, |h| {
        escribir(h, "ma");
        tecla(h, Key::ArrowDown);
        tecla(h, Key::ArrowDown);
    });
}

#[test]
fn agregar_kg_balanza() {
    capturar("18-agregar-producto-kg-balanza", global(negocio_simple(), false), |_| {}, |h| {
        escribir(h, "zapallo");
        tecla(h, Key::Enter);
        esperar_red(h);
    });
}

#[test]
fn agregar_con_listas() {
    capturar("19-agregar-producto-con-listas", global(negocio_simple(), false), |_| {}, |h| {
        escribir(h, "tomate");
        tecla(h, Key::Enter);
        esperar_red(h);
    });
}

#[test]
fn agregar_unidad() {
    capturar("20-agregar-producto-unidad", global(negocio_simple(), false), |_| {}, |h| {
        escribir(h, "palta");
        tecla(h, Key::Enter);
        esperar_red(h);
        escribir(h, "3");
    });
}

#[test]
fn agregar_peso_manual() {
    capturar("21-agregar-producto-peso-manual", global(negocio_simple(), true), |_| {}, |h| {
        escribir(h, "zapallo");
        tecla(h, Key::Enter);
        esperar_red(h);
        escribir(h, "1750");
    });
}

#[test]
fn carrito_con_items() {
    let mut g = global(negocio_simple(), false);
    g.carritos = carritos_sembrados();
    capturar("22-carrito-con-items", g, |_| {}, nada);
}

#[test]
fn pestanas_carritos() {
    let mut g = global(negocio_simple(), false);
    g.carritos = carritos_sembrados();
    capturar("23-pestanas-carritos", g, |_| {}, |h| clic(h, Pos2::new(598.0, 45.0)));
}

#[test]
fn carrito_afip() {
    let mut g = global(negocio_afip(), false);
    g.carritos = carritos_sembrados();
    capturar("44-carrito-afip", g, |_| {}, nada);
}

#[test]
fn escanear_lista_precios() {
    capturar("55-escanear-lista-precios", global(negocio_afip(), false), |_| {}, |h| {
        escribir(h, "7791234500026");
        esperar_red(h);
    });
}

#[test]
fn menu_usuario() {
    capturar("71-menu-usuario", global(negocio_simple(), false), |_| {}, |h| {
        clic(h, Pos2::new(1156.0, 44.0));
    });
}

#[test]
fn agregar_y_quitar_items() {
    let mut app = App::de_prueba(global(negocio_simple(), false), servicios());
    app.pantallas().carrito.precargar_productos(productos());
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    escribir(&mut h, "palta");
    tecla(&mut h, Key::Enter);
    esperar_red(&mut h);
    escribir(&mut h, "3");
    tecla(&mut h, Key::Enter);
    h.run_steps(3);
    let items = h.state_mut().global().carritos.items().to_vec();
    assert_eq!(items.len(), 1);
    assert_eq!((items[0].quantity, items[0].unit.as_str(), items[0].subtotal), (3.0, "Unidad", 4500.0));
    assert_eq!(items[0].lista_precio_nombre.as_deref(), Some("Precio catálogo"));
    escribir(&mut h, "005000013552");
    for _ in 0..20 {
        h.step();
        std::thread::sleep(Duration::from_millis(10));
    }
    let items = h.state_mut().global().carritos.items().to_vec();
    assert_eq!(items.len(), 2);
    assert_eq!((items[1].id, items[1].quantity, items[1].subtotal), (5, 1.355, 3794.0));
}

#[test]
fn borrar_pestana_y_cerrar_sesion() {
    let mut g = global(negocio_simple(), false);
    g.carritos = carritos_sembrados();
    let mut app = App::de_prueba(g, servicios());
    app.pantallas().carrito.precargar_productos(productos());
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    clic(&mut h, Pos2::new(671.0, 45.0));
    assert_eq!(h.state_mut().global().carritos.screens.len(), 2);
    tecla(&mut h, Key::Enter);
    assert_eq!(h.state_mut().global().carritos.screens.len(), 1);
    tecla(&mut h, Key::F4);
    assert_eq!(h.state_mut().global().ruta, Ruta::Carrito);
    h.run_steps(70);
    assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
}

fn buscador_enfocado(harness: &Harness<'_, App>) -> bool {
    harness.ctx.memory(|m| m.has_focus(id_buscador()))
}

fn buscar_muchos(h: &mut Harness<'_, App>) {
    escribir(h, "a");
    for _ in 0..14 {
        tecla(h, Key::ArrowDown);
    }
    assert!(buscador_enfocado(h));
}

#[test]
fn busqueda_muchos_resultados() {
    capturar("16b-busqueda-muchos-resultados", global(negocio_simple(), false), |_| {}, buscar_muchos);
}

#[test]
fn busqueda_muchos_resultados_1920() {
    capturar_en("16c-busqueda-muchos-resultados-1920", [1920.0, 1080.0], global(negocio_simple(), false), |_| {}, buscar_muchos);
}

#[test]
fn menu_usuario_1920() {
    let mut g = global(negocio_afip(), false);
    g.carritos = carritos_sembrados();
    capturar_en("71b-menu-usuario-1920", [1920.0, 1080.0], g, |_| {}, |h| clic(h, Pos2::new(1876.0, 44.0)));
}

#[test]
fn pestana_clickeada_enfoca_buscador() {
    let mut g = global(negocio_simple(), false);
    g.carritos = carritos_sembrados();
    let mut app = App::de_prueba(g, servicios());
    app.pantallas().carrito.precargar_productos(productos());
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    clic(&mut h, Pos2::new(598.0, 45.0));
    assert_eq!(h.state_mut().global().carritos.active_screen, 1);
    assert!(buscador_enfocado(&h));
    escribir(&mut h, "ma");
    assert_eq!(h.state_mut().global().carritos.active_screen, 1);
    tecla(&mut h, Key::Escape);
    clic(&mut h, Pos2::new(465.0, 45.0));
    assert_eq!(h.state_mut().global().carritos.active_screen, 0);
    assert!(buscador_enfocado(&h));
}

#[test]
fn menu_usuario_abre_impresion_y_balanza() {
    let mut app = App::de_prueba(global(negocio_simple(), false), servicios());
    app.pantallas().carrito.precargar_productos(productos());
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    clic(&mut h, Pos2::new(1156.0, 44.0));
    clic(&mut h, Pos2::new(1020.0, 140.0));
    assert!(h.state_mut().pantallas().carrito.impresion.abierto());
    tecla(&mut h, Key::Escape);
    assert!(!h.state_mut().pantallas().carrito.impresion.abierto());
    clic(&mut h, Pos2::new(1156.0, 44.0));
    clic(&mut h, Pos2::new(1015.0, 172.0));
    assert!(h.state_mut().pantallas().carrito.balanza.abierto());
}

