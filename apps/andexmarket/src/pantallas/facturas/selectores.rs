use eframe::egui::{self, Align, Align2, Context, CursorIcon, Id, Key, Layout, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType};
use nucleo::facturas::{self as logica};
use nucleo::formato;
use serde_json::Value;

use super::vista;
use super::{id_texto, numero, texto};
use crate::app::Contexto;
use crate::pantallas::pedidos::boton_pagina;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, fuente, fuente_mono, radio, tam};
use crate::ui::v2::{self, opcion_eleccion, tarjetas::celda_texto, toast, Avatar, Boton, CampoTexto, Contenedor, Modal, Tono};
use crate::ui::{pintar_icono_solar_en, Peso};

const CLIENTES_POR_PAGINA: u32 = 10;
const PRODUCTOS_POR_PAGINA: usize = 6;
const RELLENO_CHIP: Vec2 = Vec2::new(7.0, 2.0);
const ALTO_FIJO: f32 = 262.0;

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
    let g = ui.painter().layout(error.to_string(), fuente(tam::CONTROL, Peso::Normal), color::ROJO, ancho - 14.0 - 18.0 - 10.0 - 14.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, g.size().y.max(18.0) + 24.0), Sense::hover());
    ui.painter().rect(rect, radio::CAMPO, color::ROJO_FONDO, Stroke::new(1.0, color::ROJO_BORDE), StrokeKind::Inside);
    pintar_icono_solar_en(ui, Rect::from_min_size(rect.min + Vec2::new(14.0, 12.0), Vec2::splat(18.0)), "alert", color::ROJO);
    ui.painter().galley(rect.min + Vec2::new(42.0, 12.0), g, color::ROJO);
    ui.add_space(12.0);
}

fn vacio(ui: &mut Ui, icono: &str, mensaje: &str, limpiar: bool, pista: bool) -> bool {
    vista::vacio(ui, icono, mensaje);
    let mut clic = false;
    if limpiar {
        ui.add_space(16.0);
        clic = vista::boton_centrado(ui, Boton::secundario("Limpiar búsqueda").alto(36.0).radio(radio::CONTROL));
        if pista {
            ui.add_space(10.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
            v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, "Intenta con términos más generales", tam::XS, Peso::Normal, color::TEXTO_TENUE);
        }
    }
    ui.add_space(40.0);
    clic
}

fn resumen(ui: &mut Ui, izquierda: &str, derecha: Option<String>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
    v2::pintar_texto(ui, rect.left_center(), Align2::LEFT_CENTER, izquierda, tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
    if let Some(derecha) = derecha {
        v2::pintar_texto(ui, rect.right_center(), Align2::RIGHT_CENTER, derecha, tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
    }
}

fn paginacion(ui: &mut Ui, pagina: u32, total: u32, anterior: bool, siguiente: bool) -> Option<u32> {
    let paginas = logica::paginas_visibles(pagina, total);
    ui.add_space(14.0);
    let ancho = (paginas.len() + 2) as f32 * 34.0 + (paginas.len() + 1) as f32 * 6.0;
    let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::hover());
    let mut hijo = ui.new_child(UiBuilder::new().max_rect(Rect::from_center_size(fila.center(), Vec2::new(ancho, 34.0))).layout(Layout::left_to_right(Align::Center)));
    hijo.spacing_mut().item_spacing.x = 6.0;
    let mut destino = None;
    if boton_pagina(&mut hijo, Ok("chevron-left"), false, anterior).clicked() {
        destino = Some(pagina.saturating_sub(1));
    }
    for n in paginas {
        if boton_pagina(&mut hijo, Err(n as usize), n == pagina, true).clicked() {
            destino = Some(n);
        }
    }
    if boton_pagina(&mut hijo, Ok("chevron-right"), false, siguiente).clicked() {
        destino = Some(pagina + 1);
    }
    destino
}

fn pie(ui: &mut Ui, habilitado: bool) -> bool {
    v2::pie_modal(ui, |ui| Boton::secundario("Cancelar").modal().habilitado(habilitado).ui(ui).clicked())
}

fn alto_lista(ctx: &Context, fijo: f32) -> f32 {
    (ctx.content_rect().height() * 0.8 - fijo).max(64.0)
}

fn desplazable<R>(ui: &mut Ui, id: Id, alto_max: f32, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    let previo: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(alto_max);
    let alto = previo.min(alto_max);
    let salida = egui::ScrollArea::vertical().id_salt(id).max_height(alto).min_scrolled_height(alto).auto_shrink([false, true]).show(ui, contenido);
    let nuevo = salida.content_size.y;
    if (nuevo.min(alto_max) - alto).abs() > 0.5 {
        ui.ctx().request_repaint();
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, nuevo));
    salida.inner
}

fn rotulo(respuesta: Response, nombre: &str) -> Response {
    respuesta.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, nombre));
    respuesta
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
                toast::error(&cx.egui, "Error al cargar la lista de clientes");
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
        let paginado = self.total_paginas > 1 && !self.cargando;
        let fijo = ALTO_FIJO + if self.error.is_some() { 54.0 } else { 0.0 } + if paginado { 48.0 } else { 0.0 };
        let alto = alto_lista(ctx, fijo);
        let r = Modal::new("facturas-selector-cliente", "Seleccionar Cliente")
            .subtitulo("Busque y seleccione un cliente para la factura")
            .icono("user")
            .ancho(640.0)
            .mostrar(ctx, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    super::modal::estilo_desplazamiento(ui);
                    let anterior = self.busqueda.clone();
                    CampoTexto::new(&mut self.busqueda)
                        .icono("search")
                        .sugerencia("Buscar por nombre, razón social, CUIT...")
                        .habilitado(!self.cargando)
                        .id(Id::new("facturas-buscar-cliente"))
                        .ui(ui);
                    if self.busqueda != anterior && !self.busqueda.is_empty() {
                        self.pagina = 1;
                    }
                    ui.add_space(16.0);
                    let derecha = (self.total_paginas > 1).then(|| format!("Página {} de {}", self.pagina, self.total_paginas));
                    resumen(ui, &logica::resumen_clientes(self.cargando, !self.busqueda.is_empty(), self.total), derecha);
                    ui.add_space(12.0);
                    if let Some(error) = &self.error {
                        alerta_error(ui, error);
                    }
                    desplazable(ui, Id::new("facturas-lista-clientes"), alto, |ui| {
                        ui.set_width(ui.available_width());
                        if self.cargando {
                            vista::cargando(ui, 88.0, "Cargando clientes...");
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
                                if boton_cliente(ui, cliente, activo).clicked() {
                                    elegido = Some(cliente.clone());
                                }
                            }
                        }
                    });
                    if paginado {
                        if let Some(destino) = paginacion(ui, self.pagina, self.total_paginas, self.hay_anterior, self.hay_siguiente) {
                            self.pagina = destino.clamp(1, self.total_paginas);
                        }
                    }
                });
                pie(ui, !self.cargando)
            });
        if elegido.is_some() || r.interior || r.cerrar {
            self.abierto = false;
        }
        elegido
    }
}

fn boton_cliente(ui: &mut Ui, cliente: &Value, activo: bool) -> Response {
    let nombre = logica::nombre_cliente(cliente);
    let condicion = vista::condicion_fiscal(texto(cliente, "condicionFiscal"));
    let detalle = [texto(cliente, "cuit"), condicion.as_str()].into_iter().filter(|t| !t.is_empty()).collect::<Vec<_>>().join(" · ");
    let ancho = ui.available_width();
    let respuesta = opcion_eleccion(ui, Vec2::new(ancho, 60.0), radio::CAMPO, activo, 1.0, |ui, rect, _| {
        let derecha = rect.right() - if activo { 16.0 + 18.0 + 12.0 } else { 16.0 };
        let x = rect.left() + 16.0;
        celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=derecha, rect.top() + 11.0..=rect.top() + 29.0), &nombre, tam::CUERPO_MAS, Peso::Medio, color::TEXTO, false);
        ui.painter().text(Pos2::new(x, rect.top() + 41.0), Align2::LEFT_CENTER, &detalle, fuente_mono(tam::XS, Peso::Normal), color::TEXTO_TENUE);
        if activo {
            pintar_icono_solar_en(ui, Rect::from_center_size(Pos2::new(rect.right() - 16.0 - 9.0, rect.center().y), Vec2::splat(18.0)), "circle-check", color::VERDE);
        }
    });
    rotulo(respuesta, &nombre)
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
                    toast::error(&cx.egui, "Error al cargar la lista de productos");
                }
            }
        }
        let filtrados = logica::filtrar_productos(&self.productos, &self.busqueda, excluidos);
        let total_paginas = filtrados.len().div_ceil(PRODUCTOS_POR_PAGINA) as u32;
        let inicio = (self.pagina.saturating_sub(1)) * PRODUCTOS_POR_PAGINA;
        let pagina: Vec<Value> = filtrados.iter().skip(inicio).take(PRODUCTOS_POR_PAGINA).cloned().collect();
        let mut elegido = None;
        let mut cerrar = false;
        let paginado = total_paginas > 1 && !self.cargando;
        let fijo = ALTO_FIJO + if self.error.is_some() { 54.0 } else { 0.0 } + if paginado { 48.0 } else { 0.0 };
        let alto = alto_lista(ctx, fijo);
        let r = Modal::new("facturas-selector-producto", "Seleccionar Producto")
            .subtitulo("Busque y seleccione productos para agregar a la factura")
            .icono("cart-plus")
            .ancho(640.0)
            .mostrar(ctx, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    super::modal::estilo_desplazamiento(ui);
                    let anterior = self.busqueda.clone();
                    let respuesta = CampoTexto::new(&mut self.busqueda)
                        .icono("search")
                        .sugerencia("Buscar producto (nombre o PLU)...")
                        .habilitado(!self.cargando)
                        .id(Id::new("facturas-buscar-producto"))
                        .ui(ui)
                        .respuesta;
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
                    ui.add_space(12.0);
                    if let Some(error) = &self.error {
                        alerta_error(ui, error);
                    }
                    desplazable(ui, Id::new("facturas-lista-productos"), alto, |ui| {
                        ui.set_width(ui.available_width());
                        if self.cargando {
                            vista::cargando(ui, 88.0, "Cargando productos...");
                        } else if pagina.is_empty() {
                            let mensaje = if self.busqueda.is_empty() {
                                "No hay productos disponibles".to_string()
                            } else {
                                format!("No se encontraron productos que coincidan con \"{}\"", self.busqueda)
                            };
                            if vacio(ui, "cart-plus", &mensaje, !self.busqueda.is_empty(), true) {
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
                    if paginado {
                        if let Some(destino) = paginacion(ui, self.pagina as u32, total_paginas, self.pagina != 1, self.pagina as u32 != total_paginas) {
                            self.pagina = (destino.clamp(1, total_paginas.max(1))) as usize;
                        }
                    }
                });
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

fn boton_producto(ui: &mut Ui, producto: &Value, lista: &ListaActiva, iva_en_precios: Option<bool>) -> Response {
    let id = producto["id"].as_i64().unwrap_or_default();
    let nombre = texto(producto, "nombre");
    let base = numero(producto, "precio");
    let precio_lista = lista.precios.get(&id).copied().filter(|p| *p != 0.0);
    let precio = formato::pesos(logica::precio_con_iva(Some(producto), precio_lista.unwrap_or(base), iva_en_precios));
    let tachado = precio_lista.is_some_and(|p| p != base).then(|| formato::pesos(logica::precio_con_iva(Some(producto), base, iva_en_precios)));
    let nombre_lista = lista.nombre.as_ref().filter(|_| lista.precios.contains_key(&id));
    let plu = texto(producto, "plu");
    let detalle = [if plu.is_empty() { String::new() } else { format!("PLU {plu}") }, texto(producto, "tipoMedida").to_string()]
        .into_iter()
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let ancho = ui.available_width();
    let respuesta = opcion_eleccion(ui, Vec2::new(ancho, 64.0), radio::CAMPO, false, 1.0, |ui, rect, _| {
        let avatar = Rect::from_min_size(Pos2::new(rect.left() + 14.0, rect.center().y - 18.0), Vec2::splat(36.0));
        Avatar::producto(v2::iniciales(nombre)).pintar(ui, avatar);
        let icono = Rect::from_center_size(Pos2::new(rect.right() - 16.0 - 9.0, rect.center().y), Vec2::splat(18.0));
        pintar_icono_solar_en(ui, icono, "plus", color::VERDE);
        let x_precio = icono.left() - 14.0;
        let y_precio = if tachado.is_some() { rect.center().y - 9.0 } else { rect.center().y };
        let mut limite = v2::pintar_texto(ui, Pos2::new(x_precio, y_precio), Align2::RIGHT_CENTER, &precio, tam::ENTRADA, Peso::Semi, color::TEXTO).left();
        if let Some(tachado) = &tachado {
            let r = v2::pintar_texto(ui, Pos2::new(x_precio, rect.center().y + 10.0), Align2::RIGHT_CENTER, tachado, tam::XS, Peso::Normal, color::TEXTO_TENUE);
            ui.painter().hline(r.x_range(), r.center().y, Stroke::new(1.0, color::TEXTO_TENUE));
            limite = limite.min(r.left());
        }
        limite -= 16.0;
        let x = avatar.right() + 12.0;
        let y_nombre = rect.top() + 22.0;
        let ancho_chip = nombre_lista.map_or(0.0, |n| vista::medir_chip(ui, n, tam::MINI, RELLENO_CHIP).x + 8.0);
        let ancho_nombre = v2::ancho_texto(ui, nombre, tam::CUERPO_MAS, Peso::Medio).min(limite - ancho_chip - x);
        celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=x + ancho_nombre, y_nombre - 9.0..=y_nombre + 9.0), nombre, tam::CUERPO_MAS, Peso::Medio, color::TEXTO, false);
        if let Some(nombre_lista) = nombre_lista {
            let (fondo, tinta) = Tono::Marca.colores();
            let alto_chip = vista::medir_chip(ui, nombre_lista, tam::MINI, RELLENO_CHIP).y;
            vista::pintar_chip(ui, Pos2::new(x + ancho_nombre + 8.0, y_nombre - alto_chip / 2.0), nombre_lista, tam::MINI, RELLENO_CHIP, fondo, tinta);
        }
        let y_detalle = rect.top() + 44.0;
        let mut x_detalle = x;
        if !detalle.is_empty() {
            x_detalle = ui.painter().text(Pos2::new(x, y_detalle), Align2::LEFT_CENTER, &detalle, fuente_mono(tam::XS, Peso::Normal), color::TEXTO_TENUE).right() + 8.0;
        }
        if let Some(stock) = producto["stock"].as_f64() {
            let (fondo, tinta) = if stock > 0.0 { Tono::Exito } else { Tono::Peligro }.colores();
            let contenido = format!("Stock: {}", formato::numero(stock));
            let alto_chip = vista::medir_chip(ui, &contenido, tam::MINI, RELLENO_CHIP).y;
            vista::pintar_chip(ui, Pos2::new(x_detalle, y_detalle - alto_chip / 2.0), &contenido, tam::MINI, RELLENO_CHIP, fondo, tinta);
        }
    });
    rotulo(respuesta, nombre)
}

fn boton_paso(ui: &mut Ui, simbolo: &str, habilitado: bool) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(52.0), if habilitado { Sense::click() } else { Sense::hover() });
    let alfa = if habilitado { 1.0 } else { 0.45 };
    let borde = if habilitado && respuesta.hovered() { color::VERDE } else { color::BORDE };
    ui.painter().rect(rect, radio::CAMPO, color::BLANCO, Stroke::new(1.0, borde.gamma_multiply(alfa)), StrokeKind::Inside);
    v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, simbolo, 22.0, Peso::Normal, color::TEXTO_SECUNDARIO.gamma_multiply(alfa));
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
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

    fn tarjeta_producto(&self, ui: &mut Ui, producto: &Value) {
        let ancho = ui.available_width();
        let stock = producto["stock"].as_f64();
        let filas = if stock.is_some() { 3.0 } else { 2.0 };
        let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, 16.0 + 24.0 + 8.0 + filas * 26.0 + 12.0), Sense::hover());
        ui.painter().rect_filled(caja, radio::CAMPO, color::FONDO_APP);
        let x0 = caja.left() + 16.0;
        let x1 = caja.right() - 16.0;
        let mut y = caja.top() + 16.0 + 12.0;
        let mut x = x0;
        let plu = texto(producto, "plu");
        if !plu.is_empty() {
            let contenido = format!("PLU {plu}");
            let alto_chip = vista::medir_chip(ui, &contenido, tam::MINI, RELLENO_CHIP).y;
            let (fondo, tinta) = Tono::Marca.colores();
            x = vista::pintar_chip(ui, Pos2::new(x, y - alto_chip / 2.0), &contenido, tam::MINI, RELLENO_CHIP, fondo, tinta).right() + 8.0;
        }
        celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=x1, y - 12.0..=y + 12.0), texto(producto, "nombre"), tam::FILA, Peso::Semi, color::TEXTO, false);
        y += 12.0 + 8.0 + 13.0;
        let mut fila = |etiqueta: &str, valor: &str, tinta| {
            v2::pintar_texto(ui, Pos2::new(x0, y), Align2::LEFT_CENTER, etiqueta, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
            v2::pintar_texto(ui, Pos2::new(x1, y), Align2::RIGHT_CENTER, valor, tam::CONTROL, Peso::Medio, tinta);
            y += 26.0;
        };
        fila("Precio unitario:", &formato::pesos(self.precio_con_iva), color::TEXTO);
        fila("Unidad:", texto(producto, "tipoMedida"), color::TEXTO);
        if let Some(stock) = stock {
            fila("Stock disponible:", &formato::numero(stock), if stock > 0.0 { color::VERDE_EXITO } else { color::ROJO });
        }
    }

    pub fn mostrar(&mut self, ctx: &Context) -> Option<ResultadoCantidad> {
        let producto = self.producto.clone()?;
        let precio = self.precio;
        let subtotal = self.cantidad * self.precio_con_iva;
        let mut resultado = None;
        let mut confirmar = false;
        let r = Modal::new("facturas-cantidad", "Agregar Producto")
            .subtitulo("Especifique la cantidad para agregar a la factura")
            .icono("cart-plus")
            .ancho(480.0)
            .mostrar(ctx, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    self.tarjeta_producto(ui, &producto);
                    ui.add_space(16.0);
                    vista::etiqueta(ui, "Cantidad");
                    ui.add_space(6.0);
                    let ancho_campo = ui.available_width() - 52.0 * 2.0 - 16.0;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if boton_paso(ui, "−", self.cantidad > 1.0).clicked() && self.cantidad > 1.0 {
                            self.cantidad -= 1.0;
                            self.texto = logica::texto_entrada(self.cantidad);
                            self.error = None;
                        }
                        let entrada = Contenedor { alto: 52.0, error: self.error.is_some(), ..Default::default() }.mostrar(ui, ancho_campo, |ui| {
                            v2::entrada_desnuda(&mut self.texto).id(Id::new("facturas-cantidad-entrada")).tam(22.0).peso(Peso::Semi).centrada().ui(ui)
                        });
                        if boton_paso(ui, "+", true).clicked() {
                            self.cantidad += 1.0;
                            self.texto = logica::texto_entrada(self.cantidad);
                            self.error = None;
                        }
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
                    });
                    if let Some(error) = &self.error {
                        ui.add_space(8.0);
                        v2::texto(ui, error.clone(), tam::CONTROL, Peso::Normal, color::ROJO);
                    }
                    ui.add_space(16.0);
                    let valor = v2::galeria(ui, formato::pesos(subtotal), fuente(tam::GRANDE, Peso::Semi), color::TEXTO);
                    let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0 + valor.size().y), Sense::hover());
                    ui.painter().rect_filled(fila, radio::CAMPO, color::FONDO_APP);
                    v2::pintar_texto(ui, Pos2::new(fila.left() + 14.0, fila.center().y), Align2::LEFT_CENTER, "Subtotal", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO);
                    ui.painter().galley(Pos2::new(fila.right() - 14.0 - valor.size().x, fila.center().y - valor.size().y / 2.0), valor, color::TEXTO);
                });
                v2::pie_modal(ui, |ui| {
                    let habilitado = self.cantidad > 0.0 && self.error.is_none();
                    if Boton::primario("Agregar a Factura").icono("plus").modal().habilitado(habilitado).ui(ui).clicked() {
                        confirmar = true;
                    }
                    if Boton::secundario("Cancelar").modal().ui(ui).clicked() {
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
