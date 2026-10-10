mod dialogos;
mod diseno;
pub mod productos;
mod vista;

use std::time::Duration;

use eframe::egui::{self, Event, Id, Key, Modifiers, Pos2, Rect, ScrollArea, Sense, Ui, Vec2};
use nucleo::carrito::Producto;
use serde_json::Value;

use self::dialogos::{DialogoAgregar, DialogoEscanear, Salida};
use self::productos::{FilaLista, ProductoDisponible};
use self::vista::{AccionCabecera, AccionPestana, Cabecera, Resumen, ANCHO_BARRA};
use super::cierre::DialogoCierre;
use super::cobro::{AccionCobro, Cobro};
use super::configuracion::balanza::DialogoBalanza;
use super::configuracion::impresion::DialogoImpresion;
use super::facturas::formulario::{FormularioFactura, ProductoInicial, ResultadoFactura};
use super::pedidos::DialogoPedidos;
use super::sesion;
use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::{self, fuente, paleta, Entrada, Peso, Tam};

pub const ID_BUSCADOR: &str = "buscador-productos";
const CUADROS_ENFOQUE: u8 = 2;

pub fn id_buscador() -> Id {
    Id::new(ID_BUSCADOR)
}

#[derive(Default)]
struct Busqueda {
    consulta: String,
    resultados: Vec<usize>,
    seleccion: Option<usize>,
}

impl Busqueda {
    fn filtrar(&mut self, productos: &[ProductoDisponible]) {
        self.resultados = productos::buscar(productos, &self.consulta);
        self.seleccion = (!self.resultados.is_empty()).then_some(0);
    }

    fn limpiar(&mut self) {
        *self = Busqueda::default();
    }
}

#[derive(Default)]
struct Lector {
    buffer: String,
    procesar_en: Option<f64>,
    pegado: Option<(String, f64)>,
    ignorar_hasta: f64,
}

type ResultadoEscaneo = (ProductoDisponible, f64, Option<Vec<FilaLista>>);

#[derive(Default)]
pub struct Carrito {
    pub cobro: Cobro,
    pub factura: FormularioFactura,
    pub cierre: DialogoCierre,
    pub pedidos: DialogoPedidos,
    pub impresion: DialogoImpresion,
    pub balanza: DialogoBalanza,
    pub menu_usuario_visible: bool,
    productos: Vec<ProductoDisponible>,
    productos_precargados: bool,
    tarea_productos: Option<Tarea<Result<Vec<ProductoDisponible>, String>>>,
    tarea_logo: Option<Tarea<()>>,
    tarea_escaneo: Option<Tarea<ResultadoEscaneo>>,
    ultimo_escaneo: Option<(String, f64)>,
    busqueda: Busqueda,
    lector: Lector,
    agregar: DialogoAgregar,
    escanear: DialogoEscanear,
    borrar_pantalla: Option<usize>,
    cerrar_sesion_en: Option<f64>,
    ultimo_frame: Option<u64>,
    habia_dialogo: bool,
    enfocar: u8,
    buscador_enfocado: bool,
}

impl Carrito {
    pub fn precargar_productos(&mut self, productos: Vec<ProductoDisponible>) {
        self.productos = productos;
        self.productos_precargados = true;
    }

    fn agregar_escaneado(&mut self, producto: ProductoDisponible, cantidad: f64, cx: &mut Contexto) {
        if self.tarea_escaneo.is_some() {
            return;
        }
        let ahora = cx.egui.input(|i| i.time);
        let clave = format!("{}-{}", producto.id, producto.nombre);
        if self.ultimo_escaneo.as_ref().is_some_and(|(c, t)| *c == clave && ahora - t < 1.0) {
            return;
        }
        self.ultimo_escaneo = Some((clave, ahora));
        if cx.global.negocio_bool("popupListaPreciosEnabled") {
            let api = cx.servicios.api.clone();
            self.tarea_escaneo = Some(Tarea::lanzar(&cx.egui, move || {
                let filas = productos::listas_de_precios(&api, producto.id).filter(|f| !f.is_empty());
                (producto, cantidad, filas)
            }));
            return;
        }
        self.agregar_directo(&producto, cantidad, cx);
    }

    fn agregar_directo(&mut self, producto: &ProductoDisponible, cantidad: f64, cx: &mut Contexto) {
        let etiqueta = dialogos::etiqueta_base(cx.global.negocio.as_ref());
        let item = producto.item(cantidad, producto.unidad_o_defecto(), producto.precio, None, &etiqueta);
        cx.global.carritos.agregar(item);
        self.busqueda.limpiar();
        self.enfocar = CUADROS_ENFOQUE;
    }

    fn montar(&mut self, cx: &mut Contexto) {
        self.busqueda.limpiar();
        self.lector = Lector::default();
        self.agregar = DialogoAgregar::default();
        self.agregar.peso_manual = cx.global.usuario.as_ref().is_some_and(|u| u["permisos"]["pesoManualEnabled"] == Value::Bool(true));
        self.escanear = DialogoEscanear::default();
        self.borrar_pantalla = None;
        self.cerrar_sesion_en = None;
        self.enfocar = CUADROS_ENFOQUE;
        if !self.productos_precargados {
            let api = cx.servicios.api.clone();
            self.tarea_productos = Some(Tarea::lanzar(&cx.egui, move || productos::descargar_todos(&api)));
        }
        if cx.global.usuario.is_some() {
            let servicios = cx.servicios.clone();
            self.tarea_logo = Some(Tarea::lanzar(&cx.egui, move || sincronizar_logo(&servicios)));
        }
    }

    fn atender_tareas(&mut self, cx: &mut Contexto) {
        if let Some(resultado) = tareas::esperar(&mut self.tarea_productos) {
            match resultado {
                Ok(lista) => {
                    self.productos = lista;
                    if !self.busqueda.consulta.is_empty() {
                        self.busqueda.filtrar(&self.productos);
                    }
                }
                Err(mensaje) => ui::toast::error(&cx.egui, mensaje),
            }
        }
        let _ = tareas::esperar(&mut self.tarea_logo);
        if let Some((producto, cantidad, filas)) = tareas::esperar(&mut self.tarea_escaneo) {
            match filas {
                Some(filas) => self.escanear.abrir(producto, filas, cantidad, cx.global.negocio.as_ref()),
                None => self.agregar_directo(&producto, cantidad, cx),
            }
        }
    }

    fn items_actuales(cx: &Contexto) -> usize {
        cx.global.carritos.items().len()
    }

    fn sin_productos(cx: &Contexto, detalle: &str) -> bool {
        if Self::items_actuales(cx) == 0 {
            ui::toast::con_detalle(&cx.egui, ui::toast::Tipo::Error, "No hay productos en el carrito", detalle, 2.0);
            return true;
        }
        false
    }

    fn con_afip(cx: &Contexto) -> bool {
        cx.global.negocio_bool("facturacionHabilitada") && cx.global.negocio_bool("afipHabilitado")
    }

    fn cancelar(&mut self, cx: &mut Contexto) {
        if !Self::sin_productos(cx, "El carrito ya está vacío") {
            self.cobro.iniciar(AccionCobro::Cancelar, cx);
        }
    }

    fn pagar(&mut self, cx: &mut Contexto, afip_si_corresponde: bool) {
        if Self::sin_productos(cx, "Agrega al menos un producto antes de continuar") {
            return;
        }
        let accion = if afip_si_corresponde && Self::con_afip(cx) { AccionCobro::PagarAfip } else { AccionCobro::Pagar };
        self.cobro.iniciar(accion, cx);
    }

    fn pagar_con_descuento(&mut self, cx: &mut Contexto, afip_si_corresponde: bool) {
        if Self::sin_productos(cx, "Agrega al menos un producto antes de continuar") {
            return;
        }
        let accion = if afip_si_corresponde && Self::con_afip(cx) { AccionCobro::PagarAfipConDescuento } else { AccionCobro::PagarConDescuento };
        self.cobro.iniciar(accion, cx);
    }

    fn cuenta_corriente(&mut self, cx: &mut Contexto) {
        if !cx.global.negocio_bool("facturacionHabilitada") {
            ui::toast::con_detalle(&cx.egui, ui::toast::Tipo::Error, "Funcionalidad no disponible", "La facturación no está habilitada para este negocio", 2.0);
            return;
        }
        if Self::sin_productos(cx, "Agrega al menos un producto antes de crear la factura") {
            return;
        }
        let iniciales = cx
            .global
            .carritos
            .items()
            .iter()
            .map(|item| ProductoInicial { id: item.id, nombre: item.name.clone(), precio: item.price_per_unit, cantidad: item.quantity, subtotal: item.subtotal })
            .collect();
        self.factura.abrir(iniciales);
    }

    fn atajos(&mut self, ui: &Ui, cx: &mut Contexto) {
        let teclas = [Key::F1, Key::F2, Key::F3, Key::F4, Key::F6, Key::F7, Key::F8];
        let pulsadas: Vec<Key> = ui.input_mut(|i| teclas.into_iter().filter(|k| i.consume_key(Modifiers::NONE, *k)).collect());
        for tecla in pulsadas {
            match tecla {
                Key::F4 => {
                    ui::toast::exito(&cx.egui, "Cerrando sesión...");
                    self.cerrar_sesion_en = Some(ui.input(|i| i.time) + 1.0);
                }
                Key::F1 => self.cancelar(cx),
                Key::F2 => self.pagar(cx, true),
                Key::F3 => self.pagar(cx, false),
                Key::F6 => self.cuenta_corriente(cx),
                Key::F7 => self.pagar_con_descuento(cx, true),
                Key::F8 => self.pagar_con_descuento(cx, false),
                _ => {}
            }
        }
    }

    fn hay_dialogo(&self) -> bool {
        self.agregar.abierto()
            || self.escanear.abierto()
            || self.borrar_pantalla.is_some()
            || self.cobro.ocupado()
            || self.factura.abierto()
            || self.pedidos.abierto()
            || self.cierre.abierto()
            || self.impresion.abierto()
            || self.balanza.abierto()
    }

    fn elegir_producto(&mut self, indice: usize, cx: &Contexto) {
        let producto = self.productos[indice].clone();
        self.busqueda.limpiar();
        self.agregar.abrir(producto, cx);
    }

    fn teclas_busqueda(&mut self, ui: &Ui, cx: &Contexto) -> bool {
        if !self.buscador_enfocado {
            return false;
        }
        let (abajo, arriba, enter, escape) = ui.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::ArrowDown),
                i.consume_key(Modifiers::NONE, Key::ArrowUp),
                i.consume_key(Modifiers::NONE, Key::Enter),
                i.key_pressed(Key::Escape),
            )
        });
        if self.busqueda.resultados.is_empty() {
            return false;
        }
        let total = self.busqueda.resultados.len();
        if abajo {
            self.busqueda.seleccion = Some(self.busqueda.seleccion.map_or(0, |s| (s + 1).min(total - 1)));
        }
        if arriba {
            self.busqueda.seleccion = self.busqueda.seleccion.map(|s| s.saturating_sub(1));
        }
        if escape {
            self.busqueda.limpiar();
            ui.memory_mut(|m| m.request_focus(id_buscador()));
        }
        if enter {
            let posicion = self.busqueda.seleccion.unwrap_or(0);
            let indice = self.busqueda.resultados[posicion];
            self.elegir_producto(indice, cx);
            return true;
        }
        false
    }

    fn lector(&mut self, ui: &Ui, cx: &mut Contexto, cambio: bool, pegados: Vec<String>) {
        let ahora = ui.input(|i| i.time);
        if let Some(texto) = pegados.into_iter().last() {
            self.lector = Lector { pegado: Some((texto, ahora + 0.05)), ignorar_hasta: ahora + 0.25, ..Default::default() };
        } else if cambio && ahora >= self.lector.ignorar_hasta {
            let valor = self.busqueda.consulta.trim().to_string();
            self.lector.procesar_en = (!valor.is_empty()).then_some(ahora + 0.1);
            self.lector.buffer = valor;
        }
        if let Some((texto, cuando)) = self.lector.pegado.clone() {
            if ahora >= cuando {
                self.lector.pegado = None;
                self.procesar_codigo(&texto, cx);
            }
        }
        if let Some(cuando) = self.lector.procesar_en {
            if ahora >= cuando {
                let codigo = std::mem::take(&mut self.lector.buffer);
                self.lector.procesar_en = None;
                self.procesar_codigo(&codigo, cx);
            }
        }
        let espera = [self.lector.procesar_en, self.lector.pegado.as_ref().map(|p| p.1), self.cerrar_sesion_en]
            .into_iter()
            .flatten()
            .map(|t| (t - ahora).max(0.0))
            .fold(f64::INFINITY, f64::min);
        if espera.is_finite() {
            ui.ctx().request_repaint_after(Duration::from_secs_f64(espera));
        }
    }

    fn procesar_codigo(&mut self, codigo: &str, cx: &mut Contexto) {
        if let Some((indice, cantidad)) = productos::interpretar_codigo(codigo, &self.productos) {
            let producto = self.productos[indice].clone();
            self.agregar_escaneado(producto, cantidad, cx);
            self.busqueda.limpiar();
        }
    }

    fn cambio_busqueda(&mut self, cx: &mut Contexto) {
        self.busqueda.filtrar(&self.productos);
        let valor = self.busqueda.consulta.clone();
        if productos::es_codigo_directo(&valor) {
            if let Some(producto) = self.productos.iter().find(|p| p.codigo_barras.as_deref() == Some(valor.as_str())).cloned() {
                self.agregar_escaneado(producto, 1.0, cx);
                self.busqueda.limpiar();
            }
        }
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        let pantalla = ui.ctx().content_rect();
        let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(pantalla));
        hijo.set_clip_rect(pantalla);
        self.pantalla(&mut hijo, cx);
    }

    fn pantalla(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        let frame = ui.ctx().cumulative_frame_nr();
        if self.ultimo_frame.is_none_or(|anterior| frame > anterior + 1) {
            self.montar(cx);
        }
        self.ultimo_frame = Some(frame);
        self.atender_tareas(cx);
        if let Some(cuando) = self.cerrar_sesion_en {
            if ui.input(|i| i.time) >= cuando {
                self.cerrar_sesion_en = None;
                sesion::cerrar_sesion(cx);
                return;
            }
        }
        if !self.cobro.ocupado() {
            self.atajos(ui, cx);
        }

        let pantalla = ui.max_rect();
        let barra = Rect::from_min_size(pantalla.min, Vec2::new(ANCHO_BARRA, pantalla.height()));
        let atajos = vista::atajos(cx.global.negocio.as_ref());
        let caja_buscador = vista::barra_lateral(ui, barra, &atajos);
        let eligio = if self.hay_dialogo() { false } else { self.teclas_busqueda(ui, cx) };
        let pegados: Vec<String> = if ui.memory(|m| m.has_focus(id_buscador())) {
            ui.input(|i| i.events.iter().filter_map(|e| if let Event::Paste(t) = e { Some(t.clone()) } else { None }).collect())
        } else {
            Vec::new()
        };
        let antes = self.busqueda.consulta.clone();
        let respuesta_buscador = vista::en_rect(ui, caja_buscador, |ui| {
            Entrada::new(&mut self.busqueda.consulta)
                .id(id_buscador())
                .radio(ui::RADIO)
                .tam(Tam::Sm)
                .sugerencia("Buscar productos (nombre o PLU)...")
                .ui(ui)
        });
        self.buscador_enfocado = respuesta_buscador.has_focus();
        let cambio = !eligio && self.busqueda.consulta != antes;
        if cambio {
            self.cambio_busqueda(cx);
        }
        if !self.cobro.ocupado() {
            self.lector(ui, cx, cambio, pegados);
        }
        if !self.busqueda.resultados.is_empty() && !self.hay_dialogo() {
            let origen = Pos2::new(caja_buscador.left(), caja_buscador.bottom() + 4.0);
            let negocio = cx.global.negocio.clone();
            if let Some(posicion) = vista::resultados(ui.ctx(), origen, caja_buscador.width(), &self.productos, &self.busqueda.resultados, self.busqueda.seleccion, negocio.as_ref()) {
                let indice = self.busqueda.resultados[posicion];
                self.elegir_producto(indice, cx);
            }
        }

        let principal = Rect::from_min_max(Pos2::new(barra.right(), pantalla.top()), pantalla.max);
        ui.painter().rect_filled(principal, 0, paleta::GRAY_50);
        vista::en_rect(ui, principal, |ui| self.principal(ui, cx));

        self.dialogos(ui, cx);

        let hay_dialogo = self.hay_dialogo();
        if self.habia_dialogo && !hay_dialogo {
            self.enfocar = CUADROS_ENFOQUE;
        }
        self.habia_dialogo = hay_dialogo;
        if self.enfocar > 0 && !hay_dialogo {
            ui.memory_mut(|m| m.request_focus(id_buscador()));
            self.enfocar -= 1;
        }
        if hay_dialogo {
            ui.memory_mut(|m| m.surrender_focus(id_buscador()));
        }
    }

    fn principal(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if cx.global.negocio_cargando {
            let centro = ui.max_rect().center();
            let caja = Rect::from_center_size(centro - Vec2::new(0.0, 22.0), Vec2::splat(48.0));
            ui::girando(ui, caja, paleta::EMERALD_600);
            ui.painter().text(centro + Vec2::new(0.0, 26.0), egui::Align2::CENTER_CENTER, "Cargando configuración del negocio...", fuente(Tam::Base, Peso::Normal), paleta::GRAY_600);
            return;
        }
        let usuario = cx.global.usuario.clone();
        let nombre = usuario.as_ref().and_then(|u| u["nombre"].as_str()).unwrap_or_default().to_string();
        let email = usuario.as_ref().and_then(|u| u["email"].as_str()).unwrap_or_default().to_string();
        let cabecera = Cabecera {
            facturacion: cx.global.negocio_bool("facturacionHabilitada"),
            cierre: usuario.as_ref().is_some_and(|u| u["permisos"]["cierreDeCajaEnabled"].as_bool().unwrap_or(false)),
            usuario: self.menu_usuario_visible.then_some((nombre.as_str(), email.as_str())),
        };
        let ancho_pestanas = vista::ancho_pestanas(ui, &cx.global.carritos);
        let ancho_columna = ui.max_rect().width() - 48.0;
        let necesario = ancho_pestanas + 8.0 + cabecera.ancho(ui);
        let desborda = necesario > ancho_columna + 0.5;
        let mut estilo_scroll = ui.spacing().scroll;
        estilo_scroll.floating = false;
        estilo_scroll.bar_width = 8.0;
        estilo_scroll.bar_inner_margin = 4.0;
        estilo_scroll.bar_outer_margin = 4.0;
        estilo_scroll.foreground_color = false;
        estilo_scroll.fade.strength = 0.0;
        ui.spacing_mut().scroll = estilo_scroll;
        let visuales = ui.visuals_mut();
        visuales.extreme_bg_color = egui::Color32::from_gray(241);
        visuales.widgets.inactive.bg_fill = egui::Color32::from_gray(160);
        visuales.widgets.hovered.bg_fill = egui::Color32::from_gray(130);
        visuales.widgets.active.bg_fill = egui::Color32::from_gray(100);
        let alto_total = ui.max_rect().height() - if desborda { 16.0 } else { 0.0 };
        let ancho_total = if desborda { necesario + 48.0 } else { ui.max_rect().width() };
        ScrollArea::horizontal().id_salt("carrito-horizontal").auto_shrink(false).show(ui, |ui| {
            let (lienzo, _) = ui.allocate_exact_size(Vec2::new(ancho_total, alto_total), Sense::hover());
            let columna = Rect::from_min_size(lienzo.min + Vec2::new(24.0, 24.0), Vec2::new(ancho_columna, alto_total - 48.0));
            self.cabecera(ui, cx, &cabecera, columna, ancho_pestanas, desborda);
            let total = cx.global.carritos.total_con_iva(cx.global.negocio.as_ref().map(|n| n["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false)));
            let total = productos::redondear(total, 2);
            let resumen = Resumen::medir(ui, total, columna.width());
            let caja_resumen = Rect::from_min_max(Pos2::new(columna.left(), columna.bottom() - resumen.alto), columna.max);
            let caja_lista = Rect::from_min_max(Pos2::new(columna.left(), columna.top() + 56.0), Pos2::new(columna.right(), caja_resumen.top() - 16.0));
            vista::en_rect(ui, caja_lista, |ui| self.lista(ui, cx));
            let (cancelar, pagar) = resumen.mostrar(ui, caja_resumen);
            if cancelar {
                self.cancelar(cx);
            }
            if pagar {
                self.pagar(cx, true);
            }
        });
    }

    fn cabecera(&mut self, ui: &mut Ui, cx: &mut Contexto, cabecera: &Cabecera, columna: Rect, ancho_pestanas: f32, desborda: bool) {
        let centro_y = columna.top() + 20.0;
        match vista::pestanas(ui, &cx.global.carritos, Pos2::new(columna.left(), centro_y)) {
            Some(AccionPestana::Elegir(id)) => {
                cx.global.carritos.elegir(id);
                self.enfocar = CUADROS_ENFOQUE;
            }
            Some(AccionPestana::Agregar) => {
                if cx.global.carritos.agregar_pantalla() {
                    self.enfocar = CUADROS_ENFOQUE;
                } else {
                    ui::toast::error(&cx.egui, "No se pueden crear más de 4 pantallas");
                }
            }
            Some(AccionPestana::Borrar(id)) => {
                if cx.global.carritos.screens.len() <= 1 {
                    ui::toast::error(&cx.egui, "No se puede eliminar la última pantalla");
                } else {
                    self.borrar_pantalla = Some(id);
                }
            }
            None => {}
        }
        let derecha = if desborda { columna.left() + ancho_pestanas + 8.0 + cabecera.ancho(ui) } else { columna.right() };
        match cabecera.mostrar(ui, derecha, centro_y) {
            Some(AccionCabecera::Ordenes) => self.pedidos.abrir(),
            Some(AccionCabecera::CuentaCorriente) => cx.global.ir(Ruta::Facturas),
            Some(AccionCabecera::Cierre) => self.cierre.abrir(),
            Some(AccionCabecera::Impresion) => self.impresion.abrir(),
            Some(AccionCabecera::Balanza) => self.balanza.abrir(),
            Some(AccionCabecera::CerrarSesion) => {
                sesion::cerrar_sesion(cx);
                ui::toast::exito(&cx.egui, "Sesión cerrada exitosamente");
            }
            None => {}
        }
    }

    fn lista(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        let negocio = cx.global.negocio.clone();
        let items: Vec<Producto> = cx.global.carritos.items().iter().rev().cloned().collect();
        let permite_borrar = !cancelacion_controlada(cx);
        let mut quitar = None;
        ScrollArea::vertical().id_salt(("carrito-items", cx.global.carritos.active_screen)).auto_shrink(false).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    ui.add_space(8.0);
                }
                if vista::item_carrito(ui, item, negocio.as_ref(), permite_borrar) {
                    quitar = Some(item.cart_id.clone());
                }
            }
            });
        });
        if let Some(cart_id) = quitar {
            cx.global.carritos.quitar(&cart_id);
        }
    }

    fn dialogos(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if cx.global.negocio_cargando {
            return;
        }
        match self.agregar.mostrar(ui, cx) {
            Salida::Agregar(item) => {
                cx.global.carritos.agregar(item);
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Cerrar | Salida::Nada => {}
        }
        match self.escanear.mostrar(ui, cx) {
            Salida::Agregar(item) => {
                cx.global.carritos.agregar(item);
                self.busqueda.limpiar();
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Cerrar => {
                self.busqueda.limpiar();
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Nada => {}
        }
        if let Some(id) = self.borrar_pantalla {
            match dialogos::confirmar_borrado(ui) {
                Some(true) => {
                    self.borrar_pantalla = None;
                    if cx.global.carritos.borrar_pantalla(id) {
                        ui::toast::exito(&cx.egui, "Pantalla eliminada correctamente");
                        self.enfocar = CUADROS_ENFOQUE;
                    }
                }
                Some(false) => self.borrar_pantalla = None,
                None => {}
            }
        }
        if self.cobro.mostrar(ui, cx).is_some() {
            self.enfocar = CUADROS_ENFOQUE;
        }
        match self.factura.mostrar(ui, cx) {
            Some(ResultadoFactura::Creada(_)) => {
                cx.global.carritos.vaciar();
                self.enfocar = CUADROS_ENFOQUE;
            }
            Some(ResultadoFactura::Cerrada) => self.enfocar = CUADROS_ENFOQUE,
            None => {}
        }
        if self.cierre.mostrar(ui, cx).is_some() {
            self.enfocar = CUADROS_ENFOQUE;
        }
        self.pedidos.mostrar(ui, cx);
        self.impresion.mostrar(ui, cx);
        self.balanza.mostrar(ui, cx);
    }
}

fn cancelacion_controlada(cx: &Contexto) -> bool {
    cx.global.negocio.as_ref().is_some_and(|n| n["cancelacionHabilitada"] == Value::Bool(true))
}

fn sincronizar_logo(servicios: &crate::app::Servicios) {
    let Ok((estado, bytes)) = servicios.api.bytes("/api/business/logo-comprobante") else {
        return;
    };
    if estado == 404 {
        let _ = crate::impresion::guardar_logo_negocio(servicios, None);
    } else if (200..300).contains(&estado) && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        let _ = crate::impresion::guardar_logo_negocio(servicios, Some(&bytes));
    }
}
