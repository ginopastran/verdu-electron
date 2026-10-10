use std::path::PathBuf;

use hardware::impresora;
use impresion::tickets::{self, hora_argentina};
use serde_json::{json, Value};

use crate::app::Servicios;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Comprobante {
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
}

pub fn logo_defecto(servicios: &Servicios) -> PathBuf {
    let ruta = servicios.carpeta.join("logo-andextech.png");
    if !ruta.exists() {
        let _ = std::fs::write(&ruta, include_bytes!("../../../assets/imagenes/andextech-logo.png"));
    }
    ruta
}

pub fn ruta_logo_negocio(servicios: &Servicios) -> PathBuf {
    servicios.config.ruta_logo_negocio()
}

pub fn guardar_logo_negocio(servicios: &Servicios, bytes: Option<&[u8]>) -> Result<bool, String> {
    let ruta = ruta_logo_negocio(servicios);
    match bytes {
        Some(bytes) => std::fs::write(&ruta, bytes).map(|_| true).map_err(|e| e.to_string()),
        None => {
            if ruta.exists() {
                std::fs::remove_file(&ruta).map_err(|e| e.to_string())?;
            }
            Ok(false)
        }
    }
}

fn con_impresion(servicios: &Servicios, datos: Value) -> Value {
    let mut mapa = match datos {
        Value::Object(mapa) => mapa,
        _ => Default::default(),
    };
    mapa.insert("anchoPapel".into(), json!(servicios.config.ancho_papel()));
    mapa.insert("impresora".into(), json!(impresora::recurso(&servicios.config)));
    let logo = ruta_logo_negocio(servicios);
    if logo.exists() {
        mapa.insert("logoPath".into(), json!(logo));
    }
    Value::Object(mapa)
}

pub fn imprimir(servicios: &Servicios, comprobante: Comprobante, datos: Value) -> Value {
    let (exito, prefijo) = comprobante.textos();
    let datos = con_impresion(servicios, datos);
    let logo = logo_defecto(servicios);
    let resultado = impresora::resolver(&servicios.config).and_then(|nombre| {
        let ahora = hora_argentina();
        let bytes = match comprobante {
            Comprobante::Venta => tickets::venta::venta(&datos, &logo, ahora),
            Comprobante::Factura => tickets::factura::factura(&datos, &logo, ahora),
            Comprobante::Afip => tickets::afip::afip(&datos, &logo, ahora),
            Comprobante::Cierre => tickets::cierre::cierre(&datos, &logo, ahora),
        };
        if bytes.is_empty() {
            return Err("No se pudo armar el ticket con los datos recibidos".to_string());
        }
        impresora::imprimir(&nombre, "AndexMarket", &bytes)
    });
    match resultado {
        Ok(()) => json!({ "success": true, "printerError": null, "message": exito }),
        Err(error) => json!({ "success": false, "printerError": error, "message": format!("{prefijo}: {error}") }),
    }
}
