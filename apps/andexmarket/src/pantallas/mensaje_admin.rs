use eframe::egui::{self, Align, Id, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};

use super::sesion::{self, Parrafo};
use crate::app::{Contexto, Ruta};
use crate::ui::marca::{color, radio};
use crate::ui::v2::{self, Boton};
use crate::ui::Peso;

pub fn mostrar(ui: &mut Ui, cx: &mut Contexto) {
    let pantalla = ui.ctx().content_rect();
    ui.painter().with_clip_rect(pantalla).rect_filled(pantalla, 0, color::FONDO_APP);
    let zona = pantalla.shrink(32.0);
    let ancho = 460.0_f32.min(zona.width());
    let volver = sesion::centrado(ui, Id::new("mensaje-admin-tarjeta"), zona, ancho, Align::Center, |ui| {
        let fondo = ui.painter().add(Shape::Noop);
        let arriba = ui.cursor().top();
        let interior = ancho - 64.0 - 2.0;
        ui.add_space(33.0);
        let (caja, _) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
        ui.painter().rect_filled(caja, radio::TARJETA, color::DORADO_FONDO);
        v2::pintar_icono(ui, caja.center(), "shield", 30.0, color::DORADO_OSCURO);
        ui.add_space(16.0);
        v2::titulo(ui, "Esta es una cuenta de administrador", 22.0, -0.015, color::TEXTO);
        ui.add_space(16.0);
        Parrafo::centrado(
            ui,
            "El punto de venta es para vendedores. Para administrar productos, stock y reportes, entrá desde el panel web.",
            14.5,
            Peso::Normal,
            color::TEXTO_APAGADO,
            interior,
            1.55,
        )
        .ui(ui, interior);
        ui.add_space(16.0 + 6.0);
        let boton = Boton::primario("Elegir otro usuario").icono("arrow-left").modal().tam_texto(14.0).ui(ui);
        ui.add_space(33.0);
        let tarjeta = Rect::from_min_max(egui::pos2(ui.max_rect().left(), arriba), egui::pos2(ui.max_rect().left() + ancho, ui.cursor().top()));
        ui.painter().set(fondo, Shape::Rect(egui::epaint::RectShape::new(tarjeta, radio::MODAL, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside)));
        boton.clicked()
    });
    if volver {
        cx.global.usuario = None;
        cx.global.ir(Ruta::Inicio);
    }
}
