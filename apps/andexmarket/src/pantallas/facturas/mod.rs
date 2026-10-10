mod comun;
mod detalle;
pub mod formulario;
mod lista;
mod modal;
mod pagos;
mod selectores;
mod tarjeta;

use eframe::egui::Ui;
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

fn mostrar_valor(valor: &Value) -> String {
    match valor {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.as_f64().map(nucleo::facturas::texto_numero).unwrap_or_else(|| n.to_string()),
        otro => otro.to_string(),
    }
}
