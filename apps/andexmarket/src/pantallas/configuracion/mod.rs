pub mod actualizacion;
pub mod balanza;
pub mod impresion;

use serde_json::Value;

pub fn texto_valor(valor: &Value) -> String {
    match valor {
        Value::String(texto) => texto.clone(),
        Value::Null => String::new(),
        otro => otro.to_string(),
    }
}
