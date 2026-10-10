use eframe::egui::{self, epaint::Mesh, Align2, Color32, CornerRadius, Frame, Painter, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2};

use super::galeria_espaciada;
use crate::ui::degradado::mezclar;
use crate::ui::marca::{color, fuente, radio, tam};
use crate::ui::{pintar_icono_solar_en, Peso};

#[derive(Clone, Copy, Debug)]
pub struct Kpi<'a> {
    pub etiqueta: &'a str,
    pub valor: &'a str,
    pub detalle: &'a str,
    pub icono: &'a str,
    pub fondo_icono: Color32,
    pub color_icono: Color32,
    pub color_valor: Color32,
}

pub fn tarjeta_kpi(ui: &mut Ui, kpi: &Kpi, ancho: f32) -> Response {
    Frame::NONE
        .fill(color::BLANCO)
        .stroke(Stroke::new(1.0, color::BORDE))
        .corner_radius(radio::POPOVER)
        .inner_margin(egui::Margin::symmetric(20, 18))
        .show(ui, |ui| {
            ui.set_width(ancho - 42.0);
            ui.vertical(|ui| {
            let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::hover());
            super::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, kpi.etiqueta, tam::CONTROL, Peso::Medio, color::TEXTO_APAGADO);
            let chip = Rect::from_min_size(egui::pos2(fila.right() - 32.0, fila.top()), Vec2::splat(32.0));
            ui.painter().rect_filled(chip, radio::CHIP_ICONO, kpi.fondo_icono);
            pintar_icono_solar_en(ui, Rect::from_center_size(chip.center(), Vec2::splat(19.0)), kpi.icono, kpi.color_icono);
            ui.add_space(8.0);
            let g = galeria_espaciada(ui, kpi.valor, fuente(tam::KPI, Peso::Semi), kpi.color_valor, -0.02 * tam::KPI);
            let (rect, _) = ui.allocate_exact_size(g.size(), Sense::hover());
            ui.painter().galley(rect.min, g, kpi.color_valor);
            ui.add_space(8.0);
            super::texto(ui, kpi.detalle, tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
            });
        })
        .response
}

pub fn dato_compacto(ui: &mut Ui, etiqueta: &str, valor: &str, ancho: f32) -> Response {
    Frame::NONE
        .stroke(Stroke::new(1.0, color::BORDE_SUAVE))
        .corner_radius(radio::CAMPO)
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(ancho - 30.0);
            ui.vertical(|ui| {
                super::texto(ui, etiqueta, tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
                ui.add_space(2.0);
                super::texto(ui, valor, tam::GRANDE, Peso::Semi, color::TEXTO);
            });
        })
        .response
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ancho {
    Fijo(f32),
    Fraccion(f32, f32),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Columna {
    pub ancho: Ancho,
    pub derecha: bool,
}

impl Columna {
    pub const fn fija(ancho: f32) -> Self {
        Columna { ancho: Ancho::Fijo(ancho), derecha: false }
    }

    pub const fn fraccion(fraccion: f32, minimo: f32) -> Self {
        Columna { ancho: Ancho::Fraccion(fraccion, minimo), derecha: false }
    }

    pub const fn derecha(mut self) -> Self {
        self.derecha = true;
        self
    }
}

pub fn marco_tabla() -> Frame {
    Frame::NONE.fill(color::BLANCO).stroke(Stroke::new(1.0, color::BORDE)).corner_radius(radio::POPOVER)
}

pub struct Tabla<'a> {
    columnas: &'a [Columna],
    relleno_x: f32,
    separacion: f32,
    radio_superior: u8,
}

impl<'a> Tabla<'a> {
    pub fn new(columnas: &'a [Columna]) -> Self {
        Tabla { columnas, relleno_x: 18.0, separacion: 12.0, radio_superior: radio::POPOVER - 1 }
    }

    pub fn relleno(mut self, relleno_x: f32) -> Self {
        self.relleno_x = relleno_x;
        self
    }

    pub fn separacion(mut self, separacion: f32) -> Self {
        self.separacion = separacion;
        self
    }

    pub fn radio_superior(mut self, radio: u8) -> Self {
        self.radio_superior = radio;
        self
    }

    pub fn celdas(&self, fila: Rect) -> Vec<Rect> {
        anchos_columnas(self.columnas, fila.width() - self.relleno_x * 2.0, self.separacion)
            .into_iter()
            .scan(fila.left() + self.relleno_x, |x, ancho| {
                let celda = Rect::from_min_size(egui::pos2(*x, fila.top()), Vec2::new(ancho, fila.height()));
                *x += ancho + self.separacion;
                Some(celda)
            })
            .collect()
    }

    pub fn encabezado(&self, ui: &mut Ui, titulos: &[&str], alto_fila: f32) -> Response {
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto_fila), Sense::hover());
        let r = self.radio_superior;
        ui.painter().rect_filled(rect, CornerRadius { nw: r, ne: r, sw: 0, se: 0 }, color::FONDO_ENCABEZADO_TABLA);
        ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
        for ((celda, columna), titulo) in self.celdas(rect).into_iter().zip(self.columnas).zip(titulos) {
            celda_texto(ui.painter(), celda, titulo, tam::CHICO, Peso::Medio, color::TEXTO_APAGADO, columna.derecha);
        }
        respuesta
    }

    pub fn fila(&self, ui: &mut Ui, alto_fila: f32, fondo: Option<Color32>, contenido: impl FnOnce(&mut Ui, &[Rect])) -> Response {
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto_fila), Sense::click());
        let encima = ui.rect_contains_pointer(rect);
        if let Some(c) = if encima { Some(color::FILA_HOVER) } else { fondo } {
            ui.painter().rect_filled(rect, 0, c);
        }
        ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
        let celdas = self.celdas(rect);
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(rect));
        contenido(&mut hijo, &celdas);
        respuesta
    }
}

pub fn anchos_columnas(columnas: &[Columna], disponible: f32, separacion: f32) -> Vec<f32> {
    let fijos: f32 = columnas.iter().map(|c| if let Ancho::Fijo(a) = c.ancho { a } else { 0.0 }).sum();
    let fracciones: f32 = columnas.iter().map(|c| if let Ancho::Fraccion(f, _) = c.ancho { f } else { 0.0 }).sum();
    let libre = (disponible - fijos - separacion * columnas.len().saturating_sub(1) as f32).max(0.0);
    columnas
        .iter()
        .map(|c| match c.ancho {
            Ancho::Fijo(a) => a,
            Ancho::Fraccion(f, minimo) => (libre * f / fracciones.max(f32::EPSILON)).max(minimo),
        })
        .collect()
}

pub fn celda_texto(pintor: &Painter, celda: Rect, contenido: &str, px: f32, peso: Peso, color_texto: Color32, derecha: bool) {
    let g = pintor.layout(contenido.to_string(), fuente(px, peso), color_texto, f32::INFINITY);
    let g = if g.size().x > celda.width() {
        let mut trabajo = egui::text::LayoutJob::simple_singleline(contenido.to_string(), fuente(px, peso), color_texto);
        trabajo.wrap = egui::text::TextWrapping::truncate_at_width(celda.width());
        pintor.layout_job(trabajo)
    } else {
        g
    };
    let x = if derecha { celda.right() - g.size().x } else { celda.left() };
    pintor.galley(egui::pos2(x, celda.center().y - g.size().y / 2.0), g, color_texto);
}

pub fn shimmer(ui: &mut Ui, tam_caja: Vec2, radio_caja: u8, animado: bool) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(tam_caja, Sense::hover());
    pintar_shimmer(ui, rect, radio_caja, animado);
    respuesta
}

pub fn pintar_shimmer(ui: &Ui, rect: Rect, radio_caja: u8, animado: bool) {
    let pintor = ui.painter();
    pintor.rect_filled(rect, radio_caja, color::SHIMMER_BASE);
    if !animado {
        return;
    }
    let fase = (super::animacion::giro(ui.ctx(), 1.2) / std::f32::consts::TAU).clamp(0.0, 1.0);
    let centro_banda = rect.left() - 400.0 + 800.0 * fase + 80.0;
    let intensidad = |x: f32| {
        let mut d = (x - centro_banda).rem_euclid(800.0);
        if d > 400.0 {
            d -= 800.0;
        }
        (1.0 - d.abs() / 80.0).max(0.0)
    };
    let r = (radio_caja as f32).min(rect.width() / 2.0).min(rect.height() / 2.0);
    let margen_vertical = |x: f32| {
        let dx = (r - (x - rect.left())).max(r - (rect.right() - x)).max(0.0);
        if dx <= 0.0 {
            0.0
        } else {
            r - (r * r - dx.min(r) * dx.min(r)).sqrt()
        }
    };
    let mut malla = Mesh::default();
    let paso = 1.0;
    let mut x = rect.left();
    while x < rect.right() {
        let siguiente = (x + paso).min(rect.right());
        let medio = (x + siguiente) / 2.0;
        let t = intensidad(medio);
        if t > 0.0 {
            let m = margen_vertical(medio) + 0.5;
            let c = mezclar(color::SHIMMER_BASE, color::SHIMMER_BRILLO, t);
            malla.add_colored_rect(Rect::from_min_max(egui::pos2(x, rect.top() + m), egui::pos2(siguiente, rect.bottom() - m)), c);
        }
        x = siguiente;
    }
    pintor.add(egui::Shape::mesh(malla));
}

#[cfg(test)]
mod pruebas {
    use super::{anchos_columnas, Columna};

    #[test]
    fn reparte_las_fracciones_y_respeta_minimos() {
        let columnas = [Columna::fraccion(2.0, 0.0), Columna::fija(100.0), Columna::fraccion(1.0, 120.0)];
        let anchos = anchos_columnas(&columnas, 500.0, 10.0);
        assert_eq!(anchos[1], 100.0);
        assert!((anchos[0] - 253.333).abs() < 0.01);
        assert!((anchos[2] - 126.667).abs() < 0.01);
        let estrechos = anchos_columnas(&columnas, 200.0, 10.0);
        assert_eq!(estrechos[2], 120.0);
    }
}
