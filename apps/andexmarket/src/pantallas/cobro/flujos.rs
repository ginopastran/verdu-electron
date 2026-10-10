use std::thread::sleep;
use std::time::Duration;

use eframe::egui::Context;
use nucleo::api::Respuesta;
use nucleo::carrito::Producto;
use nucleo::cobros::{self, redondear, verdad, Descuento, Venta};
use nucleo::entorno;
use serde_json::{json, Value};

use super::{aplicar, CancelacionPendiente, Cobro, Efectivo, Lado};
use crate::app::{Contexto, Servicios};
use crate::impresion::{self, Comprobante};
use crate::ui::toast::{self, Tipo};

#[derive(Clone)]
pub(super) struct Datos {
    servicios: Servicios,
    ctx: Context,
    pub negocio: Value,
    usuario: Value,
    negocio_nombre: String,
    items: Vec<Producto>,
    pub subtotal: f64,
    total_iva: f64,
}

impl Datos {
    pub fn de(cx: &Contexto) -> Self {
        let iva = cx.global.negocio.as_ref().map(|n| verdad(&n["ivaIncluidoEnPrecios"]));
        let negocio_nombre = cx
            .global
            .admin
            .as_ref()
            .and_then(|a| a["businessName"].as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Verdulería")
            .to_string();
        Datos {
            servicios: cx.servicios.clone(),
            ctx: cx.egui.clone(),
            negocio: cx.global.negocio.clone().unwrap_or(Value::Null),
            usuario: cx.global.usuario.clone().unwrap_or(Value::Null),
            negocio_nombre,
            items: cx.global.carritos.items().to_vec(),
            subtotal: cx.global.carritos.total(),
            total_iva: cx.global.carritos.total_con_iva(iva),
        }
    }

    fn venta<'a>(&'a self, creado: &'a str) -> Venta<'a> {
        Venta { usuario: &self.usuario, negocio_nombre: &self.negocio_nombre, creado }
    }

    fn nombre_comercio(&self) -> Value {
        cobros::primero([&self.negocio["nombre"], &self.negocio["name"]]).cloned().unwrap_or(json!("Verdulería"))
    }

    fn doble(&self) -> bool {
        self.negocio["dobleImpresionEnabled"] == true
    }

    fn mp_deshabilitado(&self) -> bool {
        self.negocio["mpEnabled"] == false
    }

    fn exito(&self, titulo: impl Into<String>) {
        toast::exito(&self.ctx, titulo);
    }

    fn error(&self, titulo: impl Into<String>) {
        toast::error(&self.ctx, titulo);
    }

    fn detalle(&self, tipo: Tipo, titulo: &str, detalle: impl Into<String>, segundos: f64) {
        toast::con_detalle(&self.ctx, tipo, titulo, detalle, segundos);
    }

    fn pedir(&self, metodo: &str, ruta: &str, cuerpo: Option<&Value>) -> Result<Respuesta, String> {
        let encabezados: Vec<(&str, &str)> = entorno::APP_ID.map(|id| vec![("X-App-ID", id)]).unwrap_or_default();
        self.servicios.api.pedir(metodo, ruta, cuerpo, &encabezados).map_err(|e| e.mensaje)
    }

    fn imprimir_venta(&self, mut orden: Value) {
        cobros::ajustar_total_ticket(&mut orden);
        let imprimir = || impresion::imprimir(&self.servicios, Comprobante::Venta, orden.clone());
        let bien = |r: &Value| r["success"] == true;
        if self.doble() {
            let primera = imprimir();
            sleep(Duration::from_millis(500));
            let segunda = imprimir();
            if bien(&primera) && bien(&segunda) {
                self.detalle(Tipo::Exito, "Doble ticket impreso correctamente", "Se mandaron 2 impresiones al script PHP", 2.0);
            } else {
                let estado = |r: &Value| if bien(r) { "OK" } else { "Error" };
                let texto = format!("Primera: {}, Segunda: {}", estado(&primera), estado(&segunda));
                self.detalle(Tipo::Aviso, "Problemas con la doble impresión", texto, 2.0);
            }
        } else {
            let resultado = imprimir();
            if bien(&resultado) {
                self.exito("Ticket impreso correctamente");
            } else if verdad(&resultado["printerError"]) {
                self.error(format!("Error de impresión: {}", texto_js(&resultado["printerError"])));
            } else {
                self.error("Error al imprimir el ticket");
            }
        }
    }

    fn imprimir_afip(&self, datos: &Value) -> bool {
        let resultado = impresion::imprimir(&self.servicios, Comprobante::Afip, datos.clone());
        resultado["success"] == false || verdad(&resultado["error"]) || verdad(&resultado["printerError"])
    }

    fn crear_orden(&self, orden: &Value) -> Result<Option<Value>, String> {
        let duplicada = |mensaje: &str| mensaje.contains("No valid items to create order payload");
        let payload = match cobros::payload_valido(orden) {
            Ok(payload) => payload,
            Err(mensaje) if duplicada(&mensaje) => return Ok(None),
            Err(mensaje) => return Err(mensaje),
        };
        let respuesta = self.pedir("POST", "/api/ordenes", Some(&payload))?;
        if !respuesta.ok() {
            let mensaje = cobros::primero([&respuesta.cuerpo["message"], &respuesta.cuerpo["error"]])
                .and_then(Value::as_str)
                .unwrap_or("Error al crear la orden")
                .to_string();
            return if duplicada(&mensaje) { Ok(None) } else { Err(mensaje) };
        }
        if verdad(&respuesta.cuerpo["isDuplicate"]) {
            return Ok(None);
        }
        Ok(Some(respuesta.cuerpo))
    }

    fn generar_qr(&self, payload: &Value, defecto: &str) -> Result<Value, String> {
        let respuesta = self.pedir("POST", "/api/mercadopago/generate-qr", Some(payload))?;
        if !respuesta.ok() {
            return Err(mensaje(&respuesta.cuerpo, "message", defecto));
        }
        Ok(respuesta.cuerpo)
    }
}

pub(super) fn texto_js(valor: &Value) -> String {
    match valor {
        Value::String(s) => s.clone(),
        Value::Number(n) => cobros::numero_js(n.as_f64().unwrap_or(0.0)),
        otro => otro.to_string(),
    }
}

fn mensaje(cuerpo: &Value, clave: &str, defecto: &str) -> String {
    cuerpo[clave].as_str().filter(|s| !s.is_empty()).unwrap_or(defecto).to_string()
}

fn ahora_iso() -> String {
    cobros::fecha_iso(chrono::Utc::now())
}

fn con_qr(resultado: Value, monto: f64, items: Vec<Value>) -> Value {
    let mut qr = resultado;
    qr["monto"] = json!(monto);
    qr["items"] = Value::Array(items);
    qr
}

impl Cobro {
    pub(super) fn elegir_metodo(&mut self, cx: &mut Contexto, metodo: &'static str) {
        if self.bloqueado() {
            return;
        }
        let d = Datos::de(cx);
        if d.usuario.is_null() {
            d.error("Debes iniciar sesión para realizar una orden");
            return;
        }
        if self.normal.procesando {
            return;
        }
        match metodo {
            "efectivo" => {
                self.normal.metodo = Some("efectivo");
                let descuento = self.descuento;
                self.abrir_efectivo(Lado::Normal, &d, descuento.is_some(), descuento);
                self.pago = None;
            }
            "split" => {
                self.abrir_mixto(Lado::Normal, &d);
                self.pago = None;
            }
            _ => {
                self.normal.procesando = true;
                self.normal.metodo = Some(metodo);
                if metodo == "qr" {
                    if d.mp_deshabilitado() {
                        self.qr_directo(cx, d);
                    } else {
                        self.generar_qr(cx, d);
                    }
                } else {
                    let subtotal = redondear(d.total_iva, 2);
                    let total = match &self.descuento {
                        Some(desc) => redondear(subtotal - desc.monto, 2),
                        None => subtotal,
                    };
                    let descuento = self.descuento;
                    self.procesar_pago(cx, d, metodo, total, descuento);
                }
            }
        }
    }

    pub(super) fn elegir_metodo_afip(&mut self, cx: &mut Contexto, metodo: &'static str) {
        if self.bloqueado() {
            return;
        }
        let d = Datos::de(cx);
        let descuento = self.descuento;
        match metodo {
            "efectivo" => {
                if cx.global.negocio.is_none() {
                    d.error("Información de negocio no disponible");
                    return;
                }
                self.abrir_efectivo(Lado::Afip, &d, false, descuento);
                self.pago = None;
            }
            "qr" if d.mp_deshabilitado() => self.procesar_afip(cx, d, "qr", None, descuento, None),
            "qr" => self.generar_qr_afip(cx, d),
            "split" => {
                self.abrir_mixto(Lado::Afip, &d);
                self.pago = None;
            }
            _ => self.procesar_afip(cx, d, metodo, None, descuento, None),
        }
    }

    pub(super) fn abrir_efectivo(&mut self, lado: Lado, d: &Datos, con_descuento: bool, descuento: Option<Descuento>) {
        let exacto = d.negocio["sistemaPago"] == "pago-exacto";
        let porcentaje = if con_descuento { cobros::descuento_efectivo(&d.negocio) } else { None };
        let (original, cobrar) = cobros::montos_efectivo(d.total_iva, descuento.as_ref(), porcentaje, exacto);
        let aplicando = match lado {
            Lado::Normal => con_descuento || descuento.is_some(),
            Lado::Afip => con_descuento,
        };
        let p = self.proc_mut(lado);
        p.metodo = Some("efectivo");
        p.efectivo = Some(Efectivo { exacto, original, cobrar, aplicando, enfocar: true, ..Default::default() });
        if lado == Lado::Afip {
            self.pago = None;
        }
    }

    pub(super) fn aplicar_descuento_efectivo(&mut self, cx: &mut Contexto, lado: Lado) {
        if cx.global.negocio.is_none() {
            return;
        }
        let d = Datos::de(cx);
        self.proc_mut(lado).efectivo = None;
        let descuento = if lado == Lado::Afip { self.descuento } else { None };
        self.abrir_efectivo(lado, &d, true, descuento);
    }

    pub(super) fn cerrar_efectivo(&mut self, lado: Lado) {
        let p = self.proc_mut(lado);
        p.efectivo = None;
        p.reiniciar();
    }

    pub(super) fn confirmar_efectivo(&mut self, cx: &mut Contexto, lado: Lado) {
        let Some(efectivo) = self.proc_mut(lado).efectivo.as_ref() else { return };
        let (exacto, original, cobrar, aplicando) = (efectivo.exacto, efectivo.original, efectivo.cobrar, efectivo.aplicando);
        let d = Datos::de(cx);
        let porcentaje = cobros::descuento_efectivo(&d.negocio);
        match (lado, exacto) {
            (Lado::Normal, false) => {
                if self.normal.procesando {
                    return;
                }
                if d.usuario.is_null() {
                    d.error("Debes iniciar sesión para realizar una orden");
                    return;
                }
                if cobrar <= 0.0 {
                    d.error("Error: Monto inválido");
                    return;
                }
                self.normal.procesando = true;
                let monto = redondear(d.subtotal - cobrar, 2);
                let descuento = match (self.descuento, aplicando, porcentaje) {
                    (Some(desc), _, _) => Some(desc.con_monto(monto)),
                    (None, true, Some(p)) => Some(Descuento::porcentaje(p, monto)),
                    _ => None,
                };
                self.procesar_pago(cx, d, "efectivo", cobrar, descuento);
            }
            (Lado::Normal, true) => {
                self.normal.procesando = true;
                let base = if original != 0.0 { original } else { d.total_iva };
                let monto = redondear(base - cobrar, 2);
                let descuento = match (self.descuento, aplicando, porcentaje) {
                    (Some(desc), true, _) => Some(desc.con_monto(monto)),
                    (_, true, Some(p)) => Some(Descuento::porcentaje(p, monto)),
                    (desc, _, _) => desc,
                };
                self.procesar_pago(cx, d, "efectivo", cobrar, descuento);
            }
            (Lado::Afip, false) => {
                let monto = redondear(d.subtotal - cobrar, 2);
                let descuento = match (self.descuento, aplicando, porcentaje) {
                    (Some(desc), true, _) => Some(desc.con_monto(monto)),
                    (_, true, Some(p)) => Some(Descuento::porcentaje(p, monto)),
                    _ => None,
                };
                self.procesar_afip(cx, d, "efectivo", Some(cobrar), descuento, None);
            }
            (Lado::Afip, true) => {
                if d.usuario.is_null() {
                    d.error("Debes iniciar sesión para realizar una factura AFIP");
                    return;
                }
                self.afip.procesando = true;
                let descuento = match (self.descuento, aplicando, porcentaje) {
                    (Some(desc), _, _) => Some(desc),
                    (None, true, Some(p)) => Some(Descuento::porcentaje(p, 0.0)),
                    _ => None,
                };
                self.procesar_afip(cx, d, "efectivo", Some(cobrar), descuento, None);
            }
        }
    }

    fn procesar_pago(&mut self, cx: &Contexto, d: Datos, metodo: &'static str, total: f64, descuento: Option<Descuento>) {
        if d.usuario.is_null() {
            d.error("Debes iniciar sesión para realizar una orden");
            self.normal.reiniciar();
            return;
        }
        self.normal.metodo = Some(metodo);
        self.normal.procesando = true;
        let base = if descuento.is_some() { d.total_iva } else { total };
        let creado = ahora_iso();
        let campos = cobros::campos_descuento(
            descuento.as_ref(),
            descuento.map_or(0.0, |x| x.monto),
            redondear(base, 2),
            Some(redondear(base, 2)),
        );
        let orden = cobros::orden(&d.venta(&creado), Some(metodo), total, cobros::items_orden(&d.items, true), None, None, campos);
        self.lanzar(cx, true, move || {
            let cerrar = |c: &mut Cobro| {
                c.normal.efectivo = None;
                if c.pago == Some(Lado::Normal) {
                    c.pago = None;
                }
                c.normal.liberar();
            };
            match d.crear_orden(&orden) {
                Ok(None) => {
                    d.exito("Orden completada exitosamente");
                    aplicar(|c, cx| {
                        c.vaciar(cx);
                        c.normal.reiniciar();
                    })
                }
                Ok(Some(resultado)) => {
                    d.imprimir_venta(cobros::con_id(orden, &resultado));
                    if d.doble() {
                        toast::info(&d.ctx, "Ticket ya fue impreso anteriormente");
                    }
                    d.exito("Orden completada exitosamente");
                    sleep(Duration::from_millis(1500));
                    aplicar(move |c, cx| {
                        c.normal.reiniciar();
                        c.vaciar(cx);
                        cerrar(c);
                    })
                }
                Err(_) => {
                    d.error("Error al procesar la orden");
                    aplicar(move |c, _| {
                        c.normal.reiniciar();
                        cerrar(c);
                    })
                }
            }
        });
    }

    fn qr_directo(&mut self, cx: &Contexto, d: Datos) {
        let subtotal = redondear(d.subtotal, 2);
        let descuento = self.descuento;
        let total = descuento.map_or(subtotal, |x| redondear(subtotal - x.monto, 2));
        let creado = ahora_iso();
        let campos = cobros::campos_descuento(descuento.as_ref(), descuento.map_or(0.0, |x| x.monto), subtotal, Some(subtotal));
        let orden = cobros::orden(&d.venta(&creado), Some("qr"), total, cobros::items_orden(&d.items, true), Some("COMPLETADA"), None, campos);
        self.lanzar(cx, true, move || {
            let respuesta = d.pedir("POST", "/api/ordenes", Some(&orden)).and_then(|r| {
                if r.ok() {
                    Ok(r.cuerpo)
                } else {
                    Err(mensaje(&r.cuerpo, "message", "Error al crear la orden"))
                }
            });
            match respuesta {
                Ok(resultado) => {
                    d.imprimir_venta(cobros::con_id(orden, &resultado));
                    d.exito("Orden completada exitosamente");
                    sleep(Duration::from_millis(100));
                    aplicar(|c, cx| {
                        c.vaciar(cx);
                        c.normal.liberar();
                        c.pago = None;
                    })
                }
                Err(mensaje) => {
                    d.error(format!("Error al procesar la orden: {mensaje}"));
                    aplicar(|c, _| {
                        c.normal.liberar();
                        c.pago = None;
                    })
                }
            }
        });
    }

    fn generar_qr(&mut self, cx: &Contexto, d: Datos) {
        self.normal.procesando = true;
        self.normal.metodo = Some("qr");
        let descuento = self.descuento;
        let monto = descuento.map_or(redondear(d.total_iva, 2), |x| redondear(d.total_iva - x.monto, 2));
        let items = cobros::items_orden(&d.items, false);
        let payload = cobros::Qr {
            monto,
            descripcion: format!("Compra de {} productos", items.len()),
            usuario: &d.usuario,
            items: items.clone(),
            afip: false,
            mixto: false,
            efectivo: 0.0,
        }
        .payload(cobros::descuento_qr(descuento.as_ref(), redondear(d.total_iva, 2)));
        self.lanzar(cx, true, move || {
            let resultado = d.generar_qr(&payload, "Error al generar el QR para pago").and_then(|r| {
                if verdad(&r["qrData"]) {
                    Ok(r)
                } else {
                    Err("La respuesta del backend no incluyó los datos del QR.".to_string())
                }
            });
            match resultado {
                Ok(r) => aplicar(move |c, cx| {
                    let orden = texto_js(&r["id"]);
                    c.normal.qr = Some(con_qr(r, monto, items));
                    c.normal.estado = Some("PENDIENTE".into());
                    c.normal.iniciar_sondeo(orden, cx.egui.input(|i| i.time));
                    c.abrir_qr(cx);
                    c.pago = None;
                }),
                Err(mensaje) => {
                    d.error(format!("Error al generar QR: {mensaje}"));
                    aplicar(|c, _| c.normal.liberar())
                }
            }
        });
    }

    fn generar_qr_afip(&mut self, cx: &Contexto, d: Datos) {
        self.afip.procesando = true;
        self.afip.metodo = Some("qr");
        let descuento = self.descuento;
        let items = cobros::items_orden(&d.items, false);
        let con_descuento = descuento.map_or(d.total_iva, |x| d.total_iva - x.monto);
        let subtotal = if descuento.is_some() { d.total_iva } else { con_descuento };
        let monto = redondear(con_descuento, 2);
        let payload = cobros::Qr {
            monto,
            descripcion: format!("Compra de {} productos con Factura AFIP", items.len()),
            usuario: &d.usuario,
            items: items.clone(),
            afip: true,
            mixto: false,
            efectivo: 0.0,
        }
        .payload(cobros::descuento_qr_afip(descuento.as_ref(), redondear(subtotal, 2), redondear(subtotal - con_descuento, 2), true));
        let subtotal_carrito = redondear(d.subtotal, 2);
        self.lanzar(cx, true, move || {
            let resultado = d.generar_qr(&payload, "Error al generar el QR para pago AFIP").and_then(|r| {
                if verdad(&r["qrData"]) && verdad(&r["id"]) {
                    Ok(r)
                } else {
                    Err("La respuesta del backend no incluyó 'qrData' o 'id'.".to_string())
                }
            });
            match resultado {
                Ok(r) => aplicar(move |c, cx| {
                    let orden = texto_js(&r["id"]);
                    let qr = con_qr(r, monto, items);
                    let mut normal = qr.clone();
                    normal["monto"] = json!(subtotal_carrito);
                    c.afip.qr = Some(qr);
                    c.afip.estado = Some("PENDIENTE".into());
                    c.afip.iniciar_sondeo(orden, cx.egui.input(|i| i.time));
                    c.pago = None;
                    c.normal.qr = Some(normal);
                    c.normal.estado = Some("PENDIENTE".into());
                    c.abrir_qr(cx);
                }),
                Err(mensaje) => {
                    d.error(mensaje);
                    aplicar(|c, _| c.afip.reiniciar())
                }
            }
        });
    }

    fn procesar_afip(
        &mut self,
        cx: &Contexto,
        d: Datos,
        metodo: &'static str,
        total: Option<f64>,
        descuento: Option<Descuento>,
        mixto: Option<(f64, &'static str, f64)>,
    ) {
        if d.usuario.is_null() {
            d.error("Debes iniciar sesión para realizar una factura");
            return;
        }
        let total = total.filter(|t| *t != 0.0).unwrap_or(d.total_iva);
        self.afip.procesando = true;
        self.afip.metodo = Some(metodo);
        let fecha = cobros::fecha_es_ar(chrono::Local::now());
        self.lanzar(cx, true, move || {
            let factura = cobros::Afip {
                negocio: &d.negocio,
                usuario: &d.usuario,
                metodo,
                items: cobros::items_afip(&d.items),
                total,
                subtotal_iva: d.total_iva,
                descuento,
                mixto,
            };
            let cerrar = |c: &mut Cobro| {
                c.afip.reiniciar();
                if c.pago == Some(Lado::Afip) {
                    c.pago = None;
                }
                c.afip.mixto_abierto = false;
            };
            let resultado = d.pedir("POST", "/api/facturas/crear-afip", Some(&factura.payload())).and_then(|r| {
                if !r.ok() {
                    Err(mensaje(&r.cuerpo, "error", "Error al crear la factura AFIP"))
                } else if !verdad(&r.cuerpo["afip"]["cae"]) {
                    Err("No se recibió CAE de AFIP".to_string())
                } else {
                    Ok(r.cuerpo)
                }
            });
            match resultado {
                Ok(resultado) => {
                    let datos = factura.impresion(&resultado, &fecha);
                    let fallo = d.imprimir_afip(&datos);
                    if fallo {
                        d.detalle(Tipo::Error, "Error al imprimir factura AFIP", "La factura se creó exitosamente pero no se pudo imprimir", 4.0);
                    } else if d.doble() && d.imprimir_afip(&datos) {
                        toast::aviso(&d.ctx, "Primera impresión exitosa, pero la segunda copia falló");
                    }
                    let cae = texto_js(&resultado["afip"]["cae"]);
                    let detalle = if fallo { format!("CAE: {cae} (Error de impresión)") } else { format!("CAE: {cae}") };
                    d.detalle(Tipo::Exito, "Factura AFIP creada exitosamente", detalle, 5.0);
                    sleep(Duration::from_millis(1500));
                    aplicar(move |c, cx| {
                        c.vaciar(cx);
                        cerrar(c);
                    })
                }
                Err(mensaje) => {
                    let detalle = if mensaje.contains("Failed to copy file to printer") || mensaje.contains("Error de impresión") {
                        "Problema de conexión con la impresora".to_string()
                    } else {
                        format!("{}...", mensaje.chars().take(100).collect::<String>())
                    };
                    d.detalle(Tipo::Error, "Error al crear factura AFIP", detalle, 4.0);
                    aplicar(move |c, _| cerrar(c))
                }
            }
        });
    }

    fn abrir_mixto(&mut self, lado: Lado, d: &Datos) {
        if d.usuario.is_null() {
            d.error(if lado == Lado::Normal {
                "Debes iniciar sesión para realizar una orden"
            } else {
                "Debes iniciar sesión para realizar una factura"
            });
            return;
        }
        let p = self.proc_mut(lado);
        if p.procesando {
            return;
        }
        p.metodo = Some("split");
        p.efectivo_mixto.clear();
        p.segundo = "tarjeta";
        p.mixto_abierto = true;
        p.enfocar_mixto = true;
    }

    pub(super) fn cerrar_mixto(&mut self, lado: Lado) {
        let p = self.proc_mut(lado);
        p.mixto_abierto = false;
        p.reiniciar();
    }

    pub(super) fn total_mixto(&self, subtotal: f64) -> f64 {
        self.descuento.map_or(subtotal, |d| subtotal - d.monto)
    }

    pub(super) fn completar_mixto(&mut self, cx: &mut Contexto, lado: Lado) {
        let d = Datos::de(cx);
        let total = self.total_mixto(d.subtotal);
        let texto = self.proc_mut(lado).efectivo_mixto.clone();
        let efectivo = cobros::parse_float(&texto);
        if lado == Lado::Afip {
            let Some(efectivo) = efectivo.filter(|e| *e > 0.0 && *e < total) else {
                d.error("El monto en efectivo no es válido.");
                return;
            };
            self.afip.procesando = true;
            self.afip.metodo = Some("split");
            let segundo = self.afip.segundo;
            let descuento = self.descuento;
            if segundo == "qr" && !d.mp_deshabilitado() {
                self.generar_qr_mixto_afip(cx, d, efectivo, total - efectivo);
            } else {
                self.procesar_afip(cx, d, "split", Some(total), descuento, Some((efectivo, segundo, total - efectivo)));
            }
            return;
        }
        if d.usuario.is_null() {
            d.error("Debes iniciar sesión para realizar una orden");
            return;
        }
        if self.normal.procesando {
            return;
        }
        let Some(efectivo) = efectivo.filter(|e| *e > 0.0) else {
            d.error("Ingresa un monto válido para el pago en efectivo");
            return;
        };
        if efectivo > total {
            d.error("El monto en efectivo no puede ser mayor al total");
            return;
        }
        let segundo_monto = redondear(total - efectivo, 2);
        if segundo_monto <= 0.0 {
            d.error("El monto para el segundo método debe ser mayor a cero");
            return;
        }
        self.normal.procesando = true;
        if self.normal.segundo == "qr" {
            self.generar_qr_mixto(cx, d, efectivo, segundo_monto);
        } else {
            self.mixto_tarjeta(cx, d, total, efectivo, segundo_monto);
        }
    }

    fn mixto_tarjeta(&mut self, cx: &Contexto, d: Datos, total: f64, efectivo: f64, segundo_monto: f64) {
        let descuento = self.descuento;
        let segundo = self.normal.segundo;
        let base = if descuento.is_some() { d.total_iva } else { total };
        let creado = ahora_iso();
        let campos = cobros::campos_descuento(descuento.as_ref(), redondear(base - total, 2), redondear(d.total_iva, 2), None);
        let orden = cobros::orden(
            &d.venta(&creado),
            None,
            redondear(total, 2),
            cobros::items_orden(&d.items, false),
            None,
            Some(cobros::pagos_mixtos(efectivo, segundo, segundo_monto)),
            campos,
        );
        self.lanzar(cx, true, move || match d.crear_orden(&orden) {
            Ok(resultado) => {
                if let Some(resultado) = resultado {
                    d.imprimir_venta(cobros::con_id(orden, &resultado));
                }
                d.exito("Orden completada exitosamente");
                aplicar(|c, cx| {
                    c.normal.mixto_abierto = false;
                    c.vaciar(cx);
                    c.normal.reiniciar();
                })
            }
            Err(_) => {
                d.error("Error al procesar la orden");
                aplicar(|c, _| c.normal.reiniciar())
            }
        });
    }

    fn generar_qr_mixto(&mut self, cx: &Contexto, d: Datos, efectivo: f64, qr: f64) {
        let descuento = self.descuento;
        if d.mp_deshabilitado() {
            let creado = ahora_iso();
            let campos = cobros::campos_descuento(descuento.as_ref(), descuento.map_or(0.0, |x| x.monto), redondear(d.total_iva, 2), None);
            let orden = cobros::orden(
                &d.venta(&creado),
                None,
                redondear(efectivo + qr, 2),
                cobros::items_orden(&d.items, false),
                Some("COMPLETADA"),
                Some(cobros::pagos_mixtos(efectivo, "qr", qr)),
                campos,
            );
            self.lanzar(cx, true, move || match d.crear_orden(&orden) {
                Ok(resultado) => {
                    let texto = match resultado {
                        Some(resultado) => {
                            d.imprimir_venta(cobros::con_id(orden, &resultado));
                            "Orden mixta completada exitosamente"
                        }
                        None => "Orden completada exitosamente",
                    };
                    d.exito(texto);
                    aplicar(|c, cx| {
                        c.vaciar(cx);
                        c.normal.reiniciar();
                        c.normal.mixto_abierto = false;
                    })
                }
                Err(mensaje) => {
                    d.error(format!("Error al procesar la orden: {mensaje}"));
                    aplicar(|c, _| c.normal.reiniciar())
                }
            });
            return;
        }
        self.normal.metodo = Some("split");
        let monto = descuento.map_or(efectivo + qr, |x| efectivo + qr - x.monto);
        let items = cobros::items_orden(&d.items, false);
        let payload = cobros::Qr {
            monto,
            descripcion: format!("Pago mixto: QR ${} + Efectivo ${}", cobros::fijo(qr, 2), cobros::fijo(efectivo, 2)),
            usuario: &d.usuario,
            items: items.clone(),
            afip: false,
            mixto: true,
            efectivo,
        }
        .payload(cobros::descuento_qr(descuento.as_ref(), redondear(d.total_iva, 2)));
        self.lanzar(cx, true, move || {
            let resultado = d.generar_qr(&payload, "Error al generar el QR para pago mixto").and_then(|r| {
                if verdad(&r["qrData"]) {
                    Ok(r)
                } else {
                    Err("La respuesta del backend no incluyó los datos del QR para el pago mixto.".to_string())
                }
            });
            match resultado {
                Ok(r) => aplicar(move |c, cx| {
                    let orden = texto_js(&r["id"]);
                    c.normal.qr = Some(con_qr(r, qr, items));
                    c.normal.estado = Some("PENDIENTE".into());
                    c.normal.mixto_abierto = false;
                    c.normal.iniciar_sondeo(orden, cx.egui.input(|i| i.time));
                    c.abrir_qr(cx);
                    c.normal.liberar();
                }),
                Err(mensaje) => {
                    d.error(format!("Error: {mensaje}"));
                    aplicar(|c, _| c.normal.reiniciar())
                }
            }
        });
    }

    fn generar_qr_mixto_afip(&mut self, cx: &Contexto, d: Datos, efectivo: f64, qr: f64) {
        if d.usuario.is_null() {
            d.error("Debes iniciar sesión para realizar una orden");
            return;
        }
        let descuento = self.descuento;
        let subtotal = cobros::subtotal_mixto_afip(efectivo + qr, descuento.as_ref());
        let items = cobros::items_orden(&d.items, false);
        let payload = cobros::Qr {
            monto: redondear(qr, 2),
            descripcion: format!("Pago mixto AFIP: QR ${} + Efectivo ${}", cobros::fijo(qr, 2), cobros::fijo(efectivo, 2)),
            usuario: &d.usuario,
            items: items.clone(),
            afip: true,
            mixto: true,
            efectivo,
        }
        .payload(cobros::descuento_qr_afip(descuento.as_ref(), redondear(subtotal, 2), descuento.map_or(0.0, |x| x.monto), false));
        self.lanzar(cx, true, move || {
            let resultado = d.generar_qr(&payload, "Error al generar el QR para pago mixto AFIP").and_then(|r| {
                if verdad(&r["qrData"]) && verdad(&r["id"]) {
                    Ok(r)
                } else {
                    Err("La respuesta del backend no incluyó 'qrData' o 'id'.".to_string())
                }
            });
            match resultado {
                Ok(r) => aplicar(move |c, cx| {
                    let orden = texto_js(&r["id"]);
                    c.afip.qr = Some(con_qr(r, qr, items));
                    c.afip.estado = Some("PENDIENTE".into());
                    c.afip.mixto_abierto = false;
                    c.afip.iniciar_sondeo(orden, cx.egui.input(|i| i.time));
                    c.abrir_qr(cx);
                }),
                Err(mensaje) => {
                    d.error(mensaje);
                    aplicar(|c, _| c.afip.reiniciar())
                }
            }
        });
    }

    pub(super) fn cerrar_qr_usuario(&mut self) {
        self.qr_abierto = false;
        self.flujo_afip = false;
    }

    pub(super) fn cancelar_qr(&mut self) {
        self.normal.sondeo = None;
        self.normal.qr = None;
        self.normal.estado = None;
        self.normal.liberar();
        self.qr_abierto = false;
    }

    pub(super) fn abrir_manual(&mut self, cx: &Contexto) {
        let afip = self.flujo_afip && cx.global.negocio.as_ref().is_some_and(|n| n["facturacionHabilitada"] == true);
        let lado = if afip { Lado::Afip } else { Lado::Normal };
        self.proc_mut(lado).manual = Some(super::Manual { enfocar: true, ..Default::default() });
    }

    pub(super) fn enviar_manual(&mut self, cx: &mut Contexto, lado: Lado) {
        let d = Datos::de(cx);
        let p = self.proc_mut(lado);
        if p.enviando_manual() {
            return;
        }
        let clave = if lado == Lado::Normal { "id" } else { "orderId" };
        let Some(orden) = p.qr.as_ref().map(|qr| qr[clave].clone()).filter(verdad) else {
            d.error("No hay una orden QR pendiente para completar.");
            return;
        };
        if lado == Lado::Afip {
            p.sondeo = None;
        }
        if let Some(manual) = p.manual.as_mut() {
            manual.enviando = true;
        }
        let requiere_afip = lado == Lado::Afip || self.flujo_afip;
        self.lanzar(cx, true, move || {
            let cuerpo = json!({ "orderId": orden, "requiresAfipInvoice": requiere_afip });
            let resultado = d.pedir("POST", "/api/mercadopago/manual-complete", Some(&cuerpo)).and_then(|r| {
                if r.ok() {
                    Ok(r.cuerpo)
                } else {
                    Err(mensaje(&r.cuerpo, "message", "Error al completar el pago manualmente"))
                }
            });
            match resultado {
                Ok(resultado) => aplicar(move |c, cx| {
                    let p = c.proc_mut(lado);
                    p.manual = None;
                    p.sondeo = None;
                    if lado == Lado::Normal {
                        c.imprimir_manual(cx, d);
                    } else {
                        c.terminar_manual_afip(cx, d, resultado);
                    }
                }),
                Err(mensaje) => {
                    if lado == Lado::Normal {
                        d.error(mensaje);
                    } else {
                        d.error(format!("Error al completar el pago: {mensaje}"));
                    }
                    aplicar(move |c, _| {
                        if let Some(manual) = c.proc_mut(lado).manual.as_mut() {
                            manual.enviando = false;
                        }
                    })
                }
            }
        });
    }

    fn imprimir_manual(&mut self, cx: &Contexto, d: Datos) {
        let qr = self.normal.qr.clone().unwrap_or(Value::Null);
        let creado = ahora_iso();
        self.lanzar(cx, true, move || {
            let monto = cobros::numero(&qr["monto"]);
            let efectivo = cobros::numero(&qr["cashAmount"]);
            let mixto = (verdad(&qr["isSplitPayment"]) && verdad(&qr["cashAmount"])) || efectivo > 0.0;
            let mut datos = json!({
                "id": qr["id"],
                "idReal": qr["id"],
                "metodoPago": if mixto { "split" } else { "qr" },
                "total": if mixto { monto + efectivo } else { monto },
                "items": if qr["items"].is_array() { qr["items"].clone() } else { json!([]) },
                "vendedorId": d.usuario["id"],
                "sucursalId": d.usuario["sucursalId"],
                "vendedor": d.usuario["nombre"],
                "businessName": d.nombre_comercio(),
                "estado": "COMPLETADA",
                "createdAt": creado,
            });
            if mixto {
                datos["pagos"] = cobros::pagos_mixtos(efectivo, "qr", monto);
            }
            d.imprimir_venta(datos);
            d.exito(if mixto {
                "¡Pago mixto completado e impreso exitosamente!"
            } else {
                "¡Pago completado e impreso exitosamente!"
            });
            aplicar(|c, cx| {
                c.qr_abierto = false;
                c.vaciar(cx);
                c.normal.reiniciar();
            })
        });
    }

    fn terminar_manual_afip(&mut self, cx: &Contexto, d: Datos, resultado: Value) {
        self.lanzar(cx, true, move || {
            sleep(Duration::from_millis(500));
            if verdad(&resultado["success"]) && verdad(&resultado["facturaId"]) && verdad(&resultado["cae"]) {
                let numero = resultado.get("numero").map_or("undefined".to_string(), texto_js);
                d.exito(format!("¡Pago completado! Factura AFIP N° {numero} generada exitosamente"));
            } else {
                d.exito("¡Pago completado y factura AFIP generada!");
            }
            aplicar(|c, cx| {
                c.qr_abierto = false;
                c.vaciar(cx);
                c.afip.reiniciar();
            })
        });
    }

    pub(super) fn sondear(&mut self, cx: &Contexto) {
        let ahora = cx.egui.input(|i| i.time);
        for lado in [Lado::Normal, Lado::Afip] {
            let p = self.proc_mut(lado);
            let Some(sondeo) = p.sondeo.as_mut() else { continue };
            if sondeo.en_vuelo {
                continue;
            }
            if ahora < sondeo.siguiente {
                cx.egui.request_repaint_after(Duration::from_secs_f64(sondeo.siguiente - ahora));
                continue;
            }
            sondeo.conteo += 1;
            if lado == Lado::Afip && sondeo.conteo > 150 {
                p.sondeo = None;
                toast::error(&cx.egui, "Tiempo de espera agotado. Verifique manualmente el estado del pago.");
                p.reiniciar();
                continue;
            }
            sondeo.en_vuelo = true;
            sondeo.siguiente = ahora + if lado == Lado::Normal { 3.0 } else { 2.0 };
            let (generacion, conteo) = (sondeo.generacion, sondeo.conteo);
            let ruta = format!("/api/mercadopago/check-status/{}", sondeo.orden);
            let d = Datos::de(cx);
            self.lanzar(cx, false, move || {
                let respuesta = d.pedir("GET", &ruta, None);
                aplicar(move |c, cx| c.respuesta_sondeo(cx, lado, generacion, conteo, respuesta))
            });
        }
    }

    fn respuesta_sondeo(&mut self, cx: &mut Contexto, lado: Lado, generacion: u64, conteo: u32, respuesta: Result<Respuesta, String>) {
        let p = self.proc_mut(lado);
        match p.sondeo.as_mut() {
            Some(s) if s.generacion == generacion => s.en_vuelo = false,
            _ => return,
        }
        let datos = match respuesta {
            Ok(r) if r.estado == 404 => {
                p.sondeo = None;
                return;
            }
            Ok(r) if r.ok() => r.cuerpo,
            _ => {
                if lado == Lado::Afip && conteo > 10 {
                    p.sondeo = None;
                    toast::error(&cx.egui, "Error al verificar el estado del pago. Verifique manualmente.");
                    p.reiniciar();
                }
                return;
            }
        };
        let d = Datos::de(cx);
        if lado == Lado::Normal {
            self.normal.estado = Some(cobros::estado_qr(&datos["status"]));
            let completado = verdad(&datos["isCompleted"]);
            if !completado && !verdad(&datos["isCancelled"]) {
                return;
            }
            self.normal.sondeo = None;
            if completado {
                d.exito("¡Pago completado exitosamente!");
                let qr = self.normal.qr.clone().unwrap_or(Value::Null);
                let imprimir = !self.normal.enviando_manual();
                let creado = ahora_iso();
                self.lanzar(cx, true, move || {
                    if imprimir {
                        d.finalizar_qr(&datos, &qr, &creado);
                    }
                    aplicar(|c, cx| {
                        c.qr_abierto = false;
                        c.normal.reiniciar();
                        c.vaciar(cx);
                    })
                });
            } else {
                d.error("El pago ha sido cancelado o rechazado");
                self.qr_abierto = false;
                self.normal.reiniciar();
            }
            return;
        }
        let estado = datos["status"].as_str().unwrap_or("").to_lowercase();
        let orden = self.afip.sondeo.as_ref().map(|s| s.orden.clone()).unwrap_or_default();
        match estado.as_str() {
            "completed" | "approved" => {
                self.afip.sondeo = None;
                d.exito("¡Pago QR completado exitosamente!");
                let factura = datos["afipInvoice"].clone();
                if verdad(&factura["created"]) {
                    if verdad(&factura["numero"]) {
                        d.exito(format!("Factura AFIP N° {} generada automáticamente", texto_js(&factura["numero"])));
                    } else {
                        d.exito("Factura AFIP generada automáticamente");
                    }
                    self.lanzar(cx, false, move || {
                        d.imprimir_afip(&factura);
                        d.exito("Factura AFIP impresa correctamente");
                        aplicar(|c, cx| {
                            c.qr_abierto = false;
                            c.afip.mixto_abierto = false;
                            c.diferir(
                                cx,
                                2000,
                                aplicar(|c, cx| {
                                    c.vaciar(cx);
                                    c.afip.reiniciar();
                                }),
                            );
                        })
                    });
                } else {
                    d.exito("¡Pago QR completado! Verificando factura AFIP...");
                    self.qr_abierto = false;
                    self.afip.mixto_abierto = false;
                    if !self.afip.enviando_manual() {
                        let ruta = format!("/api/mercadopago/check-status/{orden}");
                        self.lanzar(cx, false, move || {
                            sleep(Duration::from_millis(3000));
                            let _ = d.pedir("GET", &ruta, None);
                            aplicar(|c, cx| {
                                c.qr_abierto = false;
                                c.vaciar(cx);
                                c.afip.reiniciar();
                            })
                        });
                    }
                }
            }
            "failed" | "cancelled" | "rejected" => {
                self.afip.sondeo = None;
                d.error("El pago ha sido cancelado o rechazado");
                self.qr_abierto = false;
                self.afip.mixto_abierto = false;
                self.afip.reiniciar();
            }
            _ => {}
        }
    }

    pub(super) fn confirmar_descuento(&mut self, descuento: Descuento) {
        self.descuento = Some(descuento);
        self.descuento_dialogo = None;
        if self.accion_descuento.take() == Some(Lado::Afip) {
            self.flujo_afip = true;
            self.abrir_pago(Lado::Afip);
        } else {
            self.flujo_afip = false;
            self.abrir_pago(Lado::Normal);
        }
    }

    pub(super) fn cancelar_carrito(&mut self, cx: &mut Contexto) {
        if cx.global.negocio.as_ref().is_some_and(|n| n["cancelacionHabilitada"] == true) {
            if cx.global.usuario.is_none() {
                toast::error(&cx.egui, "Debes iniciar sesión para cancelar");
            } else {
                let referencia = format!("TEMP-{}", chrono::Utc::now().timestamp_millis());
                self.cancelacion.pendiente = Some(CancelacionPendiente {
                    referencia,
                    monto: cx.global.carritos.total(),
                    productos: cobros::productos_cancelacion(cx.global.carritos.items()),
                });
                self.cancelacion.abierto = true;
                self.cancelacion.error.clear();
                self.cancelacion.enfocar = true;
                return;
            }
        }
        cx.global.carritos.vaciar();
        self.cancelar_abierto = false;
        self.resultado = Some(super::ResultadoCobro::Cancelado);
        toast::exito(&cx.egui, "Carrito cancelado");
    }

    pub(super) fn confirmar_cancelacion(&mut self, cx: &mut Contexto) {
        if let Err(error) = cobros::validar_motivo(&self.cancelacion.motivo) {
            self.cancelacion.error = error.to_string();
            return;
        }
        self.cancelacion.error.clear();
        let d = Datos::de(cx);
        let Some(pendiente) = self.cancelacion.pendiente.as_ref() else {
            d.error("Debes ingresar un motivo para la cancelación");
            return;
        };
        if d.usuario.is_null() {
            d.error("Debes iniciar sesión para cancelar");
            return;
        }
        self.cancelacion.enviando = true;
        let cuerpo = cobros::cancelacion(
            &d.negocio,
            &d.usuario,
            &pendiente.referencia,
            &self.cancelacion.motivo,
            pendiente.monto,
            pendiente.productos.clone(),
        );
        self.lanzar(cx, true, move || {
            let resultado = d.pedir("POST", "/api/cancelaciones", Some(&cuerpo)).and_then(|r| {
                if r.ok() {
                    Ok(r.cuerpo)
                } else {
                    Err(mensaje(&r.cuerpo, "message", "Error al registrar cancelación"))
                }
            });
            match resultado {
                Ok(r) => {
                    let id = cobros::primero([&r["cancelacion"]["idReal"], &r["id"]]).map_or("N/A".to_string(), texto_js);
                    d.exito(format!("Cancelación #{id} registrada exitosamente"));
                    aplicar(|c, cx| {
                        c.cancelacion = Default::default();
                        cx.global.carritos.vaciar();
                        c.cancelar_abierto = false;
                        c.resultado = Some(super::ResultadoCobro::Cancelado);
                    })
                }
                Err(mensaje) => {
                    d.error(format!("Error al registrar cancelación: {mensaje}"));
                    aplicar(|c, _| c.cancelacion.enviando = false)
                }
            }
        });
    }
}

impl Datos {
    fn finalizar_qr(&self, estado: &Value, qr: &Value, creado: &str) {
        self.exito("¡Pago completado! Procesando ticket...");
        if verdad(&estado["orderData"]) || verdad(&estado["orderId"]) {
            let datos = if verdad(&estado["orderData"]) {
                estado["orderData"].clone()
            } else {
                let mut datos = json!({
                    "id": estado["orderId"],
                    "idReal": estado["orderId"],
                    "metodoPago": "qr",
                    "total": if qr["monto"].is_null() { json!(0) } else { qr["monto"].clone() },
                    "items": if qr["items"].is_array() { qr["items"].clone() } else { json!([]) },
                    "vendedorId": self.usuario["id"],
                    "sucursalId": self.usuario["sucursalId"],
                    "vendedor": self.usuario["nombre"],
                    "businessName": self.nombre_comercio(),
                    "estado": "COMPLETADA",
                    "createdAt": creado,
                });
                let tiene = verdad(&qr["tieneDescuento"]);
                if tiene && verdad(&qr["discountData"]) {
                    datos["tieneDescuento"] = json!(true);
                    for clave in ["tipoDescuento", "valorDescuento", "subtotalSinDescuento", "subtotal"] {
                        datos[clave] = qr[clave].clone();
                    }
                    datos["discountData"] = json!({
                        "type": qr["discountData"]["type"],
                        "value": qr["discountData"]["value"],
                        "amount": qr["discountData"]["amount"],
                    });
                } else if !tiene {
                    datos["tieneDescuento"] = json!(false);
                }
                datos
            };
            self.imprimir_venta(datos);
            self.exito("Ticket impreso correctamente");
        }
        self.exito("¡Pago completado exitosamente!");
    }
}
