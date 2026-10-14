use chrono::{NaiveDateTime, Timelike};
use eframe::egui::{self, text::LayoutJob, Color32, Frame, Margin, Rect, Sense, TextFormat, Ui, Vec2};
use nucleo::cierre::{Arqueo, EstadoArqueo};
use nucleo::formato;

use crate::ui::marca::{color, fuente, radio};
use crate::ui::v2::{self, Tono};
use crate::ui::{pintar_icono_solar_en, Peso};

pub fn dinero(valor: f64) -> String {
    let valor = if valor.is_finite() { valor } else { 0.0 };
    let signo = if valor < 0.0 { "−" } else { "" };
    format!("{signo}${}", formato::numero_con(valor.abs(), 0, 2))
}

pub fn diferencia(arqueo: &Arqueo) -> (String, Color32) {
    match arqueo.estado {
        EstadoArqueo::Cuadra => ("Cuadra".to_string(), color::VERDE),
        EstadoArqueo::Faltan => (format!("Faltan {}", dinero(-arqueo.diferencia)), color::ROJO),
        EstadoArqueo::Sobran => (format!("Sobran {}", dinero(arqueo.diferencia)), color::DORADO_TEXTO),
    }
}

pub fn metodo_pago(clave: &str) -> (String, &'static str, Tono) {
    match clave {
        "efectivo" => ("Efectivo".into(), "cash", Tono::Exito),
        "tarjeta" => ("Tarjeta".into(), "card", Tono::Azul),
        "qr" => ("Transferencia".into(), "qr", Tono::Violeta),
        "transferencia" => ("Transferencia".into(), "transfer", Tono::Violeta),
        "split" => ("Mixto".into(), "split", Tono::Aviso),
        otro => (nucleo::pedidos::capitalizar(otro), "wallet", Tono::Marca),
    }
}

pub fn hora_corta(fecha: NaiveDateTime) -> String {
    let (tarde, hora) = fecha.hour12();
    format!("{hora:02}:{:02} {}", fecha.minute(), if tarde { "p. m." } else { "a. m." })
}

pub fn etiqueta(ui: &mut Ui, contenido: &str, px: f32, peso: Peso) {
    v2::texto(ui, contenido, px, peso, color::TEXTO_SECUNDARIO);
}

pub fn texto_envuelto(ui: &mut Ui, contenido: &str, px: f32, peso: Peso, tinta: Color32, interlineado: f32, ancho: f32) {
    let mut trabajo = LayoutJob::single_section(
        contenido.to_string(),
        TextFormat { font_id: fuente(px, peso), color: tinta, line_height: Some((px * interlineado).round()), ..Default::default() },
    );
    trabajo.wrap.max_width = ancho;
    let galeria = ui.painter().layout_job(trabajo);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, galeria.size().y), Sense::hover());
    ui.painter().galley(rect.min, galeria, tinta);
}

pub fn nota(ui: &mut Ui, icono: &str, tam_icono: f32, contenido: &str, px: f32, fondo: Color32, tinta: Color32, color_icono: Color32) {
    Frame::NONE.fill(fondo).corner_radius(radio::CAMPO).inner_margin(Margin::symmetric(14, 12)).show(ui, |ui| {
        let ancho = ui.available_width();
        ui.set_width(ancho);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let (caja, _) = ui.allocate_exact_size(Vec2::new(tam_icono, tam_icono + 1.0), Sense::hover());
            pintar_icono_solar_en(ui, Rect::from_min_size(caja.min + Vec2::new(0.0, 1.0), Vec2::splat(tam_icono)), icono, color_icono);
            texto_envuelto(ui, contenido, px, Peso::Normal, tinta, 1.5, ancho - tam_icono - 10.0);
        });
    });
}

pub fn caja_entrada(ui: &mut Ui, ancho: f32, alto: f32, abierto: bool) -> (Rect, egui::Response) {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::click());
    let borde = if abierto {
        color::VERDE
    } else if respuesta.hovered() {
        color::BORDE_HOVER
    } else {
        color::BORDE
    };
    ui.painter().rect(rect, radio::CONTROL, color::BLANCO, egui::Stroke::new(1.0, borde), egui::StrokeKind::Inside);
    if abierto {
        crate::ui::marca::pintar_anillo(ui.painter(), rect, radio::CONTROL, color::ANILLO_FOCO);
    }
    (rect, respuesta.on_hover_cursor(egui::CursorIcon::PointingHand))
}

#[cfg(test)]
mod pruebas {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn montos_y_horas_como_el_diseno() {
        assert_eq!(dinero(59560.0), "$59.560");
        assert_eq!(dinero(8508.571), "$8.508,57");
        assert_eq!(dinero(-239.0), "−$239");
        assert_eq!(dinero(f64::NAN), "$0");
        let texto = |esperado: f64, contado: f64| diferencia(&nucleo::cierre::arqueo(esperado, contado)).0;
        assert_eq!(texto(19800.0, 19500.5), "Faltan $299,5");
        assert_eq!(texto(19800.0, 20000.0), "Sobran $200");
        assert_eq!(texto(19800.0, 19800.4), "Cuadra");

        let fecha = |h, m| NaiveDate::from_ymd_opt(2026, 10, 10).unwrap().and_hms_opt(h, m, 0).unwrap();
        assert_eq!(hora_corta(fecha(8, 14)), "08:14 a. m.");
        assert_eq!(hora_corta(fecha(12, 36)), "12:36 p. m.");
        assert_eq!(hora_corta(fecha(0, 5)), "12:05 a. m.");
        assert_eq!(hora_corta(fecha(23, 59)), "11:59 p. m.");
    }
}
