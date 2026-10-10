use eframe::egui::{self, Align2, Color32, Id, Rect, ScrollArea, Sense, Ui, Vec2};
use serde_json::Value;

use super::comun::{self, Boton, Opcion, Relleno};
use super::formulario::{FormularioFactura, ResultadoFactura};
use super::modal::{ModalFactura, ResultadoModal};
use super::pagos::{AgregarPago, ResultadoPago};
use super::tarjeta::{self, Accion};
use super::{id_texto, pagos, recien_montada};
use crate::app::{Contexto, Ruta};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, Entrada, Peso, Tam};

const ESTADOS: [(&str, &str); 5] =
    [("", "Todos los estados"), ("pendiente", "Pendiente"), ("pagada", "Pagada"), ("anulada", "Anulada"), ("vencida", "Vencida")];
const TIPOS: [(&str, &str); 4] = [("", "Todos los tipos"), ("A", "Factura A"), ("C", "Factura C"), ("remito", "Remito")];

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
    eliminar: bool,
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
            eliminar: false,
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
            Err(_) => ui::toast::error(&cx.egui, "Error al cargar las facturas"),
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

        let mut ir_carrito = false;
        let mut accion = None;
        ui.spacing_mut().scroll = egui::style::ScrollStyle {
            floating: false,
            bar_width: 11.0,
            bar_inner_margin: 2.0,
            bar_outer_margin: 2.0,
            ..egui::style::ScrollStyle::solid()
        };
        ScrollArea::vertical().id_salt("facturas-lista").auto_shrink(false).show(ui, |ui| {
            ui.set_width(ui.available_width());
            comun::encabezado_verde(ui, 84.0, |ui| {
                if comun::boton_volver(ui).clicked() {
                    ir_carrito = true;
                }
                ui.add_space(16.0);
                ui.label(ui::texto("Cuenta Corriente", Tam::X3, Peso::Semi, Color32::WHITE));
                comun::logo(ui);
            });
            egui::Frame::new().inner_margin(24).show(ui, |ui| {
                ui.set_width(ui.available_width());
                accion = self.contenido(ui);
            });
        });
        if ir_carrito {
            cx.global.ir(Ruta::Carrito);
        }
        if let Some((accion, factura)) = accion {
            self.ejecutar(accion, factura, cx);
        }
        self.dialogos(ui, cx);
    }

    fn contenido(&mut self, ui: &mut Ui) -> Option<(Accion, Value)> {
        if self.cargando {
            comun::cargando_centro(ui, 256.0, "Cargando facturas...", paleta::EMERALD_600, paleta::GRAY_600, Tam::Base);
            return None;
        }
        self.filtros_ui(ui);
        ui.add_space(24.0);
        if self.facturas.is_empty() {
            self.vacio(ui);
            return None;
        }
        let accion = self.grilla(ui);
        let p = self.paginacion.clone();
        if p.total_pages > 1 {
            ui.add_space(24.0);
            let desde = (p.page.saturating_sub(1)) * p.limit + 1;
            let hasta = (p.page * p.limit).min(p.total);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::hover());
            comun::texto_en(
                ui,
                egui::pos2(rect.left(), rect.center().y),
                Align2::LEFT_CENTER,
                &format!("Mostrando {desde} a {hasta} de {} facturas", p.total),
                Tam::Sm,
                Peso::Normal,
                paleta::GRAY_500,
            );
            let paginas: Vec<u32> = (1..=p.total_pages.min(5)).collect();
            if let Some(destino) = comun::paginacion_compacta(ui, rect, "facturas", p.page, &paginas, p.page > 1, p.page < p.total_pages) {
                self.paginacion.page = destino;
            }
        }
        accion
    }

    fn filtros_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let ancho_busqueda = (ui.available_width() - 12.0 - 192.0 * 2.0 - 8.0).clamp(0.0, 448.0);
            let r = ui
                .scope(|ui| {
                    Entrada::new(&mut self.busqueda)
                        .ancho(ancho_busqueda)
                        .tam(Tam::Sm)
                        .icono("search")
                        .sugerencia("Buscar facturas...")
                        .id(Id::new("facturas-busqueda"))
                        .ui(ui)
                })
                .inner;
            if r.changed() {
                self.busqueda_desde = ui.input(|i| i.time);
            }
            ui.add_space(12.0);
            let mut estado = ESTADOS.iter().position(|(v, _)| *v == self.filtros.estado).unwrap_or(0);
            let opciones: Vec<Opcion> = ESTADOS.iter().map(|(_, t)| Opcion { icono: None, texto: t.to_string() }).collect();
            if comun::selector(ui, "estado", 192.0, &mut estado, &opciones, true) {
                self.actualizar_filtros(|f| f.estado = ESTADOS[estado].0.to_string());
            }
            ui.add_space(8.0);
            let mut tipo = TIPOS.iter().position(|(v, _)| *v == self.filtros.tipo_factura).unwrap_or(0);
            let opciones: Vec<Opcion> = TIPOS.iter().map(|(_, t)| Opcion { icono: None, texto: t.to_string() }).collect();
            if comun::selector(ui, "tipo", 192.0, &mut tipo, &opciones, true) {
                self.actualizar_filtros(|f| f.tipo_factura = TIPOS[tipo].0.to_string());
            }
        });
    }

    fn vacio(&mut self, ui: &mut Ui) {
        let con_filtros = !self.filtros.search.is_empty() || !self.filtros.estado.is_empty() || !self.filtros.tipo_factura.is_empty();
        ui.add_space(48.0);
        let centro = ui.max_rect().center().x;
        let renglon = |ui: &mut Ui, texto: &str, tam: Tam, color: Color32| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), tam.alto_linea()), Sense::hover());
            comun::texto_en(ui, egui::pos2(centro, rect.center().y), Align2::CENTER_CENTER, texto, tam, Peso::Normal, color);
        };
        let titulo = if con_filtros { "No se encontraron facturas con los filtros aplicados" } else { "No hay facturas registradas" };
        renglon(ui, titulo, Tam::Base, paleta::GRAY_600);
        ui.add_space(16.0);
        renglon(ui, "Esta página muestra todas las facturas registradas en el sistema.", Tam::Sm, paleta::GRAY_500);
        ui.add_space(16.0);
        let boton = if con_filtros {
            Boton::contorno("Limpiar filtros")
        } else {
            Boton::primario("Crear primera factura")
                .icono("plus")
                .separacion(16.0)
                .relleno(Relleno::Liso(paleta::EMERALD_600))
                .encima(Relleno::Liso(paleta::EMERALD_700))
        };
        let ancho = boton.ancho_natural(ui);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::hover());
        let caja = Rect::from_center_size(egui::pos2(centro, rect.center().y), Vec2::new(ancho, 40.0));
        if boton.en(ui, caja, "vacio").clicked() {
            if con_filtros {
                self.actualizar_filtros(|f| {
                    f.search.clear();
                    f.estado.clear();
                    f.tipo_factura.clear();
                });
            } else {
                self.formulario = FormularioFactura::default();
                self.formulario.abrir(Vec::new());
            }
        }
        ui.add_space(48.0);
    }

    fn grilla(&mut self, ui: &mut Ui) -> Option<(Accion, Value)> {
        let ancho_ventana = ui.ctx().content_rect().width();
        let columnas = if ancho_ventana >= 1280.0 {
            4
        } else if ancho_ventana >= 1024.0 {
            3
        } else if ancho_ventana >= 768.0 {
            2
        } else {
            1
        };
        let separacion = 24.0;
        let ancho = ui.available_width();
        let ancho_tarjeta = (ancho - separacion * (columnas as f32 - 1.0)) / columnas as f32;
        let mut accion = None;
        for (fila, grupo) in self.facturas.chunks(columnas).enumerate() {
            if fila > 0 {
                ui.add_space(separacion);
            }
            let alto = grupo.iter().map(|f| tarjeta::alto(ui, f, ancho_tarjeta)).fold(0.0, f32::max);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
            for (columna, factura) in grupo.iter().enumerate() {
                let x = rect.left() + columna as f32 * (ancho_tarjeta + separacion);
                let caja = Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(ancho_tarjeta, alto));
                if let Some(a) = tarjeta::pintar(ui, factura, caja) {
                    accion = Some((a, factura.clone()));
                }
            }
        }
        accion
    }

    fn ejecutar(&mut self, accion: Accion, factura: Value, cx: &mut Contexto) {
        match accion {
            Accion::Ver => cx.global.ir(Ruta::FacturaDetalle(id_texto(&factura["id"]))),
            Accion::Editar => self.modal = Some(ModalFactura::new(factura)),
            Accion::Descargar => pagos::descargar_pdf(cx, &factura["id"], true),
            Accion::Eliminar => self.eliminar = true,
            Accion::VerPagos => self.historial = Some(factura),
            Accion::AgregarPago => self.pago = Some(AgregarPago::new(factura)),
        }
    }

    fn dialogos(&mut self, ui: &mut Ui, cx: &mut Contexto) {
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
        if let Some(factura) = &self.historial {
            if pagos::historial(ui.ctx(), factura) {
                self.historial = None;
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
            if let ResultadoModal::Cerrar = modal.mostrar(ui.ctx(), cx) {
                self.modal = None;
            }
        }
        if self.eliminar {
            let r = ui::Dialogo::new("facturas-eliminar").ancho(448.0).mostrar(ui.ctx(), |ui| {
                ui.label(ui::texto("¿Estás seguro de que quieres eliminar esta factura?", Tam::Sm, Peso::Normal, color::TEXTO));
                ui.add_space(24.0);
                ui::pie_dialogo(ui, |ui| {
                    let aceptar = Boton::primario("Aceptar").ui(ui).clicked();
                    let cancelar = Boton::contorno("Cancelar").ui(ui).clicked();
                    aceptar || cancelar
                })
            });
            if r.cerrar || r.interior {
                self.eliminar = false;
            }
        }
    }
}
