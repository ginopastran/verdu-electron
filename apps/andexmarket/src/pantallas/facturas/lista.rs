use eframe::egui::{self, Align, Align2, Id, Layout, Rect, ScrollArea, Sense, Ui, UiBuilder, Vec2};
use nucleo::facturas as logica;
use serde_json::{json, Value};

use super::comun::{self, RELLENO_CHIP_ESTADO, RELLENO_CHIP_TIPO};
use super::formulario::{FormularioFactura, ResultadoFactura};
use super::modal::{ModalFactura, ResultadoModal};
use super::pagos::{self, AgregarPago, ResultadoHistorial, ResultadoPago};
use super::{id_texto, numero, pesos, recien_montada, texto};
use crate::app::{Contexto, Ruta};
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{alto, color, fuente_mono, radio, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, toast, Boton, BotonIcono, CampoTexto, Columna, EstiloSegmentado, Kpi, Modal, Segmento, Tabla};
use crate::ui::Peso;

const ESTADOS: [(&str, &str); 5] = [("", "Todas"), ("pendiente", "Pendientes"), ("pagada", "Pagadas"), ("vencida", "Vencidas"), ("anulada", "Anuladas")];
const TIPOS: [(&str, &str); 4] = [("", "Todos los tipos"), ("remito", "Remito"), ("A", "Factura A"), ("C", "Factura C")];
const COLUMNAS: [Columna; 8] = [
    Columna::fraccion(1.2, 170.0),
    Columna::fraccion(1.4, 170.0),
    Columna::fija(100.0),
    Columna::fija(120.0).derecha(),
    Columna::fija(120.0).derecha(),
    Columna::fija(120.0).derecha(),
    Columna::fija(110.0),
    Columna::fija(200.0).derecha(),
];
const TITULOS: [&str; 8] = ["Comprobante", "Cliente", "Fecha", "Total", "Pagado", "Saldo", "Estado", ""];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Accion {
    Ver,
    Editar,
    Descargar,
    Eliminar,
    VerPagos,
    AgregarPago,
}

#[derive(Clone, PartialEq)]
struct Filtros {
    search: String,
    estado: String,
    tipo_factura: String,
    sort: String,
    order: String,
}

impl Default for Filtros {
    fn default() -> Self {
        Filtros { search: String::new(), estado: String::new(), tipo_factura: String::new(), sort: "fecha".into(), order: "desc".into() }
    }
}

impl Filtros {
    fn activos(&self) -> bool {
        !self.search.is_empty() || !self.estado.is_empty() || !self.tipo_factura.is_empty()
    }
}

#[derive(Clone, PartialEq)]
struct Paginacion {
    total: u32,
    page: u32,
    limit: u32,
    total_pages: u32,
}

impl Default for Paginacion {
    fn default() -> Self {
        Paginacion { total: 0, page: 1, limit: 12, total_pages: 0 }
    }
}

impl Paginacion {
    fn desde(valor: &Value) -> Self {
        let campo = |clave: &str, defecto: u32| valor[clave].as_f64().map_or(defecto, |n| n as u32);
        Paginacion { total: campo("total", 0), page: campo("page", 1), limit: campo("limit", 12), total_pages: campo("totalPages", 0) }
    }
}

pub struct Lista {
    ultimo_pase: u64,
    facturas: Vec<Value>,
    cargando: bool,
    paginacion: Paginacion,
    filtros: Filtros,
    busqueda: String,
    busqueda_desde: f64,
    pedida: Option<(u32, u32, Filtros)>,
    forzar: bool,
    tarea: Option<Tarea<Result<Value, String>>>,
    pago: Option<AgregarPago>,
    historial: Option<Value>,
    formulario: FormularioFactura,
    modal: Option<ModalFactura>,
    eliminar: Option<Value>,
    anulacion: Option<Tarea<Result<String, String>>>,
}

impl Default for Lista {
    fn default() -> Self {
        Lista {
            ultimo_pase: 0,
            facturas: Vec::new(),
            cargando: false,
            paginacion: Paginacion::default(),
            filtros: Filtros::default(),
            busqueda: String::new(),
            busqueda_desde: 0.0,
            pedida: None,
            forzar: false,
            tarea: None,
            pago: None,
            historial: None,
            formulario: FormularioFactura::default(),
            modal: None,
            eliminar: None,
            anulacion: None,
        }
    }
}

impl Lista {
    fn actualizar_filtros(&mut self, cambio: impl FnOnce(&mut Filtros)) {
        cambio(&mut self.filtros);
        self.paginacion.page = 1;
    }

    fn pedir(&mut self, cx: &Contexto) {
        let clave = (self.paginacion.page, self.paginacion.limit, self.filtros.clone());
        if self.pedida.as_ref() == Some(&clave) && !self.forzar {
            return;
        }
        self.pedida = Some(clave);
        self.forzar = false;
        self.cargando = true;
        let api = cx.servicios.api.clone();
        let f = self.filtros.clone();
        let parametros = vec![
            ("excludeTipoFactura", "B".to_string()),
            ("page", self.paginacion.page.to_string()),
            ("limit", self.paginacion.limit.to_string()),
            ("search", f.search),
            ("sort", f.sort),
            ("order", f.order),
            ("estado", f.estado),
            ("tipoFactura", f.tipo_factura),
        ];
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || {
            let respuesta = api.get("/facturas", &parametros).map_err(|e| e.to_string())?;
            if respuesta["facturas"].is_array() {
                Ok(respuesta)
            } else {
                Err("respuesta sin facturas".into())
            }
        }));
    }

    fn recibir(&mut self, cx: &Contexto) {
        let Some(resultado) = esperar(&mut self.tarea) else { return };
        self.cargando = false;
        match resultado {
            Ok(respuesta) => {
                self.facturas = respuesta["facturas"]
                    .as_array()
                    .map(|f| f.iter().filter(|f| f["tipoFactura"] != "B").cloned().collect())
                    .unwrap_or_default();
                self.paginacion = Paginacion::desde(&respuesta["pagination"]);
            }
            Err(_) => toast::error(&cx.egui, "Error al cargar las facturas"),
        }
    }

    fn rebotar_busqueda(&mut self, ui: &Ui) {
        let ahora = ui.input(|i| i.time);
        let quieta = ahora - self.busqueda_desde >= 0.3;
        if !quieta {
            ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64(0.3 - (ahora - self.busqueda_desde)));
        } else if self.busqueda != self.filtros.search {
            let busqueda = self.busqueda.clone();
            self.actualizar_filtros(|f| f.search = busqueda);
        }
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if recien_montada(&mut self.ultimo_pase, ui) {
            *self = Lista { ultimo_pase: self.ultimo_pase, ..Lista::default() };
        }
        self.recibir(cx);
        self.rebotar_busqueda(ui);
        self.pedir(cx);

        comun::fondo(ui);
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let volver = comun::encabezado(ui, "Volver a la venta", "Cuenta corriente");
        let mut accion = None;
        ScrollArea::vertical().id_salt("facturas-lista").auto_shrink(false).show(ui, |ui| {
            egui::Frame::NONE.inner_margin(egui::Margin { left: 28, right: 28, top: 24, bottom: 40 }).show(ui, |ui| {
                ui.set_width(ui.available_width());
                self.kpis(ui);
                ui.add_space(18.0);
                self.filtros_ui(ui);
                ui.add_space(18.0);
                accion = self.tabla(ui);
            });
        });
        if volver {
            cx.global.ir(Ruta::Carrito);
        }
        if let Some((accion, factura)) = accion {
            self.ejecutar(accion, factura, cx);
        }
        self.dialogos(ui, cx);
    }

    fn kpis(&self, ui: &mut Ui) {
        let resumen = logica::resumen_cuenta(&self.facturas, logica::hoy_argentina());
        let completo = self.paginacion.total_pages <= 1 && !self.filtros.activos();
        let en_pantalla = self.facturas.len();
        let saldo = pesos(resumen.saldo);
        let con_saldo = resumen.con_saldo.to_string();
        let cobrado = pesos(resumen.cobrado_mes);
        let (detalle_saldo, detalle_cantidad, detalle_cobrado) = if completo {
            ("Total a cobrar entre todos los clientes".to_string(), format!("de {} en cuenta corriente", self.paginacion.total.max(en_pantalla as u32)), "Pagos registrados".to_string())
        } else {
            (
                format!("Suma de las {en_pantalla} facturas en pantalla"),
                format!("de {en_pantalla} en pantalla"),
                "Pagos de las facturas en pantalla".to_string(),
            )
        };
        let kpis = [
            Kpi {
                etiqueta: "Saldo pendiente",
                valor: &saldo,
                detalle: &detalle_saldo,
                icono: "wallet",
                fondo_icono: color::ROJO_FONDO,
                color_icono: color::ROJO,
                color_valor: color::ROJO,
            },
            Kpi {
                etiqueta: "Facturas con saldo",
                valor: &con_saldo,
                detalle: &detalle_cantidad,
                icono: "file-text",
                fondo_icono: color::VERDE_CHIP,
                color_icono: color::VERDE,
                color_valor: color::TEXTO,
            },
            Kpi {
                etiqueta: "Cobrado este mes",
                valor: &cobrado,
                detalle: &detalle_cobrado,
                icono: "hand-coins",
                fondo_icono: color::VERDE_CHIP,
                color_icono: color::VERDE,
                color_valor: color::VERDE_EXITO,
            },
        ];
        let ancho = (ui.available_width() - 14.0 * 2.0) / 3.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            for kpi in &kpis {
                v2::tarjeta_kpi(ui, kpi, ancho);
            }
        });
    }

    fn filtros_ui(&mut self, ui: &mut Ui) {
        let segmentos: Vec<Segmento> = ESTADOS.iter().map(|(_, t)| Segmento::texto(t)).collect();
        let boton_nueva = Boton::primario("Nueva factura")
            .icono("plus")
            .tam_icono(15.0)
            .alto(alto::CONTROL)
            .relleno(16.0)
            .radio(radio::CONTROL)
            .tam_texto(tam::CUERPO)
            .sin_sombra();
        let ancho_nueva = 32.0 + 15.0 + 8.0 + v2::ancho_texto(ui, "Nueva factura", tam::CUERPO, Peso::Semi);
        let ancho_segmentos = ESTADOS.iter().map(|(_, t)| v2::ancho_texto(ui, t, tam::CONTROL, Peso::Semi) + 24.0).sum::<f32>() + 2.0 * 4.0 + 8.0;
        let libre = ui.available_width() - ancho_segmentos - 170.0 - ancho_nueva - 8.0 * 4.0;
        let ancho_busqueda = (libre / 2.0).clamp(240.0, 380.0);
        let mut nueva = false;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let r = CampoTexto::new(&mut self.busqueda)
                .ancho(ancho_busqueda)
                .alto(alto::CONTROL)
                .radio(radio::CONTROL)
                .relleno(12.0, 12.0)
                .icono("search")
                .tam_texto(tam::CUERPO)
                .sugerencia("Buscar por cliente o número")
                .id(Id::new("facturas-busqueda"))
                .ui(ui);
            if r.respuesta.changed() {
                self.busqueda_desde = ui.input(|i| i.time);
            }
            let mut estado = ESTADOS.iter().position(|(v, _)| *v == self.filtros.estado).unwrap_or(0);
            if v2::segmentado(ui, &segmentos, &mut estado, None, EstiloSegmentado::Filtro).changed() {
                self.actualizar_filtros(|f| f.estado = ESTADOS[estado].0.to_string());
            }
            let mut tipo = TIPOS.iter().position(|(v, _)| *v == self.filtros.tipo_factura).unwrap_or(0);
            let opciones: Vec<String> = TIPOS.iter().map(|(_, t)| t.to_string()).collect();
            if v2::desplegable(ui, "facturas-tipo", &mut tipo, &opciones, "", 170.0, alto::CONTROL).changed() {
                self.actualizar_filtros(|f| f.tipo_factura = TIPOS[tipo].0.to_string());
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                nueva = boton_nueva.ui(ui).clicked();
            });
        });
        if nueva {
            self.formulario = FormularioFactura::default();
            self.formulario.abrir(Vec::new());
        }
    }

    fn tabla(&mut self, ui: &mut Ui) -> Option<(Accion, Value)> {
        let mut accion = None;
        v2::marco_tabla().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let tabla = Tabla::new(&COLUMNAS);
            tabla.encabezado(ui, &TITULOS, alto::ENCABEZADO_TABLA);
            if self.cargando && self.facturas.is_empty() {
                for _ in 0..6 {
                    tabla.fila(ui, alto::FILA_TABLA, None, |ui, celdas| {
                        for (i, celda) in celdas.iter().enumerate().take(7) {
                            let ancho = if i < 2 { celda.width() * 0.6 } else { celda.width() * 0.7 };
                            let x = if COLUMNAS[i].derecha { celda.right() - ancho } else { celda.left() };
                            v2::tarjetas::pintar_shimmer(ui, Rect::from_min_size(egui::pos2(x, celda.center().y - 7.0), Vec2::new(ancho, 14.0)), 6, true);
                        }
                    });
                }
            } else if self.facturas.is_empty() {
                self.vacio(ui);
            } else {
                let facturas = std::mem::take(&mut self.facturas);
                for factura in &facturas {
                    if let Some(a) = fila(ui, &tabla, factura) {
                        accion = Some((a, factura.clone()));
                    }
                }
                self.facturas = facturas;
            }
            self.pie(ui);
        });
        accion
    }

    fn vacio(&mut self, ui: &mut Ui) {
        let con_filtros = self.filtros.activos();
        let mensaje = if con_filtros { "No hay facturas que coincidan" } else { "Todavía no hay facturas registradas" };
        comun::vacio(ui, "doc", mensaje);
        if con_filtros {
            ui.add_space(16.0);
            let boton = Boton::secundario("Limpiar filtros").alto(36.0).radio(radio::CONTROL).tam_texto(tam::CONTROL);
            if comun::boton_centrado(ui, boton) {
                self.busqueda.clear();
                self.actualizar_filtros(|f| {
                    f.search.clear();
                    f.estado.clear();
                    f.tipo_factura.clear();
                });
            }
        }
        ui.add_space(48.0);
    }

    fn pie(&mut self, ui: &mut Ui) {
        let p = self.paginacion.clone();
        if p.total_pages <= 1 {
            return;
        }
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 54.0), Sense::hover());
        let desde = p.page.saturating_sub(1) * p.limit + 1;
        let hasta = (p.page * p.limit).min(p.total);
        let interior = rect.shrink2(Vec2::new(18.0, 0.0));
        let info = format!("{desde}–{hasta} de {} comprobantes", nucleo::formato::numero(p.total as f64));
        v2::pintar_texto(ui, interior.left_center(), Align2::LEFT_CENTER, info, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(interior).layout(Layout::right_to_left(Align::Center)));
        hijo.spacing_mut().item_spacing.x = 6.0;
        if comun::boton_pagina(&mut hijo, "Siguiente", "chevron-right", true, p.page < p.total_pages).clicked() {
            self.paginacion.page = p.page + 1;
        }
        if comun::boton_pagina(&mut hijo, "Anterior", "chevron-left", false, p.page > 1).clicked() {
            self.paginacion.page = p.page.saturating_sub(1).max(1);
        }
    }

    fn ejecutar(&mut self, accion: Accion, factura: Value, cx: &mut Contexto) {
        match accion {
            Accion::Ver => cx.global.ir(Ruta::FacturaDetalle(id_texto(&factura["id"]))),
            Accion::Editar => self.modal = Some(ModalFactura::new(factura)),
            Accion::Descargar => pagos::descargar_pdf(cx, &factura),
            Accion::Eliminar => self.eliminar = Some(factura),
            Accion::VerPagos => self.historial = Some(factura),
            Accion::AgregarPago => self.pago = Some(AgregarPago::new(factura)),
        }
    }

    fn dialogos(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if let Some(factura) = &self.historial {
            match pagos::historial(ui.ctx(), factura) {
                ResultadoHistorial::Nada => {}
                ResultadoHistorial::Cerrar => self.historial = None,
                ResultadoHistorial::AgregarPago => {
                    self.pago = self.historial.take().map(AgregarPago::new);
                }
            }
        }
        if let Some(pago) = &mut self.pago {
            match pago.mostrar(ui.ctx(), cx) {
                ResultadoPago::Nada => {}
                ResultadoPago::Cerrar => self.pago = None,
                ResultadoPago::Exito => {
                    self.pago = None;
                    self.forzar = true;
                }
            }
        }
        if self.formulario.abierto() {
            match self.formulario.mostrar(ui, cx) {
                Some(ResultadoFactura::Cerrada) => {}
                Some(ResultadoFactura::Creada(_)) => self.forzar = true,
                None => {}
            }
        }
        if let Some(modal) = &mut self.modal {
            match modal.mostrar(ui.ctx(), cx) {
                ResultadoModal::Nada => {}
                ResultadoModal::Cerrar => self.modal = None,
                ResultadoModal::Actualizada => self.forzar = true,
            }
        }
        self.dialogo_eliminar(ui, cx);
    }

    fn anular(&mut self, cx: &Contexto, factura: &Value) {
        let api = cx.servicios.api.clone();
        let ruta = format!("/api/facturas/{}", nucleo::api::codificar(&id_texto(&factura["id"])));
        self.anulacion = Some(Tarea::lanzar(&cx.egui, move || {
            let respuesta = api.pedir("DELETE", &ruta, Some(&json!({ "forceDelete": false })), &[]).map_err(|e| e.mensaje)?;
            if respuesta.ok() {
                Ok(respuesta.cuerpo["message"].as_str().unwrap_or("Factura anulada correctamente").to_string())
            } else {
                Err(respuesta.error())
            }
        }));
    }

    fn dialogo_eliminar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if let Some(resultado) = esperar(&mut self.anulacion) {
            match resultado {
                Ok(mensaje) => {
                    toast::exito(&cx.egui, mensaje);
                    self.eliminar = None;
                    self.forzar = true;
                }
                Err(error) => toast::error(&cx.egui, format!("No se pudo eliminar la factura: {error}")),
            }
        }
        let Some(factura) = self.eliminar.clone() else { return };
        let anulando = self.anulacion.is_some();
        let mut confirmar = false;
        let mut cancelar = false;
        let numero_factura = texto(&factura, "numero").to_string();
        let r = Modal::new("facturas-eliminar", "¿Eliminar la factura?").subtitulo(&numero_factura).icono("trash-2").peligro().ancho(440.0).mostrar(ui.ctx(), |ui| {
            v2::cuerpo_modal(ui, 16, |ui| {
                let ancho = ui.available_width();
                v2::parrafo(
                    ui,
                    "No se borra: queda anulada y se repone el stock de sus productos. Esta acción no se puede deshacer.",
                    tam::CUERPO,
                    color::TEXTO_SECUNDARIO,
                    1.55,
                    ancho,
                );
            });
            v2::pie_modal(ui, |ui| {
                let texto_boton = if anulando { "Anulando…" } else { "Anular factura" };
                confirmar = Boton::peligro(texto_boton).modal().cargando(anulando).ui(ui).clicked();
                cancelar = Boton::secundario("Cancelar").modal().habilitado(!anulando).ui(ui).clicked();
            });
        });
        if confirmar {
            self.anular(cx, &factura);
        } else if (cancelar || r.cerrar) && !anulando {
            self.eliminar = None;
        }
    }
}

fn fila(ui: &mut Ui, tabla: &Tabla, factura: &Value) -> Option<Accion> {
    let mut accion = None;
    let total = numero(factura, "total");
    let pagado = logica::total_pagado(factura);
    let saldo = logica::saldo_pendiente(factura);
    let id = id_texto(&factura["id"]);
    let respuesta = tabla.fila(ui, alto::FILA_TABLA, None, |ui, celdas| {
        let pintor = ui.painter().clone();
        let numero_factura = v2::galeria(ui, texto(factura, "numero"), fuente_mono(tam::ETIQUETA, Peso::Medio), color::TEXTO);
        let (tipo, colores_tipo) = comun::tipo(texto(factura, "tipoFactura"));
        let tam_chip = comun::medir_chip(ui, &tipo, tam::MINI, RELLENO_CHIP_TIPO);
        let arriba = celdas[0].center().y - (numero_factura.size().y + 3.0 + tam_chip.y) / 2.0;
        pintor.galley(egui::pos2(celdas[0].left(), arriba), numero_factura.clone(), color::TEXTO);
        comun::pintar_chip(ui, egui::pos2(celdas[0].left(), arriba + numero_factura.size().y + 3.0), &tipo, tam::MINI, RELLENO_CHIP_TIPO, colores_tipo.0, colores_tipo.1);

        let cantidad = factura["detalles"].as_array().map_or(0, Vec::len);
        let arriba = celdas[1].center().y - 16.5;
        celda_texto(&pintor, Rect::from_x_y_ranges(celdas[1].x_range(), arriba..=arriba + 17.0), &comun::nombre_cliente(factura), tam::CUERPO, Peso::Medio, color::TEXTO, false);
        celda_texto(&pintor, Rect::from_x_y_ranges(celdas[1].x_range(), arriba + 18.0..=arriba + 33.0), &comun::productos(cantidad), tam::XS, Peso::Normal, color::TEXTO_TENUE, false);

        celda_texto(&pintor, celdas[2], &logica::fecha_es_ar(texto(factura, "fecha")), tam::CUERPO, Peso::Normal, color::TEXTO_SECUNDARIO, false);
        celda_texto(&pintor, celdas[3], &pesos(total), tam::CUERPO, Peso::Semi, color::TEXTO, true);
        celda_texto(&pintor, celdas[4], &pesos(pagado), tam::CUERPO, Peso::Normal, color::VERDE_EXITO, true);
        celda_texto(&pintor, celdas[5], &pesos(saldo), tam::CUERPO, Peso::Semi, if saldo > 0.0 { color::ROJO } else { color::VERDE_EXITO }, true);
        let (estado, colores_estado) = comun::estado(factura);
        let tam_estado = comun::medir_chip(ui, &estado, tam::XS, RELLENO_CHIP_ESTADO);
        comun::pintar_chip(ui, egui::pos2(celdas[6].left(), celdas[6].center().y - tam_estado.y / 2.0), &estado, tam::XS, RELLENO_CHIP_ESTADO, colores_estado.0, colores_estado.1);

        let mut acciones = ui.new_child(UiBuilder::new().id_salt(("acciones", &id)).max_rect(celdas[7]).layout(Layout::right_to_left(Align::Center)));
        acciones.spacing_mut().item_spacing.x = 4.0;
        if BotonIcono::quitar().ui(&mut acciones).on_hover_text("Eliminar").clicked() {
            accion = Some(Accion::Eliminar);
        }
        if BotonIcono::new("download").ui(&mut acciones).on_hover_text("Descargar PDF").clicked() {
            accion = Some(Accion::Descargar);
        }
        if BotonIcono::new("square-pen").ui(&mut acciones).on_hover_text("Editar").clicked() {
            accion = Some(Accion::Editar);
        }
        if BotonIcono::new("history").colores(color::VERDE, color::VERDE_CHIP, color::VERDE).ui(&mut acciones).on_hover_text("Ver pagos").clicked() {
            accion = Some(Accion::VerPagos);
        }
        if saldo > 0.0 {
            let pago = Boton::primario("Pago")
                .icono("hand-coins")
                .tam_icono(15.0)
                .separacion(6.0)
                .alto(alto::BOTON_CHICO)
                .relleno(10.0)
                .radio(radio::ICONO)
                .tam_texto(tam::CHICO)
                .sin_sombra();
            if pago.ui(&mut acciones).clicked() {
                accion = Some(Accion::AgregarPago);
            }
        }
    });
    if accion.is_none() && respuesta.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        accion = Some(Accion::Ver);
    }
    accion
}
