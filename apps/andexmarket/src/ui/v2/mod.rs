pub mod animacion;
pub mod boton;
pub mod campo;
pub mod insignias;
pub mod modal;
pub mod popover;
pub mod segmentado;
pub mod tarjetas;
pub mod toast;

use std::sync::Arc;

use eframe::egui::{text::LayoutJob, Align2, Color32, FontId, Galley, Pos2, Rect, Response, Sense, TextFormat, Ui, Vec2};

pub use animacion::{cargando, girando};
pub use boton::{Boton, BotonIcono, Estilo};
pub use campo::{entrada_desnuda, CampoTexto, Contenedor, EntradaDesnuda, RespuestaCampo};
pub use insignias::{
    chip, chip_estado, chip_metodo, contador, iniciales, medir_tecla, pintar_tecla, punto_estado, tecla, Avatar, EstiloTecla, Tono,
};
pub use modal::{cuerpo_modal, pie_modal, Modal, RespuestaModal};
pub use popover::{desplegable, item_menu, menu_usuario, ItemMenu, Popover, Usuario};
pub use segmentado::{colores_eleccion, opcion_eleccion, segmentado, ColoresEleccion, EstiloSegmentado, Segmento};
pub use tarjetas::{dato_compacto, marco_tabla, shimmer, tarjeta_kpi, Ancho, Columna, Kpi, Tabla};

use super::marca::{color, fuente, tam};
use super::Peso;

pub fn galeria(ui: &Ui, contenido: impl Into<String>, fuente: FontId, color: Color32) -> Arc<Galley> {
    ui.painter().layout_no_wrap(contenido.into(), fuente, color)
}

pub fn galeria_espaciada(ui: &Ui, contenido: impl Into<String>, fuente: FontId, color: Color32, espaciado: f32) -> Arc<Galley> {
    let mut trabajo = LayoutJob::single_section(
        contenido.into(),
        TextFormat { font_id: fuente, color, extra_letter_spacing: espaciado, ..Default::default() },
    );
    trabajo.wrap.max_width = f32::INFINITY;
    ui.painter().layout_job(trabajo)
}

pub fn pintar_texto(ui: &Ui, pos: Pos2, ancla: Align2, contenido: impl Into<String>, px: f32, peso: Peso, color: Color32) -> Rect {
    ui.painter().text(pos, ancla, contenido.into(), fuente(px, peso), color)
}

pub fn texto(ui: &mut Ui, contenido: impl Into<String>, px: f32, peso: Peso, color: Color32) -> Response {
    let g = galeria(ui, contenido, fuente(px, peso), color);
    let (rect, respuesta) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    respuesta
}

pub fn titulo(ui: &mut Ui, contenido: impl Into<String>, px: f32, espaciado_em: f32, color: Color32) -> Response {
    let g = galeria_espaciada(ui, contenido, fuente(px, Peso::Semi), color, px * espaciado_em);
    let (rect, respuesta) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    respuesta
}

pub fn parrafo(ui: &mut Ui, contenido: impl Into<String>, px: f32, color: Color32, interlineado: f32, ancho: f32) -> Response {
    let mut trabajo = LayoutJob::single_section(
        contenido.into(),
        TextFormat { font_id: fuente(px, Peso::Normal), color, line_height: Some(px * interlineado), ..Default::default() },
    );
    trabajo.wrap.max_width = ancho;
    let g = ui.painter().layout_job(trabajo);
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, g.size().y), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    respuesta
}

pub fn etiqueta(ui: &mut Ui, contenido: impl Into<String>) -> Response {
    texto(ui, contenido, tam::CONTROL, Peso::Medio, color::TEXTO_SECUNDARIO)
}

pub fn separador(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::BORDE_SUAVE);
}

pub fn separador_vertical(ui: &mut Ui, alto: f32, color_linea: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, alto), Sense::hover());
    ui.painter().rect_filled(rect, 0, color_linea);
}

pub fn logo(ui: &mut Ui, tam_logo: f32) -> Response {
    ui.add(super::imagen("andex-market-logo.png").fit_to_exact_size(Vec2::splat(tam_logo)))
}

pub fn logo_texto(ui: &mut Ui, ancho: f32) -> Response {
    ui.add(super::imagen("andex-market-text.png").fit_to_exact_size(Vec2::new(ancho, ancho * 28.0 / 230.0)))
}

pub fn pintar_icono(ui: &Ui, centro: Pos2, nombre: &str, tam_icono: f32, color: Color32) {
    super::pintar_icono_solar_en(ui, Rect::from_center_size(centro, Vec2::splat(tam_icono)), nombre, color);
}

pub fn ancho_texto(ui: &Ui, contenido: &str, px: f32, peso: Peso) -> f32 {
    galeria(ui, contenido, fuente(px, peso), Color32::WHITE).size().x
}

pub(crate) fn opacidad(color: Color32, factor: f32) -> Color32 {
    if factor >= 1.0 {
        color
    } else {
        color.gamma_multiply(factor)
    }
}
