use eframe::egui::{self, Align2, Color32, Context, Id, Key, Rect, Sense, Ui, Vec2};
use nucleo::facturas::{self as logica, EstadoHistorial};
use nucleo::formato;
use serde_json::Value;

use super::comun::{self, Boton, Insignia, Opcion};
use super::{numero, texto};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, Entrada, Peso, Tam};

const METODOS: [(&str, &str, &str); 3] =
    [("efectivo", "wallet", "EFECTIVO"), ("tarjeta", "credit-card", "TARJETA"), ("transferencia", "arrow-right", "TRANSFERENCIA")];

pub fn descargar_pdf(cx: &Contexto, factura: &Value) {
    let api = cx.servicios.api.clone();
    let ctx = cx.egui.clone();
    let carpeta = super::carpeta_descargas(cx);
    let factura = factura.clone();
    let _ = Tarea::lanzar(&cx.egui, move || match logica::descargar_pdf(&api, &factura, &carpeta) {
        Ok(ruta) => ui::toast::con_detalle(&ctx, ui::toast::Tipo::Exito, "PDF descargado", format!("Guardado en {}", ruta.display()), 5.0),
        Err(error) => ui::toast::error(&ctx, format!("Error al descargar el PDF: {error}")),
    });
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
}

fn fecha_hoy() -> String {
    logica::hoy_argentina().format("%d/%m/%Y").to_string()
}

impl AgregarPago {
    pub fn new(factura: Value) -> Self {
        AgregarPago { factura, monto: String::new(), metodo: 0, fecha: fecha_hoy(), referencia: String::new(), cargando: false, tarea: None }
    }

    fn enviar(&mut self, cx: &Contexto) {
        let maximo = logica::saldo_para_pago(&self.factura);
        let monto = match logica::validar_pago(&self.monto, maximo) {
            Ok(monto) => monto,
            Err(error) => {
                ui::toast::error(&cx.egui, error.to_string());
                return;
            }
        };
        let fecha = match logica::fecha_dia_mes_anio(&self.fecha).filter(|f| *f <= logica::hoy_argentina()) {
            Some(fecha) => fecha.format("%Y-%m-%d").to_string(),
            None => {
                ui::toast::error(&cx.egui, "Ingresá una fecha válida (dd/mm/aaaa) que no sea futura");
                return;
            }
        };
        let cuerpo = logica::cuerpo_pago(&self.factura["id"], monto, METODOS[self.metodo].0, &fecha, &self.referencia);
        let api = cx.servicios.api.clone();
        let ctx = cx.egui.clone();
        self.cargando = true;
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || match api.post("/facturas/agregar-pago", &cuerpo) {
            Ok(_) => {
                ui::toast::exito(&ctx, "Pago agregado exitosamente");
                true
            }
            Err(error) => {
                ui::toast::error(&ctx, format!("Error al agregar el pago: {error}"));
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
        let r = ui::Dialogo::new("facturas-agregar-pago").ancho(448.0).mostrar(ctx, |ui| {
            comun::encabezado_dialogo(
                ui,
                "dollar-sign",
                "Agregar Pago",
                &format!("Agregar un pago a la factura. Saldo pendiente: {}", formato::pesos(maximo)),
                8.0,
            );
            ui.add_space(16.0);
            comun::etiqueta(ui, "Monto *");
            ui.add_space(8.0);
            Entrada::new(&mut self.monto).tam(Tam::Sm).icono("dollar-sign").sugerencia("0,00").id(Id::new("pago-monto")).ui(ui);
            ui.add_space(16.0);
            comun::etiqueta(ui, "Método de Pago *");
            ui.add_space(8.0);
            let opciones: Vec<Opcion> = METODOS.iter().map(|(_, i, t)| Opcion { icono: Some(i), texto: t.to_string() }).collect();
            let ancho = ui.available_width();
            comun::selector(ui, "pago-metodo", ancho, &mut self.metodo, &opciones, true);
            ui.add_space(16.0);
            comun::etiqueta(ui, "Fecha del Pago *");
            ui.add_space(8.0);
            Entrada::new(&mut self.fecha).tam(Tam::Sm).icono("calendar").sugerencia("dd/mm/aaaa").ui(ui);
            ui.add_space(16.0);
            comun::etiqueta(ui, "Referencia (opcional)");
            ui.add_space(8.0);
            Entrada::new(&mut self.referencia).tam(Tam::Sm).sugerencia("Número de comprobante, transferencia, etc.").ui(ui);
            ui.add_space(16.0);
            ui::pie_dialogo(ui, |ui| {
                let texto_boton = if self.cargando { "Agregando..." } else { "Agregar Pago" };
                if Boton::primario(texto_boton).habilitado(habilitado).ui(ui).clicked() {
                    enviar = true;
                }
                if Boton::contorno("Cancelar").habilitado(!self.cargando).ui(ui).clicked() {
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

fn icono_metodo(metodo: &str) -> &'static str {
    match metodo.to_lowercase().as_str() {
        "efectivo" => "wallet",
        "tarjeta" => "credit-card",
        "transferencia" => "arrow-right",
        _ => "dollar-sign",
    }
}

fn etiqueta_metodo(metodo: &str) -> String {
    match metodo.to_lowercase().as_str() {
        "efectivo" => "Efectivo".into(),
        "tarjeta" => "Tarjeta".into(),
        "transferencia" => "Transferencia".into(),
        _ => metodo.to_string(),
    }
}

pub fn historial(ctx: &Context, factura: &Value) -> bool {
    let total = numero(factura, "total");
    let pagado = logica::total_pagado(factura);
    let saldo = logica::saldo_pendiente(factura);
    let porcentaje = logica::porcentaje_pagado(factura);
    let mut cerrar = false;
    let alto_max = ctx.content_rect().height() * 0.8;
    let r = ui::Dialogo::new("facturas-historial").ancho(672.0).relleno(0.0).mostrar(ctx, |ui| {
        comun::desplazable(ui, Id::new("historial-desplazable"), alto_max - 2.0, |ui| {
            egui::Frame::new().inner_margin(24).show(ui, |ui| {
                ui.set_width(ui.available_width());
                comun::encabezado_dialogo(ui, "dollar-sign", "Historial de Pagos", "Detalle de todos los pagos realizados para esta factura", 8.0);
                ui.add_space(16.0);
                resumen(ui, total, pagado, saldo, porcentaje);
                ui.add_space(24.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
                let pct = format!("{}%", formato::numero_con(porcentaje, 1, 1));
                comun::texto_en(ui, egui::pos2(rect.left(), rect.center().y), Align2::LEFT_CENTER, "Progreso de pago", Tam::Sm, Peso::Normal, color::TEXTO);
                comun::texto_en(ui, egui::pos2(rect.right(), rect.center().y), Align2::RIGHT_CENTER, &pct, Tam::Sm, Peso::Normal, color::TEXTO);
                ui.add_space(8.0);
                let (barra, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 8.0), Sense::hover());
                ui.painter().rect_filled(barra, 4, color::SECUNDARIO);
                let lleno = (porcentaje as f32 / 100.0).clamp(0.0, 1.0);
                if lleno > 0.0 {
                    let parte = Rect::from_min_size(barra.min, Vec2::new(barra.width() * lleno, barra.height()));
                    ui.painter().with_clip_rect(parte).rect_filled(barra, 4, color::PRIMARIO);
                }
                ui.add_space(24.0);
                comun::linea(ui, "Pagos Realizados", Tam::Lg, Peso::Semi, color::TEXTO);
                ui.add_space(16.0);
                let pagos = factura["pagos"].as_array().cloned().unwrap_or_default();
                if pagos.is_empty() {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 88.0), Sense::hover());
                    comun::texto_en(ui, rect.center(), Align2::CENTER_CENTER, "No hay pagos registrados para esta factura", Tam::Base, Peso::Normal, paleta::GRAY_500);
                }
                for (indice, pago) in pagos.iter().enumerate() {
                    if indice > 0 {
                        ui.add_space(12.0);
                    }
                    fila_pago(ui, pago);
                }
                ui.add_space(24.0);
                let (texto_estado, insignia) = match logica::estado_historial(factura) {
                    EstadoHistorial::Pagada => ("Pagada completamente", Insignia::new("PAGADA", Color32::BLACK, Color32::WHITE, Color32::TRANSPARENT)),
                    EstadoHistorial::Parcial => ("Pago parcial", Insignia::new("PARCIAL", paleta::YELLOW_100, paleta::YELLOW_800, paleta::YELLOW_200)),
                    EstadoHistorial::Pendiente => ("Pendiente de pago", Insignia::new("PENDIENTE", paleta::RED_100, paleta::RED_800, paleta::RED_200)),
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.0), Sense::hover());
                ui.painter().rect_filled(rect, 8, paleta::GRAY_50);
                comun::texto_en(ui, egui::pos2(rect.left() + 16.0, rect.center().y), Align2::LEFT_CENTER, texto_estado, Tam::Base, Peso::Medio, color::TEXTO);
                let ancho = insignia.ancho_natural(ui);
                insignia.pintar(ui, Rect::from_center_size(egui::pos2(rect.right() - 16.0 - ancho / 2.0, rect.center().y), Vec2::new(ancho, 22.0)));
                ui.add_space(24.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if Boton::primario("Cerrar").ui(ui).clicked() {
                        cerrar = true;
                    }
                });
            });
        });
    });
    cerrar || r.cerrar
}

fn resumen(ui: &mut Ui, total: f64, pagado: f64, saldo: f64, porcentaje: f64) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 80.0), Sense::hover());
    ui.painter().rect_filled(rect, 8, paleta::GRAY_50);
    let columnas = [
        ("Total Factura", formato::pesos(total), Color32::BLACK),
        ("Total Pagado", formato::pesos(pagado), paleta::GREEN_600),
        ("Saldo Pendiente", formato::pesos(saldo), if saldo > 0.0 { paleta::RED_600 } else { paleta::GREEN_600 }),
        ("% Pagado", format!("{}%", formato::numero_con(porcentaje, 1, 1)), paleta::BLUE_600),
    ];
    let ancho = (rect.width() - 32.0 - 48.0) / 4.0;
    for (indice, (etiqueta, valor, color_valor)) in columnas.iter().enumerate() {
        let x = rect.left() + 16.0 + indice as f32 * (ancho + 16.0);
        comun::texto_en(ui, egui::pos2(x, rect.top() + 26.0), Align2::LEFT_CENTER, etiqueta, Tam::Sm, Peso::Normal, paleta::GRAY_600);
        comun::texto_en(ui, egui::pos2(x, rect.top() + 50.0), Align2::LEFT_CENTER, valor, Tam::Lg, Peso::Semi, *color_valor);
    }
}

fn fila_pago(ui: &mut Ui, pago: &Value) {
    let referencia = texto(pago, "referencia");
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 78.0), Sense::hover());
    ui.painter().rect_stroke(rect, 8, egui::Stroke::new(1.0, paleta::GRAY_200), egui::StrokeKind::Inside);
    let circulo = Rect::from_center_size(egui::pos2(rect.left() + 37.0, rect.center().y), Vec2::splat(40.0));
    ui.painter().circle_filled(circulo.center(), 20.0, paleta::EMERALD_600);
    let metodo = texto(pago, "metodoPago");
    ui::pintar_icono_en(ui, Rect::from_center_size(circulo.center(), Vec2::splat(16.0)), icono_metodo(metodo), Color32::WHITE);
    let tx = circulo.right() + 12.0;
    comun::texto_en(ui, egui::pos2(tx, rect.center().y - 10.0), Align2::LEFT_CENTER, &formato::pesos(numero(pago, "monto")), Tam::Base, Peso::Semi, color::TEXTO);
    comun::texto_en(ui, egui::pos2(tx, rect.center().y + 12.0), Align2::LEFT_CENTER, &etiqueta_metodo(metodo), Tam::Sm, Peso::Normal, paleta::GRAY_600);
    let insignia = Insignia::new("PAGADO", paleta::GREEN_100, paleta::GREEN_800, paleta::GREEN_200).icono("check-circle");
    let ancho = insignia.ancho_natural(ui);
    let caja = Rect::from_center_size(egui::pos2(rect.right() - 16.0 - ancho / 2.0, rect.center().y), Vec2::new(ancho, 22.0));
    insignia.pintar(ui, caja);
    let derecha = caja.left() - 16.0;
    let alto = if referencia.is_empty() { 20.0 } else { 36.0 };
    let arriba = rect.center().y - alto / 2.0;
    let fecha = logica::fecha_es_ar(texto(pago, "fecha"));
    let r = comun::texto_en(ui, egui::pos2(derecha, arriba + 10.0), Align2::RIGHT_CENTER, &fecha, Tam::Sm, Peso::Normal, paleta::GRAY_600);
    ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(r.left() - 10.0, r.center().y), Vec2::splat(12.0)), "calendar", paleta::GRAY_600);
    if !referencia.is_empty() {
        comun::texto_en(ui, egui::pos2(derecha, arriba + 28.0), Align2::RIGHT_CENTER, &format!("Ref: {referencia}"), Tam::Xs, Peso::Normal, paleta::GRAY_500);
    }
}
