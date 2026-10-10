mod detalle;
pub mod formulario;
mod lista;
mod modal;
mod pagos;
mod selectores;
mod vista;

use std::path::PathBuf;

use eframe::egui::Ui;
use nucleo::formato;
use serde_json::Value;

use crate::app::Contexto;

#[derive(Default)]
pub struct Facturas {
    lista: lista::Lista,
    detalle: detalle::Detalle,
}

impl Facturas {
    pub fn mostrar_lista(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        self.lista.mostrar(ui, cx);
    }

    pub fn mostrar_detalle(&mut self, ui: &mut Ui, cx: &mut Contexto, id: &str) {
        self.detalle.mostrar(ui, cx, id);
    }
}

fn recien_montada(ultimo_pase: &mut u64, ui: &Ui) -> bool {
    let pase = ui.ctx().cumulative_pass_nr();
    let montada = *ultimo_pase == 0 || pase > *ultimo_pase + 1;
    *ultimo_pase = pase;
    montada
}

fn id_texto(valor: &Value) -> String {
    valor.as_str().map(str::to_string).unwrap_or_else(|| valor.to_string())
}

fn texto<'a>(valor: &'a Value, clave: &str) -> &'a str {
    valor[clave].as_str().unwrap_or_default()
}

fn numero(valor: &Value, clave: &str) -> f64 {
    valor[clave].as_f64().unwrap_or(0.0)
}

fn pesos(valor: f64) -> String {
    format!("${}", formato::numero_con(valor, 0, 2))
}

fn iva_en_precios(cx: &Contexto) -> Option<bool> {
    cx.global.negocio.as_ref().map(|n| n["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false))
}

fn mostrar_valor(valor: &Value) -> String {
    match valor {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        otro => otro.to_string(),
    }
}

fn carpeta_descargas(cx: &Contexto) -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|inicio| PathBuf::from(inicio).join("Downloads"))
        .unwrap_or_else(|| cx.servicios.carpeta.clone())
}

#[cfg(test)]
mod tests {
    use super::pesos;

    #[test]
    fn pesos_sin_decimales_de_relleno() {
        assert_eq!(pesos(407410.0), "$407.410");
        assert_eq!(pesos(63925.62), "$63.925,62");
        assert_eq!(pesos(0.0), "$0");
    }
}
