use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
pub struct Actualizacion {
    pendiente: Mutex<Option<Update>>,
    buscando: AtomicBool,
}

async fn consultar(app: &AppHandle) -> Result<Option<Update>, String> {
    app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())
}

pub async fn buscar(app: &AppHandle) -> Value {
    let estado = app.state::<Actualizacion>();
    if estado.buscando.swap(true, Ordering::SeqCst) {
        return json!({ "error": "Ya se está verificando actualizaciones" });
    }
    let resultado = consultar(app).await;
    estado.buscando.store(false, Ordering::SeqCst);
    match resultado {
        Ok(Some(actualizacion)) => {
            let info = json!({
                "version": actualizacion.version,
                "releaseDate": actualizacion.date.map(|fecha| fecha.to_string()),
                "releaseNotes": actualizacion.body,
            });
            *estado.pendiente.lock().unwrap() = Some(actualizacion);
            json!({ "available": true, "info": info })
        }
        Ok(None) => json!({ "available": false, "info": { "message": "Ya tienes la última versión" } }),
        Err(error) => json!({ "error": error }),
    }
}

pub async fn descargar(app: &AppHandle) -> Value {
    let Some(actualizacion) = app.state::<Actualizacion>().pendiente.lock().unwrap().take() else {
        return json!({ "error": "Primero hay que buscar la actualización" });
    };
    match actualizacion.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => json!({ "success": true }),
        Err(error) => json!({ "error": error.to_string() }),
    }
}

fn ofrecer(app: &AppHandle) {
    let Ok(Some(actualizacion)) = tauri::async_runtime::block_on(consultar(app)) else {
        return;
    };
    let acepta = app
        .dialog()
        .message(format!(
            "Nueva versión {} disponible\n\n¿Deseas descargar e instalar la actualización ahora?",
            actualizacion.version
        ))
        .title("Actualización disponible")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom("Descargar".into(), "Más tarde".into()))
        .blocking_show();
    if acepta {
        let _ = tauri::async_runtime::block_on(actualizacion.download_and_install(|_, _| {}, || {}));
    }
}

pub fn programar(app: AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(3));
        loop {
            ofrecer(&app);
            std::thread::sleep(Duration::from_secs(10 * 60));
        }
    });
}
