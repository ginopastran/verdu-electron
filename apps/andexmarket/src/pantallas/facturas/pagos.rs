use eframe::egui::{self, Align2, Context, CursorIcon, Id, Key, ScrollArea, Sense, Ui, Vec2};
use nucleo::facturas as logica;
use serde_json::Value;

use super::vista;
use super::{pesos, texto};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, radio, tam};
use crate::ui::v2::{self, toast, Boton, CampoTexto, Modal};
use crate::ui::Peso;

const METODOS: [(&str, &str, &str); 3] =
    [("efectivo", "cash", "Efectivo"), ("tarjeta", "card", "Tarjeta"), ("transferencia", "transfer", "Transferencia")];

pub fn descargar_pdf(cx: &Contexto, factura: &Value) {
    let api = cx.servicios.api.clone();
    let ctx = cx.egui.clone();
    let carpeta = super::carpeta_descargas(cx);
    let factura = factura.clone();
    let _ = Tarea::lanzar(&cx.egui, move || match logica::descargar_pdf(&api, &factura, &carpeta) {
        Ok(ruta) => toast::mostrar_aviso(&ctx, toast::Tipo::Exito, "PDF descargado", Some(format!("Guardado en {}", ruta.display())), 5.0),
        Err(error) => toast::error(&ctx, format!("Error al descargar el PDF: {error}")),
    });
}

pub fn subtitulo(factura: &Value) -> String {
    format!("{} · {}", texto(factura, "numero"), vista::nombre_cliente(factura))
}

pub enum ResultadoPago {
    Nada,
    Cerrar,
    Exito,
}

pub struct AgregarPago {
    factura: Value,
    monto: String,
    metodo: usize,
    fecha: String,
    referencia: String,
    cargando: bool,
    tarea: Option<Tarea<bool>>,
    error_monto: Option<String>,
    error_fecha: Option<String>,
}

fn fecha_hoy() -> String {
    logica::hoy_argentina().format("%d/%m/%Y").to_string()
}

impl AgregarPago {
    pub fn new(factura: Value) -> Self {
        AgregarPago { factura, monto: String::new(), metodo: 0, fecha: fecha_hoy(), referencia: String::new(), cargando: false, tarea: None, error_monto: None, error_fecha: None }
    }

    fn enviar(&mut self, cx: &Contexto) {
        let maximo = logica::saldo_para_pago(&self.factura);
        let monto = match logica::validar_pago(&self.monto, maximo) {
            Ok(monto) => monto,
            Err(error) => {
                self.error_monto = Some(error.to_string());
                return;
            }
        };
        let fecha = match logica::fecha_dia_mes_anio(&self.fecha).filter(|f| *f <= logica::hoy_argentina()) {
            Some(fecha) => fecha.format("%Y-%m-%d").to_string(),
            None => {
                self.error_fecha = Some("Ingresá una fecha válida (dd/mm/aaaa) que no sea futura".into());
                return;
            }
        };
        let cuerpo = logica::cuerpo_pago(&self.factura["id"], monto, METODOS[self.metodo].0, &fecha, &self.referencia);
        let api = cx.servicios.api.clone();
        let ctx = cx.egui.clone();
        self.cargando = true;
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || match api.post("/facturas/agregar-pago", &cuerpo) {
            Ok(_) => {
                toast::exito(&ctx, "Pago agregado exitosamente");
                true
            }
            Err(error) => {
                toast::error(&ctx, format!("Error al agregar el pago: {error}"));
                false
            }
        }));
    }

    pub fn mostrar(&mut self, ctx: &Context, cx: &Contexto) -> ResultadoPago {
        if let Some(exito) = esperar(&mut self.tarea) {
            self.cargando = false;
            if exito {
                return ResultadoPago::Exito;
            }
        }
        let maximo = logica::saldo_para_pago(&self.factura);
        let habilitado = !self.cargando && logica::parsear_numero(&self.monto).is_some_and(|m| m > 0.0);
        let mut cancelar = false;
        let mut enviar = false;
        let subtitulo = subtitulo(&self.factura);
        let r = Modal::new("facturas-agregar-pago", "Agregar pago").subtitulo(&subtitulo).icono("hand-coins").ancho(500.0).mostrar(ctx, |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                saldo(ui, maximo);
                ui.add_space(14.0);
                let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 17.0), Sense::hover());
                v2::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, "Monto", tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO);
                let enlace = v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, "Saldo total", tam::CHICO, Peso::Semi, color::VERDE);
                if ui.interact(enlace, Id::new("pago-saldo-total"), Sense::click()).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    self.monto = logica::texto_entrada(maximo);
                    self.error_monto = None;
                }
                ui.add_space(6.0);
                let monto = CampoTexto::new(&mut self.monto)
                    .alto(48.0)
                    .prefijo("$")
                    .tam_texto(18.0)
                    .sugerencia("0")
                    .error(self.error_monto.is_some())
                    .id(Id::new("pago-monto"))
                    .ui(ui);
                if monto.respuesta.changed() {
                    self.error_monto = None;
                }
                mensaje_error(ui, &self.error_monto);
                ui.add_space(14.0);
                vista::etiqueta(ui, "Método");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let ancho = (ui.available_width() - 16.0) / 3.0;
                    for (indice, (_, icono, etiqueta)) in METODOS.iter().enumerate() {
                        let elegido = v2::opcion_eleccion(ui, Vec2::new(ancho, 44.0), radio::BOTON_MODAL, self.metodo == indice, 1.5, |ui, rect, colores| {
                            let g = v2::galeria(ui, *etiqueta, crate::ui::marca::fuente(tam::CONTROL, Peso::Medio), colores.texto);
                            let x = rect.center().x - (17.0 + 7.0 + g.size().x) / 2.0;
                            v2::pintar_icono(ui, egui::pos2(x + 8.5, rect.center().y), icono, 17.0, colores.texto);
                            ui.painter().galley(egui::pos2(x + 24.0, rect.center().y - g.size().y / 2.0), g, colores.texto);
                        });
                        if elegido.clicked() {
                            self.metodo = indice;
                        }
                    }
                });
                ui.add_space(14.0);
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(150.0);
                        vista::etiqueta(ui, "Fecha");
                        ui.add_space(6.0);
                        let fecha = CampoTexto::new(&mut self.fecha)
                            .alto(42.0)
                            .radio(radio::CONTROL)
                            .tam_texto(tam::CUERPO)
                            .relleno(10.0, 10.0)
                            .sugerencia("dd/mm/aaaa")
                            .error(self.error_fecha.is_some())
                            .ui(ui);
                        if fecha.respuesta.changed() {
                            self.error_fecha = None;
                        }
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            vista::etiqueta(ui, "Referencia");
                            ui.add_space(4.0);
                            v2::texto(ui, "(opcional)", tam::ETIQUETA, Peso::Normal, color::TEXTO_TENUE);
                        });
                        ui.add_space(6.0);
                        CampoTexto::new(&mut self.referencia)
                            .alto(42.0)
                            .radio(radio::CONTROL)
                            .tam_texto(tam::CUERPO)
                            .relleno(12.0, 12.0)
                            .sugerencia("N° de comprobante o transferencia")
                            .ui(ui);
                    });
                });
                mensaje_error(ui, &self.error_fecha);
            });
            v2::pie_modal(ui, |ui| {
                if Boton::primario("Registrar pago").modal().cargando(self.cargando).habilitado(habilitado || self.cargando).ui(ui).clicked() {
                    enviar = true;
                }
                if Boton::secundario("Cancelar").modal().habilitado(!self.cargando).ui(ui).clicked() {
                    cancelar = true;
                }
            });
            if habilitado && ui.input(|i| i.key_pressed(Key::Enter)) {
                enviar = true;
            }
        });
        if enviar {
            self.enviar(cx);
        }
        if cancelar || r.cerrar {
            return ResultadoPago::Cerrar;
        }
        ResultadoPago::Nada
    }
}

fn mensaje_error(ui: &mut Ui, error: &Option<String>) {
    if let Some(error) = error {
        ui.add_space(6.0);
        let ancho = ui.available_width();
        v2::parrafo(ui, error, tam::CHICO, color::ROJO, 1.4, ancho);
    }
}

fn saldo(ui: &mut Ui, saldo: f64) {
    let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 54.0), Sense::hover());
    ui.painter().rect_filled(caja, radio::CAMPO, color::FONDO_APP);
    v2::pintar_texto(ui, egui::pos2(caja.left() + 16.0, caja.center().y), Align2::LEFT_CENTER, "Saldo pendiente", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO);
    v2::pintar_texto(ui, egui::pos2(caja.right() - 16.0, caja.center().y), Align2::RIGHT_CENTER, pesos(saldo), tam::TITULO_SECCION, Peso::Semi, color::ROJO);
}

pub enum ResultadoHistorial {
    Nada,
    Cerrar,
    AgregarPago,
}

fn total(ui: &mut Ui, etiqueta: &str, valor: f64, tinta: egui::Color32) {
    let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 17.0), Sense::hover());
    v2::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, etiqueta, tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO);
    v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, pesos(valor), tam::CUERPO, Peso::Semi, tinta);
}

pub fn historial(ctx: &Context, factura: &Value) -> ResultadoHistorial {
    let pagado = logica::total_pagado(factura);
    let saldo = logica::saldo_pendiente(factura);
    let pagos = factura["pagos"].as_array().cloned().unwrap_or_default();
    let mut resultado = ResultadoHistorial::Nada;
    let subtitulo = subtitulo(factura);
    let r = Modal::new("facturas-historial", "Historial de pagos").subtitulo(&subtitulo).icono("history").ancho(500.0).mostrar(ctx, |ui| {
        v2::cuerpo_modal(ui, 16, |ui| {
            if pagos.is_empty() {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 73.0), Sense::hover());
                v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, "Todavía no se registraron pagos.", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO);
            } else {
                ScrollArea::vertical().id_salt("historial-pagos").max_height(336.0).auto_shrink([false, true]).show(ui, |ui| {
                    for pago in &pagos {
                        vista::fila_pago(ui, pago, 11.0, false);
                    }
                });
            }
            ui.add_space(14.0);
            total(ui, "Total pagado", pagado, color::TEXTO);
            ui.add_space(6.0);
            total(ui, "Saldo pendiente", saldo, if saldo > 0.0 { color::ROJO } else { color::VERDE_EXITO });
        });
        v2::pie_modal(ui, |ui| {
            if saldo > 0.0 && Boton::primario("Agregar pago").icono("hand-coins").color_icono(color::DORADO).modal().ui(ui).clicked() {
                resultado = ResultadoHistorial::AgregarPago;
            }
            if Boton::secundario("Cerrar").modal().ui(ui).clicked() {
                resultado = ResultadoHistorial::Cerrar;
            }
        });
    });
    if r.cerrar {
        ResultadoHistorial::Cerrar
    } else {
        resultado
    }
}
