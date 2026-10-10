use eframe::egui::Ui;

use crate::app::Contexto;

pub fn mostrar(ui: &mut Ui, _cx: &mut Contexto) {
    super::pendiente(ui, "MensajeAdmin");
}
