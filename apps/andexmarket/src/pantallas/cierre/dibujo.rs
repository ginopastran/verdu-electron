use std::f32::consts::{PI, TAU};

use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

use crate::ui::{self, fuente, Peso, Tam};

pub fn color_degradado_texto() -> Color32 {
    ui::mezclar(ui::color::ESMERALDA_DESDE, ui::color::ESMERALDA_HASTA, 0.6)
}

pub fn parrafo(ui: &mut Ui, contenido: &str, tam: Tam, peso: Peso, color: Color32, alto_linea: f32) -> Response {
    let ancho = ui.available_width();
    let natural = ui.ctx().fonts_mut(|f| f.row_height(&fuente(tam, peso)));
    let mut trabajo = egui::text::LayoutJob::single_section(
        contenido.to_string(),
        egui::TextFormat { font_id: fuente(tam, peso), color, line_height: Some(alto_linea), ..Default::default() },
    );
    trabajo.wrap.max_width = ancho;
    let galeria = ui.ctx().fonts_mut(|f| f.layout_job(trabajo));
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, galeria.size().y), Sense::hover());
    ui.painter().galley(rect.min + Vec2::new(0.0, ((alto_linea - natural) / 2.0).round()), galeria, color);
    respuesta
}

fn angulo(ui: &Ui) -> f32 {
    ui.ctx().request_repaint();
    (ui.input(|i| i.time) as f32 % 1.0) * TAU
}

fn punto(centro: Pos2, radio: f32, a: f32) -> Pos2 {
    centro + Vec2::new(a.cos(), a.sin()) * radio
}

fn arco(ui: &Ui, rect: Rect, grosor: f32, desde: f32, barrido: f32, color: Color32) {
    let centro = rect.center();
    let radio = rect.width() / 2.0 - grosor / 2.0;
    let pasos = 48;
    let puntos: Vec<Pos2> = (0..=pasos).map(|i| punto(centro, radio, desde + barrido * i as f32 / pasos as f32)).collect();
    ui.painter().add(Shape::line(puntos, Stroke::new(grosor, color)));
}

pub fn aro_con_hueco(ui: &Ui, rect: Rect, grosor: f32, color: Color32) {
    let giro = angulo(ui);
    arco(ui, rect, grosor, -0.25 * PI + giro, 1.5 * PI, color);
}

pub fn aro_girando(ui: &Ui, rect: Rect, grosor: f32, aro: Color32, arco: Color32) {
    let centro = rect.center();
    let radio = rect.width() / 2.0 - grosor / 2.0;
    if aro != Color32::TRANSPARENT {
        ui.painter().circle_stroke(centro, radio, Stroke::new(grosor, aro));
    }
    let giro = angulo(ui);
    self::arco(ui, rect, grosor, -0.75 * PI + giro, PI / 2.0, arco);
}

pub fn media_luna_girando(ui: &Ui, rect: Rect, grosor: f32, color: Color32) {
    let centro = rect.center();
    let giro = angulo(ui);
    let pasos = 32;
    for i in 0..pasos {
        let t0 = i as f32 / pasos as f32;
        let t1 = (i + 1) as f32 / pasos as f32;
        let ancho = grosor * (PI * (t0 + t1) / 2.0).sin();
        let radio = rect.width() / 2.0 - ancho / 2.0;
        let a0 = giro + PI * t0;
        let a1 = giro + PI * t1;
        ui.painter().line_segment([punto(centro, radio, a0), punto(centro, radio, a1)], Stroke::new(ancho, color));
    }
}
