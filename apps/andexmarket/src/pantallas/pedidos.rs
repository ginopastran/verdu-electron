use std::time::Duration;

use chrono::Utc;
use eframe::egui::{self, Align2, Color32, CursorIcon, Rect, Response, Sense, StrokeKind, Ui, Vec2};
use nucleo::cierre::{fecha_argentina, id_texto, verdadero};
use nucleo::pedidos as logica;
use serde_json::Value;

use super::cierre::dibujo::{self, color_degradado_texto};
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, fuente, paleta, toast, Dialogo, Direccion, Peso, Tam, RADIO};

const PROPORCIONES: [f32; 6] = [74.0, 154.0, 142.0, 136.0, 158.0, 164.0];

#[derive(Default)]
pub struct DialogoPedidos {
    abierto: bool,
    cargar_al_mostrar: bool,
    transacciones: Vec<Value>,
    pagina: usize,
    usuario_id: Option<String>,
    carga: Option<Tarea<Result<Vec<Value>, String>>>,
    impresion: Option<Tarea<Vec<Value>>>,
}

enum Accion {
    Reimprimir(usize),
    Pagina(usize),
    Actualizar,
    Cerrar,
}

pub fn traer_transacciones(servicios: &Servicios, vendedor_id: &str, incluir_facturas: bool) -> Result<Vec<Value>, String> {
    let traer = |ruta: String| -> Result<Value, String> {
        let respuesta = servicios.api.pedir("GET", &ruta, None, &[]).map_err(|_| "Failed to fetch".to_string())?;
        match respuesta.cuerpo {
            Value::String(_) => Err("Unexpected token".into()),
            cuerpo => Ok(cuerpo),
        }
    };
    let ordenes = traer(format!("/api/ordenes/vendedor/{vendedor_id}"))?;
    let facturas = if incluir_facturas { Some(traer(format!("/api/facturas/vendedor/{vendedor_id}"))?) } else { None };
    let todas = logica::unir_transacciones(&ordenes, facturas.as_ref())?;
    logica::del_dia(todas, &Utc::now())
}

pub fn reimprimir(servicios: &Servicios, orden: &Value, doble: bool) -> Vec<Value> {
    let primera = imprimir(servicios, Comprobante::Venta, orden.clone());
    if !doble {
        return vec![primera];
    }
    std::thread::sleep(Duration::from_millis(500));
    vec![primera, imprimir(servicios, Comprobante::Venta, orden.clone())]
}

fn avisar_reimpresion(ctx: &egui::Context, resultados: &[Value]) {
    let exito = |r: &Value| verdadero(r.get("success"));
    match resultados {
        [primera, segunda] => {
            if exito(primera) && exito(segunda) {
                toast::con_detalle(ctx, toast::Tipo::Exito, "Doble ticket impreso correctamente", "Se mandaron 2 impresiones al script PHP", 2.0);
            } else {
                let estado = |r: &Value| if exito(r) { "OK" } else { "Error" };
                toast::con_detalle(
                    ctx,
                    toast::Tipo::Aviso,
                    "Problemas con la doble impresión",
                    format!("Primera: {}, Segunda: {}", estado(primera), estado(segunda)),
                    2.0,
                );
            }
        }
        [unica] if exito(unica) => toast::exito(ctx, "Ticket impreso correctamente"),
        [unica] if verdadero(unica.get("printerError")) => {
            toast::error(ctx, format!("Error de impresión: {}", nucleo::cierre::texto_valor(&unica["printerError"])))
        }
        _ => toast::error(ctx, "Error al imprimir el ticket"),
    }
}

impl DialogoPedidos {
    pub fn abrir(&mut self) {
        self.abierto = true;
        self.cargar_al_mostrar = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    fn limpiar(&mut self) {
        self.transacciones.clear();
        self.pagina = 1;
        self.carga = None;
        self.impresion = None;
    }

    fn seguir_usuario(&mut self, cx: &Contexto) {
        let actual = cx.global.usuario.as_ref().and_then(|u| u.get("id")).filter(|id| verdadero(Some(id))).map(|id| id_texto(Some(id)));
        if actual != self.usuario_id {
            self.limpiar();
            self.cargar_al_mostrar = self.abierto;
            self.usuario_id = actual;
        }
    }

    fn cargar(&mut self, cx: &mut Contexto) {
        let Some(usuario) = cx.global.usuario.as_ref() else {
            toast::error(&cx.egui, "Debes iniciar sesión para ver órdenes");
            return;
        };
        let vendedor = usuario.get("id").filter(|id| verdadero(Some(id))).map(|id| id_texto(Some(id)));
        let incluir_facturas = cx.global.negocio.as_ref().is_some_and(|n| n["facturacionHabilitada"] == true);
        let servicios = cx.servicios.clone();
        self.carga = Some(Tarea::lanzar(&cx.egui, move || match vendedor {
            Some(id) => traer_transacciones(&servicios, &id, incluir_facturas),
            None => Err("vendedorId requerido".into()),
        }));
    }

    fn reimprimir(&mut self, indice: usize, cx: &mut Contexto) {
        if self.impresion.is_some() {
            return;
        }
        let Some(orden) = self.transacciones.get(indice) else {
            return;
        };
        let orden = logica::para_reimprimir(orden, &logica::nombre_negocio(cx.global.admin.as_ref()));
        let doble = cx.global.negocio.as_ref().is_some_and(|n| n["dobleImpresionEnabled"] == true);
        let servicios = cx.servicios.clone();
        self.impresion = Some(Tarea::lanzar(&cx.egui, move || reimprimir(&servicios, &orden, doble)));
    }

    fn recibir(&mut self, cx: &mut Contexto) {
        match esperar(&mut self.carga) {
            Some(Ok(transacciones)) => {
                self.transacciones = transacciones;
                self.pagina = 1;
            }
            Some(Err(_)) => {
                toast::error(&cx.egui, "Error al cargar las órdenes del día");
                self.transacciones.clear();
            }
            None => {}
        }
        if let Some(resultados) = esperar(&mut self.impresion) {
            avisar_reimpresion(&cx.egui, &resultados);
        }
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        self.seguir_usuario(cx);
        self.recibir(cx);
        if !self.abierto {
            return;
        }
        if std::mem::take(&mut self.cargar_al_mostrar) && self.usuario_id.is_some() {
            self.cargar(cx);
        }
        let nombre = cx.global.usuario.as_ref().and_then(|u| u["nombre"].as_str()).unwrap_or_default().to_string();
        let respuesta = Dialogo::new("pedidos").ancho(896.0).mostrar(&cx.egui, |ui| self.contenido(ui, &nombre));
        match respuesta.interior {
            Some(Accion::Reimprimir(indice)) => self.reimprimir(indice, cx),
            Some(Accion::Pagina(pagina)) => self.pagina = pagina,
            Some(Accion::Actualizar) => self.cargar(cx),
            Some(Accion::Cerrar) => self.abierto = false,
            None => {}
        }
        if respuesta.cerrar {
            self.abierto = false;
        }
    }

    fn contenido(&self, ui: &mut Ui, nombre: &str) -> Option<Accion> {
        let mut accion = None;
        let cantidad = self.transacciones.len();
        let cargando = self.carga.is_some();
        let pagina = self.pagina.max(1);
        let paginas = logica::total_paginas(cantidad);
        let (titulo, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::hover());
        ui::texto_en(ui, titulo.left_center(), Align2::LEFT_CENTER, "Transacciones del día", Tam::X2, Peso::Negrita, color_degradado_texto());
        ui.add_space(6.0);
        descripcion(ui, nombre, cantidad);
        ui.add_space(16.0);
        linea_esmeralda(ui);
        ui.add_space(16.0);
        let fijo = 83.0 + 16.0 + 57.0 + if cantidad > 0 { 69.0 + 16.0 } else { 0.0 } + 16.0;
        let alto_maximo = (ui.ctx().content_rect().height() * 0.9 - 48.0 - fijo).max(80.0);
        egui::ScrollArea::vertical().max_height(alto_maximo).min_scrolled_height(alto_maximo).auto_shrink([false, true]).show(ui, |ui| {
            ui.add_space(16.0);
            if cargando {
                cargando_ordenes(ui);
            } else if cantidad == 0 {
                vacio(ui);
            } else if let Some(indice) = self.tabla(ui, pagina) {
                accion = Some(Accion::Reimprimir(indice));
            }
            ui.add_space(16.0);
        });
        ui.add_space(16.0);
        if cantidad > 0 {
            linea_esmeralda(ui);
            ui.add_space(16.0);
            if let Some(destino) = paginacion(ui, pagina, paginas, cantidad) {
                accion = Some(Accion::Pagina(destino));
            }
            ui.add_space(16.0);
        }
        linea_esmeralda(ui);
        ui.add_space(16.0);
        ui::pie_dialogo(ui, |ui| {
            if boton_actualizar(ui, cargando).clicked() {
                accion = Some(Accion::Actualizar);
            }
            if boton_cerrar(ui).clicked() {
                accion = Some(Accion::Cerrar);
            }
        });
        accion
    }

    fn tabla(&self, ui: &mut Ui, pagina: usize) -> Option<usize> {
        let ancho = ui.available_width();
        let interior = ancho - 2.0;
        let total: f32 = PROPORCIONES.iter().sum();
        let anchos: Vec<f32> = PROPORCIONES.iter().map(|p| p / total * interior).collect();
        let desde = (pagina - 1) * logica::POR_PAGINA;
        let visibles: Vec<(usize, &Value)> = self.transacciones.iter().enumerate().skip(desde).take(logica::POR_PAGINA).collect();
        let altos: Vec<f32> = visibles.iter().map(|(_, t)| if verdadero(t.get("tieneDescuento")) { 81.0 } else { 69.0 }).collect();
        let alto = 2.0 + 49.0 + altos.iter().sum::<f32>();
        let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
        let pintor = ui.painter().with_clip_rect(ui.clip_rect().intersect(caja));
        let cabecera = Rect::from_min_size(caja.min + Vec2::splat(1.0), Vec2::new(interior, 48.0));
        pintor.rect_filled(cabecera, egui::CornerRadius { nw: 7, ne: 7, sw: 0, se: 0 }, paleta::EMERALD_50);
        pintor.hline(cabecera.x_range(), cabecera.bottom() + 0.5, ui::borde(color::BORDE));
        let titulos = ["Tipo", "Fecha", "Método de pago", "Descuento", "Total", "Acciones"];
        let alineaciones = [Align2::LEFT_CENTER, Align2::LEFT_CENTER, Align2::LEFT_CENTER, Align2::CENTER_CENTER, Align2::RIGHT_CENTER, Align2::CENTER_CENTER];
        let columnas = |fila: Rect| -> Vec<Rect> {
            let mut x = fila.left();
            anchos
                .iter()
                .map(|w| {
                    let r = Rect::from_min_max(egui::pos2(x, fila.top()), egui::pos2(x + w, fila.bottom()));
                    x += w;
                    r
                })
                .collect()
        };
        for ((celda, titulo), ancla) in columnas(cabecera).iter().zip(titulos).zip(alineaciones) {
            pintor.text(punto(*celda, ancla), ancla, titulo, fuente(Tam::Sm, Peso::Semi), paleta::EMERALD_800);
        }
        let imprimiendo = self.impresion.is_some();
        let mut elegido = None;
        let mut y = cabecera.bottom() + 1.0;
        for (posicion, ((indice, transaccion), alto)) in visibles.iter().zip(&altos).enumerate() {
            let fila = Rect::from_min_size(egui::pos2(cabecera.left(), y), Vec2::new(interior, *alto));
            let contenido = Rect::from_min_max(fila.min, egui::pos2(fila.right(), fila.bottom() - 1.0));
            if posicion % 2 == 1 {
                pintor.rect_filled(contenido, 0, paleta::GRAY_50.gamma_multiply(0.5));
            }
            if posicion + 1 < visibles.len() {
                pintor.hline(fila.x_range(), fila.bottom() - 0.5, ui::borde(color::BORDE));
            }
            let celdas = columnas(contenido);
            let centro = contenido.center().y;
            pintor.text(punto(celdas[0], Align2::LEFT_CENTER), Align2::LEFT_CENTER, logica::texto_tipo(transaccion), fuente(Tam::Sm, Peso::Normal), color::TEXTO);
            let fecha = fecha_argentina(transaccion["fecha"].as_str().unwrap_or_default());
            pintor.text(punto(celdas[1], Align2::LEFT_CENTER), Align2::LEFT_CENTER, fecha, fuente(Tam::Sm, Peso::Medio), paleta::GRAY_900);
            pildora_metodo(ui, &pintor, celdas[2], transaccion);
            match logica::insignia_descuento(transaccion) {
                Some((insignia, subtotal)) => {
                    let alto_bloque = if subtotal.is_some() { 44.0 } else { 24.0 };
                    let arriba = centro - alto_bloque / 2.0;
                    let galeria = ui.painter().layout_no_wrap(insignia, fuente(Tam::Xs, Peso::Medio), paleta::ORANGE_800);
                    let pastilla = Rect::from_center_size(egui::pos2(celdas[3].center().x, arriba + 12.0), Vec2::new(galeria.size().x + 16.0, 24.0));
                    pintor.rect_filled(pastilla, 12, paleta::ORANGE_100);
                    pintor.galley(pastilla.center() - galeria.size() / 2.0, galeria, paleta::ORANGE_800);
                    if let Some(subtotal) = subtotal {
                        pintor.text(egui::pos2(celdas[3].center().x, arriba + 36.0), Align2::CENTER_CENTER, subtotal, fuente(Tam::Xs, Peso::Normal), paleta::GRAY_500);
                    }
                }
                None => {
                    pintor.text(celdas[3].center(), Align2::CENTER_CENTER, "Sin descuento", fuente(Tam::Sm, Peso::Normal), paleta::GRAY_400);
                }
            }
            let derecha = celdas[4].right() - 16.0;
            match logica::linea_descuento(transaccion) {
                Some(linea) => {
                    pintor.text(egui::pos2(derecha, centro - 10.0), Align2::RIGHT_CENTER, logica::total_mostrado(transaccion), fuente(Tam::Lg, Peso::Semi), paleta::EMERALD_700);
                    pintor.text(egui::pos2(derecha, centro + 16.0), Align2::RIGHT_CENTER, linea, fuente(Tam::Xs, Peso::Semi), paleta::GRAY_500);
                }
                None => {
                    pintor.text(egui::pos2(derecha, centro), Align2::RIGHT_CENTER, logica::total_mostrado(transaccion), fuente(Tam::Lg, Peso::Semi), paleta::EMERALD_700);
                }
            }
            if boton_reimprimir(ui, celdas[5], *indice, imprimiendo).clicked() {
                elegido = Some(*indice);
            }
            y += alto;
        }
        pintor.rect_stroke(caja, RADIO, ui::borde(paleta::EMERALD_100), StrokeKind::Inside);
        elegido
    }
}

fn punto(celda: Rect, ancla: Align2) -> egui::Pos2 {
    match ancla {
        Align2::RIGHT_CENTER => egui::pos2(celda.right() - 16.0, celda.center().y),
        Align2::CENTER_CENTER => celda.center(),
        _ => egui::pos2(celda.left() + 16.0, celda.center().y),
    }
}

fn descripcion(ui: &mut Ui, nombre: &str, cantidad: usize) {
    let mut trabajo = egui::text::LayoutJob::default();
    let formato = |tam: Tam, peso: Peso, tinta: Color32| egui::TextFormat { font_id: fuente(tam, peso), color: tinta, ..Default::default() };
    trabajo.append("Todas las órdenes realizadas hoy por ", 0.0, formato(Tam::Lg, Peso::Normal, color::APAGADO_TEXTO));
    trabajo.append(nombre, 0.0, formato(Tam::Lg, Peso::Semi, paleta::EMERALD_700));
    if cantidad > 0 {
        trabajo.append(&format!("({cantidad} transacciones en total)"), 8.0, formato(Tam::Sm, Peso::Normal, paleta::GRAY_600));
    }
    let galeria = ui.ctx().fonts_mut(|f| f.layout_job(trabajo));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
    ui.painter().galley(egui::pos2(rect.left(), rect.center().y - galeria.size().y / 2.0), galeria, color::APAGADO_TEXTO);
}

fn linea_esmeralda(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, paleta::EMERALD_100);
}

fn cargando_ordenes(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 160.0), Sense::hover());
    let galeria = ui.painter().layout_no_wrap("Cargando órdenes del día...".into(), fuente(Tam::Base, Peso::Medio), paleta::EMERALD_700);
    let total = 48.0 + 16.0 + galeria.size().x;
    let izquierda = rect.center().x - total / 2.0;
    let rueda = Rect::from_min_size(egui::pos2(izquierda, rect.center().y - 24.0), Vec2::splat(48.0));
    dibujo::aro_girando(ui, rueda, 4.0, paleta::EMERALD_200, paleta::EMERALD_600);
    ui.painter().galley(egui::pos2(rueda.right() + 16.0, rect.center().y - galeria.size().y / 2.0), galeria, paleta::EMERALD_700);
}

fn vacio(ui: &mut Ui) {
    ui.add_space(48.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 64.0), Sense::hover());
    ui::pintar_icono_en(ui, Rect::from_center_size(rect.center(), Vec2::splat(64.0)), "receipt", paleta::EMERALD_300);
    ui.add_space(16.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
    ui::texto_en(ui, rect.center(), Align2::CENTER_CENTER, "No se encontraron órdenes ni facturas B para el día de hoy", Tam::Lg, Peso::Normal, paleta::GRAY_500);
    ui.add_space(48.0);
}

fn pildora_metodo(ui: &Ui, pintor: &egui::Painter, celda: Rect, transaccion: &Value) {
    let (fondo, tinta) = match transaccion["metodoPago"].as_str() {
        Some("efectivo") => (paleta::GREEN_100, paleta::GREEN_800),
        Some("tarjeta") => (paleta::BLUE_100, paleta::BLUE_800),
        Some("qr") => (paleta::PURPLE_100, paleta::PURPLE_800),
        Some("split") => (paleta::ORANGE_100, paleta::ORANGE_800),
        _ => (paleta::GRAY_100, paleta::GRAY_800),
    };
    let galeria = ui.painter().layout_no_wrap(logica::texto_metodo(transaccion), fuente(Tam::Sm, Peso::Medio), tinta);
    let pastilla = Rect::from_min_size(egui::pos2(celda.left() + 16.0, celda.center().y - 14.0), Vec2::new(galeria.size().x + 24.0, 28.0));
    pintor.rect_filled(pastilla, 14, fondo);
    pintor.galley(pastilla.center() - galeria.size() / 2.0, galeria, tinta);
}

fn boton_reimprimir(ui: &mut Ui, celda: Rect, indice: usize, imprimiendo: bool) -> Response {
    let texto = if imprimiendo { "Reimprimiendo..." } else { "Reimprimir" };
    let galeria = ui.painter().layout_no_wrap(texto.into(), fuente(Tam::Sm, Peso::Medio), paleta::EMERALD_700);
    let ancho = 12.0 + 16.0 + 16.0 + galeria.size().x + 12.0;
    let rect = Rect::from_center_size(celda.center(), Vec2::new(ancho, 36.0));
    let sentido = if imprimiendo { Sense::hover() } else { Sense::click() };
    let respuesta = ui.interact(rect, ui.id().with(("reimprimir", indice)), sentido);
    let opacidad = if imprimiendo { 0.5 } else { 1.0 };
    let fondo = if respuesta.hovered() && !imprimiendo { paleta::EMERALD_100 } else { paleta::EMERALD_50 };
    let tinta = if respuesta.hovered() && !imprimiendo { paleta::EMERALD_800 } else { paleta::EMERALD_700 };
    ui.painter().rect(rect, RADIO, fondo.gamma_multiply(opacidad), ui::borde(paleta::EMERALD_200.gamma_multiply(opacidad)), StrokeKind::Inside);
    let icono = Rect::from_min_size(egui::pos2(rect.left() + 12.0, rect.center().y - 8.0), Vec2::splat(16.0));
    let x_texto = if imprimiendo {
        dibujo::aro_girando(ui, icono, 2.0, paleta::EMERALD_200.gamma_multiply(opacidad), paleta::EMERALD_600.gamma_multiply(opacidad));
        icono.right() + 8.0
    } else {
        ui::pintar_icono_en(ui, icono, "receipt", tinta);
        icono.right() + 16.0
    };
    ui.painter().galley(egui::pos2(x_texto, rect.center().y - galeria.size().y / 2.0), galeria, tinta.gamma_multiply(opacidad));
    if imprimiendo {
        respuesta
    } else {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    }
}

fn boton_pagina(ui: &mut Ui, contenido: Result<&str, usize>, activo: bool, habilitado: bool) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(32.0), if habilitado { Sense::click() } else { Sense::hover() });
    let opacidad = if habilitado { 1.0 } else { 0.5 };
    let encima = habilitado && respuesta.hovered();
    if activo {
        let fondo = if encima { paleta::EMERALD_700 } else { paleta::EMERALD_600 };
        ui.painter().rect_filled(rect, RADIO, fondo.gamma_multiply(opacidad));
    } else {
        let fondo = if encima { color::ACENTO } else { color::FONDO };
        ui.painter().rect(rect, RADIO, fondo.gamma_multiply(opacidad), ui::borde(color::BORDE.gamma_multiply(opacidad)), StrokeKind::Inside);
    }
    let tinta = if activo { Color32::WHITE } else if encima { paleta::EMERALD_600 } else { paleta::GRAY_600 };
    match contenido {
        Ok(icono) => ui::pintar_icono_en(ui, Rect::from_center_size(rect.center(), Vec2::splat(16.0)), icono, color::TEXTO.gamma_multiply(opacidad)),
        Err(numero) => {
            ui::texto_en(ui, rect.center(), Align2::CENTER_CENTER, &numero.to_string(), Tam::Sm, Peso::Medio, tinta.gamma_multiply(opacidad));
        }
    }
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
}

fn paginacion(ui: &mut Ui, pagina: usize, paginas: usize, cantidad: usize) -> Option<usize> {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 52.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIO, paleta::GRAY_50);
    let interior = rect.shrink(16.0);
    ui::texto_en(ui, interior.left_center(), Align2::LEFT_CENTER, &logica::rango_mostrado(pagina, cantidad), Tam::Sm, Peso::Normal, paleta::GRAY_600);
    let derecha = if paginas > 1 { format!("Página {pagina} de {paginas}") } else { "Todas las órdenes".into() };
    let fin = ui::texto_en(ui, interior.right_center(), Align2::RIGHT_CENTER, &derecha, Tam::Sm, Peso::Normal, paleta::GRAY_600);
    if paginas <= 1 {
        return None;
    }
    let inicio_texto = ui.painter().layout_no_wrap(logica::rango_mostrado(pagina, cantidad), fuente(Tam::Sm, Peso::Normal), paleta::GRAY_600).size().x;
    let numeros = logica::numeros_pagina(pagina, paginas);
    let ancho_nav = (numeros.len() + 2) as f32 * 32.0 + (numeros.len() + 1) as f32 * 8.0;
    let hueco_izq = interior.left() + inicio_texto;
    let centro = hueco_izq + (fin.left() - hueco_izq) / 2.0;
    let nav = Rect::from_center_size(egui::pos2(centro, interior.center().y), Vec2::new(ancho_nav, 32.0));
    let mut destino = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(nav).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if boton_pagina(ui, Ok("chevron-left"), false, pagina > 1).clicked() {
            destino = Some(pagina - 1);
        }
        for numero in numeros {
            if boton_pagina(ui, Err(numero), numero == pagina, true).clicked() {
                destino = Some(numero);
            }
        }
        if boton_pagina(ui, Ok("chevron-right"), false, pagina < paginas).clicked() {
            destino = Some(pagina + 1);
        }
    });
    destino
}

fn boton_cerrar(ui: &mut Ui) -> Response {
    ui::Boton::new("Cerrar")
        .variante(ui::Variante::Contorno)
        .colores(color::FONDO, paleta::EMERALD_700)
        .color_borde(paleta::EMERALD_200)
        .ui(ui)
}

fn boton_actualizar(ui: &mut Ui, cargando: bool) -> Response {
    let texto = if cargando { "Cargando..." } else { "Actualizar" };
    let galeria = ui.painter().layout_no_wrap(texto.into(), fuente(Tam::Sm, Peso::Medio), Color32::WHITE);
    let separacion = if cargando { 8.0 } else { 16.0 };
    let ancho = 16.0 + 16.0 + separacion + galeria.size().x + 16.0;
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 40.0), if cargando { Sense::hover() } else { Sense::click() });
    let opacidad = if cargando { 0.5 } else if respuesta.hovered() { 0.9 } else { 1.0 };
    ui::pintar_degradado(
        ui.painter(),
        rect,
        RADIO,
        color::ESMERALDA_DESDE.gamma_multiply(opacidad),
        color::ESMERALDA_HASTA.gamma_multiply(opacidad),
        Direccion::AbajoDerecha,
    );
    let icono = Rect::from_min_size(egui::pos2(rect.left() + 16.0, rect.center().y - 8.0), Vec2::splat(16.0));
    let blanco = Color32::WHITE.gamma_multiply(if cargando { 0.5 } else { 1.0 });
    if cargando {
        dibujo::aro_con_hueco(ui, icono, 2.0, blanco);
    } else {
        ui::pintar_icono_en(ui, icono, "receipt", blanco);
    }
    ui.painter().galley(egui::pos2(icono.right() + separacion, rect.center().y - galeria.size().y / 2.0), galeria, blanco);
    if cargando {
        respuesta
    } else {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    }
}
