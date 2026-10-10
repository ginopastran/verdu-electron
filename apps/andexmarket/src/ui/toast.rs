use eframe::egui::{self, Align2, Color32, Context, Id, Order, Rect, Sense, Vec2};

use super::{borde, fuente, paleta, sombra_md, Peso, Tam};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    Exito,
    Error,
    Info,
    Aviso,
}

#[derive(Clone)]
struct Aviso {
    tipo: Tipo,
    titulo: String,
    detalle: Option<String>,
    desde: f64,
    duracion: f64,
}

fn clave() -> Id {
    Id::new("toasts")
}

pub fn mostrar_aviso(ctx: &Context, tipo: Tipo, titulo: impl Into<String>, detalle: Option<String>, duracion: f64) {
    let desde = ctx.input(|i| i.time);
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<Aviso>>(clave()).push(Aviso {
            tipo,
            titulo: titulo.into(),
            detalle,
            desde,
            duracion,
        })
    });
    ctx.request_repaint();
}

pub fn exito(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Exito, titulo, None, 2.0);
}

pub fn error(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Error, titulo, None, 2.0);
}

pub fn info(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Info, titulo, None, 2.0);
}

pub fn aviso(ctx: &Context, titulo: impl Into<String>) {
    mostrar_aviso(ctx, Tipo::Aviso, titulo, None, 2.0);
}

pub fn con_detalle(ctx: &Context, tipo: Tipo, titulo: impl Into<String>, detalle: impl Into<String>, duracion: f64) {
    mostrar_aviso(ctx, tipo, titulo, Some(detalle.into()), duracion);
}

fn colores(tipo: Tipo) -> (Color32, Color32, Color32, &'static str) {
    match tipo {
        Tipo::Exito => (Color32::from_rgb(0xec, 0xfd, 0xf3), Color32::from_rgb(0x00, 0x8a, 0x2e), Color32::from_rgb(0xbf, 0xfc, 0xd9), "check-circle"),
        Tipo::Error => (Color32::from_rgb(0xff, 0xf0, 0xf0), Color32::from_rgb(0xe6, 0x00, 0x00), Color32::from_rgb(0xff, 0xe0, 0xe1), "alert-circle"),
        Tipo::Info => (Color32::from_rgb(0xf0, 0xf8, 0xff), Color32::from_rgb(0x00, 0x71, 0xf3), Color32::from_rgb(0xd3, 0xe0, 0xfd), "alert-circle"),
        Tipo::Aviso => (Color32::from_rgb(0xff, 0xfc, 0xf0), Color32::from_rgb(0xdc, 0x71, 0x00), Color32::from_rgb(0xfd, 0xf5, 0xd3), "alert-triangle"),
    }
}

pub fn pintar(ctx: &Context) {
    let ahora = ctx.input(|i| i.time);
    let mut avisos = ctx.data_mut(|d| d.get_temp_mut_or_default::<Vec<Aviso>>(clave()).clone());
    avisos.retain(|a| ahora - a.desde < a.duracion + 0.2);
    ctx.data_mut(|d| *d.get_temp_mut_or_default::<Vec<Aviso>>(clave()) = avisos.clone());
    if avisos.is_empty() {
        return;
    }
    ctx.request_repaint();
    let pantalla = ctx.content_rect();
    let ancho = 356.0;
    let mut y = pantalla.top() + 32.0;
    let _ = paleta::GRAY_50;
    egui::Area::new(Id::new("capa-toasts"))
        .order(Order::Tooltip)
        .fixed_pos(pantalla.left_top())
        .interactable(false)
        .show(ctx, |ui| {
            for aviso in avisos.iter().rev() {
                let transcurrido = ahora - aviso.desde;
                let opacidad = if transcurrido > aviso.duracion { (1.0 - (transcurrido - aviso.duracion) / 0.2) as f32 } else { (transcurrido / 0.15).min(1.0) as f32 };
                let (fondo, texto_color, trazo, icono) = colores(aviso.tipo);
                let titulo = ui.painter().layout(aviso.titulo.clone(), fuente(Tam::Px(13.0), Peso::Medio), texto_color, ancho - 64.0);
                let detalle = aviso.detalle.as_ref().map(|d| ui.painter().layout(d.clone(), fuente(Tam::Px(13.0), Peso::Normal), texto_color, ancho - 64.0));
                let alto = 32.0 + titulo.size().y + detalle.as_ref().map_or(0.0, |d| d.size().y + 2.0);
                let rect = Rect::from_min_size(egui::pos2(pantalla.center().x - ancho / 2.0, y), Vec2::new(ancho, alto.max(53.0)));
                ui.painter().add(sombra_md().as_shape(rect, 8));
                ui.painter().rect(rect, 8, fondo.gamma_multiply(opacidad), borde(trazo.gamma_multiply(opacidad)), egui::StrokeKind::Inside);
                let caja_icono = Rect::from_center_size(egui::pos2(rect.left() + 26.0, rect.center().y), Vec2::splat(20.0));
                super::pintar_icono_en(ui, caja_icono, icono, texto_color.gamma_multiply(opacidad));
                let mut ty = rect.center().y - (titulo.size().y + detalle.as_ref().map_or(0.0, |d| d.size().y + 2.0)) / 2.0;
                let tx = rect.left() + 46.0;
                let alto_titulo = titulo.size().y;
                ui.painter().galley(egui::pos2(tx, ty), titulo, texto_color.gamma_multiply(opacidad));
                ty += alto_titulo + 2.0;
                if let Some(d) = detalle {
                    ui.painter().galley(egui::pos2(tx, ty), d, texto_color.gamma_multiply(opacidad));
                }
                let _ = ui.interact(rect, Id::new(("toast", aviso.desde.to_bits())), Sense::hover());
                let _ = Align2::CENTER_CENTER;
                y += rect.height() + 14.0;
            }
        });
}
