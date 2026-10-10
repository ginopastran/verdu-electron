use eframe::egui::{self, Color32, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use super::animacion::girando;
use super::insignias::{medir_tecla, pintar_tecla, EstiloTecla};
use super::{galeria, opacidad};
use crate::ui::marca::{alto, color, fuente, mezclar, radio, sombra, Sombra};
use crate::ui::{pintar_icono_solar_en, Peso};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Estilo {
    Primario,
    Secundario,
    Peligro,
    PeligroContorno,
    Dorado,
    ContornoMarca,
    Fantasma,
    FantasmaOscuro,
}

#[derive(Clone, Copy)]
struct Colores {
    fondo: Color32,
    fondo_hover: Color32,
    borde: Option<Color32>,
    borde_hover: Option<Color32>,
    texto: Color32,
    texto_hover: Color32,
    icono: Option<Color32>,
    tecla: EstiloTecla,
}

impl Estilo {
    fn colores(self) -> Colores {
        let base = Colores {
            fondo: color::BLANCO,
            fondo_hover: color::BLANCO,
            borde: Some(color::BORDE),
            borde_hover: Some(color::VERDE),
            texto: color::TEXTO,
            texto_hover: color::VERDE,
            icono: None,
            tecla: EstiloTecla::CLARA,
        };
        match self {
            Estilo::Primario => Colores {
                fondo: color::VERDE,
                fondo_hover: color::VERDE_PROFUNDO,
                borde: None,
                borde_hover: None,
                texto: color::BLANCO,
                texto_hover: color::BLANCO,
                tecla: EstiloTecla::EN_BOTON,
                ..base
            },
            Estilo::Secundario => base,
            Estilo::Peligro => Colores {
                fondo: color::ROJO_INTENSO,
                fondo_hover: color::ROJO,
                borde: None,
                borde_hover: None,
                texto: color::BLANCO,
                texto_hover: color::BLANCO,
                tecla: EstiloTecla::EN_BOTON,
                ..base
            },
            Estilo::PeligroContorno => Colores {
                fondo_hover: color::ROJO_FONDO,
                borde: Some(color::ROJO_BORDE),
                borde_hover: Some(color::ROJO_BORDE),
                texto: color::ROJO,
                texto_hover: color::ROJO,
                icono: Some(color::ROJO_INTENSO),
                ..base
            },
            Estilo::Dorado => Colores {
                fondo: color::DORADO_HOVER,
                fondo_hover: color::DORADO_FONDO,
                borde: Some(color::DORADO),
                borde_hover: Some(color::DORADO),
                texto: color::DORADO_TEXTO,
                texto_hover: color::DORADO_TEXTO,
                tecla: EstiloTecla::DORADA,
                ..base
            },
            Estilo::ContornoMarca => Colores {
                fondo_hover: color::VERDE_ELEGIDO,
                borde: Some(color::VERDE),
                texto: color::VERDE,
                ..base
            },
            Estilo::Fantasma => Colores {
                fondo: Color32::TRANSPARENT,
                fondo_hover: color::FONDO_APP,
                borde: None,
                borde_hover: None,
                texto: color::TEXTO_SECUNDARIO,
                texto_hover: color::VERDE,
                ..base
            },
            Estilo::FantasmaOscuro => Colores {
                fondo: Color32::TRANSPARENT,
                fondo_hover: color::VERDE_HOVER_OSCURO,
                borde: None,
                borde_hover: None,
                texto: color::SOBRE_OSCURO,
                texto_hover: color::BLANCO,
                icono: Some(color::SOBRE_OSCURO_ICONO),
                tecla: EstiloTecla::SOBRE_OSCURO,
                ..base
            },
        }
    }
}

pub struct Boton<'a> {
    texto: String,
    estilo: Estilo,
    icono: Option<&'a str>,
    tecla: Option<&'a str>,
    alto: f32,
    ancho: Option<f32>,
    relleno: Option<f32>,
    radio: u8,
    tam_texto: Option<f32>,
    peso: Option<Peso>,
    tam_icono: f32,
    separacion: f32,
    color_icono: Option<Color32>,
    sombra: Option<Sombra>,
    habilitado: bool,
    cargando: bool,
    activo: bool,
    repartido: bool,
}

impl<'a> Boton<'a> {
    pub fn new(texto: impl Into<String>, estilo: Estilo) -> Self {
        Boton {
            texto: texto.into(),
            estilo,
            icono: None,
            tecla: None,
            alto: alto::BOTON,
            ancho: None,
            relleno: None,
            radio: radio::CAMPO,
            tam_texto: None,
            peso: None,
            tam_icono: 17.0,
            separacion: 8.0,
            color_icono: None,
            sombra: None,
            habilitado: true,
            cargando: false,
            activo: false,
            repartido: false,
        }
    }

    pub fn primario(texto: impl Into<String>) -> Self {
        Self::new(texto, Estilo::Primario).sombra(sombra::BOTON)
    }

    pub fn secundario(texto: impl Into<String>) -> Self {
        Self::new(texto, Estilo::Secundario)
    }

    pub fn peligro(texto: impl Into<String>) -> Self {
        Self::new(texto, Estilo::Peligro)
    }

    pub fn icono(mut self, nombre: &'a str) -> Self {
        self.icono = Some(nombre);
        self
    }

    pub fn tecla(mut self, tecla: &'a str) -> Self {
        self.tecla = Some(tecla);
        self
    }

    pub fn alto(mut self, alto: f32) -> Self {
        self.alto = alto;
        self
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = Some(ancho);
        self
    }

    pub fn lleno(self, ui: &Ui) -> Self {
        let ancho = ui.available_width();
        self.ancho(ancho)
    }

    pub fn relleno(mut self, relleno: f32) -> Self {
        self.relleno = Some(relleno);
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn tam_texto(mut self, px: f32) -> Self {
        self.tam_texto = Some(px);
        self
    }

    pub fn peso(mut self, peso: Peso) -> Self {
        self.peso = Some(peso);
        self
    }

    pub fn tam_icono(mut self, tam: f32) -> Self {
        self.tam_icono = tam;
        self
    }

    pub fn separacion(mut self, separacion: f32) -> Self {
        self.separacion = separacion;
        self
    }

    pub fn color_icono(mut self, color: Color32) -> Self {
        self.color_icono = Some(color);
        self
    }

    pub fn sombra(mut self, sombra: Sombra) -> Self {
        self.sombra = Some(sombra);
        self
    }

    pub fn sin_sombra(mut self) -> Self {
        self.sombra = None;
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.habilitado = habilitado;
        self
    }

    pub fn cargando(mut self, cargando: bool) -> Self {
        self.cargando = cargando;
        self
    }

    pub fn activo(mut self, activo: bool) -> Self {
        self.activo = activo;
        self
    }

    pub fn repartido(mut self) -> Self {
        self.repartido = true;
        self
    }

    pub fn modal(self) -> Self {
        self.alto(alto::BOTON_MODAL).radio(radio::BOTON_MODAL).sin_sombra()
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let colores = self.estilo.colores();
        let fuerte = matches!(self.estilo, Estilo::Primario | Estilo::Peligro | Estilo::ContornoMarca);
        let px = self.tam_texto.unwrap_or(if fuerte { 14.5 } else { 14.0 });
        let peso = self.peso.unwrap_or(if fuerte { Peso::Semi } else { Peso::Medio });
        let relleno = self.relleno.unwrap_or(if fuerte { 18.0 } else { 16.0 });
        let texto_galeria = (!self.texto.is_empty()).then(|| galeria(ui, self.texto.clone(), fuente(px, peso), colores.texto));
        let icono = if self.cargando { Some("loader") } else { self.icono };
        let tam_tecla = self.tecla.map(|t| medir_tecla(ui, t, &colores.tecla));
        let mut piezas = Vec::new();
        if icono.is_some() {
            piezas.push(self.tam_icono);
        }
        if let Some(g) = &texto_galeria {
            piezas.push(g.size().x);
        }
        if let Some(t) = tam_tecla {
            piezas.push(t.x);
        }
        let contenido = piezas.iter().sum::<f32>() + piezas.len().saturating_sub(1) as f32 * self.separacion;
        let ancho = self.ancho.unwrap_or(if self.texto.is_empty() && self.tecla.is_none() { self.alto } else { contenido + relleno * 2.0 });
        let interactivo = self.habilitado && !self.cargando;
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, self.alto), if interactivo { Sense::click() } else { Sense::hover() });
        if !ui.is_rect_visible(rect) {
            return respuesta;
        }
        let encima = (respuesta.hovered() && interactivo) || self.activo;
        let avance = ui.ctx().animate_bool_with_time(respuesta.id.with("hover"), encima, 0.15);
        let alfa = if self.habilitado { 1.0 } else { 0.45 };
        let fondo = opacidad(mezclar(colores.fondo, colores.fondo_hover, avance), alfa);
        let pintor = ui.painter();
        if let Some(s) = self.sombra {
            pintor.add(s.con_opacidad(alfa).forma(rect, self.radio));
        }
        let trazo = match (colores.borde, colores.borde_hover) {
            (Some(a), Some(b)) => Stroke::new(1.0, opacidad(mezclar(a, b, avance), alfa)),
            (Some(a), None) => Stroke::new(1.0, opacidad(a, alfa)),
            _ => Stroke::NONE,
        };
        pintor.rect(rect, self.radio, fondo, trazo, StrokeKind::Inside);
        let texto_color = opacidad(mezclar(colores.texto, colores.texto_hover, avance), alfa);
        let icono_color = opacidad(self.color_icono.or(colores.icono).unwrap_or(texto_color), alfa);
        let mut x = if self.repartido { rect.left() + relleno } else { rect.center().x - contenido / 2.0 };
        if let Some(nombre) = icono {
            let caja = Rect::from_center_size(egui::pos2(x + self.tam_icono / 2.0, rect.center().y), Vec2::splat(self.tam_icono));
            if self.cargando {
                girando(ui, caja, icono_color);
            } else {
                pintar_icono_solar_en(ui, caja, nombre, icono_color);
            }
            x += self.tam_icono + self.separacion;
        }
        if let Some(g) = texto_galeria {
            let ancho_texto = g.size().x;
            ui.painter().galley(egui::pos2(x, rect.center().y - g.size().y / 2.0), g, texto_color);
            x += ancho_texto + self.separacion;
        }
        if let (Some(t), Some(tam)) = (self.tecla, tam_tecla) {
            let x = if self.repartido { rect.right() - relleno - tam.x } else { x };
            let caja = Rect::from_min_size(egui::pos2(x, rect.center().y - tam.y / 2.0), tam);
            pintar_tecla(ui, caja, t, &colores.tecla, texto_color);
        }
        if interactivo {
            respuesta.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            respuesta
        }
    }
}

pub struct BotonIcono<'a> {
    icono: &'a str,
    tam: f32,
    tam_icono: f32,
    radio: u8,
    color: Color32,
    fondo_hover: Color32,
    color_hover: Color32,
    habilitado: bool,
    cargando: bool,
}

impl<'a> BotonIcono<'a> {
    pub fn new(icono: &'a str) -> Self {
        BotonIcono {
            icono,
            tam: 32.0,
            tam_icono: 17.0,
            radio: radio::ICONO,
            color: color::TEXTO_APAGADO,
            fondo_hover: color::FONDO_APP,
            color_hover: color::VERDE,
            habilitado: true,
            cargando: false,
        }
    }

    pub fn cerrar() -> Self {
        BotonIcono { color_hover: color::TEXTO_APAGADO, ..Self::new("x") }.tam(32.0).tam_icono(20.0)
    }

    pub fn quitar() -> Self {
        Self::new("trash-2").colores(color::TEXTO_DESHABILITADO, color::ROJO_FONDO, color::ROJO)
    }

    pub fn tam(mut self, tam: f32) -> Self {
        self.tam = tam;
        self
    }

    pub fn tam_icono(mut self, tam: f32) -> Self {
        self.tam_icono = tam;
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn colores(mut self, color: Color32, fondo_hover: Color32, color_hover: Color32) -> Self {
        self.color = color;
        self.fondo_hover = fondo_hover;
        self.color_hover = color_hover;
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.habilitado = habilitado;
        self
    }

    pub fn cargando(mut self, cargando: bool) -> Self {
        self.cargando = cargando;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let interactivo = self.habilitado && !self.cargando;
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(self.tam), if interactivo { Sense::click() } else { Sense::hover() });
        let encima = respuesta.hovered() && interactivo;
        let alfa = if self.habilitado { 1.0 } else { 0.45 };
        if encima {
            ui.painter().rect_filled(rect, self.radio, self.fondo_hover);
        }
        let color_icono = opacidad(if encima { self.color_hover } else { self.color }, alfa);
        let caja = Rect::from_center_size(rect.center(), Vec2::splat(self.tam_icono));
        if self.cargando {
            girando(ui, caja, color_icono);
        } else {
            pintar_icono_solar_en(ui, caja, self.icono, color_icono);
        }
        if interactivo {
            respuesta.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            respuesta
        }
    }
}
