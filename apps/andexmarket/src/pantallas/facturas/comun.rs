use std::hash::Hash;
use std::sync::Arc;

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, Galley, Id, Popup, PopupCloseBehavior, Rect, RectAlign, Response,
    Sense, Stroke, StrokeKind, TextFormat, Ui, Vec2,
};

use crate::ui::{self, color, fuente, Direccion, Peso, Tam, RADIO, RADIO_SM, RADIO_XL};

pub struct Bloque {
    pub galeria: Arc<Galley>,
    pub alto_linea: f32,
    desfase: f32,
}

impl Bloque {
    pub fn ancho(&self) -> f32 {
        self.galeria.size().x
    }

    pub fn alto(&self) -> f32 {
        self.galeria.rows.len().max(1) as f32 * self.alto_linea
    }

    pub fn pintar(&self, ui: &Ui, pos: egui::Pos2, color: Color32) {
        ui.painter().galley(pos + Vec2::new(0.0, self.desfase), self.galeria.clone(), color);
    }
}

pub fn bloque(ui: &Ui, texto: &str, tam: Tam, peso: Peso, color: Color32, ancho_max: f32) -> Bloque {
    bloque_alto(ui, texto, tam, peso, color, ancho_max, tam.alto_linea())
}

pub fn bloque_alto(ui: &Ui, texto: &str, tam: Tam, peso: Peso, color: Color32, ancho_max: f32, alto_linea: f32) -> Bloque {
    let fuente_id = fuente(tam, peso);
    let alto_fila = ui.ctx().fonts_mut(|f| f.row_height(&fuente_id));
    let envuelto = envolver(ui, texto, &fuente_id, ancho_max);
    let mut formato = TextFormat::simple(fuente_id, color);
    formato.line_height = Some(alto_linea.max(alto_fila));
    let trabajo = LayoutJob::single_section(envuelto, formato);
    let galeria = ui.painter().layout_job(trabajo);
    Bloque { galeria, alto_linea: alto_linea.max(alto_fila), desfase: ((alto_linea - alto_fila) / 2.0).max(0.0) }
}

fn trozos(texto: &str) -> Vec<&str> {
    let mut trozos = Vec::new();
    let mut inicio = 0;
    for (indice, caracter) in texto.char_indices() {
        if caracter == ' ' || caracter == '-' {
            let fin = indice + caracter.len_utf8();
            trozos.push(&texto[inicio..fin]);
            inicio = fin;
        }
    }
    if inicio < texto.len() {
        trozos.push(&texto[inicio..]);
    }
    trozos
}

fn envolver(ui: &Ui, texto: &str, fuente_id: &FontId, ancho_max: f32) -> String {
    if !ancho_max.is_finite() {
        return texto.to_string();
    }
    let medir = |t: &str| ui.painter().layout_no_wrap(t.trim_end().to_string(), fuente_id.clone(), Color32::WHITE).size().x;
    let mut lineas: Vec<String> = Vec::new();
    let mut actual = String::new();
    for trozo in trozos(texto) {
        let candidata = format!("{actual}{trozo}");
        if !actual.is_empty() && medir(&candidata) > ancho_max {
            lineas.push(actual.trim_end().to_string());
            actual = trozo.to_string();
        } else {
            actual = candidata;
        }
    }
    lineas.push(actual.trim_end().to_string());
    lineas.join("\n")
}

pub fn ancho_texto(ui: &Ui, texto: &str, tam: Tam, peso: Peso) -> f32 {
    ui.painter().layout_no_wrap(texto.to_string(), fuente(tam, peso), Color32::WHITE).size().x
}

pub fn texto_en(ui: &Ui, pos: egui::Pos2, ancla: Align2, texto: &str, tam: Tam, peso: Peso, color: Color32) -> Rect {
    ui.painter().text(pos, ancla, texto, fuente(tam, peso), color)
}

pub fn linea(ui: &mut Ui, texto: impl Into<String>, tam: Tam, peso: Peso, color: Color32) -> Response {
    let texto = texto.into();
    let ancho = ui.available_width();
    let b = bloque(ui, &texto, tam, peso, color, ancho);
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(b.ancho(), b.alto()), Sense::hover());
    b.pintar(ui, rect.min, color);
    respuesta
}

pub fn encoger(base: &[f32], minimos: &[f32], disponible: f32) -> Vec<f32> {
    let mut tamanos = base.to_vec();
    if base.iter().sum::<f32>() <= disponible {
        return tamanos;
    }
    let mut congelado = vec![false; base.len()];
    loop {
        let fijo: f32 = (0..base.len()).filter(|&i| congelado[i]).map(|i| tamanos[i]).sum();
        let libres: Vec<usize> = (0..base.len()).filter(|&i| !congelado[i]).collect();
        if libres.is_empty() {
            return tamanos;
        }
        let base_libre: f32 = libres.iter().map(|&i| base[i]).sum();
        let sobra = disponible - fijo - base_libre;
        let mut violados = false;
        for &i in &libres {
            let propuesto = base[i] + sobra * base[i] / base_libre.max(f32::EPSILON);
            tamanos[i] = propuesto.max(minimos[i]);
            if propuesto < minimos[i] {
                congelado[i] = true;
                violados = true;
            }
        }
        if !violados {
            return tamanos;
        }
    }
}

#[derive(Clone, Copy)]
pub enum Relleno {
    Liso(Color32),
    Degradado(Color32, Color32, Direccion),
}

pub fn esmeralda() -> Relleno {
    Relleno::Degradado(color::ESMERALDA_DESDE, color::ESMERALDA_HASTA, Direccion::AbajoDerecha)
}

pub fn pintar_relleno(ui: &Ui, rect: Rect, radio: f32, relleno: Relleno, opacidad: f32) {
    match relleno {
        Relleno::Liso(c) => {
            ui.painter().rect_filled(rect, radio, c.gamma_multiply(opacidad));
        }
        Relleno::Degradado(desde, hasta, direccion) => ui::pintar_degradado(
            ui.painter(),
            rect,
            radio as u8,
            desde.gamma_multiply(opacidad),
            hasta.gamma_multiply(opacidad),
            direccion,
        ),
    }
}

const TAM_ICONO: f32 = 16.0;

#[derive(Clone)]
pub struct Boton<'a> {
    texto: &'a str,
    icono: Option<&'a str>,
    icono_derecha: Option<&'a str>,
    separacion: f32,
    relleno: Relleno,
    encima: Option<Relleno>,
    color: Color32,
    color_encima: Option<Color32>,
    borde: Option<Color32>,
    radio: f32,
    tam: Tam,
    alto: f32,
    relleno_x: f32,
    habilitado: bool,
}

impl<'a> Boton<'a> {
    fn base(texto: &'a str, relleno: Relleno, color: Color32) -> Self {
        Boton {
            texto,
            icono: None,
            icono_derecha: None,
            separacion: 8.0,
            relleno,
            encima: None,
            color,
            color_encima: None,
            borde: None,
            radio: RADIO as f32,
            tam: Tam::Sm,
            alto: 40.0,
            relleno_x: 16.0,
            habilitado: true,
        }
    }

    pub fn contorno(texto: &'a str) -> Self {
        let mut b = Self::base(texto, Relleno::Liso(color::FONDO), color::TEXTO);
        b.borde = Some(color::BORDE);
        b.encima = Some(Relleno::Liso(color::ACENTO));
        b
    }

    pub fn primario(texto: &'a str) -> Self {
        let mut b = Self::base(texto, Relleno::Liso(color::PRIMARIO), color::PRIMARIO_TEXTO);
        b.encima = Some(Relleno::Liso(color::PRIMARIO.gamma_multiply(0.9)));
        b
    }

    pub fn esmeralda(texto: &'a str) -> Self {
        Self::base(texto, esmeralda(), color::PRIMARIO_TEXTO)
    }

    pub fn chico(mut self) -> Self {
        self.alto = 36.0;
        self.relleno_x = 12.0;
        self
    }

    pub fn icono(mut self, icono: &'a str) -> Self {
        self.icono = Some(icono);
        self
    }

    pub fn icono_derecha(mut self, icono: &'a str) -> Self {
        self.icono_derecha = Some(icono);
        self
    }

    pub fn separacion(mut self, separacion: f32) -> Self {
        self.separacion = separacion;
        self
    }

    pub fn encima(mut self, relleno: Relleno) -> Self {
        self.encima = Some(relleno);
        self
    }

    pub fn borde(mut self, borde: Option<Color32>) -> Self {
        self.borde = borde;
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.habilitado = habilitado;
        self
    }

    fn contenido(&self, ui: &Ui) -> f32 {
        let texto = if self.texto.is_empty() { 0.0 } else { ancho_texto(ui, self.texto, self.tam, Peso::Medio) };
        let iconos = [self.icono, self.icono_derecha].iter().flatten().count();
        let piezas = iconos + usize::from(!self.texto.is_empty());
        texto + iconos as f32 * TAM_ICONO + piezas.saturating_sub(1) as f32 * self.separacion
    }

    pub fn ancho_natural(&self, ui: &Ui) -> f32 {
        self.contenido(ui) + self.relleno_x * 2.0
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let tam = Vec2::new(self.ancho_natural(ui), self.alto);
        let (rect, _) = ui.allocate_exact_size(tam, Sense::hover());
        let id = ui.next_auto_id();
        self.en(ui, rect, id)
    }

    pub fn en(self, ui: &Ui, rect: Rect, id: impl Hash + std::fmt::Debug) -> Response {
        let activo = self.habilitado;
        let sentido = if activo { Sense::click() } else { Sense::hover() };
        let respuesta = ui.interact(rect, ui.id().with(("boton", id)), sentido);
        let encima = activo && respuesta.hovered();
        let opacidad = if activo { 1.0 } else { 0.5 };
        let relleno = match (encima, self.encima) {
            (true, Some(r)) => r,
            _ => self.relleno,
        };
        pintar_relleno(ui, rect, self.radio, relleno, opacidad);
        if let Some(borde) = self.borde {
            ui.painter().rect_stroke(rect, self.radio, Stroke::new(1.0, borde.gamma_multiply(opacidad)), StrokeKind::Inside);
        }
        let color = if encima { self.color_encima.unwrap_or(self.color) } else { self.color }.gamma_multiply(opacidad);
        let contenido = self.contenido(ui);
        let mut x = rect.center().x - contenido / 2.0;
        let [izquierdo, derecho] = [self.icono, self.icono_derecha];
        if let Some(nombre) = izquierdo {
            let caja = Rect::from_center_size(egui::pos2(x + TAM_ICONO / 2.0, rect.center().y), Vec2::splat(TAM_ICONO));
            ui::pintar_icono_en(ui, caja, nombre, color);
            x += TAM_ICONO + self.separacion;
        }
        if !self.texto.is_empty() {
            let r = texto_en(ui, egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, self.texto, self.tam, Peso::Medio, color);
            x = r.right() + self.separacion;
        }
        if let Some(nombre) = derecho {
            let caja = Rect::from_center_size(egui::pos2(x + TAM_ICONO / 2.0, rect.center().y), Vec2::splat(TAM_ICONO));
            ui::pintar_icono_en(ui, caja, nombre, color);
        }
        if activo {
            respuesta.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            respuesta
        }
    }
}

pub struct Opcion<'a> {
    pub icono: Option<&'a str>,
    pub texto: String,
}

pub fn selector(
    ui: &mut Ui,
    id: impl Hash + std::fmt::Debug,
    ancho: f32,
    seleccion: &mut usize,
    opciones: &[Opcion],
    habilitado: bool,
) -> bool {
    let (rect, respuesta) =
        ui.allocate_exact_size(Vec2::new(ancho, 40.0), if habilitado { Sense::click() } else { Sense::hover() });
    let opacidad = if habilitado { 1.0 } else { 0.5 };
    ui.painter().rect(
        rect,
        RADIO_XL,
        color::FONDO,
        Stroke::new(1.0, color::BORDE_ENTRADA.gamma_multiply(opacidad)),
        StrokeKind::Inside,
    );
    if let Some(opcion) = opciones.get(*seleccion) {
        let mut x = rect.left() + 12.0;
        if let Some(icono) = opcion.icono {
            let caja = Rect::from_center_size(egui::pos2(x + 8.0, rect.center().y), Vec2::splat(16.0));
            ui::pintar_icono_en(ui, caja, icono, color::TEXTO.gamma_multiply(opacidad));
            x += 24.0;
        }
        texto_en(ui, egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, &opcion.texto, Tam::Sm, Peso::Normal, color::TEXTO.gamma_multiply(opacidad));
    }
    let flecha = Rect::from_center_size(egui::pos2(rect.right() - 20.0, rect.center().y), Vec2::splat(16.0));
    ui::pintar_icono_en(ui, flecha, "chevron-down", color::TEXTO.gamma_multiply(0.5 * opacidad));
    let popup_id = Id::new("selector-facturas").with(id);
    if habilitado && respuesta.clicked() {
        Popup::toggle_id(ui.ctx(), popup_id);
    }
    let mut elegido = None;
    Popup::new(popup_id, ui.ctx().clone(), rect, ui.layer_id())
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .align(RectAlign::BOTTOM_START)
        .gap(4.0)
        .width(ancho)
        .frame(ui::marco_popup().shadow(ui::sombra_md()))
        .show(|ui| {
            ui.set_width(ancho - 8.0);
            let alguno_encima = ui.rect_contains_pointer(ui.max_rect());
            for (indice, opcion) in opciones.iter().enumerate() {
                let (caja, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
                if r.hovered() || (!alguno_encima && indice == *seleccion) {
                    ui.painter().rect_filled(caja, RADIO_SM, color::ACENTO);
                }
                if indice == *seleccion {
                    let marca = Rect::from_center_size(egui::pos2(caja.left() + 15.0, caja.center().y), Vec2::splat(16.0));
                    ui::pintar_icono_en(ui, marca, "check", color::TEXTO);
                }
                let mut x = caja.left() + 32.0;
                if let Some(icono) = opcion.icono {
                    let marca = Rect::from_center_size(egui::pos2(x + 8.0, caja.center().y), Vec2::splat(16.0));
                    ui::pintar_icono_en(ui, marca, icono, color::TEXTO);
                    x += 24.0;
                }
                texto_en(ui, egui::pos2(x, caja.center().y), Align2::LEFT_CENTER, &opcion.texto, Tam::Sm, Peso::Normal, color::TEXTO);
                if r.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    elegido = Some(indice);
                }
            }
        });
    match elegido {
        Some(indice) if indice != *seleccion => {
            *seleccion = indice;
            true
        }
        _ => false,
    }
}

pub fn titulo_dialogo(ui: &mut Ui, icono: &str, titulo: &str, color_titulo: Color32, color_icono: Color32, separacion: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
    let caja = Rect::from_min_size(rect.min, Vec2::splat(20.0));
    ui::pintar_icono_en(ui, caja, icono, color_icono);
    texto_en(ui, egui::pos2(caja.right() + separacion, rect.center().y), Align2::LEFT_CENTER, titulo, Tam::Lg, Peso::Semi, color_titulo);
}

pub fn encabezado_dialogo(ui: &mut Ui, icono: &str, titulo: &str, descripcion: &str, separacion: f32) {
    titulo_dialogo(ui, icono, titulo, color::TEXTO, color::TEXTO, separacion);
    ui.add_space(6.0);
    linea(ui, descripcion, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
}

pub fn etiqueta(ui: &mut Ui, texto: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0), Sense::hover());
    texto_en(ui, egui::pos2(rect.left(), rect.top() + 13.0), Align2::LEFT_CENTER, texto, Tam::Sm, Peso::Medio, color::TEXTO);
}

pub fn desplazable<R>(ui: &mut Ui, id: Id, alto_max: f32, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    let previo: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(alto_max);
    let alto = previo.min(alto_max);
    let salida = egui::ScrollArea::vertical()
        .id_salt(id)
        .max_height(alto)
        .min_scrolled_height(alto)
        .auto_shrink([false, true])
        .show(ui, contenido);
    let nuevo = salida.content_size.y;
    if (nuevo.min(alto_max) - alto).abs() > 0.5 {
        ui.ctx().request_repaint();
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, nuevo));
    salida.inner
}

pub fn borde_superior(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::BORDE);
}

pub fn cargando_centro(ui: &mut Ui, alto: f32, texto: &str, color_giro: Color32, color_texto: Color32, tam: Tam) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    let ancho = 24.0 + 8.0 + ancho_texto(ui, texto, tam, Peso::Normal);
    let x = rect.center().x - ancho / 2.0;
    ui::girando(ui, Rect::from_center_size(egui::pos2(x + 12.0, rect.center().y), Vec2::splat(24.0)), color_giro);
    texto_en(ui, egui::pos2(x + 32.0, rect.center().y), Align2::LEFT_CENTER, texto, tam, Peso::Normal, color_texto);
}

pub fn fila_paginacion(
    ui: &mut Ui,
    id: &str,
    pagina: u32,
    paginas: &[u32],
    anterior: bool,
    siguiente: bool,
    separacion_numeros: f32,
) -> Option<u32> {
    let mut destino = None;
    let ancho = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 36.0), Sense::hover());
    let b_anterior = Boton::contorno("Anterior").chico().icono("chevron-left").separacion(12.0);
    let r = rect.left_top();
    let ancho_ant = b_anterior.ancho_natural(ui);
    if b_anterior.habilitado(anterior).en(ui, Rect::from_min_size(r, Vec2::new(ancho_ant, 36.0)), (id, "ant")).clicked() {
        destino = Some(pagina.saturating_sub(1));
    }
    let b_siguiente = Boton::contorno("Siguiente").chico().icono_derecha("chevron-right").separacion(12.0);
    let ancho_sig = b_siguiente.ancho_natural(ui);
    let caja_sig = Rect::from_min_size(egui::pos2(rect.right() - ancho_sig, rect.top()), Vec2::new(ancho_sig, 36.0));
    if b_siguiente.habilitado(siguiente).en(ui, caja_sig, (id, "sig")).clicked() {
        destino = Some(pagina + 1);
    }
    let total = paginas.len() as f32 * 32.0 + paginas.len().saturating_sub(1) as f32 * separacion_numeros;
    let mut x = rect.center().x - total / 2.0;
    for &numero in paginas {
        let caja = Rect::from_min_size(egui::pos2(x, rect.center().y - 16.0), Vec2::splat(32.0));
        let texto = numero.to_string();
        let boton = if numero == pagina { Boton::primario(&texto) } else { Boton::contorno(&texto) };
        if boton.chico().en(ui, caja, (id, numero)).clicked() {
            destino = Some(numero);
        }
        x += 32.0 + separacion_numeros;
    }
    destino
}

#[cfg(test)]
mod tests {
    use super::encoger;

    #[test]
    fn encoger_como_flex_shrink() {
        assert_eq!(encoger(&[100.0, 50.0], &[0.0, 0.0], 200.0), vec![100.0, 50.0]);
        let r = encoger(&[200.0, 100.0], &[0.0, 0.0], 240.0);
        assert!((r[0] - 160.0).abs() < 0.01 && (r[1] - 80.0).abs() < 0.01);
        let r = encoger(&[200.0, 100.0], &[0.0, 90.0], 240.0);
        assert!((r[1] - 90.0).abs() < 0.01 && (r[0] - 150.0).abs() < 0.01);
    }
}
