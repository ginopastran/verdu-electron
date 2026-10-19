use std::collections::HashMap;

use eframe::egui::{self, Align, Align2, CornerRadius, Id, Key, Layout, Rect, Response, ScrollArea, Sense, Stroke, Ui, UiBuilder, Vec2};
use nucleo::facturas::{self as logica, Credito, Cuenta, DatosFactura, Detalle};
use nucleo::formato;
use serde_json::{json, Value};

use super::credito::{self, Autorizacion};
use super::selectores::{self, Catalogo, Clientes, Menu, VistaProductos};
use super::texto;
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{self, color, fuente, radio, tam};
use crate::ui::v2::{self, Boton, BotonIcono, CampoTexto, Contenedor, Estilo, EstiloSegmentado, Segmento};
use crate::ui::{self, pintar_icono_solar, pintar_icono_solar_en, Peso};

#[derive(Clone, Debug, PartialEq)]
pub struct ProductoInicial {
    pub id: i64,
    pub nombre: String,
    pub precio: f64,
    pub cantidad: f64,
    pub subtotal: f64,
    pub precio_libre: bool,
}

pub enum ResultadoFactura {
    Cerrada,
    Creada(Value),
}

const TIPOS: [(&str, &str); 3] = [("remito", "Remito"), ("A", "Factura A"), ("C", "Factura C")];
const ALTO_FILA: f32 = 54.0;
const ANCHO_MODAL: f32 = 760.0;
const ID_CLIENTE: &str = "factura-cliente";
const ID_PRODUCTO: &str = "factura-producto-buscar";
const ID_CANTIDAD: &str = "factura-producto-cantidad";

type ClaveFiltro = (String, Vec<i64>, u32);

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
    cliente_texto: String,
    clientes: Clientes,
    menu_cliente: Menu,
    catalogo: Catalogo,
    filtro: Option<(ClaveFiltro, Vec<usize>)>,
    menu_producto: Menu,
    producto_texto: String,
    producto_elegido: Option<Value>,
    cantidad_texto: String,
    error_agregar: bool,
    enfocar: Option<&'static str>,
    desde_orden: bool,
    intento: bool,
    credito_de: Option<Value>,
    cuenta: Option<Cuenta>,
    tarea_cuenta: Option<Tarea<(Value, Option<Cuenta>)>>,
    enviar_con_cuenta: bool,
    autorizacion: Autorizacion,
    autorizado: Option<(Value, f64, String)>,
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
        self.desde_orden = !self.iniciales.is_empty();
        self.intento = false;
        self.cliente_texto.clear();
        self.clientes.olvidar();
        self.menu_cliente.cerrar();
        self.menu_producto.cerrar();
        self.producto_texto.clear();
        self.producto_elegido = None;
        self.cantidad_texto.clear();
        self.error_agregar = false;
        self.enfocar = Some(ID_CLIENTE);
        self.credito_de = None;
        self.cuenta = None;
        self.tarea_cuenta = None;
        self.enviar_con_cuenta = false;
        self.autorizado = None;
    }

    fn seguir_cliente(&mut self, cx: &Contexto) {
        let id = self.cliente.as_ref().map(|c| c["id"].clone()).filter(|id| !id.is_null());
        if id == self.credito_de {
            return;
        }
        self.credito_de = id.clone();
        self.cuenta = None;
        self.autorizado = None;
        self.enviar_con_cuenta = false;
        self.tarea_cuenta = id.map(|id| {
            let servicios = cx.servicios.clone();
            Tarea::lanzar(&cx.egui, move || {
                let clave = id.as_str().map(str::to_string).unwrap_or_else(|| id.to_string());
                let cuenta = obtener(&servicios, &format!("/api/clientes/{}", nucleo::api::codificar(&clave))).map(|c| logica::cuenta_cliente(&c));
                (id, cuenta)
            })
        });
    }

    fn recibir_cuenta(&mut self, cx: &mut Contexto) {
        let Some((id, cuenta)) = esperar(&mut self.tarea_cuenta) else { return };
        if Some(&id) != self.credito_de.as_ref() {
            return;
        }
        self.cuenta = cuenta;
        if std::mem::take(&mut self.enviar_con_cuenta) {
            self.enviar(cx);
        }
    }

    fn credito(&self, total: f64) -> Option<(Cuenta, Credito)> {
        self.cuenta.map(|cuenta| (cuenta, logica::evaluar_credito(cuenta, total)))
    }

    fn autorizado_para(&self, monto: f64) -> Option<&str> {
        self.autorizado
            .as_ref()
            .filter(|(id, autorizado, _)| Some(id) == self.credito_de.as_ref() && monto <= autorizado + 0.005)
            .map(|(_, _, email)| email.as_str())
    }

    fn buscar_lista(&self, id: Option<i64>) -> Option<Value> {
        id.and_then(|id| self.listas.iter().find(|l| l["id"].as_i64() == Some(id)).cloned())
    }

    pub fn abierto(&self) -> bool {
        self.abierto || self.envio.is_some()
    }

    fn iva_visible(&self, cx: &Contexto) -> Option<bool> {
        logica::iva_visible(&self.tipo_factura, super::iva_en_precios(cx))
    }

    fn cambiar_tipo(&mut self, tipo: &str, iva_en_precios: Option<bool>) {
        self.tipo_factura = tipo.to_string();
        let iva = logica::iva_visible(tipo, iva_en_precios);
        for fila in &mut self.filas {
            fila.precio = logica::texto_precio_con_iva(fila.detalle.producto.as_ref(), fila.detalle.precio_unitario, iva);
        }
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
                    if d.precio_libre {
                        return d;
                    }
                    let producto = obtener(s, &format!("/api/productos/{}", d.producto_id)).filter(|p| p["precio"].as_f64().is_some());
                    let precio_lista = lista.and_then(|lista| obtener(s, &format!("/api/listas-precios/{lista}/productos/{}", d.producto_id))).and_then(|v| v["precio"].as_f64());
                    d.precio_unitario = match producto {
                        Some(producto) => logica::precio_con_lista(&producto, precio_lista),
                        None => precio_lista.filter(|p| *p != 0.0).unwrap_or(d.precio_unitario),
                    };

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
                .map(|inicial| {
                    let detalle = match obtener(s, &format!("/api/productos/{}", inicial.id)) {
                        Some(producto) => {
                            let nombre = texto(&producto, "nombre").to_string();
                            Detalle::nuevo(inicial.id, Some(producto), nombre, inicial.cantidad, inicial.precio)
                        }
                        None => Detalle::nuevo(inicial.id, None, inicial.nombre, inicial.cantidad, inicial.precio),
                    };
                    if inicial.precio_libre { detalle.libre() } else { detalle }
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
                        let iva = self.iva_visible(cx);
                        self.filas = detalles.into_iter().map(|d| Fila::new(d, iva)).collect();
                        if let Some(id) = id_lista(&self.lista_seleccionada) {
                            self.actualizar_precios(cx, Some(id));
                        }
                    }
                }
                Respuesta::Actualizados(apertura, lista, detalles) => {
                    if apertura == self.apertura && lista == id_lista(&self.lista_seleccionada) {
                        let iva = self.iva_visible(cx);
                        self.filas = detalles.into_iter().map(|d| Fila::new(d, iva)).collect();
                    }
                }
            }
        }
    }

    fn enviar(&mut self, cx: &Contexto) {
        let Some(cliente) = self.cliente.clone() else {
            ui::v2::toast::error(&cx.egui, "Debe seleccionar un cliente para crear la factura");
            return;
        };
        if self.filas.is_empty() {
            ui::v2::toast::error(&cx.egui, "Debe agregar al menos un producto a la factura");
            return;
        }
        let negocio = cx.global.negocio.clone().unwrap_or(Value::Null);
        let detalles = self.detalles();
        if let Some(detalle) = logica::precio_libre_sin_monto(&detalles) {
            ui::v2::toast::con_detalle(&cx.egui, ui::v2::toast::Tipo::Error, "Falta el monto de un producto con precio a definir", detalle.descripcion.clone());
            return;
        }
        let totales = logica::calcular_totales(&detalles, &self.tipo_factura, negocio["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false));
        let pago = self.pago();
        if pago.is_some_and(|p| p > totales.total) {
            ui::v2::toast::error(&cx.egui, "El pago inicial no puede ser mayor al total");
            return;
        }
        if self.tarea_cuenta.is_some() {
            self.enviar_con_cuenta = true;
            return;
        }
        let a_cuenta = logica::a_cuenta(totales.total, pago);
        let mut observaciones = self.observaciones.clone();
        if let Some((cuenta, Credito::Excedido { excedente })) = self.credito(a_cuenta) {
            match self.autorizado_para(a_cuenta) {
                Some(admin) => observaciones = logica::observacion_con_autorizacion(&observaciones, admin),
                None => {
                    let email = cx.global.admin.as_ref().and_then(|a| a["email"].as_str()).unwrap_or_default();
                    self.autorizacion.abrir(cuenta, excedente, email);
                    return;
                }
            }
        }
        let afip = logica::usar_afip(&negocio, &self.tipo_factura);
        let pago_parcial = pago.is_some_and(|p| p > 0.0 && p < totales.total);
        let vencimiento = match logica::fecha_dia_mes_anio(&self.fecha_vencimiento) {
            _ if !pago_parcial || self.fecha_vencimiento.trim().is_empty() => None,
            Some(fecha) => logica::fecha_iso_desde_input(&fecha.format("%Y-%m-%d").to_string()),
            None => {
                ui::v2::toast::error(&cx.egui, "Ingresá una fecha de vencimiento válida (dd/mm/aaaa)");
                return;
            }
        };
        let datos = DatosFactura {
            cliente_id: cliente["id"].clone(),
            tipo_factura: &self.tipo_factura,
            observaciones: &observaciones,
            lista_precio_id: id_lista(&self.lista_seleccionada),
            pago_inicial: pago,
            fecha_vencimiento: vencimiento.as_deref(),
            detalles: &detalles,
        };
        let (ruta, cuerpo) = logica::cuerpo_factura(&datos, afip, totales, &negocio);
        let tipo = self.tipo_factura.clone();
        let servicios = cx.servicios.clone();
        let ctx = cx.egui.clone();
        self.guardando = true;
        self.envio = Some(Tarea::lanzar(&cx.egui, move || {
            let resultado = match servicios.api.pedir("POST", &ruta, Some(&cuerpo), &[]) {
                Ok(r) if r.ok() => r.cuerpo,
                Ok(r) => {
                    let mensaje = r.cuerpo["error"].as_str().unwrap_or("Error al guardar factura").to_string();
                    ui::v2::toast::error(&ctx, mensaje);
                    return None;
                }
                Err(e) => {
                    ui::v2::toast::error(&ctx, e.mensaje);
                    return None;
                }
            };
            if afip && resultado["afip"].is_object() {
                ui::v2::toast::exito(
                    &ctx,
                    format!(
                        "Factura AFIP creada correctamente\nCAE: {}\nVencimiento: {}",
                        super::mostrar_valor(&resultado["afip"]["cae"]),
                        super::mostrar_valor(&resultado["afip"]["vencimientoCae"])
                    ),
                );
            } else {
                ui::v2::toast::exito(&ctx, "Factura creada correctamente");
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
                ui::v2::toast::exito(&ctx, "Ticket impreso correctamente");
            } else {
                ui::v2::toast::error(&ctx, "Error al imprimir el ticket");
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
        self.seguir_cliente(cx);
        self.recibir_cuenta(cx);
        self.catalogo.pedir(cx, self.apertura);
        self.catalogo.recibir(cx);
        if self.menu_cliente.abierto {
            self.clientes.pedir(cx, &self.cliente_texto);
        }
        self.clientes.recibir(cx);
        if !self.iniciales.is_empty() && !self.cargando_iniciales {
            self.cargar_iniciales(cx);
        }
        self.pedir_precios(cx);

        let ctx = ui.ctx().clone();
        let (titulo, subtitulo) = if self.desde_orden {
            ("Enviar a cuenta corriente", "La orden actual queda a cuenta del cliente")
        } else {
            ("Nueva factura", "Comprobante a cuenta corriente")
        };
        let alto_cuerpo = (ctx.content_rect().height() - 218.0).max(220.0);
        let mut accion = Accion::Nada;
        let r = v2::Modal::new("factura-formulario", titulo).subtitulo(subtitulo).icono("file-text").ancho(ANCHO_MODAL).mostrar(&ctx, |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                ScrollArea::vertical().id_salt("factura-formulario-desplazable").max_height(alto_cuerpo).auto_shrink([false, true]).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    accion = self.contenido(ui, cx);
                });
            });
            v2::pie_modal(ui, |ui| {
                let ocupado = self.guardando || self.enviar_con_cuenta;
                let texto_crear = if ocupado { "Guardando…" } else { "Crear comprobante" };
                if Boton::primario(texto_crear).radio(radio::BOTON_MODAL).sin_sombra().cargando(ocupado).ui(ui).clicked() {
                    accion = Accion::Enviar;
                }
                if Boton::secundario("Cancelar").radio(radio::BOTON_MODAL).ui(ui).clicked() {
                    accion = Accion::Cancelar;
                }
            });
        });
        match accion {
            Accion::Nada => {}
            Accion::Lista(id) => {
                self.lista_seleccionada = self.buscar_lista(id);
                self.lista_persistida = id;
                self.pedir_precios(cx);
                self.actualizar_precios(cx, id);
            }
            Accion::Enviar => {
                self.intento = true;
                self.enviar(cx);
            }
            Accion::Cancelar => {
                self.abierto = false;
                return Some(ResultadoFactura::Cerrada);
            }
        }
        if let credito::Salida::Autorizada(admin) = self.autorizacion.mostrar(cx) {
            let iva = cx.global.negocio.as_ref().is_some_and(|n| n["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false));
            let total = logica::calcular_totales(&self.detalles(), &self.tipo_factura, iva).total;
            self.autorizado = self.credito_de.clone().map(|id| (id, logica::a_cuenta(total, self.pago()), admin));
            self.enviar(cx);
        }
        if r.cerrar && !self.autorizacion.abierta() {
            self.abierto = false;
            return Some(ResultadoFactura::Cerrada);
        }
        None
    }

    fn contenido(&mut self, ui: &mut Ui, cx: &Contexto) -> Accion {
        let mut accion = Accion::Nada;
        let negocio = cx.global.negocio.clone().unwrap_or(Value::Null);
        let totales = logica::calcular_totales(&self.detalles(), &self.tipo_factura, negocio["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false));
        let ancho = ui.available_width();
        let izquierda = ((ancho - 14.0) * 1.3 / 2.3).floor();
        let derecha = ancho - 14.0 - izquierda;
        let error_cliente = self.intento && self.cliente.is_none();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.vertical(|ui| {
                ui.set_width(izquierda);
                etiqueta(ui, "Cliente");
                ui.add_space(6.0);
                self.campo_cliente(ui, izquierda, error_cliente);
                let a_cuenta = logica::a_cuenta(totales.total, self.pago());
                if let Some((texto_credito, tinta)) = self.credito(a_cuenta).and_then(|(cuenta, credito)| aviso_credito(cuenta, credito, a_cuenta)) {
                    ui.add_space(6.0);
                    nota(ui, &texto_credito, tinta, izquierda);
                }
            });
            ui.vertical(|ui| {
                ui.set_width(derecha);
                etiqueta(ui, "Comprobante");
                ui.add_space(6.0);
                if cx.global.negocio_cargando {
                    v2::shimmer(ui, Vec2::new(derecha, 44.0), radio::BOTON_MODAL, true);
                } else if !negocio["afipHabilitado"].as_bool().unwrap_or(false) {
                    let mut cero = 0;
                    v2::segmentado(ui, &[Segmento::texto(TIPOS[0].1)], &mut cero, Some(derecha), EstiloSegmentado::Claro);
                    ui.add_space(6.0);
                    nota(ui, "AFIP no está habilitado para este negocio", color::TEXTO_TENUE, derecha);
                } else {
                    let anterior = TIPOS.iter().position(|(v, _)| *v == self.tipo_factura).unwrap_or(usize::MAX);
                    let mut indice = anterior;
                    let segmentos: Vec<Segmento> = TIPOS.iter().map(|(_, t)| Segmento::texto(t)).collect();
                    v2::segmentado(ui, &segmentos, &mut indice, Some(derecha), EstiloSegmentado::Claro);
                    if indice != anterior && indice < TIPOS.len() {
                        self.cambiar_tipo(TIPOS[indice].0, super::iva_en_precios(cx));
                    }
                }
            });
        });
        ui.add_space(16.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.vertical(|ui| {
                ui.set_width(izquierda);
                etiqueta(ui, "Lista de precios");
                ui.add_space(6.0);
                let mut opciones = vec!["Sin lista específica (precios base)".to_string()];
                let mut ids = vec![None];
                for lista in &self.listas {
                    let cantidad = super::mostrar_valor(&lista["_count"]["productos"]);
                    opciones.push(format!("{} ({cantidad} productos)", texto(lista, "nombre")));
                    ids.push(lista["id"].as_i64());
                }
                let actual = id_lista(&self.lista_seleccionada);
                let mut indice = ids.iter().position(|i| *i == actual).unwrap_or(0);
                if v2::desplegable(ui, "factura-lista", &mut indice, &opciones, "", izquierda, 44.0).changed() {
                    accion = Accion::Lista(ids[indice]);
                }
                ui.add_space(6.0);
                let ayuda = match &self.lista_seleccionada {
                    Some(l) => format!("Usando precios de \"{}\". Si un producto no está en esta lista, se usará el precio base.", texto(l, "nombre")),
                    None => "Se usarán los precios base de los productos.".into(),
                };
                nota(ui, &ayuda, color::TEXTO_TENUE, izquierda);
            });
            ui.vertical(|ui| {
                ui.set_width(derecha);
                etiqueta(ui, "Pago inicial (opcional)");
                ui.add_space(6.0);
                let contenedor = Contenedor { alto: 44.0, radio: radio::BOTON_MODAL, relleno_izquierdo: 12.0, relleno_derecho: 12.0, separacion: 6.0, ..Default::default() };
                let r = contenedor.mostrar(ui, derecha, |ui| {
                    v2::texto(ui, "$", tam::ENTRADA, Peso::Normal, color::TEXTO_TENUE);
                    v2::entrada_desnuda(&mut self.pago_inicial).tam(tam::CUERPO).sugerencia("0,00").id(Id::new("factura-pago-inicial")).ui(ui)
                });
                if r.changed() && self.pago().is_none() {
                    self.pago_inicial.clear();
                }
                let pago = self.pago().filter(|p| *p != 0.0);
                if pago.is_some_and(|p| p > totales.total) {
                    ui.add_space(6.0);
                    nota(ui, "El pago inicial no puede ser mayor al total", color::ROJO, derecha);
                }
                if let Some(p) = pago.filter(|p| *p > 0.0) {
                    ui.add_space(6.0);
                    nota(ui, &format!("Saldo pendiente: {}", formato::pesos(totales.total - p)), color::DORADO_TEXTO, derecha);
                }
                if pago.is_some_and(|p| p > 0.0 && p < totales.total) {
                    ui.add_space(12.0);
                    etiqueta(ui, "Fecha de vencimiento (opcional)");
                    ui.add_space(6.0);
                    CampoTexto::new(&mut self.fecha_vencimiento)
                        .icono("calendar")
                        .sugerencia("dd/mm/aaaa")
                        .alto(44.0)
                        .radio(radio::BOTON_MODAL)
                        .tam_texto(tam::CUERPO)
                        .ancho(derecha)
                        .id(Id::new("factura-vencimiento"))
                        .ui(ui);
                }
            });
        });
        ui.add_space(16.0);
        let iva = self.iva_visible(cx);
        self.tabla(ui, iva);
        if self.error_agregar {
            ui.add_space(8.0);
            nota(ui, "Elegí un producto y una cantidad.", color::ROJO, ancho);
        }
        ui.add_space(16.0);
        etiqueta(ui, "Observaciones");
        ui.add_space(6.0);
        CampoTexto::new(&mut self.observaciones)
            .multilinea(64.0)
            .tam_texto(tam::CUERPO)
            .sugerencia("Observaciones adicionales…")
            .id(Id::new("factura-observaciones"))
            .ui(ui);
        ui.add_space(16.0);
        pie_totales(ui, &self.tipo_factura, totales);
        if error_cliente {
            ui.add_space(10.0);
            nota(ui, "Elegí un cliente.", color::ROJO, ancho);
        }
        accion
    }

    fn pedir_foco(&mut self, ui: &Ui, campo: &'static str) {
        if self.enfocar != Some(campo) {
            return;
        }
        if ui.memory(|m| m.has_focus(Id::new(campo))) {
            self.enfocar = None;
        } else {
            ui.memory_mut(|m| m.request_focus(Id::new(campo)));
            ui.ctx().request_repaint();
        }
    }

    fn elegir_cliente(&mut self, posicion: usize) {
        if let Some(cliente) = self.clientes.lista.get(posicion).cloned() {
            self.cliente_texto = logica::nombre_cliente(&cliente);
            self.cliente = Some(cliente);
        }
        self.menu_cliente.cerrar();
    }

    fn campo_cliente(&mut self, ui: &mut Ui, ancho: f32, error: bool) {
        let id = Id::new(ID_CLIENTE);
        if let Some(posicion) = self.menu_cliente.teclas(ui, id, self.clientes.lista.len()) {
            self.elegir_cliente(posicion);
        }
        self.pedir_foco(ui, ID_CLIENTE);
        let cuit = self.cliente.as_ref().map(|c| texto(c, "cuit").to_string()).filter(|c| !c.is_empty());
        let contenedor = Contenedor { alto: 44.0, radio: radio::BOTON_MODAL, relleno_izquierdo: 12.0, relleno_derecho: 12.0, separacion: 8.0, error, ..Default::default() };
        let caja = Rect::from_min_size(ui.cursor().min, Vec2::new(ancho, 44.0));
        let respuesta = contenedor.mostrar(ui, ancho, |ui| {
            pintar_icono_solar(ui, "user", 17.0, color::TEXTO_TENUE);
            let ancho_cuit = cuit.as_ref().map_or(0.0, |c| v2::galeria(ui, c.clone(), marca::fuente_mono(tam::XS, Peso::Normal), color::TEXTO_TENUE).size().x + 8.0);
            let entrada = ui
                .allocate_ui_with_layout(Vec2::new(ui.available_width() - ancho_cuit, 42.0), Layout::left_to_right(Align::Center), |ui| {
                    v2::entrada_desnuda(&mut self.cliente_texto).tam(tam::CUERPO).sugerencia("Buscar cliente por nombre o CUIT").id(id).ui(ui)
                })
                .inner;
            if let Some(cuit) = &cuit {
                ui.label(marca::texto_mono(cuit.clone(), tam::XS, Peso::Normal, color::TEXTO_TENUE));
            }
            entrada
        });
        self.menu_cliente.enfocado = respuesta.has_focus();
        if respuesta.changed() {
            self.cliente = None;
            self.menu_cliente.abrir();
        } else if respuesta.clicked() {
            self.menu_cliente.abierto = true;
        }
        self.menu_cliente.cerrar_con_clic_afuera(ui, caja, selectores::id_popup_clientes());
        let ancla = Response { rect: caja, ..respuesta };
        if let Some(posicion) = selectores::lista_clientes(ui.ctx(), &ancla, &mut self.menu_cliente, &self.clientes) {
            self.elegir_cliente(posicion);
        }
    }

    fn elegir_producto(&mut self, producto: Value) {
        self.producto_texto = texto(&producto, "nombre").to_string();
        self.producto_elegido = Some(producto);
        self.menu_producto.cerrar();
        self.error_agregar = false;
        self.enfocar = Some(ID_CANTIDAD);
    }

    fn filtrar(&mut self, consulta: String, excluidos: Vec<i64>) -> Vec<usize> {
        let clave = (consulta, excluidos, self.catalogo.version);
        match &self.filtro {
            Some((anterior, filtrados)) if *anterior == clave => filtrados.clone(),
            _ => {
                let filtrados = logica::filtrar_productos(&self.catalogo.lista, &self.catalogo.indice, &clave.0, &clave.1);
                self.filtro = Some((clave, filtrados.clone()));
                filtrados
            }
        }
    }

    fn agregar_producto(&mut self, iva_en_precios: Option<bool>) {
        let cantidad = logica::parsear_numero(&self.cantidad_texto).filter(|c| *c > 0.0);
        let (Some(producto), Some(cantidad)) = (self.producto_elegido.clone(), cantidad) else {
            self.error_agregar = true;
            return;
        };
        let id = producto["id"].as_i64().unwrap_or_default();
        let nombre = texto(&producto, "nombre").to_string();
        let detalle = if logica::es_precio_libre(&producto) {
            let monto = logica::redondear2(cantidad);
            if monto <= 0.0 {
                self.error_agregar = true;
                return;
            }
            Detalle::nuevo(id, Some(producto), nombre, 1.0, monto).libre()
        } else {
            let precio_lista = self.lista_seleccionada.as_ref().and(self.precios_lista.get(&id).copied());
            let precio = logica::precio_con_lista(&producto, precio_lista);
            Detalle::nuevo(id, Some(producto), nombre, cantidad, precio)
        };
        self.filas.push(Fila::new(detalle, iva_en_precios));
        self.producto_elegido = None;
        self.producto_texto.clear();
        self.cantidad_texto.clear();
        self.error_agregar = false;
        self.enfocar = Some(ID_PRODUCTO);
    }

    fn barra_agregar(&mut self, ui: &mut Ui, iva_en_precios: Option<bool>) {
        let id_buscar = Id::new(ID_PRODUCTO);
        let id_cantidad = Id::new(ID_CANTIDAD);
        let excluidos: Vec<i64> = self.filas.iter().map(|f| f.detalle.producto_id).collect();
        let elegido_intacto = self.producto_elegido.as_ref().is_some_and(|p| texto(p, "nombre") == self.producto_texto);
        let consulta = if elegido_intacto { String::new() } else { self.producto_texto.clone() };
        let filtrados = self.filtrar(consulta, excluidos);
        if let Some(posicion) = self.menu_producto.teclas(ui, id_buscar, filtrados.len().min(selectores::PRODUCTOS_VISIBLES)) {
            self.elegir_producto(self.catalogo.lista[filtrados[posicion]].clone());
        }
        self.pedir_foco(ui, ID_PRODUCTO);
        self.pedir_foco(ui, ID_CANTIDAD);
        let elegido_intacto = self.producto_elegido.as_ref().is_some_and(|p| texto(p, "nombre") == self.producto_texto);
        let mut agregar = false;
        let mut elegido = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let ancho_agregar = 104.0;
            let ancho_buscar = ui.available_width() - 90.0 - ancho_agregar - 16.0;
            let caja = Rect::from_min_size(ui.cursor().min, Vec2::new(ancho_buscar, 38.0));
            let precio = self.producto_elegido.as_ref().filter(|_| elegido_intacto).map(|p| selectores::precio_producto(p, &self.precios_lista, iva_en_precios));
            let contenedor = Contenedor { alto: 38.0, radio: 9, relleno_izquierdo: 10.0, relleno_derecho: 10.0, separacion: 8.0, ..Default::default() };
            let buscar = contenedor.mostrar(ui, ancho_buscar, |ui| {
                pintar_icono_solar(ui, "search", 16.0, color::TEXTO_TENUE);
                let ancho_precio = precio.as_ref().map_or(0.0, |p| v2::ancho_texto(ui, p, tam::XS, Peso::Semi) + 8.0);
                let entrada = ui
                    .allocate_ui_with_layout(Vec2::new(ui.available_width() - ancho_precio, 36.0), Layout::left_to_right(Align::Center), |ui| {
                        v2::entrada_desnuda(&mut self.producto_texto).tam(tam::CONTROL).sugerencia("Buscar producto por nombre, PLU o código").id(id_buscar).ui(ui)
                    })
                    .inner;
                if let Some(precio) = &precio {
                    v2::texto(ui, precio.clone(), tam::XS, Peso::Semi, color::VERDE);
                }
                entrada
            });
            self.menu_producto.enfocado = buscar.has_focus();
            if buscar.changed() {
                self.producto_elegido = None;
                self.error_agregar = false;
                self.menu_producto.abrir();
            } else if buscar.clicked() {
                self.menu_producto.abierto = true;
            }
            self.menu_producto.cerrar_con_clic_afuera(ui, caja, selectores::id_popup_productos());
            let ancla = Response { rect: caja, ..buscar };
            let vista = VistaProductos { filtrados: &filtrados, precios: &self.precios_lista, iva_en_precios };
            elegido = selectores::lista_productos(ui.ctx(), &ancla, &mut self.menu_producto, &self.catalogo, &vista);
            let campo = Contenedor { alto: 38.0, radio: 9, relleno_izquierdo: 10.0, relleno_derecho: 10.0, ..Default::default() };
            let libre = self.producto_elegido.as_ref().filter(|_| elegido_intacto).is_some_and(logica::es_precio_libre);
            let sugerencia = if libre { "Monto $" } else { "Cant." };
            let cantidad = campo.mostrar(ui, 90.0, |ui| v2::entrada_desnuda(&mut self.cantidad_texto).tam(tam::CONTROL).sugerencia(sugerencia).id(id_cantidad).ui(ui));
            if cantidad.changed() {
                self.cantidad_texto.retain(|c| c.is_ascii_digit() || c == '.' || c == ',');
                self.error_agregar = false;
            }
            agregar |= cantidad.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            agregar |= Boton::new("Agregar", Estilo::ContornoMarca)
                .icono("plus")
                .tam_icono(14.0)
                .alto(38.0)
                .ancho(ancho_agregar)
                .radio(9)
                .relleno(12.0)
                .separacion(6.0)
                .tam_texto(tam::CONTROL)
                .peso(Peso::Semi)
                .ui(ui)
                .clicked();
        });
        if let Some(posicion) = elegido {
            self.elegir_producto(self.catalogo.lista[filtrados[posicion]].clone());
        }
        if agregar {
            self.agregar_producto(iva_en_precios);
        }
    }

    fn tabla(&mut self, ui: &mut Ui, iva_en_precios: Option<bool>) {
        let ancho = ui.available_width();
        let interior = ancho - 28.0;
        let fijos = [110.0, 110.0, 120.0, 36.0];
        let producto = interior - fijos.iter().sum::<f32>() - 40.0;
        let anchos = [producto, fijos[0], fijos[1], fijos[2], fijos[3]];
        let lista = self.lista_seleccionada.as_ref().map(|l| texto(l, "nombre").to_string());
        let precios = self.precios_lista.clone();
        let mut borrar = None;
        let marco = egui::Frame::NONE.stroke(Stroke::new(1.0, color::BORDE_SUAVE)).corner_radius(radio::POPOVER);
        marco.show(ui, |ui| {
            ui.set_width(ancho - 2.0);
            let (encabezado, _) = ui.allocate_exact_size(Vec2::new(ancho - 2.0, 38.0), Sense::hover());
            ui.painter().rect_filled(encabezado, CornerRadius { nw: 13, ne: 13, sw: 0, se: 0 }, color::FONDO_ENCABEZADO_TABLA);
            ui.painter().hline(encabezado.x_range(), encabezado.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            let cabeceras = columnas(encabezado, &anchos);
            for (i, titulo) in ["Producto", "Cantidad", "Precio", "Subtotal"].iter().enumerate() {
                let (pos, ancla) = if i == 0 { (cabeceras[i].left_center(), Align2::LEFT_CENTER) } else { (cabeceras[i].right_center(), Align2::RIGHT_CENTER) };
                v2::pintar_texto(ui, pos, ancla, *titulo, tam::CHICO, Peso::Medio, color::TEXTO_APAGADO);
            }
            if self.filas.is_empty() {
                let (vacio, _) = ui.allocate_exact_size(Vec2::new(ancho - 2.0, 54.0), Sense::hover());
                let mensaje = if self.cargando_iniciales { "Cargando productos…" } else { "Agregá al menos un producto." };
                v2::pintar_texto(ui, vacio.center(), Align2::CENTER_CENTER, mensaje, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
            } else {
                ScrollArea::vertical().id_salt("factura-filas").max_height(ALTO_FILA * 5.0).auto_shrink([false, true]).show(ui, |ui| {
                    ui.set_width(ancho - 2.0);
                    for (i, fila) in self.filas.iter_mut().enumerate() {
                        let (caja, respuesta) = ui.allocate_exact_size(Vec2::new(ancho - 2.0, ALTO_FILA), Sense::hover());
                        if respuesta.hovered() || ui.rect_contains_pointer(caja) {
                            ui.painter().rect_filled(caja, 0, color::FILA_HOVER);
                        }
                        ui.painter().hline(caja.x_range(), caja.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
                        let celdas = columnas(caja, &anchos);
                        let d = &mut fila.detalle;
                        let nombre = d.producto.as_ref().map(|p| texto(p, "nombre").to_string()).unwrap_or_else(|| format!("Producto {}", d.producto_id));
                        let arriba = caja.top() + 9.0;
                        let mut limite = celdas[0].right();
                        if let (Some(lista), true) = (&lista, precios.contains_key(&d.producto_id)) {
                            let g = v2::galeria(ui, lista.clone(), fuente(tam::MINI, Peso::Semi), color::DORADO_TEXTO);
                            let chip = Rect::from_min_size(egui::pos2(limite - g.size().x - 12.0, arriba), Vec2::new(g.size().x + 12.0, 18.0));
                            ui.painter().rect_filled(chip, radio::CHIP, color::DORADO_FONDO);
                            ui.painter().galley(egui::pos2(chip.left() + 6.0, chip.center().y - g.size().y / 2.0), g, color::DORADO_TEXTO);
                            limite = chip.left() - 8.0;
                        }
                        let g = v2::galeria(ui, nombre, fuente(tam::CONTROL, Peso::Medio), color::TEXTO);
                        let recorte = Rect::from_min_max(egui::pos2(celdas[0].left(), caja.top()), egui::pos2(limite, caja.bottom()));
                        ui.painter().with_clip_rect(recorte).galley(egui::pos2(celdas[0].left(), arriba + 9.0 - g.size().y / 2.0), g, color::TEXTO);
                        let descripcion = Rect::from_min_size(egui::pos2(celdas[0].left(), arriba + 20.0), Vec2::new(celdas[0].width(), 18.0));
                        let mut hijo = ui.new_child(UiBuilder::new().max_rect(descripcion));
                        v2::entrada_desnuda(&mut d.descripcion).tam(tam::CHICO).color(color::TEXTO_APAGADO).sugerencia("Agregar descripción").id(Id::new(("factura-desc", i))).ui(&mut hijo);
                        let celda = |c: usize| Rect::from_center_size(celdas[c].center(), Vec2::new(celdas[c].width(), 32.0));
                        let campo = Contenedor { alto: 32.0, radio: radio::ICONO, relleno_izquierdo: 8.0, relleno_derecho: 8.0, separacion: 4.0, ..Default::default() };
                        let mut hijo = ui.new_child(UiBuilder::new().max_rect(celda(1)));
                        let r = campo.mostrar(&mut hijo, celda(1).width(), |ui| v2::entrada_desnuda(&mut fila.cantidad).tam(tam::CONTROL).habilitado(!d.precio_libre).id(Id::new(("factura-cant", i))).ui(ui));
                        if r.changed() {
                            d.cantidad = logica::parsear_numero(&fila.cantidad).unwrap_or(0.0);
                            d.recalcular();
                        }
                        let mut hijo = ui.new_child(UiBuilder::new().max_rect(celda(2)));
                        let r = campo.mostrar(&mut hijo, celda(2).width(), |ui| {
                            v2::texto(ui, "$", tam::CONTROL, Peso::Normal, color::TEXTO_TENUE);
                            v2::entrada_desnuda(&mut fila.precio).tam(tam::CONTROL).id(Id::new(("factura-precio", i))).ui(ui)
                        });
                        if r.changed() {
                            d.precio_unitario = logica::precio_sin_iva(d.producto.as_ref(), logica::parsear_numero(&fila.precio).unwrap_or(0.0), iva_en_precios);
                            d.recalcular();
                        }
                        let subtotal = formato::pesos(logica::precio_con_iva(d.producto.as_ref(), d.subtotal, iva_en_precios));
                        v2::pintar_texto(ui, celdas[3].right_center(), Align2::RIGHT_CENTER, subtotal, tam::CONTROL, Peso::Semi, color::TEXTO);
                        let mut hijo = ui.new_child(UiBuilder::new().max_rect(Rect::from_center_size(celdas[4].center(), Vec2::splat(28.0))));
                        let quitar = BotonIcono::new("x").tam(28.0).tam_icono(15.0).radio(7).colores(color::TEXTO_DESHABILITADO, color::ROJO_FONDO, color::ROJO);
                        if quitar.ui(&mut hijo).clicked() {
                            borrar = Some(i);
                        }
                    }
                });
            }
            let (barra, _) = ui.allocate_exact_size(Vec2::new(ancho - 2.0, 58.0), Sense::hover());
            ui.painter().rect_filled(barra, CornerRadius { nw: 0, ne: 0, sw: 13, se: 13 }, color::FONDO_ENCABEZADO_TABLA);
            ui.painter().hline(barra.x_range(), barra.top() + 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            let mut hijo = ui.new_child(UiBuilder::new().max_rect(barra.shrink2(Vec2::new(14.0, 10.0))));
            self.barra_agregar(&mut hijo, iva_en_precios);
        });
        if let Some(i) = borrar {
            self.filas.remove(i);
        }
    }
}

enum Accion {
    Nada,
    Lista(Option<i64>),
    Enviar,
    Cancelar,
}

fn imprimir_factura(servicios: &Servicios, ctx: &egui::Context, factura: &Value, negocio: &Value) -> bool {
    let datos = logica::datos_impresion(factura, negocio);
    let nombre_con_logo = crate::impresion::nombre_con_logo(Some(negocio));
    if negocio["dobleImpresionEnabled"].as_bool() == Some(true) {
        let primera = imprimir(servicios, Comprobante::Factura, datos.clone(), nombre_con_logo);
        std::thread::sleep(std::time::Duration::from_millis(500));
        let segunda = imprimir(servicios, Comprobante::Factura, datos, nombre_con_logo);
        let ok = |r: &Value| r["success"].as_bool() == Some(true);
        if ok(&primera) && ok(&segunda) {
            ui::v2::toast::con_detalle(ctx, ui::v2::toast::Tipo::Exito, "Doble ticket de factura impreso correctamente", "Se mandaron 2 impresiones del ticket de factura");
        } else {
            let estado = |r: &Value| if ok(r) { "OK" } else { "Error" };
            let detalle = format!("Primera: {}, Segunda: {}", estado(&primera), estado(&segunda));
            ui::v2::toast::con_detalle(ctx, ui::v2::toast::Tipo::Aviso, "Problemas con la doble impresión de factura", detalle);
        }
        return true;
    }
    imprimir(servicios, Comprobante::Factura, datos, nombre_con_logo)["success"].as_bool() == Some(true)
}

fn aviso_credito(cuenta: Cuenta, credito: Credito, a_cuenta: f64) -> Option<(String, egui::Color32)> {
    let limite = cuenta.limite?;
    match credito {
        Credito::Cerca { uso } => Some((
            format!("Con esta venta usa el {} % del límite de crédito ({} de {}).", formato::numero((uso * 100.0).floor()), formato::pesos(cuenta.saldo + a_cuenta), formato::pesos(limite)),
            color::DORADO_TEXTO,
        )),
        Credito::Excedido { excedente } => {
            Some((format!("Supera el límite de crédito por {}. Para crearla hace falta autorización de un administrador.", formato::pesos(excedente)), color::ROJO))
        }
        Credito::SinLimite | Credito::Disponible => None,
    }
}

fn etiqueta(ui: &mut Ui, contenido: &str) {
    v2::texto(ui, contenido, tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO);
}

fn nota(ui: &mut Ui, contenido: &str, tinta: egui::Color32, ancho: f32) {
    v2::parrafo(ui, contenido, tam::CHICO, tinta, 1.4, ancho);
}

fn columnas(fila: Rect, anchos: &[f32; 5]) -> [Rect; 5] {
    let mut x = fila.left() + 14.0;
    anchos.map(|ancho| {
        let celda = Rect::from_min_max(egui::pos2(x, fila.top()), egui::pos2(x + ancho, fila.bottom()));
        x += ancho + 10.0;
        celda
    })
}

fn pie_totales(ui: &mut Ui, tipo: &str, totales: logica::Totales) {
    let con_iva = tipo != "remito" && totales.impuestos > 0.0;
    let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::hover());
    pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(fila.left() + 8.0, fila.center().y), Vec2::splat(16.0)), "info", color::VERDE);
    v2::pintar_texto(
        ui,
        egui::pos2(fila.left() + 24.0, fila.center().y),
        Align2::LEFT_CENTER,
        "Queda en la cuenta corriente del cliente hasta que registres los pagos.",
        tam::ETIQUETA,
        Peso::Normal,
        color::TEXTO_APAGADO,
    );
    let monto = v2::galeria(ui, formato::pesos(totales.total), fuente(tam::MONTO_CHICO, Peso::Semi), color::TEXTO);
    let x = fila.right() - monto.size().x;
    let base = fila.center().y + monto.size().y / 2.0;
    ui.painter().galley(egui::pos2(x, fila.center().y - monto.size().y / 2.0), monto, color::TEXTO);
    v2::pintar_texto(ui, egui::pos2(x - 10.0, base - 4.0), Align2::RIGHT_BOTTOM, "Total", tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
    if con_iva {
        ui.add_space(2.0);
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
        let detalle = format!("Subtotal {} · IVA {}", formato::pesos(totales.subtotal), formato::pesos(totales.impuestos));
        v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, detalle, tam::CHICO, Peso::Normal, color::TEXTO_TENUE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aviso_al_ochenta_por_ciento_y_al_superar_el_limite() {
        let cuenta = Cuenta { saldo: 30000.0, limite: Some(50000.0) };
        let aviso = |total: f64| aviso_credito(cuenta, logica::evaluar_credito(cuenta, total), total).map(|(texto, _)| texto);
        assert_eq!(aviso(5000.0), None);
        assert_eq!(aviso(12500.0).as_deref(), Some("Con esta venta usa el 85 % del límite de crédito ($42.500,00 de $50.000,00)."));
        assert_eq!(aviso(25000.0).as_deref(), Some("Supera el límite de crédito por $5.000,00. Para crearla hace falta autorización de un administrador."));
        let sin_limite = Cuenta { saldo: 1e6, limite: None };
        assert_eq!(aviso_credito(sin_limite, logica::evaluar_credito(sin_limite, 10.0), 10.0), None);
    }

    #[test]
    fn el_credito_se_autoriza_por_cliente_y_hasta_el_monto_autorizado() {
        let mut formulario = FormularioFactura { credito_de: Some(json!("c1")), autorizado: Some((json!("c1"), 1000.0, "jefe@b.c".into())), ..Default::default() };
        assert_eq!(formulario.autorizado_para(1000.0), Some("jefe@b.c"));
        assert_eq!(formulario.autorizado_para(1000.5), None);
        formulario.credito_de = Some(json!("c2"));
        assert_eq!(formulario.autorizado_para(10.0), None);

    }

    #[test]
    fn el_producto_con_precio_a_definir_se_agrega_con_el_monto_tipeado() {
        let varios = json!({ "id": 9, "nombre": "Varios", "precio": 0, "ivaIncluido": false, "ivaPorcentaje": 21.0, "precioLibre": true });
        let mut formulario = FormularioFactura { producto_elegido: Some(varios), cantidad_texto: "1500,5".into(), ..Default::default() };
        formulario.agregar_producto(Some(false));
        let detalle = &formulario.filas[0].detalle;
        assert!(detalle.precio_libre);
        assert_eq!((detalle.cantidad, detalle.precio_unitario, detalle.subtotal), (1.0, 1500.5, 1500.5));
        assert_eq!(formulario.filas[0].precio, "1500,5");
        let mut sin_monto = FormularioFactura { producto_elegido: Some(json!({ "id": 9, "precioLibre": true })), cantidad_texto: "0".into(), ..Default::default() };
        sin_monto.agregar_producto(Some(false));
        assert!(sin_monto.filas.is_empty() && sin_monto.error_agregar);
    }

    #[test]
    fn el_filtro_de_productos_se_recalcula_solo_si_cambia_la_consulta_o_el_catalogo() {
        let mut formulario = FormularioFactura::default();
        formulario.catalogo.lista = vec![json!({ "id": 1, "nombre": "Tomate" }), json!({ "id": 2, "nombre": "Papa" })];
        formulario.catalogo.indice = formulario.catalogo.lista.iter().map(nucleo::busqueda::ItemBusqueda::de_json).collect();
        assert_eq!(formulario.filtrar("papa".into(), vec![]), [1]);
        formulario.catalogo.lista.clear();
        assert_eq!(formulario.filtrar("papa".into(), vec![]), [1]);
        assert!(formulario.filtrar("papa".into(), vec![2]).is_empty());
        formulario.catalogo.version += 1;
        assert!(formulario.filtrar("papa".into(), vec![]).is_empty());
    }

    #[test]
    fn cambiar_de_remito_a_factura_y_volver_recalcula_los_precios() {
        let producto = json!({ "ivaIncluido": false, "ivaPorcentaje": 21.0 });
        let mut formulario = FormularioFactura::default();
        formulario.cambiar_tipo("remito", Some(false));
        let iva = logica::iva_visible(&formulario.tipo_factura, Some(false));
        formulario.filas.push(Fila::new(Detalle::nuevo(1, Some(producto), "Tomate".into(), 2.0, 100.0), iva));
        formulario.filas.push(Fila::new(Detalle::nuevo(2, None, "Varios".into(), 1.0, 50.0), iva));
        let precios = |f: &FormularioFactura| f.filas.iter().map(|f| f.precio.clone()).collect::<Vec<_>>();
        assert_eq!(precios(&formulario), ["100", "50"]);
        formulario.cambiar_tipo("B", Some(false));
        assert_eq!(formulario.tipo_factura, "B");
        assert_eq!(precios(&formulario), ["121", "60,5"]);
        formulario.cambiar_tipo("remito", Some(false));
        assert_eq!(precios(&formulario), ["100", "50"]);
        assert_eq!(formulario.detalles().iter().map(|d| d.precio_unitario).collect::<Vec<_>>(), [100.0, 50.0]);
    }
}
