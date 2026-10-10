mod comun;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use andexmarket::app::{App, Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::{admin_login, sesion};
use eframe::egui::{self, Event, Key, PointerButton, Pos2};
use egui_kittest::Harness;
use hardware::balanza::Balanza;
use hardware::config::Config;
use nucleo::api::Cliente;
use serde_json::{json, Value};

enum Respuesta {
    Json(u16, Value),
    Pendiente,
}

type Rutas = Arc<dyn Fn(&str, &str) -> Respuesta + Send + Sync>;

fn leer_pedido(conexion: &mut std::net::TcpStream) -> String {
    let mut pedido = Vec::new();
    let mut bufer = [0u8; 4096];
    loop {
        let leidos = conexion.read(&mut bufer).unwrap_or(0);
        pedido.extend_from_slice(&bufer[..leidos]);
        let texto = String::from_utf8_lossy(&pedido).to_string();
        if let Some(fin) = texto.find("\r\n\r\n") {
            let largo = texto
                .lines()
                .find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap_or(0)))
                .unwrap_or(0);
            if pedido.len() >= fin + 4 + largo {
                return texto;
            }
        }
        if leidos == 0 {
            return texto;
        }
    }
}

fn api_simulada(rutas: Rutas) -> String {
    let oyente = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", oyente.local_addr().unwrap());
    std::thread::spawn(move || {
        for conexion in oyente.incoming() {
            let Ok(mut conexion) = conexion else { continue };
            let rutas = rutas.clone();
            std::thread::spawn(move || {
                let pedido = leer_pedido(&mut conexion);
                let mut partes = pedido.split_whitespace();
                let (metodo, ruta) = (partes.next().unwrap_or_default(), partes.next().unwrap_or_default());
                match rutas(metodo, ruta) {
                    Respuesta::Json(estado, cuerpo) => {
                        let texto = cuerpo.to_string();
                        let respuesta = format!(
                            "HTTP/1.1 {estado} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}",
                            texto.len()
                        );
                        let _ = conexion.write_all(respuesta.as_bytes());
                    }
                    Respuesta::Pendiente => std::thread::sleep(Duration::from_secs(3600)),
                }
            });
        }
    });
    base
}

fn servicios(nombre: &str, base: &str) -> Servicios {
    let carpeta = std::env::temp_dir().join(format!("andexmarket-sesion-{}-{nombre}", std::process::id()));
    let _ = std::fs::remove_dir_all(&carpeta);
    let _ = std::fs::create_dir_all(&carpeta);
    Servicios {
        config: Arc::new(Config::cargar(carpeta.clone())),
        balanza: Balanza::new(|_| {}),
        api: Cliente::new(base),
        carpeta,
    }
}

fn admin() -> Value {
    json!({ "id": 1, "nombre": "Martín Álvarez", "email": "martin@donjose.com.ar", "businessId": 7, "businessName": "Verdulería Don José" })
}

fn negocio() -> Value {
    json!({
        "id": 7,
        "nombre": "Verdulería Don José",
        "sistemaPago": "redondeo",
        "sucursales": [{ "id": 1, "nombre": "Casa central", "direccion": "Av. San Martín 1450, Godoy Cruz", "telefono": "261 424-8890", "esPrincipal": true, "businessId": 7 }]
    })
}

fn vendedores() -> Value {
    json!([
        { "id": 11, "nombre": "Lucía Fernández", "email": "lucia@donjose.com.ar" },
        { "id": 12, "nombre": "Diego Romero", "email": "diego@donjose.com.ar" },
        { "id": 13, "nombre": "Sofía Medina", "email": "sofia@donjose.com.ar" },
        { "id": 14, "nombre": "Tomás Quiroga", "email": "tomas@donjose.com.ar" },
        { "id": 15, "nombre": "Martín Álvarez", "email": "martin@donjose.com.ar", "rol": { "nombre": "ADMIN" } }
    ])
}

fn sesion_de(nombre: &str, email: &str, rol: &str) -> Value {
    json!({ "user": { "id": 11, "nombre": nombre, "email": email, "rol": { "nombre": rol }, "businessId": 7, "sucursalId": 1, "sucursal": { "id": 1, "businessId": 7 }, "pesoManualEnabled": false, "cierreDeCajaEnabled": true } })
}

fn rutas_base(metodo: &str, ruta: &str) -> Option<Respuesta> {
    match (metodo, ruta) {
        ("GET", r) if r.starts_with("/api/business/7?include=") => Some(Respuesta::Json(200, negocio())),
        ("GET", "/api/business/7") => Some(Respuesta::Json(200, negocio())),
        ("GET", r) if r.starts_with("/api/usuarios/vendedores") => Some(Respuesta::Json(200, vendedores())),
        _ => None,
    }
}

fn global(ruta: Ruta, con_admin: bool) -> Global {
    let mut global = comun::global(ruta);
    global.admin = con_admin.then(admin);
    global.negocio = None;
    global.usuario = None;
    global
}

const TAMANOS: [[f32; 2]; 2] = [[1920.0, 1080.0], [1366.0, 768.0]];

fn armar(nombre: &str, global: Global, rutas: Rutas, tam: [f32; 2]) -> Harness<'static, App> {
    let base = api_simulada(rutas);
    let app = App::de_prueba(global, servicios(nombre, &base));
    Harness::builder().with_size(tam).with_step_dt(0.05).wgpu().build_ui_state(|ui, app: &mut App| app.mostrar(ui), app)
}

fn capturar(nombre: &str, global: impl Fn() -> Global, rutas: Rutas, pasos: impl Fn(&mut Harness<'_, App>)) {
    for tam in TAMANOS {
        let ancho = tam[0] as u32;
        let mut harness = armar(&format!("{nombre}-{ancho}"), global(), rutas.clone(), tam);
        harness.run_steps(2);
        pasos(&mut harness);
        harness.run_steps(8);
        let imagen = harness.render().expect("render");
        imagen.save(comun::carpeta_capturas().join(format!("{nombre}-{ancho}.png"))).expect("guardar captura");
    }
}

fn correr(nombre: &str, global: Global, rutas: Rutas, pasos: impl FnOnce(&mut Harness<'_, App>)) {
    let base = api_simulada(rutas);
    let app = App::de_prueba(global, servicios(nombre, &base));
    let mut harness = Harness::builder().with_size([1366.0, 768.0]).with_step_dt(0.05).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    harness.run_steps(2);
    pasos(&mut harness);
}

fn esperar(harness: &mut Harness<'_, App>, condicion: impl Fn(&mut App) -> bool) {
    for _ in 0..500 {
        harness.step();
        if condicion(harness.state_mut()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("la condición no se cumplió a tiempo");
}

fn clic(harness: &mut Harness<'_, App>, x: f32, y: f32) {
    let pos = Pos2::new(x, y);
    harness.event(Event::PointerMoved(pos));
    harness.step();
    harness.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Default::default() });
    harness.step();
    harness.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Default::default() });
    harness.step();
    harness.event(Event::PointerGone);
    harness.step();
}

fn escribir(harness: &mut Harness<'_, App>, texto: &str) {
    harness.event(Event::Text(texto.into()));
    harness.step();
}

fn tecla(harness: &mut Harness<'_, App>, tecla: Key) {
    harness.key_press(tecla);
    harness.step();
}

fn enfocar(harness: &mut Harness<'_, App>, id: egui::Id) {
    harness.ctx.memory_mut(|m| m.request_focus(id));
    harness.step();
}

fn vendedores_listos(app: &mut App) -> bool {
    !app.pantallas().usuario_login.cargando_vendedores
}

fn completar_admin(harness: &mut Harness<'_, App>, contrasena: &str) {
    enfocar(harness, admin_login::id_email());
    escribir(harness, "martin@donjose.com.ar");
    if !contrasena.is_empty() {
        enfocar(harness, admin_login::id_contrasena());
        escribir(harness, contrasena);
    }
}

fn rutas(f: impl Fn(&str, &str) -> Option<Respuesta> + Send + Sync + 'static) -> Rutas {
    Arc::new(move |metodo, ruta| f(metodo, ruta).or_else(|| rutas_base(metodo, ruta)).unwrap_or(Respuesta::Json(404, json!({}))))
}

fn login_pendiente() -> Rutas {
    rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then_some(Respuesta::Pendiente))
}

fn login_como(rol: &'static str) -> Rutas {
    rutas(move |metodo, ruta| match (metodo, ruta) {
        ("POST", "/api/login") => Some(Respuesta::Json(200, json!({ "token": "token-vendedor" }))),
        ("GET", "/api/auth/session") => Some(Respuesta::Json(200, sesion_de("Martín Álvarez", "martin@donjose.com.ar", rol))),
        _ => None,
    })
}

#[test]
fn config_inicial() {
    capturar("01-config-inicial", || global(Ruta::AdminLogin, false), rutas(|_, _| None), |_| {});
}

#[test]
fn config_inicial_error() {
    capturar("02-config-inicial-error", || global(Ruta::AdminLogin, false), rutas(|_, _| None), |h| {
        completar_admin(h, "");
        tecla(h, Key::Enter);
        assert_eq!(h.state_mut().pantallas().admin_login.error, "Por favor, ingresa email y contraseña");
        assert!(h.state_mut().pantallas().admin_login.tarea.is_none());
    });
}

#[test]
fn config_inicial_cargando() {
    capturar("03-config-inicial-cargando", || global(Ruta::AdminLogin, false), login_pendiente(), |h| {
        completar_admin(h, "clave-correcta");
        tecla(h, Key::Enter);
        assert!(h.state_mut().pantallas().admin_login.tarea.is_some());
    });
}

#[test]
fn quien_vende() {
    capturar("04-quien-vende", || global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| esperar(h, vendedores_listos));
}

#[test]
fn quien_vende_cargando() {
    let api = rutas(|_, ruta| ruta.starts_with("/api/usuarios/vendedores").then_some(Respuesta::Pendiente));
    capturar("05-quien-vende-cargando", || global(Ruta::UsuarioLogin, true), api, |_| {});
}

#[test]
fn modal_contrasena_vendedor() {
    capturar("06-modal-contrasena-vendedor", || global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        assert!(h.state_mut().pantallas().usuario_login.contrasena.is_empty());
    });
}

#[test]
fn modal_contrasena_vendedor_error() {
    capturar("07-modal-contrasena-vendedor-error", || global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "12");
        tecla(h, Key::Enter);
        assert!(h.state_mut().pantallas().usuario_login.inicio.is_none());
    });
}

#[test]
fn modal_contrasena_vendedor_cargando() {
    capturar("08-modal-contrasena-vendedor-cargando", || global(Ruta::UsuarioLogin, true), login_pendiente(), |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "1234");
        tecla(h, Key::Enter);
        assert!(h.state_mut().pantallas().usuario_login.inicio.is_some());
    });
}

#[test]
fn modal_confirmar_reconfigurar() {
    capturar("09-modal-confirmar-reconfigurar", || global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| {
        esperar(h, vendedores_listos);
        let ancho = h.ctx.content_rect().width();
        clic(h, ancho - 28.0 - 60.0, 32.0);
        assert!(h.state_mut().pantallas().usuario_login.dialogo_reconfigurar);
    });
}

#[test]
fn cuenta_administrador() {
    capturar("10-cuenta-administrador", || global(Ruta::UsuarioLogin, true), login_como("ADMIN"), |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num5);
        escribir(h, "1234");
        tecla(h, Key::Enter);
        esperar(h, |app| app.global().ruta == Ruta::MensajeAdmin);
        let usuario = h.state_mut().global().usuario.clone().unwrap();
        assert_eq!(usuario["permisos"], json!({ "pesoManualEnabled": false, "cierreDeCajaEnabled": true }));
    });
}

#[test]
fn cuenta_administrador_vuelve_a_elegir_usuario() {
    let mut global = global(Ruta::MensajeAdmin, true);
    global.usuario = Some(json!({ "id": 1, "nombre": "Martín Álvarez", "rol": { "nombre": "ADMIN" } }));
    correr("volver-desde-admin", global, rutas(|_, _| None), |h| {
        h.run_steps(4);
        let alto = sesion::alto_medido(&h.ctx, egui::Id::new("mensaje-admin-tarjeta")).unwrap();
        let arriba = (384.0 - alto / 2.0).round();
        clic(h, 683.0, arriba + alto - 33.0 - 21.0);
        h.run_steps(2);
        assert!(h.state_mut().global().usuario.is_none());
        assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
    });
}

#[test]
fn config_inicial_rechazada_por_la_api() {
    let api = rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then(|| Respuesta::Json(401, json!({ "message": "Credenciales inválidas" }))));
    correr("config-rechazada", global(Ruta::AdminLogin, false), api, |h| {
        completar_admin(h, "clave-incorrecta");
        tecla(h, Key::Enter);
        esperar(h, |app| app.pantallas().admin_login.tarea.is_none() && !app.pantallas().admin_login.error.is_empty());
    });
}

#[test]
fn config_inicial_guarda_el_negocio() {
    let api = rutas(|metodo, ruta| match (metodo, ruta) {
        ("POST", "/api/login") => Some(Respuesta::Json(200, json!({ "token": "token-admin" }))),
        ("GET", "/api/auth/session") => Some(Respuesta::Json(200, sesion_de("Martín Álvarez", "martin@donjose.com.ar", "ADMIN"))),
        _ => None,
    });
    correr("config-exito", global(Ruta::AdminLogin, false), api, |h| {
        completar_admin(h, "clave-correcta");
        tecla(h, Key::Enter);
        esperar(h, |app| app.global().ruta == Ruta::UsuarioLogin);
        esperar(h, vendedores_listos);
        let guardado = h.state_mut().global().admin.clone().unwrap();
        assert_eq!((guardado["businessId"].clone(), guardado["businessName"].clone()), (json!(7), json!("Verdulería Don José")));
        esperar(h, |app| app.global().negocio.as_ref().is_some_and(|n| n["sucursalActiva"]["id"] == 1));
    });
}

#[test]
fn vendedor_con_contrasena_invalida_ve_el_error() {
    let api = rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then(|| Respuesta::Json(401, json!({ "message": "Credenciales inválidas" }))));
    correr("vendedor-rechazado", global(Ruta::UsuarioLogin, true), api, |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "1234");
        tecla(h, Key::Enter);
        esperar(h, |app| app.pantallas().usuario_login.inicio.is_none());
        assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
    });
}

#[test]
fn error_al_cargar_vendedores() {
    let api = rutas(|_, ruta| ruta.starts_with("/api/usuarios/vendedores").then(|| Respuesta::Json(500, json!({ "error": "Error interno" }))));
    correr("error-vendedores", global(Ruta::UsuarioLogin, true), api, |h| {
        esperar(h, vendedores_listos);
        assert!(h.state_mut().pantallas().usuario_login.vendedores.is_empty());
    });
}

#[test]
fn vendedor_entra_al_carrito_y_cierra_sesion() {
    let api = rutas(|metodo, ruta| match (metodo, ruta) {
        ("POST", "/api/login") => Some(Respuesta::Json(200, json!({ "token": "token-lucia" }))),
        ("GET", "/api/auth/session") => Some(Respuesta::Json(200, sesion_de("Lucía Fernández", "lucia@donjose.com.ar", "VENDEDOR"))),
        _ => None,
    });
    let base = api_simulada(api);
    let servicios = servicios("flujo-vendedor", &base);
    let copia = servicios.clone();
    let app = App::de_prueba(global(Ruta::Inicio, true), servicios);
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
    esperar(&mut h, vendedores_listos);
    tecla(&mut h, Key::Num1);
    escribir(&mut h, "1234");
    tecla(&mut h, Key::Enter);
    esperar(&mut h, |app| app.global().ruta == Ruta::Carrito);
    assert_eq!(copia.config.get("offlineCredentials")["email"], "lucia@donjose.com.ar");
    let mut cx = Contexto { global: h.state_mut().global(), servicios: &copia, egui: egui::Context::default() };
    sesion::cerrar_sesion(&mut cx);
    assert!(copia.config.get("offlineCredentials").is_null());
    assert!(h.state_mut().global().usuario.is_none());
    h.run_steps(2);
    assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
}

#[test]
fn ruta_protegida_sin_usuario_vuelve_al_selector() {
    let base = api_simulada(rutas(|_, _| None));
    let app = App::de_prueba(global(Ruta::Facturas, true), servicios("protegida", &base));
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    assert_eq!(h.state_mut().global().ruta, Ruta::UsuarioLogin);
}

#[test]
fn contrasena_admin_en_texto_plano_se_migra_cifrada() {
    let base = api_simulada(rutas(|_, _| None));
    let servicios = servicios("migrar-contrasena", &base);
    servicios.config.set("adminEmail", json!("martin@donjose.com.ar"));
    servicios.config.set("adminPassword", json!("clave-correcta"));
    servicios.config.set("rememberAdminCredentials", json!("true"));
    let copia = servicios.clone();
    let app = App::de_prueba(global(Ruta::AdminLogin, false), servicios);
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    assert!(!copia.config.has("adminPassword"));
    let cifrada = copia.config.get("adminPasswordCifrada").as_str().unwrap().to_string();
    assert!(!cifrada.contains("clave"));
    assert_eq!(nucleo::sesion::descifrar(&cifrada).unwrap(), "clave-correcta");
    let pantalla = &h.state_mut().pantallas().admin_login;
    assert_eq!((pantalla.email.as_str(), pantalla.contrasena.as_str(), pantalla.recordar), ("martin@donjose.com.ar", "clave-correcta", true));
    let texto = std::fs::read_to_string(copia.carpeta.join("config.json")).unwrap();
    assert!(!texto.contains("clave-correcta"));
}

#[test]
fn admin_con_rol_nombre_no_entra_al_carrito() {
    let base = api_simulada(rutas(|_, _| None));
    let mut global = global(Ruta::Carrito, true);
    global.usuario = Some(json!({ "id": 1, "nombre": "Roberto", "rol": { "nombre": "ADMIN" }, "permisos": {} }));
    let app = App::de_prueba(global, servicios("protegida-admin", &base));
    let mut h = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    h.run_steps(3);
    assert_eq!(h.state_mut().global().ruta, Ruta::MensajeAdmin);
}
