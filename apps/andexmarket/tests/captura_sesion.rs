mod comun;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use andexmarket::app::{App, Contexto, Global, Ruta, Servicios};
use andexmarket::pantallas::sesion;
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
    json!({ "id": 1, "nombre": "Roberto Sánchez", "email": "admin@lahuerta.test", "businessId": 7, "businessName": "Verdulería La Huerta" })
}

fn negocio() -> Value {
    json!({
        "id": 7,
        "nombre": "Verdulería La Huerta",
        "sistemaPago": "redondeo",
        "sucursales": [{ "id": 1, "nombre": "Casa central", "direccion": "Av. San Martín 1450, Godoy Cruz", "telefono": "261 424-8890", "esPrincipal": true, "businessId": 7 }]
    })
}

fn vendedores() -> Value {
    json!([
        { "id": 11, "nombre": "Lucía Fernández", "email": "lucia@lahuerta.test" },
        { "id": 12, "nombre": "Martín Gómez", "email": "martin@lahuerta.test" },
        { "id": 13, "nombre": "Sofía Ramírez", "email": "sofia@lahuerta.test" },
        { "id": 14, "nombre": "Diego Álvarez", "email": "diego@lahuerta.test" },
        { "id": 15, "nombre": "Carla Benítez", "email": "carla@lahuerta.test" }
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

fn capturar(nombre: &str, global: Global, rutas: Rutas, pasos: impl FnOnce(&mut Harness<'_, App>)) {
    let base = api_simulada(rutas);
    let app = App::de_prueba(global, servicios(nombre, &base));
    let mut harness = Harness::builder()
        .with_size([1200.0, 800.0])
        .with_step_dt(0.05)
        .wgpu()
        .build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    harness.run_steps(2);
    pasos(&mut harness);
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
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

fn vendedores_listos(app: &mut App) -> bool {
    !app.pantallas().usuario_login.cargando_vendedores
}

fn completar_admin(harness: &mut Harness<'_, App>, contrasena: &str) {
    clic(harness, 600.0, 412.0);
    escribir(harness, "admin@lahuerta.test");
    clic(harness, 600.0, 500.0);
    escribir(harness, contrasena);
}

fn rutas(f: impl Fn(&str, &str) -> Option<Respuesta> + Send + Sync + 'static) -> Rutas {
    Arc::new(move |metodo, ruta| f(metodo, ruta).or_else(|| rutas_base(metodo, ruta)).unwrap_or(Respuesta::Json(404, json!({}))))
}

#[test]
fn admin_login() {
    capturar("01-admin-login", global(Ruta::AdminLogin, false), rutas(|_, _| None), |_| {});
}

#[test]
fn admin_login_error() {
    let api = rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then(|| Respuesta::Json(401, json!({ "message": "Credenciales inválidas" }))));
    capturar("02-admin-login-error", global(Ruta::AdminLogin, false), api, |h| {
        completar_admin(h, "clave-incorrecta");
        tecla(h, Key::Enter);
        esperar(h, |app| app.pantallas().admin_login.tarea.is_none() && !app.pantallas().admin_login.error.is_empty());
    });
}

#[test]
fn admin_login_cargando() {
    let api = rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then_some(Respuesta::Pendiente));
    capturar("03-admin-login-cargando", global(Ruta::AdminLogin, false), api, |h| {
        completar_admin(h, "clave-incorrecta");
        clic(h, 600.0, 592.0);
    });
}

#[test]
fn admin_login_exito() {
    let api = rutas(|metodo, ruta| match (metodo, ruta) {
        ("POST", "/api/login") => Some(Respuesta::Json(200, json!({ "token": "token-admin" }))),
        ("GET", "/api/auth/session") => Some(Respuesta::Json(200, sesion_de("Roberto Sánchez", "admin@lahuerta.test", "ADMIN"))),
        _ => None,
    });
    capturar("04-admin-login-exito", global(Ruta::AdminLogin, false), api, |h| {
        completar_admin(h, "clave-correcta");
        tecla(h, Key::Enter);
        esperar(h, |app| app.global().ruta == Ruta::UsuarioLogin);
        esperar(h, vendedores_listos);
        let guardado = h.state_mut().global().admin.clone().unwrap();
        assert_eq!((guardado["businessId"].clone(), guardado["businessName"].clone()), (json!(7), json!("Verdulería La Huerta")));
        esperar(h, |app| app.global().negocio.as_ref().is_some_and(|n| n["sucursalActiva"]["id"] == 1));
    });
}

#[test]
fn usuario_login_cargando() {
    let api = rutas(|_, ruta| ruta.starts_with("/api/usuarios/vendedores").then_some(Respuesta::Pendiente));
    capturar("05-user-login-cargando", global(Ruta::UsuarioLogin, true), api, |_| {});
}

#[test]
fn usuario_login() {
    capturar("06-user-login", global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| esperar(h, vendedores_listos));
}

#[test]
fn usuario_login_contrasena() {
    capturar("07-user-login-contrasena", global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        assert!(h.state_mut().pantallas().usuario_login.contrasena.is_empty());
    });
}

#[test]
fn usuario_login_contrasena_corta() {
    capturar("08-user-login-contrasena-corta", global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "12");
        tecla(h, Key::Enter);
        assert!(h.state_mut().pantallas().usuario_login.inicio.is_none());
    });
}

#[test]
fn usuario_login_error() {
    let api = rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then(|| Respuesta::Json(401, json!({ "message": "Credenciales inválidas" }))));
    capturar("09-user-login-error", global(Ruta::UsuarioLogin, true), api, |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "1234");
        tecla(h, Key::Enter);
        esperar(h, |app| app.pantallas().usuario_login.inicio.is_none());
    });
}

#[test]
fn usuario_login_iniciando() {
    let api = rutas(|metodo, ruta| (metodo == "POST" && ruta == "/api/login").then_some(Respuesta::Pendiente));
    capturar("10-user-login-iniciando", global(Ruta::UsuarioLogin, true), api, |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "1234");
        tecla(h, Key::Enter);
    });
}

#[test]
fn usuario_login_reconfigurar() {
    capturar("11-user-login-reconfigurar", global(Ruta::UsuarioLogin, true), rutas(|_, _| None), |h| {
        esperar(h, vendedores_listos);
        clic(h, 1019.0, 80.0);
    });
}

#[test]
fn usuario_login_error_vendedores() {
    let api = rutas(|_, ruta| ruta.starts_with("/api/usuarios/vendedores").then(|| Respuesta::Json(500, json!({ "error": "Error interno" }))));
    capturar("12-user-login-error-vendedores", global(Ruta::UsuarioLogin, true), api, |h| esperar(h, vendedores_listos));
}

#[test]
fn mensaje_admin() {
    let api = rutas(|metodo, ruta| match (metodo, ruta) {
        ("POST", "/api/login") => Some(Respuesta::Json(200, json!({ "token": "token-lucia" }))),
        ("GET", "/api/auth/session") => Some(Respuesta::Json(200, sesion_de("Lucía Fernández", "lucia@lahuerta.test", "ADMIN"))),
        _ => None,
    });
    capturar("13-admin-message", global(Ruta::UsuarioLogin, true), api, |h| {
        esperar(h, vendedores_listos);
        tecla(h, Key::Num1);
        escribir(h, "1234");
        tecla(h, Key::Enter);
        esperar(h, |app| app.global().ruta == Ruta::MensajeAdmin);
        let usuario = h.state_mut().global().usuario.clone().unwrap();
        assert_eq!(usuario["permisos"], json!({ "pesoManualEnabled": false, "cierreDeCajaEnabled": true }));
    });
}

#[test]
fn vendedor_entra_al_carrito_y_cierra_sesion() {
    let api = rutas(|metodo, ruta| match (metodo, ruta) {
        ("POST", "/api/login") => Some(Respuesta::Json(200, json!({ "token": "token-lucia" }))),
        ("GET", "/api/auth/session") => Some(Respuesta::Json(200, sesion_de("Lucía Fernández", "lucia@lahuerta.test", "VENDEDOR"))),
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
    assert_eq!(copia.config.get("offlineCredentials")["email"], "lucia@lahuerta.test");
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
