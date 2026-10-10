use std::hash::Hash;

use eframe::egui::{self, Align, Area, Color32, Context, Frame, Id, Layout, Margin, Order, Rect, Shape, Ui, Vec2};

use super::animacion::{apertura, escala_pop, escalar_capa, DURACION_FONDO, DURACION_MODAL};
use super::boton::BotonIcono;
use crate::ui::marca::{color, radio, sombra, tam};
use crate::ui::pintar_icono_solar_en;

pub struct RespuestaModal<R> {
    pub interior: R,
    pub cerrar: bool,
}

pub struct Modal<'a> {
    id: Id,
    ancho: f32,
    titulo: &'a str,
    subtitulo: &'a str,
    icono: Option<&'a str>,
    fondo_icono: Color32,
    color_icono: Color32,
    cerrable: bool,
}

impl<'a> Modal<'a> {
    pub fn new(id: impl Hash + std::fmt::Debug, titulo: &'a str) -> Self {
        Modal {
            id: Id::new("modal-v2").with(id),
            ancho: 460.0,
            titulo,
            subtitulo: "",
            icono: None,
            fondo_icono: color::VERDE_CHIP,
            color_icono: color::VERDE,
            cerrable: true,
        }
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = ancho;
        self
    }

    pub fn subtitulo(mut self, subtitulo: &'a str) -> Self {
        self.subtitulo = subtitulo;
        self
    }

    pub fn icono(mut self, icono: &'a str) -> Self {
        self.icono = Some(icono);
        self
    }

    pub fn tono_icono(mut self, fondo: Color32, color: Color32) -> Self {
        self.fondo_icono = fondo;
        self.color_icono = color;
        self
    }

    pub fn peligro(self) -> Self {
        self.tono_icono(color::ROJO_FONDO, color::ROJO_INTENSO)
    }

    pub fn aviso(self) -> Self {
        self.tono_icono(color::DORADO_FONDO, color::DORADO_OSCURO)
    }

    pub fn exito(self) -> Self {
        self.tono_icono(color::VERDE, color::BLANCO)
    }

    pub fn sin_cerrar(mut self) -> Self {
        self.cerrable = false;
        self
    }

    pub fn mostrar<R>(self, ctx: &Context, contenido: impl FnOnce(&mut Ui) -> R) -> RespuestaModal<R> {
        let pantalla = ctx.content_rect();
        let fundido = apertura(ctx, self.id.with("fondo"), DURACION_FONDO);
        let avance = apertura(ctx, self.id, DURACION_MODAL);
        Area::new(self.id.with("velo")).order(Order::Middle).fixed_pos(pantalla.min).interactable(false).show(ctx, |ui| {
            ui.painter().rect_filled(pantalla, 0, color::OVERLAY_MODAL.gamma_multiply(fundido));
        });
        let ancho = self.ancho.min(pantalla.width() - 48.0);
        let mut x_clic = false;
        let respuesta = egui::Modal::new(self.id).backdrop_color(Color32::TRANSPARENT).frame(Frame::NONE).show(ctx, |ui| {
            ui.set_opacity(avance);
            escalar_capa(ctx, ui.layer_id(), pantalla.center(), escala_pop(avance));
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ancho);
            let fondo = ui.painter().add(Shape::Noop);
            let interior = ui
                .vertical(|ui| {
                    ui.set_width(ancho);
                    Frame::NONE.inner_margin(Margin { left: 24, right: 24, top: 22, bottom: 0 }).show(ui, |ui| {
                        ui.set_width(ancho - 48.0);
                        x_clic = self.encabezado(ui);
                    });
                    contenido(ui)
                })
                .inner;
            let rect = Rect::from_min_size(ui.min_rect().min, Vec2::new(ancho, ui.min_rect().height()));
            ui.painter().set(
                fondo,
                Shape::Vec(vec![sombra::MODAL.forma(rect, radio::MODAL), Shape::rect_filled(rect, radio::MODAL, color::BLANCO)]),
            );
            interior
        });
        RespuestaModal { cerrar: x_clic || (self.cerrable && respuesta.should_close()), interior: respuesta.inner }
    }

    fn encabezado(&self, ui: &mut Ui) -> bool {
        let mut clic = false;
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            if let Some(nombre) = self.icono {
                let (caja, _) = ui.allocate_exact_size(Vec2::splat(42.0), egui::Sense::hover());
                ui.painter().rect_filled(caja, radio::CAMPO, self.fondo_icono);
                pintar_icono_solar_en(ui, Rect::from_center_size(caja.center(), Vec2::splat(23.0)), nombre, self.color_icono);
            }
            let ancho_textos = ui.available_width() - if self.cerrable { 32.0 + 12.0 } else { 0.0 };
            ui.allocate_ui_with_layout(Vec2::new(ancho_textos, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.add_space(1.0);
                super::titulo(ui, self.titulo, tam::TITULO_MODAL, -0.01, color::TEXTO);
                if !self.subtitulo.is_empty() {
                    ui.add_space(3.0);
                    super::parrafo(ui, self.subtitulo, tam::CONTROL, color::TEXTO_APAGADO, 1.45, ancho_textos);
                }
            });
            if self.cerrable {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    clic = BotonIcono::cerrar().ui(ui).clicked();
                });
            }
        });
        clic
    }
}

pub fn cuerpo_modal<R>(ui: &mut Ui, arriba: i8, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::NONE
        .inner_margin(Margin { left: 24, right: 24, top: arriba, bottom: 0 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenido(ui)
        })
        .inner
}

pub fn pie_modal<R>(ui: &mut Ui, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::NONE
        .inner_margin(Margin { left: 24, right: 24, top: 20, bottom: 22 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 44.0), Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                contenido(ui)
            })
            .inner
        })
        .inner
}
