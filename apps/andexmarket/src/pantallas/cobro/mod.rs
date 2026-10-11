mod dialogos;
mod flujos;

use eframe::egui::{self, Ui};
use nucleo::cobros::Descuento;
use serde_json::Value;

use crate::app::Contexto;
use crate::tareas::Tarea;
use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionCobro {
    Cancelar,
    Pagar,
    PagarAfip,
    PagarConDescuento,
    PagarAfipConDescuento,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResultadoCobro {
    VentaRegistrada,
    Cancelado,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lado {
    Normal,
    Afip,
}

type Aplicar = Box<dyn for<'a, 'b, 'c> FnOnce(&'a mut Cobro, &'b mut Contexto<'c>) + Send>;

fn aplicar(f: impl for<'a, 'b, 'c> FnOnce(&'a mut Cobro, &'b mut Contexto<'c>) + Send + 'static) -> Aplicar {
    Box::new(f)
}

struct Pendiente {
    bloquea: bool,
    tarea: Tarea<Aplicar>,
}

#[derive(Default)]
struct Efectivo {
    exacto: bool,
    original: f64,
    cobrar: f64,
    aplicando: bool,
    pagado: String,
    anterior: String,
    error: Option<&'static str>,
    enfocar: bool,
}

#[derive(Default)]
struct Manual {
    clave: String,
    enviando: bool,
    enfocar: bool,
}

struct Sondeo {
    orden: String,
    siguiente: f64,
    en_vuelo: bool,
    inicio: f64,
    errores: u32,
    generacion: u64,
}

impl Sondeo {
    fn fallar(&mut self) -> bool {
        self.errores += 1;
        self.errores >= nucleo::cobros::SONDEO_MAXIMO_ERRORES
    }
}

struct Procesador {
    lado: Lado,
    procesando: bool,
    metodo: Option<&'static str>,
    efectivo: Option<Efectivo>,
    mixto_abierto: bool,
    efectivo_mixto: String,
    segundo: &'static str,
    enfocar_mixto: bool,
    qr: Option<Value>,
    ultimo_qr: Value,
    estado: Option<String>,
    manual: Option<Manual>,
    sondeo: Option<Sondeo>,
    generacion: u64,
}

impl Procesador {
    fn new(lado: Lado) -> Self {
        Procesador {
            lado,
            procesando: false,
            metodo: None,
            efectivo: None,
            mixto_abierto: false,
            efectivo_mixto: String::new(),
            segundo: "tarjeta",
            enfocar_mixto: false,
            qr: None,
            ultimo_qr: Value::Null,
            estado: None,
            manual: None,
            sondeo: None,
            generacion: 0,
        }
    }

    fn reiniciar(&mut self) {
        self.qr = None;
        self.estado = None;
        self.liberar();
        self.manual = None;
        self.efectivo = None;
        self.efectivo_mixto.clear();
        self.segundo = "tarjeta";
        self.sondeo = None;
    }

    fn fijar_qr(&mut self, qr: Value) {
        self.ultimo_qr = qr.clone();
        self.qr = Some(qr);
    }

    fn liberar(&mut self) {
        self.procesando = false;
        self.metodo = None;
    }

    fn iniciar_sondeo(&mut self, orden: String, ahora: f64) {
        if self.lado == Lado::Normal && self.sondeo.as_ref().is_some_and(|s| s.orden == orden) {
            return;
        }
        self.generacion += 1;
        let siguiente = if self.lado == Lado::Normal { ahora } else { ahora + 2.0 };
        self.sondeo = Some(Sondeo { orden, siguiente, en_vuelo: false, inicio: ahora, errores: 0, generacion: self.generacion });
    }

    fn enviando_manual(&self) -> bool {
        self.manual.as_ref().is_some_and(|m| m.enviando)
    }

    fn con_dialogo(&self) -> bool {
        self.efectivo.is_some() || self.mixto_abierto || self.manual.is_some()
    }
}

#[derive(Default)]
struct DialogoDescuento {
    cantidad_fija: bool,
    porcentaje: String,
    monto: String,
    enfocar: bool,
}

struct CancelacionPendiente {
    referencia: String,
    monto: f64,
    productos: Vec<Value>,
}

#[derive(Default)]
struct Cancelacion {
    abierto: bool,
    motivo: String,
    error: String,
    enviando: bool,
    pendiente: Option<CancelacionPendiente>,
    enfocar: bool,
}

#[derive(Clone, Default, Debug, PartialEq)]
struct Resumen {
    total: f64,
    metodo: String,
    vuelto: Option<f64>,
    comprobante: Option<String>,
    cae: Option<String>,
    impreso: Option<bool>,
}

struct Completada {
    resumen: Resumen,
    desde: f64,
}

pub struct Cobro {
    cancelar_abierto: bool,
    cancelacion: Cancelacion,
    descuento_dialogo: Option<DialogoDescuento>,
    accion_descuento: Option<Lado>,
    descuento: Option<Descuento>,
    pago: Option<Lado>,
    flujo_afip: bool,
    qr_abierto: bool,
    normal: Procesador,
    afip: Procesador,
    tareas: Vec<Pendiente>,
    imagen_qr: Option<(String, Option<String>)>,
    teclado: bool,
    resultado: Option<ResultadoCobro>,
    venta: Option<Resumen>,
    completada: Option<Completada>,
}

impl Default for Cobro {
    fn default() -> Self {
        Cobro {
            cancelar_abierto: false,
            cancelacion: Cancelacion::default(),
            descuento_dialogo: None,
            accion_descuento: None,
            descuento: None,
            pago: None,
            flujo_afip: false,
            qr_abierto: false,
            normal: Procesador::new(Lado::Normal),
            afip: Procesador::new(Lado::Afip),
            tareas: Vec::new(),
            imagen_qr: None,
            teclado: true,
            resultado: None,
            venta: None,
            completada: None,
        }
    }
}

impl Cobro {
    pub fn iniciar(&mut self, accion: AccionCobro, cx: &mut Contexto) {
        if cx.global.carritos.items().is_empty() {
            let detalle = if accion == AccionCobro::Cancelar {
                "El carrito ya está vacío"
            } else {
                "Agrega al menos un producto antes de continuar"
            };
            ui::v2::toast::con_detalle(&cx.egui, ui::v2::toast::Tipo::Error, "No hay productos en el carrito", detalle);
            return;
        }
        match accion {
            AccionCobro::Cancelar => self.cancelar_abierto = true,
            AccionCobro::Pagar => {
                self.afip.reiniciar();
                self.flujo_afip = false;
                self.descuento = None;
                self.abrir_pago(Lado::Normal);
            }
            AccionCobro::PagarAfip => {
                self.normal.reiniciar();
                self.flujo_afip = true;
                self.descuento = None;
                self.abrir_pago(Lado::Afip);
            }
            AccionCobro::PagarConDescuento => self.abrir_descuento(Lado::Normal),
            AccionCobro::PagarAfipConDescuento => self.abrir_descuento(Lado::Afip),
        }
    }

    pub fn ocupado(&self) -> bool {
        self.cancelar_abierto
            || self.cancelacion.abierto
            || self.descuento_dialogo.is_some()
            || self.pago.is_some()
            || self.qr_abierto
            || self.normal.con_dialogo()
            || self.afip.con_dialogo()
            || self.tareas.iter().any(|t| t.bloquea)
    }

    pub fn venta_cobrada(&self) -> bool {
        self.completada.is_some()
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) -> Option<ResultadoCobro> {
        cx.egui.input(|i| {
            for evento in &i.events {
                match evento {
                    egui::Event::Key { pressed: true, .. } => self.teclado = true,
                    egui::Event::PointerButton { pressed: true, .. } => self.teclado = false,
                    _ => {}
                }
            }
        });
        self.atender_tareas(cx);
        self.sondear(cx);
        self.limpiar_normal();
        self.dibujar(ui, cx);
        self.limpiar_normal();
        self.resultado.take()
    }

    fn proc_mut(&mut self, lado: Lado) -> &mut Procesador {
        match lado {
            Lado::Normal => &mut self.normal,
            Lado::Afip => &mut self.afip,
        }
    }

    fn abrir_pago(&mut self, lado: Lado) {
        self.pago = Some(lado);
        self.proc_mut(lado).reiniciar();
    }

    fn abrir_descuento(&mut self, lado: Lado) {
        self.accion_descuento = Some(lado);
        self.descuento_dialogo = Some(DialogoDescuento { enfocar: true, ..Default::default() });
    }

    fn abrir_qr(&mut self) {
        self.qr_abierto = true;
    }

    fn vaciar(&mut self, cx: &mut Contexto) {
        cx.global.carritos.vaciar();
        self.resultado = Some(ResultadoCobro::VentaRegistrada);
        if let Some(resumen) = self.venta.take() {
            self.completada = Some(Completada { resumen, desde: cx.egui.input(|i| i.time) });
        }
    }

    fn anotar(&mut self, total: f64, metodo: String, vuelto: Option<f64>) {
        self.venta = Some(Resumen { total, metodo, vuelto, ..Default::default() });
    }

    fn anotar_impresion(&mut self, impreso: Option<bool>) {
        if let Some(venta) = self.venta.as_mut() {
            venta.impreso = impreso;
        }
    }

    fn anotar_factura(&mut self, datos: &Value) {
        if let Some(venta) = self.venta.as_mut() {
            (venta.comprobante, venta.cae) = comprobante(datos);
        }
    }

    fn vuelto(&self, lado: Lado) -> Option<f64> {
        let p = match lado {
            Lado::Normal => &self.normal,
            Lado::Afip => &self.afip,
        };
        p.efectivo.as_ref().and_then(|e| nucleo::cobros::parse_float(&e.pagado).map(|pagado| pagado - e.cobrar)).filter(|v| *v > 0.0)
    }

    fn limpiar_normal(&mut self) {
        let sin_dialogos = !self.qr_abierto
            && self.pago != Some(Lado::Normal)
            && !self.normal.mixto_abierto
            && !self.afip.mixto_abierto
            && self.normal.efectivo.is_none();
        if sin_dialogos && (self.normal.procesando || self.normal.metodo.is_some()) {
            self.normal.reiniciar();
        }
    }

    fn lanzar(&mut self, cx: &Contexto, bloquea: bool, trabajo: impl FnOnce() -> Aplicar + Send + 'static) {
        self.tareas.push(Pendiente { bloquea, tarea: Tarea::lanzar(&cx.egui, trabajo) });
    }

    fn diferir(&mut self, cx: &Contexto, milisegundos: u64, despues: Aplicar) {
        self.lanzar(cx, false, move || {
            std::thread::sleep(std::time::Duration::from_millis(milisegundos));
            despues
        });
    }

    fn bloqueado(&self) -> bool {
        self.tareas.iter().any(|t| t.bloquea)
    }

    fn atender_tareas(&mut self, cx: &mut Contexto) {
        let mut listas = Vec::new();
        self.tareas.retain_mut(|p| match p.tarea.listo() {
            Some(aplicar) => {
                listas.push(aplicar);
                false
            }
            None => true,
        });
        for aplicar in listas {
            aplicar(self, cx);
        }
    }
}

fn dinero(valor: f64) -> String {
    let signo = if valor < -0.004 { "−" } else { "" };
    format!("{signo}${}", nucleo::formato::numero_con(nucleo::cobros::redondear(valor.abs(), 2), 0, 2))
}

fn nombre_metodo(metodo: &str) -> &'static str {
    match metodo {
        "efectivo" => "Efectivo",
        "tarjeta" => "Tarjeta",
        "qr" => "Transferencia",
        "split" => "Mixto",
        _ => "Otro medio",
    }
}

fn texto_mixto(efectivo: f64, segundo: &str, resto: f64) -> String {
    let medio = if segundo == "tarjeta" { "con tarjeta" } else { "por transferencia" };
    format!("Mixto · {} en efectivo + {} {medio}", dinero(efectivo), dinero(resto))
}

fn comprobante(datos: &Value) -> (Option<String>, Option<String>) {
    use nucleo::cobros::{primero, verdad};
    let texto = |v: &Value| flujos::texto_js(v);
    let numero = primero([&datos["factura"]["numero"], &datos["numero"], &datos["numeroFactura"]]).filter(|v| verdad(v)).map(texto);
    let tipo = primero([&datos["afip"]["tipoFactura"], &datos["tipoFactura"], &datos["factura"]["tipoFactura"]])
        .filter(|v| verdad(v))
        .map(texto)
        .map(|t| {
            let letra = t.trim().trim_start_matches("FACTURA").trim_start_matches("Factura").trim();
            format!("Factura {letra}")
        })
        .unwrap_or_else(|| "Factura".to_string());
    let cae = primero([&datos["afip"]["cae"], &datos["cae"]]).filter(|v| verdad(v)).map(texto);
    (numero.map(|n| format!("{tipo} {n}")), cae)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use serde_json::json;

    #[test]
    fn dinero_como_el_diseno() {
        assert_eq!(dinero(17806.55), "$17.806,55");
        assert_eq!(dinero(17800.0), "$17.800");
        assert_eq!(dinero(1793.5), "$1.793,5");
        assert_eq!(dinero(-6.55), "−$6,55");
        assert_eq!(dinero(0.0), "$0");
    }

    #[test]
    fn comprobante_de_la_factura_afip() {
        let datos = json!({ "factura": { "numero": "0002-00000213" }, "tipoFactura": "FACTURA B", "cae": "75412369899123" });
        assert_eq!(comprobante(&datos), (Some("Factura B 0002-00000213".into()), Some("75412369899123".into())));
        let manual = json!({ "numero": 45, "cae": "123" });
        assert_eq!(comprobante(&manual), (Some("Factura 45".into()), Some("123".into())));
        assert_eq!(comprobante(&json!({})), (None, None));
    }

    fn con_sondeo(lado: Lado) -> Procesador {
        let mut p = Procesador::new(lado);
        p.fijar_qr(json!({ "id": 9023, "orderId": 9023 }));
        p.estado = Some("PENDIENTE".into());
        p.iniciar_sondeo("9023".into(), 10.0);
        p
    }

    #[test]
    fn sondeo_se_corta_con_errores_seguidos() {
        let mut p = con_sondeo(Lado::Normal);
        let s = p.sondeo.as_mut().unwrap();
        for _ in 1..nucleo::cobros::SONDEO_MAXIMO_ERRORES {
            assert!(!s.fallar());
        }
        s.errores = 0;
        for _ in 1..nucleo::cobros::SONDEO_MAXIMO_ERRORES {
            assert!(!s.fallar());
        }
        assert!(s.fallar());
        assert_eq!(s.inicio, 10.0);
    }

    #[test]
    fn cancelar_o_cerrar_el_qr_corta_el_sondeo_afip() {
        for cancelar in [true, false] {
            let mut cobro = Cobro { flujo_afip: true, qr_abierto: true, afip: con_sondeo(Lado::Afip), normal: con_sondeo(Lado::Normal), ..Default::default() };
            cobro.afip.procesando = true;
            if cancelar {
                cobro.cancelar_qr();
            } else {
                cobro.cerrar_qr_usuario();
            }
            assert!(!cobro.qr_abierto);
            assert!(cobro.afip.sondeo.is_none());
            assert!(cobro.normal.sondeo.is_none());
            assert!(!cobro.afip.procesando);
        }
    }

    #[test]
    fn qr_afip_con_order_id_no_arranca_el_sondeo_normal() {
        let mut cobro = Cobro { flujo_afip: true, afip: con_sondeo(Lado::Afip), ..Default::default() };
        cobro.normal.fijar_qr(json!({ "id": 9023, "orderId": 9023, "monto": 22449.5 }));
        cobro.abrir_qr();
        assert!(cobro.qr_abierto);
        assert!(cobro.normal.sondeo.is_none());
        assert!(cobro.afip.sondeo.is_some());
    }

    #[test]
    fn ticket_afip_con_la_respuesta_del_backend() {
        let negocio = json!({ "nombre": "Verduleria La Huerta", "configuracionAfip": { "razonSocial": "La Huerta SRL", "cuit": "30712345678", "condicionIva": "Responsable Inscripto" } });
        let usuario = json!({ "id": 11, "nombre": "Lucia", "sucursalId": 1 });
        let factura = nucleo::cobros::Afip {
            negocio: &negocio,
            usuario: &usuario,
            metodo: "efectivo",
            items: vec![json!({ "productoId": 1, "cantidad": 1, "precio": 1000.0, "subtotal": 1000.0, "nombre": "Tomate" })],
            total: 1000.0,
            subtotal_iva: 1000.0,
            descuento: None,
            mixto: None,
        };
        let respuesta = json!({
            "message": "Factura AFIP creada correctamente",
            "factura": { "id": 88, "numero": "0003-00000045", "numeroFactura": "45", "puntoVenta": "0003", "pagos": [] },
            "afip": { "cae": "75412369899123", "vencimientoCae": "20261020" },
        });
        let datos = factura.impresion(&respuesta, "10/10/2026 12:00:00");
        let bytes = impresion::tickets::afip::afip(&datos, std::path::Path::new("sin-logo.png"), impresion::tickets::hora_argentina());
        let texto = String::from_utf8_lossy(&bytes);
        assert!(texto.contains("Nro: 0003-00000045\n"), "{texto}");
        assert!(texto.contains("Fecha Vto CAE: 20/10/2026\n"), "{texto}");
    }
}
