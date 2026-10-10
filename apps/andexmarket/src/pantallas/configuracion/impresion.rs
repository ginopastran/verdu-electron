use eframe::egui::Ui;

use crate::app::Contexto;

#[derive(Default)]
pub struct DialogoImpresion {
    abierto: bool,
}

impl DialogoImpresion {
    pub fn abrir(&mut self) {
        self.abierto = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, _cx: &mut Contexto) {}
}
