use eframe::egui::{self, Align2, Color32, Context, Id, Key, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::facturas::{self as logica};
use nucleo::formato;
use serde_json::Value;

use super::comun::{self, Boton, Relleno};
use super::{id_texto, numero, texto};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, Entrada, Peso, Tam};

const CLIENTES_POR_PAGINA: u32 = 10;
const PRODUCTOS_POR_PAGINA: usize = 6;

fn mensaje_error(error: &str) -> String {
    if error.contains("<!doctype") {
        "El servidor no está respondiendo correctamente. Verifique que esté ejecutándose en http://localhost:3000".into()
    } else if error.contains("error sending request") || error.contains("connect") {
        "No se puede conectar al servidor. Verifique que esté ejecutándose".into()
    } else {
        error.to_string()
    }
}

fn pedir_json(cx: &Contexto, ruta: String) -> Tarea<Result<Value, String>> {
    let api = cx.servicios.api.clone();
    Tarea::lanzar(&cx.egui, move || {
        let respuesta = api.pedir("GET", &ruta, None, &[]).map_err(|e| e.mensaje)?;
        if !respuesta.ok() {
            return Err(format!("Error {}: {}", respuesta.estado, razon(respuesta.estado)));
        }
        match respuesta.cuerpo {
            Value::String(texto) => Err(format!("Unexpected token '<', \"{}\"... is not valid JSON", texto.chars().take(10).collect::<String>())),
            cuerpo => Ok(cuerpo),
        }
    })
}

fn razon(estado: u16) -> &'static str {
    match estado {
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "",
    }
}

fn alerta_error(ui: &mut Ui, error: &str) {
    let ancho = ui.available_width();
    let b = comun::bloque(ui, error, Tam::Sm, Peso::Normal, paleta::RED_700, ancho - 50.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, b.alto() + 26.0), Sense::hover());
    ui.painter().rect(rect, 6, paleta::RED_50, Stroke::new(1.0, paleta::RED_200), StrokeKind::Inside);
    ui::pintar_icono_en(ui, Rect::from_min_size(rect.min + Vec2::new(13.0, 15.0), Vec2::splat(16.0)), "alert-circle", paleta::RED_500);
    b.pintar(ui, rect.min + Vec2::new(37.0, 13.0), paleta::RED_700);
    ui.add_space(16.0);
}

fn vacio(ui: &mut Ui, icono: &str, mensaje: &str, limpiar: bool, pista: bool) -> bool {
    let ancho = ui.available_width();
    let mut alto = 32.0 + 48.0 + 8.0 + 24.0 + 32.0;
    if limpiar {
        alto += 8.0 + 36.0 + if pista { 8.0 + 16.0 + 8.0 } else { 0.0 };
    }
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    let centro = rect.center().x;
    let mut y = rect.top() + 32.0;
    ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(centro - 24.0, y), Vec2::splat(48.0)), icono, color::APAGADO_TEXTO);
    y += 56.0;
    comun::texto_en(ui, egui::pos2(centro, y + 12.0), Align2::CENTER_CENTER, mensaje, Tam::Base, Peso::Normal, color::APAGADO_TEXTO);
    y += 24.0;
    let mut clic = false;
    if limpiar {
        y += if pista { 16.0 } else { 8.0 };
        let boton = Boton::contorno("Limpiar búsqueda").chico();
        let w = boton.ancho_natural(ui);
        clic = boton.en(ui, Rect::from_min_size(egui::pos2(centro - w / 2.0, y), Vec2::new(w, 36.0)), "limpiar").clicked();
        if pista {
            comun::texto_en(ui, egui::pos2(centro, y + 36.0 + 8.0 + 8.0), Align2::CENTER_CENTER, "Intenta con términos más generales", Tam::Xs, Peso::Normal, color::APAGADO_TEXTO);
        }
    }
    clic
}

fn cargando(ui: &mut Ui, texto: &str) {
    comun::cargando_centro(ui, 88.0, texto, color::APAGADO_TEXTO, color::APAGADO_TEXTO, Tam::Base);
}

fn resumen(ui: &mut Ui, izquierda: &str, derecha: Option<String>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
    comun::texto_en(ui, egui::pos2(rect.left(), rect.center().y), Align2::LEFT_CENTER, izquierda, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
    if let Some(derecha) = derecha {
        comun::texto_en(ui, egui::pos2(rect.right(), rect.center().y), Align2::RIGHT_CENTER, &derecha, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
    }
}

fn paginacion(ui: &mut Ui, id: &str, pagina: u32, total: u32, anterior: bool, siguiente: bool) -> Option<u32> {
    ui.add_space(16.0);
    comun::borde_superior(ui);
    ui.add_space(8.0);
    let paginas = logica::paginas_visibles(pagina, total);
    comun::fila_paginacion(ui, id, pagina, &paginas, anterior, siguiente, 8.0)
}

fn pie(ui: &mut Ui, habilitado: bool) -> bool {
    ui.add_space(16.0);
    comun::borde_superior(ui);
    ui.add_space(16.0);
    ui::pie_dialogo(ui, |ui| Boton::contorno("Cancelar").habilitado(habilitado).ui(ui).clicked())
}

fn alto_lista(ctx: &Context, fijo: f32) -> f32 {
    (ctx.content_rect().height() * 0.8 - fijo).max(64.0)
}

#[derive(Default)]
pub struct SelectorCliente {
    abierto: bool,
    busqueda: String,
    pedida: Option<(u32, String)>,
    cargando: bool,
    error: Option<String>,
    clientes: Vec<Value>,
    pagina: u32,
    total: usize,
    total_paginas: u32,
    hay_siguiente: bool,
    hay_anterior: bool,
    tarea: Option<Tarea<Result<Value, String>>>,
}

impl SelectorCliente {
    pub fn abrir(&mut self) {
        self.abierto = true;
        self.busqueda.clear();
        self.error = None;
        self.pagina = 1;
        self.pedida = None;
    }

    fn pedir(&mut self, cx: &Contexto) {
        let clave = (self.pagina, self.busqueda.clone());
        if self.pedida.as_ref() == Some(&clave) {
            return;
        }
        self.pedida = Some(clave);
        self.cargando = true;
        self.error = None;
        let mut ruta = format!("/api/clientes?page={}&limit={CLIENTES_POR_PAGINA}", self.pagina);
        if !self.busqueda.is_empty() {
            ruta.push_str(&format!("&search={}", nucleo::api::codificar(&self.busqueda)));
        }
        self.tarea = Some(pedir_json(cx, ruta));
    }

    fn recibir(&mut self, cx: &Contexto) {
        let Some(resultado) = esperar(&mut self.tarea) else { return };
        self.cargando = false;
        match resultado {
            Ok(datos) => {
                self.clientes = datos["clientes"].as_array().cloned().unwrap_or_default();
                let p = &datos["pagination"];
                let campo = |clave: &str, defecto: f64| p[clave].as_f64().unwrap_or(defecto);
                self.pagina = campo("page", 1.0) as u32;
                self.total = campo("total", 0.0) as usize;
                self.total_paginas = campo("totalPages", 0.0) as u32;
                self.hay_siguiente = p["hasNext"].as_bool().unwrap_or(false);
                self.hay_anterior = p["hasPrev"].as_bool().unwrap_or(false);
                self.pedida = Some((self.pagina, self.busqueda.clone()));
            }
            Err(error) => {
                self.error = Some(mensaje_error(&error));
                ui::v2::toast::error(&cx.egui, "Error al cargar la lista de clientes");
            }
        }
    }

    pub fn mostrar(&mut self, ctx: &Context, cx: &Contexto, seleccionado: Option<String>) -> Option<Value> {
        if !self.abierto {
            return None;
        }
        self.recibir(cx);
        self.pedir(cx);
        let mut elegido = None;
        let mut cerrar = false;
        let fijo = 277.0 + if self.error.is_some() { 64.0 } else { 0.0 } + if self.total_paginas > 1 && !self.cargando { 61.0 } else { 0.0 };
        let alto = alto_lista(ctx, fijo);
        let r = ui::Dialogo::new("facturas-selector-cliente").ancho(672.0).mostrar(ctx, |ui| {
            super::modal::estilo_desplazamiento(ui);
            comun::encabezado_dialogo(ui, "user", "Seleccionar Cliente", "Busque y seleccione un cliente para la factura", 12.0);
            ui.add_space(16.0);
            let anterior = self.busqueda.clone();
            ui.scope(|ui| {
                Entrada::new(&mut self.busqueda)
                    .tam(Tam::Sm)
                    .icono("search")
                    .sugerencia("Buscar por nombre, razón social, CUIT...")
                    .habilitado(!self.cargando)
                    .id(Id::new("facturas-buscar-cliente"))
                    .ui(ui)
            });
            if self.busqueda != anterior && !self.busqueda.is_empty() {
                self.pagina = 1;
            }
            ui.add_space(16.0);
            let derecha = (self.total_paginas > 1).then(|| format!("Página {} de {}", self.pagina, self.total_paginas));
            resumen(ui, &logica::resumen_clientes(self.cargando, !self.busqueda.is_empty(), self.total), derecha);
            ui.add_space(16.0);
            if let Some(error) = &self.error {
                alerta_error(ui, error);
            }
            comun::desplazable(ui, Id::new("facturas-lista-clientes"), alto, |ui| {
                ui.set_width(ui.available_width());
                if self.cargando {
                    cargando(ui, "Cargando clientes...");
                } else if self.clientes.is_empty() {
                    let mensaje = if self.busqueda.is_empty() {
                        "No hay clientes disponibles"
                    } else {
                        "No se encontraron clientes con ese criterio de búsqueda"
                    };
                    if vacio(ui, "user", mensaje, !self.busqueda.is_empty(), false) {
                        self.busqueda.clear();
                    }
                } else {
                    for (i, cliente) in self.clientes.iter().enumerate() {
                        if i > 0 {
                            ui.add_space(8.0);
                        }
                        let activo = seleccionado.as_deref() == Some(id_texto(&cliente["id"]).as_str());
                        if boton_cliente(ui, cliente, activo, i).clicked() {
                            elegido = Some(cliente.clone());
                        }
                    }
                }
            });
            if self.total_paginas > 1 && !self.cargando {
                if let Some(destino) = paginacion(ui, "clientes", self.pagina, self.total_paginas, self.hay_anterior, self.hay_siguiente) {
                    self.pagina = destino.clamp(1, self.total_paginas);
                }
            }
            cerrar = pie(ui, !self.cargando);
        });
        if elegido.is_some() || cerrar || r.cerrar {
            self.abierto = false;
        }
        elegido
    }
}

fn boton_cliente(ui: &mut Ui, cliente: &Value, activo: bool, indice: usize) -> egui::Response {
    let ancho = ui.available_width();
    let cuit = texto(cliente, "cuit");
    let alto = 2.0 + 32.0 + 20.0 + 4.0 + if cuit.is_empty() { 20.0 } else { 44.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
    let boton = if activo { Boton::primario("") } else { Boton::contorno("").encima(Relleno::Liso(color::ACENTO)) };
    let respuesta = boton.borde(Some(color::BORDE_ENTRADA)).en(ui, rect, ("cliente", indice));
    let x = rect.left() + 17.0;
    let mut y = rect.top() + 17.0;
    let principal = if activo { color::PRIMARIO_TEXTO } else { color::TEXTO };
    comun::texto_en(ui, egui::pos2(x, y + 10.0), Align2::LEFT_CENTER, &logica::nombre_cliente(cliente), Tam::Sm, Peso::Medio, principal);
    y += 24.0;
    let mut dato = |etiqueta: &str, valor: &str| {
        let r = comun::texto_en(ui, egui::pos2(x, y + 10.0), Align2::LEFT_CENTER, etiqueta, Tam::Sm, Peso::Medio, color::APAGADO_TEXTO);
        comun::texto_en(ui, egui::pos2(r.right() + 4.0, y + 10.0), Align2::LEFT_CENTER, valor, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
        y += 24.0;
    };
    if !cuit.is_empty() {
        dato("CUIT:", cuit);
    }
    dato("Condición Fiscal:", texto(cliente, "condicionFiscal"));
    if activo {
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(rect.right() - 25.0, rect.center().y), Vec2::splat(16.0)), "user", color::PRIMARIO);
    }
    respuesta
}

#[derive(Default)]
pub struct SelectorProducto {
    abierto: bool,
    busqueda: String,
    cargando: bool,
    error: Option<String>,
    productos: Vec<Value>,
    pagina: usize,
    tarea: Option<Tarea<Result<Value, String>>>,
    enfocar: bool,
}

pub struct ListaActiva<'a> {
    pub nombre: Option<String>,
    pub precios: &'a std::collections::HashMap<i64, f64>,
}

impl SelectorProducto {
    pub fn abrir(&mut self, cx: &Contexto) {
        self.abierto = true;
        self.busqueda.clear();
        self.error = None;
        self.pagina = 1;
        self.cargando = true;
        self.enfocar = true;
        self.tarea = Some(pedir_json(cx, "/api/productos?limit=1000".into()));
    }

    pub fn mostrar(&mut self, ctx: &Context, cx: &Contexto, excluidos: &[i64], lista: ListaActiva, iva_en_precios: Option<bool>) -> Option<Value> {
        if !self.abierto {
            return None;
        }
        if let Some(resultado) = esperar(&mut self.tarea) {
            self.cargando = false;
            match resultado {
                Ok(datos) => self.productos = datos["productos"].as_array().cloned().unwrap_or_default(),
                Err(error) => {
                    self.error = Some(mensaje_error(&error));
                    ui::v2::toast::error(&cx.egui, "Error al cargar la lista de productos");
                }
            }
        }
        let filtrados = logica::filtrar_productos(&self.productos, &self.busqueda, excluidos);
        let total_paginas = filtrados.len().div_ceil(PRODUCTOS_POR_PAGINA) as u32;
        let inicio = (self.pagina.saturating_sub(1)) * PRODUCTOS_POR_PAGINA;
        let pagina: Vec<Value> = filtrados.iter().skip(inicio).take(PRODUCTOS_POR_PAGINA).cloned().collect();
        let mut elegido = None;
        let mut cerrar = false;
        let fijo = 277.0 + if self.error.is_some() { 64.0 } else { 0.0 } + if total_paginas > 1 && !self.cargando { 61.0 } else { 0.0 };
        let alto = alto_lista(ctx, fijo);
        let r = ui::Dialogo::new("facturas-selector-producto").ancho(672.0).mostrar(ctx, |ui| {
            super::modal::estilo_desplazamiento(ui);
            comun::encabezado_dialogo(ui, "package", "Seleccionar Producto", "Busque y seleccione productos para agregar a la factura", 12.0);
            ui.add_space(16.0);
            let anterior = self.busqueda.clone();
            let id = Id::new("facturas-buscar-producto");
            let respuesta = ui
                .scope(|ui| {
                    Entrada::new(&mut self.busqueda)
                        .tam(Tam::Sm)
                        .icono("search")
                        .sugerencia("Buscar producto (nombre o PLU)...")
                        .habilitado(!self.cargando)
                        .id(id)
                        .ui(ui)
                })
                .inner;
            if self.enfocar {
                respuesta.request_focus();
                self.enfocar = false;
            }
            if respuesta.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
                cerrar = true;
            }
            if self.busqueda != anterior {
                self.pagina = 1;
            }
            ui.add_space(16.0);
            let derecha = (total_paginas > 1).then(|| format!("Página {} de {}", self.pagina, total_paginas));
            resumen(ui, &logica::resumen_productos(self.cargando, !self.busqueda.is_empty(), filtrados.len()), derecha);
            ui.add_space(16.0);
            if let Some(error) = &self.error {
                alerta_error(ui, error);
            }
            comun::desplazable(ui, Id::new("facturas-lista-productos"), alto, |ui| {
                ui.set_width(ui.available_width());
                if self.cargando {
                    cargando(ui, "Cargando productos...");
                } else if pagina.is_empty() {
                    let mensaje = if self.busqueda.is_empty() {
                        "No hay productos disponibles".to_string()
                    } else {
                        format!("No se encontraron productos que coincidan con \"{}\"", self.busqueda)
                    };
                    if vacio(ui, "package", &mensaje, !self.busqueda.is_empty(), true) {
                        self.busqueda.clear();
                    }
                } else {
                    for (i, producto) in pagina.iter().enumerate() {
                        if i > 0 {
                            ui.add_space(8.0);
                        }
                        if boton_producto(ui, producto, &lista, iva_en_precios).clicked() {
                            elegido = Some(producto.clone());
                        }
                    }
                }
            });
            if total_paginas > 1 && !self.cargando {
                if let Some(destino) = paginacion(ui, "productos", self.pagina as u32, total_paginas, self.pagina != 1, self.pagina as u32 != total_paginas) {
                    self.pagina = (destino.clamp(1, total_paginas.max(1))) as usize;
                }
            }
            if pie(ui, !self.cargando) {
                cerrar = true;
            }
        });
        if elegido.is_some() {
            self.busqueda.clear();
        }
        if elegido.is_some() || cerrar || r.cerrar {
            self.abierto = false;
        }
        elegido
    }
}

fn chip(ui: &Ui, x: f32, centro: f32, contenido: &str, fondo: Color32, tinta: Color32, peso: Peso) -> f32 {
    let ancho = comun::ancho_texto(ui, contenido, Tam::Xs, peso) + 16.0;
    let rect = Rect::from_min_size(egui::pos2(x, centro - 12.0), Vec2::new(ancho, 24.0));
    ui.painter().rect_filled(rect, 4, fondo);
    comun::texto_en(ui, egui::pos2(x + 8.0, centro), Align2::LEFT_CENTER, contenido, Tam::Xs, peso, tinta);
    ancho
}

fn boton_producto(ui: &mut Ui, producto: &Value, lista: &ListaActiva, iva_en_precios: Option<bool>) -> egui::Response {
    let ancho = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 86.0), Sense::hover());
    let id = producto["id"].as_i64().unwrap_or_default();
    let respuesta = Boton::contorno("").borde(Some(color::BORDE_ENTRADA)).en(ui, rect, ("producto", id));
    let mut x = rect.left() + 17.0;
    let fila1 = rect.top() + 17.0 + 12.0;
    let plu = texto(producto, "plu");
    if !plu.is_empty() {
        x += chip(ui, x, fila1, &format!("PLU: {plu}"), paleta::BLUE_100, paleta::BLUE_800, Peso::Medio) + 8.0;
    }
    let precio_lista = lista.precios.get(&id).copied().filter(|p| *p != 0.0);
    if let (Some(nombre), true) = (&lista.nombre, lista.precios.contains_key(&id)) {
        let w = comun::ancho_texto(ui, nombre, Tam::Xs, Peso::Medio) + 16.0 + 16.0;
        let caja = Rect::from_min_size(egui::pos2(x, fila1 - 12.0), Vec2::new(w, 24.0));
        comun::pintar_relleno(ui, caja, 4.0, comun::esmeralda(), 1.0);
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x + 14.0, fila1), Vec2::splat(12.0)), "file-text", Color32::WHITE);
        comun::texto_en(ui, egui::pos2(x + 24.0, fila1), Align2::LEFT_CENTER, nombre, Tam::Xs, Peso::Medio, Color32::WHITE);
        x += w + 8.0;
    }
    comun::texto_en(ui, egui::pos2(x, fila1), Align2::LEFT_CENTER, texto(producto, "nombre"), Tam::Sm, Peso::Medio, color::TEXTO);
    let fila2 = fila1 + 12.0 + 4.0 + 12.0;
    let mut x = rect.left() + 17.0;
    x += chip(ui, x, fila2, texto(producto, "tipoMedida"), color::APAGADO, color::APAGADO_TEXTO, Peso::Normal) + 8.0;
    let base = numero(producto, "precio");
    let precio = formato::pesos(logica::precio_con_iva(Some(producto), precio_lista.unwrap_or(base), iva_en_precios));
    let r = comun::texto_en(ui, egui::pos2(x, fila2), Align2::LEFT_CENTER, &precio, Tam::Sm, Peso::Medio, color::PRIMARIO);
    x = r.right() + 8.0;
    if precio_lista.is_some_and(|p| p != base) {
        let tachado = formato::pesos(logica::precio_con_iva(Some(producto), base, iva_en_precios));
        let r = comun::texto_en(ui, egui::pos2(x, fila2), Align2::LEFT_CENTER, &tachado, Tam::Xs, Peso::Normal, color::APAGADO_TEXTO);
        ui.painter().hline(r.x_range(), r.center().y, Stroke::new(1.0, color::APAGADO_TEXTO));
        x = r.right() + 8.0;
    }
    if let Some(stock) = producto["stock"].as_f64() {
        let (fondo, tinta) = if stock > 0.0 { (paleta::GREEN_100, paleta::GREEN_700) } else { (paleta::RED_100, paleta::RED_700) };
        chip(ui, x, fila2, &format!("Stock: {}", formato::numero(stock)), fondo, tinta, Peso::Normal);
    }
    ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(rect.right() - 27.0, rect.center().y), Vec2::splat(20.0)), "plus", color::APAGADO_TEXTO);
    respuesta
}

pub struct CantidadProducto {
    producto: Option<Value>,
    precio: f64,
    precio_con_iva: f64,
    cantidad: f64,
    texto: String,
    error: Option<String>,
    enfocar: bool,
}

impl Default for CantidadProducto {
    fn default() -> Self {
        CantidadProducto { producto: None, precio: 0.0, precio_con_iva: 0.0, cantidad: 1.0, texto: "1".into(), error: None, enfocar: false }
    }
}

pub enum ResultadoCantidad {
    Confirmada(Value, f64, f64),
    Cerrada,
}

impl CantidadProducto {
    pub fn abrir(&mut self, producto: Value, precio: f64, iva_en_precios: Option<bool>) {
        let precio_con_iva = logica::precio_con_iva(Some(&producto), precio, iva_en_precios);
        *self = CantidadProducto { producto: Some(producto), precio, precio_con_iva, enfocar: true, ..CantidadProducto::default() };
    }

    fn cambiar_texto(&mut self) {
        match logica::parsear_numero(&self.texto).filter(|v| *v > 0.0) {
            Some(valor) => {
                self.error = None;
                self.cantidad = valor;
            }
            None => {
                self.cantidad = 0.0;
                self.error = Some("La cantidad debe ser mayor a 0".into());
            }
        }
    }

    pub fn mostrar(&mut self, ctx: &Context) -> Option<ResultadoCantidad> {
        let producto = self.producto.clone()?;
        let precio = self.precio;
        let subtotal = self.cantidad * self.precio_con_iva;
        let mut resultado = None;
        let mut confirmar = false;
        let r = ui::Dialogo::new("facturas-cantidad").ancho(448.0).mostrar(ctx, |ui| {
            comun::encabezado_dialogo(ui, "package", "Agregar Producto", "Especifique la cantidad para agregar a la factura", 12.0);
            ui.add_space(16.0);
            let ancho = ui.available_width();
            let stock = producto["stock"].as_f64();
            let alto = 16.0 + 28.0 + 8.0 + 20.0 + 8.0 + 20.0 + if stock.is_some() { 28.0 } else { 0.0 } + 16.0;
            let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::hover());
            ui.painter().rect_filled(caja, 8, color::APAGADO.gamma_multiply(0.3));
            let x0 = caja.left() + 16.0;
            let x1 = caja.right() - 16.0;
            let mut y = caja.top() + 16.0;
            let mut x = x0;
            let plu = texto(&producto, "plu");
            if !plu.is_empty() {
                x += chip(ui, x, y + 14.0, &format!("PLU: {plu}"), paleta::BLUE_100, paleta::BLUE_800, Peso::Medio) + 8.0;
            }
            comun::texto_en(ui, egui::pos2(x, y + 14.0), Align2::LEFT_CENTER, texto(&producto, "nombre"), Tam::Lg, Peso::Semi, color::TEXTO);
            y += 36.0;
            let mut fila = |etiqueta: &str, valor: &str, tinta: Color32| {
                comun::texto_en(ui, egui::pos2(x0, y + 10.0), Align2::LEFT_CENTER, etiqueta, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
                comun::texto_en(ui, egui::pos2(x1, y + 10.0), Align2::RIGHT_CENTER, valor, Tam::Sm, Peso::Medio, tinta);
                y += 28.0;
            };
            fila("Precio unitario:", &formato::pesos(self.precio_con_iva), color::TEXTO);
            fila("Unidad:", texto(&producto, "tipoMedida"), color::TEXTO);
            if let Some(stock) = stock {
                fila("Stock disponible:", &formato::numero(stock), if stock > 0.0 { paleta::GREEN_600 } else { paleta::RED_600 });
            }
            ui.add_space(24.0);
            comun::etiqueta(ui, "Cantidad");
            ui.add_space(16.0);
            let (fila_rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 40.0), Sense::hover());
            let partes = comun::encoger(&[40.0, ancho, 40.0], &[18.0, 0.0, 18.0], ancho - 24.0);
            let menos = Boton::contorno("").chico().icono("minus").habilitado(self.cantidad > 1.0);
            if menos.en(ui, Rect::from_min_size(fila_rect.min, Vec2::new(partes[0], 40.0)), "menos").clicked() && self.cantidad > 1.0 {
                self.cantidad -= 1.0;
                self.texto = logica::texto_entrada(self.cantidad);
                self.error = None;
            }
            let mas = Boton::contorno("").chico().icono("plus");
            if mas.en(ui, Rect::from_min_size(egui::pos2(fila_rect.right() - partes[2], fila_rect.top()), Vec2::new(partes[2], 40.0)), "mas").clicked() {
                self.cantidad += 1.0;
                self.texto = logica::texto_entrada(self.cantidad);
                self.error = None;
            }
            let campo = Rect::from_min_size(egui::pos2(fila_rect.left() + partes[0] + 12.0, fila_rect.top()), Vec2::new(partes[1], 40.0));
            let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(campo));
            let entrada = Entrada::new(&mut self.texto).tam(Tam::Lg).peso(Peso::Medio).centrado().id(Id::new("facturas-cantidad-entrada")).ui(&mut hijo);
            if self.enfocar {
                entrada.request_focus();
                self.enfocar = false;
            }
            if entrada.changed() {
                self.cambiar_texto();
            }
            if entrada.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                confirmar = true;
            }
            if let Some(error) = &self.error {
                ui.add_space(16.0);
                comun::linea(ui, error.clone(), Tam::Sm, Peso::Normal, paleta::RED_600);
            }
            ui.add_space(16.0);
            let (total, _) = ui.allocate_exact_size(Vec2::new(ancho, 52.0), Sense::hover());
            ui.painter().rect_filled(total, 8, color::PRIMARIO.gamma_multiply(0.05));
            comun::texto_en(ui, egui::pos2(total.left() + 12.0, total.center().y), Align2::LEFT_CENTER, "Subtotal:", Tam::Base, Peso::Medio, color::TEXTO);
            comun::texto_en(ui, egui::pos2(total.right() - 12.0, total.center().y), Align2::RIGHT_CENTER, &formato::pesos(subtotal), Tam::Lg, Peso::Negrita, color::PRIMARIO);
            ui.add_space(16.0);
            comun::borde_superior(ui);
            ui.add_space(16.0);
            ui::pie_dialogo(ui, |ui| {
                let habilitado = self.cantidad > 0.0 && self.error.is_none();
                let agregar = Boton::esmeralda("Agregar a Factura").icono("plus").separacion(16.0).habilitado(habilitado);
                if agregar.ui(ui).clicked() {
                    confirmar = true;
                }
                ui.add_space(4.0);
                if Boton::contorno("Cancelar").ui(ui).clicked() {
                    resultado = Some(ResultadoCantidad::Cerrada);
                }
            });
        });
        if confirmar {
            if self.cantidad <= 0.0 {
                self.error = Some("La cantidad debe ser mayor a 0".into());
            } else {
                resultado = Some(ResultadoCantidad::Confirmada(producto, self.cantidad, precio));
            }
        }
        if r.cerrar && resultado.is_none() {
            resultado = Some(ResultadoCantidad::Cerrada);
        }
        if resultado.is_some() {
            self.producto = None;
        }
        resultado
    }
}
