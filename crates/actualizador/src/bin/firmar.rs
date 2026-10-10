use std::io::Cursor;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use minisign::SecretKeyBox;

fn firmar(ruta_clave: &str, ruta_archivo: &str) -> Result<String, String> {
    let clave_base64 = std::fs::read_to_string(ruta_clave).map_err(|e| format!("No se pudo leer la clave {ruta_clave}: {e}"))?;
    let texto_clave = String::from_utf8(STANDARD.decode(clave_base64.trim()).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let secreta = SecretKeyBox::from_string(&texto_clave)
        .and_then(|caja| caja.into_secret_key(Some(String::new())))
        .map_err(|e| format!("Clave privada inválida: {e}"))?;
    let datos = std::fs::read(ruta_archivo).map_err(|e| format!("No se pudo leer {ruta_archivo}: {e}"))?;
    let nombre = Path::new(ruta_archivo).file_name().and_then(|n| n.to_str()).unwrap_or(ruta_archivo);
    let marca = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs();
    let comentario = format!("timestamp:{marca}\tfile:{nombre}");
    let caja = minisign::sign(None, &secreta, Cursor::new(&datos), Some(&comentario), Some("signature from tauri secret key"))
        .map_err(|e| e.to_string())?;
    let firma = STANDARD.encode(caja.to_string());
    actualizador::verificar(&datos, &firma, actualizador::CLAVE_PUBLICA)
        .map_err(|e| format!("La clave privada no corresponde a la clave pública de la app: {e}"))?;
    Ok(firma)
}

fn main() {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let [clave, archivo] = argumentos.as_slice() else {
        eprintln!("Uso: firmar <clave privada> <archivo>");
        std::process::exit(2);
    };
    match firmar(clave, archivo).and_then(|firma| std::fs::write(format!("{archivo}.sig"), firma).map_err(|e| e.to_string())) {
        Ok(()) => println!("Firmado: {archivo}.sig"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
