use eframe::egui::{Align2, Id, Key, Sense, Ui, Vec2};
use nucleo::facturas::Cuenta;
use nucleo::formato;

use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, radio, tam};
use crate::ui::v2::{self, Boton, CampoTexto, Modal};
use crate::ui::Peso;

pub enum Salida {
    Nada,
    Autorizada,
    Cancelada,
}

#[derive(Default)]
pub struct Autorizacion {
    exceso: Option<(Cuenta, f64)>,
    email: String,
    contrasena: String,
    error: Option<String>,
    tarea: Option<Tarea<Result<(), String>>>,
    enfocar: bool,
}

fn fila(ui: &mut Ui, etiqueta: &str, valor: String, tinta: eframe::egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::hover());
    v2::pintar_texto(ui, rect.left_center(), Align2::LEFT_CENTER, etiqueta, tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO);
    v2::pintar_texto(ui, rect.right_center(), Align2::RIGHT_CENTER, valor, tam::CUERPO_MAS, Peso::Semi, tinta);
}

impl Autorizacion {
    pub fn abierta(&self) -> bool {
        self.exceso.is_some()
    }

    pub fn abrir(&mut self, cuenta: Cuenta, excedente: f64, email: &str) {
        self.exceso = Some((cuenta, excedente));
        self.email = email.to_string();
        self.contrasena.clear();
        self.error = None;
        self.enfocar = true;
    }

    fn enviar(&mut self, cx: &Contexto) {
        let Some(negocio) = cx.global.business_id() else {
            self.error = Some("No se pudo identificar el negocio".into());
            return;
        };
        if self.email.trim().is_empty() || self.contrasena.is_empty() {
            self.error = Some("Completá el email y la contraseña del administrador".into());
            return;
        }
        self.error = None;
        let (base, email, contrasena) = (cx.servicios.api.base().to_string(), self.email.clone(), self.contrasena.clone());
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || nucleo::sesion::autorizar_admin(&base, &email, &contrasena, negocio)));
    }

    pub fn mostrar(&mut self, cx: &Contexto) -> Salida {
        if let Some(resultado) = esperar(&mut self.tarea) {
            self.contrasena.clear();
            match resultado {
                Ok(()) => {
                    self.exceso = None;
                    return Salida::Autorizada;
                }
                Err(mensaje) => {
                    self.error = Some(mensaje);
                    self.enfocar = true;
                }
            }
        }
        let Some((cuenta, excedente)) = self.exceso else {
            return Salida::Nada;
        };
        let validando = self.tarea.is_some();
        let mut enviar = false;
        let mut cancelar = false;
        let respuesta = Modal::new("credito-autorizacion", "Supera el límite de crédito")
            .subtitulo("Para mandarlo a cuenta corriente hace falta la contraseña de un administrador")
            .icono("alert")
            .aviso()
            .ancho(460.0)
            .mostrar(&cx.egui, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    eframe::egui::Frame::NONE.fill(color::FONDO_APP).corner_radius(radio::POPOVER).inner_margin(eframe::egui::Margin::symmetric(16, 10)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        fila(ui, "Saldo en cuenta corriente", formato::pesos(cuenta.saldo), color::TEXTO);
                        fila(ui, "Límite de crédito", formato::pesos(cuenta.limite.unwrap_or(0.0)), color::TEXTO);
                        fila(ui, "Excedente", formato::pesos(excedente), color::ROJO);
                    });
                    ui.add_space(16.0);
                    v2::etiqueta(ui, "Email del administrador");
                    ui.add_space(6.0);
                    CampoTexto::new(&mut self.email).icono("user").alto(44.0).tam_texto(tam::CUERPO).habilitado(!validando).id(Id::new("credito-admin-email")).ui(ui);
                    ui.add_space(12.0);
                    v2::etiqueta(ui, "Contraseña");
                    ui.add_space(6.0);
                    let campo = CampoTexto::new(&mut self.contrasena)
                        .icono("lock")
                        .contrasena(true)
                        .alto(44.0)
                        .tam_texto(tam::CUERPO)
                        .error(self.error.is_some())
                        .habilitado(!validando)
                        .id(Id::new("credito-admin-contrasena"))
                        .ui(ui)
                        .respuesta;
                    if self.enfocar {
                        campo.request_focus();
                        self.enfocar = false;
                    }
                    if campo.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        enviar = true;
                    }
                    if let Some(error) = &self.error {
                        ui.add_space(8.0);
                        v2::texto(ui, error.as_str(), tam::ETIQUETA, Peso::Normal, color::ROJO);
                    }
                });
                v2::pie_modal(ui, |ui| {
                    enviar |= Boton::primario(if validando { "Validando…" } else { "Autorizar" }).modal().cargando(validando).ui(ui).clicked();
                    cancelar = Boton::secundario("Cancelar").modal().habilitado(!validando).ui(ui).clicked();
                });
            });
        if (cancelar || respuesta.cerrar) && !validando {
            self.exceso = None;
            self.contrasena.clear();
            return Salida::Cancelada;
        }
        if enviar && !validando {
            self.enviar(cx);
        }
        Salida::Nada
    }
}
