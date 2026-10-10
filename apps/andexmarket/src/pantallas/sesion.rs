use crate::app::{Contexto, Ruta};

#[derive(Default)]
pub struct Sesion;

impl Sesion {
    pub fn actualizar(&mut self, _cx: &mut Contexto) {}
}

pub fn cerrar_sesion(cx: &mut Contexto) {
    cx.global.usuario = None;
    cx.global.ir(Ruta::UsuarioLogin);
}
