use eframe::egui::{self, Align, Align2, Color32, Context, CursorIcon, Id, Key, Layout, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2};
use nucleo::facturas::{self as logica};
use nucleo::formato;
use serde_json::Value;

use super::comun;
use super::{id_texto, numero, pesos, texto};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, fuente, fuente_mono, radio, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, toast, Boton, CampoTexto, Modal};
use crate::ui::Peso;

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
    egui::Frame::NONE
        .fill(color::ROJO_FONDO)
        .stroke(Stroke::new(1.0, color::ROJO_BORDE))
        .corner_radius(radio::CAMPO)
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (caja, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                v2::pintar_icono(ui, caja.center(), "alert", 18.0, color::ROJO_INTENSO);
                let ancho = ui.available_width();
                v2::parrafo(ui, error, tam::ETIQUETA, color::ROJO, 1.45, ancho);
            });
        });
    ui.add_space(12.0);
}

fn vacio(ui: &mut Ui, icono: &str, mensaje: &str, limpiar: bool, pista: bool) -> bool {
    comun::vacio(ui, icono, mensaje);
    let mut clic = false;
    if limpiar {
        ui.add_space(16.0);
        clic = comun::boton_centrado(ui, Boton::secundario("Limpiar búsqueda").alto(36.0).radio(radio::CONTROL).tam_texto(tam::CONTROL));
        if pista {
            ui.add_space(8.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
            v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, "Intentá con términos más generales", tam::XS, Peso::Normal, color::TEXTO_TENUE);
        }
    }
    ui.add_space(32.0);
    clic
}

fn resumen(ui: &mut Ui, izquierda: &str, derecha: Option<String>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
    v2::pintar_texto(ui, rect.left_center(), Align2::LEFT_CENTER, izquierda, tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
    if let Some(derecha) = derecha {
        v2::pintar_texto(ui, rect.right_center(), Align2::RIGHT_CENTER, derecha, tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
    }
}

fn paginacion(ui: &mut Ui, pagina: u32, total: u32, anterior: bool, siguiente: bool) -> Option<u32> {
    ui.add_space(12.0);
    v2::separador(ui);
    ui.add_space(12.0);
    let mut destino = None;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::hover());
    v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, format!("Página {pagina} de {total}"), tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
    let mut izquierda = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
    if comun::boton_pagina(&mut izquierda, "Anterior", "chevron-left", false, anterior).clicked() {
        destino = Some(pagina.saturating_sub(1));
    }
    let mut derecha = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::right_to_left(Align::Center)));
    if comun::boton_pagina(&mut derecha, "Siguiente", "chevron-right", true, siguiente).clicked() {
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

fn fila_opcion(ui: &mut Ui, id: Id, alto: f32, activo: bool, contenido: impl FnOnce(&Ui, Rect)) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    let respuesta = ui.interact(rect, id, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    let fondo = if activo {
        color::VERDE_ELEGIDO
    } else if respuesta.hovered() {
        color::FONDO_APP
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, radio::ICONO, fondo);
    contenido(ui, rect.shrink2(Vec2::new(10.0, 0.0)));
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
        let mut cerrar = false;
        let fijo = 260.0 + if self.error.is_some() { 64.0 } else { 0.0 } + if self.total_paginas > 1 && !self.cargando { 57.0 } else { 0.0 };
        let alto = alto_lista(ctx, fijo);
        let r = Modal::new("facturas-selector-cliente", "Seleccionar cliente")
            .subtitulo("Buscá y elegí el cliente de la factura")
            .icono("user")
            .ancho(560.0)
            .mostrar(ctx, |ui| {
                v2::cuerpo_modal(ui, 18, |ui| {
                    let anterior = self.busqueda.clone();
                    CampoTexto::new(&mut self.busqueda)
                        .alto(44.0)
                        .radio(radio::BOTON_MODAL)
                        .relleno(12.0, 12.0)
                        .icono("search")
                        .tam_texto(tam::CUERPO)
                        .sugerencia("Buscar por nombre, razón social o CUIT")
                        .habilitado(!self.cargando)
                        .id(Id::new("facturas-buscar-cliente"))
                        .ui(ui);
                    if self.busqueda != anterior && !self.busqueda.is_empty() {
                        self.pagina = 1;
                    }
                    ui.add_space(12.0);
                    let derecha = (self.total_paginas > 1).then(|| format!("Página {} de {}", self.pagina, self.total_paginas));
                    resumen(ui, &logica::resumen_clientes(self.cargando, !self.busqueda.is_empty(), self.total), derecha);
                    ui.add_space(10.0);
                    if let Some(error) = &self.error {
                        alerta_error(ui, error);
                    }
                    ScrollArea::vertical().id_salt("facturas-lista-clientes").max_height(alto).auto_shrink([false, true]).show(ui, |ui| {
                        if self.cargando {
                            comun::cargando(ui, 88.0, "Cargando clientes…");
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
                                let activo = seleccionado.as_deref() == Some(id_texto(&cliente["id"]).as_str());
                                if fila_cliente(ui, cliente, activo, i).clicked() {
                                    elegido = Some(cliente.clone());
                                }
                            }
                        }
                    });
                    if self.total_paginas > 1 && !self.cargando {
                        if let Some(destino) = paginacion(ui, self.pagina, self.total_paginas, self.hay_anterior, self.hay_siguiente) {
                            self.pagina = destino.clamp(1, self.total_paginas);
                        }
                    }
                });
                cerrar = pie(ui, !self.cargando);
            });
        if elegido.is_some() || cerrar || r.cerrar {
            self.abierto = false;
        }
        elegido
    }
}

fn fila_cliente(ui: &mut Ui, cliente: &Value, activo: bool, indice: usize) -> egui::Response {
    fila_opcion(ui, Id::new(("facturas-cliente", indice)), 50.0, activo, |ui, rect| {
        let derecha = if activo { rect.right() - 28.0 } else { rect.right() };
        let arriba = rect.center().y - 16.0;
        celda_texto(ui.painter(), Rect::from_x_y_ranges(rect.left()..=derecha, arriba..=arriba + 17.0), &logica::nombre_cliente(cliente), tam::CUERPO, Peso::Medio, color::TEXTO, false);
        let cuit = texto(cliente, "cuit");
        let condicion = comun::condicion_fiscal(texto(cliente, "condicionFiscal"));
        let detalle = match (cuit.is_empty(), condicion.is_empty()) {
            (false, false) => format!("{cuit} · {condicion}"),
            (false, true) => cuit.to_string(),
            _ => condicion,
        };
        let g = v2::galeria(ui, detalle, fuente_mono(tam::XS, Peso::Normal), color::TEXTO_TENUE);
        ui.painter().with_clip_rect(rect.with_max_x(derecha)).galley(egui::pos2(rect.left(), arriba + 18.0), g, color::TEXTO_TENUE);
        if activo {
            v2::pintar_icono(ui, egui::pos2(rect.right() - 8.0, rect.center().y), "check", 16.0, color::VERDE);
        }
    })
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
        let fijo = 260.0 + if self.error.is_some() { 64.0 } else { 0.0 } + if total_paginas > 1 && !self.cargando { 57.0 } else { 0.0 };
        let alto = alto_lista(ctx, fijo);
        let r = Modal::new("facturas-selector-producto", "Agregar producto")
            .subtitulo("Buscá el producto para sumarlo a la factura")
            .icono("cart-plus")
            .ancho(600.0)
            .mostrar(ctx, |ui| {
                v2::cuerpo_modal(ui, 18, |ui| {
                    let anterior = self.busqueda.clone();
                    let id = Id::new("facturas-buscar-producto");
                    let respuesta = CampoTexto::new(&mut self.busqueda)
                        .alto(44.0)
                        .radio(radio::BOTON_MODAL)
                        .relleno(12.0, 12.0)
                        .icono("search")
                        .tam_texto(tam::CUERPO)
                        .sugerencia("Buscar producto por nombre o PLU")
                        .habilitado(!self.cargando)
                        .id(id)
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
                    ui.add_space(12.0);
                    let derecha = (total_paginas > 1).then(|| format!("Página {} de {}", self.pagina, total_paginas));
                    resumen(ui, &logica::resumen_productos(self.cargando, !self.busqueda.is_empty(), filtrados.len()), derecha);
                    ui.add_space(10.0);
                    if let Some(error) = &self.error {
                        alerta_error(ui, error);
                    }
                    ScrollArea::vertical().id_salt("facturas-lista-productos").max_height(alto).auto_shrink([false, true]).show(ui, |ui| {
                        if self.cargando {
                            comun::cargando(ui, 88.0, "Cargando productos…");
                        } else if pagina.is_empty() {
                            let mensaje = if self.busqueda.is_empty() {
                                "No hay productos disponibles".to_string()
                            } else {
                                format!("No se encontraron productos que coincidan con \"{}\"", self.busqueda)
                            };
                            if vacio(ui, "cart", &mensaje, !self.busqueda.is_empty(), true) {
                                self.busqueda.clear();
                            }
                        } else {
                            for producto in &pagina {
                                if fila_producto(ui, producto, &lista, iva_en_precios).clicked() {
                                    elegido = Some(producto.clone());
                                }
                            }
                        }
                    });
                    if total_paginas > 1 && !self.cargando {
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

fn fila_producto(ui: &mut Ui, producto: &Value, lista: &ListaActiva, iva_en_precios: Option<bool>) -> egui::Response {
    let id = producto["id"].as_i64().unwrap_or_default();
    fila_opcion(ui, Id::new(("facturas-producto", id)), 50.0, false, |ui, rect| {
        let precio_lista = lista.precios.get(&id).copied().filter(|p| *p != 0.0);
        let base = numero(producto, "precio");
        let precio = v2::pintar_texto(
            ui,
            egui::pos2(rect.right(), rect.center().y),
            Align2::RIGHT_CENTER,
            pesos(logica::precio_con_iva(Some(producto), precio_lista.unwrap_or(base), iva_en_precios)),
            tam::CONTROL,
            Peso::Semi,
            color::TEXTO,
        );
        let mut derecha = precio.left() - 10.0;
        if precio_lista.is_some_and(|p| p != base) {
            let tachado = v2::pintar_texto(
                ui,
                egui::pos2(derecha, rect.center().y),
                Align2::RIGHT_CENTER,
                pesos(logica::precio_con_iva(Some(producto), base, iva_en_precios)),
                tam::XS,
                Peso::Normal,
                color::TEXTO_DESHABILITADO,
            );
            ui.painter().hline(tachado.x_range(), tachado.center().y, Stroke::new(1.0, color::TEXTO_DESHABILITADO));
            derecha = tachado.left() - 10.0;
        }
        let arriba = rect.center().y - 16.0;
        let mut izquierda = rect.left();
        if let (Some(nombre), true) = (&lista.nombre, lista.precios.contains_key(&id)) {
            let chip = comun::pintar_chip(ui, egui::pos2(izquierda, arriba + 0.5), nombre, tam::MINI, comun::RELLENO_CHIP_TIPO, color::VERDE_CHIP, color::VERDE);
            izquierda = chip.right() + 6.0;
        }
        celda_texto(ui.painter(), Rect::from_x_y_ranges(izquierda..=derecha, arriba..=arriba + 17.0), texto(producto, "nombre"), tam::CONTROL, Peso::Medio, color::TEXTO, false);
        let mut detalle = Vec::new();
        let plu = texto(producto, "plu");
        if !plu.is_empty() {
            detalle.push(format!("PLU {plu}"));
        }
        let medida = texto(producto, "tipoMedida");
        if !medida.is_empty() {
            detalle.push(medida.to_string());
        }
        let g = v2::galeria(ui, detalle.join(" · "), fuente_mono(tam::MINI, Peso::Normal), color::TEXTO_TENUE);
        let ancho_detalle = g.size().x;
        let y = arriba + 18.0;
        ui.painter().galley(egui::pos2(rect.left(), y), g, color::TEXTO_TENUE);
        if let Some(stock) = producto["stock"].as_f64() {
            let tinta = if stock > 0.0 { color::VERDE_EXITO } else { color::ROJO };
            let prefijo = if ancho_detalle > 0.0 { " · " } else { "" };
            ui.painter().text(
                egui::pos2(rect.left() + ancho_detalle, y),
                Align2::LEFT_TOP,
                format!("{prefijo}Stock {}", formato::numero(stock)),
                fuente_mono(tam::MINI, Peso::Normal),
                tinta,
            );
        }
    })
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
        let nombre = texto(&producto, "nombre").to_string();
        let r = Modal::new("facturas-cantidad", "Cantidad").subtitulo(&nombre).icono("cart-plus").ancho(440.0).mostrar(ctx, |ui| {
            v2::cuerpo_modal(ui, 18, |ui| {
                egui::Frame::NONE.fill(color::FONDO_APP).corner_radius(radio::CAMPO).inner_margin(egui::Margin::symmetric(16, 12)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let plu = texto(&producto, "plu");
                    let mut filas = vec![("Precio unitario", pesos(self.precio_con_iva), color::TEXTO), ("Unidad", texto(&producto, "tipoMedida").to_string(), color::TEXTO)];
                    if !plu.is_empty() {
                        filas.insert(0, ("PLU", plu.to_string(), color::TEXTO));
                    }
                    if let Some(stock) = producto["stock"].as_f64() {
                        filas.push(("Stock disponible", formato::numero(stock), if stock > 0.0 { color::VERDE_EXITO } else { color::ROJO }));
                    }
                    for (indice, (etiqueta, valor, tinta)) in filas.iter().enumerate() {
                        if indice > 0 {
                            ui.add_space(6.0);
                        }
                        let (fila, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 17.0), Sense::hover());
                        v2::pintar_texto(ui, fila.left_center(), Align2::LEFT_CENTER, *etiqueta, tam::CONTROL, Peso::Normal, color::TEXTO_APAGADO);
                        v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, valor.as_str(), tam::CONTROL, Peso::Semi, *tinta);
                    }
                });
                ui.add_space(14.0);
                comun::etiqueta(ui, "Cantidad");
                ui.add_space(6.0);
                let ancho = ui.available_width();
                let (fila, _) = ui.allocate_exact_size(Vec2::new(ancho, 46.0), Sense::hover());
                let mut menos = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(fila.min, Vec2::splat(46.0))));
                if paso(&mut menos, "menos", self.cantidad > 1.0).clicked() && self.cantidad > 1.0 {
                    self.cantidad -= 1.0;
                    self.texto = logica::texto_entrada(self.cantidad);
                    self.error = None;
                }
                let mut mas = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(egui::pos2(fila.right() - 46.0, fila.top()), Vec2::splat(46.0))));
                if paso(&mut mas, "mas", true).clicked() {
                    self.cantidad += 1.0;
                    self.texto = logica::texto_entrada(self.cantidad);
                    self.error = None;
                }
                let campo = Rect::from_min_max(egui::pos2(fila.left() + 56.0, fila.top()), egui::pos2(fila.right() - 56.0, fila.bottom()));
                let mut hijo = ui.new_child(UiBuilder::new().max_rect(campo));
                let entrada = CampoTexto::new(&mut self.texto)
                    .tam_texto(18.0)
                    .peso(Peso::Medio)
                    .centrado()
                    .error(self.error.is_some())
                    .id(Id::new("facturas-cantidad-entrada"))
                    .ui(&mut hijo)
                    .respuesta;
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
                    ui.add_space(6.0);
                    v2::texto(ui, error.clone(), tam::CHICO, Peso::Normal, color::ROJO);
                }
                ui.add_space(14.0);
                let (total, _) = ui.allocate_exact_size(Vec2::new(ancho, 52.0), Sense::hover());
                ui.painter().rect_filled(total, radio::CAMPO, color::VERDE_ELEGIDO);
                v2::pintar_texto(ui, egui::pos2(total.left() + 16.0, total.center().y), Align2::LEFT_CENTER, "Subtotal", tam::CUERPO, Peso::Normal, color::TEXTO_SECUNDARIO);
                v2::pintar_texto(ui, egui::pos2(total.right() - 16.0, total.center().y), Align2::RIGHT_CENTER, pesos(subtotal), tam::GRANDE, Peso::Semi, color::VERDE);
            });
            v2::pie_modal(ui, |ui| {
                let habilitado = self.cantidad > 0.0 && self.error.is_none();
                if Boton::primario("Agregar a la factura").icono("plus").modal().habilitado(habilitado).ui(ui).clicked() {
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

fn paso(ui: &mut Ui, clave: &str, habilitado: bool) -> egui::Response {
    let rect = ui.max_rect();
    let respuesta = ui.interact(rect, ui.id().with(clave), if habilitado { Sense::click() } else { Sense::hover() });
    let encima = habilitado && respuesta.hovered();
    let alfa = if habilitado { 1.0 } else { 0.45 };
    let tinta = if encima { color::VERDE } else { color::TEXTO_SECUNDARIO }.gamma_multiply(alfa);
    let borde = if encima { color::VERDE } else { color::BORDE }.gamma_multiply(alfa);
    ui.painter().rect(rect, radio::CAMPO, color::BLANCO, Stroke::new(1.0, borde), StrokeKind::Inside);
    if clave == "mas" {
        v2::pintar_icono(ui, rect.center(), "plus", 18.0, tinta);
    } else {
        ui.painter().hline(rect.center().x - 6.0..=rect.center().x + 6.0, rect.center().y, Stroke::new(2.0, tinta));
    }
    let _ = fuente;
    if habilitado {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
}
