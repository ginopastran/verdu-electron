use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::Value;

use crate::entorno;

#[derive(Clone, Debug)]
pub struct Respuesta {
    pub estado: u16,
    pub cuerpo: Value,
}

impl Respuesta {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.estado)
    }

    pub fn error(&self) -> String {
        self.cuerpo["error"]
            .as_str()
            .or(self.cuerpo["message"].as_str())
            .map(str::to_string)
            .unwrap_or_else(|| format!("HTTP error! status: {}", self.estado))
    }
}

#[derive(Clone, Debug)]
pub struct ErrorApi {
    pub estado: u16,
    pub mensaje: String,
}

impl std::fmt::Display for ErrorApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.mensaje)
    }
}

#[derive(Clone)]
pub struct Cliente {
    http: Client,
    base: String,
}

impl Default for Cliente {
    fn default() -> Self {
        Self::new(entorno::API_URL)
    }
}

impl Cliente {
    pub fn new(base: &str) -> Self {
        let http = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("cliente HTTP");
        Cliente { http, base: base.trim_end_matches('/').to_string() }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn url(&self, ruta: &str) -> String {
        if ruta.starts_with("http") {
            ruta.to_string()
        } else {
            format!("{}{}", self.base, ruta)
        }
    }

    pub fn pedir(
        &self,
        metodo: &str,
        ruta: &str,
        cuerpo: Option<&Value>,
        encabezados: &[(&str, &str)],
    ) -> Result<Respuesta, ErrorApi> {
        let metodo = Method::from_bytes(metodo.as_bytes()).unwrap_or(Method::GET);
        let mut pedido = self.http.request(metodo, self.url(ruta)).header("Content-Type", "application/json");
        for (clave, valor) in encabezados {
            pedido = pedido.header(*clave, *valor);
        }
        if let Some(cuerpo) = cuerpo {
            pedido = pedido.body(cuerpo.to_string());
        }
        let respuesta = pedido.send().map_err(|e| ErrorApi { estado: 0, mensaje: e.to_string() })?;
        let estado = respuesta.status().as_u16();
        let texto = respuesta.text().unwrap_or_default();
        let cuerpo = serde_json::from_str(&texto).unwrap_or(if texto.is_empty() { Value::Null } else { Value::String(texto) });
        Ok(Respuesta { estado, cuerpo })
    }

    fn ruta_api(ruta: &str) -> String {
        if ruta.starts_with("/api/") {
            ruta.to_string()
        } else if ruta.starts_with('/') {
            format!("/api{ruta}")
        } else {
            format!("/api/{ruta}")
        }
    }

    fn exigir(&self, metodo: &str, ruta: &str, cuerpo: Option<&Value>) -> Result<Value, ErrorApi> {
        let respuesta = self.pedir(metodo, &Self::ruta_api(ruta), cuerpo, &[])?;
        if respuesta.ok() {
            Ok(respuesta.cuerpo)
        } else {
            Err(ErrorApi { estado: respuesta.estado, mensaje: respuesta.error() })
        }
    }

    pub fn get(&self, ruta: &str, parametros: &[(&str, String)]) -> Result<Value, ErrorApi> {
        let consulta: Vec<String> = parametros
            .iter()
            .filter(|(_, valor)| !valor.is_empty())
            .map(|(clave, valor)| format!("{clave}={}", codificar(valor)))
            .collect();
        let ruta = if consulta.is_empty() { ruta.to_string() } else { format!("{ruta}?{}", consulta.join("&")) };
        self.exigir("GET", &ruta, None)
    }

    pub fn post(&self, ruta: &str, cuerpo: &Value) -> Result<Value, ErrorApi> {
        self.exigir("POST", ruta, Some(cuerpo))
    }

    pub fn put(&self, ruta: &str, cuerpo: &Value) -> Result<Value, ErrorApi> {
        self.exigir("PUT", ruta, Some(cuerpo))
    }

    pub fn delete(&self, ruta: &str) -> Result<Value, ErrorApi> {
        self.exigir("DELETE", ruta, None)
    }

    pub fn bytes(&self, ruta: &str) -> Result<(u16, Vec<u8>), ErrorApi> {
        let respuesta = self.http.get(self.url(ruta)).send().map_err(|e| ErrorApi { estado: 0, mensaje: e.to_string() })?;
        let estado = respuesta.status().as_u16();
        let bytes = respuesta.bytes().map_err(|e| ErrorApi { estado, mensaje: e.to_string() })?;
        Ok((estado, bytes.to_vec()))
    }
}

pub fn codificar(valor: &str) -> String {
    let mut salida = String::new();
    for byte in valor.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => salida.push(byte as char),
            b' ' => salida.push('+'),
            _ => salida.push_str(&format!("%{byte:02X}")),
        }
    }
    salida
}
