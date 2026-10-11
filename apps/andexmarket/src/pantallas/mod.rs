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

use eframe::egui::{Key, Modifiers, Ui};
use serde_json::Value;

use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui;
use crate::ui::v2::toast;
use carrito::productos::ProductoDisponible;

struct Recargado {
    negocio: Option<(i64, Result<Value, String>)>,
    productos: Option<Result<Vec<ProductoDisponible>, String>>,
    logo: Option<Result<(), String>>,
}

#[derive(Default)]
pub struct Pantallas {
    pub sesion: sesion::Sesion,
    pub admin_login: admin_login::AdminLogin,
    pub usuario_login: usuario_login::UsuarioLogin,
    pub carrito: carrito::Carrito,
    pub facturas: facturas::Facturas,
    pub actualizacion: configuracion::actualizacion::Actualizacion,
    recarga: Option<Tarea<Recargado>>,
}

impl Pantallas {
    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::R)) {
            self.recargar(cx);
        }
        self.atender_recarga(cx);
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

    pub fn recargando(&self) -> bool {
        self.recarga.is_some()
    }

    fn recargar(&mut self, cx: &mut Contexto) {
        if self.recarga.is_some() || (cx.global.ruta == Ruta::Carrito && self.carrito.hay_dialogo()) {
            return;
        }
        cx.egui.forget_all_images();
        ui::iconos::registrar(&cx.egui);
        toast::info(&cx.egui, "Actualizando datos…");
        let servicios = cx.servicios.clone();
        let negocio = cx.global.business_id();
        let con_sesion = cx.global.usuario.is_some();
        self.recarga = Some(Tarea::lanzar(&cx.egui, move || Recargado {
            negocio: negocio.map(|id| (id, sesion::descargar_negocio(&servicios, id))),
            productos: con_sesion.then(|| carrito::productos::descargar_todos(&servicios.api)),
            logo: con_sesion.then(|| carrito::sincronizar_logo(&servicios)),
        }));
    }

    fn atender_recarga(&mut self, cx: &mut Contexto) {
        let Some(recargado) = tareas::esperar(&mut self.recarga) else {
            return;
        };
        let mut errores = Vec::new();
        match recargado.negocio {
            Some((id, Ok(negocio))) if cx.global.business_id() == Some(id) => cx.global.negocio = Some(negocio),
            Some((_, Err(error))) => errores.push(error),
            _ => {}
        }
        match recargado.productos {
            Some(Ok(lista)) => self.carrito.fijar_productos(lista),
            Some(Err(error)) => errores.push(error),
            None => {}
        }
        if let Some(Err(error)) = recargado.logo {
            errores.push(error);
        }
        if errores.is_empty() {
            toast::exito(&cx.egui, "Datos actualizados");
        } else {
            toast::con_detalle(&cx.egui, toast::Tipo::Error, "No se pudieron actualizar los datos", errores.join(" · "));
        }
    }
}
