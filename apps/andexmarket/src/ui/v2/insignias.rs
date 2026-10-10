use eframe::egui::{self, Color32, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use super::animacion::pulso;
use super::galeria;
use crate::ui::marca::{color, fuente, fuente_mono, radio, sombra, tam, Sombra};
use crate::ui::{pintar_icono_solar_en, Peso};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct EstiloTecla {
    pub tam: f32,
    pub peso: Peso,
    pub relleno: Vec2,
    pub radio: u8,
    pub fondo: Color32,
    pub borde: Option<Color32>,
    pub borde_inferior: f32,
    pub texto: Option<Color32>,
    pub lado: Option<f32>,
}

impl EstiloTecla {
    pub const CLARA: EstiloTecla = EstiloTecla {
        tam: tam::XS,
        peso: Peso::Semi,
        relleno: Vec2::new(7.0, 2.0),
        radio: radio::CHIP,
        fondo: color::FONDO_ENCABEZADO_TABLA,
        borde: Some(color::BORDE),
        borde_inferior: 2.0,
        texto: Some(color::TEXTO_SECUNDARIO),
        lado: None,
    };
    pub const EN_LINEA: EstiloTecla = EstiloTecla {
        tam: tam::XS,
        peso: Peso::Normal,
        relleno: Vec2::new(6.0, 1.0),
        radio: radio::TECLA,
        fondo: color::BLANCO,
        ..Self::CLARA
    };
    pub const CONTORNO: EstiloTecla = EstiloTecla {
        tam: tam::MINI,
        peso: Peso::Normal,
        relleno: Vec2::new(6.0, 1.0),
        radio: radio::TECLA,
        fondo: Color32::TRANSPARENT,
        borde_inferior: 1.0,
        texto: None,
        ..Self::CLARA
    };
    pub const EN_BOTON: EstiloTecla = EstiloTecla {
        tam: tam::DIMINUTO,
        peso: Peso::Normal,
        relleno: Vec2::new(5.0, 1.0),
        radio: radio::TECLA,
        fondo: color::BLANCO_18,
        borde: None,
        borde_inferior: 0.0,
        texto: None,
        lado: None,
    };
    pub const DORADA: EstiloTecla = EstiloTecla {
        fondo: Color32::TRANSPARENT,
        borde: Some(color::DORADO_BORDE),
        borde_inferior: 1.0,
        ..Self::EN_BOTON
    };
    pub const SOBRE_OSCURO: EstiloTecla = EstiloTecla {
        tam: tam::CHICO,
        peso: Peso::Semi,
        relleno: Vec2::new(8.0, 3.0),
        radio: radio::TECLA_CUADRADA,
        fondo: color::BLANCO_12,
        borde: Some(color::BLANCO_20),
        borde_inferior: 1.0,
        texto: Some(color::BLANCO),
        lado: None,
    };
    pub const CUADRADA: EstiloTecla = EstiloTecla {
        tam: tam::XS,
        peso: Peso::Semi,
        relleno: Vec2::ZERO,
        radio: radio::TECLA_CUADRADA,
        fondo: color::BLANCO,
        borde: Some(color::BORDE),
        borde_inferior: 2.0,
        texto: Some(color::TEXTO_SECUNDARIO),
        lado: Some(24.0),
    };
    pub const CUADRADA_OSCURA: EstiloTecla = EstiloTecla {
        fondo: color::BLANCO_10,
        borde: Some(color::BLANCO_25),
        texto: Some(color::BLANCO),
        ..Self::CUADRADA
    };
    pub const NUMERO_AVATAR: EstiloTecla = EstiloTecla {
        tam: tam::ETIQUETA,
        radio: radio::ICONO,
        fondo: color::BLANCO_18,
        borde: Some(color::BLANCO_25),
        borde_inferior: 1.0,
        texto: Some(color::BLANCO),
        lado: Some(26.0),
        ..Self::CUADRADA
    };
}

pub fn medir_tecla(ui: &Ui, contenido: &str, estilo: &EstiloTecla) -> Vec2 {
    if let Some(lado) = estilo.lado {
        return Vec2::splat(lado);
    }
    let g = galeria(ui, contenido, fuente_mono(estilo.tam, estilo.peso), Color32::WHITE);
    let alto_borde = if estilo.borde.is_some() { 1.0 + estilo.borde_inferior } else { 0.0 };
    Vec2::new(g.size().x + estilo.relleno.x * 2.0 + if estilo.borde.is_some() { 2.0 } else { 0.0 }, g.size().y + estilo.relleno.y * 2.0 + alto_borde)
}

pub fn pintar_tecla(ui: &Ui, rect: Rect, contenido: &str, estilo: &EstiloTecla, heredado: Color32) {
    let pintor = ui.painter();
    let interior = match estilo.borde {
        Some(borde) => {
            let interior = Rect::from_min_max(rect.min + Vec2::splat(1.0), rect.max - Vec2::new(1.0, estilo.borde_inferior));
            if estilo.fondo.is_opaque() {
                pintor.rect_filled(rect, estilo.radio, borde);
                pintor.rect_filled(interior, estilo.radio.saturating_sub(1), estilo.fondo);
            } else {
                pintor.rect(rect, estilo.radio, estilo.fondo, Stroke::new(1.0, borde), StrokeKind::Inside);
                if estilo.borde_inferior > 1.0 {
                    let x = rect.x_range().shrink(estilo.radio as f32);
                    pintor.hline(x, rect.bottom() - 1.5, Stroke::new(1.0, borde));
                }
            }
            interior
        }
        None => {
            pintor.rect_filled(rect, estilo.radio, estilo.fondo);
            rect
        }
    };
    let texto_color = estilo.texto.unwrap_or(heredado);
    let g = galeria(ui, contenido, fuente_mono(estilo.tam, estilo.peso), texto_color);
    pintor.galley(interior.center() - g.size() / 2.0, g, texto_color);
}

pub fn tecla(ui: &mut Ui, contenido: &str, estilo: &EstiloTecla) -> Response {
    let tam_tecla = medir_tecla(ui, contenido, estilo);
    let (rect, respuesta) = ui.allocate_exact_size(tam_tecla, Sense::hover());
    pintar_tecla(ui, rect, contenido, estilo, color::TEXTO_SECUNDARIO);
    respuesta
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tono {
    Marca,
    Exito,
    Aviso,
    Peligro,
    Azul,
    Violeta,
    Admin,
}

impl Tono {
    pub fn colores(self) -> (Color32, Color32) {
        match self {
            Tono::Marca => (color::VERDE_CHIP, color::VERDE),
            Tono::Exito => (color::VERDE_EXITO_FONDO, color::VERDE_EXITO),
            Tono::Aviso => (color::DORADO_FONDO, color::DORADO_TEXTO),
            Tono::Peligro => (color::ROJO_FONDO, color::ROJO),
            Tono::Azul => (color::AZUL_FONDO, color::AZUL),
            Tono::Violeta => (color::VIOLETA_FONDO, color::VIOLETA),
            Tono::Admin => (color::DORADO, color::VERDE_PROFUNDO),
        }
    }
}

fn pastilla(ui: &mut Ui, icono: Option<&str>, contenido: &str, px: f32, peso: Peso, relleno: Vec2, radio_px: u8, fondo: Color32, texto_color: Color32) -> Response {
    let g = galeria(ui, contenido, fuente(px, peso), texto_color);
    let tam_icono = 13.0;
    let extra = if icono.is_some() { tam_icono + 5.0 } else { 0.0 };
    let tam_total = Vec2::new(g.size().x + extra + relleno.x * 2.0, g.size().y + relleno.y * 2.0);
    let (rect, respuesta) = ui.allocate_exact_size(tam_total, Sense::hover());
    ui.painter().rect_filled(rect, radio_px, fondo);
    let mut x = rect.left() + relleno.x;
    if let Some(nombre) = icono {
        pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(x + tam_icono / 2.0, rect.center().y), Vec2::splat(tam_icono)), nombre, texto_color);
        x += extra;
    }
    ui.painter().galley(egui::pos2(x, rect.center().y - g.size().y / 2.0), g, texto_color);
    respuesta
}

pub fn chip(ui: &mut Ui, contenido: &str, tono: Tono) -> Response {
    let (fondo, texto_color) = tono.colores();
    pastilla(ui, None, contenido, tam::MINI, Peso::Semi, Vec2::new(7.0, 1.0), radio::CHIP, fondo, texto_color)
}

pub fn chip_estado(ui: &mut Ui, contenido: &str, tono: Tono) -> Response {
    let (fondo, texto_color) = tono.colores();
    pastilla(ui, None, contenido, tam::XS, Peso::Semi, Vec2::new(8.0, 2.0), radio::CHIP, fondo, texto_color)
}

pub fn chip_metodo(ui: &mut Ui, icono: &str, contenido: &str, tono: Tono) -> Response {
    let (fondo, texto_color) = tono.colores();
    pastilla(ui, Some(icono), contenido, tam::XS, Peso::Semi, Vec2::new(9.0, 3.0), radio::PASTILLA, fondo, texto_color)
}

pub fn contador(ui: &mut Ui, contenido: &str, fondo: Color32, texto_color: Color32) -> Response {
    pastilla(ui, None, contenido, tam::XS, Peso::Medio, Vec2::new(7.0, 1.0), radio::PASTILLA, fondo, texto_color)
}

pub fn punto_estado(ui: &mut Ui, contenido: &str, color_punto: Color32, color_texto: Color32, px: f32, peso: Peso, latiendo: bool) -> Response {
    let g = galeria(ui, contenido, fuente(px, peso), color_texto);
    let diametro = if px >= tam::ETIQUETA { 8.0 } else { 7.0 };
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(diametro + 6.0 + g.size().x, g.size().y), Sense::hover());
    let alfa = if latiendo { pulso(ui.ctx(), 0.9) } else { 1.0 };
    ui.painter().circle_filled(egui::pos2(rect.left() + diametro / 2.0, rect.center().y), diametro / 2.0, color_punto.gamma_multiply(alfa));
    ui.painter().galley(egui::pos2(rect.left() + diametro + 6.0, rect.center().y - g.size().y / 2.0), g, color_texto);
    respuesta
}

pub fn iniciales(nombre: &str) -> String {
    nombre.split_whitespace().filter_map(|p| p.chars().next()).flat_map(char::to_uppercase).collect()
}

pub struct Avatar {
    texto: String,
    lado: f32,
    radio: u8,
    fondo: Color32,
    color: Color32,
    tam_texto: f32,
    peso: Peso,
    sombra: Option<Sombra>,
}

impl Avatar {
    pub fn new(texto: impl Into<String>) -> Self {
        Avatar {
            texto: texto.into(),
            lado: 32.0,
            radio: radio::CHIP_ICONO,
            fondo: color::DORADO,
            color: color::VERDE_PROFUNDO,
            tam_texto: tam::ETIQUETA,
            peso: Peso::Negrita,
            sombra: None,
        }
    }

    pub fn producto(texto: impl Into<String>) -> Self {
        Avatar { lado: 36.0, radio: radio::CONTROL, fondo: color::VERDE_CHIP, color: color::VERDE, tam_texto: tam::CUERPO, peso: Peso::Semi, ..Self::new(texto) }
    }

    pub fn vendedor(texto: impl Into<String>, fondo: Color32) -> Self {
        Avatar { lado: 148.0, radio: radio::AVATAR_GRANDE, fondo, color: color::BLANCO, tam_texto: 48.0, peso: Peso::Semi, sombra: Some(sombra::AVATAR), ..Self::new(texto) }
    }

    pub fn lado(mut self, lado: f32) -> Self {
        self.lado = lado;
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn colores(mut self, fondo: Color32, color: Color32) -> Self {
        self.fondo = fondo;
        self.color = color;
        self
    }

    pub fn texto(mut self, px: f32, peso: Peso) -> Self {
        self.tam_texto = px;
        self.peso = peso;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(self.lado), Sense::hover());
        self.pintar(ui, rect);
        respuesta
    }

    pub fn pintar(&self, ui: &Ui, rect: Rect) {
        if let Some(s) = self.sombra {
            ui.painter().add(s.forma(rect, self.radio));
        }
        ui.painter().rect_filled(rect, self.radio, self.fondo);
        let g = super::galeria_espaciada(ui, self.texto.clone(), fuente(self.tam_texto, self.peso), self.color, if self.lado > 100.0 { -0.02 * self.tam_texto } else { 0.0 });
        ui.painter().galley(rect.center() - g.size() / 2.0, g, self.color);
    }
}

#[cfg(test)]
mod pruebas {
    use super::iniciales;

    #[test]
    fn iniciales_de_nombre_y_apellido() {
        assert_eq!(iniciales("Lucía Fernández"), "LF");
        assert_eq!(iniciales("martín  álvarez"), "MÁ");
        assert_eq!(iniciales(""), "");
    }
}
