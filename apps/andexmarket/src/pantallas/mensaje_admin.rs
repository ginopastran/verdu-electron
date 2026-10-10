use eframe::egui::{self, Align, Rect, Sense, Ui, Vec2};

use super::sesion::{linea, Parrafo};
use crate::app::{Contexto, Ruta};
use crate::ui::{fuente, paleta, Peso, Tam};

pub fn mostrar(ui: &mut Ui, cx: &mut Contexto) {
    let pantalla = ui.ctx().content_rect();
    ui.painter().with_clip_rect(pantalla).rect_filled(pantalla, 0, crate::ui::color::FONDO);
    let ancho = 448.0 - 64.0;
    let cuerpo = Parrafo::new(
        ui,
        "Esta aplicación está diseñada exclusivamente para vendedores. Los administradores deben utilizar la aplicación web.",
        Tam::Base,
        Peso::Normal,
        paleta::GRAY_600,
        ancho + 8.0,
        24.0,
        Align::Center,
    );
    let alto = 32.0 + 16.0 + cuerpo.alto() + 16.0 + 20.0;
    let mut y = (pantalla.center().y - alto / 2.0).round();
    let fila = |y: f32, alto: f32| Rect::from_min_size(egui::pos2(pantalla.center().x - ancho / 2.0, y), Vec2::new(ancho, alto));
    linea(ui, fila(y, 32.0), "Acceso no permitido", Tam::X2, Peso::Negrita, paleta::GRAY_900, Align::Center);
    y += 32.0 + 16.0;
    cuerpo.pintar(ui, fila(y, cuerpo.alto()));
    y += cuerpo.alto() + 16.0;
    let texto = "Volver al inicio";
    let ancho_enlace = ui.painter().layout_no_wrap(texto.into(), fuente(Tam::Sm, Peso::Normal), paleta::EMERALD_600).size().x;
    let enlace = Rect::from_center_size(egui::pos2(pantalla.center().x, y + 10.0), Vec2::new(ancho_enlace, 20.0));
    let respuesta = ui.interact(enlace, ui.id().with("volver-al-inicio"), Sense::click());
    let color = if respuesta.hovered() { paleta::EMERALD_700 } else { paleta::EMERALD_600 };
    linea(ui, enlace, texto, Tam::Sm, Peso::Normal, color, Align::Center);
    if respuesta.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        cx.global.usuario = None;
        cx.global.ir(Ruta::Inicio);
    }
}
