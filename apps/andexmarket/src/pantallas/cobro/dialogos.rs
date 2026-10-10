use eframe::egui::{
    self, Align, Align2, Color32, Context, CursorIcon, Id, Key, Modifiers, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind,
    TextEdit, Ui, Vec2,
};
use nucleo::cobros::{self, fijo, local, numero_js, Descuento};
use serde_json::Value;

use super::{Cobro, Lado};
use crate::app::Contexto;
use crate::ui::{self, color, fuente, paleta, Boton, Peso, Tam, Variante};

const GRIS_ENTRADA: Color32 = color::BORDE_ENTRADA;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Capa {
    Cancelar,
    PagoAfip,
    Descuento,
    PagoNormal,
    EfectivoAfip,
    EfectivoNormal,
    Cancelacion,
    Qr,
    ManualNormal,
    ManualAfip,
    MixtoNormal,
    MixtoAfip,
}

fn opacar(c: Color32, habilitado: bool) -> Color32 {
    if habilitado {
        c
    } else {
        c.gamma_multiply(0.5)
    }
}

fn medir(ui: &Ui, texto: &str, tam: Tam, peso: Peso) -> f32 {
    ui.painter().layout_no_wrap(texto.to_string(), fuente(tam, peso), Color32::WHITE).size().x
}

fn pintar(ui: &Ui, pos: Pos2, ancla: Align2, texto: &str, tam: Tam, peso: Peso, color: Color32) -> Rect {
    ui.painter().text(pos, ancla, texto, fuente(tam, peso), color)
}

fn tachar(ui: &Ui, rect: Rect, color: Color32) {
    let y = rect.center().y + 1.0;
    ui.painter().line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], Stroke::new(1.0, color));
}

fn caja(ui: &mut Ui, alto: f32) -> Rect {
    ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover()).0
}

fn envolver(ui: &Ui, texto: &str, tam: Tam, peso: Peso, ancho: f32) -> Vec<String> {
    let mut piezas: Vec<(String, bool)> = Vec::new();
    for palabra in texto.split(' ') {
        let mut primera = true;
        let mut resto = palabra;
        while let Some(i) = resto.find('-').filter(|i| *i + 1 < resto.len()) {
            piezas.push((resto[..=i].to_string(), primera));
            primera = false;
            resto = &resto[i + 1..];
        }
        piezas.push((resto.to_string(), primera));
    }
    let mut lineas: Vec<String> = Vec::new();
    let mut actual = String::new();
    for (pieza, con_espacio) in piezas {
        let candidato = if actual.is_empty() {
            pieza.clone()
        } else if con_espacio {
            format!("{actual} {pieza}")
        } else {
            format!("{actual}{pieza}")
        };
        if !actual.is_empty() && medir(ui, candidato.trim_end(), tam, peso) > ancho {
            lineas.push(actual.trim_end().to_string());
            actual = pieza;
        } else {
            actual = candidato;
        }
    }
    lineas.push(actual);
    lineas
}

fn parrafo(ui: &mut Ui, texto: &str, tam: Tam, peso: Peso, color: Color32, alto_linea: f32, ancla: Align2) {
    let ancho = ui.available_width();
    let lineas = envolver(ui, texto, tam, peso, ancho);
    let rect = caja(ui, alto_linea * lineas.len() as f32);
    for (i, linea) in lineas.iter().enumerate() {
        let y = rect.top() + alto_linea * (i as f32 + 0.5);
        let x = match ancla.x() {
            Align::Min => rect.left(),
            Align::Center => rect.center().x,
            Align::Max => rect.right(),
        };
        pintar(ui, egui::pos2(x, y), Align2([ancla.x(), Align::Center]), linea, tam, peso, color);
    }
}

fn titulo(ui: &mut Ui, texto: &str, tam: Tam, alto: f32, color: Color32, icono: Option<&str>) {
    let rect = caja(ui, alto.max(if icono.is_some() { 20.0 } else { 0.0 }));
    let mut x = rect.left();
    if let Some(nombre) = icono {
        ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(x, rect.center().y - 10.0), Vec2::splat(20.0)), nombre, color);
        x += 28.0;
    }
    let ancho = rect.right() - x;
    let lineas = envolver(ui, texto, tam, Peso::Semi, ancho);
    if lineas.len() == 1 {
        pintar(ui, egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, texto, tam, Peso::Semi, color);
    } else {
        ui.add_space(alto * (lineas.len() as f32 - 1.0));
        for (i, linea) in lineas.iter().enumerate() {
            let y = rect.top() + alto * (i as f32 + 0.5);
            pintar(ui, egui::pos2(x, y), Align2::LEFT_CENTER, linea, tam, Peso::Semi, color);
        }
    }
}

fn encabezado(ui: &mut Ui, texto: &str, tam: Tam, alto: f32, color_titulo: Color32, icono: Option<&str>, descripcion: &str) {
    titulo(ui, texto, tam, alto, color_titulo, icono);
    if !descripcion.is_empty() {
        ui.add_space(6.0);
        parrafo(ui, descripcion, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO, 20.0, Align2::LEFT_TOP);
    }
}

fn anillo(ui: &Ui, rect: Rect, radio: f32) {
    ui.painter().rect_stroke(rect.expand(3.0), radio + 3.0, Stroke::new(2.0, color::ANILLO), StrokeKind::Middle);
}

fn girador(ui: &Ui, centro: Pos2, radio: f32, color: Color32) {
    let t = ui.input(|i| i.time) as f32 * std::f32::consts::TAU;
    let puntos: Vec<Pos2> = (0..=24)
        .map(|k| {
            let a = t + std::f32::consts::FRAC_PI_4 + (k as f32 / 24.0) * std::f32::consts::FRAC_PI_2;
            centro + Vec2::angled(a) * (radio - 1.0)
        })
        .collect();
    ui.painter().add(Shape::line(puntos, Stroke::new(2.0, color)));
    ui.ctx().request_repaint();
}

fn fila(ui: &mut Ui, alto: f32, izquierda: (&str, Tam, Peso, Color32), derecha: (&str, Tam, Peso, Color32)) -> (Rect, Rect) {
    let rect = caja(ui, alto);
    let i = pintar(ui, rect.left_center(), Align2::LEFT_CENTER, izquierda.0, izquierda.1, izquierda.2, izquierda.3);
    let d = pintar(ui, rect.right_center(), Align2::RIGHT_CENTER, derecha.0, derecha.1, derecha.2, derecha.3);
    (i, d)
}

fn recuadro(ui: &mut Ui, fondo: Color32, borde: Color32, relleno: f32, contenido: impl FnOnce(&mut Ui)) {
    ui::marco()
        .fill(fondo)
        .stroke(Stroke::new(1.0, borde))
        .corner_radius(ui::RADIO)
        .inner_margin(egui::Margin::same(relleno as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenido(ui);
        });
}

struct Campo<'a> {
    id: Id,
    texto: &'a mut String,
    ancho: f32,
    alto: f32,
    sugerencia: &'a str,
    alineacion: Align,
    habilitado: bool,
    numerico: bool,
    clave: bool,
    multilinea: bool,
}

impl<'a> Campo<'a> {
    fn new(id: impl std::hash::Hash + std::fmt::Debug, texto: &'a mut String, sugerencia: &'a str) -> Self {
        Campo {
            id: Id::new(("cobro-campo", id)),
            texto,
            ancho: f32::INFINITY,
            alto: 40.0,
            sugerencia,
            alineacion: Align::Min,
            habilitado: true,
            numerico: true,
            clave: false,
            multilinea: false,
        }
    }

    fn ui(self, ui: &mut Ui, enfocar: &mut bool) -> Response {
        let ancho = self.ancho.min(ui.available_width());
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, self.alto), Sense::hover());
        let (radio, borde) = if self.multilinea { (ui::RADIO_MD, color::BORDE) } else { (ui::RADIO_XL, GRIS_ENTRADA) };
        ui.painter().rect(
            rect,
            radio,
            opacar(color::FONDO, self.habilitado),
            Stroke::new(1.0, opacar(borde, self.habilitado)),
            StrokeKind::Inside,
        );
        let enfocado = ui.memory(|m| m.has_focus(self.id));
        let flechas = self.numerico && enfocado;
        let mut interior = rect.shrink2(Vec2::new(12.0, if self.multilinea { 8.0 } else { 0.0 }));
        if self.multilinea {
            interior.min.y += 2.0;
        }
        if flechas {
            interior.max.x -= 15.0;
            let centro = egui::pos2(rect.right() - 20.5, rect.center().y);
            let gris = Color32::from_gray(110);
            ui.painter().add(Shape::convex_polygon(
                vec![centro + Vec2::new(0.0, -7.5), centro + Vec2::new(4.0, -1.5), centro + Vec2::new(-4.0, -1.5)],
                gris,
                Stroke::NONE,
            ));
            ui.painter().add(Shape::convex_polygon(
                vec![centro + Vec2::new(-4.0, 1.5), centro + Vec2::new(4.0, 1.5), centro + Vec2::new(0.0, 7.5)],
                gris,
                Stroke::NONE,
            ));
        }
        let alto_linea = if self.multilinea { Some(20.0) } else { None };
        let sugerencia = egui::WidgetText::from(
            egui::RichText::new(self.sugerencia).font(fuente(Tam::Sm, Peso::Normal)).color(color::APAGADO_TEXTO).line_height(alto_linea),
        );
        let mut maquetar = |ui: &Ui, texto: &dyn egui::TextBuffer, ancho: f32| {
            let mut trabajo = egui::text::LayoutJob::simple(texto.as_str().to_owned(), fuente(Tam::Sm, Peso::Normal), color::TEXTO, ancho);
            trabajo.sections[0].format.line_height = alto_linea;
            ui.fonts_mut(|f| f.layout_job(trabajo))
        };
        let editor = if self.multilinea { TextEdit::multiline(self.texto).layouter(&mut maquetar) } else { TextEdit::singleline(self.texto) };
        let mut hijo = ui.new_child(
            egui::UiBuilder::new().max_rect(interior).layout(egui::Layout::centered_and_justified(egui::Direction::TopDown)),
        );
        let respuesta = hijo.add(
            editor
                .id(self.id)
                .frame(egui::Frame::NONE)
                .hint_text(sugerencia)
                .password(self.clave)
                .font(fuente(Tam::Sm, Peso::Normal))
                .text_color(color::TEXTO)
                .horizontal_align(self.alineacion)
                .vertical_align(if self.multilinea { Align::Min } else { Align::Center })
                .margin(egui::Margin::ZERO)
                .desired_width(interior.width())
                .desired_rows(if self.multilinea { 4 } else { 1 })
                .interactive(self.habilitado),
        );
        if self.multilinea && enfocado {
            anillo(ui, rect, radio as f32);
        }
        if *enfocar {
            respuesta.request_focus();
            *enfocar = false;
        }
        respuesta
    }
}

fn filtrar_numero(texto: &mut String) {
    texto.retain(|c| c.is_ascii_digit() || c == '.' || c == '-');
}

fn ancho_boton(ui: &Ui, texto: &str, tam: Tam, cargando: bool, relleno: f32) -> f32 {
    medir(ui, texto, tam, Peso::Medio) + if cargando { 24.0 } else { 0.0 } + relleno * 2.0
}

fn boton_confirmar(ui: &mut Ui, cargando: bool, habilitado: bool) -> Response {
    let texto = if cargando { "Procesando..." } else { "Confirmar pago" };
    Boton::new(texto)
        .variante(Variante::Esmeralda)
        .texto_tam(Tam::Lg)
        .cargando(cargando)
        .ancho(ancho_boton(ui, texto, Tam::Lg, cargando, 24.0))
        .habilitado(habilitado)
        .ui(ui)
}

fn enter(ctx: &Context) -> bool {
    ctx.input(|i| i.key_pressed(Key::Enter))
}

fn mantener_foco(ctx: &Context, respuesta: &Response) {
    if respuesta.lost_focus() && enter(ctx) {
        respuesta.request_focus();
    }
}

enum Icono {
    Uno(&'static str),
    Mixto,
}

#[allow(clippy::too_many_arguments)]
fn boton_metodo(ui: &mut Ui, rect: Rect, id: &str, etiqueta: &str, icono: Icono, seleccionado: bool, girando: bool, habilitado: bool, foco: bool) -> bool {
    let respuesta = ui.interact(rect, Id::new(("cobro-metodo", id)), if habilitado { Sense::click() } else { Sense::hover() });
    let encima = habilitado && respuesta.hovered();
    let fondo = if seleccionado {
        paleta::EMERALD_100
    } else if encima {
        color::ACENTO
    } else {
        color::FONDO
    };
    let (borde, grosor) = if seleccionado { (paleta::EMERALD_600, 2.0) } else { (GRIS_ENTRADA, 1.0) };
    ui.painter().rect(rect, ui::RADIO, opacar(fondo, habilitado), Stroke::new(grosor, opacar(borde, habilitado)), StrokeKind::Inside);
    let tinta = opacar(color::TEXTO, habilitado);
    let centro_icono = egui::pos2(rect.center().x, rect.top() + 45.5);
    if girando {
        girador(ui, centro_icono, 12.0, opacar(paleta::GRAY_900, habilitado));
    } else {
        match icono {
            Icono::Uno(nombre) => ui::pintar_icono_en(ui, Rect::from_center_size(centro_icono, Vec2::splat(32.0)), nombre, tinta),
            Icono::Mixto => {
                let mas = medir(ui, "+", Tam::Xl, Peso::Normal);
                let total = 32.0 + 4.0 + mas + 4.0 + 32.0;
                let x = rect.center().x - total / 2.0;
                ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(x, centro_icono.y - 16.0), Vec2::splat(32.0)), "wallet", tinta);
                pintar(ui, egui::pos2(x + 36.0, centro_icono.y), Align2::LEFT_CENTER, "+", Tam::Xl, Peso::Normal, tinta);
                let x2 = x + 36.0 + mas + 4.0;
                ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(x2, centro_icono.y - 16.0), Vec2::splat(32.0)), "credit-card", tinta);
            }
        }
    }
    pintar(ui, egui::pos2(rect.center().x, rect.top() + 95.5), Align2::CENTER_CENTER, etiqueta, Tam::Sm, Peso::Medio, tinta);
    if foco && habilitado {
        anillo(ui, rect, ui::RADIO as f32);
    }
    respuesta.on_hover_cursor(if habilitado { CursorIcon::PointingHand } else { CursorIcon::Default }).clicked()
}

fn decodificar_base64(texto: &str) -> Option<Vec<u8>> {
    let valor = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return None,
        } as u32)
    };
    let limpio: Vec<u8> = texto.bytes().filter(|b| !b.is_ascii_whitespace() && *b != b'=').collect();
    let mut salida = Vec::with_capacity(limpio.len() * 3 / 4);
    for bloque in limpio.chunks(4) {
        let mut acumulado = 0u32;
        for (i, b) in bloque.iter().enumerate() {
            acumulado |= valor(*b)? << (18 - 6 * i);
        }
        let bytes = acumulado.to_be_bytes();
        salida.extend_from_slice(&bytes[1..bloque.len()]);
    }
    Some(salida)
}

pub(super) fn imagen_de_url(ctx: &Context, url: &str) -> Option<String> {
    let resto = url.strip_prefix("data:")?;
    let (cabecera, datos) = resto.split_once(',')?;
    let bytes = if cabecera.ends_with(";base64") {
        decodificar_base64(datos)?
    } else {
        datos.as_bytes().to_vec()
    };
    let extension = if cabecera.contains("svg") { "svg" } else if cabecera.contains("webp") { "webp" } else { "png" };
    let uri = format!("bytes://cobro-qr/{}.{extension}", Id::new(url).value());
    ctx.include_bytes(uri.clone(), bytes);
    Some(uri)
}

impl Cobro {
    fn tope(&self) -> Option<Capa> {
        let capas = [
            (Capa::Cancelar, self.cancelar_abierto),
            (Capa::PagoAfip, self.pago == Some(Lado::Afip)),
            (Capa::Descuento, self.descuento_dialogo.is_some()),
            (Capa::PagoNormal, self.pago == Some(Lado::Normal)),
            (Capa::EfectivoAfip, self.afip.efectivo.is_some()),
            (Capa::EfectivoNormal, self.normal.efectivo.is_some()),
            (Capa::Cancelacion, self.cancelacion.abierto),
            (Capa::Qr, self.qr_abierto),
            (Capa::ManualNormal, self.normal.manual.is_some()),
            (Capa::ManualAfip, self.afip.manual.is_some()),
            (Capa::MixtoNormal, self.normal.mixto_abierto),
            (Capa::MixtoAfip, self.afip.mixto_abierto),
        ];
        capas.iter().rev().find(|(_, abierta)| *abierta).map(|(capa, _)| *capa)
    }

    pub(super) fn dibujar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        let tope = self.tope();
        if self.cancelar_abierto {
            self.dialogo_cancelar(cx, tope == Some(Capa::Cancelar));
        }
        if self.pago == Some(Lado::Afip) {
            self.dialogo_pago(cx, Lado::Afip, tope == Some(Capa::PagoAfip));
        }
        if self.descuento_dialogo.is_some() {
            self.dialogo_descuento(cx, tope == Some(Capa::Descuento));
        }
        if self.pago == Some(Lado::Normal) {
            self.dialogo_pago(cx, Lado::Normal, tope == Some(Capa::PagoNormal));
        }
        for (lado, capa) in [(Lado::Afip, Capa::EfectivoAfip), (Lado::Normal, Capa::EfectivoNormal)] {
            match self.proc_mut(lado).efectivo.as_ref().map(|e| e.exacto) {
                Some(true) => self.dialogo_exacto(cx, lado, tope == Some(capa)),
                Some(false) => self.dialogo_efectivo(cx, lado, tope == Some(capa)),
                None => {}
            }
        }
        if self.cancelacion.abierto {
            self.dialogo_cancelacion(cx, tope == Some(Capa::Cancelacion));
        }
        if self.qr_abierto {
            self.dialogo_qr(cx, tope == Some(Capa::Qr));
        }
        for (lado, capa) in [(Lado::Normal, Capa::ManualNormal), (Lado::Afip, Capa::ManualAfip)] {
            if self.proc_mut(lado).manual.is_some() {
                self.dialogo_manual(cx, lado, tope == Some(capa));
            }
        }
        for (lado, capa) in [(Lado::Normal, Capa::MixtoNormal), (Lado::Afip, Capa::MixtoAfip)] {
            if self.proc_mut(lado).mixto_abierto {
                self.dialogo_mixto(cx, lado, tope == Some(capa));
            }
        }
    }

    fn dialogo_cancelar(&mut self, cx: &mut Contexto, tope: bool) {
        let ctx = cx.egui.clone();
        let cargando = self.normal.procesando;
        let con_foco = !cargando && !self.cancelacion.abierto && self.teclado;
        let mut confirmar = false;
        let mut mantener = tope && !cargando && enter(&ctx);
        let r = ui::Dialogo::new("cobro-cancelar").ancho(448.0).mostrar(&ctx, |ui| {
            encabezado(
                ui,
                "¿Cancelar orden?",
                Tam::Lg,
                18.0,
                color::TEXTO,
                None,
                "¿Estás seguro de que deseas cancelar la orden? Se eliminarán todos los productos del carrito.",
            );
            ui.add_space(16.0);
            ui::pie_dialogo(ui, |ui| {
                let texto = if cargando { "Cancelando..." } else { "Sí, cancelar orden" };
                confirmar = Boton::new(texto).variante(Variante::Cancelar).habilitado(!cargando).ui(ui).clicked();
                let no = Boton::new("No, mantener productos").variante(Variante::Contorno).habilitado(!cargando).ui(ui);
                if con_foco {
                    anillo(ui, no.rect, ui::RADIO as f32);
                }
                mantener |= no.clicked();
            });
        });
        if r.cerrar || mantener {
            self.cancelar_abierto = false;
        } else if confirmar {
            self.cancelar_carrito(cx);
        }
    }

    fn dialogo_pago(&mut self, cx: &mut Contexto, lado: Lado, tope: bool) {
        let ctx = cx.egui.clone();
        let teclado = self.teclado;
        let afip = lado == Lado::Afip;
        let (procesando, metodo) = {
            let p = self.proc_mut(lado);
            (p.procesando, p.metodo)
        };
        let subtotal = cx.global.carritos.total();
        let descuento = self.descuento;
        let mut elegido: Option<&'static str> = None;
        if tope && !procesando && metodo.is_none() {
            for (tecla, m) in [(Key::Num1, "qr"), (Key::Num2, "tarjeta"), (Key::Num3, "efectivo"), (Key::Num4, "split")] {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, tecla)) {
                    elegido = Some(m);
                }
            }
        }
        let habilitado = |m: &str| !procesando && (metodo.is_none() || metodo == Some(m));
        if tope && elegido.is_none() && habilitado("qr") && enter(&ctx) {
            elegido = Some("qr");
        }
        let (titulo_texto, descripcion) = if afip {
            (
                "Seleccionar método de pago - Factura AFIP",
                "Presiona el número correspondiente al método de pago para generar factura AFIP",
            )
        } else {
            (
                "Seleccionar método de pago",
                "Presiona el número correspondiente al método de pago o haz clic en el botón",
            )
        };
        let r = ui::Dialogo::new(("cobro-pago", afip)).ancho(448.0).mostrar(&ctx, |ui| {
            encabezado(ui, titulo_texto, Tam::Lg, 18.0, color::TEXTO, None, descripcion);
            ui.add_space(16.0);
            if let Some(d) = descuento {
                caja_descuento(ui, &d, subtotal);
                ui.add_space(32.0);
            }
            let rect = caja(ui, 272.0);
            let ancho = (rect.width() - 16.0) / 2.0;
            let botones = [
                ("qr", "Transferencia (1)", Icono::Uno("qr-code")),
                ("tarjeta", "Tarjeta (2)", Icono::Uno("credit-card")),
                ("efectivo", "Efectivo (3)", Icono::Uno("wallet")),
                ("split", "Pago Mixto (4)", Icono::Mixto),
            ];
            for (i, (m, etiqueta, icono)) in botones.into_iter().enumerate() {
                let x = rect.left() + (i % 2) as f32 * (ancho + 16.0);
                let y = rect.top() + (i / 2) as f32 * 144.0;
                let caja_boton = Rect::from_min_size(egui::pos2(x, y), Vec2::new(ancho, 128.0));
                let clic = boton_metodo(
                    ui,
                    caja_boton,
                    &format!("{afip}-{m}"),
                    etiqueta,
                    icono,
                    metodo == Some(m),
                    procesando && metodo == Some(m),
                    habilitado(m),
                    i == 0 && teclado,
                );
                if clic {
                    elegido = Some(m);
                }
            }
        });
        if r.cerrar {
            self.pago = None;
            self.proc_mut(lado).reiniciar();
            return;
        }
        if let Some(m) = elegido {
            if afip {
                self.elegir_metodo_afip(cx, m);
            } else {
                self.elegir_metodo(cx, m);
            }
        }
    }

    fn dialogo_descuento(&mut self, cx: &mut Contexto, tope: bool) {
        let ctx = cx.egui.clone();
        let subtotal = cx.global.carritos.total();
        let Some(dialogo) = self.descuento_dialogo.as_mut() else { return };
        let escape = tope && ctx.input(|i| i.key_pressed(Key::Escape));
        let mut confirmar = false;
        let mut cancelar = escape;
        let r = ui::Dialogo::new("cobro-descuento").ancho(448.0).mostrar(&ctx, |ui| {
            encabezado(
                ui,
                "Aplicar Descuento",
                Tam::Lg,
                18.0,
                color::TEXTO,
                Some("percent"),
                "Selecciona el tipo de descuento y el valor a aplicar",
            );
            ui.add_space(16.0);
            let anterior = dialogo.cantidad_fija;
            pestanas_descuento(ui, &mut dialogo.cantidad_fija);
            if anterior && !dialogo.cantidad_fija {
                dialogo.enfocar = true;
            }
            ui.add_space(8.0);
            let etiqueta = if dialogo.cantidad_fija { "Cantidad fija de descuento" } else { "Porcentaje de descuento (1-100%)" };
            let rect = caja(ui, 24.0);
            pintar(ui, rect.left_center(), Align2::LEFT_CENTER, etiqueta, Tam::Sm, Peso::Medio, color::TEXTO);
            ui.add_space(8.0);
            let maximo = format!("Máximo: ${}", fijo(subtotal, 2));
            let (texto, sugerencia, id) = if dialogo.cantidad_fija {
                (&mut dialogo.monto, maximo.as_str(), "monto")
            } else {
                (&mut dialogo.porcentaje, "Ej: 10", "porcentaje")
            };
            let mut campo = Campo::new(("descuento", id), &mut *texto, sugerencia);
            campo.alineacion = Align::Center;
            let respuesta = campo.ui(ui, &mut dialogo.enfocar);
            filtrar_numero(texto);
            mantener_foco(&ctx, &respuesta);
            ui.add_space(16.0);
            let valor = if dialogo.cantidad_fija { &dialogo.monto } else { &dialogo.porcentaje };
            let calculado = cobros::descuento_calculado(subtotal, !dialogo.cantidad_fija, valor);
            ui::marco().fill(paleta::GRAY_50).corner_radius(ui::RADIO).inner_margin(egui::Margin::same(24)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let sub = format!("${}", fijo(subtotal, 2));
                fila(ui, 28.0, ("Subtotal original:", Tam::Lg, Peso::Medio, color::TEXTO), (&sub, Tam::Xl, Peso::Negrita, color::TEXTO));
                ui.add_space(12.0);
                let menos = format!("-${}", fijo(calculado, 2));
                fila(
                    ui,
                    28.0,
                    ("Descuento aplicado:", Tam::Lg, Peso::Medio, paleta::GREEN_600),
                    (&menos, Tam::Xl, Peso::Negrita, paleta::GREEN_600),
                );
                ui.add_space(12.0);
                let linea = caja(ui, 1.0);
                ui.painter().rect_filled(linea, 0.0, paleta::GRAY_300);
                ui.add_space(12.0);
                let total = format!("${}", fijo(subtotal - calculado, 2));
                fila(ui, 36.0, ("Total final:", Tam::X2, Peso::Negrita, color::TEXTO), (&total, Tam::X3, Peso::Negrita, paleta::BLUE_600));
            });
            ui.add_space(16.0);
            let valido = calculado > 0.0;
            if tope && valido && enter(&ctx) {
                confirmar = true;
            }
            let rect = caja(ui, 40.0);
            let ancho = (rect.width() - 16.0) / 2.0;
            let mut izquierda = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(rect.min, Vec2::new(ancho, 40.0))));
            cancelar |= Boton::new("Cancelar").variante(Variante::Contorno).ancho(ancho).ui(&mut izquierda).clicked();
            let mut derecha =
                ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(egui::pos2(rect.left() + ancho + 16.0, rect.top()), Vec2::new(ancho, 40.0))));
            let fondo = if derecha.rect_contains_pointer(derecha.max_rect()) && valido { paleta::GREEN_700 } else { paleta::GREEN_600 };
            confirmar |= Boton::new("Aplicar Descuento").colores(fondo, Color32::WHITE).ancho(ancho).habilitado(valido).ui(&mut derecha).clicked();
            calculado
        });
        let calculado = r.interior;
        if cancelar {
            self.descuento_dialogo = None;
            self.accion_descuento = None;
            self.descuento = None;
        } else if confirmar && calculado > 0.0 {
            let Some(dialogo) = self.descuento_dialogo.as_ref() else { return };
            let descuento = if dialogo.cantidad_fija {
                Descuento::fijo(cobros::parse_float(&dialogo.monto).unwrap_or(f64::NAN), calculado)
            } else {
                Descuento::porcentaje(cobros::parse_float(&dialogo.porcentaje).unwrap_or(f64::NAN), calculado)
            };
            self.confirmar_descuento(descuento);
        } else if r.cerrar {
            self.descuento_dialogo = None;
        }
    }

    fn dialogo_efectivo(&mut self, cx: &mut Contexto, lado: Lado, tope: bool) {
        let ctx = cx.egui.clone();
        let teclado = self.teclado;
        let bloquea = self.bloqueado();
        let p = self.proc_mut(lado);
        let cargando = p.procesando && bloquea;
        let Some(e) = p.efectivo.as_ref() else { return };
        let (original, cobrar, aplicando) = (e.original, e.cobrar, e.aplicando);
        let con_descuento = cobrar < original;
        let mut confirmar = tope && !cargando && enter(&ctx);
        let mut aplicar_d = false;
        let mut cancelar = false;
        let r = ui::Dialogo::new(("cobro-efectivo", lado == Lado::Afip)).ancho(512.0).mostrar(&ctx, |ui| {
            let descripcion = if con_descuento { "Se ha aplicado un descuento por pago en efectivo" } else { "Confirma el monto a cobrar" };
            encabezado(ui, "Confirmar pago en efectivo", Tam::Lg, 18.0, color::TEXTO, None, descripcion);
            ui.add_space(32.0);
            if con_descuento {
                let tachado = format!("${}", local(original));
                let (_, d) = fila(ui, 24.0, ("Monto original:", Tam::Base, Peso::Normal, color::TEXTO), (&tachado, Tam::Base, Peso::Normal, color::TEXTO));
                tachar(ui, d, color::TEXTO);
                ui.add_space(8.0);
                let nuevo = format!("${}", local(cobrar));
                fila(
                    ui,
                    28.0,
                    ("Nuevo monto con descuento:", Tam::Lg, Peso::Medio, color::TEXTO),
                    (&nuevo, Tam::Lg, Peso::Medio, paleta::EMERALD_600),
                );
                ui.add_space(8.0);
                let rect = caja(ui, 20.0);
                let texto = format!("Descuento aplicado: ${}", local(original - cobrar));
                pintar(ui, rect.left_center(), Align2::LEFT_CENTER, &texto, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
            } else {
                let monto = format!("${}", local(cobrar));
                fila(ui, 36.0, ("Monto a cobrar:", Tam::Lg, Peso::Medio, color::TEXTO), (&monto, Tam::X3, Peso::Negrita, paleta::EMERALD_600));
            }
            ui.add_space(32.0);
            ui::pie_dialogo(ui, |ui| {
                let b = boton_confirmar(ui, cargando, !cargando);
                if !cargando && teclado {
                    anillo(ui, b.rect, ui::RADIO as f32);
                }
                confirmar |= b.clicked();
                if !aplicando {
                    aplicar_d = Boton::new("Aplicar D (F5)")
                        .variante(Variante::Contorno)
                        .colores(paleta::BLUE_50, color::TEXTO)
                        .color_borde(paleta::BLUE_200)
                        .texto_tam(Tam::Base)
                        .habilitado(!cargando)
                        .ui(ui)
                        .clicked();
                }
                cancelar = Boton::new("Cancelar").variante(Variante::Contorno).texto_tam(Tam::Base).habilitado(!cargando).ui(ui).clicked();
            });
        });
        if r.cerrar || cancelar {
            self.cerrar_efectivo(lado);
        } else if aplicar_d {
            self.aplicar_descuento_efectivo(cx, lado);
        } else if confirmar && !cargando {
            self.confirmar_efectivo(cx, lado);
        }
    }

    fn dialogo_exacto(&mut self, cx: &mut Contexto, lado: Lado, tope: bool) {
        let ctx = cx.egui.clone();
        let subtotal = cx.global.carritos.total();
        let descuento = self.descuento;
        let bloquea = self.bloqueado();
        let p = self.proc_mut(lado);
        let cargando = p.procesando && bloquea;
        let Some(e) = p.efectivo.as_mut() else { return };
        let total = e.cobrar;
        let mut confirmar = false;
        let mut cancelar = false;
        let r = ui::Dialogo::new(("cobro-exacto", lado == Lado::Afip)).ancho(512.0).mostrar(&ctx, |ui| {
            encabezado(
                ui,
                "Pago exacto - Calcular vuelto",
                Tam::Lg,
                18.0,
                color::TEXTO,
                Some("calculator"),
                "Ingresa con cuánto dinero paga el cliente para calcular el vuelto",
            );
            ui.add_space(32.0);
            if let (Some(d), true) = (descuento, subtotal != 0.0) {
                let sub = format!("${}", local(subtotal));
                fila(ui, 20.0, ("Subtotal:", Tam::Sm, Peso::Normal, paleta::GRAY_600), (&sub, Tam::Sm, Peso::Normal, color::TEXTO));
                ui.add_space(8.0);
                let etiqueta = format!("Descuento ({}):", d.etiqueta());
                let menos = format!("-${}", local(d.monto));
                fila(ui, 20.0, (&etiqueta, Tam::Sm, Peso::Normal, paleta::GRAY_600), (&menos, Tam::Sm, Peso::Normal, paleta::GREEN_600));
                ui.add_space(8.0);
                let linea = caja(ui, 1.0);
                ui.painter().rect_filled(linea, 0.0, paleta::GRAY_200);
                ui.add_space(24.0);
            }
            recuadro(ui, paleta::EMERALD_50, paleta::EMERALD_200, 16.0, |ui| {
                let monto = format!("${}", local(total));
                fila(ui, 32.0, ("Total a pagar:", Tam::Lg, Peso::Medio, paleta::EMERALD_700), (&monto, Tam::X2, Peso::Negrita, paleta::EMERALD_600));
            });
            ui.add_space(24.0);
            let rect = caja(ui, 20.0);
            ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0)), "dollar-sign", color::TEXTO);
            pintar(ui, egui::pos2(rect.left() + 24.0, rect.center().y), Align2::LEFT_CENTER, "Con cuánto paga el cliente:", Tam::Sm, Peso::Medio, color::TEXTO);
            ui.add_space(8.0);
            let mut campo = Campo::new(("exacto", lado == Lado::Afip), &mut e.pagado, "Ej: 5000");
            campo.alineacion = Align::Center;
            campo.habilitado = !cargando;
            let respuesta = campo.ui(ui, &mut e.enfocar);
            filtrar_numero(&mut e.pagado);
            mantener_foco(&ctx, &respuesta);
            if e.pagado != e.anterior {
                e.anterior = e.pagado.clone();
                e.error = cobros::vuelto(&e.pagado, total).1;
            }
            let cambio = cobros::vuelto(&e.pagado, total).0;
            let error = e.error;
            if error.is_some() {
                let borde = Rect::from_x_y_ranges(ui.min_rect().x_range(), respuesta.rect.y_range());
                ui.painter().rect_stroke(borde, ui::RADIO_XL, Stroke::new(1.0, paleta::RED_500), StrokeKind::Inside);
            }
            if let Some(texto) = error {
                ui.add_space(8.0);
                let rect = caja(ui, 20.0);
                pintar(ui, rect.left_center(), Align2::LEFT_CENTER, texto, Tam::Sm, Peso::Normal, paleta::RED_600);
            }
            if !e.pagado.is_empty() && error.is_none() && cambio >= 0.0 {
                ui.add_space(24.0);
                recuadro(ui, paleta::BLUE_50, paleta::BLUE_200, 16.0, |ui| {
                    let monto = format!("${}", fijo(cambio, 2));
                    fila(ui, 32.0, ("Vuelto a entregar:", Tam::Lg, Peso::Medio, paleta::BLUE_700), (&monto, Tam::X2, Peso::Negrita, paleta::BLUE_600));
                    if cambio == 0.0 {
                        ui.add_space(8.0);
                        let rect = caja(ui, 20.0);
                        pintar(ui, rect.left_center(), Align2::LEFT_CENTER, "✅ Pago exacto - No hay vuelto", Tam::Sm, Peso::Normal, paleta::BLUE_600);
                    }
                });
            }
            ui.add_space(32.0);
            let habilitado = !cargando && error.is_none() && !e.pagado.is_empty();
            if tope && habilitado && enter(&ctx) {
                confirmar = true;
            }
            ui::pie_dialogo(ui, |ui| {
                confirmar |= boton_confirmar(ui, cargando, habilitado).clicked();
                cancelar = Boton::new("Cancelar").variante(Variante::Contorno).texto_tam(Tam::Base).habilitado(!cargando).ui(ui).clicked();
            });
        });
        if r.cerrar || cancelar {
            self.cerrar_efectivo(lado);
            return;
        }
        if confirmar {
            let Some(e) = self.proc_mut(lado).efectivo.as_mut() else { return };
            match cobros::validar_pago_exacto(&e.pagado, e.cobrar) {
                Ok(_) => self.confirmar_efectivo(cx, lado),
                Err(error) => e.error = Some(error),
            }
        }
    }

    fn dialogo_mixto(&mut self, cx: &mut Contexto, lado: Lado, tope: bool) {
        let ctx = cx.egui.clone();
        let afip = lado == Lado::Afip;
        let total = self.total_mixto(cx.global.carritos.total());
        let descuento = self.descuento;
        let p = self.proc_mut(lado);
        let procesando = p.procesando;
        let mut completar = false;
        let mut cancelar = false;
        let r = ui::Dialogo::new(("cobro-mixto", afip)).ancho(500.0).mostrar(&ctx, |ui| {
            let (titulo_texto, descripcion) = if afip {
                (
                    "Pago mixto - Factura AFIP",
                    "Ingresa el monto a pagar con efectivo y selecciona el método para el resto. Se generará una factura AFIP.",
                )
            } else {
                ("Pago mixto", "Ingresa el monto a pagar con efectivo y selecciona el método para el resto")
            };
            encabezado(ui, titulo_texto, Tam::Xl, 28.0, color::TEXTO, None, descripcion);
            ui.add_space(32.0);
            if let Some(d) = descuento {
                recuadro(ui, paleta::GREEN_50, paleta::GREEN_200, 16.0, |ui| {
                    let etiqueta = format!("Descuento ({})", d.etiqueta());
                    let menos = format!("-${}", local(d.monto));
                    fila(ui, 20.0, (&etiqueta, Tam::Sm, Peso::Medio, paleta::GREEN_800), (&menos, Tam::Sm, Peso::Negrita, paleta::GREEN_800));
                    ui.add_space(8.0);
                });
                ui.add_space(24.0);
            }
            let rect = caja(ui, 28.0);
            pintar(ui, rect.center(), Align2::CENTER_CENTER, "Total a pagar", Tam::Lg, Peso::Normal, color::APAGADO_TEXTO);
            ui.add_space(4.0);
            let rect = caja(ui, 36.0);
            pintar(ui, rect.center(), Align2::CENTER_CENTER, &format!("${}", local(total)), Tam::X3, Peso::Negrita, paleta::EMERALD_600);
            ui.add_space(24.0);
            let fila_efectivo = caja(ui, 40.0);
            ui::pintar_icono_en(
                ui,
                Rect::from_min_size(egui::pos2(fila_efectivo.left(), fila_efectivo.center().y - 8.0), Vec2::splat(16.0)),
                "wallet",
                color::TEXTO,
            );
            pintar(ui, egui::pos2(fila_efectivo.left() + 24.0, fila_efectivo.center().y), Align2::LEFT_CENTER, "Efectivo", Tam::Sm, Peso::Medio, color::TEXTO);
            let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(egui::pos2(fila_efectivo.right() - 160.0, fila_efectivo.top()), Vec2::new(160.0, 40.0))));
            let mut campo = Campo::new(("mixto", afip), &mut p.efectivo_mixto, "Monto en efectivo");
            campo.alineacion = Align::Max;
            campo.habilitado = !procesando;
            let respuesta = campo.ui(&mut hijo, &mut p.enfocar_mixto);
            filtrar_numero(&mut p.efectivo_mixto);
            mantener_foco(&ctx, &respuesta);
            ui.add_space(16.0);
            let rect = caja(ui, 24.0);
            pintar(ui, rect.left_center(), Align2::LEFT_CENTER, "Segundo método", Tam::Sm, Peso::Medio, color::TEXTO);
            ui.add_space(8.0);
            let rect = caja(ui, 40.0);
            let ancho = (rect.width() - 12.0) / 2.0;
            for (i, (m, texto, icono)) in [("tarjeta", "Tarjeta", "credit-card"), ("qr", "Transferencia", "qr-code")].into_iter().enumerate() {
                let caja_boton = Rect::from_min_size(egui::pos2(rect.left() + i as f32 * (ancho + 12.0), rect.top()), Vec2::new(ancho, 40.0));
                let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(caja_boton));
                let activo = p.segundo == m;
                let boton = Boton::new(texto).icono(icono).ancho(ancho).habilitado(!procesando);
                let boton = if activo {
                    let fondo = if hijo.rect_contains_pointer(caja_boton) { paleta::EMERALD_700 } else { paleta::EMERALD_600 };
                    boton.colores(fondo, Color32::WHITE)
                } else {
                    boton.variante(Variante::Contorno)
                };
                if boton.ui(&mut hijo).clicked() {
                    p.segundo = m;
                }
            }
            ui.add_space(16.0);
            let (icono, texto) = if p.segundo == "tarjeta" { ("credit-card", "Tarjeta") } else { ("qr-code", "Transferencia") };
            let restante = format!("${}", local(cobros::mixto_restante(total, &p.efectivo_mixto)));
            if afip {
                let rect = caja(ui, 20.0);
                ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0)), icono, color::TEXTO);
                pintar(ui, egui::pos2(rect.left() + 24.0, rect.center().y), Align2::LEFT_CENTER, texto, Tam::Sm, Peso::Medio, color::TEXTO);
                ui.add_space(8.0);
                let rect = caja(ui, 28.0);
                pintar(ui, rect.left_center(), Align2::LEFT_CENTER, &restante, Tam::Xl, Peso::Semi, paleta::EMERALD_600);
            } else {
                ui.add_space(8.0);
                let rect = caja(ui, 28.0);
                ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0)), icono, color::TEXTO);
                pintar(ui, egui::pos2(rect.left() + 24.0, rect.center().y), Align2::LEFT_CENTER, texto, Tam::Sm, Peso::Medio, color::TEXTO);
                pintar(ui, rect.right_center(), Align2::RIGHT_CENTER, &restante, Tam::Xl, Peso::Semi, paleta::EMERALD_600);
            }
            ui.add_space(32.0);
            let habilitado = cobros::mixto_habilitado(total, &p.efectivo_mixto, procesando);
            if tope && habilitado && enter(&ctx) {
                completar = true;
            }
            ui::pie_dialogo(ui, |ui| {
                let texto = if afip { "Completar pago con AFIP" } else { "Completar pago" };
                let boton = if procesando { Boton::new("Procesando...").cargando(true) } else { Boton::new(texto) };
                completar |= boton.variante(Variante::Esmeralda).habilitado(habilitado).ui(ui).clicked();
                cancelar = Boton::new("Cancelar").variante(Variante::Contorno).habilitado(!procesando).ui(ui).clicked();
            });
        });
        if r.cerrar || cancelar {
            self.cerrar_mixto(lado);
        } else if completar {
            self.completar_mixto(cx, lado);
        }
    }

    fn dialogo_qr(&mut self, cx: &mut Contexto, tope: bool) {
        let ctx = cx.egui.clone();
        let teclado = self.teclado;
        let afip = self.flujo_afip;
        let fuente_qr = if afip { &self.afip } else { &self.normal };
        let qr = fuente_qr.qr.clone();
        let estado = fuente_qr.estado.clone();
        let descuento = self.descuento;
        let subtotal = cx.global.carritos.total();
        let uri = qr.as_ref().and_then(|q| q["qrImageUrl"].as_str()).and_then(|url| self.uri_qr(cx, url));
        let pendiente = estado.as_deref() == Some("PENDIENTE");
        let mut manual = tope && pendiente && qr.is_some() && enter(&ctx);
        let mut cancelar = false;
        let r = ui::Dialogo::new("cobro-qr").ancho(448.0).mostrar(&ctx, |ui| {
            let (titulo_texto, descripcion) = if afip {
                (
                    "Escanea el código QR para pagar y generar factura AFIP",
                    "Usa la app de Mercado Pago para escanear. La factura AFIP se generará automáticamente después del pago.",
                )
            } else {
                ("Escanea el código QR para pagar", "Usa la app de Mercado Pago para escanear")
            };
            encabezado(ui, titulo_texto, Tam::Xl, 28.0, color::TEXTO, None, descripcion);
            ui.add_space(32.0);
            match &qr {
                Some(datos) => {
                    let aviso = "No cierres esta ventana hasta que el pago sea completado";
                    let texto_boton = if afip { "Completar manualmente y generar factura" } else { "Completar manualmente" };
                    let ancho_columna = 256f32
                        .max(medir(ui, aviso, Tam::Sm, Peso::Normal))
                        .max(medir(ui, texto_boton, Tam::Sm, Peso::Medio) + 56.0)
                        .min(ui.available_width());
                    let columna = Rect::from_min_size(
                        egui::pos2(ui.cursor().left() + (ui.available_width() - ancho_columna) / 2.0, ui.cursor().top()),
                        Vec2::new(ancho_columna, f32::INFINITY),
                    );
                    ui.scope_builder(egui::UiBuilder::new().max_rect(columna), |ui| {
                        let rect = caja(ui, 256.0);
                        let marco_qr = Rect::from_center_size(rect.center(), Vec2::splat(256.0));
                        ui.painter().rect_stroke(marco_qr, 0.0, Stroke::new(1.0, paleta::GRAY_200), StrokeKind::Inside);
                        if let Some(uri) = &uri {
                            egui::Image::new(uri.clone()).paint_at(ui, marco_qr.shrink(9.0));
                        }
                        ui.add_space(16.0);
                        if verdad_json(&datos["isSplitPayment"]) {
                            let rect = caja(ui, 20.0);
                            pintar(ui, rect.center(), Align2::CENTER_CENTER, "Pago mixto:", Tam::Sm, Peso::Medio, color::APAGADO_TEXTO);
                            ui.add_space(4.0);
                            let efectivo = format!("${}", local(cobros::numero(&datos["cashAmount"])));
                            let rect = caja(ui, 24.0);
                            ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0)), "wallet", color::TEXTO);
                            pintar(ui, egui::pos2(rect.left() + 20.0, rect.center().y), Align2::LEFT_CENTER, "Efectivo:", Tam::Base, Peso::Medio, color::TEXTO);
                            pintar(ui, rect.right_center(), Align2::RIGHT_CENTER, &efectivo, Tam::Base, Peso::Medio, color::TEXTO);
                            ui.add_space(4.0);
                            let monto = format!("${}", local(cobros::numero(&datos["monto"])));
                            let rect = caja(ui, 28.0);
                            ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0)), "qr-code", color::TEXTO);
                            pintar(ui, egui::pos2(rect.left() + 20.0, rect.center().y), Align2::LEFT_CENTER, "QR:", Tam::Lg, Peso::Semi, color::TEXTO);
                            pintar(ui, rect.right_center(), Align2::RIGHT_CENTER, &monto, Tam::Lg, Peso::Semi, color::TEXTO);
                        } else if let Some(d) = descuento {
                            let sub = format!("${}", local(subtotal));
                            fila(ui, 24.0, ("Subtotal:", Tam::Base, Peso::Medio, color::TEXTO), (&sub, Tam::Base, Peso::Medio, color::TEXTO));
                            ui.add_space(8.0);
                            let etiqueta = format!("Descuento ({}):", d.etiqueta());
                            let menos = format!("-${}", local(d.monto));
                            fila(ui, 24.0, (&etiqueta, Tam::Base, Peso::Medio, paleta::GREEN_600), (&menos, Tam::Base, Peso::Medio, paleta::GREEN_600));
                            ui.add_space(8.0);
                            let linea = caja(ui, 1.0);
                            ui.painter().rect_filled(linea, 0.0, color::BORDE);
                            ui.add_space(8.0);
                            let total = format!("${}", local(subtotal - d.monto));
                            fila(ui, 28.0, ("Total:", Tam::Lg, Peso::Semi, color::TEXTO), (&total, Tam::Lg, Peso::Semi, color::TEXTO));
                        } else {
                            let rect = caja(ui, 28.0);
                            let texto = format!("Monto a pagar: ${}", local(cobros::numero(&datos["monto"])));
                            pintar(ui, rect.center(), Align2::CENTER_CENTER, &texto, Tam::Base, Peso::Medio, color::TEXTO);
                        }
                        ui.add_space(16.0);
                        pastilla_estado(ui, estado.as_deref());
                        ui.add_space(8.0);
                        let rect = caja(ui, 20.0);
                        pintar(ui, rect.center(), Align2::CENTER_CENTER, aviso, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
                        if pendiente {
                            ui.add_space(8.0);
                            let b = Boton::new(texto_boton)
                                .variante(Variante::Contorno)
                                .icono("receipt")
                                .colores(paleta::EMERALD_50, color::TEXTO)
                                .color_borde(paleta::EMERALD_200)
                                .lleno(ui)
                                .ui(ui);
                            if tope && teclado {
                                anillo(ui, b.rect, ui::RADIO as f32);
                            }
                            manual |= b.clicked();
                        }
                    });
                }
                None => {
                    let rect = caja(ui, 256.0);
                    girador(ui, rect.center() - Vec2::new(0.0, 22.0), 24.0, paleta::GRAY_900);
                    pintar(ui, rect.center() + Vec2::new(0.0, 24.0), Align2::CENTER_CENTER, "Generando código QR...", Tam::Base, Peso::Normal, color::TEXTO);
                }
            }
            ui.add_space(32.0);
            cancelar = Boton::new("Cancelar").variante(Variante::Contorno).lleno(ui).ui(ui).clicked();
        });
        if r.cerrar {
            self.cerrar_qr_usuario();
        } else if cancelar {
            self.cancelar_qr();
        } else if manual && pendiente {
            self.abrir_manual(cx);
        }
    }

    fn uri_qr(&mut self, cx: &mut Contexto, url: &str) -> Option<String> {
        if let Some((guardada, uri)) = &self.imagen_qr {
            if guardada == url {
                return uri.clone();
            }
        }
        if url.starts_with("http") {
            let api = cx.servicios.api.clone();
            let url_owned = url.to_string();
            self.imagen_qr = Some((url.to_string(), None));
            self.lanzar(cx, false, move || {
                let bytes = api.bytes(&url_owned).ok().filter(|(estado, _)| (200..300).contains(estado)).map(|(_, b)| b);
                super::aplicar(move |c, cx| {
                    if let Some(bytes) = bytes {
                        let uri = format!("bytes://cobro-qr/{}.png", Id::new(&url_owned).value());
                        cx.egui.include_bytes(uri.clone(), bytes);
                        c.imagen_qr = Some((url_owned, Some(uri)));
                    }
                })
            });
            return None;
        }
        let uri = imagen_de_url(&cx.egui, url);
        self.imagen_qr = Some((url.to_string(), uri.clone()));
        uri
    }

    fn dialogo_manual(&mut self, cx: &mut Contexto, lado: Lado, tope: bool) {
        let ctx = cx.egui.clone();
        let p = self.proc_mut(lado);
        let Some(manual) = p.manual.as_mut() else { return };
        let enviando = manual.enviando;
        let mut enviar = false;
        let mut cancelar = false;
        let r = ui::Dialogo::new(("cobro-manual", lado == Lado::Afip)).ancho(448.0).mostrar(&ctx, |ui| {
            encabezado(
                ui,
                "Contraseña para Completar Manualmente",
                Tam::Lg,
                18.0,
                color::TEXTO,
                None,
                "Ingresa la contraseña para completar la orden manualmente.",
            );
            ui.add_space(32.0);
            let mut campo = Campo::new(("manual", lado == Lado::Afip), &mut manual.clave, "Contraseña");
            campo.numerico = false;
            campo.clave = true;
            campo.habilitado = !enviando;
            let respuesta = campo.ui(ui, &mut manual.enfocar);
            if tope && respuesta.lost_focus() && enter(&ctx) {
                enviar = true;
                respuesta.request_focus();
            }
            ui.add_space(32.0);
            ui::pie_dialogo(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                let boton = if enviando { Boton::new("Procesando...").cargando(true) } else { Boton::new("Confirmar") };
                enviar |= boton.colores(paleta::EMERALD_600, Color32::WHITE).habilitado(!enviando).ui(ui).clicked();
                cancelar = Boton::new("Cancelar").variante(Variante::Contorno).habilitado(!enviando).ui(ui).clicked();
            });
        });
        if r.cerrar || cancelar {
            self.proc_mut(lado).manual = None;
        } else if enviar && !enviando {
            self.enviar_manual(cx, lado);
        }
    }

    fn dialogo_cancelacion(&mut self, cx: &mut Contexto, tope: bool) {
        let ctx = cx.egui.clone();
        let escape = tope && ctx.input(|i| i.key_pressed(Key::Escape));
        let ctrl_enter = tope && ctx.input(|i| i.key_pressed(Key::Enter) && i.modifiers.command);
        let c = &mut self.cancelacion;
        let enviando = c.enviando;
        let mut confirmar = ctrl_enter && !enviando;
        let mut mantener = escape;
        let r = ui::Dialogo::new("cobro-cancelacion").ancho(448.0).mostrar(&ctx, |ui| {
            let descripcion = c
                .pendiente
                .as_ref()
                .map(|p| format!("¿Estás seguro de que deseas cancelar la orden #{}? Esta acción quedará registrada en el sistema.", p.referencia))
                .unwrap_or_default();
            let (titulo_texto, icono) = if c.pendiente.is_some() { ("Cancelar Orden", "file-text") } else { ("Cancelar", "alert-triangle") };
            encabezado(ui, titulo_texto, Tam::Lg, 18.0, paleta::RED_600, Some(icono), &descripcion);
            ui.add_space(32.0);
            let rect = caja(ui, 24.0);
            pintar(ui, rect.left_center(), Align2::LEFT_CENTER, "Motivo de la cancelación *", Tam::Sm, Peso::Medio, color::TEXTO);
            ui.add_space(8.0);
            let mut campo = Campo::new("cancelacion", &mut c.motivo, "Ej: Cliente cambió de opinión, error en el pedido, producto no disponible...");
            campo.numerico = false;
            campo.multilinea = true;
            campo.alto = 100.0;
            campo.habilitado = !enviando;
            campo.ui(ui, &mut c.enfocar);
            if c.motivo.chars().count() > 500 {
                c.motivo = c.motivo.chars().take(500).collect();
            }
            ui.add_space(8.0);
            let contador = format!("{}/500", c.motivo.chars().count());
            fila(ui, 16.0, ("Mínimo 3 caracteres", Tam::Xs, Peso::Normal, color::APAGADO_TEXTO), (&contador, Tam::Xs, Peso::Normal, color::APAGADO_TEXTO));
            if !c.error.is_empty() {
                ui.add_space(8.0);
                let rect = caja(ui, 20.0);
                pintar(ui, rect.left_center(), Align2::LEFT_CENTER, &c.error, Tam::Sm, Peso::Normal, paleta::RED_600);
            }
            if let Some(p) = c.pendiente.as_ref().filter(|p| !p.productos.is_empty()) {
                ui.add_space(16.0);
                let rect = caja(ui, 24.0);
                let etiqueta = format!("Productos a cancelar ({})", p.productos.len());
                pintar(ui, rect.left_center(), Align2::LEFT_CENTER, &etiqueta, Tam::Sm, Peso::Medio, paleta::GRAY_700);
                ui.add_space(8.0);
                ui::marco()
                    .fill(paleta::GRAY_50)
                    .stroke(Stroke::new(1.0, color::BORDE))
                    .corner_radius(ui::RADIO_MD)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().scroll.fade.strength = 0.0;
                        egui::ScrollArea::vertical().max_height(110.0).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for producto in &p.productos {
                                let derecha = format!(
                                    "{} x ${} = ${}",
                                    numero_js(cobros::numero(&producto["cantidad"])),
                                    fijo(cobros::numero(&producto["precioUnitario"]), 2),
                                    fijo(cobros::numero(&producto["subtotal"]), 2)
                                );
                                let nombre = producto["nombreProducto"].as_str().unwrap_or("");
                                fila(ui, 28.0, (nombre, Tam::Sm, Peso::Medio, color::TEXTO), (&derecha, Tam::Sm, Peso::Normal, paleta::GRAY_600));
                            }
                            ui.add_space(8.0);
                            let linea = caja(ui, 1.0);
                            ui.painter().rect_filled(linea, 0.0, color::BORDE);
                            ui.add_space(8.0);
                            let total = format!("${}", fijo(p.monto, 2));
                            fila(ui, 24.0, ("Total a cancelar:", Tam::Base, Peso::Semi, color::TEXTO), (&total, Tam::Base, Peso::Semi, paleta::RED_600));
                        });
                    });
            }
            ui.add_space(32.0);
            let valido = cobros::validar_motivo(&c.motivo).is_ok();
            ui::pie_dialogo(ui, |ui| {
                let boton = if enviando { Boton::new("Registrando...").cargando(true) } else { Boton::new("Sí, cancelar y registrar") };
                confirmar |= boton.colores(paleta::RED_600, Color32::WHITE).habilitado(!enviando && valido).ui(ui).clicked();
                mantener |= Boton::new("No, mantener").variante(Variante::Contorno).habilitado(!enviando).ui(ui).clicked();
            });
        });
        if mantener {
            self.cancelacion.abierto = false;
            self.cancelacion.motivo.clear();
            self.cancelacion.pendiente = None;
        } else if r.cerrar {
            self.cancelacion.abierto = false;
        } else if confirmar {
            self.confirmar_cancelacion(cx);
        }
    }
}

fn verdad_json(v: &Value) -> bool {
    cobros::verdad(v)
}

fn caja_descuento(ui: &mut Ui, d: &Descuento, subtotal: f64) {
    recuadro(ui, paleta::GREEN_50, paleta::GREEN_200, 16.0, |ui| {
        let rect = caja(ui, 64.0);
        let texto_valor = match d.tipo {
            cobros::TipoDescuento::Porcentaje => format!("{}% de descuento", numero_js(d.valor)),
            cobros::TipoDescuento::Fijo => format!("${} de descuento", fijo(d.valor, 2)),
        };
        pintar(ui, egui::pos2(rect.left(), rect.center().y - 10.0), Align2::LEFT_CENTER, "Descuento Aplicado", Tam::Base, Peso::Semi, paleta::GREEN_800);
        pintar(ui, egui::pos2(rect.left(), rect.center().y + 12.0), Align2::LEFT_CENTER, &texto_valor, Tam::Sm, Peso::Normal, paleta::GREEN_600);
        let x = rect.right();
        pintar(ui, egui::pos2(x, rect.top() + 10.0), Align2::RIGHT_CENTER, &format!("Subtotal: ${}", fijo(subtotal, 2)), Tam::Sm, Peso::Normal, paleta::GRAY_600);
        pintar(ui, egui::pos2(x, rect.top() + 30.0), Align2::RIGHT_CENTER, &format!("Descuento: -${}", fijo(d.monto, 2)), Tam::Sm, Peso::Semi, paleta::GREEN_600);
        pintar(ui, egui::pos2(x, rect.top() + 52.0), Align2::RIGHT_CENTER, &format!("Total: ${}", fijo(subtotal - d.monto, 2)), Tam::Base, Peso::Negrita, paleta::GREEN_800);
    });
}

fn pestanas_descuento(ui: &mut Ui, cantidad_fija: &mut bool) {
    let rect = caja(ui, 40.0);
    ui.painter().rect_filled(rect, ui::RADIO_MD, color::APAGADO);
    let interior = rect.shrink(4.0);
    let ancho = interior.width() / 2.0;
    for (i, (texto, icono)) in [("Porcentaje", "percent"), ("Cantidad Fija", "dollar-sign")].into_iter().enumerate() {
        let caja_pestana = Rect::from_min_size(egui::pos2(interior.left() + i as f32 * ancho, interior.top()), Vec2::new(ancho, interior.height()));
        let respuesta = ui.interact(caja_pestana, Id::new(("cobro-pestana", i)), Sense::click());
        if respuesta.clicked() {
            *cantidad_fija = i == 1;
        }
        let activa = *cantidad_fija == (i == 1);
        if activa {
            ui.painter().add(ui::sombra_sm().as_shape(caja_pestana, ui::RADIO_SM));
            ui.painter().rect_filled(caja_pestana, ui::RADIO_SM, color::FONDO);
        }
        let tinta = if activa { color::TEXTO } else { color::APAGADO_TEXTO };
        let ancho_texto = medir(ui, texto, Tam::Sm, Peso::Medio);
        let total = 16.0 + 8.0 + ancho_texto;
        let x = caja_pestana.center().x - total / 2.0;
        ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(x, caja_pestana.center().y - 8.0), Vec2::splat(16.0)), icono, tinta);
        pintar(ui, egui::pos2(x + 24.0, caja_pestana.center().y), Align2::LEFT_CENTER, texto, Tam::Sm, Peso::Medio, tinta);
        respuesta.on_hover_cursor(CursorIcon::PointingHand);
    }
}

fn pastilla_estado(ui: &mut Ui, estado: Option<&str>) {
    let (fondo, tinta, texto) = match estado {
        Some("PENDIENTE") => (paleta::YELLOW_100, paleta::YELLOW_800, "Esperando pago..."),
        Some("COMPLETADA") => (paleta::GREEN_100, paleta::GREEN_800, "¡Pago completado!"),
        Some("CANCELADA") => (paleta::RED_100, paleta::RED_800, "Pago cancelado"),
        _ => (paleta::GRAY_100, paleta::GRAY_800, "Desconocido"),
    };
    let rect = caja(ui, 40.0);
    ui.painter().rect_filled(rect, 20.0, fondo);
    let etiqueta = format!("Estado: {texto}");
    let ancho = medir(ui, &etiqueta, Tam::Base, Peso::Medio);
    let puntos = if estado == Some("PENDIENTE") { 8.0 + 36.0 } else { 0.0 };
    let x = rect.center().x - (ancho + puntos) / 2.0;
    pintar(ui, egui::pos2(x, rect.center().y), Align2::LEFT_CENTER, &etiqueta, Tam::Base, Peso::Medio, tinta);
    if puntos > 0.0 {
        let t = ui.input(|i| i.time);
        for k in 0..3 {
            let fase = ((t - k as f64 * 0.15) * std::f64::consts::PI).sin() * 0.5 + 0.5;
            let alfa = 0.5 + 0.5 * fase as f32;
            let centro = egui::pos2(x + ancho + 8.0 + 2.0 + 4.0 + k as f32 * 12.0, rect.center().y + 2.0);
            ui.painter().circle_filled(centro, 4.0, paleta::YELLOW_500.gamma_multiply(alfa));
        }
        ui.ctx().request_repaint();
    }
}
