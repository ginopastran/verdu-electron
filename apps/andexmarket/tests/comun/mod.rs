#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;

use andexmarket::app::{App, Global, Ruta, Servicios};
use egui_kittest::Harness;
use hardware::balanza::Balanza;
use hardware::config::Config;
use nucleo::api::Cliente;
use nucleo::carrito::Carritos;
use serde_json::{json, Value};

pub fn servicios() -> Servicios {
    let carpeta = std::env::temp_dir().join(format!("andexmarket-prueba-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&carpeta);
    Servicios {
        config: Arc::new(Config::cargar(carpeta.clone())),
        balanza: Balanza::new(|_| {}),
        api: Cliente::new("http://127.0.0.1:9"),
        carpeta,
    }
}

pub fn negocio() -> Value {
    json!({
        "id": 4,
        "nombre": "Verdulería Don José",
        "sistemaPago": "redondeo",
        "afipHabilitado": true,
        "facturacionHabilitada": true,
        "ivaIncluidoEnPrecios": true
    })
}

pub fn global(ruta: Ruta) -> Global {
    Global {
        ruta,
        admin: Some(json!({ "id": 1, "nombre": "Andres Pravata", "email": "admin@andex.tech", "businessId": 4, "businessName": "Andex Market" })),
        negocio: Some(negocio()),
        negocio_cargando: false,
        usuario: Some(json!({ "id": 7, "nombre": "Andres", "email": "andres@andex.tech", "sucursalId": 1, "role": "VENDEDOR", "permisos": { "pesoManualEnabled": true, "cierreDeCajaEnabled": true } })),
        carritos: Carritos::default(),
        peso: json!(1250),
        estado_balanza: json!({ "conectada": false }),
        en_linea: true,
    }
}

pub fn carpeta_capturas() -> PathBuf {
    let carpeta = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/capturas");
    let _ = std::fs::create_dir_all(&carpeta);
    carpeta
}

pub fn capturar(nombre: &str, global: Global, preparar: impl FnOnce(&mut App)) {
    let mut app = App::de_prueba(global, servicios());
    preparar(&mut app);
    let mut harness = Harness::builder()
        .with_size([1200.0, 800.0])
        .wgpu()
        .build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}

pub fn capturar_con(nombre: &str, global: Global, preparar: impl FnOnce(&mut App), pasos: impl FnOnce(&mut Harness<'_, App>)) {
    let mut app = App::de_prueba(global, servicios());
    preparar(&mut app);
    let mut harness = Harness::builder()
        .with_size([1200.0, 800.0])
        .wgpu()
        .build_ui_state(|ui, app: &mut App| app.mostrar(ui), app);
    harness.run_steps(2);
    pasos(&mut harness);
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(carpeta_capturas().join(format!("{nombre}.png"))).expect("guardar captura");
}
