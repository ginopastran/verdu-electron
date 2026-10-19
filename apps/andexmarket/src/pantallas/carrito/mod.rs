mod dialogos;
mod diseno;
mod fotos;
pub mod productos;
mod vista;

use std::time::Duration;

use eframe::egui::{self, Event, Id, Key, Modifiers, Pos2, Rect, Ui, Vec2};
use nucleo::busqueda::{self, Resultado};
use nucleo::carrito::Producto;
use serde_json::Value;

use self::dialogos::{DialogoAgregar, DialogoEscanear, DialogoMonto, Salida};
use self::fotos::Fotos;
use self::productos::{FilaLista, ProductoDisponible};
use self::vista::{AccionCabecera, AccionFila, AccionIndicador, AccionPanel, AccionPestana, Cabecera, Indicadores, Panel};
use super::cierre::DialogoCierre;
use super::cobro::{AccionCobro, Cobro};
use super::configuracion::balanza::DialogoBalanza;
use super::configuracion::impresion::DialogoImpresion;
use super::facturas::formulario::{FormularioFactura, ProductoInicial, ResultadoFactura};
use super::pedidos::DialogoPedidos;
use super::sesion;
use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::marca::{color, fuente, tam};
use crate::ui::v2::{self, toast};
use crate::ui::Peso;

pub const ID_BUSCADOR: &str = "buscador-productos";
const CUADROS_ENFOQUE: u8 = 2;
const DURACION_DESTELLO: f64 = 1.6;

pub fn id_buscador() -> Id {
    Id::new(ID_BUSCADOR)
}

#[derive(Default)]
struct Busqueda {
    consulta: String,
    resultados: Vec<Resultado>,
    otras_medidas: Vec<Resultado>,
    seleccion: Option<usize>,
    mostrar_seleccion: bool,
    no_encontrado: bool,
}

impl Busqueda {
    fn filtrar(&mut self, productos: &[ProductoDisponible]) {
        let encontrados = busqueda::buscar(productos, &self.consulta);
        self.resultados = encontrados.resultados;
        self.otras_medidas = encontrados.otras_medidas;
        self.seleccion = (!self.resultados.is_empty()).then_some(0);
        self.mostrar_seleccion = true;
        self.no_encontrado = false;
    }

    fn visibles(&self) -> &[Resultado] {
        if self.resultados.is_empty() { &self.otras_medidas } else { &self.resultados }
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

struct Nuevo {
    cart_id: String,
    desde: f64,
    desplazar: bool,
}

#[derive(Default)]
pub struct Carrito {
    pub cobro: Cobro,
    pub factura: FormularioFactura,
    pub cierre: DialogoCierre,
    pub pedidos: DialogoPedidos,
    pub impresion: DialogoImpresion,
    pub balanza: DialogoBalanza,
    productos: Vec<ProductoDisponible>,
    productos_precargados: bool,
    fotos: Fotos,
    tarea_productos: Option<Tarea<Result<Vec<ProductoDisponible>, String>>>,
    tarea_logo: Option<Tarea<Result<(), String>>>,
    tarea_escaneo: Option<Tarea<ResultadoEscaneo>>,
    ultimo_escaneo: Option<(String, f64)>,
    busqueda: Busqueda,
    lector: Lector,
    agregar: DialogoAgregar,
    escanear: DialogoEscanear,
    monto: DialogoMonto,
    borrar_pantalla: Option<usize>,
    cerrar_sesion_en: Option<f64>,
    ultimo_frame: Option<u64>,
    habia_dialogo: bool,
    enfocar: u8,
    buscador_enfocado: bool,
    nuevo: Option<Nuevo>,
}

impl Carrito {
    pub fn precargar_productos(&mut self, productos: Vec<ProductoDisponible>) {
        self.productos = productos;
        self.productos_precargados = true;
    }

    pub fn productos(&self) -> &[ProductoDisponible] {
        &self.productos
    }

    pub fn fijar_productos(&mut self, lista: Vec<ProductoDisponible>) {
        self.productos = lista;
        if !self.busqueda.consulta.is_empty() {
            self.busqueda.filtrar(&self.productos);
        }
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
        if producto.precio_libre {
            self.monto.abrir(producto);
            return;
        }
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
        let item = producto.item(cantidad, producto.unidad_o_defecto(), producto.precio_venta(chrono::Utc::now()), None, &etiqueta);
        self.agregar_item(item, cx);
        self.busqueda.limpiar();
        self.enfocar = CUADROS_ENFOQUE;
    }

    fn agregar_item(&mut self, item: Producto, cx: &mut Contexto) {
        self.nuevo = Some(Nuevo { cart_id: item.cart_id.clone(), desde: cx.egui.input(|i| i.time), desplazar: true });
        cx.global.carritos.agregar(item);
    }

    fn anunciar_cierre_sesion(&mut self, ui: &Ui, cx: &Contexto) {
        toast::exito(&cx.egui, "Cerrando sesión...");
        self.cerrar_sesion_en = Some(ui.input(|i| i.time) + 1.0);
    }

    fn montar(&mut self, cx: &mut Contexto) {
        self.busqueda.limpiar();
        self.lector = Lector::default();
        self.agregar = DialogoAgregar::default();
        self.agregar.peso_manual = cx.global.usuario.as_ref().is_some_and(|u| u["permisos"]["pesoManualEnabled"] == Value::Bool(true));
        self.escanear = DialogoEscanear::default();
        self.monto = DialogoMonto::default();
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
                Ok(lista) => self.fijar_productos(lista),
                Err(mensaje) => toast::error(&cx.egui, mensaje),
            }
        }
        let _ = tareas::esperar(&mut self.tarea_logo);
        self.fotos.recibir(&cx.egui);
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
            toast::con_detalle(&cx.egui, toast::Tipo::Error, "No hay productos en el carrito", detalle);
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
            toast::con_detalle(&cx.egui, toast::Tipo::Error, "Funcionalidad no disponible", "La facturación no está habilitada para este negocio");
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
            .map(|item| ProductoInicial {
                id: item.id,
                nombre: item.name.clone(),
                precio: item.price_per_unit,
                cantidad: item.quantity,
                subtotal: item.subtotal,
                precio_libre: item.precio_libre,
            })
            .collect();
        self.factura.abrir(iniciales);
    }

    fn atajos(&mut self, ui: &Ui, cx: &mut Contexto) {
        let teclas = [Key::F1, Key::F2, Key::F3, Key::F4, Key::F6, Key::F7, Key::F8];
        let pulsadas: Vec<Key> = ui.input_mut(|i| teclas.into_iter().filter(|k| i.consume_key(Modifiers::NONE, *k)).collect());
        for tecla in pulsadas {
            match tecla {
                Key::F4 => self.anunciar_cierre_sesion(ui, cx),
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

    pub fn hay_dialogo(&self) -> bool {
        self.agregar.abierto()
            || self.escanear.abierto()
            || self.monto.abierto()
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
        if producto.precio_libre {
            self.monto.abrir(producto);
        } else {
            self.agregar.abrir(producto, cx);
        }
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
        if self.busqueda.visibles().is_empty() {
            return false;
        }
        let total = self.busqueda.visibles().len();
        self.busqueda.mostrar_seleccion |= abajo || arriba;
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
        if let (true, Some(posicion)) = (enter, self.busqueda.seleccion) {
            let indice = self.busqueda.visibles()[posicion].indice;
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
        } else if codigo.len() >= 8 && codigo.bytes().all(|b| b.is_ascii_digit()) && codigo == self.busqueda.consulta.trim() {
            self.busqueda.no_encontrado = true;
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
        ui.painter().rect_filled(pantalla, 0, color::FONDO_APP);
        let usuario = cx.global.usuario.clone();
        let nombre = usuario.as_ref().and_then(|u| u["nombre"].as_str()).unwrap_or_default().to_string();
        let email = usuario.as_ref().and_then(|u| u["email"].as_str()).unwrap_or_default().to_string();
        let reloj = diseno::reloj(chrono::Local::now().naive_local());
        ui.ctx().request_repaint_after(Duration::from_secs(30));
        let cabecera = Cabecera {
            facturacion: cx.global.negocio_bool("facturacionHabilitada"),
            cierre: usuario.as_ref().is_some_and(|u| u["permisos"]["cierreDeCajaEnabled"].as_bool().unwrap_or(false)),
            nombre: &nombre,
            email: &email,
            reloj: &reloj,
        };
        let disposicion = cabecera.disponer(ui, pantalla);
        Cabecera::pintar_fondo(ui, &disposicion);

        let eligio = if self.hay_dialogo() || self.cobro.venta_cobrada() { false } else { self.teclas_busqueda(ui, cx) };
        let pegados: Vec<String> = if ui.memory(|m| m.has_focus(id_buscador())) {
            ui.input(|i| i.events.iter().filter_map(|e| if let Event::Paste(t) = e { Some(t.clone()) } else { None }).collect())
        } else {
            Vec::new()
        };
        let antes = self.busqueda.consulta.clone();
        let buscador = vista::buscador(ui, disposicion.buscador, &mut self.busqueda.consulta, id_buscador());
        if buscador.limpiar {
            self.busqueda.limpiar();
            ui.memory_mut(|m| m.request_focus(id_buscador()));
        }
        self.buscador_enfocado = buscador.campo.has_focus();
        let cambio = !eligio && self.busqueda.consulta != antes;
        if cambio {
            self.cambio_busqueda(cx);
        }
        if !self.cobro.ocupado() {
            self.lector(ui, cx, cambio, pegados);
        }
        let consulta = self.busqueda.consulta.trim();
        let hay_consulta = !consulta.is_empty() && !(consulta.len() >= 8 && consulta.bytes().all(|b| b.is_ascii_digit()));
        let visible = (hay_consulta || !self.busqueda.visibles().is_empty() || self.busqueda.no_encontrado) && !self.hay_dialogo();
        let negocio = cx.global.negocio.clone();
        let datos = vista::Resultados {
            productos: &self.productos,
            resultados: self.busqueda.visibles(),
            otras_medidas: self.busqueda.resultados.is_empty() && !self.busqueda.otras_medidas.is_empty(),
            no_encontrado: self.busqueda.no_encontrado,
            seleccion: self.busqueda.seleccion,
            mostrar_seleccion: self.busqueda.mostrar_seleccion,
            consulta,
            negocio: negocio.as_ref(),
            fotos: &self.fotos,
            ahora: chrono::Utc::now(),
        };
        let (elegido, sin_foto) = vista::resultados(ui.ctx(), &buscador.caja, visible, &datos);
        for indice in sin_foto {
            self.fotos.pedir(cx, &self.productos[indice]);
        }
        if let Some(posicion) = elegido {
            let indice = self.busqueda.visibles()[posicion].indice;
            self.elegir_producto(indice, cx);
        }
        if visible {
            self.busqueda.mostrar_seleccion = false;
        }

        match cabecera.acciones(ui, &disposicion) {
            Some(AccionCabecera::Ordenes) => self.pedidos.abrir(),
            Some(AccionCabecera::CuentaCorriente) => cx.global.ir(Ruta::Facturas),
            Some(AccionCabecera::Cierre) => self.cierre.abrir(),
            Some(AccionCabecera::Impresion) => self.impresion.abrir(),
            Some(AccionCabecera::Balanza) => self.balanza.abrir(),
            Some(AccionCabecera::CerrarSesion) => {
                sesion::cerrar_sesion(cx);
                toast::exito(&cx.egui, "Sesión cerrada exitosamente");
            }
            None => {}
        }

        let cuerpo = Rect::from_min_max(Pos2::new(pantalla.left(), disposicion.barra.bottom()), pantalla.max);
        self.cuerpo(ui, cx, cuerpo);

        self.dialogos(ui, cx);

        let hay_dialogo = self.hay_dialogo();
        if self.habia_dialogo && !hay_dialogo {
            self.enfocar = CUADROS_ENFOQUE;
        }
        self.habia_dialogo = hay_dialogo;
        if self.enfocar > 0 && !hay_dialogo {
            ui.memory_mut(|m| m.request_focus(id_buscador()));
            self.enfocar -= 1;
            ui.ctx().request_repaint();
        }
        if hay_dialogo {
            ui.memory_mut(|m| m.surrender_focus(id_buscador()));
        }
    }

    fn cuerpo(&mut self, ui: &mut Ui, cx: &mut Contexto, rect: Rect) {
        if cx.global.negocio_cargando {
            let centro = rect.center();
            v2::girando(ui, Rect::from_center_size(centro - Vec2::new(0.0, 22.0), Vec2::splat(48.0)), color::VERDE);
            ui.painter().text(centro + Vec2::new(0.0, 26.0), egui::Align2::CENTER_CENTER, "Cargando configuración del negocio...", fuente(tam::CUERPO, Peso::Normal), color::TEXTO_APAGADO);
            return;
        }
        let interior = Rect::from_min_max(Pos2::new(rect.left() + 20.0, rect.top() + 18.0), Pos2::new(rect.right() - 20.0, rect.bottom() - 20.0));
        let izquierda = Rect::from_min_max(interior.min, Pos2::new(interior.right() - vista::ANCHO_PANEL - 20.0, interior.bottom()));
        let derecha = Rect::from_min_max(Pos2::new(interior.right() - vista::ANCHO_PANEL, interior.top()), interior.max);

        let indicadores = Indicadores {
            peso: vista::kilos(productos::gramos_de_peso(&cx.global.peso) / 1000.0),
            balanza_conectada: cx.global.estado_balanza["conectada"] == Value::Bool(true),
            impresora: cx.servicios.config.get("impresoraWindows").as_str().map_or_else(|| "Impresora predeterminada".to_string(), |n| format!("Impresora {n}")),
        };
        let fila = vista::fila_pestanas(ui, izquierda, &cx.global.carritos, &indicadores);
        match fila.pestana {
            Some(AccionPestana::Elegir(indice)) => {
                cx.global.carritos.elegir(indice);
                self.enfocar = CUADROS_ENFOQUE;
            }
            Some(AccionPestana::Agregar) => {
                if cx.global.carritos.agregar_pantalla() {
                    self.enfocar = CUADROS_ENFOQUE;
                } else {
                    toast::error(&cx.egui, "No se pueden crear más de 4 pantallas");
                }
            }
            Some(AccionPestana::Borrar(indice)) => {
                if cx.global.carritos.screens.len() <= 1 {
                    toast::error(&cx.egui, "No se puede eliminar la última pantalla");
                } else {
                    self.borrar_pantalla = Some(indice);
                }
            }
            None => {}
        }
        match fila.indicador {
            Some(AccionIndicador::Balanza) => self.balanza.abrir(),
            Some(AccionIndicador::Impresora) => self.impresion.abrir(),
            None => {}
        }

        let caja_tabla = Rect::from_min_max(Pos2::new(izquierda.left(), izquierda.top() + fila.alto + 12.0), izquierda.max);
        self.lista(ui, cx, caja_tabla);

        let total = cx.global.carritos.total_con_iva(cx.global.negocio.as_ref().map(|n| n["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false)));
        let datos = Panel {
            total: productos::redondear(total, 2),
            productos: Self::items_actuales(cx),
            orden: cx.global.carritos.active_screen + 1,
            afip: Self::con_afip(cx),
            facturacion: cx.global.negocio_bool("facturacionHabilitada"),
        };
        match vista::panel(ui, derecha, &datos) {
            Some(AccionPanel::Cobrar) => self.pagar(cx, true),
            Some(AccionPanel::CobrarSinFactura) => self.pagar(cx, false),
            Some(AccionPanel::Descuento) => self.pagar_con_descuento(cx, true),
            Some(AccionPanel::DescuentoSinFactura) => self.pagar_con_descuento(cx, false),
            Some(AccionPanel::CuentaCorriente) => self.cuenta_corriente(cx),
            Some(AccionPanel::Cancelar) => self.cancelar(cx),
            Some(AccionPanel::CerrarSesion) => self.anunciar_cierre_sesion(ui, cx),
            None => {}
        }
    }

    fn lista(&mut self, ui: &mut Ui, cx: &mut Contexto, caja: Rect) {
        let negocio = cx.global.negocio.clone();
        let activa = cx.global.carritos.active_screen;
        let filas: Vec<vista::Fila> = cx.global.carritos.items().iter().map(|item| vista::fila_de_item(item, negocio.as_ref())).collect();
        if filas.is_empty() {
            vista::orden_vacia(ui, caja, activa + 1);
            return;
        }
        let ahora = ui.input(|i| i.time);
        let desplazar = self.nuevo.as_mut().is_some_and(|n| std::mem::take(&mut n.desplazar));
        let destello = self.nuevo.as_ref().map(|n| {
            let avance = ((ahora - n.desde) / DURACION_DESTELLO) as f32;
            if avance < 1.0 {
                ui.ctx().request_repaint();
            }
            vista::Destello { cart_id: &n.cart_id, avance: avance.clamp(0.0, 1.0), desplazar }
        });
        match vista::tabla(ui, caja, &filas, !cancelacion_controlada(cx), destello.as_ref(), Id::new(("carrito-items", activa))) {
            Some(AccionFila::Quitar(cart_id)) => cx.global.carritos.quitar(&cart_id),
            Some(AccionFila::Editar(cart_id)) => {
                if let Some(item) = cx.global.carritos.items().iter().find(|item| item.cart_id == cart_id) {
                    self.monto.editar(item);
                }
            }
            None => {}
        }
    }

    fn dialogos(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if cx.global.negocio_cargando {
            return;
        }
        match self.agregar.mostrar(ui, cx) {
            Salida::Agregar(item) => {
                self.agregar_item(item, cx);
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Cerrar | Salida::Nada => {}
        }
        match self.escanear.mostrar(ui, cx) {
            Salida::Agregar(item) => {
                self.agregar_item(item, cx);
                self.busqueda.limpiar();
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Cerrar => {
                self.busqueda.limpiar();
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Nada => {}
        }
        match self.monto.mostrar(ui, cx) {
            Salida::Agregar(item) => {
                self.agregar_item(item, cx);
                self.busqueda.limpiar();
                self.enfocar = CUADROS_ENFOQUE;
            }
            Salida::Cerrar => self.enfocar = CUADROS_ENFOQUE,
            Salida::Nada => {}
        }
        if let Some(indice) = self.borrar_pantalla {
            let productos = cx.global.carritos.screens.get(indice).map_or(0, |p| p.items.len());
            match dialogos::confirmar_borrado(ui, indice + 1, productos) {
                Some(true) => {
                    self.borrar_pantalla = None;
                    if cx.global.carritos.borrar_pantalla(indice) {
                        toast::exito(&cx.egui, "Pantalla eliminada correctamente");
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

pub fn sincronizar_logo(servicios: &crate::app::Servicios) -> Result<(), String> {
    let (estado, bytes) = servicios.api.bytes("/api/business/logo-comprobante").map_err(|e| e.mensaje)?;
    if estado == 404 {
        crate::impresion::guardar_logo_negocio(servicios, None)?;
    } else if (200..300).contains(&estado) && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        crate::impresion::guardar_logo_negocio(servicios, Some(&bytes))?;
    } else if !(200..300).contains(&estado) {
        return Err(format!("No se pudo descargar el logo del negocio (HTTP {estado})"));
    }
    Ok(())
}
