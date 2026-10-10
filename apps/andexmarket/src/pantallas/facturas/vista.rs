use eframe::egui::{self, Align, Align2, Color32, CursorIcon, Frame, Layout, Margin, Pos2, Rect, Response, ScrollArea, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2};
use nucleo::facturas::{self as logica, EstadoTarjeta};
use nucleo::formato;
use serde_json::Value;

use super::{numero, pesos, texto};
use crate::ui::marca::{alto, color, fuente, radio, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, Boton, Estilo};
use crate::ui::{imagen, pintar_icono_solar_en, Peso};

pub const RELLENO_CHIP_TIPO: Vec2 = Vec2::new(7.0, 1.0);
pub const RELLENO_CHIP_ESTADO: Vec2 = Vec2::new(8.0, 2.0);

pub fn fondo(ui: &Ui) {
    ui.painter().rect_filled(ui.max_rect(), 0, color::FONDO_APP);
}

pub fn encabezado(ui: &mut Ui, volver: &str, titulo: &str) -> bool {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto::ENCABEZADO), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::VERDE_PROFUNDO);
    let mut fila = ui.new_child(UiBuilder::new().max_rect(rect.shrink2(Vec2::new(20.0, 0.0))).layout(Layout::left_to_right(Align::Center)));
    fila.spacing_mut().item_spacing.x = 16.0;
    let clic = Boton::new(volver, Estilo::FantasmaOscuro)
        .icono("arrow-left")
        .tam_icono(19.0)
        .color_icono(color::SOBRE_OSCURO)
        .alto(alto::CONTROL)
        .relleno(12.0)
        .radio(radio::CONTROL)
        .tam_texto(tam::CUERPO)
        .peso(Peso::Medio)
        .ui(&mut fila)
        .clicked();
    v2::separador_vertical(&mut fila, 28.0, color::VERDE);
    v2::texto(&mut fila, titulo, tam::FILA, Peso::Semi, color::BLANCO);
    fila.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add(imagen("andex-market-text.png").fit_to_exact_size(Vec2::new(112.0, 112.0 * 28.0 / 230.0)).tint(Color32::from_white_alpha(230)));
    });
    clic
}

pub fn tarjeta(relleno_x: i8, relleno_y: i8) -> Frame {
    Frame::NONE
        .fill(color::BLANCO)
        .stroke(Stroke::new(1.0, color::BORDE))
        .corner_radius(radio::TARJETA)
        .inner_margin(Margin::symmetric(relleno_x, relleno_y))
}

pub fn medir_chip(ui: &Ui, contenido: &str, px: f32, relleno: Vec2) -> Vec2 {
    v2::galeria(ui, contenido, fuente(px, Peso::Semi), color::TEXTO).size() + relleno * 2.0
}

pub fn pintar_chip(ui: &Ui, pos: Pos2, contenido: &str, px: f32, relleno: Vec2, fondo: Color32, tinta: Color32) -> Rect {
    let g = v2::galeria(ui, contenido, fuente(px, Peso::Semi), tinta);
    let rect = Rect::from_min_size(pos, g.size() + relleno * 2.0);
    ui.painter().rect_filled(rect, radio::CHIP, fondo);
    ui.painter().galley(rect.min + relleno, g, tinta);
    rect
}

pub fn chip(ui: &mut Ui, contenido: &str, px: f32, relleno: Vec2, colores: (Color32, Color32)) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(medir_chip(ui, contenido, px, relleno), Sense::hover());
    pintar_chip(ui, rect.min, contenido, px, relleno, colores.0, colores.1);
    respuesta
}

pub fn tipo(tipo_factura: &str) -> (String, (Color32, Color32)) {
    match tipo_factura.to_lowercase().as_str() {
        "remito" => ("Remito".into(), (color::DORADO_FONDO, color::DORADO_TEXTO)),
        "a" => ("Factura A".into(), (color::AZUL_FONDO, color::AZUL)),
        "b" => ("Factura B".into(), (color::VERDE_EXITO_FONDO, color::VERDE_EXITO)),
        "c" => ("Factura C".into(), (color::VIOLETA_FONDO, color::VIOLETA)),
        "" => ("Sin tipo".into(), (color::FONDO_APP, color::TEXTO_APAGADO)),
        _ => (tipo_factura.to_string(), (color::FONDO_APP, color::TEXTO_APAGADO)),
    }
}

pub fn estado(factura: &Value) -> (String, (Color32, Color32)) {
    let pagada = (color::VERDE_EXITO_FONDO, color::VERDE_EXITO);
    let pendiente = (color::ROJO_FONDO, color::ROJO);
    match logica::estado_tarjeta(factura) {
        EstadoTarjeta::Pagada | EstadoTarjeta::SegunBackend { pagada: true, .. } => ("Pagada".into(), pagada),
        EstadoTarjeta::Parcial => ("Parcial".into(), (color::DORADO_FONDO, color::DORADO_TEXTO)),
        EstadoTarjeta::Impaga => ("Pendiente".into(), pendiente),
        EstadoTarjeta::SegunBackend { texto, .. } => match texto.as_str() {
            "anulada" => ("Anulada".into(), (color::FONDO_APP, color::TEXTO_APAGADO)),
            "" => ("Pendiente".into(), pendiente),
            otro => (capitalizar(otro), pendiente),
        },
    }
}

pub fn capitalizar(texto: &str) -> String {
    let mut salida = String::new();
    let mut inicio = true;
    for c in texto.chars() {
        if inicio {
            salida.extend(c.to_uppercase());
        } else {
            salida.push(c);
        }
        inicio = c.is_whitespace() || c == '-' || c == '_';
    }
    salida
}

pub fn condicion_fiscal(condicion: &str) -> String {
    match condicion {
        "consumidor_final" => "Consumidor Final".into(),
        "responsable_inscripto" => "Responsable Inscripto".into(),
        "exento" => "Exento".into(),
        "monotributo" => "Monotributo".into(),
        otra => otra.to_string(),
    }
}

pub fn nombre_cliente(factura: &Value) -> String {
    logica::nombre_o_razon(&factura["cliente"]).unwrap_or_else(|| "Cliente no especificado".into())
}

pub fn cantidad_con_medida(cantidad: f64, tipo_medida: &str) -> String {
    let unidad = match tipo_medida.to_lowercase().as_str() {
        "kg" | "kilo" | "kilos" | "kilogramo" => "kg".to_string(),
        "" | "u" | "unidad" | "unidades" => "u".to_string(),
        otra => otra.to_string(),
    };
    format!("{} {unidad}", formato::numero(cantidad))
}

pub fn productos(cantidad: usize) -> String {
    format!("{cantidad} producto{}", if cantidad == 1 { "" } else { "s" })
}

pub fn metodo(metodo: &str) -> (&'static str, String) {
    match metodo.to_lowercase().as_str() {
        "efectivo" => ("cash", "Efectivo".into()),
        "tarjeta" => ("card", "Tarjeta".into()),
        "transferencia" => ("transfer", "Transferencia".into()),
        "qr" => ("qr", "QR".into()),
        _ => ("wallet", capitalizar(&metodo.replacen('_', " ", 1))),
    }
}

pub fn fila_pago(ui: &mut Ui, pago: &Value, relleno: f32, borde_arriba: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0 + relleno * 2.0), Sense::hover());
    let linea = if borde_arriba { rect.top() + 0.5 } else { rect.bottom() - 0.5 };
    ui.painter().hline(rect.x_range(), linea, Stroke::new(1.0, color::BORDE_SUAVE));
    let (icono, etiqueta) = metodo(texto(pago, "metodoPago"));
    let caja = Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 17.0), Vec2::splat(34.0));
    ui.painter().rect_filled(caja, radio::CONTROL, color::VERDE_CHIP);
    v2::pintar_icono(ui, caja.center(), icono, 18.0, color::VERDE);
    let monto = v2::pintar_texto(
        ui,
        egui::pos2(rect.right(), rect.center().y),
        Align2::RIGHT_CENTER,
        pesos(numero(pago, "monto")),
        tam::CUERPO_MAS,
        Peso::Semi,
        color::VERDE_EXITO,
    );
    let mut detalle = logica::fecha_es_ar(texto(pago, "fecha"));
    let referencia = texto(pago, "referencia");
    if !referencia.is_empty() {
        detalle.push_str(" · ");
        detalle.push_str(referencia);
    }
    let x = caja.right() + 12.0;
    let derecha = monto.left() - 12.0;
    let arriba = rect.center().y - 16.5;
    celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=derecha, arriba..=arriba + 17.0), &etiqueta, tam::CUERPO, Peso::Medio, color::TEXTO, false);
    celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=derecha, arriba + 18.0..=arriba + 33.0), &detalle, tam::XS, Peso::Normal, color::TEXTO_TENUE, false);
}

pub fn etiqueta(ui: &mut Ui, contenido: &str) -> Response {
    v2::texto(ui, contenido, tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO)
}

pub fn cargando(ui: &mut Ui, alto: f32, mensaje: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    let g = v2::galeria(ui, mensaje, fuente(tam::CUERPO, Peso::Normal), color::TEXTO_APAGADO);
    let x = rect.center().x - (30.0 + g.size().x) / 2.0;
    v2::girando(ui, Rect::from_center_size(egui::pos2(x + 10.0, rect.center().y), Vec2::splat(20.0)), color::VERDE);
    ui.painter().galley(egui::pos2(x + 30.0, rect.center().y - g.size().y / 2.0), g, color::TEXTO_APAGADO);
}

pub fn vacio(ui: &mut Ui, icono: &str, mensaje: &str) {
    ui.add_space(48.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::hover());
    v2::pintar_icono(ui, rect.center(), icono, 34.0, color::SOBRE_OSCURO_APAGADO);
    ui.add_space(8.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
    v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, mensaje, tam::CUERPO_MAS, Peso::Normal, color::TEXTO_APAGADO);
}

pub fn boton_centrado(ui: &mut Ui, boton: Boton) -> bool {
    ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 36.0), Layout::top_down(Align::Center), |ui| boton.ui(ui).clicked()).inner
}

pub fn boton_pagina(ui: &mut Ui, etiqueta: &str, icono: &str, icono_a_la_derecha: bool, habilitado: bool) -> Response {
    let g = v2::galeria(ui, etiqueta, fuente(tam::CONTROL, Peso::Normal), color::TEXTO_SECUNDARIO);
    let ancho = 20.0 + 15.0 + 4.0 + g.size().x;
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 32.0), if habilitado { Sense::click() } else { Sense::hover() });
    let encima = habilitado && respuesta.hovered();
    let alfa = if habilitado { 1.0 } else { 0.4 };
    let tinta = if encima { color::VERDE } else { color::TEXTO_SECUNDARIO }.gamma_multiply(alfa);
    let borde = if encima { color::VERDE } else { color::BORDE }.gamma_multiply(alfa);
    ui.painter().rect(rect, radio::ICONO, color::BLANCO, Stroke::new(1.0, borde), StrokeKind::Inside);
    let (x_icono, x_texto) = if icono_a_la_derecha {
        (rect.right() - 10.0 - 7.5, rect.left() + 10.0)
    } else {
        (rect.left() + 10.0 + 7.5, rect.left() + 10.0 + 19.0)
    };
    pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(x_icono, rect.center().y), Vec2::splat(15.0)), icono, tinta);
    ui.painter().galley(egui::pos2(x_texto, rect.center().y - g.size().y / 2.0), g, tinta);
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
}

pub fn fila_dato(ui: &mut Ui, etiqueta: &str, valor: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 42.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
    let izquierda = v2::pintar_texto(ui, rect.left_center(), Align2::LEFT_CENTER, etiqueta, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
    let celda = Rect::from_x_y_ranges(izquierda.right() + 16.0..=rect.right(), rect.y_range());
    celda_texto(ui.painter(), celda, valor, tam::CUERPO, Peso::Medio, color::TEXTO, true);
}

pub fn cuerpo_desplazable<R>(ui: &mut Ui, id: &str, reservado: f32, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    let alto = (ui.ctx().content_rect().height() - 48.0 - reservado).max(160.0);
    ScrollArea::vertical().id_salt(id).max_height(alto).auto_shrink([false, true]).show(ui, contenido).inner
}

#[cfg(test)]
mod tests {
    use super::{cantidad_con_medida, capitalizar, metodo, productos};

    #[test]
    fn textos_de_la_tabla() {
        assert_eq!(capitalizar("remito"), "Remito");
        assert_eq!(capitalizar("responsable inscripto"), "Responsable Inscripto");
        assert_eq!(cantidad_con_medida(18.0, "Kg"), "18 kg");
        assert_eq!(cantidad_con_medida(2.5, "Unidad"), "2,5 u");
        assert_eq!(productos(1), "1 producto");
        assert_eq!(productos(3), "3 productos");
        assert_eq!(metodo("transferencia"), ("transfer", "Transferencia".to_string()));
        assert_eq!(metodo("cuenta_corriente").1, "Cuenta Corriente");
    }
}
