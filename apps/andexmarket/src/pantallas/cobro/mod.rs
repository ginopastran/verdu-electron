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
    conteo: u32,
    generacion: u64,
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
        self.sondeo = Some(Sondeo { orden, siguiente, en_vuelo: false, conteo: 0, generacion: self.generacion });
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
            ui::toast::con_detalle(&cx.egui, ui::toast::Tipo::Error, "No hay productos en el carrito", detalle, 2.0);
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

    fn abrir_qr(&mut self, cx: &Contexto) {
        self.qr_abierto = true;
        if self.normal.sondeo.is_none() {
            let orden = self.normal.qr.as_ref().map(|qr| &qr["orderId"]).filter(|v| nucleo::cobros::verdad(v)).map(flujos::texto_js);
            if let Some(orden) = orden {
                self.normal.iniciar_sondeo(orden, cx.egui.input(|i| i.time));
            }
        }
    }

    fn vaciar(&mut self, cx: &mut Contexto) {
        cx.global.carritos.vaciar();
        self.resultado = Some(ResultadoCobro::VentaRegistrada);
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
