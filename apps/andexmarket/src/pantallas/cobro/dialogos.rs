use std::time::Duration;

use eframe::egui::{self, Align, Align2, Area, Color32, Context, CursorIcon, Frame, Id, Key, Layout, Margin, Modifiers, Order, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder, Vec2};
use nucleo::carrito::Producto;
use nucleo::cobros::{self, Descuento, TipoDescuento};
use nucleo::formato;

use super::{dinero, Cobro, Lado, ResultadoCobro};
use crate::app::Contexto;
use crate::ui::marca::{self, color, fuente, radio, sombra, tam};
use crate::ui::v2::{self, animacion, Boton, BotonIcono, Contenedor, Estilo, EstiloSegmentado, EstiloTecla, Segmento};
use crate::ui::{pintar_icono_solar_en, Peso};

const SEGUNDOS_COMPLETADA: f64 = 6.0;
const ALTO_PIE: f32 = 44.0;

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

fn alto_linea(px: f32) -> f32 {
    (px * 1.21).round()
}

fn linea(ui: &mut Ui, ancla: Align2, contenido: &str, px: f32, peso: Peso, color_texto: Color32, espaciado_em: f32) -> Rect {
    let g = v2::galeria_espaciada(ui, contenido, fuente(px, peso), color_texto, px * espaciado_em);
    let alto = alto_linea(px).max(g.size().y.min(alto_linea(px)));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    let x = match ancla.x() {
        Align::Min => rect.left(),
        Align::Center => rect.center().x - g.size().x / 2.0,
        Align::Max => rect.right() - g.size().x,
    };
    let pos = egui::pos2(x, rect.center().y - g.size().y / 2.0);
    let caja = Rect::from_min_size(pos, g.size());
    ui.painter().galley(pos, g, color_texto);
    caja
}

struct Texto<'a> {
    contenido: &'a str,
    px: f32,
    peso: Peso,
    color: Color32,
    espaciado_em: f32,
}

fn t(contenido: &str, px: f32, peso: Peso, color_texto: Color32) -> Texto<'_> {
    Texto { contenido, px, peso, color: color_texto, espaciado_em: 0.0 }
}

impl Texto<'_> {
    fn espaciado(mut self, em: f32) -> Self {
        self.espaciado_em = em;
        self
    }
}

fn fila(ui: &mut Ui, izquierda: Texto, derecha: Texto) -> (Rect, Rect) {
    let gi = v2::galeria_espaciada(ui, izquierda.contenido, fuente(izquierda.px, izquierda.peso), izquierda.color, izquierda.px * izquierda.espaciado_em);
    let gd = v2::galeria_espaciada(ui, derecha.contenido, fuente(derecha.px, derecha.peso), derecha.color, derecha.px * derecha.espaciado_em);
    let alto = alto_linea(izquierda.px).max(alto_linea(derecha.px));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    let pi = egui::pos2(rect.left(), rect.center().y - gi.size().y / 2.0);
    let pd = egui::pos2(rect.right() - gd.size().x, rect.center().y - gd.size().y / 2.0);
    let (ri, rd) = (Rect::from_min_size(pi, gi.size()), Rect::from_min_size(pd, gd.size()));
    ui.painter().galley(pi, gi, izquierda.color);
    ui.painter().galley(pd, gd, derecha.color);
    (ri, rd)
}

fn tachar(ui: &Ui, rect: Rect, color_linea: Color32) {
    let y = rect.center().y + 0.5;
    ui.painter().hline(rect.x_range(), y, Stroke::new(1.0, color_linea));
}

fn panel<R>(ui: &mut Ui, fondo: Color32, borde: Option<Color32>, radio_panel: u8, relleno: Margin, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::NONE
        .fill(fondo)
        .stroke(borde.map_or(Stroke::NONE, |c| Stroke::new(1.0, c)))
        .corner_radius(radio_panel)
        .inner_margin(relleno)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contenido(ui)
        })
        .inner
}

fn margen(vertical: i8, horizontal: i8) -> Margin {
    Margin::symmetric(horizontal, vertical)
}

fn pie<R>(ui: &mut Ui, arriba: i8, abajo: i8, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::NONE
        .inner_margin(Margin { left: 24, right: 24, top: arriba, bottom: abajo })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), ALTO_PIE), Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                contenido(ui)
            })
            .inner
        })
        .inner
}

fn boton(texto: impl Into<String>, estilo: Estilo) -> Boton<'static> {
    Boton::new(texto, estilo).alto(ALTO_PIE).radio(radio::BOTON_MODAL).sin_sombra()
}

fn etiqueta_campo(ui: &mut Ui, contenido: &str) {
    linea(ui, Align2::LEFT_CENTER, contenido, tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO, 0.0);
}

fn chip_valor(ui: &mut Ui, contenido: &str, ancho: Option<f32>, alto: f32, radio_chip: u8, px: f32, activo: bool) -> bool {
    let ancho = ancho.unwrap_or_else(|| v2::ancho_texto(ui, contenido, px, Peso::Medio) + 24.0);
    v2::opcion_eleccion(ui, Vec2::new(ancho, alto), radio_chip, activo, 1.0, |ui, rect, _| {
        v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, contenido, px, Peso::Medio, color::TEXTO);
    })
    .clicked()
}

fn campo_monto(ui: &mut Ui, id: Id, texto: &mut String, sugerencia: &str, px: f32, error: bool, habilitado: bool, enfocar: &mut bool) -> Response {
    let ancho = ui.available_width();
    let contenedor = Contenedor { alto: marca::alto::CAMPO_GRANDE, separacion: 6.0, error, ..Default::default() };
    let respuesta = contenedor.mostrar(ui, ancho, |ui| {
        v2::texto(ui, "$", tam::DESTACADO, Peso::Normal, color::TEXTO_TENUE);
        let alto = ui.available_height();
        ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), alto), Layout::left_to_right(Align::Center), |ui| {
            v2::entrada_desnuda(texto).id(id).tam(px).sugerencia(sugerencia).habilitado(habilitado).ui(ui)
        })
        .inner
    });
    if *enfocar {
        respuesta.request_focus();
        *enfocar = false;
    }
    respuesta
}

fn filtrar_numero(texto: &mut String) {
    if texto.contains(',') {
        *texto = texto.replace(',', ".");
    }
    texto.retain(|c| c.is_ascii_digit() || c == '.' || c == '-');
}

fn enter(ctx: &Context) -> bool {
    ctx.input(|i| i.key_pressed(Key::Enter))
}

fn mantener_foco(ctx: &Context, respuesta: &Response) {
    if respuesta.lost_focus() && enter(ctx) {
        respuesta.request_focus();
    }
}

fn cantidad_producto(item: &Producto) -> String {
    if item.unit.eq_ignore_ascii_case("kg") {
        format!("{} kg", formato::numero_con(item.quantity, 3, 3))
    } else {
        format!("{} u", formato::numero(item.quantity))
    }
}

fn lista_productos(ui: &mut Ui, items: &[Producto], total: f64) {
    panel(ui, color::BLANCO, Some(color::BORDE_SUAVE), radio::CAMPO, Margin::ZERO, |ui| {
        ui.spacing_mut().scroll.fade.strength = 0.0;
        egui::ScrollArea::vertical().max_height(178.0).auto_shrink([false, true]).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for item in items {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 35.0), Sense::hover());
                let interior = rect.shrink2(Vec2::new(14.0, 0.0));
                let detalle = format!("{} · {}", cantidad_producto(item), dinero(item.subtotal));
                let ancho_detalle = v2::pintar_texto(ui, interior.right_center(), Align2::RIGHT_CENTER, detalle, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO).width();
                let nombre = v2::galeria(ui, item.name.clone(), fuente(tam::CONTROL, Peso::Normal), color::TEXTO);
                let recorte = Rect::from_min_max(interior.min, egui::pos2(interior.right() - ancho_detalle - 12.0, interior.bottom()));
                ui.painter().with_clip_rect(recorte).galley(egui::pos2(interior.left(), interior.center().y - nombre.size().y / 2.0), nombre, color::TEXTO);
                ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            }
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 37.0), Sense::hover());
            let interior = rect.shrink2(Vec2::new(14.0, 0.0));
            v2::pintar_texto(ui, interior.left_center(), Align2::LEFT_CENTER, "Total", tam::CUERPO, Peso::Semi, color::TEXTO);
            v2::pintar_texto(ui, interior.right_center(), Align2::RIGHT_CENTER, dinero(total), tam::CUERPO, Peso::Semi, color::ROJO);
        });
    });
}

fn texto_descuento(d: &Descuento) -> String {
    match d.tipo {
        TipoDescuento::Porcentaje => format!("{}% de descuento · {}", formato::numero(d.valor), dinero(-d.monto)),
        TipoDescuento::Fijo => format!("Descuento · {}", dinero(-d.monto)),
    }
}

fn tarjeta_metodo(ui: &mut Ui, rect: Rect, id: Id, numero: &str, etiqueta: &str, icono: &str, elegido: bool, girando: bool, habilitado: bool, foco: bool) -> bool {
    let respuesta = ui.interact(rect, id, if habilitado { Sense::click() } else { Sense::hover() });
    let colores = v2::colores_eleccion(elegido || girando);
    let encima = habilitado && respuesta.hovered();
    let mut hijo = ui.new_child(UiBuilder::new().max_rect(rect));
    hijo.multiply_opacity(if habilitado || elegido || girando { 1.0 } else { 0.45 });
    let borde = if encima { color::VERDE } else { colores.borde };
    hijo.painter().rect(rect, radio::TARJETA, colores.fondo, Stroke::new(1.5, borde), StrokeKind::Inside);
    let tecla = Rect::from_min_size(rect.min + Vec2::splat(10.0), Vec2::splat(24.0));
    v2::pintar_tecla(&hijo, tecla, numero, &EstiloTecla::CUADRADA, color::TEXTO_SECUNDARIO);
    let alto_etiqueta = alto_linea(tam::ENTRADA);
    let arriba = rect.center().y - (36.0 + 10.0 + alto_etiqueta) / 2.0;
    let centro_icono = egui::pos2(rect.center().x, arriba + 18.0);
    if girando {
        v2::girando(&hijo, Rect::from_center_size(centro_icono, Vec2::splat(34.0)), color::VERDE);
    } else {
        pintar_icono_solar_en(&hijo, Rect::from_center_size(centro_icono, Vec2::splat(36.0)), icono, color::VERDE);
    }
    v2::pintar_texto(&hijo, egui::pos2(rect.center().x, arriba + 46.0 + alto_etiqueta / 2.0), Align2::CENTER_CENTER, etiqueta, tam::ENTRADA, Peso::Semi, color::TEXTO);
    if foco && habilitado {
        marca::pintar_anillo(ui.painter(), rect, radio::TARJETA, color::ANILLO_FOCO);
    }
    respuesta.on_hover_cursor(if habilitado { CursorIcon::PointingHand } else { CursorIcon::Default }).clicked()
}

fn opcion_medio(ui: &mut Ui, ancho: f32, etiqueta: &str, icono: &str, activo: bool) -> bool {
    v2::opcion_eleccion(ui, Vec2::new(ancho, ALTO_PIE), radio::BOTON_MODAL, activo, 1.5, |ui, rect, colores| {
        let g = v2::galeria(ui, etiqueta, fuente(tam::CUERPO, Peso::Medio), colores.texto);
        let total = 18.0 + 8.0 + g.size().x;
        let x = rect.center().x - total / 2.0;
        pintar_icono_solar_en(ui, Rect::from_min_size(egui::pos2(x, rect.center().y - 9.0), Vec2::splat(18.0)), icono, colores.texto);
        ui.painter().galley(egui::pos2(x + 26.0, rect.center().y - g.size().y / 2.0), g, colores.texto);
    })
    .clicked()
}

fn pastilla_estado(ui: &mut Ui, estado: Option<&str>) {
    let (fondo, tinta, texto, latiendo) = match estado {
        Some("PENDIENTE") => (color::DORADO_FONDO, color::DORADO_TEXTO, "Esperando el pago…", true),
        Some("COMPLETADA") => (color::VERDE_EXITO_FONDO, color::VERDE_EXITO, "Pago acreditado", false),
        Some("CANCELADA") => (color::ROJO_FONDO, color::ROJO, "Pago cancelado", false),
        _ => (color::FONDO_APP, color::TEXTO_APAGADO, "Estado desconocido", false),
    };
    let g = v2::galeria(ui, texto, fuente(tam::CONTROL, Peso::Medio), tinta);
    let tam_pastilla = Vec2::new(12.0 + 7.0 + 8.0 + g.size().x + 12.0, alto_linea(tam::CONTROL) + 10.0);
    let (fila_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), tam_pastilla.y), Sense::hover());
    let rect = Rect::from_center_size(fila_rect.center(), tam_pastilla);
    ui.painter().rect_filled(rect, radio::PASTILLA, fondo);
    let alfa = if latiendo { animacion::pulso(ui.ctx(), 1.1) } else { 1.0 };
    ui.painter().circle_filled(egui::pos2(rect.left() + 12.0 + 3.5, rect.center().y), 3.5, tinta.gamma_multiply(alfa));
    ui.painter().galley(egui::pos2(rect.left() + 27.0, rect.center().y - g.size().y / 2.0), g, tinta);
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
        if self.cancelar_abierto && !self.cancelacion.abierto {
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
        if tope.is_none() {
            self.venta_completada(cx);
        }
    }

    fn dialogo_cancelar(&mut self, cx: &mut Contexto, tope: bool) {
        let ctx = cx.egui.clone();
        let cargando = self.normal.procesando;
        let mut confirmar = false;
        let mut mantener = tope && !cargando && enter(&ctx);
        let items = cx.global.carritos.items().to_vec();
        let total = cx.global.carritos.total();
        let r = v2::Modal::new("cobro-cancelar", "¿Cancelar la orden?")
            .subtitulo("Se quitan todos los productos de la orden")
            .icono("trash-2")
            .peligro()
            .ancho(500.0)
            .mostrar(&ctx, |ui| {
                v2::cuerpo_modal(ui, 16, |ui| {
                    v2::parrafo(ui, "Esta acción no se puede deshacer.", tam::CUERPO, color::TEXTO_SECUNDARIO, 1.55, ui.available_width());
                    ui.add_space(14.0);
                    lista_productos(ui, &items, total);
                });
                pie(ui, 20, 22, |ui| {
                    let texto = if cargando { "Cancelando…" } else { "Sí, cancelar" };
                    confirmar = boton(texto, Estilo::Peligro).alto(42.0).cargando(cargando).ui(ui).clicked();
                    mantener |= boton("No, volver", Estilo::Secundario).alto(42.0).tecla("Enter").habilitado(!cargando).ui(ui).clicked();
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
        let tipo_factura = cx
            .global
            .negocio
            .as_ref()
            .and_then(|n| n["configuracionAfip"]["tipoFactura"].as_str())
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().to_string());
        let subtitulo = match (afip, tipo_factura) {
            (true, Some(tipo)) => format!("Se emite Factura {tipo}. Presioná 1 a 4 para elegir."),
            (true, None) => "Se emite la factura en AFIP. Presioná 1 a 4 para elegir.".to_string(),
            (false, _) => "Sin factura. Presioná 1 a 4 para elegir.".to_string(),
        };
        let titulo = if afip { "Cobrar con factura AFIP" } else { "Cobrar orden" };
        let r = v2::Modal::new(("cobro-pago", afip), titulo).subtitulo(&subtitulo).icono("wallet").ancho(520.0).mostrar(&ctx, |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                panel(ui, color::FONDO_APP, None, radio::POPOVER, margen(16, 18), |ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto_linea(tam::ETIQUETA) + 4.0 + tam::MONTO), Sense::hover());
                    v2::pintar_texto(ui, rect.left_top() + Vec2::new(0.0, alto_linea(tam::ETIQUETA) / 2.0), Align2::LEFT_CENTER, "Total a cobrar", tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
                    let total = subtotal - descuento.map_or(0.0, |d| d.monto);
                    let g = v2::galeria_espaciada(ui, dinero(total), fuente(tam::MONTO, Peso::Semi), color::TEXTO, marca::em(tam::MONTO, -0.02));
                    ui.painter().galley(egui::pos2(rect.left(), rect.bottom() - tam::MONTO / 2.0 - g.size().y / 2.0), g, color::TEXTO);
                    if let Some(d) = descuento {
                        let abajo = egui::pos2(rect.right(), rect.bottom() - 2.0);
                        v2::pintar_texto(ui, abajo, Align2::RIGHT_BOTTOM, texto_descuento(&d), tam::ETIQUETA, Peso::Semi, color::VERDE_EXITO);
                        let arriba = abajo - Vec2::new(0.0, alto_linea(tam::ETIQUETA) + 3.0);
                        let tachado = v2::pintar_texto(ui, arriba, Align2::RIGHT_BOTTOM, dinero(subtotal), tam::ETIQUETA, Peso::Normal, color::TEXTO_TENUE);
                        tachar(ui, tachado, color::TEXTO_TENUE);
                    }
                });
                ui.add_space(16.0);
                let ancho = (ui.available_width() - 10.0) / 2.0;
                let (grilla, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 124.0 * 2.0 + 10.0), Sense::hover());
                let metodos = [("qr", "Transferencia / QR", "qr"), ("tarjeta", "Tarjeta", "card"), ("efectivo", "Efectivo", "cash"), ("split", "Pago mixto", "split")];
                for (i, (m, etiqueta, icono)) in metodos.into_iter().enumerate() {
                    let x = grilla.left() + (i % 2) as f32 * (ancho + 10.0);
                    let y = grilla.top() + (i / 2) as f32 * (124.0 + 10.0);
                    let rect = Rect::from_min_size(egui::pos2(x, y), Vec2::new(ancho, 124.0));
                    let numero = (i + 1).to_string();
                    let clic = tarjeta_metodo(
                        ui,
                        rect,
                        Id::new(("cobro-metodo", afip, m)),
                        &numero,
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
                ui.add_space(24.0);
            });
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
        let afip = self.accion_descuento == Some(Lado::Afip);
        let Some(dialogo) = self.descuento_dialogo.as_mut() else { return };
        let escape = tope && ctx.input(|i| i.key_pressed(Key::Escape));
        let mut confirmar = false;
        let mut cancelar = escape;
        let subtitulo = if afip { "Después se emite la factura en AFIP" } else { "Orden sin factura" };
        let r = v2::Modal::new("cobro-descuento", "Aplicar descuento").subtitulo(subtitulo).icono("percent").ancho(460.0).mostrar(&ctx, |ui| {
            let calculado = v2::cuerpo_modal(ui, 20, |ui| {
                let anterior = dialogo.cantidad_fija;
                let mut indice = usize::from(dialogo.cantidad_fija);
                let pestanas = [Segmento::con_icono("Porcentaje", "percent"), Segmento::con_icono("Monto fijo", "dollar-sign")];
                v2::segmentado(ui, &pestanas, &mut indice, Some(ui.available_width()), EstiloSegmentado::Claro);
                dialogo.cantidad_fija = indice == 1;
                if anterior && !dialogo.cantidad_fija {
                    dialogo.enfocar = true;
                }
                ui.add_space(16.0);
                let presets: [(&str, String); 4] = if dialogo.cantidad_fija {
                    ["500", "1000", "2000", "5000"].map(|v| (v, dinero(v.parse().unwrap_or(0.0))))
                } else {
                    ["5", "10", "15", "20"].map(|v| (v, format!("{v}%")))
                };
                let ancho_preset = (ui.available_width() - 8.0 * 3.0) / 4.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for (valor, etiqueta) in &presets {
                        let texto = if dialogo.cantidad_fija { &mut dialogo.monto } else { &mut dialogo.porcentaje };
                        if chip_valor(ui, etiqueta, Some(ancho_preset), 36.0, radio::CHIP_ICONO, tam::CONTROL, texto == valor) {
                            *texto = valor.to_string();
                            dialogo.enfocar = true;
                        }
                    }
                });
                ui.add_space(16.0);
                let (prefijo, sufijo) = if dialogo.cantidad_fija { ("$", "") } else { ("", "%") };
                let id = Id::new(("cobro-descuento-valor", dialogo.cantidad_fija));
                let texto = if dialogo.cantidad_fija { &mut dialogo.monto } else { &mut dialogo.porcentaje };
                let contenedor = Contenedor { alto: 60.0, radio: radio::POPOVER, relleno_izquierdo: 16.0, relleno_derecho: 16.0, separacion: 8.0, ..Default::default() };
                let ancho = ui.available_width();
                let respuesta = contenedor.mostrar(ui, ancho, |ui| {
                    v2::texto(ui, prefijo, tam::GRANDE, Peso::Normal, color::TEXTO_TENUE);
                    let reservado = if sufijo.is_empty() { 0.0 } else { v2::ancho_texto(ui, sufijo, tam::GRANDE, Peso::Normal) + 8.0 };
                    let alto = ui.available_height();
                    let respuesta = ui
                        .allocate_ui_with_layout(Vec2::new(ui.available_width() - reservado, alto), Layout::left_to_right(Align::Center), |ui| {
                            v2::entrada_desnuda(texto).id(id).tam(28.0).peso(Peso::Semi).centrada().sugerencia("0").ui(ui)
                        })
                        .inner;
                    if !sufijo.is_empty() {
                        v2::texto(ui, sufijo, tam::GRANDE, Peso::Normal, color::TEXTO_TENUE);
                    }
                    respuesta
                });
                if dialogo.enfocar {
                    respuesta.request_focus();
                    dialogo.enfocar = false;
                }
                filtrar_numero(texto);
                mantener_foco(&ctx, &respuesta);
                ui.add_space(16.0);
                let valor = if dialogo.cantidad_fija { &dialogo.monto } else { &dialogo.porcentaje };
                let calculado = cobros::descuento_calculado(subtotal, !dialogo.cantidad_fija, valor);
                panel(ui, color::FONDO_APP, None, radio::POPOVER, margen(14, 16), |ui| {
                    fila(ui, t("Subtotal", tam::CUERPO_MAS, Peso::Normal, color::TEXTO_APAGADO), t(&dinero(subtotal), tam::CUERPO_MAS, Peso::Normal, color::TEXTO));
                    ui.add_space(8.0);
                    let menos = if calculado > 0.0 { dinero(-calculado) } else { dinero(0.0) };
                    fila(ui, t("Descuento", tam::CUERPO_MAS, Peso::Normal, color::VERDE_EXITO), t(&menos, tam::CUERPO_MAS, Peso::Normal, color::VERDE_EXITO));
                    ui.add_space(8.0);
                    let (separador, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
                    ui.painter().rect_filled(separador, 0, color::BORDE);
                    ui.add_space(10.0);
                    fila(ui, t("Total", tam::GRANDE, Peso::Semi, color::TEXTO), t(&dinero(subtotal - calculado), tam::GRANDE, Peso::Semi, color::VERDE));
                });
                calculado
            });
            let valido = calculado > 0.0;
            if tope && valido && enter(&ctx) {
                confirmar = true;
            }
            pie(ui, 20, 22, |ui| {
                confirmar |= boton("Aplicar y cobrar", Estilo::Primario).tecla("Enter").habilitado(valido).ui(ui).clicked();
                cancelar |= boton("Cancelar", Estilo::Secundario).ui(ui).clicked();
            });
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
        let descuento = self.descuento;
        let porcentaje = cx.global.negocio.as_ref().and_then(cobros::descuento_efectivo);
        let minimo = cx.global.negocio.as_ref().and_then(cobros::minimo_descuento_efectivo);
        let p = self.proc_mut(lado);
        let cargando = p.procesando;
        let Some(e) = p.efectivo.as_mut() else { return };
        let (original, cobrar, aplicando) = (e.original, e.cobrar, e.aplicando);
        let alcanza = minimo.is_none_or(|m| original >= m);
        let con_descuento = cobrar < original;
        let mut confirmar = tope && !cargando && enter(&ctx);
        let mut aplicar_d = tope && !cargando && !aplicando && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5)) && alcanza;
        let mut cancelar = false;
        let r = v2::Modal::new(("cobro-efectivo", lado == Lado::Afip), "Cobro en efectivo")
            .subtitulo("Confirmá el monto a cobrar")
            .icono("cash")
            .ancho(480.0)
            .mostrar(&ctx, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    panel(ui, color::FONDO_APP, None, radio::POPOVER, margen(16, 18), |ui| {
                        let (etiqueta, nota) = match (descuento, aplicando) {
                            _ if !con_descuento => ("Monto a cobrar".to_string(), String::new()),
                            (Some(d), _) => ("Monto con descuento".to_string(), format!("Descuento {}: {}", d.etiqueta(), dinero(cobrar - original))),
                            (None, true) => (
                                "Con descuento en efectivo".to_string(),
                                format!("Descuento por pago en efectivo ({}%): {}", formato::numero(porcentaje.unwrap_or(0.0)), dinero(cobrar - original)),
                            ),
                            (None, false) => ("Monto a cobrar (redondeado)".to_string(), format!("Redondeo: {}", dinero(cobrar - original))),
                        };
                        if con_descuento {
                            let (_, tachado) = fila(ui, t("Monto original", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO), t(&dinero(original), tam::CUERPO, Peso::Normal, color::TEXTO_TENUE));
                            tachar(ui, tachado, color::TEXTO_TENUE);
                            ui.add_space(8.0);
                        }
                        fila(ui, t(&etiqueta, tam::ENTRADA, Peso::Medio, color::TEXTO), t(&dinero(cobrar), tam::MONTO_MEDIO, Peso::Semi, color::VERDE).espaciado(-0.02));
                        if con_descuento {
                            ui.add_space(8.0);
                            linea(ui, Align2::LEFT_CENTER, &nota, tam::CHICO, Peso::Normal, color::VERDE_EXITO, 0.0);
                        }
                    });
                    ui.add_space(14.0);
                    etiqueta_campo(ui, "Recibido del cliente");
                    ui.add_space(6.0);
                    billetes(ui, &mut e.pagado, cobrar, &mut e.enfocar);
                    ui.add_space(6.0);
                    let id = Id::new(("cobro-recibido", lado == Lado::Afip));
                    let respuesta = campo_monto(ui, id, &mut e.pagado, "Opcional, para calcular el vuelto", tam::FILA, false, !cargando, &mut e.enfocar);
                    filtrar_numero(&mut e.pagado);
                    mantener_foco(&ctx, &respuesta);
                    let recibido = cobros::parse_float(&e.pagado).unwrap_or(0.0);
                    if recibido > 0.0 && recibido >= cobrar {
                        ui.add_space(14.0);
                        caja_vuelto(ui, "Vuelto", recibido - cobrar);
                    }
                });
                let texto = if cargando { "Registrando…" } else { "Confirmar cobro" };
                if aplicando {
                    pie(ui, 20, 22, |ui| {
                        confirmar |= boton(texto, Estilo::Primario).tecla("Enter").cargando(cargando).ui(ui).clicked();
                        cancelar = boton("Cancelar", Estilo::Secundario).habilitado(!cargando).ui(ui).clicked();
                    });
                } else {
                    pie(ui, 20, 0, |ui| {
                        let etiqueta = match minimo.filter(|_| !alcanza) {
                            Some(m) => format!("Desde {}", dinero(m)),
                            None => porcentaje.map_or("Descuento efectivo".to_string(), |p| format!("Descuento efectivo {}%", formato::numero(p))),
                        };
                        aplicar_d |= boton(etiqueta, Estilo::Dorado).tecla("F5").relleno(14.0).habilitado(!cargando && alcanza).ui(ui).clicked();
                        cancelar = boton("Cancelar", Estilo::Secundario).habilitado(!cargando).ui(ui).clicked();
                    });
                    pie(ui, 10, 22, |ui| {
                        confirmar |= boton(texto, Estilo::Primario).tecla("Enter").cargando(cargando).ui(ui).clicked();
                    });
                }
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
        let porcentaje = cx.global.negocio.as_ref().and_then(cobros::descuento_efectivo);
        let minimo = cx.global.negocio.as_ref().and_then(cobros::minimo_descuento_efectivo);
        let p = self.proc_mut(lado);
        let cargando = p.procesando;
        let Some(e) = p.efectivo.as_mut() else { return };
        let (total, original, aplicando) = (e.cobrar, e.original, e.aplicando);
        let alcanza = minimo.is_none_or(|m| original >= m);
        let ofrecer_descuento = !aplicando && descuento.is_none();
        let mut aplicar_d = tope && !cargando && ofrecer_descuento && alcanza && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5));
        let mut confirmar = false;
        let mut cancelar = false;
        let r = v2::Modal::new(("cobro-exacto", lado == Lado::Afip), "Cobro en efectivo")
            .subtitulo("Ingresá con cuánto paga el cliente para calcular el vuelto")
            .icono("cash")
            .ancho(600.0)
            .mostrar(&ctx, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    panel(ui, color::FONDO_APP, None, radio::POPOVER, margen(16, 18), |ui| {
                        if let (Some(d), true) = (descuento, subtotal != 0.0) {
                            fila(ui, t("Subtotal", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO), t(&dinero(subtotal), tam::CUERPO, Peso::Normal, color::TEXTO));
                            ui.add_space(8.0);
                            let etiqueta = format!("Descuento ({})", d.etiqueta());
                            fila(ui, t(&etiqueta, tam::CUERPO, Peso::Normal, color::VERDE_EXITO), t(&dinero(-d.monto), tam::CUERPO, Peso::Normal, color::VERDE_EXITO));
                            ui.add_space(8.0);
                        } else if aplicando && total < original {
                            fila(ui, t("Subtotal", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO), t(&dinero(original), tam::CUERPO, Peso::Normal, color::TEXTO));
                            ui.add_space(8.0);
                            let etiqueta = format!("Descuento efectivo ({}%)", formato::numero(porcentaje.unwrap_or(0.0)));
                            fila(ui, t(&etiqueta, tam::CUERPO, Peso::Normal, color::VERDE_EXITO), t(&dinero(total - original), tam::CUERPO, Peso::Normal, color::VERDE_EXITO));
                            ui.add_space(8.0);
                        }
                        fila(ui, t("Total a pagar", tam::ENTRADA, Peso::Medio, color::TEXTO), t(&dinero(total), tam::MONTO_MEDIO, Peso::Semi, color::VERDE).espaciado(-0.02));
                    });
                    ui.add_space(14.0);
                    etiqueta_campo(ui, "Con cuánto paga el cliente");
                    ui.add_space(6.0);
                    billetes(ui, &mut e.pagado, total, &mut e.enfocar);
                    ui.add_space(6.0);
                    let id = Id::new(("cobro-exacto-campo", lado == Lado::Afip));
                    let error_previo = e.error.is_some();
                    let respuesta = campo_monto(ui, id, &mut e.pagado, "Ej: 5000", tam::FILA, error_previo, !cargando, &mut e.enfocar);
                    filtrar_numero(&mut e.pagado);
                    mantener_foco(&ctx, &respuesta);
                    if e.pagado != e.anterior {
                        e.anterior = e.pagado.clone();
                        e.error = cobros::vuelto(&e.pagado, total).1;
                    }
                    let cambio = cobros::vuelto(&e.pagado, total).0;
                    if let Some(texto) = e.error {
                        ui.add_space(6.0);
                        linea(ui, Align2::LEFT_CENTER, texto, tam::CHICO, Peso::Normal, color::ROJO, 0.0);
                    }
                    if !e.pagado.is_empty() && e.error.is_none() && cambio >= 0.0 {
                        ui.add_space(14.0);
                        caja_vuelto(ui, if cambio == 0.0 { "Pago exacto, sin vuelto" } else { "Vuelto" }, cambio);
                    }
                });
                let habilitado = !cargando && e.error.is_none() && !e.pagado.is_empty();
                if tope && habilitado && enter(&ctx) {
                    confirmar = true;
                }
                let texto = if cargando { "Registrando…" } else { "Confirmar cobro" };
                pie(ui, 20, 22, |ui| {
                    confirmar |= boton(texto, Estilo::Primario).tecla("Enter").cargando(cargando).habilitado(habilitado || cargando).ui(ui).clicked();
                    cancelar = boton("Cancelar", Estilo::Secundario).habilitado(!cargando).ui(ui).clicked();
                    if ofrecer_descuento {
                        let etiqueta = match minimo.filter(|_| !alcanza) {
                            Some(m) => format!("Desde {}", dinero(m)),
                            None => porcentaje.map_or("Descuento efectivo".to_string(), |p| format!("Descuento efectivo {}%", formato::numero(p))),
                        };
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            aplicar_d |= boton(etiqueta, Estilo::Dorado).tecla("F5").relleno(14.0).habilitado(!cargando && alcanza).ui(ui).clicked();
                        });
                    }
                });
            });
        if r.cerrar || cancelar {
            self.cerrar_efectivo(lado);
            return;
        }
        if aplicar_d {
            self.aplicar_descuento_efectivo(cx, lado);
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
        let subtitulo = if afip {
            "Una parte en efectivo y el resto con otro medio. Se emite factura AFIP."
        } else {
            "Una parte en efectivo y el resto con otro medio"
        };
        let r = v2::Modal::new(("cobro-mixto", afip), "Pago mixto").subtitulo(subtitulo).icono("split").ancho(480.0).mostrar(&ctx, |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                panel(ui, color::FONDO_APP, None, radio::POPOVER, margen(14, 14), |ui| {
                    linea(ui, Align2::CENTER_CENTER, "Total a pagar", tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO, 0.0);
                    ui.add_space(4.0);
                    linea(ui, Align2::CENTER_CENTER, &dinero(total), tam::MONTO_MEDIO, Peso::Semi, color::VERDE, -0.02);
                    if let Some(d) = descuento {
                        ui.add_space(4.0);
                        let texto = format!("Incluye descuento {}: {}", d.etiqueta(), dinero(-d.monto));
                        linea(ui, Align2::CENTER_CENTER, &texto, tam::CHICO, Peso::Normal, color::VERDE_EXITO, 0.0);
                    }
                });
                ui.add_space(14.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto_linea(tam::ETIQUETA)), Sense::hover());
                pintar_icono_solar_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0)), "cash", color::VERDE);
                v2::pintar_texto(ui, egui::pos2(rect.left() + 22.0, rect.center().y), Align2::LEFT_CENTER, "Parte en efectivo", tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO);
                ui.add_space(6.0);
                let id = Id::new(("cobro-mixto-efectivo", afip));
                let respuesta = campo_monto(ui, id, &mut p.efectivo_mixto, "0", 18.0, false, !procesando, &mut p.enfocar_mixto);
                filtrar_numero(&mut p.efectivo_mixto);
                mantener_foco(&ctx, &respuesta);
                ui.add_space(14.0);
                etiqueta_campo(ui, "El resto con");
                ui.add_space(6.0);
                let ancho = (ui.available_width() - 8.0) / 2.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for (m, texto, icono) in [("tarjeta", "Tarjeta", "card"), ("qr", "Transferencia", "qr")] {
                        if opcion_medio(ui, ancho, texto, icono, p.segundo == m) && !procesando {
                            p.segundo = m;
                        }
                    }
                });
                ui.add_space(14.0);
                let etiqueta = if p.segundo == "tarjeta" { "Resto con tarjeta" } else { "Resto por transferencia" };
                let restante = dinero(cobros::mixto_restante(total, &p.efectivo_mixto));
                panel(ui, color::BLANCO, Some(color::BORDE_SUAVE), radio::CAMPO, margen(12, 16), |ui| {
                    fila(ui, t(etiqueta, tam::CUERPO, Peso::Normal, color::TEXTO_SECUNDARIO), t(&restante, tam::GRANDE, Peso::Semi, color::VERDE));
                });
            });
            let habilitado = cobros::mixto_habilitado(total, &p.efectivo_mixto, procesando);
            if tope && habilitado && enter(&ctx) {
                completar = true;
            }
            pie(ui, 20, 22, |ui| {
                completar |= boton("Completar pago", Estilo::Primario).tecla("Enter").cargando(procesando).habilitado(habilitado || procesando).ui(ui).clicked();
                cancelar = boton("Cancelar", Estilo::Secundario).habilitado(!procesando).ui(ui).clicked();
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
        let afip = self.flujo_afip;
        let fuente_qr = if afip { &self.afip } else { &self.normal };
        let qr = fuente_qr.qr.clone();
        let estado = fuente_qr.estado.clone();
        let descuento = self.descuento;
        let subtotal = cx.global.carritos.total();
        let uri = qr.as_ref().and_then(|q| q["qrImageUrl"].as_str()).and_then(|url| self.uri_qr(cx, url));
        let pendiente = estado.as_deref() == Some("PENDIENTE");
        let acreditado = estado.as_deref() == Some("COMPLETADA");
        let mut manual = tope && pendiente && qr.is_some() && enter(&ctx);
        let mut cancelar = false;
        let subtitulo = if afip { "Mercado Pago o transferencia · la factura AFIP se emite al acreditarse" } else { "Mercado Pago o transferencia" };
        let r = v2::Modal::new("cobro-qr", "Cobro con QR").subtitulo(subtitulo).icono("qr").ancho(440.0).mostrar(&ctx, |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                let (fila_qr, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 250.0), Sense::hover());
                let marco = Rect::from_center_size(fila_qr.center(), Vec2::splat(250.0));
                ui.painter().rect(marco, radio::PANEL, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
                let area = marco.shrink(15.0);
                match (&qr, &uri) {
                    (Some(_), Some(uri)) => {
                        let tinte = if acreditado { Color32::WHITE.gamma_multiply(0.15) } else { Color32::WHITE };
                        egui::Image::new(uri.clone()).tint(tinte).paint_at(ui, area);
                    }
                    (Some(_), None) => v2::tarjetas::pintar_shimmer(ui, area, radio::ICONO, true),
                    (None, _) => {
                        v2::girando(ui, Rect::from_center_size(area.center() - Vec2::new(0.0, 14.0), Vec2::splat(30.0)), color::VERDE);
                        v2::pintar_texto(ui, area.center() + Vec2::new(0.0, 20.0), Align2::CENTER_CENTER, "Generando código QR…", tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
                    }
                }
                if acreditado {
                    let circulo = Rect::from_center_size(marco.center(), Vec2::splat(84.0));
                    ui.painter().add(sombra::ICONO_EXITO.forma(circulo, radio::PASTILLA));
                    ui.painter().rect_filled(circulo, radio::PASTILLA, color::VERDE);
                    pintar_icono_solar_en(ui, Rect::from_center_size(circulo.center(), Vec2::splat(46.0)), "check", color::BLANCO);
                }
                if let Some(datos) = &qr {
                    ui.add_space(16.0);
                    let mixto = cobros::verdad(&datos["isSplitPayment"]);
                    let (monto, detalle) = match descuento {
                        _ if mixto => (cobros::numero(&datos["monto"]), Some(format!("Pago mixto: {} en efectivo + QR", dinero(cobros::numero(&datos["cashAmount"]))))),
                        Some(d) => (subtotal - d.monto, Some(format!("Subtotal {} · Descuento ({}) {}", dinero(subtotal), d.etiqueta(), dinero(-d.monto)))),
                        None => (cobros::numero(&datos["monto"]), None),
                    };
                    linea(ui, Align2::CENTER_CENTER, &dinero(monto), 30.0, Peso::Semi, color::TEXTO, -0.02);
                    if let Some(detalle) = detalle {
                        ui.add_space(4.0);
                        linea(ui, Align2::CENTER_CENTER, &detalle, tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO, 0.0);
                    }
                    ui.add_space(6.0);
                    pastilla_estado(ui, estado.as_deref());
                }
                ui.add_space(16.0);
                let ayuda = "El cliente escanea el código con Mercado Pago o la app de su banco. La venta se registra sola cuando se acredita.";
                let ancho = ui.available_width().min(360.0);
                let mut trabajo = egui::text::LayoutJob::simple(ayuda.to_string(), fuente(tam::ETIQUETA, Peso::Normal), color::TEXTO_APAGADO, ancho);
                trabajo.halign = Align::Center;
                trabajo.sections[0].format.line_height = Some(tam::ETIQUETA * 1.5);
                let g = ui.painter().layout_job(trabajo);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), g.size().y), Sense::hover());
                ui.painter().galley(egui::pos2(rect.center().x, rect.top()), g, color::TEXTO_APAGADO);
            });
            Frame::NONE.inner_margin(Margin { left: 24, right: 24, top: 20, bottom: 22 }).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 42.0), Layout::left_to_right(Align::Center), |ui| {
                    cancelar = Boton::secundario("Cancelar cobro").modal().ui(ui).clicked();
                    if pendiente && qr.is_some() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            manual |= Boton::secundario("Confirmar manualmente").modal().icono("check").tam_icono(16.0).color_icono(color::VERDE).ui(ui).clicked();
                        });
                    }
                });
            });
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
        let r = v2::Modal::new(("cobro-manual", lado == Lado::Afip), "Confirmar manualmente")
            .subtitulo("Ingresá la contraseña para completar la orden sin esperar la acreditación")
            .icono("lock")
            .ancho(440.0)
            .mostrar(&ctx, |ui| {
                v2::cuerpo_modal(ui, 22, |ui| {
                    v2::etiqueta(ui, "Contraseña");
                    ui.add_space(8.0);
                    let respuesta = v2::CampoTexto::new(&mut manual.clave)
                        .id(Id::new(("cobro-manual-clave", lado == Lado::Afip)))
                        .icono("lock")
                        .contrasena(true)
                        .alto(marca::alto::CAMPO_GRANDE)
                        .tam_texto(tam::DESTACADO)
                        .sugerencia("Contraseña")
                        .habilitado(!enviando)
                        .ui(ui)
                        .respuesta;
                    if manual.enfocar {
                        respuesta.request_focus();
                        manual.enfocar = false;
                    }
                    if tope && respuesta.lost_focus() && enter(&ctx) {
                        enviar = true;
                        respuesta.request_focus();
                    }
                });
                pie(ui, 22, 22, |ui| {
                    let texto = if enviando { "Procesando…" } else { "Confirmar" };
                    enviar |= Boton::primario(texto).modal().tecla("Enter").cargando(enviando).ui(ui).clicked();
                    cancelar = Boton::secundario("Cancelar").modal().habilitado(!enviando).ui(ui).clicked();
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
        let items = cx.global.carritos.items().to_vec();
        let c = &mut self.cancelacion;
        let total = c.pendiente.as_ref().map_or(0.0, |p| p.monto);
        let enviando = c.enviando;
        let mut confirmar = ctrl_enter && !enviando;
        let mut mantener = escape;
        let r = v2::Modal::new("cobro-cancelacion", "Cancelar y registrar")
            .subtitulo("La cancelación queda registrada con su motivo")
            .icono("trash-2")
            .peligro()
            .ancho(500.0)
            .mostrar(&ctx, |ui| {
                v2::cuerpo_modal(ui, 16, |ui| {
                    if c.pendiente.as_ref().is_some_and(|p| !p.productos.is_empty()) {
                        ui.add_space(4.0);
                        lista_productos(ui, &items, total);
                    }
                    ui.add_space(14.0);
                    v2::etiqueta(ui, "Motivo de la cancelación");
                    ui.add_space(6.0);
                    let respuesta = v2::CampoTexto::new(&mut c.motivo)
                        .id(Id::new("cobro-cancelacion-motivo"))
                        .multilinea(96.0)
                        .tam_texto(tam::CUERPO)
                        .sugerencia("Ej: el cliente cambió de opinión, error en el pedido…")
                        .caracteres_maximos(500)
                        .habilitado(!enviando)
                        .ui(ui)
                        .respuesta;
                    if c.enfocar {
                        respuesta.request_focus();
                        c.enfocar = false;
                    }
                    if c.motivo.chars().count() > 500 {
                        c.motivo = c.motivo.chars().take(500).collect();
                    }
                    ui.add_space(6.0);
                    let contador = format!("{}/500", c.motivo.chars().count());
                    fila(ui, t("Queda registrado en el sistema. Mínimo 3 caracteres.", tam::XS, Peso::Normal, color::TEXTO_TENUE), t(&contador, tam::XS, Peso::Normal, color::TEXTO_TENUE));
                    if !c.error.is_empty() {
                        ui.add_space(6.0);
                        linea(ui, Align2::LEFT_CENTER, &c.error, tam::CHICO, Peso::Normal, color::ROJO, 0.0);
                    }
                });
                let valido = cobros::validar_motivo(&c.motivo).is_ok();
                pie(ui, 20, 22, |ui| {
                    let texto = if enviando { "Registrando…" } else { "Cancelar y registrar" };
                    confirmar |= boton(texto, Estilo::Peligro).alto(42.0).tecla("Ctrl+Enter").cargando(enviando).habilitado(valido || enviando).ui(ui).clicked();
                    mantener |= boton("No, volver", Estilo::Secundario).alto(42.0).habilitado(!enviando).ui(ui).clicked();
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

    fn cerrar_completada(&mut self) {
        self.completada = None;
        self.resultado = Some(ResultadoCobro::VentaRegistrada);
    }

    fn venta_completada(&mut self, cx: &mut Contexto) {
        let Some(completada) = self.completada.as_ref() else { return };
        let ctx = cx.egui.clone();
        let transcurrido = ctx.input(|i| i.time) - completada.desde;
        if !cx.global.carritos.items().is_empty() || transcurrido >= SEGUNDOS_COMPLETADA {
            self.completada = None;
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Escape)) {
            self.cerrar_completada();
            return;
        }
        ctx.request_repaint_after(Duration::from_secs_f64(SEGUNDOS_COMPLETADA - transcurrido));
        let resumen = completada.resumen.clone();
        let id = Id::new("cobro-venta-completada");
        let pantalla = ctx.content_rect();
        let fundido = animacion::apertura(&ctx, id.with("fondo"), animacion::DURACION_FONDO);
        let avance = animacion::apertura(&ctx, id, animacion::DURACION_MODAL);
        Area::new(id.with("velo")).order(Order::Middle).fixed_pos(pantalla.min).interactable(false).show(&ctx, |ui| {
            ui.painter().rect_filled(pantalla, 0, color::OVERLAY_MODAL.gamma_multiply(fundido));
        });
        let ancho = 440f32.min(pantalla.width() - 48.0);
        let mut cerrar = false;
        Area::new(id).order(Order::Foreground).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(&ctx, |ui| {
            ui.set_opacity(avance);
            animacion::escalar_capa(&ctx, ui.layer_id(), pantalla.center(), animacion::escala_pop(avance));
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ancho);
            let fondo = ui.painter().add(Shape::Noop);
            ui.vertical(|ui| {
                ui.set_width(ancho);
                Frame::NONE.inner_margin(Margin { left: 24, right: 24, top: 22, bottom: 0 }).show(ui, |ui| {
                    ui.set_width(ancho - 48.0);
                    cerrar |= encabezado_completada(ui, resumen.comprobante.is_some());
                });
                v2::cuerpo_modal(ui, 8, |ui| cuerpo_completada(ui, &resumen));
                Frame::NONE.inner_margin(Margin { left: 24, right: 24, top: 20, bottom: 22 }).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let boton_nueva = boton("Nueva venta", Estilo::Primario).tecla("Enter").relleno(20.0);
                    ui.vertical_centered(|ui| {
                        cerrar |= boton_nueva.ui(ui).clicked();
                    });
                });
            });
            let rect = Rect::from_min_size(ui.min_rect().min, Vec2::new(ancho, ui.min_rect().height()));
            ui.painter().set(fondo, Shape::Vec(vec![sombra::MODAL.forma(rect, radio::MODAL), Shape::rect_filled(rect, radio::MODAL, color::BLANCO)]));
        });
        if cerrar {
            self.cerrar_completada();
        }
    }
}

fn billetes(ui: &mut Ui, recibido: &mut String, total: f64, enfocar: &mut bool) {
    let exacto = cobros::numero_js(total);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for (valor, etiqueta) in [(exacto.as_str(), "Exacto".to_string()), ("5000", dinero(5000.0)), ("10000", dinero(10000.0)), ("20000", dinero(20000.0))] {
            if chip_valor(ui, &etiqueta, None, 34.0, radio::CHIP_ICONO, tam::CONTROL, recibido == valor) {
                *recibido = valor.to_string();
                *enfocar = true;
            }
        }
    });
}

fn caja_vuelto(ui: &mut Ui, etiqueta: &str, monto: f64) {
    panel(ui, color::DORADO_FONDO, None, radio::CAMPO, margen(12, 16), |ui| {
        fila(ui, t(etiqueta, tam::CUERPO, Peso::Medio, color::DORADO_TEXTO), t(&dinero(monto), tam::TITULO_SECCION, Peso::Semi, color::DORADO_TEXTO));
    });
}

fn encabezado_completada(ui: &mut Ui, con_factura: bool) -> bool {
    let mut clic = false;
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        let (caja, _) = ui.allocate_exact_size(Vec2::splat(42.0), Sense::hover());
        ui.painter().rect_filled(caja, radio::CAMPO, color::VERDE);
        pintar_icono_solar_en(ui, Rect::from_center_size(caja.center(), Vec2::splat(23.0)), "check", color::BLANCO);
        let ancho_textos = ui.available_width() - 32.0 - 12.0;
        ui.allocate_ui_with_layout(Vec2::new(ancho_textos, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.add_space(1.0);
            v2::titulo(ui, "Venta cobrada", tam::TITULO_MODAL, -0.01, color::TEXTO);
            if con_factura {
                ui.add_space(3.0);
                v2::parrafo(ui, "Factura emitida", tam::CONTROL, color::TEXTO_APAGADO, 1.45, ancho_textos);
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            clic = BotonIcono::cerrar().ui(ui).clicked();
        });
    });
    clic
}

fn bloque_centrado(ui: &mut Ui, tam_bloque: Vec2) -> Rect {
    let (fila_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), tam_bloque.y), Sense::hover());
    Rect::from_center_size(fila_rect.center(), tam_bloque)
}

fn cuerpo_completada(ui: &mut Ui, resumen: &super::Resumen) {
    linea(ui, Align2::CENTER_CENTER, &dinero(resumen.total), 42.0, Peso::Semi, color::TEXTO, -0.025);
    ui.add_space(12.0);
    let mut trabajo = egui::text::LayoutJob::simple(resumen.metodo.clone(), fuente(tam::CUERPO, Peso::Normal), color::TEXTO_APAGADO, ui.available_width());
    trabajo.halign = Align::Center;
    let g = ui.painter().layout_job(trabajo);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), g.size().y), Sense::hover());
    ui.painter().galley(egui::pos2(rect.center().x, rect.top()), g, color::TEXTO_APAGADO);
    if let Some(vuelto) = resumen.vuelto {
        ui.add_space(12.0);
        let etiqueta = v2::galeria(ui, "Vuelto", fuente(tam::ENTRADA, Peso::Semi), color::DORADO_TEXTO);
        let monto = v2::galeria(ui, dinero(vuelto), fuente(tam::ENTRADA, Peso::Semi), color::DORADO_TEXTO);
        let caja = bloque_centrado(ui, Vec2::new(16.0 + etiqueta.size().x + 10.0 + monto.size().x + 16.0, alto_linea(tam::ENTRADA) + 20.0));
        ui.painter().rect_filled(caja, radio::CAMPO, color::DORADO_FONDO);
        let x = caja.left() + 16.0;
        let ancho_etiqueta = etiqueta.size().x;
        ui.painter().galley(egui::pos2(x, caja.center().y - etiqueta.size().y / 2.0), etiqueta, color::DORADO_TEXTO);
        ui.painter().galley(egui::pos2(x + ancho_etiqueta + 10.0, caja.center().y - monto.size().y / 2.0), monto, color::DORADO_TEXTO);
    }
    if let Some(comprobante) = &resumen.comprobante {
        ui.add_space(12.0);
        let titulo = v2::galeria(ui, comprobante.clone(), fuente(tam::ETIQUETA, Peso::Semi), color::TEXTO_SECUNDARIO);
        let cae = resumen.cae.as_ref().map(|c| v2::galeria(ui, format!("CAE {c}"), marca::fuente_mono(tam::XS, Peso::Normal), color::TEXTO_APAGADO));
        let ancho = titulo.size().x.max(cae.as_ref().map_or(0.0, |g| g.size().x)) + 32.0;
        let alto = alto_linea(tam::ETIQUETA) + cae.as_ref().map_or(0.0, |_| 3.0 + alto_linea(tam::XS)) + 20.0;
        let caja = bloque_centrado(ui, Vec2::new(ancho, alto));
        ui.painter().rect(caja, radio::CAMPO, color::BLANCO, Stroke::new(1.0, color::BORDE_SUAVE), StrokeKind::Inside);
        let mut y = caja.top() + 10.0;
        ui.painter().galley(egui::pos2(caja.center().x - titulo.size().x / 2.0, y + (alto_linea(tam::ETIQUETA) - titulo.size().y) / 2.0), titulo, color::TEXTO_SECUNDARIO);
        y += alto_linea(tam::ETIQUETA) + 3.0;
        if let Some(cae) = cae {
            ui.painter().galley(egui::pos2(caja.center().x - cae.size().x / 2.0, y + (alto_linea(tam::XS) - cae.size().y) / 2.0), cae, color::TEXTO_APAGADO);
        }
    }
    if let Some(impreso) = resumen.impreso {
        ui.add_space(12.0);
        let (icono, texto, tinta) = if impreso {
            ("circle-check", "Ticket impreso", color::VERDE_EXITO)
        } else {
            ("alert", "No se pudo imprimir el ticket", color::ROJO)
        };
        let g = v2::galeria(ui, texto, fuente(tam::CONTROL, Peso::Normal), tinta);
        let caja = bloque_centrado(ui, Vec2::new(17.0 + 8.0 + g.size().x, alto_linea(tam::CONTROL).max(17.0)));
        pintar_icono_solar_en(ui, Rect::from_min_size(egui::pos2(caja.left(), caja.center().y - 8.5), Vec2::splat(17.0)), icono, tinta);
        ui.painter().galley(egui::pos2(caja.left() + 25.0, caja.center().y - g.size().y / 2.0), g, tinta);
    }
}

#[cfg(test)]
mod pruebas {
    use super::filtrar_numero;

    #[test]
    fn coma_decimal_del_teclado_numerico() {
        let mut texto = "10,5".to_string();
        filtrar_numero(&mut texto);
        assert_eq!(texto, "10.5");
        let mut texto = "$1a5".to_string();
        filtrar_numero(&mut texto);
        assert_eq!(texto, "15");
    }
}
