pub mod actualizacion;
pub mod balanza;
pub mod impresion;

use std::hash::Hash;

use eframe::egui::{
    self, text::LayoutJob, Color32, CursorIcon, Id, Popup, PopupCloseBehavior, Rect, RectAlign, Response, Sense, StrokeKind, TextFormat, Ui, Vec2,
};
use serde_json::Value;

use crate::ui::{self, color, Peso, Tam, RADIO_SM, RADIO_XL};

pub fn texto_valor(valor: &Value) -> String {
    match valor {
        Value::String(texto) => texto.clone(),
        Value::Null => String::new(),
        otro => otro.to_string(),
    }
}

fn medir(ui: &Ui, contenido: &str) -> f32 {
    ui.painter().layout_no_wrap(contenido.to_string(), ui::fuente(Tam::Sm, Peso::Normal), color::TEXTO).size().x
}

pub fn recortar_linea(ui: &Ui, contenido: &str, ancho: f32) -> String {
    if medir(ui, contenido) <= ancho {
        return contenido.to_string();
    }
    let mut linea = String::new();
    for palabra in contenido.split(' ') {
        let candidata = if linea.is_empty() { palabra.to_string() } else { format!("{linea} {palabra}") };
        if medir(ui, &format!("{candidata}…")) > ancho {
            break;
        }
        linea = candidata;
    }
    format!("{linea}…")
}

pub fn trabajo(contenido: &str, tam: Tam, peso: Peso, color: Color32, alto_linea: f32, espaciado: f32) -> LayoutJob {
    LayoutJob::single_section(
        contenido.to_string(),
        TextFormat {
            font_id: ui::fuente(tam, peso),
            color,
            extra_letter_spacing: espaciado,
            line_height: Some(alto_linea),
            ..Default::default()
        },
    )
}

pub fn media_interlinea(ctx: &egui::Context, tam: Tam, peso: Peso, alto_linea: f32) -> f32 {
    (alto_linea - ctx.fonts_mut(|fuentes| fuentes.row_height(&ui::fuente(tam, peso)))) / 2.0
}

fn texto_css(ui: &mut Ui, contenido: &str, tam: Tam, peso: Peso, color: Color32, alto_linea: f32, espaciado: f32) -> Response {
    let mut renglones = trabajo(contenido, tam, peso, color, alto_linea, espaciado);
    renglones.wrap.max_width = ui.available_width();
    let galeria = ui.ctx().fonts_mut(|fuentes| fuentes.layout_job(renglones));
    let (rect, respuesta) = ui.allocate_exact_size(galeria.size(), Sense::hover());
    let desplazamiento = media_interlinea(ui.ctx(), tam, peso, alto_linea);
    ui.painter().galley(rect.min + Vec2::new(0.0, desplazamiento), galeria, color);
    respuesta
}

pub fn parrafo(ui: &mut Ui, contenido: &str, tam: Tam, peso: Peso, color: Color32, alto_linea: f32) -> Response {
    texto_css(ui, contenido, tam, peso, color, alto_linea, 0.0)
}

pub fn encabezado(ui: &mut Ui, titulo: &str, descripcion: &str) {
    texto_css(ui, titulo, Tam::Lg, Peso::Semi, color::TEXTO, 18.0, -0.45);
    ui.add_space(6.0);
    parrafo(ui, descripcion, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO, 20.0);
}

pub fn campo<R>(ui: &mut Ui, etiqueta: &str, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    parrafo(ui, etiqueta, Tam::Sm, Peso::Medio, color::TEXTO, 24.0);
    ui.add_space(4.0);
    contenido(ui)
}

pub fn selector(
    ui: &mut Ui,
    id: impl Hash + std::fmt::Debug,
    seleccion: Option<usize>,
    opciones: &[String],
    sugerencia: &str,
    habilitado: bool,
) -> Option<usize> {
    let ancho = ui.available_width();
    let sentido = if habilitado { Sense::click() } else { Sense::hover() };
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 40.0), sentido);
    let opacidad = if habilitado { 1.0 } else { 0.5 };
    ui.painter().rect(
        rect,
        RADIO_XL,
        color::FONDO,
        ui::borde(color::BORDE_ENTRADA.gamma_multiply(opacidad)),
        StrokeKind::Inside,
    );
    let (contenido, texto_color) = match seleccion.and_then(|indice| opciones.get(indice)) {
        Some(opcion) => (opcion.as_str(), color::TEXTO),
        None => (sugerencia, color::APAGADO_TEXTO),
    };
    let visible = recortar_linea(ui, contenido, ancho - 24.0 - 16.0);
    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        visible,
        ui::fuente(Tam::Sm, Peso::Normal),
        texto_color.gamma_multiply(opacidad),
    );
    let flecha = Rect::from_center_size(egui::pos2(rect.right() - 20.0, rect.center().y), Vec2::splat(16.0));
    ui::pintar_icono_en(ui, flecha, "chevron-down", color::TEXTO.gamma_multiply(0.5 * opacidad));
    let popup_id = Id::new("selector-configuracion").with(id);
    if respuesta.clicked() {
        Popup::toggle_id(ui.ctx(), popup_id);
    }
    let natural = opciones.iter().map(|opcion| medir(ui, opcion)).fold(0.0, f32::max) + 32.0 + 8.0 + 8.0;
    let ancho_lista = ancho.max(128.0).max(natural);
    let mut elegido = None;
    Popup::new(popup_id, ui.ctx().clone(), rect, ui.layer_id())
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .align(RectAlign::BOTTOM_START)
        .gap(4.0)
        .width(ancho_lista)
        .frame(ui::marco_popup().shadow(ui::sombra_md()))
        .show(|ui| {
            ui.set_width(ancho_lista - 8.0);
            egui::ScrollArea::vertical().max_height(384.0).show(ui, |ui| {
                for (indice, opcion) in opciones.iter().enumerate() {
                    let (caja, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
                    if r.hovered() {
                        ui.painter().rect_filled(caja, RADIO_SM, color::ACENTO);
                    }
                    if Some(indice) == seleccion {
                        let marca = Rect::from_center_size(egui::pos2(caja.left() + 15.0, caja.center().y), Vec2::splat(16.0));
                        ui::pintar_icono_en(ui, marca, "check", color::TEXTO);
                    }
                    ui.painter().text(
                        egui::pos2(caja.left() + 32.0, caja.center().y),
                        egui::Align2::LEFT_CENTER,
                        opcion,
                        ui::fuente(Tam::Sm, Peso::Normal),
                        color::TEXTO,
                    );
                    if r.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        elegido = Some(indice);
                    }
                }
            });
        });
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand);
    }
    elegido.filter(|indice| Some(*indice) != seleccion)
}
