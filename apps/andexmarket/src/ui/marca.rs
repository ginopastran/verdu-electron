use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Painter, Rect, RichText, Shape, Stroke, StrokeKind, Vec2};

use super::Peso;

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied_const(r, g, b, a)
}

pub mod color {
    use eframe::egui::Color32;

    use super::rgba;

    pub const VERDE_PROFUNDO: Color32 = Color32::from_rgb(0x0A, 0x3A, 0x2B);
    pub const VERDE: Color32 = Color32::from_rgb(0x1F, 0x54, 0x44);
    pub const VERDE_HOVER_OSCURO: Color32 = Color32::from_rgb(0x13, 0x46, 0x34);
    pub const VERDE_EXITO: Color32 = Color32::from_rgb(0x14, 0x65, 0x3F);
    pub const VERDE_PUNTO: Color32 = Color32::from_rgb(0x22, 0xA0, 0x6B);
    pub const VERDE_CHIP: Color32 = Color32::from_rgb(0xEA, 0xF2, 0xEE);
    pub const VERDE_ELEGIDO: Color32 = Color32::from_rgb(0xF1, 0xF7, 0xF4);
    pub const VERDE_EXITO_FONDO: Color32 = Color32::from_rgb(0xE3, 0xF1, 0xEA);
    pub const VERDE_TOAST: Color32 = Color32::from_rgb(0x6F, 0xC7, 0x9F);

    pub const DORADO: Color32 = Color32::from_rgb(0xD3, 0x98, 0x01);
    pub const DORADO_OSCURO: Color32 = Color32::from_rgb(0xB0, 0x7D, 0x00);
    pub const DORADO_TEXTO: Color32 = Color32::from_rgb(0x7A, 0x58, 0x00);
    pub const DORADO_FONDO: Color32 = Color32::from_rgb(0xFD, 0xF4, 0xDC);
    pub const DORADO_HOVER: Color32 = Color32::from_rgb(0xFD, 0xF8, 0xEC);
    pub const DORADO_BORDE: Color32 = Color32::from_rgb(0xE8, 0xC7, 0x70);
    pub const DORADO_AVATAR: Color32 = Color32::from_rgb(0x9A, 0x6F, 0x00);

    pub const FONDO_APP: Color32 = Color32::from_rgb(0xF6, 0xF8, 0xF6);
    pub const BLANCO: Color32 = Color32::WHITE;
    pub const FONDO_ENCABEZADO_TABLA: Color32 = Color32::from_rgb(0xFB, 0xFC, 0xFB);
    pub const FILA_HOVER: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xF9);
    pub const SHIMMER_BASE: Color32 = Color32::from_rgb(0xED, 0xF2, 0xEF);
    pub const SHIMMER_BRILLO: Color32 = Color32::from_rgb(0xF7, 0xFA, 0xF8);

    pub const BORDE: Color32 = Color32::from_rgb(0xDC, 0xE4, 0xDF);
    pub const BORDE_SUAVE: Color32 = Color32::from_rgb(0xED, 0xF2, 0xEF);
    pub const BORDE_HOVER: Color32 = Color32::from_rgb(0xB6, 0xC4, 0xBD);
    pub const INTERRUPTOR_APAGADO: Color32 = Color32::from_rgb(0xC9, 0xD3, 0xCE);

    pub const TEXTO: Color32 = Color32::from_rgb(0x10, 0x18, 0x28);
    pub const TEXTO_SECUNDARIO: Color32 = Color32::from_rgb(0x34, 0x40, 0x54);
    pub const TEXTO_APAGADO: Color32 = Color32::from_rgb(0x66, 0x70, 0x85);
    pub const TEXTO_TENUE: Color32 = Color32::from_rgb(0x7C, 0x8B, 0x84);
    pub const TEXTO_DESHABILITADO: Color32 = Color32::from_rgb(0x98, 0xA2, 0xB3);
    pub const TEXTO_SUGERENCIA: Color32 = Color32::from_rgb(0x75, 0x75, 0x75);

    pub const SOBRE_OSCURO: Color32 = Color32::from_rgb(0xE3, 0xEC, 0xE7);
    pub const SOBRE_OSCURO_ICONO: Color32 = Color32::from_rgb(0xB6, 0xC4, 0xBD);
    pub const SOBRE_OSCURO_APAGADO: Color32 = Color32::from_rgb(0x9D, 0xB8, 0xAC);
    pub const SOBRE_OSCURO_TENUE: Color32 = Color32::from_rgb(0x7F, 0xA8, 0x97);
    pub const SOBRE_OSCURO_PARRAFO: Color32 = Color32::from_rgb(0xB6, 0xCE, 0xC3);
    pub const SOBRE_OSCURO_LISTA: Color32 = Color32::from_rgb(0xD0, 0xDD, 0xD6);

    pub const ROJO: Color32 = Color32::from_rgb(0xB4, 0x23, 0x18);
    pub const ROJO_INTENSO: Color32 = Color32::from_rgb(0xD9, 0x2D, 0x20);
    pub const ROJO_PESTANA: Color32 = Color32::from_rgb(0xDC, 0x26, 0x26);
    pub const ROJO_ERROR_BORDE: Color32 = Color32::from_rgb(0xF0, 0x44, 0x38);
    pub const ROJO_FONDO: Color32 = Color32::from_rgb(0xFE, 0xF3, 0xF2);
    pub const ROJO_BORDE: Color32 = Color32::from_rgb(0xFE, 0xCD, 0xCA);
    pub const ROJO_TOAST: Color32 = Color32::from_rgb(0xF9, 0x70, 0x66);

    pub const AZUL: Color32 = Color32::from_rgb(0x1D, 0x4E, 0x89);
    pub const AZUL_FONDO: Color32 = Color32::from_rgb(0xE6, 0xEE, 0xF9);
    pub const VIOLETA: Color32 = Color32::from_rgb(0x5B, 0x3E, 0x96);
    pub const VIOLETA_FONDO: Color32 = Color32::from_rgb(0xEF, 0xEA, 0xF8);
    pub const GRIS_AVATAR: Color32 = Color32::from_rgb(0x3B, 0x4A, 0x43);

    pub const OVERLAY_MODAL: Color32 = rgba(10, 24, 18, 115);
    pub const ANILLO_FOCO: Color32 = rgba(31, 84, 68, 38);
    pub const ANILLO_BUSQUEDA: Color32 = rgba(211, 152, 1, 89);
    pub const BLANCO_10: Color32 = rgba(255, 255, 255, 26);
    pub const BLANCO_12: Color32 = rgba(255, 255, 255, 31);
    pub const BLANCO_18: Color32 = rgba(255, 255, 255, 46);
    pub const BLANCO_20: Color32 = rgba(255, 255, 255, 51);
    pub const BLANCO_25: Color32 = rgba(255, 255, 255, 64);
    pub const ROJO_12: Color32 = rgba(220, 38, 38, 31);

    pub const AVATARES: [Color32; 5] = [VERDE, VERDE_PROFUNDO, DORADO_AVATAR, VERDE_HOVER_OSCURO, GRIS_AVATAR];
}

pub mod tam {
    pub const DIMINUTO: f32 = 11.0;
    pub const MINI: f32 = 11.5;
    pub const XS: f32 = 12.0;
    pub const CHICO: f32 = 12.5;
    pub const ETIQUETA: f32 = 13.0;
    pub const CONTROL: f32 = 13.5;
    pub const CUERPO: f32 = 14.0;
    pub const CUERPO_MAS: f32 = 14.5;
    pub const ENTRADA: f32 = 15.0;
    pub const BUSQUEDA: f32 = 15.5;
    pub const DESTACADO: f32 = 16.0;
    pub const FILA: f32 = 17.0;
    pub const TITULO_MODAL: f32 = 19.0;
    pub const GRANDE: f32 = 20.0;
    pub const TITULO_SECCION: f32 = 22.0;
    pub const MONTO_CHICO: f32 = 24.0;
    pub const KPI: f32 = 26.0;
    pub const MONTO_MEDIO: f32 = 32.0;
    pub const MONTO: f32 = 34.0;
    pub const TITULO_PANTALLA: f32 = 40.0;
    pub const TOTAL: f32 = 46.0;
}

pub mod radio {
    pub const TECLA: u8 = 5;
    pub const CHIP: u8 = 6;
    pub const TECLA_CUADRADA: u8 = 7;
    pub const ICONO: u8 = 8;
    pub const CHIP_ICONO: u8 = 9;
    pub const CONTROL: u8 = 10;
    pub const BOTON_MODAL: u8 = 11;
    pub const CAMPO: u8 = 12;
    pub const POPOVER: u8 = 14;
    pub const TARJETA: u8 = 16;
    pub const PANEL: u8 = 18;
    pub const MODAL: u8 = 20;
    pub const AVATAR_GRANDE: u8 = 24;
    pub const PASTILLA: u8 = 99;
}

pub mod espacio {
    pub const E2: f32 = 2.0;
    pub const E4: f32 = 4.0;
    pub const E6: f32 = 6.0;
    pub const E8: f32 = 8.0;
    pub const E10: f32 = 10.0;
    pub const E12: f32 = 12.0;
    pub const E14: f32 = 14.0;
    pub const E16: f32 = 16.0;
    pub const E18: f32 = 18.0;
    pub const E20: f32 = 20.0;
    pub const E22: f32 = 22.0;
    pub const E24: f32 = 24.0;
    pub const E28: f32 = 28.0;
    pub const E32: f32 = 32.0;
    pub const E40: f32 = 40.0;
}

pub mod alto {
    pub const CAMPO: f32 = 46.0;
    pub const CAMPO_GRANDE: f32 = 48.0;
    pub const BOTON_GRANDE: f32 = 48.0;
    pub const BOTON: f32 = 44.0;
    pub const BOTON_MODAL: f32 = 42.0;
    pub const CONTROL: f32 = 40.0;
    pub const BOTON_CHICO: f32 = 32.0;
    pub const ENCABEZADO: f32 = 68.0;
    pub const ENCABEZADO_TABLA: f32 = 42.0;
    pub const FILA_TABLA: f32 = 62.0;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sombra {
    pub x: f32,
    pub y: f32,
    pub desenfoque: f32,
    pub extension: f32,
    pub color: Color32,
}

impl Sombra {
    pub const fn new(y: f32, desenfoque: f32, extension: f32, color: Color32) -> Self {
        Sombra { x: 0.0, y, desenfoque, extension, color }
    }

    pub fn forma(&self, rect: Rect, radio: impl Into<CornerRadius>) -> Shape {
        let caja = rect.translate(Vec2::new(self.x, self.y)).expand(self.extension);
        let radio: CornerRadius = radio.into();
        let ajuste = self.extension.round().clamp(-255.0, 255.0) as i32;
        let radio = CornerRadius {
            nw: (radio.nw as i32 + ajuste).clamp(0, 255) as u8,
            ne: (radio.ne as i32 + ajuste).clamp(0, 255) as u8,
            sw: (radio.sw as i32 + ajuste).clamp(0, 255) as u8,
            se: (radio.se as i32 + ajuste).clamp(0, 255) as u8,
        };
        Shape::Rect(egui::epaint::RectShape::filled(caja, radio, self.color).with_blur_width(self.desenfoque))
    }

    pub fn con_opacidad(mut self, opacidad: f32) -> Self {
        self.color = self.color.gamma_multiply(opacidad);
        self
    }
}

pub mod sombra {
    use super::{rgba, Sombra};

    pub const BOTON: Sombra = Sombra::new(8.0, 18.0, -10.0, rgba(10, 58, 43, 179));
    pub const BOTON_COBRAR: Sombra = Sombra::new(10.0, 20.0, -12.0, rgba(10, 58, 43, 204));
    pub const POPOVER: Sombra = Sombra::new(24.0, 48.0, -16.0, rgba(10, 58, 43, 89));
    pub const DESPLEGABLE: Sombra = Sombra::new(20.0, 40.0, -12.0, rgba(10, 58, 43, 77));
    pub const AUTOCOMPLETAR: Sombra = Sombra::new(18.0, 36.0, -12.0, rgba(10, 58, 43, 77));
    pub const MODAL: Sombra = Sombra::new(30.0, 60.0, -20.0, rgba(10, 58, 43, 128));
    pub const TOAST: Sombra = Sombra::new(16.0, 32.0, -12.0, rgba(0, 0, 0, 115));
    pub const AVATAR: Sombra = Sombra::new(14.0, 28.0, -16.0, rgba(10, 58, 43, 153));
    pub const SEGMENTO: Sombra = Sombra::new(1.0, 2.0, 0.0, rgba(16, 24, 40, 26));
    pub const ICONO_EXITO: Sombra = Sombra::new(12.0, 24.0, -8.0, rgba(10, 58, 43, 153));
}

pub fn pintar_sombra(pintor: &Painter, rect: Rect, radio: impl Into<CornerRadius>, sombra: Sombra) {
    pintor.add(sombra.forma(rect, radio));
}

pub fn pintar_anillo(pintor: &Painter, rect: Rect, radio: impl Into<CornerRadius>, color: Color32) {
    pintor.rect_stroke(rect, radio, Stroke::new(3.0, color), StrokeKind::Outside);
}

pub fn familia_mono(peso: Peso) -> FontFamily {
    match peso {
        Peso::Normal => FontFamily::Monospace,
        Peso::Medio => FontFamily::Name("geist-mono-500".into()),
        Peso::Semi | Peso::Negrita => FontFamily::Name("geist-mono-600".into()),
    }
}

pub fn fuente(px: f32, peso: Peso) -> FontId {
    FontId::new(px, peso.familia())
}

pub fn fuente_mono(px: f32, peso: Peso) -> FontId {
    FontId::new(px, familia_mono(peso))
}

pub fn texto(contenido: impl Into<String>, px: f32, peso: Peso, color: Color32) -> RichText {
    RichText::new(contenido).font(fuente(px, peso)).color(color)
}

pub fn texto_mono(contenido: impl Into<String>, px: f32, peso: Peso, color: Color32) -> RichText {
    RichText::new(contenido).font(fuente_mono(px, peso)).color(color)
}

pub fn em(px: f32, proporcion: f32) -> f32 {
    px * proporcion
}

pub fn mezclar(desde: Color32, hasta: Color32, t: f32) -> Color32 {
    let canal = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        canal(desde.r(), hasta.r()),
        canal(desde.g(), hasta.g()),
        canal(desde.b(), hasta.b()),
        canal(desde.a(), hasta.a()),
    )
}
