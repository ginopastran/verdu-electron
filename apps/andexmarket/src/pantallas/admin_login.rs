use eframe::egui::Ui;

use crate::app::Contexto;

#[derive(Default)]
pub struct AdminLogin;

impl AdminLogin {
    pub fn mostrar(&mut self, ui: &mut Ui, _cx: &mut Contexto) {
        super::pendiente(ui, "AdminLogin");
    }
}
