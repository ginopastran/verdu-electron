use std::f32::consts::PI;
use std::sync::Arc;

use chrono::{Datelike, NaiveDateTime, Timelike};
use eframe::egui::{
    text::{LayoutJob, TextWrapping},
    Align, Color32, FontId, Galley, Painter, Pos2, Rect, Shape, Stroke, TextFormat, Vec2,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Pista {
    Fija(f32),
    Fraccion(f32, f32),
}

pub fn anchos_grilla(pistas: &[Pista], disponible: f32, separacion: f32) -> Vec<f32> {
    let fijos: f32 = pistas.iter().map(|p| if let Pista::Fija(a) = p { *a } else { 0.0 }).sum();
    let mut libre = disponible - fijos - separacion * pistas.len().saturating_sub(1) as f32;
    let mut anchos: Vec<f32> = pistas.iter().map(|p| if let Pista::Fija(a) = p { *a } else { 0.0 }).collect();
    let mut congelada: Vec<bool> = pistas.iter().map(|p| matches!(p, Pista::Fija(_))).collect();
    loop {
        let fracciones: f32 = pistas.iter().zip(&congelada).filter(|(_, c)| !**c).map(|(p, _)| if let Pista::Fraccion(f, _) = p { *f } else { 0.0 }).sum();
        if fracciones <= 0.0 {
            return anchos;
        }
        let unidad = libre.max(0.0) / fracciones;
        let mut cambio = false;
        for (i, pista) in pistas.iter().enumerate() {
            if let (Pista::Fraccion(f, minimo), false) = (pista, congelada[i]) {
                if unidad * f < *minimo {
                    anchos[i] = *minimo;
                    congelada[i] = true;
                    libre -= minimo;
                    cambio = true;
                }
            }
        }
        if !cambio {
            for (i, pista) in pistas.iter().enumerate() {
                if let (Pista::Fraccion(f, _), false) = (pista, congelada[i]) {
                    anchos[i] = unidad * f;
                }
            }
            return anchos;
        }
    }
}

pub fn celdas(fila: Rect, pistas: &[Pista], relleno: f32, separacion: f32) -> Vec<Rect> {
    let mut x = fila.left() + relleno;
    anchos_grilla(pistas, fila.width() - relleno * 2.0, separacion)
        .into_iter()
        .map(|ancho| {
            let celda = Rect::from_min_size(Pos2::new(x, fila.top()), Vec2::new(ancho, fila.height()));
            x += ancho + separacion;
            celda
        })
        .collect()
}

pub fn borde_punteado(pintor: &Painter, rect: Rect, radio: f32, color: Color32) {
    let rect = rect.shrink(0.5);
    let r = radio.min(rect.width() / 2.0).min(rect.height() / 2.0);
    let esquinas = [
        (Pos2::new(rect.right() - r, rect.top() + r), -PI / 2.0),
        (Pos2::new(rect.right() - r, rect.bottom() - r), 0.0),
        (Pos2::new(rect.left() + r, rect.bottom() - r), PI / 2.0),
        (Pos2::new(rect.left() + r, rect.top() + r), PI),
    ];
    let mut puntos = vec![Pos2::new(rect.left() + r, rect.top())];
    for (centro, desde) in esquinas {
        for paso in 0..=8 {
            let angulo = desde + PI / 2.0 * paso as f32 / 8.0;
            puntos.push(centro + Vec2::new(angulo.cos(), angulo.sin()) * r);
        }
    }
    puntos.push(puntos[0]);
    pintor.extend(Shape::dashed_line(&puntos, Stroke::new(1.0, color), 3.0, 3.0));
}

pub fn galeria_cortada(pintor: &Painter, contenido: &str, fuente: FontId, color: Color32, ancho: f32) -> Arc<Galley> {
    let mut trabajo = LayoutJob::simple_singleline(contenido.to_string(), fuente, color);
    trabajo.wrap = TextWrapping::truncate_at_width(ancho.max(0.0));
    pintor.layout_job(trabajo)
}

pub fn galeria_centrada(pintor: &Painter, contenido: &str, fuente: FontId, color: Color32, interlineado: f32, ancho: f32) -> Arc<Galley> {
    let alto_linea = fuente.size * interlineado;
    let mut trabajo = LayoutJob::single_section(contenido.to_string(), TextFormat { font_id: fuente, color, line_height: Some(alto_linea), ..Default::default() });
    trabajo.wrap.max_width = ancho;
    trabajo.halign = Align::Center;
    pintor.layout_job(trabajo)
}

pub fn reloj(fecha: NaiveDateTime) -> String {
    const DIAS: [&str; 7] = ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"];
    const MESES: [&str; 12] = ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"];
    let hora = match fecha.hour() % 12 {
        0 => 12,
        h => h,
    };
    let meridiano = if fecha.hour() < 12 { "a. m." } else { "p. m." };
    format!(
        "{}, {} {} · {:02}:{:02} {}",
        DIAS[fecha.weekday().num_days_from_monday() as usize],
        fecha.day(),
        MESES[fecha.month0() as usize],
        hora,
        fecha.minute(),
        meridiano
    )
}

#[cfg(test)]
mod pruebas {
    use chrono::NaiveDate;

    use super::{anchos_grilla, reloj, Pista};

    #[test]
    fn reparte_como_grid_de_css() {
        let pistas = [Pista::Fraccion(2.2, 0.0), Pista::Fraccion(1.0, 110.0), Pista::Fraccion(1.0, 110.0), Pista::Fraccion(1.0, 120.0), Pista::Fija(44.0)];
        let amplio = anchos_grilla(&pistas, 1418.0, 14.0);
        let unidad = (1418.0 - 44.0 - 56.0) / 5.2;
        assert!((amplio[0] - unidad * 2.2).abs() < 0.01);
        assert!((amplio[3] - unidad).abs() < 0.01);
        assert_eq!(amplio[4], 44.0);
        let angosto = anchos_grilla(&pistas, 542.0, 14.0);
        assert_eq!(&angosto[1..], &[110.0, 110.0, 120.0, 44.0]);
        assert!((angosto[0] - (542.0 - 44.0 - 56.0 - 340.0)).abs() < 0.01);
        let sin_lugar = anchos_grilla(&pistas, 300.0, 14.0);
        assert_eq!(sin_lugar[0], 0.0);
    }

    #[test]
    fn reloj_como_el_navegador() {
        let tarde = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap().and_hms_opt(17, 36, 0).unwrap();
        assert_eq!(reloj(tarde), "sáb, 10 oct · 05:36 p. m.");
        let medianoche = NaiveDate::from_ymd_opt(2026, 9, 7).unwrap().and_hms_opt(0, 5, 0).unwrap();
        assert_eq!(reloj(medianoche), "lun, 7 sept · 12:05 a. m.");
    }
}
