use eframe::egui::Ui;

use crate::app::Contexto;

#[derive(Default)]
pub struct Carrito;

impl Carrito {
    pub fn mostrar(&mut self, ui: &mut Ui, _cx: &mut Contexto) {
        super::pendiente(ui, "Carrito");
    }
}
