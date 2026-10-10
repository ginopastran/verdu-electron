use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use minisign_verify::{PublicKey, Signature};
use serde_json::Value;

pub const ENDPOINT: &str = "https://github.com/ginopastran/verdu-electron/releases/latest/download/latest.json";
pub const CLAVE_PUBLICA: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEY5ODg2ODM4RkE2RUJGRTMKUldUanYyNzZPR2lJK1Nsdjk2cUlldEFXZzFNTDRjbW5ic2REdkxpUFY1Mm1nMkREWmp3eElBSmEK";
pub const PLATAFORMA: &str = "windows-x86_64";
pub const ARGUMENTOS_PASIVO: [&str; 2] = ["/P", "/R"];

#[derive(Clone, Debug, PartialEq)]
pub struct Publicacion {
    pub version: String,
    pub notas: Option<String>,
    pub fecha: Option<String>,
    pub url: String,
    pub firma: String,
}

fn version_numerica(version: &str) -> Option<([u64; 3], bool)> {
    let version = version.trim().trim_start_matches('v');
    let version = version.split('+').next()?;
    let (nucleo, previa) = match version.split_once('-') {
        Some((nucleo, _)) => (nucleo, true),
        None => (version, false),
    };
    let mut partes = nucleo.split('.').map(|parte| parte.parse::<u64>().ok());
    let numeros = [partes.next()??, partes.next()??, partes.next()??];
    partes.next().is_none().then_some((numeros, !previa))
}

pub fn es_mas_nueva(candidata: &str, actual: &str) -> bool {
    match (version_numerica(candidata), version_numerica(actual)) {
        (Some(candidata), Some(actual)) => candidata > actual,
        _ => false,
    }
}

fn texto_base64(contenido: &str) -> Result<String, String> {
    let bytes = STANDARD.decode(contenido.trim()).map_err(|e| format!("Base64 inválido: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("Texto inválido: {e}"))
}

pub fn verificar(datos: &[u8], firma: &str, clave_publica: &str) -> Result<(), String> {
    let clave = PublicKey::decode(&texto_base64(clave_publica)?).map_err(|e| format!("Clave pública inválida: {e}"))?;
    let firma = Signature::decode(&texto_base64(firma)?).map_err(|e| format!("Firma inválida: {e}"))?;
    clave
        .verify(datos, &firma, true)
        .map_err(|_| "La firma del instalador no coincide con la clave de AndexMarket".to_string())
}

pub fn leer_manifiesto(manifiesto: &Value, actual: &str) -> Result<Option<Publicacion>, String> {
    let version = manifiesto["version"].as_str().ok_or("latest.json sin versión")?;
    if version_numerica(version).is_none() {
        return Err(format!("Versión inválida en latest.json: {version}"));
    }
    if !es_mas_nueva(version, actual) {
        return Ok(None);
    }
    let plataforma = &manifiesto["platforms"][PLATAFORMA];
    let url = plataforma["url"].as_str().ok_or_else(|| format!("latest.json sin instalador para {PLATAFORMA}"))?;
    let firma = plataforma["signature"].as_str().ok_or_else(|| format!("latest.json sin firma para {PLATAFORMA}"))?;
    Ok(Some(Publicacion {
        version: version.trim_start_matches('v').to_string(),
        notas: manifiesto["notes"].as_str().map(str::to_string),
        fecha: manifiesto["pub_date"].as_str().map(str::to_string),
        url: url.to_string(),
        firma: firma.to_string(),
    }))
}

fn cliente(espera: Duration) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(espera)
        .user_agent(concat!("AndexMarket-actualizador/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())
}

fn descripcion(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "Timeout verificando actualizaciones".to_string()
    } else {
        error.to_string()
    }
}

pub fn buscar(actual: &str) -> Result<Option<Publicacion>, String> {
    let respuesta = cliente(Duration::from_secs(30))?.get(ENDPOINT).send().map_err(descripcion)?;
    if !respuesta.status().is_success() {
        return Err(format!("No se pudo leer latest.json (HTTP {})", respuesta.status().as_u16()));
    }
    let manifiesto: Value = serde_json::from_slice(&respuesta.bytes().map_err(descripcion)?).map_err(|e| e.to_string())?;
    leer_manifiesto(&manifiesto, actual)
}

pub fn nombre_instalador(publicacion: &Publicacion) -> String {
    let nombre = publicacion.url.rsplit('/').next().unwrap_or_default();
    let limpio: String = nombre.chars().filter(|c| c.is_ascii_alphanumeric() || "._-".contains(*c)).collect();
    if limpio.is_empty() || limpio.starts_with('.') {
        format!("AndexMarket_{}_x64-setup.exe", publicacion.version)
    } else {
        limpio
    }
}

pub fn descargar(publicacion: &Publicacion) -> Result<PathBuf, String> {
    let respuesta = cliente(Duration::from_secs(15 * 60))?.get(&publicacion.url).send().map_err(|e| e.to_string())?;
    if !respuesta.status().is_success() {
        return Err(format!("HTTP {}", respuesta.status().as_u16()));
    }
    let datos = respuesta.bytes().map_err(|e| e.to_string())?;
    verificar(&datos, &publicacion.firma, CLAVE_PUBLICA)?;
    let ruta = std::env::temp_dir().join(nombre_instalador(publicacion));
    std::fs::write(&ruta, &datos).map_err(|e| e.to_string())?;
    Ok(ruta)
}

pub fn instalar(ruta: &Path, firma: &str) -> Result<(), String> {
    let datos = std::fs::read(ruta).map_err(|e| e.to_string())?;
    verificar(&datos, firma, CLAVE_PUBLICA)?;
    Command::new(ruta).args(ARGUMENTOS_PASIVO).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
