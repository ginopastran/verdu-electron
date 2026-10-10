use std::path::PathBuf;

use base64::Engine;
use impresion::tickets::{self, hora_argentina};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::balanza::{self, Balanza};
use crate::config::Config;
use crate::{actualizador, impresora, APP_ID};

#[derive(Clone, Copy)]
enum Comprobante {
    Venta,
    Factura,
    Afip,
    Cierre,
}

impl Comprobante {
    fn textos(self) -> (&'static str, &'static str) {
        match self {
            Comprobante::Venta => ("Ticket impreso correctamente", "Error de impresión"),
            Comprobante::Factura => ("Ticket de factura impreso correctamente", "Error de impresión de factura"),
            Comprobante::Afip => ("Ticket AFIP impreso correctamente", "Error de impresión AFIP"),
            Comprobante::Cierre => ("Ticket de cierre impreso correctamente", "Error de impresión"),
        }
    }

    fn generar(self, datos: &Value, logo: &std::path::Path) -> Vec<u8> {
        let ahora = hora_argentina();
        match self {
            Comprobante::Venta => tickets::venta::venta(datos, logo, ahora),
            Comprobante::Factura => tickets::factura::factura(datos, logo, ahora),
            Comprobante::Afip => tickets::afip::afip(datos, logo, ahora),
            Comprobante::Cierre => tickets::cierre::cierre(datos, logo, ahora),
        }
    }
}

fn con_impresion(config: &Config, datos: Value) -> Value {
    let mut mapa = match datos {
        Value::Object(mapa) => mapa,
        _ => Default::default(),
    };
    mapa.insert("anchoPapel".into(), json!(config.ancho_papel()));
    mapa.insert("impresora".into(), json!(impresora::recurso(config)));
    let logo = config.ruta_logo_negocio();
    if logo.exists() {
        mapa.insert("logoPath".into(), json!(logo));
    }
    Value::Object(mapa)
}

async fn imprimir(app: &AppHandle, comprobante: Comprobante, datos: Value) -> Value {
    let (exito, prefijo) = comprobante.textos();
    let config = app.state::<Config>();
    let datos = con_impresion(&config, datos);
    let destino = impresora::resolver(&config);
    let logo = app.path().resource_dir().map(|carpeta| carpeta.join("logo.png")).unwrap_or_default();
    let resultado = tauri::async_runtime::spawn_blocking(move || {
        let nombre = destino?;
        let bytes = comprobante.generar(&datos, &logo);
        if bytes.is_empty() {
            return Err("No se pudo armar el ticket con los datos recibidos".to_string());
        }
        impresora::imprimir(&nombre, "AndexMarket", &bytes)
    })
    .await
    .unwrap_or_else(|error| Err(error.to_string()));
    match resultado {
        Ok(()) => json!({ "success": true, "printerError": null, "message": exito }),
        Err(error) => json!({ "success": false, "printerError": error, "message": format!("{prefijo}: {error}") }),
    }
}

fn guardar_logo(config: &Config, base64: &Value) -> Result<Value, String> {
    let ruta: PathBuf = config.ruta_logo_negocio();
    let Some(contenido) = base64.as_str().filter(|c| !c.is_empty()) else {
        if ruta.exists() {
            std::fs::remove_file(&ruta).map_err(|e| e.to_string())?;
        }
        return Ok(json!({ "ok": true, "logo": false }));
    };
    let bytes = base64::engine::general_purpose::STANDARD.decode(contenido.trim()).map_err(|e| e.to_string())?;
    std::fs::write(&ruta, bytes).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "logo": true }))
}

#[tauri::command]
pub async fn ipc(app: AppHandle, canal: String, args: Vec<Value>) -> Result<Value, String> {
    let arg = |indice: usize| args.get(indice).cloned().unwrap_or(Value::Null);
    let clave = arg(0).as_str().unwrap_or_default().to_string();
    let config = app.state::<Config>();
    let balanza = app.state::<Balanza>();
    Ok(match canal.as_str() {
        "store-get" => config.get(&clave),
        "store-set" => {
            config.set(&clave, arg(1));
            json!(true)
        }
        "store-delete" => {
            config.delete(&clave);
            json!(true)
        }
        "store-has" => json!(config.has(&clave)),
        "read-peso" => balanza::leer_peso(&config, &balanza),
        "read-weight" => balanza::leer_peso_simple(),
        "balanza-config-get" => json!({
            "config": balanza::config_balanza(&config),
            "estado": balanza.estado(),
            "peso": balanza.peso_serie(),
        }),
        "balanza-config-set" => {
            config.set("balanza", arg(0));
            balanza.reiniciar_lectura();
            balanza::iniciar(&app);
            balanza::config_balanza(&config)
        }
        "balanza-puertos" => balanza::puertos(),
        "logo-negocio-guardar" => guardar_logo(&config, &arg(0))?,
        "impresion-config-get" => json!({
            "anchoPapel": config.ancho_papel(),
            "tieneLogo": config.ruta_logo_negocio().exists(),
            "impresora": config.get("impresoraWindows"),
            "recurso": impresora::recurso(&config),
        }),
        "impresion-config-set" => {
            config.set("anchoPapel", json!(if arg(0)["anchoPapel"] == 58 { 58 } else { 80 }));
            json!({ "ok": true })
        }
        "impresora-elegir" => impresora::elegir(&config, &arg(0)),
        "get-available-printers" => impresora::listar(),
        "print-ticket" => imprimir(&app, Comprobante::Venta, arg(0)).await,
        "print-factura-ticket" => imprimir(&app, Comprobante::Factura, arg(0)).await,
        "print-afip-ticket" => imprimir(&app, Comprobante::Afip, arg(0)).await,
        "print-closing" => imprimir(&app, Comprobante::Cierre, arg(0)).await,
        "check-for-updates" => actualizador::buscar(&app).await,
        "download-update" => actualizador::descargar(&app).await,
        "install-update" => app.restart(),
        "show-app-id" => {
            app.dialog()
                .message(format!("APP_ID actual: {APP_ID}"))
                .title("APP ID")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
            json!(APP_ID)
        }
        "toggle-devtools" => {
            if let Some(ventana) = app.get_webview_window("main") {
                if ventana.is_devtools_open() {
                    ventana.close_devtools();
                } else {
                    ventana.open_devtools();
                }
            }
            Value::Null
        }
        otro => return Err(format!("Canal desconocido: {otro}")),
    })
}
