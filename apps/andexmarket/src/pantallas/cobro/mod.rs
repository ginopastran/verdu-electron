use eframe::egui::Ui;

use crate::app::Contexto;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionCobro {
    Cancelar,
    Pagar,
    PagarAfip,
    PagarConDescuento,
    PagarAfipConDescuento,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResultadoCobro {
    VentaRegistrada,
    Cancelado,
}

#[derive(Default)]
pub struct Cobro;

impl Cobro {
    pub fn iniciar(&mut self, _accion: AccionCobro, _cx: &mut Contexto) {}

    pub fn ocupado(&self) -> bool {
        false
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, _cx: &mut Contexto) -> Option<ResultadoCobro> {
        None
    }
}
