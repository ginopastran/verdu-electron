pub mod dibujo;
pub mod manual;
pub mod resultado;

use chrono::Utc;
use eframe::egui::{self, Color32, CursorIcon, Key, Modifiers, Rect, Response, Sense, StrokeKind, Ui, Vec2};
use nucleo::cierre::{self as logica, AvisoImpresion, FalloCierre};
use nucleo::entorno;
use serde_json::Value;

use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, toast, Dialogo, Direccion, Peso, Tam, RADIO};

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
    manual: manual::CierreManual,
    resultado: resultado::DialogoResultadoCierre,
}

pub fn encabezados_app() -> Vec<(&'static str, &'static str)> {
    entorno::APP_ID.map(|id| vec![("X-App-ID", id)]).unwrap_or_default()
}

pub fn avisar_impresion(ctx: &egui::Context, resultado: &Value) {
    match logica::aviso_impresion(resultado) {
        AvisoImpresion::Exito => toast::exito(ctx, "Ticket de cierre impreso correctamente"),
        AvisoImpresion::ErrorImpresora(error) => toast::con_detalle(
            ctx,
            toast::Tipo::Error,
            format!("Error de impresión: {error}"),
            "El cierre se completó correctamente pero no se pudo imprimir el ticket",
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
        if self.abierto {
            let ocupado = self.cerrando();
            let con_atajos = !ocupado && !self.manual.abierto();
            let respuesta = Dialogo::new("cierre-periodo").ancho(448.0).mostrar(&cx.egui, |ui| {
                let (titulo, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
                ui::texto_en(ui, titulo.left_center(), egui::Align2::LEFT_CENTER, "Seleccionar período de cierre", Tam::Lg, Peso::Semi, color::TEXTO);
                ui.add_space(6.0);
                dibujo::parrafo(
                    ui,
                    "Presiona el número correspondiente al período o haz clic en el botón",
                    Tam::Sm,
                    Peso::Normal,
                    color::APAGADO_TEXTO,
                    20.0,
                );
                ui.add_space(16.0);
                let ancho = (ui.available_width() - 16.0) / 2.0;
                let mut accion = None;
                let botones: [[(&str, &str, Option<&'static str>); 2]; 2] = [
                    [("sun", "Mañana (1)", Some("mañana")), ("moon", "Tarde (2)", Some("tarde"))],
                    [("calendar", "Todo el día (3)", Some("todo")), ("clock", "Cierre Manual (4)", None)],
                ];
                for (fila, opciones) in botones.iter().enumerate() {
                    if fila > 0 {
                        ui.add_space(16.0);
                    }
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        for (icono, texto, periodo) in opciones {
                            let girando = ocupado && periodo.is_some();
                            if boton_periodo(ui, ancho, icono, texto, girando, !ocupado, periodo.is_none()).clicked() {
                                accion = Some(periodo.map_or(Accion::Manual, Accion::Periodo));
                            }
                        }
                    });
                }
                if con_atajos {
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
                toast::con_detalle(
                    &cx.egui,
                    toast::Tipo::Error,
                    "No puedes realizar un cierre de tarde sin haber realizado el cierre de mañana del día actual.",
                    "Primero debes realizar el cierre de mañana",
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
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || registrar_cierre(&servicios, &cuerpo, periodo.to_string())));
        None
    }
}

fn boton_periodo(ui: &mut Ui, ancho: f32, icono: &str, texto: &str, girando: bool, habilitado: bool, esmeralda: bool) -> Response {
    let sentido = if habilitado { Sense::click() } else { Sense::hover() };
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 128.0), sentido);
    let opacidad = if habilitado { 1.0 } else { 0.5 };
    let encima = habilitado && respuesta.hovered();
    let pintor = ui.painter();
    if esmeralda {
        ui::pintar_degradado(
            pintor,
            rect,
            RADIO,
            color::ESMERALDA_DESDE.gamma_multiply(opacidad),
            color::ESMERALDA_HASTA.gamma_multiply(opacidad),
            Direccion::AbajoDerecha,
        );
        pintor.rect_stroke(rect, RADIO, ui::borde(color::BORDE.gamma_multiply(opacidad)), StrokeKind::Inside);
    } else {
        let fondo = if encima { color::ACENTO } else { color::FONDO };
        pintor.rect(rect, RADIO, fondo.gamma_multiply(opacidad), ui::borde(color::BORDE.gamma_multiply(opacidad)), StrokeKind::Inside);
    }
    let contenido = if esmeralda { Color32::WHITE } else { color::TEXTO };
    let arriba = rect.center().y - 36.0;
    let caja_icono = Rect::from_center_size(egui::pos2(rect.center().x, arriba + 16.0), Vec2::splat(32.0));
    if girando {
        dibujo::media_luna_girando(ui, caja_icono, 2.0, paleta::GRAY_900.gamma_multiply(opacidad));
    } else {
        ui::pintar_icono_en(ui, caja_icono, icono, contenido.gamma_multiply(opacidad));
    }
    ui::texto_en(
        ui,
        egui::pos2(rect.center().x, arriba + 48.0 + 12.0),
        egui::Align2::CENTER_CENTER,
        texto,
        Tam::Base,
        Peso::Medio,
        contenido.gamma_multiply(opacidad),
    );
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
}
