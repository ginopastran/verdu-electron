use eframe::egui::Ui;

use crate::app::Contexto;

#[derive(Default)]
pub struct Actualizacion;

impl Actualizacion {
    pub fn mostrar(&mut self, _ui: &mut Ui, _cx: &mut Contexto) {}
}
