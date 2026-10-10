pub mod dibujo;
pub mod manual;
pub mod resultado;

use chrono::Utc;
use eframe::egui::{self, Color32, CursorIcon, Key, Modifiers, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::cierre::{self as logica, AvisoImpresion, FalloCierre};
use nucleo::entorno;
use serde_json::Value;

use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::degradado::mezclar;
use crate::ui::marca::{color, fuente, radio, tam};
use crate::ui::v2::{self, pintar_tecla, toast, EstiloTecla};
use crate::ui::{pintar_icono_solar_en, Peso};

pub enum ResultadoCierre {
    Cancelado,
    Completado,
}

pub enum RespuestaCierre {
    Hecho { periodo: String, cierre: Value, impresion: Value },
    MananaRequerida,
    Error(String),
}

#[derive(Default)]
pub struct DialogoCierre {
    abierto: bool,
    tarea: Option<Tarea<RespuestaCierre>>,
    periodo_en_curso: Option<&'static str>,
    manual: manual::CierreManual,
    resultado: resultado::DialogoResultadoCierre,
}

pub fn encabezados_app() -> Vec<(&'static str, &'static str)> {
    entorno::APP_ID.map(|id| vec![("X-App-ID", id)]).unwrap_or_default()
}

pub fn avisar_impresion(ctx: &egui::Context, resultado: &Value) {
    match logica::aviso_impresion(resultado) {
        AvisoImpresion::Exito => toast::exito(ctx, "Ticket de cierre impreso correctamente"),
        AvisoImpresion::ErrorImpresora(error) => toast::mostrar_aviso(
            ctx,
            toast::Tipo::Error,
            format!("Error de impresión: {error}"),
            Some("El cierre se completó correctamente pero no se pudo imprimir el ticket".into()),
            2.0,
        ),
        AvisoImpresion::ErrorGeneral(mensaje) => toast::error(ctx, format!("Error al imprimir el ticket de cierre: {mensaje}")),
    }
}

pub fn registrar_cierre(servicios: &Servicios, cuerpo: &Value, periodo: String) -> RespuestaCierre {
    let encabezados = encabezados_app();
    let Ok(respuesta) = servicios.api.pedir("POST", "/api/cierres", Some(cuerpo), &encabezados) else {
        return RespuestaCierre::Error("Failed to fetch".into());
    };
    match logica::fallo_cierre(respuesta.ok(), &respuesta.cuerpo) {
        Some(FalloCierre::MananaRequerida) => RespuestaCierre::MananaRequerida,
        Some(FalloCierre::Mensaje(mensaje)) => RespuestaCierre::Error(mensaje),
        None => {
            let impresion = imprimir(servicios, Comprobante::Cierre, respuesta.cuerpo.clone());
            RespuestaCierre::Hecho { periodo, cierre: respuesta.cuerpo, impresion }
        }
    }
}

#[derive(Clone, Copy)]
enum Accion {
    Periodo(&'static str),
    Manual,
}

const ATAJOS: [(Key, Accion); 4] = [
    (Key::Num1, Accion::Periodo("mañana")),
    (Key::Num2, Accion::Periodo("tarde")),
    (Key::Num3, Accion::Periodo("todo")),
    (Key::Num4, Accion::Manual),
];

fn atajo(ctx: &egui::Context) -> Option<Accion> {
    ctx.input_mut(|i| ATAJOS.iter().find(|(tecla, _)| i.consume_key(Modifiers::NONE, *tecla)).map(|(_, accion)| *accion))
}

impl DialogoCierre {
    pub fn abrir(&mut self) {
        self.abierto = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto || self.resultado.abierto()
    }

    pub fn cerrando(&self) -> bool {
        self.tarea.is_some()
    }

    pub fn manual_abierto(&self) -> bool {
        self.manual.abierto()
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) -> Option<ResultadoCierre> {
        let mut resultado = self.recibir(cx);
        if self.abierto && !self.manual.abierto() {
            let ocupado = self.cerrando();
            let en_curso = self.periodo_en_curso.filter(|_| ocupado);
            let respuesta = v2::Modal::new("cierre-periodo", "Cierre de caja")
                .subtitulo("Elegí el período. Presioná 1 a 4.")
                .icono("store")
                .ancho(520.0)
                .mostrar(&cx.egui, |ui| {
                    let mut accion = None;
                    v2::cuerpo_modal(ui, 20, |ui| {
                        let ancho = (ui.available_width() - 10.0) / 2.0;
                        for fila in 0..2 {
                            if fila > 0 {
                                ui.add_space(10.0);
                            }
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 10.0;
                                for indice in fila * 2..fila * 2 + 2 {
                                    let opcion = &OPCIONES[indice];
                                    if opcion_periodo(ui, ancho, indice + 1, opcion, ocupado, en_curso).clicked() {
                                        accion = Some(opcion.periodo.map_or(Accion::Manual, Accion::Periodo));
                                    }
                                }
                            });
                        }
                        ui.add_space(24.0);
                    });
                    if !ocupado {
                        accion = accion.or_else(|| atajo(ui.ctx()));
                    }
                    accion
                });
            match respuesta.interior {
                Some(Accion::Periodo(periodo)) => resultado = self.cerrar_caja(periodo, cx).or(resultado),
                Some(Accion::Manual) => self.manual.abrir(cx),
                None => {}
            }
            if respuesta.cerrar && self.abierto {
                self.abierto = false;
                resultado = Some(ResultadoCierre::Cancelado);
            }
        }
        if let Some(cierre) = self.manual.mostrar(ui, cx) {
            self.abierto = false;
            self.resultado.abrir(cierre);
            resultado = Some(ResultadoCierre::Completado);
        }
        self.resultado.mostrar(ui, cx);
        resultado
    }

    fn recibir(&mut self, cx: &mut Contexto) -> Option<ResultadoCierre> {
        match esperar(&mut self.tarea)? {
            RespuestaCierre::Hecho { periodo, cierre, impresion } => {
                avisar_impresion(&cx.egui, &impresion);
                toast::exito(&cx.egui, format!("Cierre de {periodo} realizado correctamente"));
                self.abierto = false;
                self.resultado.abrir(cierre);
                Some(ResultadoCierre::Completado)
            }
            RespuestaCierre::MananaRequerida => {
                toast::mostrar_aviso(
                    &cx.egui,
                    toast::Tipo::Error,
                    "No puedes realizar un cierre de tarde sin haber realizado el cierre de mañana del día actual.",
                    Some("Primero debes realizar el cierre de mañana".into()),
                    5.0,
                );
                None
            }
            RespuestaCierre::Error(mensaje) => {
                toast::error(&cx.egui, format!("Error: {mensaje}"));
                None
            }
        }
    }

    fn cerrar_caja(&mut self, periodo: &'static str, cx: &mut Contexto) -> Option<ResultadoCierre> {
        let Some(usuario) = cx.global.usuario.clone() else {
            toast::error(&cx.egui, "Debes iniciar sesión para realizar el cierre");
            return None;
        };
        if !logica::verdadero(usuario.get("permisos").and_then(|p| p.get("cierreDeCajaEnabled"))) {
            toast::error(&cx.egui, "No tienes permiso para realizar cierres de caja");
            self.abierto = false;
            return Some(ResultadoCierre::Cancelado);
        }
        let (inicio, fin) = logica::rango_periodo(periodo, &Utc::now());
        let cuerpo = logica::cuerpo_cierre(&usuario, periodo, &inicio, &fin);
        let servicios = cx.servicios.clone();
        self.periodo_en_curso = Some(periodo);
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || registrar_cierre(&servicios, &cuerpo, periodo.to_string())));
        None
    }
}

struct OpcionPeriodo {
    periodo: Option<&'static str>,
    titulo: &'static str,
    detalle: &'static str,
    icono: &'static str,
}

const OPCIONES: [OpcionPeriodo; 4] = [
    OpcionPeriodo { periodo: Some("mañana"), titulo: "Mañana", detalle: "Hasta las 13:59", icono: "sun" },
    OpcionPeriodo { periodo: Some("tarde"), titulo: "Tarde", detalle: "Desde las 14:00", icono: "moon" },
    OpcionPeriodo { periodo: Some("todo"), titulo: "Todo el día", detalle: "00:00 a 23:59", icono: "calendar" },
    OpcionPeriodo { periodo: None, titulo: "Cierre manual", detalle: "Fechas y horas a elección", icono: "clock-time" },
];

fn opcion_periodo(ui: &mut Ui, ancho: f32, numero: usize, opcion: &OpcionPeriodo, ocupado: bool, en_curso: Option<&str>) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 128.0), if ocupado { Sense::hover() } else { Sense::click() });
    let manual = opcion.periodo.is_none();
    let girando = en_curso.is_some() && en_curso == opcion.periodo;
    let alfa = if ocupado && !girando { 0.45 } else { 1.0 };
    let (fondo, borde, texto, detalle, icono, tecla) = if manual {
        (color::VERDE_PROFUNDO, color::VERDE_PROFUNDO, color::BLANCO, color::SOBRE_OSCURO_APAGADO, color::DORADO, EstiloTecla::CUADRADA_OSCURA)
    } else if girando {
        (color::VERDE_ELEGIDO, color::VERDE, color::TEXTO, color::TEXTO_APAGADO, color::VERDE, EstiloTecla::CUADRADA)
    } else {
        (color::BLANCO, color::BORDE, color::TEXTO, color::TEXTO_APAGADO, color::VERDE, EstiloTecla::CUADRADA)
    };
    let avance = ui.ctx().animate_bool_with_time(respuesta.id.with("hover"), !ocupado && respuesta.hovered(), 0.15);
    let tinta = |c: Color32| c.gamma_multiply(alfa);
    let borde = mezclar(borde, color::VERDE, avance);
    ui.painter().rect(rect, radio::TARJETA, tinta(fondo), Stroke::new(1.5, tinta(borde)), StrokeKind::Inside);
    let tecla = EstiloTecla { fondo: tinta(tecla.fondo), borde: tecla.borde.map(tinta), texto: tecla.texto.map(tinta), ..tecla };
    pintar_tecla(ui, Rect::from_min_size(rect.min + Vec2::splat(10.0), Vec2::splat(24.0)), &numero.to_string(), &tecla, tinta(texto));
    let titulo = v2::galeria(ui, opcion.titulo, fuente(15.0, Peso::Semi), tinta(texto));
    let subtitulo = v2::galeria(ui, opcion.detalle, fuente(tam::XS, Peso::Normal), tinta(detalle));
    let lado_icono = if girando { 32.0 } else { 34.0 };
    let alto = lado_icono + 8.0 + titulo.size().y + 8.0 + subtitulo.size().y;
    let mut y = rect.center().y - alto / 2.0;
    let caja_icono = Rect::from_min_size(egui::pos2(rect.center().x - lado_icono / 2.0, y), Vec2::splat(lado_icono));
    if girando {
        v2::girando(ui, caja_icono, tinta(icono));
    } else {
        pintar_icono_solar_en(ui, caja_icono, opcion.icono, tinta(icono));
    }
    y += lado_icono + 8.0;
    let alto_titulo = titulo.size().y;
    ui.painter().galley(egui::pos2(rect.center().x - titulo.size().x / 2.0, y), titulo, tinta(texto));
    y += alto_titulo + 8.0;
    ui.painter().galley(egui::pos2(rect.center().x - subtitulo.size().x / 2.0, y), subtitulo, tinta(detalle));
    if ocupado {
        respuesta
    } else {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    }
}

