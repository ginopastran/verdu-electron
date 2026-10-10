use std::f32::consts::TAU;

use eframe::egui::{self, emath::TSTransform, Color32, Context, Id, LayerId, Pos2, Rect, Ui, Vec2};

pub const DURACION_POP: f32 = 0.14;
pub const DURACION_MENU: f32 = 0.15;
pub const DURACION_MODAL: f32 = 0.2;
pub const DURACION_FONDO: f32 = 0.16;
pub const DURACION_TOAST: f32 = 0.26;
pub const PERIODO_GIRO: f32 = 0.9;
pub const ESCALA_INICIAL: f32 = 0.96;

pub fn salida_suave(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(5)
}

pub fn giro(ctx: &Context, periodo: f32) -> f32 {
    ctx.request_repaint();
    (ctx.input(|i| i.time) as f32 / periodo).fract() * TAU
}

pub fn pulso(ctx: &Context, periodo: f32) -> f32 {
    ctx.request_repaint();
    let fase = (ctx.input(|i| i.time) as f32 / periodo).fract();
    0.675 + 0.325 * (fase * TAU).cos()
}

pub fn aparicion(ctx: &Context, id: Id, visible: bool, duracion: f32) -> f32 {
    ctx.animate_bool_with_time_and_easing(id, visible, duracion, salida_suave)
}

pub fn apertura(ctx: &Context, id: Id, duracion: f32) -> f32 {
    let cuadro = ctx.cumulative_frame_nr();
    let ahora = ctx.input(|i| i.time);
    let clave = id.with("apertura");
    let inicio = ctx.data_mut(|d| {
        let (ultimo_cuadro, inicio) = d.get_temp::<(u64, f64)>(clave).unwrap_or((u64::MAX, ahora));
        let inicio = if ultimo_cuadro.wrapping_add(1) >= cuadro && ultimo_cuadro != u64::MAX { inicio } else { ahora };
        d.insert_temp(clave, (cuadro, inicio));
        inicio
    });
    let avance = ((ahora - inicio) as f32 / duracion).clamp(0.0, 1.0);
    if avance < 1.0 {
        ctx.request_repaint();
    }
    salida_suave(avance)
}

pub fn escalar_capa(ctx: &Context, capa: LayerId, origen: Pos2, escala: f32) {
    ctx.set_transform_layer(capa, TSTransform::new(origen.to_vec2() * (1.0 - escala), escala));
}

pub fn escala_pop(avance: f32) -> f32 {
    ESCALA_INICIAL + (1.0 - ESCALA_INICIAL) * avance
}

pub fn girando(ui: &Ui, caja: Rect, color: Color32) {
    let angulo = giro(ui.ctx(), PERIODO_GIRO);
    super::super::icono_solar("loader", caja.width(), color).rotate(angulo, Vec2::splat(0.5)).paint_at(ui, caja);
}

pub fn cargando(ui: &mut Ui, tam: f32, color: Color32) -> egui::Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(tam), egui::Sense::hover());
    girando(ui, rect, color);
    respuesta
}
