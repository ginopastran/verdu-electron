use std::time::Duration;

use eframe::egui::{self, Align, Align2, Color32, CursorIcon, Frame, Layout, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2};
use nucleo::cierre::{ahora_argentina, fecha_literal, id_texto, numero, verdadero};
use nucleo::pedidos as logica;
use serde_json::Value;

use super::cierre::dibujo::{dinero, hora_corta, metodo_pago};
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, fuente, radio, tam};
use crate::ui::v2::{self, chip_metodo, dato_compacto, tarjetas::celda_texto, toast, Boton, Columna, Tabla};
use crate::ui::{pintar_icono_solar_en, Peso};

const COLUMNAS: [Columna; 6] = [
    Columna::fija(110.0),
    Columna::fija(80.0),
    Columna::fija(130.0),
    Columna::fraccion(1.0, 0.0),
    Columna::fija(130.0).derecha(),
    Columna::fija(130.0),
];

#[derive(Default)]
pub struct DialogoPedidos {
    abierto: bool,
    cargar_al_mostrar: bool,
    transacciones: Vec<Value>,
    pagina: usize,
    usuario_id: Option<String>,
    imprimiendo: Option<usize>,
    carga: Option<Tarea<Result<Vec<Value>, String>>>,
    impresion: Option<Tarea<Vec<Value>>>,
}

enum Accion {
    Reimprimir(usize),
    Pagina(usize),
    Actualizar,
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
    Ok(logica::del_dia(todas, ahora_argentina().date()))
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
                toast::mostrar_aviso(ctx, toast::Tipo::Exito, "Doble ticket impreso correctamente", Some("Se mandaron 2 impresiones al script PHP".into()), 2.0);
            } else {
                let estado = |r: &Value| if exito(r) { "OK" } else { "Error" };
                toast::mostrar_aviso(
                    ctx,
                    toast::Tipo::Aviso,
                    "Problemas con la doble impresión",
                    Some(format!("Primera: {}, Segunda: {}", estado(primera), estado(segunda))),
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

pub fn texto_descuento(transaccion: &Value) -> Option<String> {
    let (insignia, _) = logica::insignia_descuento(transaccion)?;
    Some(if verdadero(transaccion.get("montoDescuento")) {
        format!("{insignia} · −{}", dinero(numero(transaccion.get("montoDescuento"))))
    } else {
        insignia
    })
}

pub fn texto_total(transaccion: &Value) -> String {
    let total = logica::total_mostrado(transaccion);
    total.strip_suffix(",00").map(str::to_string).unwrap_or(total)
}

pub fn rango_visible(pagina: usize, cantidad: usize) -> String {
    let desde = (pagina - 1) * logica::POR_PAGINA;
    format!("Mostrando {}–{} de {cantidad}", desde + 1, (desde + logica::POR_PAGINA).min(cantidad))
}

fn hora(transaccion: &Value) -> String {
    transaccion["fecha"].as_str().and_then(fecha_literal).map(hora_corta).unwrap_or_else(|| "—".into())
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
        self.imprimiendo = Some(indice);
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
        let subtitulo = format!("Órdenes y facturas B de {nombre} · hoy");
        let respuesta = v2::Modal::new("pedidos", "Transacciones del día")
            .subtitulo(&subtitulo)
            .icono("history")
            .ancho(960.0)
            .mostrar(&cx.egui, |ui| self.contenido(ui));
        match respuesta.interior {
            Some(Accion::Reimprimir(indice)) => self.reimprimir(indice, cx),
            Some(Accion::Pagina(pagina)) => self.pagina = pagina,
            Some(Accion::Actualizar) => self.cargar(cx),
            None => {}
        }
        if respuesta.cerrar {
            self.abierto = false;
        }
    }

    fn contenido(&self, ui: &mut Ui) -> Option<Accion> {
        let mut accion = None;
        let cantidad = self.transacciones.len();
        let cargando = self.carga.is_some();
        let pagina = self.pagina.max(1);
        let paginas = logica::total_paginas(cantidad);
        v2::cuerpo_modal(ui, 18, |ui| {
            self.estadisticas(ui);
            ui.add_space(12.0);
            if cargando {
                cargando_transacciones(ui);
            } else if cantidad == 0 {
                vacio(ui);
            } else if let Some(indice) = self.tabla(ui, pagina) {
                accion = Some(Accion::Reimprimir(indice));
            }
        });
        Frame::NONE.inner_margin(egui::Margin { left: 24, right: 24, top: 14, bottom: 22 }).show(ui, |ui| {
            let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::hover());
            if cantidad > 0 {
                v2::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, rango_visible(pagina, cantidad), tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
            }
            let mut derecha = ui.new_child(UiBuilder::new().max_rect(fila).layout(Layout::right_to_left(Align::Center)));
            derecha.spacing_mut().item_spacing.x = 6.0;
            let actualizar = Boton::primario("Actualizar")
                .icono("loader")
                .alto(36.0)
                .radio(radio::CONTROL)
                .relleno(14.0)
                .tam_texto(tam::CONTROL)
                .tam_icono(16.0)
                .sin_sombra()
                .cargando(cargando)
                .ui(&mut derecha);
            if actualizar.clicked() {
                accion = Some(Accion::Actualizar);
            }
            if paginas > 1 {
                derecha.add_space(8.0);
                if let Some(destino) = paginacion(&mut derecha, pagina, paginas) {
                    accion = Some(Accion::Pagina(destino));
                }
            }
        });
        accion
    }

    fn estadisticas(&self, ui: &mut Ui) {
        let cantidad = self.transacciones.len();
        let total: f64 = self.transacciones.iter().map(|t| numero(t.get("total"))).filter(|n| n.is_finite()).sum();
        let promedio = if cantidad > 0 { dinero(total / cantidad as f64) } else { "—".into() };
        let datos = [("Transacciones", cantidad.to_string()), ("Total vendido", dinero(total)), ("Ticket promedio", promedio)];
        let ancho = (ui.available_width() - 20.0) / 3.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            for (etiqueta, valor) in datos {
                dato_compacto(ui, etiqueta, &valor, ancho);
            }
        });
    }

    fn tabla(&self, ui: &mut Ui, pagina: usize) -> Option<usize> {
        let tabla = Tabla::new(&COLUMNAS).relleno(16.0).radio_superior(radio::POPOVER - 1);
        let desde = (pagina - 1) * logica::POR_PAGINA;
        let imprimiendo = self.impresion.is_some();
        let mut elegido = None;
        let marco = Frame::NONE.corner_radius(radio::POPOVER).show(ui, |ui| {
            ui.set_width(ui.available_width());
            tabla.encabezado(ui, &["Tipo", "Hora", "Método", "Descuento", "Total", ""], 40.0);
            for (posicion, (indice, transaccion)) in self.transacciones.iter().enumerate().skip(desde).take(logica::POR_PAGINA).enumerate() {
                let fondo = (posicion % 2 == 1).then_some(color::FONDO_ENCABEZADO_TABLA);
                tabla.fila(ui, 54.0, fondo, |ui, celdas| {
                    let pintor = ui.painter().clone();
                    celda_texto(&pintor, celdas[0], logica::texto_tipo(transaccion), tam::CUERPO, Peso::Medio, color::TEXTO, false);
                    celda_texto(&pintor, celdas[1], &hora(transaccion), tam::CUERPO, Peso::Normal, color::TEXTO_SECUNDARIO, false);
                    let clave = transaccion["metodoPago"].as_str().map(str::to_string).unwrap_or_else(|| logica::texto_metodo(transaccion));
                    let (etiqueta, icono, tono) = metodo_pago(&clave);
                    let mut chip = ui.new_child(UiBuilder::new().max_rect(celdas[2]).layout(Layout::left_to_right(Align::Center)));
                    chip_metodo(&mut chip, icono, &etiqueta, tono);
                    match texto_descuento(transaccion) {
                        Some(descuento) => celda_texto(&pintor, celdas[3], &descuento, tam::ETIQUETA, Peso::Normal, color::DORADO_TEXTO, false),
                        None => celda_texto(&pintor, celdas[3], "Sin descuento", tam::ETIQUETA, Peso::Normal, color::TEXTO_DESHABILITADO, false),
                    }
                    celda_texto(&pintor, celdas[4], &texto_total(transaccion), tam::CUERPO, Peso::Semi, color::TEXTO, true);
                    let girando = imprimiendo && self.imprimiendo == Some(indice);
                    if boton_reimprimir(ui, celdas[5], indice, imprimiendo, girando).clicked() {
                        elegido = Some(indice);
                    }
                });
            }
        });
        ui.painter().rect_stroke(marco.response.rect, radio::POPOVER, Stroke::new(1.0, color::BORDE_SUAVE), StrokeKind::Inside);
        elegido
    }
}

fn cargando_transacciones(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 280.0), Sense::hover());
    let g = v2::galeria(ui, "Cargando transacciones…", fuente(tam::CUERPO, Peso::Normal), color::VERDE);
    let izquierda = rect.center().x - (24.0 + 12.0 + g.size().x) / 2.0;
    v2::girando(ui, Rect::from_min_size(egui::pos2(izquierda, rect.center().y - 12.0), Vec2::splat(24.0)), color::VERDE);
    ui.painter().galley(egui::pos2(izquierda + 36.0, rect.center().y - g.size().y / 2.0), g, color::VERDE);
}

fn vacio(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 240.0), Sense::hover());
    let g = v2::galeria(ui, "No hay órdenes ni facturas B registradas hoy", fuente(tam::CUERPO_MAS, Peso::Normal), color::TEXTO_APAGADO);
    let arriba = rect.center().y - (38.0 + 10.0 + g.size().y) / 2.0;
    pintar_icono_solar_en(ui, Rect::from_min_size(egui::pos2(rect.center().x - 19.0, arriba), Vec2::splat(38.0)), "receipt", color::SOBRE_OSCURO_APAGADO);
    ui.painter().galley(egui::pos2(rect.center().x - g.size().x / 2.0, arriba + 48.0), g, color::TEXTO_APAGADO);
}

fn boton_reimprimir(ui: &mut Ui, celda: Rect, indice: usize, imprimiendo: bool, girando: bool) -> Response {
    let texto = if girando { "Imprimiendo…" } else { "Reimprimir" };
    let g = v2::galeria(ui, texto, fuente(tam::CHICO, Peso::Medio), color::VERDE);
    let ancho = 10.0 + 15.0 + 6.0 + g.size().x + 10.0;
    let rect = Rect::from_min_size(egui::pos2(celda.right() - ancho, celda.center().y - 16.0), Vec2::new(ancho, 32.0));
    let respuesta = ui.interact(rect, ui.id().with(("reimprimir", indice)), if imprimiendo { Sense::hover() } else { Sense::click() });
    let encima = !imprimiendo && respuesta.hovered();
    let (fondo, borde) = if encima { (color::VERDE_ELEGIDO, color::VERDE) } else { (color::BLANCO, color::BORDE) };
    ui.painter().rect(rect, radio::ICONO, fondo, Stroke::new(1.0, borde), StrokeKind::Inside);
    let icono = Rect::from_min_size(egui::pos2(rect.left() + 10.0, rect.center().y - 7.5), Vec2::splat(15.0));
    if girando {
        v2::girando(ui, icono, color::VERDE);
    } else {
        pintar_icono_solar_en(ui, icono, "printer", color::VERDE);
    }
    ui.painter().galley(egui::pos2(icono.right() + 6.0, rect.center().y - g.size().y / 2.0), g, color::VERDE);
    if imprimiendo {
        respuesta
    } else {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    }
}

fn boton_pagina(ui: &mut Ui, contenido: Result<&str, usize>, activo: bool, habilitado: bool) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(34.0), if habilitado { Sense::click() } else { Sense::hover() });
    let alfa = if habilitado { 1.0 } else { 0.4 };
    let encima = habilitado && respuesta.hovered();
    match contenido {
        Ok(icono) => {
            let borde = if encima { color::VERDE } else { color::BORDE };
            ui.painter().rect(rect, radio::ICONO, Color32::TRANSPARENT, Stroke::new(1.0, borde.gamma_multiply(alfa)), StrokeKind::Inside);
            pintar_icono_solar_en(ui, Rect::from_center_size(rect.center(), Vec2::splat(16.0)), icono, color::TEXTO.gamma_multiply(alfa));
        }
        Err(numero) => {
            let (fondo, tinta, peso) = if activo {
                (color::VERDE, color::BLANCO, Peso::Semi)
            } else if encima {
                (color::FONDO_APP, color::VERDE, Peso::Normal)
            } else {
                (Color32::TRANSPARENT, color::TEXTO_SECUNDARIO, Peso::Normal)
            };
            ui.painter().rect_filled(rect, radio::ICONO, fondo);
            v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, numero.to_string(), tam::CONTROL, peso, tinta);
        }
    }
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
}

fn paginacion(ui: &mut Ui, pagina: usize, paginas: usize) -> Option<usize> {
    let mut destino = None;
    if boton_pagina(ui, Ok("chevron-right"), false, pagina < paginas).clicked() {
        destino = Some(pagina + 1);
    }
    for numero in logica::numeros_pagina(pagina, paginas).into_iter().rev() {
        if boton_pagina(ui, Err(numero), numero == pagina, true).clicked() {
            destino = Some(numero);
        }
    }
    if boton_pagina(ui, Ok("chevron-left"), false, pagina > 1).clicked() {
        destino = Some(pagina - 1);
    }
    destino
}

#[cfg(test)]
mod pruebas {
    use serde_json::json;

    use super::*;

    #[test]
    fn textos_de_la_tabla_como_el_diseno() {
        let con_descuento = json!({ "total": 2150, "tieneDescuento": true, "tipoDescuento": "porcentual", "valorDescuento": 10, "montoDescuento": 239, "subtotalSinDescuento": 2389 });
        assert_eq!(texto_descuento(&con_descuento).as_deref(), Some("10% · −$239"));
        assert_eq!(texto_total(&con_descuento), "$2.150");
        assert_eq!(texto_descuento(&json!({ "total": 4380 })), None);
        assert_eq!(texto_total(&json!({ "total": 4380.5 })), "$4.380,50");
        assert_eq!(rango_visible(1, 7), "Mostrando 1–6 de 7");
        assert_eq!(rango_visible(2, 7), "Mostrando 7–7 de 7");
        assert_eq!(hora(&json!({ "fecha": "2026-10-10T08:14:00.000Z" })), "08:14 a. m.");
    }
}
