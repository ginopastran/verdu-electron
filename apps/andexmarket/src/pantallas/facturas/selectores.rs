use std::collections::HashMap;

use eframe::egui::{self, Context, CursorIcon, Id, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use nucleo::busqueda::ItemBusqueda;
use nucleo::facturas::{self as logica};
use nucleo::formato;
use serde_json::Value;

use super::{numero, texto, vista};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, fuente, fuente_mono, radio, sombra, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, toast, Popover};
use crate::ui::Peso;

const CLIENTES_POR_PAGINA: u32 = 10;
pub const PRODUCTOS_VISIBLES: usize = 30;
const ALTO_CLIENTE: f32 = 50.0;
const ALTO_PRODUCTO: f32 = 48.0;

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

#[derive(Default)]
pub struct Menu {
    pub abierto: bool,
    pub enfocado: bool,
    resaltado: usize,
    desplazar: bool,
}

impl Menu {
    pub fn abrir(&mut self) {
        self.abierto = true;
        self.resaltado = 0;
    }

    pub fn cerrar(&mut self) {
        self.abierto = false;
        self.resaltado = 0;
    }

    pub fn teclas(&mut self, ui: &Ui, campo: Id, total: usize) -> Option<usize> {
        if !self.enfocado && !ui.memory(|m| m.has_focus(campo)) {
            return None;
        }
        let abierto = self.abierto;
        let (abajo, arriba, enter, escape) = ui.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::ArrowDown),
                i.consume_key(Modifiers::NONE, Key::ArrowUp),
                abierto && total > 0 && i.consume_key(Modifiers::NONE, Key::Enter),
                abierto && i.consume_key(Modifiers::NONE, Key::Escape),
            )
        });
        if abajo {
            if self.abierto {
                self.resaltado += 1;
            }
            self.abierto = true;
            self.desplazar = true;
        }
        if arriba {
            self.resaltado = self.resaltado.saturating_sub(1);
            self.desplazar = true;
        }
        self.resaltado = self.resaltado.min(total.saturating_sub(1));
        if escape {
            self.cerrar();
            ui.memory_mut(|m| m.request_focus(campo));
        }
        enter.then_some(self.resaltado)
    }

    pub fn cerrar_con_clic_afuera(&mut self, ui: &Ui, campo: Rect, popup: Id) {
        let clic = ui.input(|i| if i.pointer.any_pressed() { i.pointer.interact_pos() } else { None });
        let Some(punto) = clic else { return };
        let en_popup = ui.ctx().memory(|m| m.area_rect(popup)).is_some_and(|r| r.contains(punto));
        if !campo.contains(punto) && !en_popup {
            self.cerrar();
        }
    }

    fn fila(&mut self, ui: &mut Ui, posicion: usize, alto: f32, hover: egui::Color32) -> (Rect, Response) {
        let (caja, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::click());
        let resaltada = posicion == self.resaltado;
        if resaltada && std::mem::take(&mut self.desplazar) {
            ui.scroll_to_rect(caja, None);
        }
        if resaltada {
            ui.painter().rect_filled(caja, radio::ICONO, color::VERDE_ELEGIDO);
        } else if respuesta.hovered() {
            ui.painter().rect_filled(caja, radio::ICONO, hover);
        }
        (caja, respuesta.on_hover_cursor(CursorIcon::PointingHand))
    }
}

#[derive(Default)]
pub struct Clientes {
    pedida: Option<String>,
    cargando: bool,
    error: Option<String>,
    pub lista: Vec<Value>,
    tarea: Option<Tarea<Result<Value, String>>>,
}

impl Clientes {
    pub fn pedir(&mut self, cx: &Contexto, busqueda: &str) {
        let busqueda = busqueda.trim();
        if self.pedida.as_deref() == Some(busqueda) {
            return;
        }
        self.pedida = Some(busqueda.to_string());
        self.cargando = true;
        self.error = None;
        let mut ruta = format!("/api/clientes?page=1&limit={CLIENTES_POR_PAGINA}");
        if !busqueda.is_empty() {
            ruta.push_str(&format!("&search={}", nucleo::api::codificar(busqueda)));
        }
        self.tarea = Some(pedir_json(cx, ruta));
    }

    pub fn recibir(&mut self, cx: &Contexto) {
        let Some(resultado) = esperar(&mut self.tarea) else { return };
        self.cargando = false;
        match resultado {
            Ok(datos) => self.lista = datos["clientes"].as_array().cloned().unwrap_or_default(),
            Err(error) => {
                self.lista.clear();
                self.error = Some(mensaje_error(&error));
                toast::error(&cx.egui, "Error al cargar la lista de clientes");
            }
        }
    }

    pub fn olvidar(&mut self) {
        *self = Clientes::default();
    }
}

#[derive(Default)]
pub struct Catalogo {
    de_apertura: Option<u32>,
    cargando: bool,
    error: Option<String>,
    pub lista: Vec<Value>,
    pub indice: Vec<ItemBusqueda>,
    tarea: Option<Tarea<Result<Value, String>>>,
}

impl Catalogo {
    pub fn pedir(&mut self, cx: &Contexto, apertura: u32) {
        if self.de_apertura == Some(apertura) {
            return;
        }
        self.de_apertura = Some(apertura);
        self.cargando = true;
        self.error = None;
        self.tarea = Some(pedir_json(cx, "/api/productos?limit=1000".into()));
    }

    pub fn recibir(&mut self, cx: &Contexto) {
        let Some(resultado) = esperar(&mut self.tarea) else { return };
        self.cargando = false;
        match resultado {
            Ok(datos) => {
                self.lista = datos["productos"].as_array().cloned().unwrap_or_default();
                self.indice = self.lista.iter().map(ItemBusqueda::de_json).collect();
            }
            Err(error) => {
                self.error = Some(mensaje_error(&error));
                toast::error(&cx.egui, "Error al cargar la lista de productos");
            }
        }
    }
}

fn aviso(ui: &mut Ui, mensaje: &str, tinta: egui::Color32) {
    let ancho = ui.available_width();
    let g = ui.painter().layout(mensaje.to_string(), fuente(tam::CONTROL, Peso::Normal), tinta, ancho - 20.0);
    let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, g.size().y + 28.0), Sense::hover());
    ui.painter().galley(caja.center() - g.size() / 2.0, g, tinta);
}

pub fn id_popup_clientes() -> Id {
    Id::new("factura-clientes-resultados")
}

pub fn id_popup_productos() -> Id {
    Id::new("factura-productos-resultados")
}

pub fn lista_clientes(ctx: &Context, ancla: &Response, menu: &mut Menu, clientes: &Clientes) -> Option<usize> {
    Popover::new(id_popup_clientes(), ancla)
        .visible(menu.abierto)
        .ancho(ancla.rect.width())
        .separacion(6.0)
        .sombra(sombra::AUTOCOMPLETAR)
        .alto_maximo(220.0)
        .mostrar(ctx, |ui| {
            if clientes.cargando && clientes.lista.is_empty() {
                vista::cargando(ui, 48.0, "Cargando clientes...");
                return None;
            }
            if let Some(error) = &clientes.error {
                aviso(ui, error, color::ROJO);
                return None;
            }
            if clientes.lista.is_empty() {
                let mensaje = if clientes.pedida.as_deref().is_some_and(|b| !b.is_empty()) {
                    "No se encontraron clientes con ese criterio de búsqueda"
                } else {
                    "No hay clientes disponibles"
                };
                aviso(ui, mensaje, color::TEXTO_APAGADO);
                return None;
            }
            let mut elegido = None;
            for (posicion, cliente) in clientes.lista.iter().enumerate() {
                let (caja, respuesta) = menu.fila(ui, posicion, ALTO_CLIENTE, color::FONDO_APP);
                let nombre = logica::nombre_cliente(cliente);
                let condicion = vista::condicion_fiscal(texto(cliente, "condicionFiscal"));
                let detalle = [texto(cliente, "cuit"), condicion.as_str()].into_iter().filter(|t| !t.is_empty()).collect::<Vec<_>>().join(" · ");
                let x = caja.left() + 10.0;
                let derecha = caja.right() - 10.0;
                celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=derecha, caja.top() + 8.0..=caja.top() + 26.0), &nombre, tam::CUERPO, Peso::Medio, color::TEXTO, false);
                ui.painter().with_clip_rect(caja).text(Pos2::new(x, caja.top() + 35.0), egui::Align2::LEFT_CENTER, &detalle, fuente_mono(tam::XS, Peso::Normal), color::TEXTO_TENUE);
                if respuesta.clicked() {
                    elegido = Some(posicion);
                }
            }
            elegido
        })
        .flatten()
}

pub struct VistaProductos<'a> {
    pub filtrados: &'a [Value],
    pub precios: &'a HashMap<i64, f64>,
    pub iva_en_precios: Option<bool>,
}

pub fn precio_producto(producto: &Value, precios: &HashMap<i64, f64>, iva_en_precios: Option<bool>) -> String {
    if logica::es_precio_libre(producto) {
        return "Precio a definir".into();
    }
    let id = producto["id"].as_i64().unwrap_or_default();
    let base = numero(producto, "precio");
    let base = nucleo::carrito::oferta_de(producto, chrono::Utc::now()).map_or(base, |p| nucleo::carrito::precio_con_oferta(base, p));
    let precio = precios.get(&id).copied().filter(|p| *p != 0.0).unwrap_or(base);
    let unidad = if texto(producto, "tipoMedida").eq_ignore_ascii_case("kg") { "/kg" } else { "/u" };
    format!("{}{unidad}", formato::pesos(logica::precio_con_iva(Some(producto), precio, iva_en_precios)))
}

pub fn lista_productos(ctx: &Context, ancla: &Response, menu: &mut Menu, catalogo: &Catalogo, datos: &VistaProductos) -> Option<usize> {
    Popover::new(id_popup_productos(), ancla)
        .visible(menu.abierto)
        .encima()
        .ancho(ancla.rect.width())
        .separacion(6.0)
        .alto_maximo(300.0)
        .mostrar(ctx, |ui| {
            if catalogo.cargando {
                vista::cargando(ui, 48.0, "Cargando productos...");
                return None;
            }
            if let Some(error) = &catalogo.error {
                aviso(ui, error, color::ROJO);
                return None;
            }
            let mut elegido = None;
            for (posicion, producto) in datos.filtrados.iter().take(PRODUCTOS_VISIBLES).enumerate() {
                let (caja, respuesta) = menu.fila(ui, posicion, ALTO_PRODUCTO, color::VERDE_ELEGIDO);
                let precio = precio_producto(producto, datos.precios, datos.iva_en_precios);
                let precio = v2::pintar_texto(ui, Pos2::new(caja.right() - 10.0, caja.center().y), egui::Align2::RIGHT_CENTER, precio, tam::CONTROL, Peso::Semi, color::TEXTO);
                let x = caja.left() + 10.0;
                let derecha = precio.left() - 10.0;
                let plu = texto(producto, "plu");
                let codigos = [if plu.is_empty() { String::new() } else { format!("PLU {plu}") }, texto(producto, "codigoBarras").to_string()]
                    .into_iter()
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join(" · ");
                let arriba = if codigos.is_empty() { caja.center().y - 9.0 } else { caja.top() + 7.0 };
                celda_texto(ui.painter(), Rect::from_x_y_ranges(x..=derecha, arriba..=arriba + 18.0), texto(producto, "nombre"), tam::CONTROL, Peso::Medio, color::TEXTO, false);
                if !codigos.is_empty() {
                    ui.painter().text(Pos2::new(x, caja.top() + 34.0), egui::Align2::LEFT_CENTER, &codigos, fuente_mono(tam::MINI, Peso::Normal), color::TEXTO_TENUE);
                }
                if respuesta.clicked() {
                    elegido = Some(posicion);
                }
            }
            if datos.filtrados.is_empty() {
                aviso(ui, "No hay productos que coincidan", color::TEXTO_APAGADO);
            }
            let total = datos.filtrados.len();
            let pie = if total > PRODUCTOS_VISIBLES {
                format!("Mostrando {PRODUCTOS_VISIBLES} de {total} · seguí escribiendo para filtrar")
            } else {
                format!("{total} producto{} · ↑ ↓ y Enter para elegir", if total == 1 { "" } else { "s" })
            };
            ui.add_space(4.0);
            let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::hover());
            ui.painter().hline(caja.x_range(), caja.top() + 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            v2::pintar_texto(ui, Pos2::new(caja.left() + 10.0, caja.top() + 15.0), egui::Align2::LEFT_CENTER, pie, tam::MINI, Peso::Normal, color::TEXTO_TENUE);
            elegido
        })
        .flatten()
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use serde_json::json;

    #[test]
    fn el_selector_muestra_la_oferta_y_con_lista_manda_la_lista() {
        let producto = json!({ "id": 5, "precio": 2600, "tipoMedida": "Kg", "ofertaPorcentaje": 10, "ofertaHasta": "2999-01-01T00:00:00.000Z" });
        let sin_listas = HashMap::new();
        assert_eq!(precio_producto(&producto, &sin_listas, None), "$2.340,00/kg");
        assert_eq!(precio_producto(&producto, &HashMap::from([(5, 2450.0)]), None), "$2.450,00/kg");
        assert_eq!(precio_producto(&producto, &HashMap::from([(5, 0.0)]), None), "$2.340,00/kg");
        let vencida = json!({ "id": 5, "precio": 2600, "tipoMedida": "Unidad", "ofertaPorcentaje": 10, "ofertaHasta": "2001-01-01T00:00:00.000Z" });
        assert_eq!(precio_producto(&vencida, &sin_listas, None), "$2.600,00/u");
    }
}
