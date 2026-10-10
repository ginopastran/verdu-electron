use eframe::egui::{self, Align2, Context, Frame, Margin, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::facturas as logica;
use serde_json::Value;

use super::vista::{self, RELLENO_CHIP_ESTADO};
use super::pagos::{self, descargar_pdf};
use super::{id_texto, mostrar_valor, numero, pesos, texto};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, radio, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, toast, Boton, CampoTexto, Columna, Modal, Tabla};
use crate::ui::Peso;

const COLUMNAS: [Columna; 4] = [Columna::fraccion(2.0, 0.0), Columna::fija(110.0).derecha(), Columna::fija(110.0).derecha(), Columna::fija(120.0).derecha()];

pub enum ResultadoModal {
    Nada,
    Cerrar,
    Actualizada,
}

pub struct ModalFactura {
    factura: Value,
    edicion: bool,
    estado: String,
    observaciones: String,
    guardado: Option<Tarea<Result<Value, String>>>,
}

impl ModalFactura {
    pub fn new(factura: Value) -> Self {
        let estado = texto(&factura, "estado").to_string();
        let observaciones = texto(&factura, "observaciones").to_string();
        let edicion = estado != "anulada";
        ModalFactura { factura, edicion, estado, observaciones, guardado: None }
    }

    fn guardar(&mut self, cx: &Contexto) {
        let Some(cuerpo) = logica::cuerpo_edicion(&self.factura, &self.estado, &self.observaciones) else {
            self.edicion = false;
            return;
        };
        let api = cx.servicios.api.clone();
        let ruta = format!("/facturas/{}", nucleo::api::codificar(&id_texto(&self.factura["id"])));
        self.guardado = Some(Tarea::lanzar(&cx.egui, move || api.put(&ruta, &cuerpo).map_err(|e| e.mensaje)));
    }

    fn recibir(&mut self, cx: &Contexto) -> bool {
        let Some(resultado) = esperar(&mut self.guardado) else { return false };
        match resultado {
            Ok(actualizada) => {
                for clave in ["estado", "observaciones"] {
                    if let Some(valor) = actualizada.get(clave) {
                        self.factura[clave] = valor.clone();
                    }
                }
                self.estado = texto(&self.factura, "estado").to_string();
                self.observaciones = texto(&self.factura, "observaciones").to_string();
                self.edicion = false;
                toast::exito(&cx.egui, "Factura actualizada correctamente");
                true
            }
            Err(error) => {
                toast::error(&cx.egui, format!("Error al actualizar factura: {error}"));
                false
            }
        }
    }

    fn opciones_estado(&self) -> Vec<(&'static str, &'static str)> {
        let mut opciones = vec![("pendiente", "Pendiente"), ("pagada", "Pagada")];
        if texto(&self.factura, "cae").is_empty() {
            opciones.push(("anulada", "Anulada"));
        }
        opciones
    }

    pub fn mostrar(&mut self, ctx: &Context, cx: &Contexto) -> ResultadoModal {
        let actualizada = self.recibir(cx);
        let mut accion = None;
        let titulo = if self.edicion { "Editar factura" } else { "Factura" };
        let subtitulo = pagos::subtitulo(&self.factura);
        let r = Modal::new("facturas-modal", titulo).subtitulo(&subtitulo).icono("file-text").ancho(760.0).mostrar(ctx, |ui| {
            vista::cuerpo_desplazable(ui, "facturas-modal-cuerpo", 160.0, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| self.cuerpo(ui));
            });
            v2::pie_modal(ui, |ui| accion = self.botones(ui));
        });
        match accion {
            Some("pdf") => descargar_pdf(cx, &self.factura),
            Some("editar") => self.edicion = true,
            Some("cancelar") => {
                self.edicion = false;
                self.estado = texto(&self.factura, "estado").to_string();
                self.observaciones = texto(&self.factura, "observaciones").to_string();
            }
            Some("guardar") if self.guardado.is_none() => self.guardar(cx),
            _ => {}
        }
        if r.cerrar {
            ResultadoModal::Cerrar
        } else if actualizada {
            ResultadoModal::Actualizada
        } else {
            ResultadoModal::Nada
        }
    }

    fn botones(&self, ui: &mut Ui) -> Option<&'static str> {
        let mut accion = None;
        let estado = texto(&self.factura, "estado");
        if self.edicion {
            let guardando = self.guardado.is_some();
            if Boton::primario("Guardar cambios").modal().cargando(guardando).ui(ui).clicked() {
                accion = Some("guardar");
            }
            if Boton::secundario("Cancelar").modal().habilitado(!guardando).ui(ui).clicked() {
                accion = Some("cancelar");
            }
        } else if estado != "anulada" && Boton::secundario("Editar").icono("square-pen").modal().ui(ui).clicked() {
            accion = Some("editar");
        }
        if Boton::secundario("PDF").icono("download").modal().ui(ui).clicked() {
            accion = Some("pdf");
        }
        accion
    }

    fn cuerpo(&mut self, ui: &mut Ui) {
        let f = self.factura.clone();
        let ancho = ui.available_width();
        let izquierda = (ancho - 14.0) * 1.3 / 2.3;
        let derecha = ancho - 14.0 - izquierda;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(izquierda);
                vista::etiqueta(ui, "Cliente");
                ui.add_space(6.0);
                let (caja, _) = ui.allocate_exact_size(Vec2::new(izquierda, 44.0), Sense::hover());
                ui.painter().rect(caja, radio::BOTON_MODAL, color::FONDO_APP, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
                v2::pintar_icono(ui, egui::pos2(caja.left() + 20.5, caja.center().y), "user", 17.0, color::TEXTO_TENUE);
                let celda = Rect::from_x_y_ranges(caja.left() + 37.0..=caja.right() - 12.0, caja.y_range());
                celda_texto(ui.painter(), celda, &vista::nombre_cliente(&f), tam::CUERPO, Peso::Normal, color::TEXTO, false);
            });
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.set_width(derecha);
                vista::etiqueta(ui, "Estado");
                ui.add_space(6.0);
                if self.edicion {
                    let opciones = self.opciones_estado();
                    let mut indice = opciones.iter().position(|(v, _)| *v == self.estado).unwrap_or(usize::MAX);
                    let etiquetas: Vec<String> = opciones.iter().map(|(_, t)| t.to_string()).collect();
                    if v2::desplegable(ui, "modal-estado", &mut indice, &etiquetas, "Elegí un estado", derecha, 44.0).changed() {
                        self.estado = opciones[indice].0.to_string();
                    }
                } else {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(derecha, 44.0), Sense::hover());
                    let estado = vista::capitalizar(texto(&f, "estado"));
                    let tam_chip = vista::medir_chip(ui, &estado, tam::XS, RELLENO_CHIP_ESTADO);
                    let colores = match texto(&f, "estado") {
                        "pagada" => (color::VERDE_EXITO_FONDO, color::VERDE_EXITO),
                        "anulada" => (color::FONDO_APP, color::TEXTO_APAGADO),
                        _ => (color::DORADO_FONDO, color::DORADO_TEXTO),
                    };
                    vista::pintar_chip(ui, egui::pos2(rect.left(), rect.center().y - tam_chip.y / 2.0), &estado, tam::XS, RELLENO_CHIP_ESTADO, colores.0, colores.1);
                }
            });
        });
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let (tipo, colores) = vista::tipo(texto(&f, "tipoFactura"));
            vista::chip(ui, &tipo, tam::XS, RELLENO_CHIP_ESTADO, colores);
            let mut datos = format!("Emitida el {} · Punto de venta {}", logica::fecha_es_ar(texto(&f, "fecha")), mostrar_valor(&f["puntoVenta"]));
            if !texto(&f, "cae").is_empty() {
                datos.push_str(&format!(" · CAE {}", texto(&f, "cae")));
                if !texto(&f, "vencimientoCae").is_empty() {
                    datos.push_str(&format!(" (vence {})", logica::fecha_es_ar(texto(&f, "vencimientoCae"))));
                }
            }
            v2::texto(ui, datos, tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
        });
        ui.add_space(16.0);
        tabla(ui, &f);
        ui.add_space(16.0);
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::hover());
        let total = v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, pesos(numero(&f, "total")), tam::MONTO_CHICO, Peso::Semi, color::TEXTO);
        v2::pintar_texto(ui, egui::pos2(total.left() - 10.0, total.bottom() - 4.0), Align2::RIGHT_BOTTOM, "Total", tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
        if numero(&f, "impuestos") > 0.0 {
            let desglose = format!("Subtotal {} · IVA {}", pesos(numero(&f, "subtotal")), pesos(numero(&f, "impuestos")));
            v2::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, desglose, tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
        }
        ui.add_space(16.0);
        vista::etiqueta(ui, "Observaciones");
        ui.add_space(6.0);
        if self.edicion {
            CampoTexto::new(&mut self.observaciones)
                .multilinea(72.0)
                .radio(radio::BOTON_MODAL)
                .tam_texto(tam::CUERPO)
                .sugerencia("Ingresá observaciones para la factura…")
                .ui(ui);
        } else {
            let obs = texto(&f, "observaciones");
            Frame::NONE.fill(color::FONDO_APP).corner_radius(radio::BOTON_MODAL).inner_margin(Margin::symmetric(12, 10)).show(ui, |ui| {
                let ancho = ui.available_width();
                let (contenido, tinta) = if obs.is_empty() { ("Sin observaciones", color::TEXTO_APAGADO) } else { (obs, color::TEXTO) };
                v2::parrafo(ui, contenido, tam::CUERPO, tinta, 1.45, ancho);
            });
        }
    }
}

fn tabla(ui: &mut Ui, f: &Value) {
    let detalles = f["detalles"].as_array().cloned().unwrap_or_default();
    Frame::NONE.stroke(Stroke::new(1.0, color::BORDE_SUAVE)).corner_radius(radio::POPOVER).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let tabla = Tabla::new(&COLUMNAS).relleno(14.0).separacion(10.0).radio_superior(radio::POPOVER - 1);
        tabla.encabezado(ui, &["Producto", "Cantidad", "Precio", "Subtotal"], 38.0);
        ScrollArea::vertical().id_salt("facturas-modal-productos").max_height(240.0).auto_shrink([false, true]).show(ui, |ui| {
            for (indice, detalle) in detalles.iter().enumerate() {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 46.0), Sense::hover());
                if indice + 1 < detalles.len() {
                    ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
                }
                let celdas = tabla.celdas(rect);
                let producto = &detalle["producto"];
                let nombre = Some(texto(producto, "nombre")).filter(|n| !n.is_empty()).unwrap_or(texto(detalle, "descripcion"));
                let pintor = ui.painter();
                celda_texto(pintor, celdas[0], nombre, tam::CONTROL, Peso::Medio, color::TEXTO, false);
                let cantidad = vista::cantidad_con_medida(numero(detalle, "cantidad"), texto(producto, "tipoMedida"));
                celda_texto(pintor, celdas[1], &cantidad, tam::CONTROL, Peso::Normal, color::TEXTO, true);
                celda_texto(pintor, celdas[2], &pesos(numero(detalle, "precioUnitario")), tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO, true);
                celda_texto(pintor, celdas[3], &pesos(numero(detalle, "subtotal")), tam::CONTROL, Peso::Semi, color::TEXTO, true);
            }
        });
    });
}

pub fn estilo_desplazamiento(ui: &mut Ui) {
    ui.spacing_mut().scroll = egui::style::ScrollStyle {
        floating: false,
        bar_width: 11.0,
        bar_inner_margin: 2.0,
        bar_outer_margin: 2.0,
        ..egui::style::ScrollStyle::solid()
    };
}
