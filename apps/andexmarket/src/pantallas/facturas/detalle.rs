use eframe::egui::{self, Align2, Color32, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::facturas as logica;
use nucleo::formato;
use serde_json::Value;

use super::comun::{self, Boton, Insignia, Relleno};
use super::modal::condicion_fiscal;
use super::pagos::{descargar_pdf, AgregarPago, ResultadoPago};
use super::{mostrar_valor, numero, recien_montada, texto};
use crate::app::{Contexto, Ruta};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, Peso, Tam};

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

pub fn capitalizar(texto: &str) -> String {
    let mut salida = String::new();
    let mut inicio = true;
    for c in texto.chars() {
        if inicio {
            salida.extend(c.to_uppercase());
        } else {
            salida.push(c);
        }
        inicio = c.is_whitespace() || c == '-' || c == '_';
    }
    salida
}

pub fn anchos_tabla(maximos: &[f32], fijos: &[Option<f32>], disponible: f32) -> Vec<f32> {
    let fijo: f32 = fijos.iter().flatten().sum();
    let libres: f32 = maximos.iter().zip(fijos).filter(|(_, f)| f.is_none()).map(|(m, _)| m).sum();
    let extra = (disponible - fijo - libres).max(0.0);
    maximos
        .iter()
        .zip(fijos)
        .map(|(maximo, f)| f.unwrap_or(maximo + extra * maximo / libres.max(1.0)))
        .collect()
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
        if self.cargando {
            let rect = ui.max_rect();
            let centro = rect.center();
            let anillo = Rect::from_center_size(egui::pos2(centro.x, centro.y - 18.0), Vec2::splat(32.0));
            let angulo = ui.input(|i| i.time) as f32 * std::f32::consts::TAU;
            let puntos: Vec<egui::Pos2> = (0..=24)
                .map(|i| {
                    let a = angulo + i as f32 / 24.0 * std::f32::consts::PI * 1.5;
                    anillo.center() + Vec2::angled(a) * 14.0
                })
                .collect();
            ui.painter().add(egui::Shape::line(puntos, Stroke::new(4.0, paleta::EMERALD_400)));
            comun::texto_en(ui, egui::pos2(centro.x, centro.y + 26.0), Align2::CENTER_CENTER, "Cargando factura...", Tam::Base, Peso::Normal, color::TEXTO);
            ui.ctx().request_repaint();
            return;
        }
        let Some(factura) = self.factura.clone().filter(|_| self.error.is_none()) else {
            let centro = ui.max_rect().center();
            let error = format!("Error: {}", self.error.clone().unwrap_or_default());
            comun::texto_en(ui, egui::pos2(centro.x, centro.y - 28.0), Align2::CENTER_CENTER, &error, Tam::Base, Peso::Normal, paleta::RED_500);
            let boton = Boton::primario("Volver");
            let ancho = boton.ancho_natural(ui);
            if boton.en(ui, Rect::from_center_size(egui::pos2(centro.x, centro.y + 20.0), Vec2::new(ancho, 40.0)), "volver-error").clicked() {
                cx.global.ir(Ruta::Facturas);
            }
            return;
        };
        self.pagina(ui, cx, &factura);
    }

    fn pagina(&mut self, ui: &mut Ui, cx: &mut Contexto, factura: &Value) {
        let pagado = logica::total_pagado(factura);
        let saldo = logica::saldo_pendiente(factura);
        let pagada = saldo <= 0.0;
        let mut volver = false;
        let mut pdf = false;
        let mut agregar_pago = false;
        ui.painter().rect_filled(ui.max_rect(), 0, paleta::GRAY_50);
        ui.spacing_mut().scroll = egui::style::ScrollStyle {
            floating: false,
            bar_width: 11.0,
            bar_inner_margin: 2.0,
            bar_outer_margin: 2.0,
            ..egui::style::ScrollStyle::solid()
        };
        ScrollArea::vertical().id_salt("factura-detalle").auto_shrink(false).show(ui, |ui| {
            ui.set_width(ui.available_width());
            comun::encabezado_verde(ui, 104.0, |ui| {
                if comun::boton_volver(ui).clicked() {
                    volver = true;
                }
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    let titulo = format!("Factura {} {}", texto(factura, "tipoFactura"), texto(factura, "numero"));
                    comun::linea(ui, titulo, Tam::X2, Peso::Semi, Color32::WHITE);
                    comun::linea(ui, format!("Punto de Venta: {}", mostrar_valor(&factura["puntoVenta"])), Tam::Base, Peso::Normal, paleta::EMERALD_100);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let negro = |texto: &'static str, icono: &'static str| {
                        Boton::contorno(texto)
                            .chico()
                            .icono(icono)
                            .separacion(16.0)
                            .relleno(Relleno::Liso(Color32::BLACK))
                            .encima(Relleno::Liso(color::ACENTO))
                            .color(Color32::WHITE)
                            .color_encima(color::ACENTO_TEXTO)
                    };
                    if saldo > 0.0 {
                        if negro("Agregar Pago", "dollar-sign").ui(ui).clicked() {
                            agregar_pago = true;
                        }
                        ui.add_space(8.0);
                    }
                    if negro("Descargar PDF", "download").ui(ui).clicked() {
                        pdf = true;
                    }
                    ui.add_space(8.0);
                    let insignia = if pagada {
                        Insignia::new("PAGADA", paleta::GREEN_600, color::PRIMARIO_TEXTO, Color32::TRANSPARENT)
                    } else {
                        Insignia::new("PENDIENTE", paleta::YELLOW_600, color::SECUNDARIO_TEXTO, Color32::TRANSPARENT)
                    };
                    insignia.ui(ui);
                });
            });
            egui::Frame::new().inner_margin(24).show(ui, |ui| {
                let ancho = ui.available_width().min(1152.0);
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.set_width(ancho);
                    if self.tarjetas(ui, factura, pagado, saldo) {
                        self.ver_cliente = true;
                    }
                    ui.add_space(24.0);
                    productos(ui, factura);
                    if logica::tiene_pagos(factura) {
                        ui.add_space(24.0);
                        historial(ui, factura);
                    }
                    if !texto(factura, "observaciones").is_empty() {
                        ui.add_space(24.0);
                        observaciones(ui, factura);
                    }
                });
            });
        });
        if volver {
            cx.global.ir(Ruta::Facturas);
        }
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

    fn tarjetas(&self, ui: &mut Ui, factura: &Value, pagado: f64, saldo: f64) -> bool {
        let ancho_ventana = ui.ctx().content_rect().width();
        let columnas = if ancho_ventana >= 1024.0 { 3 } else { 1 };
        let ancho = ui.available_width();
        let ancho_tarjeta = (ancho - 24.0 * (columnas as f32 - 1.0)) / columnas as f32;
        let mut info = vec![
            ("FECHA DE EMISIÓN:".to_string(), logica::fecha_es_ar(texto(factura, "fecha"))),
            ("TIPO DE FACTURA:".to_string(), capitalizar(texto(factura, "tipoFactura"))),
        ];
        if !texto(factura, "cae").is_empty() {
            info.push(("CAE:".to_string(), texto(factura, "cae").to_string()));
        }
        let cliente = &factura["cliente"];
        let mut datos_cliente = vec![(
            "Cliente:".to_string(),
            logica::nombre_o_razon(cliente).unwrap_or_else(|| "Sin cliente".into()),
        )];
        if !texto(cliente, "cuit").is_empty() {
            datos_cliente.push(("CUIT:".to_string(), texto(cliente, "cuit").to_string()));
        }
        let condicion = texto(cliente, "condicionFiscal");
        datos_cliente.push(("Condición Fiscal:".to_string(), capitalizar(if condicion.is_empty() { "N/A" } else { condicion })));
        let alto_filas = |n: usize| n as f32 * 24.0 + n.saturating_sub(1) as f32 * 12.0;
        let alto_info = alto_filas(info.len());
        let alto_totales = 24.0 + 12.0 + 24.0 + 12.0 + 1.0 + 8.0 + 28.0 + 12.0 + 20.0 + 12.0 + 20.0;
        let alto_cliente = alto_filas(datos_cliente.len()) + 12.0 + 36.0;
        let alto_contenido = alto_info.max(alto_totales).max(alto_cliente);
        let alto = 1.0 + 24.0 + 28.0 + 12.0 + alto_contenido + 24.0 + 1.0;
        let filas = 3usize.div_ceil(columnas);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto * filas as f32 + 24.0 * (filas as f32 - 1.0)), Sense::hover());
        let caja = |i: usize| {
            let (fila, columna) = (i / columnas, i % columnas);
            Rect::from_min_size(
                egui::pos2(rect.left() + columna as f32 * (ancho_tarjeta + 24.0), rect.top() + fila as f32 * (alto + 24.0)),
                Vec2::new(ancho_tarjeta, alto),
            )
        };
        let fila = |ui: &Ui, x0: f32, x1: f32, y: f32, etiqueta: &str, valor: &str, tam: Tam, color_etiqueta: Color32, color_valor: Color32, peso: Peso| {
            let alto = tam.alto_linea();
            comun::texto_en(ui, egui::pos2(x0, y + alto / 2.0), Align2::LEFT_CENTER, etiqueta, tam, Peso::Normal, color_etiqueta);
            comun::texto_en(ui, egui::pos2(x1, y + alto / 2.0), Align2::RIGHT_CENTER, valor, tam, peso, color_valor);
        };
        let r = caja(0);
        let (x0, x1, mut y) = tarjeta_detalle(ui, r, paleta::BLUE_500, "file-text", "Información de la Factura");
        for (etiqueta, valor) in &info {
            fila(ui, x0, x1, y, etiqueta, valor, Tam::Base, paleta::GRAY_600, color::TEXTO, Peso::Medio);
            y += 36.0;
        }
        let r = caja(1);
        let (x0, x1, mut y) = tarjeta_detalle(ui, r, paleta::GREEN_500, "dollar-sign", "Totales");
        fila(ui, x0, x1, y, "Subtotal:", &formato::pesos(numero(factura, "subtotal")), Tam::Base, paleta::GRAY_600, color::TEXTO, Peso::Medio);
        y += 36.0;
        fila(ui, x0, x1, y, "Impuestos:", &formato::pesos(numero(factura, "impuestos")), Tam::Base, paleta::GRAY_600, color::TEXTO, Peso::Medio);
        y += 36.0;
        ui.painter().hline(x0..=x1, y + 0.5, Stroke::new(1.0, color::BORDE));
        y += 9.0;
        comun::texto_en(ui, egui::pos2(x0, y + 14.0), Align2::LEFT_CENTER, "Total:", Tam::Lg, Peso::Semi, paleta::GREEN_600);
        comun::texto_en(ui, egui::pos2(x1, y + 14.0), Align2::RIGHT_CENTER, &formato::pesos(numero(factura, "total")), Tam::Lg, Peso::Semi, paleta::GREEN_600);
        y += 40.0;
        fila(ui, x0, x1, y, "Total pagado:", &formato::pesos(pagado), Tam::Sm, paleta::GRAY_600, paleta::YELLOW_600, Peso::Medio);
        y += 32.0;
        let color_saldo = if saldo > 0.0 { paleta::RED_600 } else { paleta::GREEN_600 };
        fila(ui, x0, x1, y, "Saldo pendiente:", &formato::pesos(saldo), Tam::Sm, paleta::GRAY_600, color_saldo, Peso::Medio);
        let r = caja(2);
        let (x0, x1, mut y) = tarjeta_detalle(ui, r, paleta::PURPLE_500, "user", "Información del Cliente");
        for (etiqueta, valor) in &datos_cliente {
            fila(ui, x0, x1, y, etiqueta, valor, Tam::Base, paleta::GRAY_600, color::TEXTO, Peso::Medio);
            y += 36.0;
        }
        let ver = Boton::contorno("Ver Cliente").chico().icono("user").separacion(16.0).habilitado(cliente.is_object());
        ver.en(ui, Rect::from_min_max(egui::pos2(x0, y), egui::pos2(x1, y + 36.0)), "ver-cliente").clicked()
    }
}

fn dialogo_cliente(ctx: &egui::Context, cliente: &Value) -> bool {
    let mut datos = vec![("Nombre/Razón Social", logica::nombre_cliente(cliente))];
    for (clave, etiqueta) in [("cuit", "CUIT"), ("email", "Email"), ("telefono", "Teléfono"), ("direccion", "Dirección")] {
        if !texto(cliente, clave).is_empty() {
            datos.push((etiqueta, texto(cliente, clave).to_string()));
        }
    }
    let condicion = texto(cliente, "condicionFiscal");
    datos.push(("Condición Fiscal", if condicion.is_empty() { "N/A".into() } else { condicion_fiscal(condicion) }));
    let mut cerrar = false;
    let r = ui::Dialogo::new("factura-detalle-cliente").ancho(448.0).mostrar(ctx, |ui| {
        comun::encabezado_dialogo(ui, "user", "Información del Cliente", "Datos del cliente de esta factura", 8.0);
        ui.add_space(16.0);
        for (indice, (etiqueta, valor)) in datos.iter().enumerate() {
            if indice > 0 {
                ui.add_space(12.0);
            }
            comun::etiqueta(ui, etiqueta);
            comun::linea(ui, valor.clone(), Tam::Base, Peso::Normal, color::TEXTO);
        }
        ui.add_space(24.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if Boton::primario("Cerrar").ui(ui).clicked() {
                cerrar = true;
            }
        });
    });
    cerrar || r.cerrar
}

fn marco_tarjeta(ui: &Ui, rect: Rect, borde_izquierdo: Color32) {
    ui.painter().add(ui::sombra_sm().as_shape(rect, 8));
    ui.painter().rect(rect, 8, color::FONDO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let izquierda = Rect::from_min_max(rect.min, egui::pos2(rect.left() + 8.0, rect.bottom()));
    let interior = Rect::from_min_max(rect.min + Vec2::new(4.0, 1.0), rect.max - Vec2::new(1.0, 1.0));
    let pintor = ui.painter().with_clip_rect(izquierda);
    pintor.rect_filled(rect, 8, borde_izquierdo);
    pintor.rect_filled(interior, egui::CornerRadius { nw: 4, sw: 4, ne: 7, se: 7 }, color::FONDO);
}

fn tarjeta_detalle(ui: &Ui, rect: Rect, borde_izquierdo: Color32, icono: &str, titulo: &str) -> (f32, f32, f32) {
    marco_tarjeta(ui, rect, borde_izquierdo);
    let x0 = rect.left() + 28.0;
    let x1 = rect.right() - 25.0;
    let y = rect.top() + 25.0;
    ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x0 + 10.0, y + 14.0), Vec2::splat(20.0)), icono, color::TEXTO);
    comun::texto_en(ui, egui::pos2(x0 + 28.0, y + 14.0), Align2::LEFT_CENTER, titulo, Tam::Lg, Peso::Semi, color::TEXTO);
    (x0, x1, y + 28.0 + 12.0)
}

fn encabezado_grande(ui: &Ui, rect: Rect, icono: Option<&str>, titulo: &str) -> (f32, f32, f32) {
    let x0 = rect.left() + 28.0;
    let y = rect.top() + 25.0;
    let mut x = x0;
    if let Some(icono) = icono {
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x0 + 10.0, y + 12.0), Vec2::splat(20.0)), icono, color::TEXTO);
        x += 28.0;
    }
    comun::texto_en(ui, egui::pos2(x, y + 12.0), Align2::LEFT_CENTER, titulo, Tam::X2, Peso::Semi, color::TEXTO);
    (x0, rect.right() - 25.0, y + 24.0 + 24.0)
}

fn productos(ui: &mut Ui, factura: &Value) {
    let detalles = factura["detalles"].as_array().cloned().unwrap_or_default();
    let ancho = ui.available_width();
    let ancho_tabla = ancho - 4.0 - 1.0 - 48.0;
    let encabezados = ["Producto", "Descripción", "Cantidad", "Precio Unit.", "Subtotal"];
    let mut maximos: Vec<f32> = encabezados.iter().map(|e| comun::ancho_texto(ui, e, Tam::Base, Peso::Medio)).collect();
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
    for f in &filas {
        maximos[0] = maximos[0].max(comun::ancho_texto(ui, &f[0], Tam::Base, Peso::Medio)).max(comun::ancho_texto(ui, &f[1], Tam::Sm, Peso::Normal));
        maximos[1] = maximos[1].max(comun::ancho_texto(ui, &f[2], Tam::Base, Peso::Normal));
        maximos[2] = maximos[2].max(comun::ancho_texto(ui, &f[3], Tam::Base, Peso::Normal));
        maximos[3] = maximos[3].max(comun::ancho_texto(ui, &f[4], Tam::Base, Peso::Normal));
        maximos[4] = maximos[4].max(comun::ancho_texto(ui, &f[5], Tam::Base, Peso::Medio));
    }
    let maximos: Vec<f32> = maximos.iter().map(|m| m + 32.0).collect();
    let anchos = anchos_tabla(&maximos, &[None; 5], ancho_tabla);
    let alto = 1.0 + 24.0 + 24.0 + 24.0 + 48.0 + 1.0 + filas.len() as f32 * 69.0 + 24.0 + 1.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    marco_tarjeta(ui, rect, paleta::BLUE_500);
    let (x0, x1, mut y) = encabezado_grande(ui, rect, Some("package"), "Detalle de Productos");
    let columnas: Vec<f32> = anchos.iter().scan(x0, |x, w| {
        let inicio = *x;
        *x += w;
        Some(inicio)
    }).collect();
    let celda = |i: usize| (columnas[i] + 16.0, columnas[i] + anchos[i] - 16.0);
    for (i, encabezado) in encabezados.iter().enumerate() {
        let (izq, der) = celda(i);
        let (x, ancla) = if i >= 3 { (der, Align2::RIGHT_CENTER) } else { (izq, Align2::LEFT_CENTER) };
        comun::texto_en(ui, egui::pos2(x, y + 24.0), ancla, encabezado, Tam::Base, Peso::Medio, paleta::GRAY_600);
    }
    y += 48.0;
    ui.painter().hline(x0..=x1, y + 0.5, Stroke::new(1.0, paleta::GRAY_200));
    y += 1.0;
    for f in &filas {
        let centro = y + 34.0;
        let (izq, _) = celda(0);
        comun::texto_en(ui, egui::pos2(izq, y + 12.0 + 12.0), Align2::LEFT_CENTER, &f[0], Tam::Base, Peso::Medio, color::TEXTO);
        comun::texto_en(ui, egui::pos2(izq, y + 36.0 + 10.0), Align2::LEFT_CENTER, &f[1], Tam::Sm, Peso::Normal, paleta::GRAY_500);
        comun::texto_en(ui, egui::pos2(celda(1).0, centro), Align2::LEFT_CENTER, &f[2], Tam::Base, Peso::Normal, paleta::GRAY_600);
        comun::texto_en(ui, egui::pos2(celda(2).0, centro), Align2::LEFT_CENTER, &f[3], Tam::Base, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(celda(3).1, centro), Align2::RIGHT_CENTER, &f[4], Tam::Base, Peso::Normal, color::TEXTO);
        comun::texto_en(ui, egui::pos2(celda(4).1, centro), Align2::RIGHT_CENTER, &f[5], Tam::Base, Peso::Medio, color::TEXTO);
        y += 68.0;
        ui.painter().hline(x0..=x1, y + 0.5, Stroke::new(1.0, paleta::GRAY_100));
        y += 1.0;
    }
}

fn historial(ui: &mut Ui, factura: &Value) {
    let pagos = factura["pagos"].as_array().cloned().unwrap_or_default();
    let ancho = ui.available_width();
    let alto_pago = |p: &Value| 32.0 + if texto(p, "referencia").is_empty() { 44.0 } else { 60.0 };
    let alto_lista: f32 = pagos.iter().map(alto_pago).sum::<f32>() + pagos.len().saturating_sub(1) as f32 * 12.0;
    let alto = 1.0 + 24.0 + 24.0 + 24.0 + alto_lista + 24.0 + 1.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    marco_tarjeta(ui, rect, paleta::GREEN_500);
    let (x0, x1, mut y) = encabezado_grande(ui, rect, Some("dollar-sign"), "Historial de Pagos");
    for pago in &pagos {
        let h = alto_pago(pago);
        let caja = Rect::from_min_max(egui::pos2(x0, y), egui::pos2(x1, y + h));
        ui.painter().rect_filled(caja, 8, paleta::GRAY_50);
        let circulo = egui::pos2(caja.left() + 36.0, caja.center().y);
        ui.painter().circle_filled(circulo, 20.0, paleta::GREEN_100);
        ui::pintar_icono_en(ui, Rect::from_center_size(circulo, Vec2::splat(20.0)), "dollar-sign", paleta::GREEN_600);
        let tx = caja.left() + 68.0;
        let arriba = caja.top() + 16.0;
        comun::texto_en(ui, egui::pos2(tx, arriba + 12.0), Align2::LEFT_CENTER, &formato::pesos(numero(pago, "monto")), Tam::Base, Peso::Semi, color::TEXTO);
        let metodo = capitalizar(&texto(pago, "metodoPago").replacen('_', " ", 1));
        comun::texto_en(ui, egui::pos2(tx, arriba + 34.0), Align2::LEFT_CENTER, &metodo, Tam::Sm, Peso::Normal, paleta::GRAY_600);
        let referencia = texto(pago, "referencia");
        if !referencia.is_empty() {
            comun::texto_en(ui, egui::pos2(tx, arriba + 52.0), Align2::LEFT_CENTER, &format!("Ref: {referencia}"), Tam::Xs, Peso::Normal, paleta::GRAY_500);
        }
        let derecha = caja.right() - 16.0;
        let medio = caja.center().y;
        comun::texto_en(ui, egui::pos2(derecha, medio - 11.0), Align2::RIGHT_CENTER, &logica::fecha_es_ar(texto(pago, "fecha")), Tam::Sm, Peso::Normal, paleta::GRAY_600);
        let insignia = Insignia::new(texto(pago, "estado"), paleta::GREEN_600, color::PRIMARIO_TEXTO, Color32::TRANSPARENT);
        let w = insignia.ancho_natural(ui);
        insignia.pintar(ui, Rect::from_min_size(egui::pos2(derecha - w, medio - 1.0), Vec2::new(w, 22.0)));
        y += h + 12.0;
    }
}

fn observaciones(ui: &mut Ui, factura: &Value) {
    let ancho = ui.available_width();
    let cuerpo = comun::bloque(ui, texto(factura, "observaciones"), Tam::Base, Peso::Normal, paleta::GRAY_600, ancho - 50.0);
    let alto = 1.0 + 24.0 + 24.0 + 24.0 + cuerpo.alto() + 24.0 + 1.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    ui.painter().add(ui::sombra_sm().as_shape(rect, 8));
    ui.painter().rect(rect, 8, color::FONDO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let x0 = rect.left() + 25.0;
    let y = rect.top() + 25.0;
    comun::texto_en(ui, egui::pos2(x0, y + 12.0), Align2::LEFT_CENTER, "Observaciones", Tam::X2, Peso::Semi, color::TEXTO);
    cuerpo.pintar(ui, egui::pos2(x0, y + 48.0), paleta::GRAY_600);
}

#[cfg(test)]
mod tests {
    use super::{anchos_tabla, capitalizar};

    #[test]
    fn capitaliza_como_css() {
        assert_eq!(capitalizar("remito"), "Remito");
        assert_eq!(capitalizar("responsable inscripto"), "Responsable Inscripto");
    }

    #[test]
    fn reparte_el_sobrante_de_la_tabla() {
        let anchos = anchos_tabla(&[100.0, 100.0], &[None, None], 300.0);
        assert_eq!(anchos, vec![150.0, 150.0]);
        let anchos = anchos_tabla(&[100.0, 50.0], &[None, Some(120.0)], 320.0);
        assert_eq!(anchos, vec![200.0, 120.0]);
    }
}
