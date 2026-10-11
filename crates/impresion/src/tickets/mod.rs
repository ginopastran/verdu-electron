pub mod afip;
pub mod cierre;
pub mod factura;
mod formato;
pub mod venta;

use std::path::Path;

use chrono::{DateTime, FixedOffset, Utc};
use serde_json::Value;

use crate::escpos::{cargar_logo, Imagen};

pub type Ahora = DateTime<FixedOffset>;

pub fn hora_argentina() -> Ahora {
    Utc::now().with_timezone(&FixedOffset::west_opt(3 * 3600).unwrap())
}

pub fn logo(datos: &Value, defecto: &Path, ancho: usize) -> Option<Imagen> {
    let propio = datos["logoPath"].as_str().filter(|ruta| !ruta.is_empty()).map(Path::new);
    let ruta = [propio, Some(defecto)]
        .into_iter()
        .flatten()
        .find(|ruta| std::fs::metadata(ruta).is_ok_and(|m| m.len() > 0))?;
    cargar_logo(ruta, ancho, propio == Some(ruta))
}

pub fn primero<'a, const N: usize>(valores: [&'a Value; N]) -> &'a Value {
    valores.into_iter().find(|valor| !valor.is_null()).unwrap_or(&Value::Null)
}

pub fn vacio(valor: &Value) -> bool {
    match valor {
        Value::Null => true,
        Value::Bool(b) => !b,
        Value::Number(n) => n.as_f64() == Some(0.0),
        Value::String(s) => s.is_empty() || s == "0",
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
    }
}

pub fn es_arreglo(valor: &Value) -> bool {
    valor.is_array() || valor.is_object()
}

pub fn elementos(valor: &Value) -> Vec<&Value> {
    match valor {
        Value::Array(a) => a.iter().collect(),
        Value::Object(o) => o.values().collect(),
        _ => Vec::new(),
    }
}

pub fn flotante(valor: &Value) -> f64 {
    match valor {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => prefijo_numerico(s),
        Value::Bool(b) => *b as u8 as f64,
        Value::Array(a) => !a.is_empty() as u8 as f64,
        Value::Object(o) => !o.is_empty() as u8 as f64,
        Value::Null => 0.0,
    }
}

fn prefijo_numerico(texto: &str) -> f64 {
    let texto = texto.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let bytes = texto.as_bytes();
    let digitos = |desde: usize| bytes[desde..].iter().take_while(|b| b.is_ascii_digit()).count();
    let mut fin = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let enteros = digitos(fin);
    fin += enteros;
    let mut decimales = 0;
    if bytes.get(fin) == Some(&b'.') {
        decimales = digitos(fin + 1);
        if enteros + decimales > 0 {
            fin += 1 + decimales;
        }
    }
    if enteros + decimales == 0 {
        return 0.0;
    }
    if matches!(bytes.get(fin), Some(b'e' | b'E')) {
        let signo = usize::from(matches!(bytes.get(fin + 1), Some(b'+' | b'-')));
        let exponente = digitos(fin + 1 + signo);
        if exponente > 0 {
            fin += 1 + signo + exponente;
        }
    }
    texto[..fin].parse().unwrap_or(0.0)
}

pub fn texto(valor: &Value) -> String {
    match valor {
        Value::Null | Value::Bool(false) => String::new(),
        Value::Bool(true) => "1".into(),
        Value::String(s) => s.clone(),
        Value::Number(n) if n.is_f64() => flotante_a_texto(n.as_f64().unwrap_or(0.0)),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => "Array".into(),
    }
}

fn flotante_a_texto(valor: f64) -> String {
    if !valor.is_finite() {
        return if valor.is_nan() { "NAN" } else if valor > 0.0 { "INF" } else { "-INF" }.into();
    }
    let cientifico = format!("{valor:.13e}");
    let (mantisa, exponente) = cientifico.split_once('e').unwrap();
    let exponente: i32 = exponente.parse().unwrap();
    if (-4..14).contains(&exponente) {
        return cientifico.parse::<f64>().unwrap().to_string();
    }
    let mantisa = mantisa.trim_end_matches('0');
    let mantisa = if mantisa.ends_with('.') { format!("{mantisa}0") } else { mantisa.to_string() };
    format!("{mantisa}E{}{}", if exponente < 0 { '-' } else { '+' }, exponente.abs())
}

pub fn mayusculas(valor: &Value) -> String {
    texto(valor).to_ascii_uppercase()
}

pub fn rellenar_izquierda(valor: &str, ancho: usize) -> String {
    format!("{}{valor}", " ".repeat(ancho.saturating_sub(valor.len())))
}

pub fn largo(valor: &str) -> usize {
    valor.chars().count()
}

pub fn rellenar_derecha_mb(valor: &str, ancho: usize) -> String {
    format!("{valor}{}", " ".repeat(ancho.saturating_sub(largo(valor))))
}

pub fn rellenar_izquierda_mb(valor: &str, ancho: usize) -> String {
    format!("{}{valor}", " ".repeat(ancho.saturating_sub(largo(valor))))
}

fn potencia10(exponente: i32) -> f64 {
    if (0..=22).contains(&exponente) {
        10f64.powi(exponente)
    } else {
        10f64.powf(exponente as f64)
    }
}

fn redondear_mitad_arriba(valor: f64) -> f64 {
    if valor >= 0.0 {
        (valor + 0.5).floor()
    } else {
        (valor - 0.5).ceil()
    }
}

pub fn redondear(valor: f64, lugares: i32) -> f64 {
    if !valor.is_finite() || valor == 0.0 {
        return valor;
    }
    let precision = 14 - valor.abs().log10().floor() as i32;
    let factor = potencia10(lugares.abs());
    let mut temporal;
    if precision > lugares && precision - 15 < lugares {
        let escala = potencia10(precision.abs());
        temporal = redondear_mitad_arriba(if precision >= 0 { valor * escala } else { valor / escala });
        temporal /= potencia10((lugares - precision).max(-60).abs());
    } else {
        temporal = if lugares >= 0 { valor * factor } else { valor / factor };
        if temporal.abs() >= 1e15 {
            return valor;
        }
    }
    temporal = redondear_mitad_arriba(temporal);
    if lugares > 0 {
        temporal / factor
    } else {
        temporal * factor
    }
}

pub fn numero(valor: f64, decimales: i32) -> String {
    let redondeado = redondear(valor.abs(), decimales);
    let decimales = decimales.max(0) as usize;
    let crudo = format!("{redondeado:.decimales$}");
    let (entero, fraccion) = match crudo.split_once('.') {
        Some((entero, fraccion)) => (entero, Some(fraccion)),
        None => (crudo.as_str(), None),
    };
    let mut resultado = String::new();
    if valor < 0.0 && redondeado != 0.0 {
        resultado.push('-');
    }
    for (posicion, digito) in entero.chars().enumerate() {
        if posicion > 0 && (entero.len() - posicion) % 3 == 0 {
            resultado.push(',');
        }
        resultado.push(digito);
    }
    if let Some(fraccion) = fraccion {
        resultado.push('.');
        resultado.push_str(fraccion);
    }
    resultado
}

#[cfg(test)]
pub mod pruebas {
    use std::path::PathBuf;

    use chrono::{FixedOffset, TimeZone};

    use super::*;

    pub fn ruta(relativa: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("golden").join(relativa)
    }

    pub fn ahora_fija() -> Ahora {
        FixedOffset::west_opt(3 * 3600).unwrap().with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()
    }

    pub fn logo_defecto() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/imagenes/andextech-logo.png")
    }

    pub fn comparar(nombre: &str, render: impl Fn(&Value) -> Vec<u8>) {
        let carpeta = ruta(nombre);
        let mut casos = 0;
        for entrada in std::fs::read_dir(&carpeta).unwrap() {
            let fixture = entrada.unwrap().path();
            if fixture.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let datos: Value = serde_json::from_str(&std::fs::read_to_string(&fixture).unwrap()).unwrap();
            let esperado = std::fs::read(fixture.with_extension("bin")).unwrap();
            let obtenido = render(&datos);
            let es_jpeg = fixture.file_stem().is_some_and(|nombre| nombre.to_string_lossy().ends_with("_jpeg"));
            if es_jpeg && obtenido.len() == esperado.len() {
                let bits: u32 = obtenido.iter().zip(&esperado).map(|(a, b)| (a ^ b).count_ones()).sum();
                assert!(bits as usize * 1000 < esperado.len() * 8, "{}: {bits} bits distintos", fixture.display());
                casos += 1;
                continue;
            }
            if obtenido != esperado {
                let diferencia = obtenido.iter().zip(&esperado).position(|(a, b)| a != b).unwrap_or(obtenido.len().min(esperado.len()));
                let desde = diferencia.saturating_sub(40);
                panic!(
                    "{} difiere en el byte {diferencia} (largo {} vs {})\nobtenido: {:?}\nesperado: {:?}",
                    fixture.display(),
                    obtenido.len(),
                    esperado.len(),
                    String::from_utf8_lossy(&obtenido[desde..(diferencia + 40).min(obtenido.len())]),
                    String::from_utf8_lossy(&esperado[desde..(diferencia + 40).min(esperado.len())]),
                );
            }
            casos += 1;
        }
        assert!(casos > 0, "sin fixtures en {}", carpeta.display());
    }

    #[test]
    fn number_format_igual_que_php() {
        let casos = std::fs::read_to_string(ruta("number_format.txt")).unwrap();
        for linea in casos.lines() {
            let partes: Vec<&str> = linea.split('|').collect();
            let valor: f64 = partes[0].parse().unwrap();
            assert_eq!(numero(valor, partes[1].parse().unwrap()), partes[2], "{linea}");
        }
    }

    #[test]
    fn conversiones_como_php() {
        let casos = [(1e14, "1.0E+14"), (1e13, "10000000000000"), (123456789012345.0, "1.2345678901234E+14"), (0.1 + 0.2, "0.3"), (1.0 / 3.0, "0.33333333333333"), (12.0, "12"), (-0.0, "-0"), (1.5e-5, "1.5E-5"), (0.0001, "0.0001"), (99999999999999.9, "1.0E+14")];
        for (valor, esperado) in casos {
            assert_eq!(texto(&serde_json::json!(valor)), esperado);
        }
        assert_eq!(flotante(&serde_json::json!(" 12.5kg")), 12.5);
        assert_eq!(flotante(&serde_json::json!("1e3")), 1000.0);
        assert_eq!(flotante(&serde_json::json!("abc")), 0.0);
        assert_eq!(mayusculas(&serde_json::json!("Verdulería")), "VERDULERíA");
    }
}
