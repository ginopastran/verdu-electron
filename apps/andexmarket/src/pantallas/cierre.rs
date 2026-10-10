use eframe::egui::Ui;

use crate::app::Contexto;

pub enum ResultadoCierre {
    Cancelado,
    Completado,
}

#[derive(Default)]
pub struct DialogoCierre {
    abierto: bool,
}

impl DialogoCierre {
    pub fn abrir(&mut self) {
        self.abierto = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, _cx: &mut Contexto) -> Option<ResultadoCierre> {
        None
    }
}
