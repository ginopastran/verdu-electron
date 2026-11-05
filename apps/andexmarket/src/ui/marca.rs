use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Painter, Rect, RichText, Shape, Stroke, StrokeKind, Vec2};

use super::Peso;

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied_const(r, g, b, a)
}

pub mod color {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use eframe::egui::Color32;

    use super::rgba;

    pub const VERDE_EXITO: Color32 = Color32::from_rgb(0x14, 0x65, 0x3F);
    pub const VERDE_PUNTO: Color32 = Color32::from_rgb(0x22, 0xA0, 0x6B);
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
    pub const ANILLO_BUSQUEDA: Color32 = rgba(211, 152, 1, 89);
    pub const BLANCO_10: Color32 = rgba(255, 255, 255, 26);
    pub const BLANCO_12: Color32 = rgba(255, 255, 255, 31);
    pub const BLANCO_18: Color32 = rgba(255, 255, 255, 46);
    pub const BLANCO_20: Color32 = rgba(255, 255, 255, 51);
    pub const BLANCO_25: Color32 = rgba(255, 255, 255, 64);
    pub const ROJO_12: Color32 = rgba(220, 38, 38, 31);

    struct Paleta {
        id: &'static str,
        profundo: Color32,
        activo: Color32,
        primario: Color32,
        chip: Color32,
        elegido: Color32,
        icono_oscuro: Color32,
        texto_oscuro: Color32,
    }

    const fn rgb(c: [u8; 3]) -> Color32 {
        Color32::from_rgb(c[0], c[1], c[2])
    }

    const fn paleta(id: &'static str, c: [[u8; 3]; 7]) -> Paleta {
        Paleta { id, profundo: rgb(c[0]), activo: rgb(c[1]), primario: rgb(c[2]), chip: rgb(c[3]), elegido: rgb(c[4]), icono_oscuro: rgb(c[5]), texto_oscuro: rgb(c[6]) }
    }

    const PALETAS: [Paleta; 6] = [
        paleta("andex", [[10, 58, 43], [19, 70, 52], [31, 84, 68], [234, 242, 238], [241, 247, 244], [127, 168, 151], [157, 184, 172]]),
        paleta("oceano", [[11, 42, 74], [19, 56, 95], [30, 79, 134], [232, 240, 250], [241, 246, 252], [127, 156, 192], [160, 183, 212]]),
        paleta("vino", [[59, 13, 28], [79, 21, 39], [122, 35, 64], [247, 234, 238], [251, 242, 245], [184, 134, 149], [209, 169, 181]]),
        paleta("grafito", [[28, 33, 40], [42, 49, 58], [61, 71, 84], [237, 239, 242], [244, 245, 247], [143, 153, 166], [174, 182, 193]]),
        paleta("terracota", [[58, 26, 14], [79, 36, 20], [138, 59, 30], [248, 237, 231], [252, 244, 240], [192, 140, 116], [214, 174, 155]]),
        paleta("violeta", [[37, 23, 74], [51, 33, 95], [75, 51, 145], [239, 234, 249], [246, 243, 252], [156, 140, 200], [184, 171, 218]]),
    ];

    static TEMA: AtomicUsize = AtomicUsize::new(0);

    fn actual() -> &'static Paleta {
        &PALETAS[TEMA.load(Ordering::Relaxed).min(PALETAS.len() - 1)]
    }

    fn indice(id: Option<&str>) -> usize {
        id.and_then(|id| PALETAS.iter().position(|p| p.id == id)).unwrap_or(0)
    }

    pub fn aplicar_tema(id: Option<&str>) -> bool {
        let indice = indice(id);
        TEMA.swap(indice, Ordering::Relaxed) != indice
    }

    pub fn tema() -> &'static str {
        actual().id
    }

    pub fn verde_profundo() -> Color32 {
        actual().profundo
    }

    pub fn verde_hover_oscuro() -> Color32 {
        actual().activo
    }

    pub fn verde() -> Color32 {
        actual().primario
    }

    pub fn verde_chip() -> Color32 {
        actual().chip
    }

    pub fn verde_elegido() -> Color32 {
        actual().elegido
    }

    pub fn sobre_oscuro_tenue() -> Color32 {
        actual().icono_oscuro
    }

    pub fn sobre_oscuro_apagado() -> Color32 {
        actual().texto_oscuro
    }

    pub fn anillo_foco() -> Color32 {
        let c = verde();
        rgba(c.r(), c.g(), c.b(), 38)
    }

    pub fn avatares() -> [Color32; 5] {
        [verde(), verde_profundo(), DORADO_AVATAR, verde_hover_oscuro(), GRIS_AVATAR]
    }

    #[cfg(test)]
    mod pruebas {
        use super::*;

        #[test]
        fn sin_tema_o_desconocido_usa_andex_con_los_colores_de_siempre() {
            for id in [None, Some("andex"), Some("inexistente")] {
                let p = &PALETAS[indice(id)];
                assert_eq!(p.id, "andex");
                assert_eq!(p.profundo, Color32::from_rgb(0x0A, 0x3A, 0x2B));
                assert_eq!(p.primario, Color32::from_rgb(0x1F, 0x54, 0x44));
                assert_eq!(p.chip, Color32::from_rgb(0xEA, 0xF2, 0xEE));
            }
        }

        #[test]
        fn cada_tema_de_la_web_tiene_su_paleta() {
            for id in ["andex", "oceano", "vino", "grafito", "terracota", "violeta"] {
                assert_eq!(PALETAS[indice(Some(id))].id, id);
            }
            assert_eq!(PALETAS[indice(Some("oceano"))].primario, Color32::from_rgb(30, 79, 134));
        }
    }
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
