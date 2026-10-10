pub mod iconos;
pub mod marca;
pub mod v2;

use std::sync::Arc;

use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke};

pub use iconos::*;

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
    let mono: [(&str, &'static [u8]); 3] = [
        ("geist-mono-400", include_bytes!("../../../../assets/fuentes/GeistMono-Regular.ttf")),
        ("geist-mono-500", include_bytes!("../../../../assets/fuentes/GeistMono-Medium.ttf")),
        ("geist-mono-600", include_bytes!("../../../../assets/fuentes/GeistMono-SemiBold.ttf")),
    ];
    for (nombre, bytes) in mono {
        fuentes.font_data.insert(nombre.into(), Arc::new(FontData::from_static(bytes)));
        let mut cadena = vec![nombre.to_string()];
        cadena.extend(fuentes.families[&FontFamily::Monospace].iter().cloned());
        if nombre == "geist-mono-400" {
            fuentes.families.insert(FontFamily::Monospace, cadena);
        } else {
            fuentes.families.insert(FontFamily::Name(nombre.into()), cadena);
        }
    }
    ctx.set_fonts(fuentes);
    egui_extras::install_image_loaders(ctx);
    iconos::registrar(ctx);

    ctx.set_theme(egui::Theme::Light);
    let texto = Color32::from_rgb(0x0a, 0x0a, 0x0a);
    let borde = Color32::from_rgb(0xe5, 0xe5, 0xe5);
    ctx.all_styles_mut(|estilo| {
        estilo.text_styles = [
            (egui::TextStyle::Small, FontId::new(12.0, Peso::Normal.familia())),
            (egui::TextStyle::Body, FontId::new(14.0, Peso::Normal.familia())),
            (egui::TextStyle::Button, FontId::new(14.0, Peso::Medio.familia())),
            (egui::TextStyle::Heading, FontId::new(18.0, Peso::Semi.familia())),
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
        visuales.panel_fill = marca::color::BLANCO;
        visuales.window_fill = marca::color::BLANCO;
        visuales.extreme_bg_color = marca::color::BLANCO;
        visuales.override_text_color = Some(texto);
        visuales.selection.bg_fill = Color32::from_rgb(0xbf, 0xdb, 0xfe);
        visuales.selection.stroke = Stroke::new(1.0, texto);
        visuales.text_cursor.stroke = Stroke::new(1.5, texto);
        visuales.window_shadow = Shadow { offset: [0, 10], blur: 15, spread: 0, color: Color32::from_black_alpha(26) };
        visuales.popup_shadow = Shadow { offset: [0, 4], blur: 6, spread: 0, color: Color32::from_black_alpha(26) };
        visuales.window_stroke = Stroke::new(1.0, borde);
        visuales.window_corner_radius = CornerRadius::same(8);
        visuales.menu_corner_radius = CornerRadius::same(6);
        for widget in [
            &mut visuales.widgets.noninteractive,
            &mut visuales.widgets.inactive,
            &mut visuales.widgets.hovered,
            &mut visuales.widgets.active,
            &mut visuales.widgets.open,
        ] {
            widget.corner_radius = CornerRadius::same(6);
            widget.fg_stroke.color = texto;
        }
        visuales.widgets.noninteractive.bg_stroke = Stroke::new(1.0, borde);
    });
}
