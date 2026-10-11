use eframe::egui::{self, Area, Context, Id, Order, Rect, Vec2};

use super::animacion::{salida_suave, DURACION_TOAST};
use crate::ui::marca::{color, fuente, radio, sombra, tam};
use crate::ui::{icono_solar, Peso};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Exito,
    Error,
    Info,
    Aviso,
}

const DURACION: f64 = 2.8;

#[derive(Clone)]
struct Aviso {
    tipo: Tipo,
    titulo: String,
    detalle: Option<String>,
    desde: f64,
    duracion: f64,
}

fn clave() -> Id {
    Id::new("toast-v2")
}

pub fn mostrar_aviso(ctx: &Context, tipo: Tipo, titulo: impl Into<String>, detalle: Option<String>, duracion: f64) {
    let desde = ctx.input(|i| i.time);
    let aviso = Aviso { tipo, titulo: titulo.into(), detalle, desde, duracion };
    ctx.data_mut(|d| d.insert_temp(clave(), Some(aviso)));
    ctx.request_repaint();
}

pub fn exito(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Exito, titulo, None, DURACION);
}

pub fn error(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Error, titulo, None, DURACION);
}

pub fn info(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Info, titulo, None, DURACION);
}

pub fn aviso(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Aviso, titulo, None, DURACION);
}

pub fn con_detalle(ctx: &Context, tipo: Tipo, titulo: impl Into<String>, detalle: impl Into<String>) {
    mostrar_aviso(ctx, tipo, titulo, Some(detalle.into()), DURACION);
}

pub fn titulo_actual(ctx: &Context) -> Option<String> {
    ctx.data(|d| d.get_temp::<Option<Aviso>>(clave()).flatten()).map(|aviso| aviso.titulo)
}

fn estilo(tipo: Tipo) -> (&'static str, egui::Color32) {
    match tipo {
        Tipo::Exito => ("circle-check", color::VERDE_TOAST),
        Tipo::Error => ("alert", color::ROJO_TOAST),
        Tipo::Info => ("info", color::DORADO),
        Tipo::Aviso => ("alert", color::DORADO),
    }
}

pub fn pintar(ctx: &Context) {
    let ahora = ctx.input(|i| i.time);
    let Some(aviso) = ctx.data(|d| d.get_temp::<Option<Aviso>>(clave()).flatten()) else {
        return;
    };
    let transcurrido = ahora - aviso.desde;
    if transcurrido > aviso.duracion {
        ctx.data_mut(|d| d.insert_temp::<Option<Aviso>>(clave(), None));
        return;
    }
    ctx.request_repaint();
    let entrada = salida_suave((transcurrido as f32) / DURACION_TOAST);
    let pantalla = ctx.content_rect();
    let (nombre_icono, color_icono) = estilo(aviso.tipo);
    Area::new(Id::new("capa-toast-v2")).order(Order::Tooltip).fixed_pos(pantalla.min).interactable(false).show(ctx, |ui| {
        let ancho_maximo = (pantalla.width() - 32.0).min(520.0) - 29.0 - 19.0 - 10.0;
        let titulo = ui.painter().layout(aviso.titulo.clone(), fuente(tam::CUERPO, Peso::Medio), color::BLANCO, ancho_maximo);
        let detalle = aviso.detalle.as_ref().map(|d| ui.painter().layout(d.clone(), fuente(tam::CHICO, Peso::Normal), color::TEXTO_DESHABILITADO, ancho_maximo));
        let ancho_texto = titulo.size().x.max(detalle.as_ref().map_or(0.0, |d| d.size().x));
        let alto_texto = titulo.size().y + detalle.as_ref().map_or(0.0, |d| d.size().y + 1.0);
        let tam_caja = Vec2::new(13.0 + 19.0 + 10.0 + ancho_texto + 16.0, alto_texto.max(19.0) + 22.0);
        let y = pantalla.top() + 18.0 - 10.0 * (1.0 - entrada);
        let rect = Rect::from_min_size(egui::pos2(pantalla.center().x - tam_caja.x / 2.0, y), tam_caja);
        let pintor = ui.painter();
        pintor.add(sombra::TOAST.con_opacidad(entrada).forma(rect, radio::CAMPO));
        pintor.rect_filled(rect, radio::CAMPO, color::TEXTO.gamma_multiply(entrada));
        let caja_icono = Rect::from_center_size(egui::pos2(rect.left() + 13.0 + 9.5, rect.center().y), Vec2::splat(19.0));
        icono_solar(nombre_icono, 19.0, color_icono.gamma_multiply(entrada)).paint_at(ui, caja_icono);
        let x = caja_icono.right() + 10.0;
        let mut ty = rect.center().y - alto_texto / 2.0;
        let alto_titulo = titulo.size().y;
        pintor.galley_with_override_text_color(egui::pos2(x, ty), titulo, color::BLANCO.gamma_multiply(entrada));
        ty += alto_titulo + 1.0;
        if let Some(d) = detalle {
            pintor.galley_with_override_text_color(egui::pos2(x, ty), d, color::TEXTO_DESHABILITADO.gamma_multiply(entrada));
        }
    });
}
