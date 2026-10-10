use serde_json::{json, Value};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Graphics::Printing::{
    ClosePrinter, EndDocPrinter, EndPagePrinter, EnumPrintersW, GetDefaultPrinterW, OpenPrinterW, StartDocPrinterW,
    StartPagePrinter, WritePrinter, DOC_INFO_1W, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL, PRINTER_HANDLE,
    PRINTER_INFO_2W,
};

use crate::config::{Config, IMPRESORA_DEFAULT};

struct Impresora {
    nombre: String,
    recurso: Option<String>,
    descripcion: String,
    estado: u32,
}

fn texto_ancho(puntero: PWSTR) -> Option<String> {
    if puntero.is_null() {
        return None;
    }
    unsafe { puntero.to_string().ok() }.filter(|texto| !texto.is_empty())
}

fn enumerar() -> Vec<Impresora> {
    let banderas = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
    let (mut necesarios, mut cantidad) = (0u32, 0u32);
    unsafe {
        let _ = EnumPrintersW(banderas, PCWSTR::null(), 2, None, &mut necesarios, &mut cantidad);
        if necesarios == 0 {
            return Vec::new();
        }
        let mut memoria = vec![0u64; (necesarios as usize).div_ceil(8)];
        let buffer = std::slice::from_raw_parts_mut(memoria.as_mut_ptr().cast::<u8>(), necesarios as usize);
        if EnumPrintersW(banderas, PCWSTR::null(), 2, Some(buffer), &mut necesarios, &mut cantidad).is_err() {
            return Vec::new();
        }
        std::slice::from_raw_parts(memoria.as_ptr().cast::<PRINTER_INFO_2W>(), cantidad as usize)
            .iter()
            .filter_map(|info| {
                Some(Impresora {
                    nombre: texto_ancho(info.pPrinterName)?,
                    recurso: texto_ancho(info.pShareName),
                    descripcion: texto_ancho(info.pComment).or_else(|| texto_ancho(info.pDriverName)).unwrap_or_default(),
                    estado: info.Status,
                })
            })
            .collect()
    }
}

fn predeterminada() -> Option<String> {
    let mut largo = 0u32;
    unsafe {
        let _ = GetDefaultPrinterW(None, &mut largo);
        let mut buffer = vec![0u16; largo as usize];
        GetDefaultPrinterW(Some(PWSTR(buffer.as_mut_ptr())), &mut largo).as_bool().then(|| {
            String::from_utf16_lossy(&buffer[..(largo as usize).saturating_sub(1)])
        })
    }
}

pub fn listar() -> Value {
    let predeterminada = predeterminada();
    Value::Array(
        enumerar()
            .into_iter()
            .map(|impresora| {
                let estado_texto = match impresora.estado {
                    0 => "Disponible",
                    1 => "Imprimiendo",
                    2 => "Error",
                    3 => "No disponible",
                    _ => "Estado desconocido",
                };
                json!({
                    "name": impresora.nombre,
                    "displayName": impresora.nombre,
                    "description": impresora.descripcion,
                    "status": impresora.estado,
                    "isDefault": predeterminada.as_deref() == Some(impresora.nombre.as_str()),
                    "statusText": estado_texto,
                })
            })
            .collect(),
    )
}

pub fn recurso(config: &Config) -> String {
    config
        .get("impresoraWindows")
        .as_str()
        .or(config.get("impresora").as_str())
        .unwrap_or(IMPRESORA_DEFAULT)
        .to_string()
}

pub fn resolver(config: &Config) -> Result<String, String> {
    if let Some(nombre) = config.get("impresoraWindows").as_str() {
        return Ok(nombre.to_string());
    }
    let buscada = recurso(config);
    let impresoras = enumerar();
    impresoras
        .iter()
        .find(|i| i.recurso.as_deref().is_some_and(|r| r.eq_ignore_ascii_case(&buscada)))
        .or_else(|| impresoras.iter().find(|i| i.nombre.eq_ignore_ascii_case(&buscada)))
        .map(|impresora| impresora.nombre.clone())
        .ok_or_else(|| format!("No se encontró la impresora «{buscada}». Elegila en la configuración de impresión."))
}

pub fn elegir(config: &Config, nombre: &Value) -> Value {
    let Some(nombre) = nombre.as_str().filter(|n| !n.is_empty()) else {
        config.delete("impresora");
        config.delete("impresoraWindows");
        return json!({ "ok": true, "recurso": IMPRESORA_DEFAULT });
    };
    if !enumerar().iter().any(|impresora| impresora.nombre == nombre) {
        return json!({ "ok": false, "error": format!("No se encontró la impresora «{nombre}»") });
    }
    config.set("impresoraWindows", json!(nombre));
    json!({ "ok": true, "recurso": nombre })
}

pub fn imprimir(nombre: &str, documento: &str, bytes: &[u8]) -> Result<(), String> {
    let nombre_ancho = HSTRING::from(nombre);
    let documento_ancho = HSTRING::from(documento);
    let tipo = HSTRING::from("RAW");
    unsafe {
        let mut impresora = PRINTER_HANDLE::default();
        OpenPrinterW(&nombre_ancho, &mut impresora, None)
            .map_err(|error| format!("No se pudo abrir la impresora «{nombre}»: {}", error.message()))?;
        let info = DOC_INFO_1W {
            pDocName: PWSTR(documento_ancho.as_ptr() as *mut u16),
            pOutputFile: PWSTR::null(),
            pDatatype: PWSTR(tipo.as_ptr() as *mut u16),
        };
        let resultado = if StartDocPrinterW(impresora, 1, &info) == 0 {
            Err(format!("La impresora «{nombre}» rechazó el trabajo: {}", windows::core::Error::from_thread().message()))
        } else {
            let mut escritos = 0u32;
            let _ = StartPagePrinter(impresora);
            let escrito = WritePrinter(impresora, bytes.as_ptr().cast(), bytes.len() as u32, &mut escritos).as_bool();
            let _ = EndPagePrinter(impresora);
            let _ = EndDocPrinter(impresora);
            if escrito && escritos as usize == bytes.len() {
                Ok(())
            } else {
                Err(format!("No se pudo enviar el ticket a «{nombre}»"))
            }
        };
        let _ = ClosePrinter(impresora);
        resultado
    }
}
