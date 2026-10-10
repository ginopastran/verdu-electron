use std::collections::HashMap;

use eframe::egui::{self, Align2, Color32, CursorIcon, Id, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::facturas::{self as logica, DatosFactura, Detalle};
use nucleo::formato;
use serde_json::{json, Value};

use super::comun::{self, Boton, Opcion};
use super::modal::estilo_desplazamiento;
use super::selectores::{CantidadProducto, ListaActiva, ResultadoCantidad, SelectorCliente, SelectorProducto};
use super::{id_texto, texto};
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, Entrada, Peso, Tam};

#[derive(Clone, Debug, PartialEq)]
pub struct ProductoInicial {
    pub id: i64,
    pub nombre: String,
    pub precio: f64,
    pub cantidad: f64,
    pub subtotal: f64,
}

pub enum ResultadoFactura {
    Cerrada,
    Creada(Value),
}

const TIPOS: [(&str, &str); 3] =
    [("A", "Factura A - Responsable Inscripto"), ("C", "Factura C - Monotributista"), ("remito", "Remito - Solo entrega (sin AFIP)")];

struct Fila {
    detalle: Detalle,
    cantidad: String,
    precio: String,
}

impl Fila {
    fn new(detalle: Detalle, iva_en_precios: Option<bool>) -> Self {
        let cantidad = logica::texto_entrada(detalle.cantidad);
        let precio = logica::texto_precio_con_iva(detalle.producto.as_ref(), detalle.precio_unitario, iva_en_precios);
        Fila { detalle, cantidad, precio }
    }
}

enum Respuesta {
    Listas(Vec<Value>),
    Precios(Option<i64>, HashMap<i64, f64>),
    Iniciales(u32, Vec<Detalle>),
    Actualizados(u32, Option<i64>, Vec<Detalle>),
}

#[derive(Default)]
pub struct FormularioFactura {
    abierto: bool,
    apertura: u32,
    iniciales: Vec<ProductoInicial>,
    cargando_iniciales: bool,
    cliente: Option<Value>,
    filas: Vec<Fila>,
    tipo_factura: String,
    observaciones: String,
    pago_inicial: String,
    fecha_vencimiento: String,
    lista_persistida: Option<i64>,
    listas_pedidas: bool,
    listas: Vec<Value>,
    lista_seleccionada: Option<Value>,
    precios_de: Option<i64>,
    precios_lista: HashMap<i64, f64>,
    tareas: Vec<Tarea<Respuesta>>,
    envio: Option<Tarea<Option<Value>>>,
    guardando: bool,
    selector_cliente: SelectorCliente,
    selector_producto: SelectorProducto,
    cantidad: CantidadProducto,
}

fn obtener(servicios: &Servicios, ruta: &str) -> Option<Value> {
    servicios.api.pedir("GET", ruta, None, &[]).ok().filter(|r| r.ok()).map(|r| r.cuerpo)
}

fn id_lista(lista: &Option<Value>) -> Option<i64> {
    lista.as_ref().and_then(|l| l["id"].as_i64())
}

impl FormularioFactura {
    pub fn abrir(&mut self, productos_iniciales: Vec<ProductoInicial>) {
        self.abierto = true;
        self.apertura += 1;
        self.tipo_factura = "remito".into();
        self.observaciones.clear();
        self.filas.clear();
        self.pago_inicial.clear();
        self.fecha_vencimiento.clear();
        self.cliente = None;
        self.lista_seleccionada = self.buscar_lista(self.lista_persistida);
        self.iniciales = productos_iniciales;
        self.cargando_iniciales = false;
        self.guardando = false;
    }

    fn buscar_lista(&self, id: Option<i64>) -> Option<Value> {
        id.and_then(|id| self.listas.iter().find(|l| l["id"].as_i64() == Some(id)).cloned())
    }

    pub fn abierto(&self) -> bool {
        self.abierto || self.envio.is_some()
    }

    fn detalles(&self) -> Vec<Detalle> {
        self.filas.iter().map(|f| f.detalle.clone()).collect()
    }

    fn pago(&self) -> Option<f64> {
        if self.pago_inicial.is_empty() {
            None
        } else {
            logica::parsear_numero(&self.pago_inicial)
        }
    }

    fn lanzar(&mut self, cx: &Contexto, trabajo: impl FnOnce(&Servicios) -> Respuesta + Send + 'static) {
        let servicios = cx.servicios.clone();
        self.tareas.push(Tarea::lanzar(&cx.egui, move || trabajo(&servicios)));
    }

    fn pedir_precios(&mut self, cx: &Contexto) {
        let id = id_lista(&self.lista_seleccionada);
        if self.precios_de == id {
            return;
        }
        self.precios_de = id;
        match id {
            None => self.precios_lista.clear(),
            Some(id) => self.lanzar(cx, move |s| {
                let mut precios = HashMap::new();
                if let Some(items) = obtener(s, &format!("/api/listas-precios/{id}/productos")).and_then(|v| v.as_array().cloned()) {
                    for item in items {
                        let clave = item["productoId"].as_i64().or(item["id"].as_i64());
                        if let (Some(clave), Some(precio)) = (clave, item["precio"].as_f64()) {
                            precios.insert(clave, precio);
                        }
                    }
                }
                Respuesta::Precios(Some(id), precios)
            }),
        }
    }

    fn actualizar_precios(&mut self, cx: &Contexto, lista: Option<i64>) {
        let detalles = self.detalles();
        if detalles.is_empty() {
            return;
        }
        let apertura = self.apertura;
        self.lanzar(cx, move |s| {
            let actualizados = detalles
                .into_iter()
                .map(|mut d| {
                    let base = obtener(s, &format!("/api/productos/{}", d.producto_id))
                        .and_then(|p| p["precio"].as_f64())
                        .unwrap_or(d.precio_unitario);
                    let mut nuevo = base;
                    if let Some(lista) = lista {
                        if let Some(precio) = obtener(s, &format!("/api/listas-precios/{lista}/productos/{}", d.producto_id))
                            .and_then(|v| v["precio"].as_f64())
                            .filter(|p| *p != 0.0)
                        {
                            nuevo = precio;
                        }
                    }
                    d.precio_unitario = nuevo;
                    d.recalcular();
                    d
                })
                .collect();
            Respuesta::Actualizados(apertura, lista, actualizados)
        });
    }

    fn cargar_iniciales(&mut self, cx: &Contexto) {
        self.cargando_iniciales = true;
        let iniciales = std::mem::take(&mut self.iniciales);
        let apertura = self.apertura;
        self.lanzar(cx, move |s| {
            let detalles = iniciales
                .into_iter()
                .map(|inicial| match obtener(s, &format!("/api/productos/{}", inicial.id)) {
                    Some(producto) => {
                        let nombre = texto(&producto, "nombre").to_string();
                        Detalle::nuevo(inicial.id, Some(producto), nombre, inicial.cantidad, inicial.precio)
                    }
                    None => Detalle::nuevo(inicial.id, None, inicial.nombre, inicial.cantidad, inicial.precio),
                })
                .collect();
            Respuesta::Iniciales(apertura, detalles)
        });
    }

    fn recibir(&mut self, cx: &Contexto) {
        let mut listas = Vec::new();
        self.tareas.retain_mut(|t| match t.listo() {
            Some(r) => {
                listas.push(r);
                false
            }
            None => true,
        });
        for respuesta in listas {
            match respuesta {
                Respuesta::Listas(listas) => {
                    self.listas = listas;
                    if self.lista_seleccionada.is_none() {
                        self.lista_seleccionada = self.buscar_lista(self.lista_persistida);
                    }
                }
                Respuesta::Precios(id, precios) => {
                    if id == id_lista(&self.lista_seleccionada) {
                        self.precios_lista = precios;
                        if !self.filas.is_empty() && self.lista_seleccionada.is_some() && !self.precios_lista.is_empty() {
                            self.actualizar_precios(cx, id);
                        }
                    }
                }
                Respuesta::Iniciales(apertura, detalles) => {
                    if apertura == self.apertura {
                        self.cargando_iniciales = false;
                        self.filas = detalles.into_iter().map(|d| Fila::new(d, super::iva_en_precios(cx))).collect();
                        if let Some(id) = id_lista(&self.lista_seleccionada) {
                            self.actualizar_precios(cx, Some(id));
                        }
                    }
                }
                Respuesta::Actualizados(apertura, lista, detalles) => {
                    if apertura == self.apertura && lista == id_lista(&self.lista_seleccionada) {
                        self.filas = detalles.into_iter().map(|d| Fila::new(d, super::iva_en_precios(cx))).collect();
                    }
                }
            }
        }
    }

    fn enviar(&mut self, cx: &Contexto) {
        let Some(cliente) = self.cliente.clone() else {
            ui::toast::error(&cx.egui, "Debe seleccionar un cliente para crear la factura");
            return;
        };
        if self.filas.is_empty() {
            ui::toast::error(&cx.egui, "Debe agregar al menos un producto a la factura");
            return;
        }
        let negocio = cx.global.negocio.clone().unwrap_or(Value::Null);
        let detalles = self.detalles();
        let totales = logica::calcular_totales(&detalles, &self.tipo_factura, negocio["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false));
        let pago = self.pago();
        if pago.is_some_and(|p| p > totales.total) {
            ui::toast::error(&cx.egui, "El pago inicial no puede ser mayor al total");
            return;
        }
        let afip = logica::usar_afip(&negocio, &self.tipo_factura);
        let pago_parcial = pago.is_some_and(|p| p > 0.0 && p < totales.total);
        let vencimiento = match logica::fecha_dia_mes_anio(&self.fecha_vencimiento) {
            _ if !pago_parcial || self.fecha_vencimiento.trim().is_empty() => None,
            Some(fecha) => logica::fecha_iso_desde_input(&fecha.format("%Y-%m-%d").to_string()),
            None => {
                ui::toast::error(&cx.egui, "Ingresá una fecha de vencimiento válida (dd/mm/aaaa)");
                return;
            }
        };
        let datos = DatosFactura {
            cliente_id: cliente["id"].clone(),
            tipo_factura: &self.tipo_factura,
            observaciones: &self.observaciones,
            lista_precio_id: id_lista(&self.lista_seleccionada),
            pago_inicial: pago,
            fecha_vencimiento: vencimiento.as_deref(),
            detalles: &detalles,
        };
        let (ruta, cuerpo) = logica::cuerpo_factura(&datos, afip, totales, &negocio);
        let tipo = self.tipo_factura.clone();
        let observaciones = self.observaciones.clone();
        let servicios = cx.servicios.clone();
        let ctx = cx.egui.clone();
        self.guardando = true;
        self.envio = Some(Tarea::lanzar(&cx.egui, move || {
            let resultado = match servicios.api.pedir("POST", &ruta, Some(&cuerpo), &[]) {
                Ok(r) if r.ok() => r.cuerpo,
                Ok(r) => {
                    let mensaje = r.cuerpo["error"].as_str().unwrap_or("Error al guardar factura").to_string();
                    ui::toast::error(&ctx, mensaje);
                    return None;
                }
                Err(e) => {
                    ui::toast::error(&ctx, e.mensaje);
                    return None;
                }
            };
            if afip && resultado["afip"].is_object() {
                ui::toast::exito(
                    &ctx,
                    format!(
                        "Factura AFIP creada correctamente\nCAE: {}\nVencimiento: {}",
                        super::mostrar_valor(&resultado["afip"]["cae"]),
                        super::mostrar_valor(&resultado["afip"]["vencimientoCae"])
                    ),
                );
            } else {
                ui::toast::exito(&ctx, "Factura creada correctamente");
            }
            let o = |a: &Value, b: &Value| if a.is_null() { b.clone() } else { a.clone() };
            let factura = json!({
                "id": o(&resultado["factura"]["id"], &resultado["id"]),
                "numero": o(&resultado["factura"]["numero"], &resultado["numero"]),
                "tipoFactura": tipo,
                "fecha": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                "cliente": cliente,
                "detalles": detalles.iter().map(Detalle::a_json).collect::<Vec<_>>(),
                "subtotal": totales.subtotal,
                "impuestos": totales.impuestos,
                "total": totales.total,
                "observaciones": observaciones,
                "afip": o(&resultado["afip"], &Value::Null),
                "afipQrUrl": o(&resultado["factura"]["afipQrUrl"], &Value::Null),
            });
            if imprimir_factura(&servicios, &ctx, &factura, &negocio) {
                ui::toast::exito(&ctx, "Ticket impreso correctamente");
            } else {
                ui::toast::error(&ctx, "Error al imprimir el ticket");
            }
            Some(resultado)
        }));
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) -> Option<ResultadoFactura> {
        if let Some(resultado) = esperar(&mut self.envio) {
            self.guardando = false;
            if let Some(valor) = resultado {
                self.abierto = false;
                return Some(ResultadoFactura::Creada(valor));
            }
        }
        if !self.abierto {
            return None;
        }
        if !self.listas_pedidas {
            self.listas_pedidas = true;
            self.lanzar(cx, |s| {
                let listas = obtener(s, "/api/listas-precios?activa=true")
                    .and_then(|v| v["listasPrecios"].as_array().cloned())
                    .unwrap_or_default();
                Respuesta::Listas(listas)
            });
        }
        self.recibir(cx);
        if !self.iniciales.is_empty() && !self.cargando_iniciales {
            self.cargar_iniciales(cx);
        }
        self.pedir_precios(cx);

        let ctx = ui.ctx().clone();
        let alto_max = ctx.content_rect().height() * 0.9;
        let mut accion = Accion::Nada;
        let r = ui::Dialogo::new("factura-formulario").ancho(1152.0).relleno(0.0).mostrar(&ctx, |ui| {
            estilo_desplazamiento(ui);
            comun::desplazable(ui, Id::new("factura-formulario-desplazable"), alto_max - 2.0, |ui| {
                egui::Frame::new().inner_margin(24).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    accion = self.contenido(ui, cx);
                });
            });
        });
        match accion {
            Accion::Nada => {}
            Accion::Cliente => self.selector_cliente.abrir(),
            Accion::Producto => self.selector_producto.abrir(cx),
            Accion::Lista(id) => {
                self.lista_seleccionada = self.buscar_lista(id);
                self.lista_persistida = id;
                self.pedir_precios(cx);
                self.actualizar_precios(cx, id);
            }
            Accion::Enviar => self.enviar(cx),
            Accion::Cancelar => {
                self.abierto = false;
                return Some(ResultadoFactura::Cerrada);
            }
        }
        let seleccionado = self.cliente.as_ref().map(|c| id_texto(&c["id"]));
        if let Some(cliente) = self.selector_cliente.mostrar(&ctx, cx, seleccionado) {
            self.cliente = Some(cliente);
        }
        let excluidos: Vec<i64> = self.filas.iter().map(|f| f.detalle.producto_id).collect();
        let nombre_lista = self.lista_seleccionada.as_ref().map(|l| texto(l, "nombre").to_string());
        let lista = ListaActiva { nombre: nombre_lista, precios: &self.precios_lista };
        if let Some(producto) = self.selector_producto.mostrar(&ctx, cx, &excluidos, lista) {
            let id = producto["id"].as_i64().unwrap_or_default();
            let precio_lista = self.lista_seleccionada.as_ref().and(self.precios_lista.get(&id).copied());
            let precio = logica::precio_con_lista(&producto, precio_lista);
            self.cantidad.abrir(producto, precio, super::iva_en_precios(cx));
        }
        if let Some(ResultadoCantidad::Confirmada(producto, cantidad, precio)) = self.cantidad.mostrar(&ctx) {
            let id = producto["id"].as_i64().unwrap_or_default();
            let nombre = texto(&producto, "nombre").to_string();
            self.filas.push(Fila::new(Detalle::nuevo(id, Some(producto), nombre, cantidad, precio), super::iva_en_precios(cx)));
        }
        if r.cerrar {
            self.abierto = false;
            return Some(ResultadoFactura::Cerrada);
        }
        None
    }

    fn contenido(&mut self, ui: &mut Ui, cx: &Contexto) -> Accion {
        let mut accion = Accion::Nada;
        let negocio = cx.global.negocio.clone().unwrap_or(Value::Null);
        let totales = logica::calcular_totales(&self.detalles(), &self.tipo_factura, negocio["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false));
        comun::titulo_dialogo(ui, "file-text", "Nueva Factura", Color32::from_rgb(0x15, 0x47, 0x37), paleta::EMERALD_900, 12.0);
        ui.add_space(6.0);
        comun::linea(ui, "Complete los datos para crear una nueva factura", Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
        ui.add_space(16.0);
        let ancho = ui.available_width();
        let mitad = (ancho - 24.0) / 2.0;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(mitad);
                comun::etiqueta(ui, "Cliente *");
                if self.boton_cliente(ui, mitad).clicked() {
                    accion = Accion::Cliente;
                }
                if self.cliente.is_none() {
                    ui.add_space(4.0);
                    comun::linea(ui, "* Es obligatorio seleccionar un cliente", Tam::Sm, Peso::Normal, paleta::RED_500);
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.set_width(mitad);
                if cx.global.negocio_cargando {
                    comun::linea(ui, "Cargando...", Tam::Base, Peso::Normal, color::TEXTO);
                } else if !negocio["afipHabilitado"].as_bool().unwrap_or(false) {
                    comun::etiqueta(ui, "Tipo de Factura");
                    let mut cero = 0;
                    comun::selector(ui, "factura-tipo", mitad, &mut cero, &[Opcion { icono: None, texto: TIPOS[2].1.into() }], false);
                    ui.add_space(4.0);
                    comun::linea(ui, "AFIP no está habilitado para este negocio", Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
                } else {
                    comun::etiqueta(ui, "Tipo de Factura");
                    let mut indice = TIPOS.iter().position(|(v, _)| *v == self.tipo_factura).unwrap_or(usize::MAX);
                    let opciones: Vec<Opcion> = TIPOS.iter().map(|(_, t)| Opcion { icono: None, texto: t.to_string() }).collect();
                    if comun::selector(ui, "factura-tipo", mitad, &mut indice, &opciones, true) {
                        self.tipo_factura = TIPOS[indice].0.to_string();
                    }
                }
            });
        });
        ui.add_space(24.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(mitad);
                comun::etiqueta(ui, "Pago inicial (opcional)");
                let r = ui
                    .scope(|ui| Entrada::new(&mut self.pago_inicial).tam(Tam::Sm).sugerencia("0,00").id(Id::new("factura-pago-inicial")).ui(ui))
                    .inner;
                if r.changed() && self.pago().is_none() {
                    self.pago_inicial.clear();
                }
                let pago = self.pago().filter(|p| *p != 0.0);
                if pago.is_some_and(|p| p > totales.total) {
                    comun::linea(ui, "El pago inicial no puede ser mayor al total", Tam::Sm, Peso::Normal, paleta::RED_500);
                }
                if let Some(p) = pago.filter(|p| *p > 0.0) {
                    saldo_pendiente(ui, totales.total - p);
                }
                if pago.is_some_and(|p| p > 0.0 && p < totales.total) {
                    ui.add_space(16.0);
                    comun::etiqueta(ui, "Fecha de vencimiento (opcional)");
                    ui.scope(|ui| {
                        Entrada::new(&mut self.fecha_vencimiento)
                            .tam(Tam::Sm)
                            .sugerencia("dd/mm/aaaa")
                            .icono("calendar")
                            .id(Id::new("factura-vencimiento"))
                            .ui(ui)
                    });
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.set_width(mitad);
                comun::etiqueta(ui, "Lista de Precios (opcional)");
                let mut opciones = vec![Opcion { icono: None, texto: "Sin lista específica (precios base)".into() }];
                let mut ids = vec![None];
                for lista in &self.listas {
                    let cantidad = super::mostrar_valor(&lista["_count"]["productos"]);
                    opciones.push(Opcion { icono: None, texto: format!("{} ({cantidad} productos)", texto(lista, "nombre")) });
                    ids.push(lista["id"].as_i64());
                }
                let actual = id_lista(&self.lista_seleccionada);
                let mut indice = ids.iter().position(|i| *i == actual).unwrap_or(0);
                if comun::selector(ui, "factura-lista", mitad, &mut indice, &opciones, true) {
                    accion = Accion::Lista(ids[indice]);
                }
                ui.add_space(4.0);
                let ayuda = match &self.lista_seleccionada {
                    Some(l) => format!(
                        "Usando precios de \"{}\". Si un producto no está en esta lista, se usará el precio base.",
                        texto(l, "nombre")
                    ),
                    None => "Se usarán los precios base de los productos.".into(),
                };
                comun::linea(ui, ayuda, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
            });
        });
        ui.add_space(24.0);
        caja_iva(ui);
        ui.add_space(24.0);
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ancho, 36.0), Sense::hover());
        comun::texto_en(ui, egui::pos2(fila.left(), fila.center().y), Align2::LEFT_CENTER, "Productos", Tam::Lg, Peso::Semi, color::TEXTO);
        let agregar = Boton::esmeralda("Agregar Producto").chico().icono("plus").separacion(16.0).borde(Some(color::BORDE));
        let w = agregar.ancho_natural(ui);
        if agregar.en(ui, Rect::from_min_size(egui::pos2(fila.right() - w, fila.top()), Vec2::new(w, 36.0)), "agregar-producto").clicked() {
            accion = Accion::Producto;
        }
        if !self.filas.is_empty() {
            ui.add_space(16.0);
            self.tabla(ui, super::iva_en_precios(cx));
            ui.add_space(16.0);
            resumen(ui, &self.tipo_factura, totales);
        }
        ui.add_space(24.0);
        comun::etiqueta(ui, "Observaciones");
        ui.add_space(8.0);
        ui.scope(|ui| {
            Entrada::new(&mut self.observaciones)
                .multilinea(80.0)
                .tam(Tam::Sm)
                .sugerencia("Observaciones adicionales...")
                .id(Id::new("factura-observaciones"))
                .ui(ui)
        });
        ui.add_space(24.0);
        comun::borde_superior(ui);
        ui.add_space(16.0);
        ui::pie_dialogo(ui, |ui| {
            let texto_crear = if self.guardando { "Guardando..." } else { "Crear Factura" };
            if Boton::esmeralda(texto_crear).icono("save").separacion(16.0).habilitado(!self.guardando).ui(ui).clicked() {
                accion = Accion::Enviar;
            }
            ui.add_space(4.0);
            if Boton::contorno("Cancelar").icono("x").separacion(16.0).ui(ui).clicked() {
                accion = Accion::Cancelar;
            }
        });
        accion
    }

    fn boton_cliente(&self, ui: &mut Ui, ancho: f32) -> egui::Response {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 66.0), Sense::hover());
        let respuesta = Boton::contorno("").borde(Some(color::BORDE_ENTRADA)).en(ui, rect, "boton-cliente");
        let x = rect.left() + 13.0;
        let (principal, secundario, color_principal) = match &self.cliente {
            Some(c) => {
                let cuit = texto(c, "cuit");
                (logica::nombre_cliente(c), (!cuit.is_empty()).then(|| format!("CUIT: {cuit}")), color::TEXTO)
            }
            None => ("Seleccionar cliente".into(), Some("Debe elegir un cliente específico".into()), color::APAGADO_TEXTO),
        };
        let alto = if secundario.is_some() { 40.0 } else { 20.0 };
        let arriba = rect.center().y - alto / 2.0;
        comun::texto_en(ui, egui::pos2(x, arriba + 10.0), Align2::LEFT_CENTER, &principal, Tam::Sm, Peso::Medio, color_principal);
        if let Some(s) = secundario {
            comun::texto_en(ui, egui::pos2(x, arriba + 30.0), Align2::LEFT_CENTER, &s, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
        }
        let derecha = rect.right() - 13.0;
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(derecha - 8.0, rect.center().y), Vec2::splat(16.0)), "chevron-down", color::TEXTO);
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(derecha - 32.0, rect.center().y), Vec2::splat(16.0)), "user", color::TEXTO);
        respuesta
    }

    fn tabla(&mut self, ui: &mut Ui, iva_en_precios: Option<bool>) {
        let ancho = ui.available_width();
        let interior = ancho - 2.0;
        let fijos = [interior * 305.0 / 1085.0, interior * 356.0 / 1085.0, 120.0, 120.0, 120.0];
        let ultima = interior - fijos.iter().sum::<f32>();
        let anchos = [fijos[0], fijos[1], fijos[2], fijos[3], fijos[4], ultima];
        let lista = self.lista_seleccionada.as_ref().map(|l| texto(l, "nombre").to_string());
        let alto_fila = |f: &Fila, precios: &HashMap<i64, f64>| {
            if lista.is_some() && precios.contains_key(&f.detalle.producto_id) {
                104.0
            } else {
                72.0
            }
        };
        let alto_filas: f32 = self.filas.iter().map(|f| alto_fila(f, &self.precios_lista)).sum::<f32>() + self.filas.len().saturating_sub(1) as f32;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 2.0 + 48.0 + 1.0 + alto_filas), Sense::hover());
        ui.painter().rect_stroke(rect, 12, Stroke::new(1.0, color::BORDE_ENTRADA), StrokeKind::Inside);
        let mut x = rect.left() + 1.0;
        let columnas: Vec<f32> = anchos
            .iter()
            .map(|w| {
                let inicio = x;
                x += w;
                inicio
            })
            .collect();
        let mut y = rect.top() + 1.0;
        for (i, e) in ["Producto", "Descripción", "Cantidad", "P. Unitario", "Subtotal"].iter().enumerate() {
            comun::texto_en(ui, egui::pos2(columnas[i] + 16.0, y + 24.0), Align2::LEFT_CENTER, e, Tam::Sm, Peso::Medio, color::APAGADO_TEXTO);
        }
        y += 48.0;
        ui.painter().hline(rect.left()..=rect.right(), y + 0.5, Stroke::new(1.0, color::BORDE));
        y += 1.0;
        let mut borrar = None;
        let precios = self.precios_lista.clone();
        let cantidad_filas = self.filas.len();
        for (i, fila) in self.filas.iter_mut().enumerate() {
            let alto = alto_fila(fila, &precios);
            let caja = Rect::from_min_size(egui::pos2(rect.left() + 1.0, y), Vec2::new(interior, alto));
            if ui.rect_contains_pointer(caja) {
                ui.painter().rect_filled(caja, 0, color::APAGADO.gamma_multiply(0.5));
            }
            let d = &mut fila.detalle;
            let nombre = d.producto.as_ref().map(|p| texto(p, "nombre").to_string()).unwrap_or_else(|| format!("Producto {}", d.producto_id));
            let medida = d.producto.as_ref().map(|p| texto(p, "tipoMedida").to_string()).unwrap_or_default();
            let tx = columnas[0] + 16.0;
            comun::texto_en(ui, egui::pos2(tx, y + 26.0), Align2::LEFT_CENTER, &nombre, Tam::Sm, Peso::Medio, color::TEXTO);
            comun::texto_en(ui, egui::pos2(tx, y + 46.0), Align2::LEFT_CENTER, &medida, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
            if let (Some(lista), true) = (&lista, precios.contains_key(&d.producto_id)) {
                let w = 8.0 + 16.0 + 4.0 + comun::ancho_texto(ui, &format!(" {lista}"), Tam::Xs, Peso::Medio) + 8.0;
                let chip = Rect::from_min_size(egui::pos2(tx, y + 64.0), Vec2::new(w, 24.0));
                ui.painter().add(ui::sombra_sm().as_shape(chip, 12));
                comun::pintar_relleno(ui, chip, 12.0, comun::esmeralda(), 1.0);
                ui::pintar_icono_en(ui, Rect::from_min_size(chip.min + Vec2::new(8.0, 4.0), Vec2::splat(16.0)), "file-text", Color32::WHITE);
                comun::texto_en(ui, egui::pos2(chip.left() + 28.0, chip.center().y), Align2::LEFT_CENTER, &format!(" {lista}"), Tam::Xs, Peso::Medio, Color32::WHITE);
            }
            let centro = y + alto / 2.0;
            let celda = |c: usize| Rect::from_min_max(egui::pos2(columnas[c] + 16.0, centro - 20.0), egui::pos2(columnas[c] + anchos[c] - 16.0, centro + 20.0));
            let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(celda(1)));
            Entrada::new(&mut d.descripcion).tam(Tam::Sm).id(Id::new(("factura-desc", i))).ui(&mut hijo);
            let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(celda(2)));
            if Entrada::new(&mut fila.cantidad).tam(Tam::Sm).id(Id::new(("factura-cant", i))).ui(&mut hijo).changed() {
                d.cantidad = logica::parsear_numero(&fila.cantidad).unwrap_or(0.0);
                d.recalcular();
            }
            let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(celda(3)));
            if Entrada::new(&mut fila.precio).tam(Tam::Sm).id(Id::new(("factura-precio", i))).ui(&mut hijo).changed() {
                d.precio_unitario = logica::precio_sin_iva(d.producto.as_ref(), logica::parsear_numero(&fila.precio).unwrap_or(0.0), iva_en_precios);
                d.recalcular();
            }
            comun::texto_en(ui, egui::pos2(celda(4).right(), centro), Align2::RIGHT_CENTER, &formato::pesos(logica::precio_con_iva(d.producto.as_ref(), d.subtotal, iva_en_precios)), Tam::Sm, Peso::Medio, color::TEXTO);
            let papelera = Rect::from_min_size(egui::pos2(columnas[5] + 16.0, centro - 16.0), Vec2::splat(32.0));
            let r = ui.interact(papelera, Id::new(("factura-borrar", i)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            if r.hovered() {
                ui.painter().rect_filled(papelera, 8, color::ACENTO);
            }
            ui::pintar_icono_en(ui, Rect::from_center_size(papelera.center(), Vec2::splat(16.0)), "trash-2", color::DESTRUCTIVO);
            if r.clicked() {
                borrar = Some(i);
            }
            y += alto;
            if i + 1 < cantidad_filas {
                ui.painter().hline(rect.left()..=rect.right(), y + 0.5, Stroke::new(1.0, color::BORDE));
                y += 1.0;
            }
        }
        if let Some(i) = borrar {
            self.filas.remove(i);
        }
    }
}

enum Accion {
    Nada,
    Cliente,
    Producto,
    Lista(Option<i64>),
    Enviar,
    Cancelar,
}

fn imprimir_factura(servicios: &Servicios, ctx: &egui::Context, factura: &Value, negocio: &Value) -> bool {
    let datos = logica::datos_impresion(factura, negocio);
    if negocio["dobleImpresionEnabled"].as_bool() == Some(true) {
        let primera = imprimir(servicios, Comprobante::Factura, datos.clone());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let segunda = imprimir(servicios, Comprobante::Factura, datos);
        let ok = |r: &Value| r["success"].as_bool() == Some(true);
        if ok(&primera) && ok(&segunda) {
            ui::toast::con_detalle(ctx, ui::toast::Tipo::Exito, "Doble ticket de factura impreso correctamente", "Se mandaron 2 impresiones del ticket de factura", 2.0);
        } else {
            let estado = |r: &Value| if ok(r) { "OK" } else { "Error" };
            let detalle = format!("Primera: {}, Segunda: {}", estado(&primera), estado(&segunda));
            ui::toast::con_detalle(ctx, ui::toast::Tipo::Aviso, "Problemas con la doble impresión de factura", detalle, 2.0);
        }
        return true;
    }
    imprimir(servicios, Comprobante::Factura, datos)["success"].as_bool() == Some(true)
}

fn saldo_pendiente(ui: &mut Ui, saldo: f64) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
    let r = comun::texto_en(ui, egui::pos2(rect.left(), rect.center().y), Align2::LEFT_CENTER, "Saldo pendiente:", Tam::Sm, Peso::Negrita, paleta::BLUE_600);
    comun::texto_en(ui, egui::pos2(r.right() + 4.0, rect.center().y), Align2::LEFT_CENTER, &formato::pesos(saldo), Tam::Sm, Peso::Normal, paleta::BLUE_600);
}

fn caja_iva(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 110.0), Sense::hover());
    ui.painter().rect(rect, 16, paleta::BLUE_50, Stroke::new(1.0, paleta::BLUE_200), StrokeKind::Inside);
    let x = rect.left() + 17.0;
    let mut y = rect.top() + 17.0;
    ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x + 9.0, y + 12.0), Vec2::splat(16.0)), "bar-chart-3", paleta::BLUE_800);
    comun::texto_en(ui, egui::pos2(x + 24.0, y + 12.0), Align2::LEFT_CENTER, "Cálculo Automático de IVA", Tam::Base, Peso::Medio, paleta::BLUE_800);
    y += 32.0;
    for (negrita, resto) in [("IVA automático:", " Se calcula según la configuración de cada producto"), ("Alícuotas soportadas:", " 0%, 10.5%, 21% y 27%")] {
        let r = comun::texto_en(ui, egui::pos2(x, y + 10.0), Align2::LEFT_CENTER, "• ", Tam::Sm, Peso::Normal, paleta::BLUE_700);
        let r = comun::texto_en(ui, egui::pos2(r.right(), y + 10.0), Align2::LEFT_CENTER, negrita, Tam::Sm, Peso::Negrita, paleta::BLUE_700);
        comun::texto_en(ui, egui::pos2(r.right(), y + 10.0), Align2::LEFT_CENTER, resto, Tam::Sm, Peso::Normal, paleta::BLUE_700);
        y += 24.0;
    }
}

fn resumen(ui: &mut Ui, tipo: &str, totales: logica::Totales) {
    let con_iva = tipo != "remito" && totales.impuestos > 0.0;
    let alto = 1.0 + 16.0 + 24.0 + 12.0 + 24.0 + if con_iva { 32.0 } else { 0.0 } + 8.0 + 1.0 + 8.0 + 28.0 + 16.0 + 1.0;
    let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    let rect = Rect::from_min_max(egui::pos2(fila.right() - 320.0, fila.top()), fila.max);
    ui.painter().rect(rect, 8, color::APAGADO.gamma_multiply(0.3), Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let x0 = rect.left() + 17.0;
    let x1 = rect.right() - 17.0;
    let mut y = rect.top() + 17.0;
    ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x0 + 8.0, y + 12.0), Vec2::splat(16.0)), "calculator", color::TEXTO);
    comun::texto_en(ui, egui::pos2(x0 + 24.0, y + 12.0), Align2::LEFT_CENTER, "Resumen", Tam::Base, Peso::Semi, color::TEXTO);
    y += 36.0;
    let mut fila_total = |etiqueta: &str, valor: f64| {
        comun::texto_en(ui, egui::pos2(x0, y + 12.0), Align2::LEFT_CENTER, etiqueta, Tam::Base, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(x1, y + 12.0), Align2::RIGHT_CENTER, &formato::pesos(valor), Tam::Base, Peso::Normal, color::TEXTO);
        y += 32.0;
    };
    fila_total("Subtotal:", totales.subtotal);
    if con_iva {
        fila_total("IVA:", totales.impuestos);
    }
    ui.painter().hline(x0..=x1, y + 0.5, Stroke::new(1.0, color::BORDE));
    y += 9.0;
    comun::texto_en(ui, egui::pos2(x0, y + 14.0), Align2::LEFT_CENTER, "Total:", Tam::Lg, Peso::Negrita, color::TEXTO);
    comun::texto_en(ui, egui::pos2(x1, y + 14.0), Align2::RIGHT_CENTER, &formato::pesos(totales.total), Tam::Lg, Peso::Negrita, color::TEXTO);
}
