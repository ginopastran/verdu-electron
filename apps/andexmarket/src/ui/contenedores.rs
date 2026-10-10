use std::hash::Hash;

use eframe::egui::{
    self, Color32, Context, CursorIcon, Frame, Id, Layout, Popup, PopupCloseBehavior, Rect, RectAlign, Response,
    Sense, StrokeKind, Ui, Vec2,
};

use super::{borde, color, fuente, margen, marco, paleta, sombra_lg, sombra_md, sombra_sm, texto, Peso, Tam, RADIO, RADIO_MD, RADIO_SM, RADIO_XL};

pub fn tarjeta() -> Frame {
    marco()
        .fill(color::FONDO)
        .stroke(borde(color::BORDE))
        .corner_radius(RADIO)
        .shadow(sombra_sm())
}

pub fn titulo_tarjeta(ui: &mut Ui, contenido: &str) -> Response {
    ui.label(texto(contenido, Tam::X2, Peso::Semi, color::TEXTO))
}

pub fn descripcion(ui: &mut Ui, contenido: &str) -> Response {
    ui.add(egui::Label::new(texto(contenido, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO)).wrap())
}

pub struct RespuestaDialogo<R> {
    pub interior: R,
    pub cerrar: bool,
}

pub struct Dialogo {
    id: Id,
    ancho: f32,
    cerrable: bool,
    relleno: f32,
    radio: u8,
    fondo: Color32,
}

impl Dialogo {
    pub fn new(id: impl Hash + std::fmt::Debug) -> Self {
        Dialogo { id: Id::new("dialogo").with(id), ancho: 512.0, cerrable: true, relleno: 24.0, radio: RADIO, fondo: color::FONDO }
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = ancho;
        self
    }

    pub fn sin_cerrar(mut self) -> Self {
        self.cerrable = false;
        self
    }

    pub fn relleno(mut self, relleno: f32) -> Self {
        self.relleno = relleno;
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn fondo(mut self, fondo: Color32) -> Self {
        self.fondo = fondo;
        self
    }

    pub fn mostrar<R>(self, ctx: &Context, contenido: impl FnOnce(&mut Ui) -> R) -> RespuestaDialogo<R> {
        let relleno = self.relleno;
        let ancho = self.ancho.min(ctx.content_rect().width() - 32.0);
        let cerrable = self.cerrable;
        let mut x_clic = false;
        let respuesta = egui::Modal::new(self.id)
            .backdrop_color(Color32::from_black_alpha(204))
            .frame(
                marco()
                    .fill(self.fondo)
                    .stroke(borde(color::BORDE))
                    .corner_radius(self.radio)
                    .shadow(sombra_lg())
                    .inner_margin(egui::Margin::same(relleno as i8)),
            )
            .show(ctx, |ui| {
                ui.set_width(ancho - relleno * 2.0);
                let caja = ui.max_rect();
                let interior = contenido(ui);
                if cerrable {
                    let x = Rect::from_min_size(
                        egui::pos2(caja.right() + relleno - 32.0, caja.top() - relleno + 16.0),
                        Vec2::splat(16.0),
                    );
                    let r = ui.interact(x, ui.id().with("cerrar"), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
                    let opacidad = if r.hovered() { 1.0 } else { 0.7 };
                    super::pintar_icono_en(ui, x, "x", color::TEXTO.gamma_multiply(opacidad));
                    x_clic = r.clicked();
                }
                interior
            });
        RespuestaDialogo {
            cerrar: x_clic || (cerrable && respuesta.should_close()),
            interior: respuesta.inner,
        }
    }
}

pub fn encabezado_dialogo(ui: &mut Ui, titulo: &str, descripcion_texto: Option<&str>) {
    ui.label(texto(titulo, Tam::Lg, Peso::Semi, color::TEXTO));
    if let Some(d) = descripcion_texto {
        ui.add_space(6.0);
        descripcion(ui, d);
    }
}

pub fn pie_dialogo<R>(ui: &mut Ui, botones: impl FnOnce(&mut Ui) -> R) -> R {
    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        botones(ui)
    })
    .inner
}

pub fn marco_popup() -> Frame {
    marco()
        .fill(color::FONDO)
        .stroke(borde(color::BORDE))
        .corner_radius(RADIO_MD)
        .shadow(sombra_lg())
        .inner_margin(egui::Margin::same(4))
}

pub fn menu<R>(boton: &Response, ancho: f32, contenido: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    Popup::menu(boton)
        .align(RectAlign::BOTTOM_END)
        .gap(4.0)
        .width(ancho)
        .frame(marco_popup())
        .show(|ui| {
            ui.set_width(ancho - 8.0);
            contenido(ui)
        })
        .map(|r| r.inner)
}

pub fn item_menu(ui: &mut Ui, icono: Option<&str>, contenido: &str) -> Response {
    item_menu_colores(ui, icono, contenido, None)
}

pub fn item_menu_colores(ui: &mut Ui, icono: Option<&str>, contenido: &str, degradado: Option<(Color32, Color32)>) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
    let texto_color = match degradado {
        Some((desde, hasta)) => {
            super::pintar_degradado(ui.painter(), rect, RADIO_SM, desde, hasta, super::Direccion::Derecha);
            Color32::WHITE
        }
        None => {
            if respuesta.hovered() {
                ui.painter().rect_filled(rect, RADIO_SM, color::ACENTO);
            }
            color::TEXTO
        }
    };
    let mut x = rect.left() + 8.0;
    if let Some(nombre) = icono {
        let caja = Rect::from_min_size(egui::pos2(x, rect.center().y - 8.0), Vec2::splat(16.0));
        super::pintar_icono_en(ui, caja, nombre, texto_color);
        x += 24.0;
    }
    ui.painter().text(egui::pos2(x, rect.center().y), egui::Align2::LEFT_CENTER, contenido, fuente(Tam::Sm, Peso::Normal), texto_color);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn separador_menu(ui: &mut Ui) {
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width() + 8.0, 1.0), Sense::hover());
    ui.painter().rect_filled(rect.translate(Vec2::new(-4.0, 0.0)), 0, color::APAGADO);
    ui.add_space(4.0);
}

pub fn selector(ui: &mut Ui, id: impl Hash + std::fmt::Debug, seleccion: &mut usize, opciones: &[String], sugerencia: &str, ancho: Option<f32>) -> Response {
    let ancho = ancho.unwrap_or_else(|| ui.available_width());
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 40.0), Sense::click());
    ui.painter().rect(rect, RADIO_XL, color::FONDO, borde(color::BORDE_ENTRADA), StrokeKind::Inside);
    let (contenido, texto_color) = match opciones.get(*seleccion) {
        Some(o) => (o.as_str(), color::TEXTO),
        None => (sugerencia, color::APAGADO_TEXTO),
    };
    ui.painter().text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        contenido,
        fuente(Tam::Sm, Peso::Normal),
        texto_color,
    );
    let flecha = Rect::from_center_size(egui::pos2(rect.right() - 20.0, rect.center().y), Vec2::splat(16.0));
    super::pintar_icono_en(ui, flecha, "chevron-down", color::TEXTO.gamma_multiply(0.5));
    let popup_id = Id::new("selector").with(id);
    if respuesta.clicked() {
        Popup::toggle_id(ui.ctx(), popup_id);
    }
    let mut elegido = None;
    Popup::new(popup_id, ui.ctx().clone(), rect, ui.layer_id())
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .align(RectAlign::BOTTOM_START)
        .gap(4.0)
        .width(ancho)
        .frame(marco_popup().shadow(sombra_md()))
        .show(|ui| {
            ui.set_width(ancho - 8.0);
            egui::ScrollArea::vertical().max_height(384.0).show(ui, |ui| {
                for (indice, opcion) in opciones.iter().enumerate() {
                    let (caja, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
                    if r.hovered() {
                        ui.painter().rect_filled(caja, RADIO_SM, color::ACENTO);
                    }
                    if indice == *seleccion {
                        let marca = Rect::from_center_size(egui::pos2(caja.left() + 15.0, caja.center().y), Vec2::splat(16.0));
                        super::pintar_icono_en(ui, marca, "check", color::TEXTO);
                    }
                    ui.painter().text(
                        egui::pos2(caja.left() + 32.0, caja.center().y),
                        egui::Align2::LEFT_CENTER,
                        opcion,
                        fuente(Tam::Sm, Peso::Normal),
                        color::TEXTO,
                    );
                    if r.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        elegido = Some(indice);
                    }
                }
            });
        });
    if let Some(indice) = elegido {
        if indice != *seleccion {
            *seleccion = indice;
            respuesta.mark_changed();
        }
    }
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn celda_encabezado(ui: &mut Ui, ancho: f32, contenido: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 48.0), Sense::hover());
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        contenido,
        fuente(Tam::Sm, Peso::Medio),
        color::APAGADO_TEXTO,
    );
}

pub fn fila_tabla(ui: &mut Ui, alto: f32, contenido: impl FnOnce(&mut Ui)) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::click());
    if respuesta.hovered() {
        ui.painter().rect_filled(rect, 0, color::APAGADO.gamma_multiply(0.5));
    }
    ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, borde(color::BORDE));
    let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(egui::Align::Center)));
    contenido(&mut hijo);
    respuesta
}

pub fn alerta(ui: &mut Ui, icono: &str, titulo: &str, cuerpo: &str, destructiva: bool) {
    let (trazo, texto_color) = if destructiva {
        (color::DESTRUCTIVO.gamma_multiply(0.5), color::DESTRUCTIVO)
    } else {
        (color::BORDE, color::TEXTO)
    };
    marco()
        .stroke(borde(trazo))
        .corner_radius(RADIO)
        .inner_margin(margen(16.0, 16.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                super::pintar_icono(ui, icono, 16.0, texto_color);
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(texto(titulo, Tam::Base, Peso::Medio, texto_color));
                    ui.add_space(4.0);
                    ui.label(texto(cuerpo, Tam::Sm, Peso::Normal, texto_color));
                });
            });
        });
}

pub fn fondo_gris(ui: &Ui) {
    ui.painter().rect_filled(ui.max_rect(), 0, paleta::GRAY_50);
}
