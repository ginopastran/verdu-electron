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

pub fn nombre_con_logo(negocio: Option<&Value>) -> bool {
    negocio.is_none_or(|n| n["imprimirNombreConLogo"] != false)
}

fn con_impresion(servicios: &Servicios, datos: Value, nombre_con_logo: bool) -> Value {
    let mut mapa = match datos {
        Value::Object(mapa) => mapa,
        _ => Default::default(),
    };
    mapa.insert("anchoPapel".into(), json!(servicios.config.ancho_papel()));
    mapa.insert("impresora".into(), json!(impresora::recurso(&servicios.config)));
    mapa.insert("imprimirNombreConLogo".into(), json!(nombre_con_logo));
    let logo = ruta_logo_negocio(servicios);
    if logo.exists() {
        mapa.insert("logoPath".into(), json!(logo));
    }
    Value::Object(mapa)
}

pub fn imprimir(servicios: &Servicios, comprobante: Comprobante, datos: Value, nombre_con_logo: bool) -> Value {
    let (exito, prefijo) = comprobante.textos();
    let datos = con_impresion(servicios, datos, nombre_con_logo);
    let resultado = impresora::resolver(&servicios.config).and_then(|nombre| {
        let ahora = hora_argentina();
        let bytes = match comprobante {
            Comprobante::Venta => tickets::venta::venta(&datos, ahora),
            Comprobante::Factura => tickets::factura::factura(&datos, ahora),
            Comprobante::Afip => tickets::afip::afip(&datos, ahora),
            Comprobante::Cierre => tickets::cierre::cierre(&datos, ahora),
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

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_nombre_con_logo_se_imprime_salvo_que_el_negocio_diga_que_no() {
        assert!(nombre_con_logo(None));
        assert!(nombre_con_logo(Some(&json!({ "nombre": "La Huerta" }))));
        assert!(nombre_con_logo(Some(&json!({ "imprimirNombreConLogo": true }))));
        assert!(nombre_con_logo(Some(&json!({ "imprimirNombreConLogo": null }))));
        assert!(!nombre_con_logo(Some(&json!({ "imprimirNombreConLogo": false }))));
    }
}
