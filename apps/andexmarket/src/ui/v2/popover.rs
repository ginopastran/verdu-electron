use std::hash::Hash;

use eframe::egui::{
    self, Color32, Context, CursorIcon, Frame, Id, LayerId, Margin, Popup, PopupCloseBehavior, Rect, RectAlign, Response, ScrollArea, Sense,
    Shape, Stroke, StrokeKind, Ui, Vec2,
};

use super::animacion::{aparicion, escala_pop, escalar_capa, DURACION_MENU, DURACION_POP};
use super::insignias::Avatar;
use super::galeria;
use crate::ui::marca::{color, fuente, fuente_mono, radio, sombra, tam, Sombra};
use crate::ui::{pintar_icono_solar_en, Peso};

pub struct Popover {
    id: Id,
    ancla: Rect,
    capa: LayerId,
    ancho: Option<f32>,
    separacion: f32,
    derecha: bool,
    radio: u8,
    sombra: Sombra,
    relleno: f32,
    alto_maximo: Option<f32>,
    visible: Option<bool>,
    duracion: f32,
}

impl Popover {
    pub fn new(id: Id, ancla: &Response) -> Self {
        Popover {
            id,
            ancla: ancla.rect,
            capa: ancla.layer_id,
            ancho: None,
            separacion: 6.0,
            derecha: false,
            radio: radio::CAMPO,
            sombra: sombra::DESPLEGABLE,
            relleno: 6.0,
            alto_maximo: None,
            visible: None,
            duracion: DURACION_POP,
        }
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = Some(ancho);
        self
    }

    pub fn separacion(mut self, separacion: f32) -> Self {
        self.separacion = separacion;
        self
    }

    pub fn a_la_derecha(mut self) -> Self {
        self.derecha = true;
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn sombra(mut self, sombra: Sombra) -> Self {
        self.sombra = sombra;
        self
    }

    pub fn alto_maximo(mut self, alto: f32) -> Self {
        self.alto_maximo = Some(alto);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn duracion(mut self, duracion: f32) -> Self {
        self.duracion = duracion;
        self
    }

    pub fn abierto(ctx: &Context, id: Id) -> bool {
        Popup::is_id_open(ctx, id)
    }

    pub fn alternar(ctx: &Context, id: Id) {
        Popup::toggle_id(ctx, id);
    }

    pub fn cerrar(ctx: &Context, id: Id) {
        Popup::close_id(ctx, id);
    }

    pub fn mostrar<R>(self, ctx: &Context, contenido: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let abierto = self.visible.unwrap_or_else(|| Popup::is_id_open(ctx, self.id));
        let avance = aparicion(ctx, self.id.with("pop"), abierto, self.duracion);
        let popup = Popup::new(self.id, ctx.clone(), self.ancla, self.capa)
            .align(if self.derecha { RectAlign::BOTTOM_END } else { RectAlign::BOTTOM_START })
            .gap(self.separacion)
            .frame(Frame::NONE);
        let popup = match self.visible {
            Some(v) => popup.open(v).close_behavior(PopupCloseBehavior::IgnoreClicks),
            None => popup.open_memory(None).close_behavior(PopupCloseBehavior::CloseOnClickOutside),
        };
        let popup = match self.ancho {
            Some(a) => popup.width(a),
            None => popup,
        };
        let origen = egui::pos2(if self.derecha { self.ancla.right() } else { self.ancla.left() }, self.ancla.bottom() + self.separacion);
        let (radio_px, sombra_pop, relleno, alto_maximo, ancho) = (self.radio, self.sombra, self.relleno, self.alto_maximo, self.ancho);
        popup
            .show(|ui| {
                ui.set_opacity(avance);
                escalar_capa(ui.ctx(), ui.layer_id(), origen, escala_pop(avance));
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let fondo = ui.painter().add(Shape::Noop);
                let marco = Frame::NONE.inner_margin(Margin::same(relleno as i8)).show(ui, |ui| {
                    if let Some(a) = ancho {
                        ui.set_min_width(a - relleno * 2.0);
                        ui.set_max_width(a - relleno * 2.0);
                    }
                    match alto_maximo {
                        Some(h) => ScrollArea::vertical().max_height(h - relleno * 2.0).show(ui, contenido).inner,
                        None => contenido(ui),
                    }
                });
                let rect = marco.response.rect;
                ui.painter().set(
                    fondo,
                    Shape::Vec(vec![
                        sombra_pop.forma(rect, radio_px),
                        Shape::Rect(egui::epaint::RectShape::new(rect, radio_px, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside)),
                    ]),
                );
                marco.inner
            })
            .map(|r| r.inner)
    }
}

pub fn desplegable(ui: &mut Ui, id: impl Hash + std::fmt::Debug, seleccion: &mut usize, opciones: &[String], sugerencia: &str, ancho: f32, alto: f32) -> Response {
    let id = ui.make_persistent_id(id);
    let abierto = Popup::is_id_open(ui.ctx(), id);
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::click());
    let borde = if abierto {
        color::VERDE
    } else if respuesta.hovered() {
        color::BORDE_HOVER
    } else {
        color::BORDE
    };
    ui.painter().rect(rect, radio::CONTROL, color::BLANCO, Stroke::new(1.0, borde), StrokeKind::Inside);
    let (contenido, texto_color) = match opciones.get(*seleccion) {
        Some(o) => (o.as_str(), color::TEXTO),
        None => (sugerencia, color::TEXTO_SUGERENCIA),
    };
    let g = galeria(ui, contenido, fuente(tam::CONTROL, Peso::Normal), texto_color);
    let pintor = ui.painter().with_clip_rect(rect.shrink2(Vec2::new(12.0, 0.0)).with_max_x(rect.right() - 34.0));
    pintor.galley(egui::pos2(rect.left() + 12.0, rect.center().y - g.size().y / 2.0), g, texto_color);
    pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(rect.right() - 18.0, rect.center().y), Vec2::splat(16.0)), "chevron-down", color::TEXTO_TENUE);
    if respuesta.clicked() {
        Popover::alternar(ui.ctx(), id);
    }
    let mut elegido = None;
    Popover::new(id, &respuesta).ancho(ancho).alto_maximo(260.0).mostrar(ui.ctx(), |ui| {
        for (indice, opcion) in opciones.iter().enumerate() {
            let activo = indice == *seleccion;
            let (caja, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
            let fondo = if activo {
                color::VERDE_ELEGIDO
            } else if r.hovered() {
                color::FONDO_APP
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect_filled(caja, radio::ICONO, fondo);
            let g = galeria(ui, opcion.as_str(), fuente(tam::CONTROL, if activo { Peso::Semi } else { Peso::Normal }), color::TEXTO);
            ui.painter().galley(egui::pos2(caja.left() + 10.0, caja.center().y - g.size().y / 2.0), g, color::TEXTO);
            if activo {
                pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(caja.right() - 18.0, caja.center().y), Vec2::splat(16.0)), "check", color::VERDE);
            }
            if r.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                elegido = Some(indice);
            }
        }
    });
    if let Some(indice) = elegido {
        Popover::cerrar(ui.ctx(), id);
        if indice != *seleccion {
            *seleccion = indice;
            respuesta.mark_changed();
        }
    }
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

#[derive(Clone, Copy, Debug)]
pub struct ItemMenu<'a> {
    pub icono: &'a str,
    pub etiqueta: &'a str,
    pub tecla: Option<&'a str>,
    pub peligro: bool,
}

pub fn item_menu(ui: &mut Ui, item: &ItemMenu) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::click());
    if respuesta.hovered() {
        ui.painter().rect_filled(rect, radio::CHIP_ICONO, color::FONDO_APP);
    }
    let (texto_color, icono_color) = if item.peligro { (color::ROJO, color::ROJO) } else { (color::TEXTO, color::VERDE) };
    pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(rect.left() + 19.0, rect.center().y), Vec2::splat(18.0)), item.icono, icono_color);
    let g = galeria(ui, item.etiqueta, fuente(tam::CUERPO, Peso::Normal), texto_color);
    ui.painter().galley(egui::pos2(rect.left() + 38.0, rect.center().y - g.size().y / 2.0), g, texto_color);
    if let Some(t) = item.tecla {
        let g = galeria(ui, t, fuente_mono(tam::MINI, Peso::Normal), color::TEXTO_TENUE);
        ui.painter().galley(egui::pos2(rect.right() - 10.0 - g.size().x, rect.center().y - g.size().y / 2.0), g, color::TEXTO_TENUE);
    }
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

#[derive(Clone, Debug)]
pub struct Usuario<'a> {
    pub nombre: &'a str,
    pub email: &'a str,
    pub detalle: &'a str,
}

pub fn menu_usuario(ui: &mut Ui, id: impl Hash + std::fmt::Debug, usuario: &Usuario, items: &[ItemMenu], compacto: bool) -> Option<usize> {
    let id = ui.make_persistent_id(id);
    let abierto = Popup::is_id_open(ui.ctx(), id);
    let nombre = galeria(ui, usuario.nombre, fuente(tam::CONTROL, Peso::Medio), color::BLANCO);
    let detalle = galeria(ui, usuario.detalle, fuente(tam::MINI, Peso::Normal), color::SOBRE_OSCURO_APAGADO);
    let ancho_textos = if compacto { 0.0 } else { nombre.size().x.max(detalle.size().x) + 10.0 };
    let ancho = 6.0 + 32.0 + 10.0 + ancho_textos + 16.0 + 10.0;
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 44.0), Sense::click());
    if abierto || respuesta.hovered() {
        ui.painter().rect_filled(rect, radio::CAMPO, color::VERDE_HOVER_OSCURO);
    }
    let caja_avatar = Rect::from_min_size(egui::pos2(rect.left() + 6.0, rect.center().y - 16.0), Vec2::splat(32.0));
    Avatar::new(super::iniciales(usuario.nombre)).pintar(ui, caja_avatar);
    let mut x = caja_avatar.right() + 10.0;
    if !compacto {
        let alto_textos = nombre.size().y + detalle.size().y;
        let y = rect.center().y - alto_textos / 2.0;
        let alto_nombre = nombre.size().y;
        ui.painter().galley(egui::pos2(x, y), nombre, color::BLANCO);
        ui.painter().galley(egui::pos2(x, y + alto_nombre), detalle, color::SOBRE_OSCURO_APAGADO);
        x += ancho_textos;
    }
    pintar_icono_solar_en(ui, Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), Vec2::splat(16.0)), "chevron-down", color::SOBRE_OSCURO_APAGADO);
    if respuesta.clicked() {
        Popover::alternar(ui.ctx(), id);
    }
    let elegido = Popover::new(id, &respuesta)
        .ancho(250.0)
        .separacion(8.0)
        .a_la_derecha()
        .radio(radio::POPOVER)
        .sombra(sombra::POPOVER)
        .duracion(DURACION_MENU)
        .mostrar(ui.ctx(), |ui| {
            Frame::NONE.inner_margin(Margin { left: 10, right: 10, top: 10, bottom: 12 }).show(ui, |ui| {
                ui.set_width(ui.available_width());
                super::texto(ui, usuario.nombre, tam::CUERPO, Peso::Semi, color::TEXTO);
                ui.add_space(2.0);
                super::texto(ui, usuario.email, tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
            });
            super::separador(ui);
            ui.add_space(4.0);
            let mut elegido = None;
            for (indice, item) in items.iter().enumerate() {
                if item_menu(ui, item).clicked() {
                    elegido = Some(indice);
                }
            }
            elegido
        })
        .flatten();
    if elegido.is_some() {
        Popover::cerrar(ui.ctx(), id);
    }
    elegido
}
