use eframe::egui::Ui;
use serde_json::Value;

use crate::app::Contexto;

#[derive(Clone, Debug, PartialEq)]
pub struct ProductoInicial {
    pub id: i64,
    pub nombre: String,
    pub precio: f64,
    pub cantidad: f64,
    pub subtotal: f64,
}

pub enum ResultadoFactura {
    Cerrada,
    Creada(Value),
}

#[derive(Default)]
pub struct FormularioFactura {
    abierto: bool,
}

impl FormularioFactura {
    pub fn abrir(&mut self, _productos_iniciales: Vec<ProductoInicial>) {
        self.abierto = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, _cx: &mut Contexto) -> Option<ResultadoFactura> {
        None
    }
}
