use eframe::egui::{self, Align2, Context, Margin, Rect, ScrollArea, Sense, Stroke, Ui, Vec2};
use nucleo::facturas as logica;
use serde_json::Value;

use super::vista::{self, RELLENO_CHIP_ESTADO};
use super::pagos::{descargar_pdf, AgregarPago, ResultadoPago};
use super::{mostrar_valor, numero, pesos, recien_montada, texto};
use crate::app::{Contexto, Ruta};
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, fuente_mono, radio, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, Boton, Columna, Modal, Tabla};
use crate::ui::Peso;

const COLUMNAS: [Columna; 4] = [Columna::fraccion(2.0, 0.0), Columna::fija(110.0).derecha(), Columna::fija(120.0).derecha(), Columna::fija(130.0).derecha()];

#[derive(Default)]
pub struct Detalle {
    ultimo_pase: u64,
    id: String,
    factura: Option<Value>,
    cargando: bool,
    error: Option<String>,
    tarea: Option<Tarea<Result<Value, String>>>,
    pago: Option<AgregarPago>,
    ver_cliente: bool,
}

impl Detalle {
    fn pedir(&mut self, cx: &Contexto) {
        self.cargando = true;
        self.error = None;
        self.factura = None;
        let api = cx.servicios.api.clone();
        let ruta = format!("/facturas/{}", self.id);
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || api.get(&ruta, &[]).map_err(|e| e.to_string())));
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto, id: &str) {
        if recien_montada(&mut self.ultimo_pase, ui) || self.id != id {
            self.id = id.to_string();
            if id.is_empty() {
                self.cargando = false;
                self.error = Some("No se encontró el ID de la factura".into());
            } else {
                self.pedir(cx);
            }
        }
        if let Some(resultado) = esperar(&mut self.tarea) {
            self.cargando = false;
            match resultado {
                Ok(factura) => self.factura = Some(factura),
                Err(_) => self.error = Some("Error al cargar la factura".into()),
            }
        }
        vista::fondo(ui);
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let factura = self.factura.clone().filter(|_| self.error.is_none() && !self.cargando);
        let titulo = factura.as_ref().map_or("Factura".to_string(), |f| texto(f, "numero").to_string());
        if vista::encabezado(ui, "Cuenta corriente", &titulo) {
            cx.global.ir(Ruta::Facturas);
        }
        if self.cargando {
            vista::cargando(ui, ui.available_height() - 68.0, "Cargando factura…");
            return;
        }
        let Some(factura) = factura else {
            self.error_ui(ui, cx);
            return;
        };
        self.pagina(ui, cx, &factura);
    }

    fn error_ui(&self, ui: &mut Ui, cx: &mut Contexto) {
        let error = format!("Error: {}", self.error.clone().unwrap_or_default());
        ui.add_space((ui.available_height() / 2.0 - 60.0).max(24.0));
        vista::vacio(ui, "alert", &error);
        ui.add_space(16.0);
        if vista::boton_centrado(ui, Boton::secundario("Volver").alto(36.0).radio(radio::CONTROL).tam_texto(tam::CONTROL)) {
            cx.global.ir(Ruta::Facturas);
        }
    }

    fn pagina(&mut self, ui: &mut Ui, cx: &mut Contexto, factura: &Value) {
        let mut pdf = false;
        let mut agregar_pago = false;
        ScrollArea::vertical().id_salt("factura-detalle").auto_shrink(false).show(ui, |ui| {
            egui::Frame::NONE.inner_margin(Margin { left: 28, right: 28, top: 24, bottom: 40 }).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let ancho_izquierda = ui.available_width() - 360.0 - 20.0;
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(ancho_izquierda);
                        cabecera(ui, factura);
                        ui.add_space(16.0);
                        productos(ui, factura);
                        if !texto(factura, "observaciones").is_empty() {
                            ui.add_space(16.0);
                            observaciones(ui, factura);
                        }
                    });
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.set_width(360.0);
                        let acciones = saldo(ui, factura);
                        agregar_pago = acciones.agregar_pago;
                        pdf = acciones.pdf;
                        if acciones.cliente {
                            self.ver_cliente = true;
                        }
                        ui.add_space(16.0);
                        historial(ui, factura);
                    });
                });
            });
        });
        if pdf {
            descargar_pdf(cx, factura);
        }
        if agregar_pago {
            self.pago = Some(AgregarPago::new(factura.clone()));
        }
        if let Some(pago) = &mut self.pago {
            match pago.mostrar(ui.ctx(), cx) {
                ResultadoPago::Nada => {}
                ResultadoPago::Cerrar => self.pago = None,
                ResultadoPago::Exito => {
                    self.pago = None;
                    self.pedir(cx);
                }
            }
        }
        if self.ver_cliente && dialogo_cliente(ui.ctx(), &factura["cliente"]) {
            self.ver_cliente = false;
        }
    }
}

fn cabecera(ui: &mut Ui, factura: &Value) {
    let cliente = &factura["cliente"];
    let nombre = logica::nombre_o_razon(cliente).unwrap_or_else(|| "Sin cliente".into());
    let cuit = texto(cliente, "cuit");
    let condicion = texto(cliente, "condicionFiscal");
    let ancho_cliente = v2::ancho_texto(ui, &nombre, tam::DESTACADO, Peso::Semi).max(220.0);
    vista::tarjeta(24, 22).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let ancho_datos = ui.available_width() - ancho_cliente - 16.0;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(ancho_datos);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let (tipo, colores_tipo) = vista::tipo(texto(factura, "tipoFactura"));
                    vista::chip(ui, &tipo, tam::XS, RELLENO_CHIP_ESTADO, colores_tipo);
                    let (estado, colores_estado) = vista::estado(factura);
                    vista::chip(ui, &estado, tam::XS, RELLENO_CHIP_ESTADO, colores_estado);
                });
                ui.add_space(8.0);
                let g = v2::galeria(ui, texto(factura, "numero"), fuente_mono(tam::TITULO_SECCION, Peso::Medio), color::TEXTO);
                let (rect, _) = ui.allocate_exact_size(g.size(), Sense::hover());
                ui.painter().galley(rect.min, g, color::TEXTO);
                ui.add_space(8.0);
                let emision = format!("Emitida el {} · Punto de venta {}", logica::fecha_es_ar(texto(factura, "fecha")), mostrar_valor(&factura["puntoVenta"]));
                v2::texto(ui, emision, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
                let cae = texto(factura, "cae");
                if !cae.is_empty() {
                    ui.add_space(4.0);
                    let mut linea = format!("CAE {cae}");
                    if !texto(factura, "vencimientoCae").is_empty() {
                        linea.push_str(&format!(" · vence {}", logica::fecha_es_ar(texto(factura, "vencimientoCae"))));
                    }
                    let g = v2::galeria(ui, linea, fuente_mono(tam::ETIQUETA, Peso::Normal), color::TEXTO_APAGADO);
                    let (rect, _) = ui.allocate_exact_size(g.size(), Sense::hover());
                    ui.painter().galley(rect.min, g, color::TEXTO_APAGADO);
                }
            });
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.set_width(ancho_cliente);
                ui.spacing_mut().item_spacing.y = 4.0;
                v2::texto(ui, "Cliente", tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
                v2::texto(ui, &nombre, tam::DESTACADO, Peso::Semi, color::TEXTO);
                if !cuit.is_empty() {
                    let g = v2::galeria(ui, format!("CUIT {cuit}"), fuente_mono(tam::ETIQUETA, Peso::Normal), color::TEXTO_APAGADO);
                    let (rect, _) = ui.allocate_exact_size(g.size(), Sense::hover());
                    ui.painter().galley(rect.min, g, color::TEXTO_APAGADO);
                }
                if !condicion.is_empty() {
                    v2::texto(ui, vista::condicion_fiscal(condicion), tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
                }
            });
        });
    });
}

fn productos(ui: &mut Ui, factura: &Value) {
    let detalles = factura["detalles"].as_array().cloned().unwrap_or_default();
    vista::tarjeta(0, 0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 52.0), Sense::hover());
        let interior = fila.shrink2(Vec2::new(22.0, 0.0));
        v2::pintar_texto(ui, interior.left_center(), Align2::LEFT_CENTER, "Productos", tam::ENTRADA, Peso::Semi, color::TEXTO);
        v2::pintar_texto(ui, interior.right_center(), Align2::RIGHT_CENTER, vista::productos(detalles.len()), tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
        let tabla = Tabla::new(&COLUMNAS).relleno(22.0).radio_superior(0);
        let arriba = ui.cursor().top();
        tabla.encabezado(ui, &["Producto", "Cantidad", "Precio", "Subtotal"], 38.0);
        ui.painter().hline(ui.max_rect().x_range(), arriba + 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
        for detalle in &detalles {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 50.0), Sense::hover());
            ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            let celdas = tabla.celdas(rect);
            let producto = &detalle["producto"];
            let nombre = Some(texto(producto, "nombre")).filter(|n| !n.is_empty()).unwrap_or(texto(detalle, "descripcion"));
            let pintor = ui.painter();
            celda_texto(pintor, celdas[0], nombre, tam::CUERPO, Peso::Medio, color::TEXTO, false);
            let cantidad = vista::cantidad_con_medida(numero(detalle, "cantidad"), texto(producto, "tipoMedida"));
            celda_texto(pintor, celdas[1], &cantidad, tam::CUERPO, Peso::Normal, color::TEXTO, true);
            celda_texto(pintor, celdas[2], &pesos(numero(detalle, "precioUnitario")), tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO, true);
            celda_texto(pintor, celdas[3], &pesos(numero(detalle, "subtotal")), tam::CUERPO, Peso::Semi, color::TEXTO, true);
        }
        ui.add_space(14.0);
        let mut lineas = Vec::new();
        if numero(factura, "impuestos") > 0.0 {
            lineas.push(("Subtotal", numero(factura, "subtotal"), Peso::Normal));
            lineas.push(("IVA", numero(factura, "impuestos"), Peso::Normal));
        }
        lineas.push(("Total", numero(factura, "total"), Peso::Semi));
        for (indice, (etiqueta, valor, peso)) in lineas.iter().enumerate() {
            if indice > 0 {
                ui.add_space(6.0);
            }
            let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 19.0), Sense::hover());
            let valor = v2::pintar_texto(ui, egui::pos2(fila.right() - 22.0, fila.center().y), Align2::RIGHT_CENTER, pesos(*valor), tam::ENTRADA, *peso, color::TEXTO);
            v2::pintar_texto(ui, egui::pos2(valor.left() - 24.0, fila.center().y), Align2::RIGHT_CENTER, *etiqueta, tam::ENTRADA, Peso::Normal, color::TEXTO_APAGADO);
        }
        ui.add_space(14.0);
    });
}

fn observaciones(ui: &mut Ui, factura: &Value) {
    vista::tarjeta(20, 18).show(ui, |ui| {
        ui.set_width(ui.available_width());
        v2::texto(ui, "Observaciones", tam::ENTRADA, Peso::Semi, color::TEXTO);
        ui.add_space(8.0);
        let ancho = ui.available_width();
        v2::parrafo(ui, texto(factura, "observaciones"), tam::CUERPO, color::TEXTO_SECUNDARIO, 1.5, ancho);
    });
}

struct AccionesSaldo {
    agregar_pago: bool,
    pdf: bool,
    cliente: bool,
}

fn saldo(ui: &mut Ui, factura: &Value) -> AccionesSaldo {
    let total = numero(factura, "total");
    let pagado = logica::total_pagado(factura);
    let saldo = logica::saldo_pendiente(factura);
    let mut acciones = AccionesSaldo { agregar_pago: false, pdf: false, cliente: false };
    vista::tarjeta(22, 20).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 14.0;
        v2::texto(ui, "Saldo pendiente", tam::ETIQUETA, Peso::Medio, color::TEXTO_APAGADO);
        v2::titulo(ui, pesos(saldo), tam::MONTO, -0.02, if saldo > 0.0 { color::ROJO } else { color::VERDE_EXITO });
        let (barra, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 8.0), Sense::hover());
        ui.painter().rect_filled(barra, radio::PASTILLA, color::VERDE_CHIP);
        let avance = (logica::porcentaje_pagado(factura) as f32 / 100.0).clamp(0.0, 1.0);
        if avance > 0.0 {
            ui.painter().rect_filled(Rect::from_min_size(barra.min, Vec2::new(barra.width() * avance, 8.0)), radio::PASTILLA, color::VERDE);
        }
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
        v2::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, format!("Pagado {}", pesos(pagado)), tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
        v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, format!("de {}", pesos(total)), tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
        if saldo > 0.0 {
            let boton = Boton::primario("Agregar pago").icono("hand-coins").tam_icono(19.0).color_icono(color::DORADO).radio(radio::BOTON_MODAL).sin_sombra().lleno(ui);
            acciones.agregar_pago = boton.ui(ui).clicked();
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let ancho = (ui.available_width() - 8.0) / 2.0;
            let boton = |texto: &'static str, icono: &'static str| {
                Boton::secundario(texto).icono(icono).tam_icono(16.0).separacion(6.0).alto(38.0).radio(radio::CONTROL).tam_texto(tam::ETIQUETA).ancho(ancho)
            };
            acciones.pdf = boton("PDF", "download").ui(ui).clicked();
            acciones.cliente = boton("Cliente", "user").habilitado(factura["cliente"].is_object()).ui(ui).clicked();
        });
    });
    acciones
}

fn historial(ui: &mut Ui, factura: &Value) {
    let pagos = factura["pagos"].as_array().cloned().unwrap_or_default();
    vista::tarjeta(20, 18).show(ui, |ui| {
        ui.set_width(ui.available_width());
        v2::texto(ui, "Historial de pagos", tam::ENTRADA, Peso::Semi, color::TEXTO);
        ui.add_space(12.0);
        if pagos.is_empty() {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 37.0), Sense::hover());
            ui.painter().hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            v2::pintar_texto(ui, rect.left_center(), Align2::LEFT_CENTER, "Todavía no se registraron pagos.", tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
        }
        for (indice, pago) in pagos.iter().enumerate() {
            if indice > 0 {
                ui.add_space(4.0);
            }
            vista::fila_pago(ui, pago, 10.0, true);
        }
    });
}

fn dialogo_cliente(ctx: &Context, cliente: &Value) -> bool {
    let nombre = logica::nombre_cliente(cliente);
    let mut datos = vec![("Nombre/Razón Social", nombre.clone())];
    for (clave, etiqueta) in [("cuit", "CUIT"), ("email", "Email"), ("telefono", "Teléfono"), ("direccion", "Dirección")] {
        if !texto(cliente, clave).is_empty() {
            datos.push((etiqueta, texto(cliente, clave).to_string()));
        }
    }
    let condicion = texto(cliente, "condicionFiscal");
    datos.push(("Condición Fiscal", if condicion.is_empty() { "N/A".into() } else { vista::condicion_fiscal(condicion) }));
    let mut cerrar = false;
    let r = Modal::new("factura-detalle-cliente", "Datos del cliente").subtitulo(&nombre).icono("user").ancho(460.0).mostrar(ctx, |ui| {
        v2::cuerpo_modal(ui, 12, |ui| {
            for (etiqueta, valor) in &datos {
                vista::fila_dato(ui, etiqueta, valor);
            }
        });
        v2::pie_modal(ui, |ui| {
            cerrar = Boton::secundario("Cerrar").modal().ui(ui).clicked();
        });
    });
    cerrar || r.cerrar
}
