use eframe::egui::{self, Align2, Color32, Context, Id, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::facturas as logica;
use nucleo::formato;
use serde_json::Value;

use super::comun::{self, bloque, bloque_alto, Boton, Insignia, Opcion};
use super::detalle::anchos_tabla;
use super::pagos::descargar_pdf;
use super::{id_texto, mostrar_valor, numero, texto};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, Entrada, Peso, Tam};

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

fn insignia_tipo(tipo: &str) -> (Color32, Color32) {
    match tipo {
        "A" => (paleta::BLUE_100, paleta::BLUE_800),
        "C" => (paleta::ORANGE_100, paleta::ORANGE_800),
        _ => (paleta::PURPLE_100, paleta::PURPLE_800),
    }
}

fn insignia_estado(estado: &str) -> (Color32, Color32) {
    match estado {
        "pagada" => (paleta::GREEN_100, paleta::GREEN_800),
        "anulada" => (paleta::RED_100, paleta::RED_800),
        _ => (paleta::YELLOW_100, paleta::YELLOW_800),
    }
}

pub fn condicion_fiscal(condicion: &str) -> String {
    match condicion {
        "consumidor_final" => "Consumidor Final".into(),
        "responsable_inscripto" => "Responsable Inscripto".into(),
        "exento" => "Exento".into(),
        "monotributo" => "Monotributo".into(),
        otra => otra.to_string(),
    }
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
                ui::toast::exito(&cx.egui, "Factura actualizada correctamente");
                true
            }
            Err(error) => {
                ui::toast::error(&cx.egui, format!("Error al actualizar factura: {error}"));
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
        let alto_max = ctx.content_rect().height() * 0.9;
        let mut accion = None;
        let r = ui::Dialogo::new("facturas-modal").ancho(896.0).relleno(0.0).mostrar(ctx, |ui| {
            estilo_desplazamiento(ui);
            comun::desplazable(ui, Id::new("facturas-modal-desplazable"), alto_max - 2.0, |ui| {
                egui::Frame::new().inner_margin(24).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    accion = self.titulo(ui);
                    ui.add_space(6.0);
                    let f = &self.factura;
                    let descripcion = format!("Detalles completos de la factura {}-{}", texto(f, "tipoFactura"), texto(f, "numero"));
                    comun::linea(ui, descripcion, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
                    ui.add_space(16.0);
                    self.cuerpo(ui);
                });
            });
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

    fn titulo(&mut self, ui: &mut Ui) -> Option<&'static str> {
        let f = self.factura.clone();
        let tipo = texto(&f, "tipoFactura");
        let estado = texto(&f, "estado");
        let titulo = format!("Factura {tipo}-{}", texto(&f, "numero"));
        let texto_tipo = format!("Factura {tipo}");
        let estado_mayus = estado.to_uppercase();
        let (fondo_tipo, color_tipo) = insignia_tipo(tipo);
        let (fondo_estado, color_estado) = insignia_estado(estado);
        let i_tipo = Insignia::new(&texto_tipo, fondo_tipo, color_tipo, Color32::TRANSPARENT);
        let i_estado = Insignia::new(&estado_mayus, fondo_estado, color_estado, Color32::TRANSPARENT);
        let mut botones: Vec<(Boton, &'static str)> = vec![(Boton::contorno("PDF").chico().icono("download").separacion(16.0), "pdf")];
        if !self.edicion && estado != "anulada" {
            botones.push((Boton::contorno("Editar").chico().icono("edit").separacion(16.0), "editar"));
        }
        if self.edicion {
            let guardando = self.guardado.is_some();
            let texto_guardar = if guardando { "Guardando..." } else { "Guardar" };
            botones.push((Boton::contorno("Cancelar").chico().icono("x").separacion(16.0).habilitado(!guardando), "cancelar"));
            botones.push((Boton::primario(texto_guardar).chico().icono("save").separacion(16.0).habilitado(!guardando), "guardar"));
        }
        let ancho = ui.available_width();
        let anchos_botones: Vec<f32> = botones.iter().map(|(b, _)| b.ancho_natural(ui)).collect();
        let derecha: f32 = anchos_botones.iter().sum::<f32>() + 8.0 * (botones.len() as f32 - 1.0);
        let titulo_nat = comun::ancho_texto(ui, &titulo, Tam::Lg, Peso::Semi);
        let titulo_min = comun::ancho_minimo(ui, &titulo, Tam::Lg, Peso::Semi);
        let base = [20.0, titulo_nat, i_tipo.ancho_natural(ui), i_estado.ancho_natural(ui)];
        let minimos = [20.0, titulo_min, i_tipo.ancho_minimo(ui), i_estado.ancho_minimo(ui)];
        let izquierda_nat = base.iter().sum::<f32>() + 36.0;
        let izquierda_min = minimos.iter().sum::<f32>() + 36.0;
        let [izq, _der] = comun::encoger(&[izquierda_nat, derecha], &[izquierda_min, derecha], ancho)[..] else { unreachable!() };
        let partes = comun::encoger(&base, &minimos, izq - 36.0);
        let titulo_b = bloque_alto(ui, &titulo, Tam::Lg, Peso::Semi, color::TEXTO, partes[1] + 0.5, 18.0);
        let alto = titulo_b.alto().max(i_tipo.alto(ui, partes[2])).max(i_estado.alto(ui, partes[3])).max(36.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
        let mut x = rect.left();
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x + 10.0, rect.center().y), Vec2::splat(20.0)), "file-text", color::TEXTO);
        x += partes[0] + 12.0;
        titulo_b.pintar(ui, egui::pos2(x, rect.center().y - titulo_b.alto() / 2.0), color::TEXTO);
        x += partes[1] + 12.0;
        let alto_tipo = i_tipo.alto(ui, partes[2]);
        i_tipo.pintar(ui, Rect::from_min_size(egui::pos2(x, rect.center().y - alto_tipo / 2.0), Vec2::new(partes[2], alto_tipo)));
        x += partes[2] + 12.0;
        let alto_estado = i_estado.alto(ui, partes[3]);
        i_estado.pintar(ui, Rect::from_min_size(egui::pos2(x, rect.center().y - alto_estado / 2.0), Vec2::new(partes[3], alto_estado)));
        let mut x = rect.right() - derecha;
        let mut accion = None;
        for ((boton, clave), w) in botones.into_iter().zip(anchos_botones) {
            if boton.en(ui, Rect::from_min_size(egui::pos2(x, rect.center().y - 18.0), Vec2::new(w, 36.0)), clave).clicked() {
                accion = Some(clave);
            }
            x += w + 8.0;
        }
        accion
    }

    fn cuerpo(&mut self, ui: &mut Ui) {
        let f = self.factura.clone();
        let ancho = ui.available_width();
        let mitad = (ancho - 24.0) / 2.0;
        let campo = |ui: &mut Ui, etiqueta: &str, valor: &str| {
            comun::etiqueta(ui, etiqueta);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0), Sense::hover());
            comun::texto_en(ui, egui::pos2(rect.left(), rect.center().y), Align2::LEFT_CENTER, valor, Tam::Base, Peso::Normal, color::TEXTO);
        };
        let seccion = |ui: &mut Ui, titulo: &str| {
            comun::linea(ui, titulo, Tam::Lg, Peso::Semi, color::TEXTO);
            ui.add_space(16.0);
        };
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(mitad);
                seccion(ui, "Información General");
                let mut campos = vec![
                    ("Número", format!("{}-{}", texto(&f, "tipoFactura"), texto(&f, "numero"))),
                    ("Fecha", logica::fecha_es_ar(texto(&f, "fecha"))),
                    ("Punto de Venta", mostrar_valor(&f["puntoVenta"])),
                ];
                if !texto(&f, "cae").is_empty() {
                    campos.push(("CAE", texto(&f, "cae").to_string()));
                    if !texto(&f, "vencimientoCae").is_empty() {
                        campos.push(("Vencimiento CAE", logica::fecha_es_ar(texto(&f, "vencimientoCae"))));
                    }
                }
                for (i, (etiqueta, valor)) in campos.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(8.0);
                    }
                    campo(ui, etiqueta, valor);
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.set_width(mitad);
                seccion(ui, "Estado y Control");
                comun::etiqueta(ui, "Estado");
                if self.edicion {
                    let opciones = self.opciones_estado();
                    let mut indice = opciones.iter().position(|(v, _)| *v == self.estado).unwrap_or(usize::MAX);
                    let lista: Vec<Opcion> = opciones.iter().map(|(_, t)| Opcion { icono: None, texto: t.to_string() }).collect();
                    if comun::selector(ui, "modal-estado", mitad, &mut indice, &lista, true) {
                        self.estado = opciones[indice].0.to_string();
                    }
                } else {
                    let estado = texto(&f, "estado");
                    let (fondo, tinta) = insignia_estado(estado);
                    Insignia::new(&estado.to_uppercase(), fondo, tinta, Color32::TRANSPARENT).ui(ui);
                }
            });
        });
        let cliente = &f["cliente"];
        if cliente.is_object() {
            ui.add_space(24.0);
            seccion(ui, "Datos del Cliente");
            let mut campos = vec![("Nombre/Razón Social", logica::nombre_cliente(cliente))];
            if !texto(cliente, "cuit").is_empty() {
                campos.push(("CUIT", texto(cliente, "cuit").to_string()));
            }
            campos.push(("Condición Fiscal", condicion_fiscal(texto(cliente, "condicionFiscal"))));
            for (clave, etiqueta) in [("email", "Email"), ("telefono", "Teléfono")] {
                if !texto(cliente, clave).is_empty() {
                    campos.push((etiqueta, texto(cliente, clave).to_string()));
                }
            }
            for (fila, par) in campos.chunks(2).enumerate() {
                if fila > 0 {
                    ui.add_space(16.0);
                }
                ui.horizontal_top(|ui| {
                    for (columna, (etiqueta, valor)) in par.iter().enumerate() {
                        if columna > 0 {
                            ui.add_space(16.0);
                        }
                        ui.vertical(|ui| {
                            ui.set_width((ancho - 16.0) / 2.0);
                            campo(ui, etiqueta, valor);
                        });
                    }
                });
            }
            if !texto(cliente, "direccion").is_empty() {
                ui.add_space(16.0);
                campo(ui, "Dirección", texto(cliente, "direccion"));
            }
        }
        ui.add_space(24.0);
        seccion(ui, "Detalles de la Factura");
        tabla(ui, &f);
        ui.add_space(16.0);
        totales(ui, &f);
        ui.add_space(24.0);
        seccion(ui, "Observaciones");
        if self.edicion {
            Entrada::new(&mut self.observaciones)
                .multilinea(80.0)
                .tam(Tam::Sm)
                .sugerencia("Ingresa observaciones para la factura...")
                .ui(ui);
        } else {
            let obs = texto(&f, "observaciones");
            let contenido = if obs.is_empty() { "Sin observaciones" } else { obs };
            let b = bloque(ui, contenido, Tam::Base, Peso::Normal, color::TEXTO, ancho - 26.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, (b.alto() + 26.0).max(60.0)), Sense::hover());
            ui.painter().rect(rect, 6, color::APAGADO.gamma_multiply(0.3), Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
            b.pintar(ui, rect.min + Vec2::splat(13.0), color::TEXTO);
        }
    }
}

fn tabla(ui: &mut Ui, f: &Value) {
    let detalles = f["detalles"].as_array().cloned().unwrap_or_default();
    let ancho = ui.available_width();
    let encabezados = ["Producto", "Descripción", "Cantidad", "P. Unitario", "Subtotal"];
    let filas: Vec<[String; 6]> = detalles
        .iter()
        .map(|d| {
            [
                texto(&d["producto"], "nombre").to_string(),
                texto(&d["producto"], "tipoMedida").to_string(),
                texto(d, "descripcion").to_string(),
                formato::numero(numero(d, "cantidad")),
                formato::pesos(numero(d, "precioUnitario")),
                formato::pesos(numero(d, "subtotal")),
            ]
        })
        .collect();
    let mut maximos: Vec<f32> = encabezados.iter().map(|e| comun::ancho_texto(ui, e, Tam::Sm, Peso::Medio)).collect();
    for fila in &filas {
        maximos[0] = maximos[0].max(comun::ancho_texto(ui, &fila[0], Tam::Sm, Peso::Medio));
        for (i, celda) in [(1, &fila[2]), (2, &fila[3]), (3, &fila[4])] {
            maximos[i] = maximos[i].max(comun::ancho_texto(ui, celda, Tam::Sm, Peso::Normal));
        }
        maximos[4] = maximos[4].max(comun::ancho_texto(ui, &fila[5], Tam::Sm, Peso::Medio));
    }
    let maximos: Vec<f32> = maximos.iter().map(|m| m + 32.0).collect();
    let anchos = anchos_tabla(&maximos, &[None; 5], ancho - 2.0);
    let alto = 2.0 + 48.0 + filas.len() as f32 * 77.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    ui.painter().rect_stroke(rect, 6, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let mut x = rect.left() + 1.0;
    let columnas: Vec<(f32, f32)> = anchos
        .iter()
        .map(|w| {
            let c = (x + 16.0, x + w - 16.0);
            x += w;
            c
        })
        .collect();
    let mut y = rect.top() + 1.0;
    for (i, e) in encabezados.iter().enumerate() {
        let (pos, ancla) = if i >= 2 { (columnas[i].1, Align2::RIGHT_CENTER) } else { (columnas[i].0, Align2::LEFT_CENTER) };
        comun::texto_en(ui, egui::pos2(pos, y + 24.0), ancla, e, Tam::Sm, Peso::Medio, color::APAGADO_TEXTO);
    }
    y += 48.0;
    for fila in &filas {
        ui.painter().hline(rect.left()..=rect.right(), y - 0.5, Stroke::new(1.0, color::BORDE));
        let centro = y + 38.0;
        comun::texto_en(ui, egui::pos2(columnas[0].0, y + 16.0 + 10.0), Align2::LEFT_CENTER, &fila[0], Tam::Sm, Peso::Medio, color::TEXTO);
        comun::texto_en(ui, egui::pos2(columnas[0].0, y + 36.0 + 10.0), Align2::LEFT_CENTER, &fila[1], Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
        comun::texto_en(ui, egui::pos2(columnas[1].0, centro), Align2::LEFT_CENTER, &fila[2], Tam::Sm, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(columnas[2].1, centro), Align2::RIGHT_CENTER, &fila[3], Tam::Sm, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(columnas[3].1, centro), Align2::RIGHT_CENTER, &fila[4], Tam::Sm, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(columnas[4].1, centro), Align2::RIGHT_CENTER, &fila[5], Tam::Sm, Peso::Medio, color::TEXTO);
        y += 77.0;
    }
}

fn totales(ui: &mut Ui, f: &Value) {
    let ancho = ui.available_width();
    let impuestos = numero(f, "impuestos");
    let filas = if impuestos > 0.0 { 2 } else { 1 };
    let alto = filas as f32 * 24.0 + filas as f32 * 8.0 + 1.0 + 8.0 + 28.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    let x0 = rect.right() - 320.0;
    let x1 = rect.right();
    let mut y = rect.top();
    let mut fila = |etiqueta: &str, valor: f64| {
        comun::texto_en(ui, egui::pos2(x0, y + 12.0), Align2::LEFT_CENTER, etiqueta, Tam::Base, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(x1, y + 12.0), Align2::RIGHT_CENTER, &formato::pesos(valor), Tam::Base, Peso::Normal, color::TEXTO);
        y += 32.0;
    };
    fila("Subtotal:", numero(f, "subtotal"));
    if impuestos > 0.0 {
        fila("IVA:", impuestos);
    }
    ui.painter().hline(x0..=x1, y + 0.5, Stroke::new(1.0, color::BORDE));
    y += 9.0;
    comun::texto_en(ui, egui::pos2(x0, y + 14.0), Align2::LEFT_CENTER, "Total:", Tam::Lg, Peso::Negrita, color::TEXTO);
    comun::texto_en(ui, egui::pos2(x1, y + 14.0), Align2::RIGHT_CENTER, &formato::pesos(numero(f, "total")), Tam::Lg, Peso::Negrita, color::TEXTO);
}
