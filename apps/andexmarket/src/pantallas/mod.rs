pub mod admin_login;
pub mod carrito;
pub mod cierre;
pub mod cobro;
pub mod configuracion;
pub mod facturas;
pub mod mensaje_admin;
pub mod pedidos;
pub mod sesion;
pub mod usuario_login;

use eframe::egui::Ui;

use crate::app::{Contexto, Ruta};

#[derive(Default)]
pub struct Pantallas {
    pub sesion: sesion::Sesion,
    pub admin_login: admin_login::AdminLogin,
    pub usuario_login: usuario_login::UsuarioLogin,
    pub carrito: carrito::Carrito,
    pub facturas: facturas::Facturas,
    pub actualizacion: configuracion::actualizacion::Actualizacion,
}

impl Pantallas {
    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        self.sesion.actualizar(cx);
        match cx.global.ruta.clone() {
            Ruta::Inicio => {
                let destino = if cx.global.business_id().is_some() { Ruta::UsuarioLogin } else { Ruta::AdminLogin };
                cx.global.ir(destino);
                ui.ctx().request_repaint();
            }
            Ruta::AdminLogin => self.admin_login.mostrar(ui, cx),
            Ruta::UsuarioLogin => self.usuario_login.mostrar(ui, cx),
            Ruta::MensajeAdmin => mensaje_admin::mostrar(ui, cx),
            Ruta::Carrito => self.carrito.mostrar(ui, cx),
            Ruta::Facturas => self.facturas.mostrar_lista(ui, cx),
            Ruta::FacturaDetalle(id) => self.facturas.mostrar_detalle(ui, cx, &id),
        }
        self.actualizacion.mostrar(ui, cx);
    }
}
