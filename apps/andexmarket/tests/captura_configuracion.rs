mod comun;

use std::path::PathBuf;

use actualizador::Publicacion;
use andexmarket::app::{App, Contexto, Ruta, Servicios};
use andexmarket::pantallas::configuracion::actualizacion::Estado;
use andexmarket::pantallas::configuracion::balanza::DialogoBalanza;
use andexmarket::pantallas::configuracion::impresion::DialogoImpresion;
use eframe::egui;
use egui_kittest::Harness;
use serde_json::json;

struct Escena {
    app: App,
    servicios: Servicios,
    impresion: DialogoImpresion,
    balanza: DialogoBalanza,
}

fn escena(servicios: Servicios, global: andexmarket::app::Global) -> Escena {
    Escena {
        app: App::de_prueba(global, servicios.clone()),
        servicios,
        impresion: DialogoImpresion::default(),
        balanza: DialogoBalanza::default(),
    }
}

fn capturar(nombre: &str, escena: Escena, preparar: impl FnOnce(&mut Harness<'_, Escena>)) {
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(
        |ui, e: &mut Escena| {
            e.app.mostrar(ui);
            let mut cx = Contexto { global: e.app.global(), servicios: &e.servicios, egui: ui.ctx().clone() };
            e.impresion.mostrar(ui, &mut cx);
            e.balanza.mostrar(ui, &mut cx);
        },
        escena,
    );
    harness.run_steps(2);
    preparar(&mut harness);
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}

fn publicacion() -> Publicacion {
    Publicacion {
        version: "4.1.0".into(),
        notas: None,
        fecha: None,
        url: "https://github.com/ginopastran/verdu-electron/releases/download/v4.1.0/AndexMarket_4.1.0_x64-setup.exe".into(),
        firma: String::new(),
    }
}

#[test]
fn impresion() {
    let servicios = comun::servicios();
    let _ = std::fs::write(servicios.config.ruta_logo_negocio(), b"logo");
    capturar("72-impresion", escena(servicios, comun::global(Ruta::Carrito)), |harness| harness.state_mut().impresion.abrir());
}

#[test]
fn balanza() {
    let servicios = comun::servicios();
    servicios.config.set(
        "balanza",
        json!({ "modo": "serie", "puerto": "COM3", "baudRate": 9600, "formato": "8N1", "consulta": "enq", "unidadEntera": "gramos" }),
    );
    let mut global = comun::global(Ruta::Carrito);
    global.peso = json!(1245);
    global.estado_balanza = json!({ "conectada": true });
    capturar("73-balanza", escena(servicios, global), |harness| {
        harness.state_mut().balanza.abrir();
        harness.run_steps(1);
        harness.state_mut().balanza.fijar_puertos(vec![("COM3".into(), Some("Prolific".into()))]);
    });
}

#[test]
fn actualizacion_disponible() {
    capturar("41-actualizacion-disponible", escena(comun::servicios(), comun::global(Ruta::Carrito)), |harness| {
        let ctx = harness.ctx.clone();
        harness.state_mut().app.pantallas().actualizacion.ofrecer(&ctx, publicacion());
    });
}

#[test]
fn actualizacion_descargando() {
    capturar("42-actualizacion-descargando", escena(comun::servicios(), comun::global(Ruta::Carrito)), |harness| {
        let ctx = harness.ctx.clone();
        let actualizacion = &mut harness.state_mut().app.pantallas().actualizacion;
        actualizacion.ofrecer(&ctx, publicacion());
        actualizacion.estado = Estado::Descargando;
    });
}

#[test]
fn actualizacion_descargada() {
    capturar("43-actualizacion-descargada", escena(comun::servicios(), comun::global(Ruta::Carrito)), |harness| {
        let ctx = harness.ctx.clone();
        let actualizacion = &mut harness.state_mut().app.pantallas().actualizacion;
        actualizacion.ofrecer(&ctx, publicacion());
        actualizacion.al_descargar(&ctx, Ok(PathBuf::from("AndexMarket_4.1.0_x64-setup.exe")));
    });
}

fn clic_en_el_aviso(harness: &mut Harness<'_, Escena>) {
    let pos = egui::Pos2::new(723.0, 77.0);
    harness.event(egui::Event::PointerMoved(pos));
    harness.step();
    harness.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    harness.step();
    harness.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
    harness.step();
}

fn harness_con_aviso(publicacion: Publicacion, descargada: bool) -> Harness<'static, Escena> {
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(
        |ui, e: &mut Escena| e.app.mostrar(ui),
        escena(comun::servicios(), comun::global(Ruta::Carrito)),
    );
    harness.run_steps(2);
    let ctx = harness.ctx.clone();
    let actualizacion = &mut harness.state_mut().app.pantallas().actualizacion;
    actualizacion.ofrecer(&ctx, publicacion);
    if descargada {
        actualizacion.al_descargar(&ctx, Ok(PathBuf::from("AndexMarket_4.1.0_x64-setup.exe")));
    }
    harness.run_steps(2);
    harness
}

#[test]
fn el_aviso_descargar_inicia_la_descarga() {
    let local = Publicacion { url: "http://127.0.0.1:9/AndexMarket_4.1.0_x64-setup.exe".into(), ..publicacion() };
    let mut harness = harness_con_aviso(local, false);
    clic_en_el_aviso(&mut harness);
    assert_eq!(harness.state_mut().app.pantallas().actualizacion.estado, Estado::Descargando);
}

#[test]
fn el_aviso_reiniciar_ahora_inicia_la_instalacion() {
    let mut harness = harness_con_aviso(publicacion(), true);
    assert!(!harness.state_mut().app.pantallas().actualizacion.reiniciando());
    clic_en_el_aviso(&mut harness);
    assert!(harness.state_mut().app.pantallas().actualizacion.reiniciando());
}
