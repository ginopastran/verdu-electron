pub mod formulario;

use eframe::egui::Ui;

use crate::app::Contexto;

#[derive(Default)]
pub struct Facturas;

impl Facturas {
    pub fn mostrar_lista(&mut self, ui: &mut Ui, _cx: &mut Contexto) {
        super::pendiente(ui, "Facturas");
    }

    pub fn mostrar_detalle(&mut self, ui: &mut Ui, _cx: &mut Contexto, _id: &str) {
        super::pendiente(ui, "Detalle de factura");
    }
}
