pub mod contenedores;
pub mod controles;
pub mod degradado;
pub mod iconos;
pub mod paleta;
pub mod toast;

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, RichText, Shadow, Stroke,
};

pub use contenedores::*;
pub use controles::*;
pub use degradado::*;
pub use iconos::*;

pub mod color {
    use eframe::egui::Color32;

    pub const FONDO: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
    pub const TEXTO: Color32 = Color32::from_rgb(0x0a, 0x0a, 0x0a);
    pub const PRIMARIO: Color32 = Color32::from_rgb(0x17, 0x17, 0x17);
    pub const PRIMARIO_TEXTO: Color32 = Color32::from_rgb(0xfa, 0xfa, 0xfa);
    pub const SECUNDARIO: Color32 = Color32::from_rgb(0xf5, 0xf5, 0xf5);
    pub const SECUNDARIO_TEXTO: Color32 = Color32::from_rgb(0x17, 0x17, 0x17);
    pub const APAGADO: Color32 = Color32::from_rgb(0xf5, 0xf5, 0xf5);
    pub const APAGADO_TEXTO: Color32 = Color32::from_rgb(0x73, 0x73, 0x73);
    pub const ACENTO: Color32 = Color32::from_rgb(0xf5, 0xf5, 0xf5);
    pub const ACENTO_TEXTO: Color32 = Color32::from_rgb(0x17, 0x17, 0x17);
    pub const DESTRUCTIVO: Color32 = Color32::from_rgb(0xef, 0x44, 0x44);
    pub const DESTRUCTIVO_TEXTO: Color32 = Color32::from_rgb(0xfa, 0xfa, 0xfa);
    pub const BORDE: Color32 = Color32::from_rgb(0xe5, 0xe5, 0xe5);
    pub const BORDE_ENTRADA: Color32 = Color32::from_rgb(0xa7, 0xa7, 0xa7);
    pub const ANILLO: Color32 = Color32::from_rgb(0x0a, 0x0a, 0x0a);
    pub const BLANCO_CREMA: Color32 = Color32::from_rgb(0xf4, 0xf4, 0xf4);
    pub const ESMERALDA_DESDE: Color32 = Color32::from_rgb(0x0a, 0x3a, 0x2b);
    pub const ESMERALDA_HASTA: Color32 = Color32::from_rgb(0x1f, 0x54, 0x44);
}

pub const RADIO: u8 = 8;
pub const RADIO_MD: u8 = 6;
pub const RADIO_SM: u8 = 4;
pub const RADIO_XL: u8 = 12;

pub fn s(unidades: f32) -> f32 {
    unidades * 4.0
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Peso {
    Normal,
    Medio,
    Semi,
    Negrita,
}

impl Peso {
    pub fn familia(self) -> FontFamily {
        match self {
            Peso::Normal => FontFamily::Proportional,
            Peso::Medio => FontFamily::Name("inter-500".into()),
            Peso::Semi => FontFamily::Name("inter-600".into()),
            Peso::Negrita => FontFamily::Name("inter-700".into()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tam {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    X2,
    X3,
    X4,
    X5,
    X6,
    Px(f32),
}

impl Tam {
    pub fn px(self) -> f32 {
        match self {
            Tam::Xs => 12.0,
            Tam::Sm => 14.0,
            Tam::Base => 16.0,
            Tam::Lg => 18.0,
            Tam::Xl => 20.0,
            Tam::X2 => 24.0,
            Tam::X3 => 30.0,
            Tam::X4 => 36.0,
            Tam::X5 => 48.0,
            Tam::X6 => 60.0,
            Tam::Px(px) => px,
        }
    }

    pub fn alto_linea(self) -> f32 {
        match self {
            Tam::Xs => 16.0,
            Tam::Sm => 20.0,
            Tam::Base => 24.0,
            Tam::Lg => 28.0,
            Tam::Xl => 28.0,
            Tam::X2 => 32.0,
            Tam::X3 => 36.0,
            Tam::X4 => 40.0,
            Tam::X5 | Tam::X6 => self.px(),
            Tam::Px(px) => px * 1.5,
        }
    }
}

pub fn fuente(tam: Tam, peso: Peso) -> FontId {
    FontId::new(tam.px(), peso.familia())
}

pub fn texto(contenido: impl Into<String>, tam: Tam, peso: Peso, color: Color32) -> RichText {
    RichText::new(contenido).font(fuente(tam, peso)).color(color)
}

pub fn marco() -> egui::Frame {
    egui::Frame::new()
}

pub fn borde(color: Color32) -> Stroke {
    Stroke::new(1.0, color)
}

pub fn margen(horizontal: f32, vertical: f32) -> Margin {
    Margin::symmetric(horizontal as i8, vertical as i8)
}

pub fn radio(px: u8) -> CornerRadius {
    CornerRadius::same(px)
}

pub fn sombra_sm() -> Shadow {
    Shadow { offset: [0, 1], blur: 2, spread: 0, color: Color32::from_black_alpha(13) }
}

pub fn sombra_md() -> Shadow {
    Shadow { offset: [0, 4], blur: 6, spread: 0, color: Color32::from_black_alpha(26) }
}

pub fn sombra_lg() -> Shadow {
    Shadow { offset: [0, 10], blur: 15, spread: 0, color: Color32::from_black_alpha(26) }
}

pub fn sombra_xl() -> Shadow {
    Shadow { offset: [0, 20], blur: 25, spread: 0, color: Color32::from_black_alpha(26) }
}

pub fn instalar(ctx: &egui::Context) {
    let mut fuentes = FontDefinitions::default();
    let pesos: [(&str, &'static [u8]); 4] = [
        ("inter-400", include_bytes!("../../../../assets/fuentes/Inter-Regular.ttf")),
        ("inter-500", include_bytes!("../../../../assets/fuentes/Inter-Medium.ttf")),
        ("inter-600", include_bytes!("../../../../assets/fuentes/Inter-SemiBold.ttf")),
        ("inter-700", include_bytes!("../../../../assets/fuentes/Inter-Bold.ttf")),
    ];
    for (nombre, bytes) in pesos {
        fuentes.font_data.insert(nombre.into(), Arc::new(FontData::from_static(bytes)));
        let mut cadena = vec![nombre.to_string()];
        cadena.extend(fuentes.families[&FontFamily::Proportional].iter().cloned());
        if nombre == "inter-400" {
            fuentes.families.insert(FontFamily::Proportional, cadena);
        } else {
            fuentes.families.insert(FontFamily::Name(nombre.into()), cadena);
        }
    }
    ctx.set_fonts(fuentes);
    egui_extras::install_image_loaders(ctx);
    iconos::registrar(ctx);

    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(|estilo| {
        estilo.text_styles = [
            (egui::TextStyle::Small, fuente(Tam::Xs, Peso::Normal)),
            (egui::TextStyle::Body, fuente(Tam::Sm, Peso::Normal)),
            (egui::TextStyle::Button, fuente(Tam::Sm, Peso::Medio)),
            (egui::TextStyle::Heading, fuente(Tam::Lg, Peso::Semi)),
            (egui::TextStyle::Monospace, FontId::monospace(13.0)),
        ]
        .into();
        estilo.spacing.item_spacing = egui::vec2(0.0, 0.0);
        estilo.spacing.button_padding = egui::vec2(0.0, 0.0);
        estilo.spacing.interact_size.y = 0.0;
        estilo.spacing.scroll.bar_width = 8.0;
        estilo.spacing.scroll.floating = true;
        estilo.interaction.selectable_labels = false;
        let visuales = &mut estilo.visuals;
        *visuales = egui::Visuals::light();
        visuales.panel_fill = color::FONDO;
        visuales.window_fill = color::FONDO;
        visuales.extreme_bg_color = color::FONDO;
        visuales.override_text_color = Some(color::TEXTO);
        visuales.selection.bg_fill = paleta::BLUE_200;
        visuales.selection.stroke = Stroke::new(1.0, color::TEXTO);
        visuales.text_cursor.stroke = Stroke::new(1.5, color::TEXTO);
        visuales.window_shadow = sombra_lg();
        visuales.popup_shadow = sombra_md();
        visuales.window_stroke = borde(color::BORDE);
        visuales.window_corner_radius = radio(RADIO);
        visuales.menu_corner_radius = radio(RADIO_MD);
        for widget in [
            &mut visuales.widgets.noninteractive,
            &mut visuales.widgets.inactive,
            &mut visuales.widgets.hovered,
            &mut visuales.widgets.active,
            &mut visuales.widgets.open,
        ] {
            widget.corner_radius = radio(RADIO_MD);
            widget.fg_stroke.color = color::TEXTO;
        }
        visuales.widgets.noninteractive.bg_stroke = borde(color::BORDE);
    });
}
