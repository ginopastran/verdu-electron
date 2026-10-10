use eframe::egui::{self, Color32, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use super::galeria;
use crate::ui::marca::{color, fuente, radio, sombra, tam};
use crate::ui::{pintar_icono_solar_en, Peso};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EstiloSegmentado {
    Claro,
    Filtro,
}

#[derive(Clone, Copy, Debug)]
pub struct Segmento<'a> {
    pub etiqueta: &'a str,
    pub icono: Option<&'a str>,
}

impl<'a> Segmento<'a> {
    pub fn texto(etiqueta: &'a str) -> Self {
        Segmento { etiqueta, icono: None }
    }

    pub fn con_icono(etiqueta: &'a str, icono: &'a str) -> Self {
        Segmento { etiqueta, icono: Some(icono) }
    }
}

pub fn segmentado(ui: &mut Ui, segmentos: &[Segmento], activo: &mut usize, ancho: Option<f32>, estilo: EstiloSegmentado) -> Response {
    let (alto_item, relleno, separacion, radio_caja, radio_item, fondo, px) = match estilo {
        EstiloSegmentado::Claro => (36.0, 3.0, 3.0, radio::BOTON_MODAL, radio::ICONO, color::FONDO_APP, tam::CUERPO),
        EstiloSegmentado::Filtro => (28.0, 3.0, 2.0, radio::CONTROL, 7, color::BLANCO, tam::CONTROL),
    };
    let fuente_activa = fuente(px, Peso::Semi);
    let anchos_naturales: Vec<f32> = segmentos
        .iter()
        .map(|s| galeria(ui, s.etiqueta, fuente_activa.clone(), Color32::WHITE).size().x + if s.icono.is_some() { 22.0 } else { 0.0 } + 24.0)
        .collect();
    let natural = anchos_naturales.iter().sum::<f32>() + separacion * (segmentos.len().saturating_sub(1)) as f32 + relleno * 2.0 + 2.0;
    let ancho_total = ancho.unwrap_or(natural);
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(ancho_total, alto_item + relleno * 2.0 + 2.0), Sense::hover());
    ui.painter().rect(rect, radio_caja, fondo, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let interior = rect.shrink(1.0 + relleno);
    let ancho_parejo = (interior.width() - separacion * (segmentos.len().saturating_sub(1)) as f32) / segmentos.len().max(1) as f32;
    let mut x = interior.left();
    for (indice, segmento) in segmentos.iter().enumerate() {
        let ancho_item = if ancho.is_some() { ancho_parejo } else { anchos_naturales[indice] };
        let caja = Rect::from_min_size(egui::pos2(x, interior.top()), Vec2::new(ancho_item, interior.height()));
        let r = ui.interact(caja, respuesta.id.with(indice), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        if r.clicked() && *activo != indice {
            *activo = indice;
            respuesta.mark_changed();
        }
        let elegido = *activo == indice;
        let (fondo_item, texto_color, peso) = match (estilo, elegido) {
            (EstiloSegmentado::Claro, true) => (color::BLANCO, color::VERDE_PROFUNDO, Peso::Semi),
            (EstiloSegmentado::Claro, false) => (Color32::TRANSPARENT, color::TEXTO_APAGADO, Peso::Normal),
            (EstiloSegmentado::Filtro, true) => (color::VERDE, color::BLANCO, Peso::Semi),
            (EstiloSegmentado::Filtro, false) => (Color32::TRANSPARENT, color::TEXTO_SECUNDARIO, Peso::Medio),
        };
        if elegido {
            if estilo == EstiloSegmentado::Claro {
                ui.painter().add(sombra::SEGMENTO.forma(caja, radio_item));
            }
            ui.painter().rect_filled(caja, radio_item, fondo_item);
        }
        let g = galeria(ui, segmento.etiqueta, fuente(px, peso), texto_color);
        let ancho_contenido = g.size().x + if segmento.icono.is_some() { 22.0 } else { 0.0 };
        let mut cx = caja.center().x - ancho_contenido / 2.0;
        if let Some(nombre) = segmento.icono {
            pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(cx + 8.0, caja.center().y), Vec2::splat(16.0)), nombre, texto_color);
            cx += 22.0;
        }
        ui.painter().galley(egui::pos2(cx, caja.center().y - g.size().y / 2.0), g, texto_color);
        x += ancho_item + separacion;
    }
    respuesta
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ColoresEleccion {
    pub borde: Color32,
    pub fondo: Color32,
    pub texto: Color32,
}

pub fn colores_eleccion(activo: bool) -> ColoresEleccion {
    if activo {
        ColoresEleccion { borde: color::VERDE, fondo: color::VERDE_ELEGIDO, texto: color::VERDE_PROFUNDO }
    } else {
        ColoresEleccion { borde: color::BORDE, fondo: color::BLANCO, texto: color::TEXTO_SECUNDARIO }
    }
}

pub fn opcion_eleccion(ui: &mut Ui, tam_opcion: Vec2, radio_opcion: u8, activo: bool, grosor: f32, contenido: impl FnOnce(&Ui, Rect, ColoresEleccion)) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(tam_opcion, Sense::click());
    let colores = colores_eleccion(activo);
    let borde = if respuesta.hovered() { color::VERDE } else { colores.borde };
    ui.painter().rect(rect, radio_opcion, colores.fondo, Stroke::new(grosor, borde), StrokeKind::Inside);
    contenido(ui, rect, colores);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}
