use eframe::egui::{self, Align2, Sense, Ui, Vec2};
use nucleo::cierre::{self as logica, miles, numero, texto_valor};
use serde_json::Value;

use crate::app::Contexto;
use crate::ui::{self, color, paleta, Boton, Dialogo, Peso, Tam, RADIO};

#[derive(Default)]
pub struct DialogoResultadoCierre {
    abierto: bool,
    resultado: Option<Value>,
}

fn como_jsx(valor: Option<&Value>) -> String {
    match valor {
        None | Some(Value::Null) | Some(Value::Bool(_)) => String::new(),
        Some(otro) => texto_valor(otro),
    }
}

fn monto(valor: Option<&Value>) -> String {
    format!("${}", miles(numero(valor)))
}

fn tarjeta_tabla(ui: &mut Ui, titulo: &str, columnas: &[(f32, &str, bool)], filas: Vec<Vec<(String, Peso)>>) {
    ui::marco().stroke(ui::borde(color::BORDE)).corner_radius(RADIO).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(ui::texto(titulo, Tam::Lg, Peso::Semi, color::TEXTO));
        ui.add_space(8.0);
        let ancho = ui.available_width();
        let celdas = |ui: &mut Ui, alto: f32, textos: Vec<(String, Peso, egui::Color32)>| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
            let mut x = rect.left();
            for ((fraccion, _, derecha), (texto, peso, tinta)) in columnas.iter().zip(textos) {
                let w = ancho * fraccion;
                let (pos, ancla) = if *derecha {
                    (egui::pos2(x + w - 16.0, rect.center().y), Align2::RIGHT_CENTER)
                } else {
                    (egui::pos2(x + 16.0, rect.center().y), Align2::LEFT_CENTER)
                };
                ui::texto_en(ui, pos, ancla, &texto, Tam::Sm, peso, tinta);
                x += w;
            }
            rect
        };
        let cabecera = celdas(ui, 48.0, columnas.iter().map(|(_, t, _)| (t.to_string(), Peso::Medio, color::APAGADO_TEXTO)).collect());
        ui.painter().hline(cabecera.x_range(), cabecera.bottom() - 0.5, ui::borde(color::BORDE));
        let total = filas.len();
        for (indice, fila) in filas.into_iter().enumerate() {
            let rect = celdas(ui, 53.0, fila.into_iter().map(|(t, p)| (t, p, color::TEXTO)).collect());
            if indice + 1 < total {
                ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, ui::borde(color::BORDE));
            }
        }
    });
}

impl DialogoResultadoCierre {
    pub fn abrir(&mut self, resultado: Value) {
        self.resultado = Some(resultado);
        self.abierto = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        let Some(resultado) = self.resultado.as_ref().filter(|_| self.abierto) else {
            return;
        };
        let alto_maximo = cx.egui.content_rect().height() * 0.8 - 48.0;
        let mut cerrar = false;
        let respuesta = Dialogo::new("cierre-resultado").ancho(700.0).mostrar(&cx.egui, |ui| {
            egui::ScrollArea::vertical().max_height(alto_maximo).min_scrolled_height(alto_maximo).show(ui, |ui| {
                ui::encabezado_dialogo(ui, "Resultado del Cierre de Caja", Some("Resumen de ventas del período seleccionado"));
                ui.add_space(16.0);
                let ancho = (ui.available_width() - 16.0) / 2.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.vertical(|ui| {
                        ui.set_width(ancho);
                        ui.label(ui::texto("Período", Tam::Sm, Peso::Medio, color::TEXTO));
                        ui.label(ui::texto(texto_valor(&resultado["periodo"]).to_uppercase(), Tam::Xl, Peso::Negrita, color::TEXTO));
                    });
                    ui.vertical(|ui| {
                        ui.set_width(ancho);
                        ui.label(ui::texto("Fecha", Tam::Sm, Peso::Medio, color::TEXTO));
                        let fechas = format!(
                            "{} - {}",
                            logica::fecha_argentina(resultado["fechaInicio"].as_str().unwrap_or_default()),
                            logica::fecha_argentina(resultado["fechaCierre"].as_str().unwrap_or_default())
                        );
                        ui.label(ui::texto(fechas, Tam::Base, Peso::Normal, color::TEXTO));
                    });
                });
                ui.add_space(16.0);
                ui::marco().stroke(ui::borde(color::BORDE)).corner_radius(RADIO).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(ui::texto("Resumen General", Tam::Lg, Peso::Semi, color::TEXTO));
                    ui.add_space(8.0);
                    let ancho = (ui.available_width() - 16.0) / 2.0;
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        let datos = [
                            ("Total Ventas", monto(resultado.get("totalVentas")), paleta::EMERALD_600),
                            ("Cantidad de Ventas", como_jsx(resultado.get("cantidadVentas")), color::TEXTO),
                        ];
                        for (titulo, valor, tinta) in datos {
                            ui.vertical(|ui| {
                                ui.set_width(ancho);
                                ui.label(ui::texto(titulo, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO));
                                ui.label(ui::texto(valor, Tam::X2, Peso::Negrita, tinta));
                            });
                        }
                    });
                });
                if let Some(metodos) = resultado["ventasPorMetodo"].as_object().filter(|_| logica::verdadero(resultado.get("ventasPorMetodo"))) {
                    ui.add_space(16.0);
                    let filas = metodos
                        .iter()
                        .map(|(metodo, valor)| vec![(nucleo::pedidos::capitalizar(metodo), Peso::Medio), (monto(Some(valor)), Peso::Normal)])
                        .collect();
                    tarjeta_tabla(ui, "Ventas por Método de Pago", &[(0.5, "Método", false), (0.5, "Monto", true)], filas);
                }
                if let Some(vendedores) = resultado["ventasPorVendedor"].as_array().filter(|v| !v.is_empty()) {
                    ui.add_space(16.0);
                    let filas = vendedores
                        .iter()
                        .map(|v| {
                            vec![
                                (como_jsx(v.get("nombre")), Peso::Medio),
                                (como_jsx(v.get("email")), Peso::Normal),
                                (monto(v.get("totalVentas")), Peso::Normal),
                                (como_jsx(v.get("cantidadVentas")), Peso::Normal),
                            ]
                        })
                        .collect();
                    tarjeta_tabla(
                        ui,
                        "Desglose por Vendedor",
                        &[(0.3, "Vendedor", false), (0.36, "Email", false), (0.2, "Total", true), (0.14, "Ventas", true)],
                        filas,
                    );
                }
                ui.add_space(16.0);
                ui::pie_dialogo(ui, |ui| cerrar = Boton::new("Cerrar").ui(ui).clicked());
            });
        });
        if cerrar || respuesta.cerrar {
            self.abierto = false;
        }
    }
}
