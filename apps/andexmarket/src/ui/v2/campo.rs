use eframe::egui::{self, Align, Color32, Id, Layout, Rect, Response, Sense, Shape, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2};

use super::boton::BotonIcono;
use crate::ui::marca::{alto, color, fuente, radio, tam};
use crate::ui::{pintar_icono_solar_en, Peso};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Contenedor {
    pub alto: f32,
    pub radio: u8,
    pub fondo: Color32,
    pub borde: Option<Color32>,
    pub error: bool,
    pub anillo: Option<Color32>,
    pub relleno_izquierdo: f32,
    pub relleno_derecho: f32,
    pub separacion: f32,
    pub multilinea: bool,
}

impl Default for Contenedor {
    fn default() -> Self {
        Contenedor {
            alto: alto::CAMPO,
            radio: radio::CAMPO,
            fondo: color::BLANCO,
            borde: Some(color::BORDE),
            error: false,
            anillo: None,
            relleno_izquierdo: 14.0,
            relleno_derecho: 14.0,
            separacion: 10.0,
            multilinea: false,
        }
    }
}

impl Contenedor {
    pub fn mostrar(&self, ui: &mut Ui, ancho: f32, contenido: impl FnOnce(&mut Ui) -> Response) -> Response {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, self.alto), Sense::hover());
        let fondo = ui.painter().add(Shape::Noop);
        let interior = Rect::from_min_max(
            egui::pos2(rect.left() + self.relleno_izquierdo, rect.top() + if self.multilinea { 10.0 } else { 0.0 }),
            egui::pos2(rect.right() - self.relleno_derecho, rect.bottom() - if self.multilinea { 10.0 } else { 0.0 }),
        );
        let disposicion = if self.multilinea { Layout::top_down(Align::Min) } else { Layout::left_to_right(Align::Center) };
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(interior).layout(disposicion));
        hijo.spacing_mut().item_spacing.x = self.separacion;
        let respuesta = contenido(&mut hijo);
        let enfocado = respuesta.has_focus();
        let mut formas = Vec::new();
        let borde = match (self.error, enfocado, self.borde) {
            (true, _, _) => Some(color::ROJO_ERROR_BORDE),
            (false, true, _) => Some(color::VERDE),
            (false, false, borde) => borde,
        };
        let anillo = if enfocado && !self.error { Some(self.anillo.unwrap_or(color::ANILLO_FOCO)) } else { self.anillo };
        if let Some(c) = anillo {
            formas.push(Shape::Rect(egui::epaint::RectShape::stroke(rect, self.radio, Stroke::new(3.0, c), StrokeKind::Outside)));
        }
        formas.push(Shape::Rect(egui::epaint::RectShape::new(
            rect,
            self.radio,
            self.fondo,
            borde.map_or(Stroke::NONE, |c| Stroke::new(1.0, c)),
            StrokeKind::Inside,
        )));
        ui.painter().set(fondo, Shape::Vec(formas));
        respuesta
    }
}

pub struct EntradaDesnuda<'a> {
    texto: &'a mut String,
    sugerencia: String,
    contrasena: bool,
    tam: f32,
    peso: Peso,
    id: Option<Id>,
    alineacion: Align,
    multilinea: bool,
    habilitado: bool,
    color: Color32,
    caracteres_maximos: Option<usize>,
}

pub fn entrada_desnuda(texto: &mut String) -> EntradaDesnuda<'_> {
    EntradaDesnuda {
        texto,
        sugerencia: String::new(),
        contrasena: false,
        tam: tam::ENTRADA,
        peso: Peso::Normal,
        id: None,
        alineacion: Align::Min,
        multilinea: false,
        habilitado: true,
        color: color::TEXTO,
        caracteres_maximos: None,
    }
}

impl<'a> EntradaDesnuda<'a> {
    pub fn sugerencia(mut self, sugerencia: impl Into<String>) -> Self {
        self.sugerencia = sugerencia.into();
        self
    }

    pub fn contrasena(mut self, contrasena: bool) -> Self {
        self.contrasena = contrasena;
        self
    }

    pub fn tam(mut self, px: f32) -> Self {
        self.tam = px;
        self
    }

    pub fn peso(mut self, peso: Peso) -> Self {
        self.peso = peso;
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn centrada(mut self) -> Self {
        self.alineacion = Align::Center;
        self
    }

    pub fn multilinea(mut self) -> Self {
        self.multilinea = true;
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.habilitado = habilitado;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn caracteres_maximos(mut self, maximo: usize) -> Self {
        self.caracteres_maximos = Some(maximo);
        self
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let fuente_entrada = fuente(self.tam, self.peso);
        let sugerencia = egui::RichText::new(self.sugerencia).font(fuente_entrada.clone()).color(color::TEXTO_SUGERENCIA);
        let editor = if self.multilinea { TextEdit::multiline(self.texto) } else { TextEdit::singleline(self.texto) };
        let mut editor = editor
            .frame(egui::Frame::NONE)
            .hint_text(sugerencia)
            .password(self.contrasena)
            .font(fuente_entrada)
            .text_color(self.color)
            .horizontal_align(self.alineacion)
            .vertical_align(if self.multilinea { Align::Min } else { Align::Center })
            .margin(egui::Margin::ZERO)
            .desired_width(ui.available_width())
            .interactive(self.habilitado);
        if self.multilinea {
            editor = editor.desired_rows(1);
        }
        if let Some(maximo) = self.caracteres_maximos {
            editor = editor.char_limit(maximo);
        }
        if let Some(id) = self.id {
            editor = editor.id(id);
        }
        let ancho = ui.available_width();
        let alto_minimo = if self.multilinea { ui.available_height() } else { 0.0 };
        ui.add_sized(Vec2::new(ancho, alto_minimo.max(self.tam * 1.3)), editor)
    }
}

pub struct RespuestaCampo {
    pub respuesta: Response,
    pub accion: bool,
}

pub struct CampoTexto<'a> {
    entrada: EntradaDesnuda<'a>,
    contenedor: Contenedor,
    ancho: Option<f32>,
    icono: Option<&'a str>,
    prefijo: Option<&'a str>,
    sufijo: Option<&'a str>,
    accion: Option<&'a str>,
}

impl<'a> CampoTexto<'a> {
    pub fn new(texto: &'a mut String) -> Self {
        CampoTexto { entrada: entrada_desnuda(texto), contenedor: Contenedor::default(), ancho: None, icono: None, prefijo: None, sufijo: None, accion: None }
    }

    pub fn sugerencia(mut self, sugerencia: impl Into<String>) -> Self {
        self.entrada = self.entrada.sugerencia(sugerencia);
        self
    }

    pub fn contrasena(mut self, contrasena: bool) -> Self {
        self.entrada = self.entrada.contrasena(contrasena);
        self
    }

    pub fn tam_texto(mut self, px: f32) -> Self {
        self.entrada = self.entrada.tam(px);
        self
    }

    pub fn peso(mut self, peso: Peso) -> Self {
        self.entrada = self.entrada.peso(peso);
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.entrada = self.entrada.id(id);
        self
    }

    pub fn centrado(mut self) -> Self {
        self.entrada = self.entrada.centrada();
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.entrada = self.entrada.habilitado(habilitado);
        self
    }

    pub fn caracteres_maximos(mut self, maximo: usize) -> Self {
        self.entrada = self.entrada.caracteres_maximos(maximo);
        self
    }

    pub fn multilinea(mut self, alto_minimo: f32) -> Self {
        self.entrada = self.entrada.multilinea();
        self.contenedor.multilinea = true;
        self.contenedor.alto = alto_minimo;
        self.contenedor.relleno_izquierdo = 12.0;
        self.contenedor.relleno_derecho = 12.0;
        self
    }

    pub fn alto(mut self, alto: f32) -> Self {
        self.contenedor.alto = alto;
        self
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = Some(ancho);
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.contenedor.radio = radio;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.contenedor.error = error;
        self
    }

    pub fn sin_borde(mut self) -> Self {
        self.contenedor.borde = None;
        self
    }

    pub fn anillo(mut self, anillo: Option<Color32>) -> Self {
        self.contenedor.anillo = anillo;
        self
    }

    pub fn relleno(mut self, izquierdo: f32, derecho: f32) -> Self {
        self.contenedor.relleno_izquierdo = izquierdo;
        self.contenedor.relleno_derecho = derecho;
        self
    }

    pub fn icono(mut self, nombre: &'a str) -> Self {
        self.icono = Some(nombre);
        self
    }

    pub fn prefijo(mut self, texto: &'a str) -> Self {
        self.prefijo = Some(texto);
        self.contenedor.separacion = 6.0;
        self
    }

    pub fn sufijo(mut self, texto: &'a str) -> Self {
        self.sufijo = Some(texto);
        self.contenedor.separacion = 6.0;
        self
    }

    pub fn accion(mut self, icono: &'a str) -> Self {
        self.accion = Some(icono);
        self.contenedor.relleno_derecho = 6.0;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> RespuestaCampo {
        let CampoTexto { entrada, contenedor, ancho, icono, prefijo, sufijo, accion } = self;
        let ancho = ancho.unwrap_or_else(|| ui.available_width());
        let mut accion_clic = false;
        let tam_afijo = entrada.tam.max(16.0).min(20.0);
        let respuesta = contenedor.mostrar(ui, ancho, |ui| {
            if let Some(nombre) = icono {
                let (caja, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                pintar_icono_solar_en(ui, caja, nombre, color::TEXTO_TENUE);
            }
            if let Some(p) = prefijo {
                super::texto(ui, p, tam_afijo, Peso::Normal, color::TEXTO_TENUE);
            }
            let reservado = sufijo.map_or(0.0, |s| super::ancho_texto(ui, s, tam_afijo, Peso::Normal) + 6.0) + if accion.is_some() { 44.0 } else { 0.0 };
            let ancho_entrada = (ui.available_width() - reservado).max(10.0);
            let respuesta = ui
                .allocate_ui_with_layout(Vec2::new(ancho_entrada, ui.available_height()), Layout::left_to_right(Align::Center), |ui| entrada.ui(ui))
                .inner;
            if let Some(s) = sufijo {
                super::texto(ui, s, tam_afijo, Peso::Normal, color::TEXTO_TENUE);
            }
            if let Some(nombre) = accion {
                accion_clic = BotonIcono::new(nombre).tam(34.0).tam_icono(18.0).colores(color::TEXTO_TENUE, color::FONDO_APP, color::TEXTO_TENUE).ui(ui).clicked();
            }
            respuesta
        });
        RespuestaCampo { respuesta, accion: accion_clic }
    }
}
