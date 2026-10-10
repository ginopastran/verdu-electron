#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actualizador;
mod balanza;
mod config;
mod impresora;
mod ipc;

use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
use windows::core::Interface;

const APP_ID: &str = "b2fa850b9a1782595da81d0699892e93a3f29f9d5b0fd74ef4ede03f05658942";

const ARGUMENTOS_WEBVIEW: &str =
    "--disable-web-security --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

fn sin_atajos_de_navegador(ventana: &WebviewWindow) -> tauri::Result<()> {
    ventana.with_webview(|webview| unsafe {
        let ajustes = webview
            .controller()
            .CoreWebView2()
            .and_then(|nucleo| nucleo.Settings())
            .and_then(|ajustes| ajustes.cast::<ICoreWebView2Settings3>());
        if let Ok(ajustes) = ajustes {
            let _ = ajustes.SetAreBrowserAcceleratorKeysEnabled(false);
        }
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(balanza::Balanza::new())
        .manage(actualizador::Actualizacion::default())
        .setup(|app| {
            app.manage(config::Config::cargar(app.path().app_config_dir()?));
            let argumentos = serde_json::to_string(&std::env::args().collect::<Vec<_>>())?;
            let ventana = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("AndexMarket")
                .inner_size(1200.0, 800.0)
                .additional_browser_args(ARGUMENTOS_WEBVIEW)
                .initialization_script(format!("window.__ARGV__ = {argumentos};"))
                .build()?;
            sin_atajos_de_navegador(&ventana)?;
            balanza::iniciar(app.handle());
            actualizador::programar(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ipc::ipc])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar AndexMarket");
}
