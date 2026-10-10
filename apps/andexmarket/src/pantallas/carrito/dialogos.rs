use eframe::egui::{self, Color32, CursorIcon, Id, Key, Popup, PopupCloseBehavior, Pos2, Rect, RectAlign, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::carrito::Producto;
use serde_json::Value;

use super::diseno::Texto;
use super::productos::{self, formato_local, numero_js, parse_float, texto_peso, FilaLista, ProductoDisponible};
use super::vista::precio_visual;
use crate::app::Contexto;
use crate::tareas::{self, Tarea};
use crate::ui::{self, borde, color, fuente, paleta, Boton, Dialogo, Entrada, Peso, Tam, Variante};

pub enum Salida {
    Nada,
    Cerrar,
    Agregar(Producto),
}

pub fn etiqueta_base(negocio: Option<&Value>) -> String {
    negocio
        .and_then(|n| n["etiquetaPrecioBase"].as_str())
        .filter(|e| !e.is_empty())
        .unwrap_or("Precio catálogo")
        .trim()
        .to_string()
}

fn color_titulo() -> Color32 {
    ui::degradado::mezclar(color::ESMERALDA_DESDE, color::ESMERALDA_HASTA, 0.5)
}

fn renglon(ui: &mut Ui, contenido: &str, tam: Tam, peso: Peso, color_texto: Color32, alto: f32) {
    let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    ui.painter().text(Pos2::new(caja.left(), caja.center().y), egui::Align2::LEFT_CENTER, contenido, fuente(tam, peso), color_texto);
}

fn parrafo(ui: &mut Ui, contenido: String, tam: Tam, peso: Peso, color_texto: Color32) {
    let texto = Texto::new(contenido, tam, peso, color_texto);
    let pintor = ui.painter().clone();
    let galeria = texto.galeria(&pintor, ui.available_width());
    let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), texto.alto(&galeria)), Sense::hover());
    texto.pintar(&pintor, &galeria, caja.min);
}

fn encabezado(ui: &mut Ui, titulo: &str, descripcion: String) {
    renglon(ui, titulo, Tam::X2, Peso::Negrita, color_titulo(), 32.0);
    ui.add_space(6.0);
    parrafo(ui, descripcion, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO);
}

fn etiqueta_campo(ui: &mut Ui, contenido: &str) {
    renglon(ui, contenido, Tam::Sm, Peso::Medio, paleta::GRAY_700, 24.0);
    ui.add_space(4.0);
}

pub fn selector_nativo(ui: &mut Ui, id: Id, seleccion: &mut usize, opciones: &[String]) -> Response {
    let ancho = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 40.0), Sense::hover());
    let mut respuesta = ui.interact(rect, id, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    if respuesta.has_focus() {
        ui.painter().rect_stroke(rect, ui::RADIO_MD, Stroke::new(2.0, color::ANILLO), StrokeKind::Outside);
    }
    ui.painter().rect(rect, ui::RADIO_MD, color::FONDO, borde(color::BORDE), StrokeKind::Inside);
    if let Some(opcion) = opciones.get(*seleccion) {
        ui.painter().text(Pos2::new(rect.left() + 17.0, rect.center().y), egui::Align2::LEFT_CENTER, opcion, fuente(Tam::Sm, Peso::Normal), color::TEXTO);
    }
    let flecha = Rect::from_center_size(Pos2::new(rect.right() - 16.0, rect.center().y), Vec2::splat(14.0));
    ui::pintar_icono_en(ui, flecha, "chevron-down", color::TEXTO);
    let popup = id.with("opciones");
    if respuesta.clicked() {
        Popup::toggle_id(ui.ctx(), popup);
    }
    let mut elegido = None;
    Popup::new(popup, ui.ctx().clone(), rect, ui.layer_id())
        .open_memory(None)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .align(RectAlign::BOTTOM_START)
        .gap(4.0)
        .width(ancho)
        .frame(ui::marco_popup())
        .show(|ui| {
            ui.set_width(ancho - 8.0);
            for (indice, opcion) in opciones.iter().enumerate() {
                let (caja, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
                if r.hovered() || indice == *seleccion {
                    ui.painter().rect_filled(caja, ui::RADIO_SM, color::ACENTO);
                }
                ui.painter().text(Pos2::new(caja.left() + 12.0, caja.center().y), egui::Align2::LEFT_CENTER, opcion, fuente(Tam::Sm, Peso::Normal), color::TEXTO);
                if r.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    elegido = Some(indice);
                }
            }
        });
    if let Some(indice) = elegido {
        if indice != *seleccion {
            *seleccion = indice;
            respuesta.mark_changed();
        }
    }
    respuesta
}

pub fn entrada_numero(ui: &mut Ui, texto: &mut String, id: Id, sugerencia: &str) -> Response {
    let respuesta = Entrada::new(texto).id(id).tam(Tam::Sm).sugerencia(sugerencia).ui(ui);
    texto.retain(|c| c.is_ascii_digit() || c == '.' || c == '-');
    let caja = respuesta.rect.expand2(Vec2::new(12.0, 0.0));
    let encima = ui.rect_contains_pointer(caja);
    if respuesta.has_focus() || encima {
        let centro = Pos2::new(caja.right() - 20.0, caja.center().y);
        let gris = Color32::from_gray(110);
        let arriba = [centro + Vec2::new(-4.0, -2.0), centro + Vec2::new(4.0, -2.0), centro + Vec2::new(0.0, -8.0)];
        let abajo = [centro + Vec2::new(-4.0, 2.0), centro + Vec2::new(4.0, 2.0), centro + Vec2::new(0.0, 8.0)];
        ui.painter().add(egui::Shape::convex_polygon(arriba.to_vec(), gris, Stroke::NONE));
        ui.painter().add(egui::Shape::convex_polygon(abajo.to_vec(), gris, Stroke::NONE));
        let zona_arriba = Rect::from_min_max(centro + Vec2::new(-6.0, -10.0), centro + Vec2::new(6.0, 0.0));
        let zona_abajo = Rect::from_min_max(centro + Vec2::new(-6.0, 0.0), centro + Vec2::new(6.0, 10.0));
        let actual = parse_float(texto);
        let actual = if actual.is_nan() { 0.0 } else { actual };
        if ui.interact(zona_arriba, id.with("subir"), Sense::click()).clicked() {
            *texto = numero_js(actual.floor() + 1.0);
            ui.memory_mut(|m| m.request_focus(id));
        }
        if ui.interact(zona_abajo, id.with("bajar"), Sense::click()).clicked() {
            *texto = numero_js((actual.ceil() - 1.0).max(0.0));
            ui.memory_mut(|m| m.request_focus(id));
        }
    }
    respuesta
}

fn interruptor(ui: &mut Ui, activo: &mut bool) -> Response {
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(44.0, 24.0), Sense::click());
    if respuesta.clicked() {
        *activo = !*activo;
        respuesta.mark_changed();
    }
    let avance = ui.ctx().animate_bool_with_time(respuesta.id, *activo, 0.15);
    ui.painter().rect_filled(rect, 12, ui::degradado::mezclar(color::BORDE, paleta::EMERALD_700, avance));
    let perilla = Rect::from_center_size(Pos2::new(rect.left() + 12.0 + avance * 20.0, rect.center().y), Vec2::splat(20.0));
    ui.painter().add(ui::sombra_lg().as_shape(perilla, 10));
    ui.painter().rect_filled(perilla, 10, color::FONDO);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

fn botonera<R>(ui: &mut Ui, botones: impl FnOnce(&mut Ui) -> R) -> R {
    let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::hover());
    let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(caja).layout(egui::Layout::right_to_left(egui::Align::Center)));
    hijo.spacing_mut().item_spacing.x = 8.0;
    botones(&mut hijo)
}

fn pie(ui: &mut Ui, puede_agregar: bool, agregando: bool) -> (bool, bool) {
    botonera(ui, |ui| {
        let texto = if agregando { "Agregando..." } else { "Agregar al carrito" };
        let agregar = Boton::new(texto).variante(Variante::Esmeralda).texto_tam(Tam::Base).habilitado(puede_agregar).cargando(agregando).ui(ui).clicked();
        let cancelar = Boton::new("Cancelar").variante(Variante::Cancelar).texto_tam(Tam::Base).ui(ui).clicked();
        (agregar, cancelar)
    })
}

#[derive(Default)]
pub struct DialogoAgregar {
    producto: Option<ProductoDisponible>,
    cantidad: String,
    pub peso_manual: bool,
    filas: Vec<FilaLista>,
    tarea: Option<Tarea<Option<Vec<FilaLista>>>>,
    lista: Option<i64>,
    agregando_hasta: Option<f64>,
    enfocar: bool,
}

impl DialogoAgregar {
    pub fn abierto(&self) -> bool {
        self.producto.is_some()
    }

    pub fn abrir(&mut self, producto: ProductoDisponible, cx: &Contexto) {
        let api = cx.servicios.api.clone();
        let id = producto.id;
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || productos::listas_de_precios(&api, id)));
        self.producto = Some(producto);
        self.enfocar = true;
    }

    fn cerrar(&mut self) {
        let peso_manual = self.peso_manual;
        *self = DialogoAgregar { peso_manual, ..Default::default() };
    }

    fn fila(&self) -> Option<&FilaLista> {
        self.lista.and_then(|id| self.filas.iter().find(|f| f.lista_precio_id == id))
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) -> Salida {
        let Some(producto) = self.producto.clone() else {
            return Salida::Nada;
        };
        let negocio = cx.global.negocio.clone();
        let negocio = negocio.as_ref();
        if let Some(resultado) = tareas::esperar(&mut self.tarea) {
            self.filas = resultado.unwrap_or_default();
            let por_defecto = negocio.and_then(|n| n["listaPrecioPorDefectoId"].as_i64());
            self.lista = por_defecto.filter(|id| self.filas.iter().any(|f| f.lista_precio_id == *id));
        }
        let ahora = ui.input(|i| i.time);
        if self.agregando_hasta.is_some_and(|hasta| ahora >= hasta) {
            self.agregando_hasta = None;
        }
        let agregando = self.agregando_hasta.is_some();
        let cargando = self.tarea.is_some();
        let etiqueta = etiqueta_base(negocio);
        let fila = self.fila().cloned();
        let precio = fila.as_ref().map_or(producto.precio, |f| f.precio);
        let medida = match &fila {
            Some(f) => f.tipo_medida.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| producto.unidad_o_defecto()),
            None => producto.unidad_o_defecto(),
        };
        let es_peso = medida == "Kg";
        let visual = |unitario: f64| precio_visual(unitario, producto.iva_incluido, producto.iva_porcentaje, negocio);
        let sufijo = match (&self.lista, &fila) {
            (None, _) => format!(" — {etiqueta}"),
            (Some(_), Some(f)) => format!(" — {}", f.nombre_lista),
            (Some(_), None) => String::new(),
        };
        let descripcion = format!("{}{sufijo} — ${} / {medida}", producto.nombre, formato_local(visual(precio)));
        let permiso_manual = cx.global.usuario.as_ref().is_some_and(|u| u["permisos"]["pesoManualEnabled"] == Value::Bool(true));
        let id_cantidad = Id::new("agregar-producto-cantidad");
        let peso = cx.global.peso.clone();
        let mut agregar = false;
        let respuesta = Dialogo::new("agregar-producto").mostrar(ui.ctx(), |ui| {
            encabezado(ui, "Agregar producto", descripcion);
            if cargando {
                ui.add_space(16.0);
                let (caja, _) = ui.allocate_exact_size(Vec2::new(144.0, 16.0), Sense::hover());
                ui.painter().rect_filled(caja, ui::RADIO_MD, color::APAGADO);
                ui.add_space(8.0);
                let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::hover());
                ui.painter().rect_filled(caja, ui::RADIO_MD, color::APAGADO);
            }
            if !cargando && !self.filas.is_empty() {
                ui.add_space(16.0);
                etiqueta_campo(ui, "Lista de precios");
                let mut opciones = vec![etiqueta.clone()];
                opciones.extend(self.filas.iter().map(|f| format!("{} — ${}", f.nombre_lista, formato_local(visual(f.precio)))));
                let mut indice = self.lista.and_then(|id| self.filas.iter().position(|f| f.lista_precio_id == id)).map_or(0, |i| i + 1);
                if selector_nativo(ui, Id::new("agregar-producto-lista"), &mut indice, &opciones).changed() {
                    self.lista = indice.checked_sub(1).map(|i| self.filas[i].lista_precio_id);
                }
            }
            if es_peso {
                ui.add_space(16.0);
                if permiso_manual {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        interruptor(ui, &mut self.peso_manual);
                        ui.add_space(16.0);
                        let (caja, _) = ui.allocate_exact_size(Vec2::new(120.0, 24.0), Sense::hover());
                        ui.painter().text(Pos2::new(caja.left(), caja.center().y), egui::Align2::LEFT_CENTER, "Peso manual", fuente(Tam::Sm, Peso::Medio), color::TEXTO);
                    });
                    ui.add_space(8.0);
                } else {
                    ui.add_space(8.0);
                    renglon(ui, "Modo de peso automático", Tam::Sm, Peso::Normal, paleta::GRAY_500, 20.0);
                    ui.add_space(8.0);
                }
            }
            ui.add_space(16.0);
            if !es_peso || self.peso_manual {
                let sugerencia = if es_peso { "Ingrese el peso en gramos" } else { "Ingrese las unidades" };
                entrada_numero(ui, &mut self.cantidad, id_cantidad, sugerencia);
                if self.enfocar {
                    ui.memory_mut(|m| m.request_focus(id_cantidad));
                }
            } else {
                ui.add_space(8.0);
                renglon(ui, &format!("{} g", texto_peso(&peso)), Tam::X3, Peso::Negrita, color::TEXTO, 36.0);
                renglon(ui, "Peso de la balanza", Tam::Sm, Peso::Normal, paleta::GRAY_500, 20.0);
                ui.add_space(8.0);
            }
            self.enfocar = false;
            ui.add_space(16.0);
            let (clic_agregar, clic_cancelar) = pie(ui, !agregando && !cargando, agregando);
            agregar = clic_agregar;
            clic_cancelar
        });
        if respuesta.interior || respuesta.cerrar {
            self.cerrar();
            return Salida::Cerrar;
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        if enter && !agregando && !cargando && ((es_peso && !self.peso_manual) || !self.cantidad.is_empty()) {
            agregar = true;
        }
        if !agregar || agregando || cargando {
            return Salida::Nada;
        }
        self.agregando_hasta = Some(ahora + 0.5);
        let cantidad = if es_peso && !self.peso_manual {
            Ok(productos::gramos_de_peso(&peso) / 1000.0)
        } else if self.cantidad.is_empty() {
            Err("Se requiere especificar una cantidad para este producto")
        } else if es_peso {
            Ok(parse_float(&self.cantidad) / 1000.0)
        } else {
            Ok(parse_float(&self.cantidad))
        };
        let cantidad = cantidad.and_then(|c| if c.is_nan() || c <= 0.0 { Err("La cantidad debe ser un número mayor que cero") } else { Ok(c) });
        match cantidad {
            Ok(cantidad) => {
                let lista = self.lista.map(|id| (id, fila.as_ref().map(|f| f.nombre_lista.clone())));
                let item = producto.item(cantidad, medida, precio, lista, &etiqueta);
                self.cerrar();
                Salida::Agregar(item)
            }
            Err(mensaje) => {
                ui::toast::error(&cx.egui, mensaje);
                self.enfocar = true;
                Salida::Nada
            }
        }
    }
}

#[derive(Default)]
pub struct DialogoEscanear {
    datos: Option<(ProductoDisponible, Vec<FilaLista>, f64)>,
    lista: Option<i64>,
    cantidad: String,
    era_peso: Option<bool>,
    enfocar: bool,
}

impl DialogoEscanear {
    pub fn abierto(&self) -> bool {
        self.datos.is_some()
    }

    pub fn abrir(&mut self, producto: ProductoDisponible, filas: Vec<FilaLista>, cantidad: f64, negocio: Option<&Value>) {
        let por_defecto = negocio.and_then(|n| n["listaPrecioPorDefectoId"].as_i64());
        self.lista = por_defecto
            .filter(|id| filas.iter().any(|f| f.lista_precio_id == *id))
            .or(filas.first().map(|f| f.lista_precio_id));
        self.datos = Some((producto, filas, cantidad));
        self.cantidad.clear();
        self.era_peso = None;
        self.enfocar = true;
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) -> Salida {
        let Some((producto, filas, cantidad_inicial)) = self.datos.clone() else {
            return Salida::Nada;
        };
        let negocio = cx.global.negocio.clone();
        let negocio = negocio.as_ref();
        let etiqueta = etiqueta_base(negocio);
        let medida_producto = producto.unidad_o_defecto();
        let fila = self.lista.and_then(|id| filas.iter().find(|f| f.lista_precio_id == id)).cloned();
        let precio = fila.as_ref().map_or(producto.precio, |f| f.precio);
        let medida = fila.as_ref().and_then(|f| f.tipo_medida.clone()).filter(|t| !t.is_empty()).unwrap_or_else(|| medida_producto.clone());
        let es_peso = medida == "Kg";
        if self.era_peso != Some(es_peso) {
            let base = if cantidad_inicial > 0.0 { cantidad_inicial } else { 1.0 };
            self.cantidad = if es_peso { numero_js((base * 1000.0).round()) } else { numero_js(base) };
            self.era_peso = Some(es_peso);
        }
        let visual = |unitario: f64| precio_visual(unitario, producto.iva_incluido, producto.iva_porcentaje, negocio);
        let cantidad_final = {
            let n = parse_float(&self.cantidad);
            if n.is_nan() || n <= 0.0 { 0.0 } else if es_peso { n / 1000.0 } else { n }
        };
        let descripcion = format!("{} — ${} / {medida}", producto.nombre, formato_local(visual(precio)));
        let mut opciones = vec![format!("{etiqueta} — ${} / {medida_producto}", formato_local(visual(producto.precio)))];
        opciones.extend(filas.iter().map(|f| {
            let medida_fila = f.tipo_medida.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| medida_producto.clone());
            format!("{} — ${} / {medida_fila}", f.nombre_lista, formato_local(visual(f.precio)))
        }));
        let id_selector = Id::new("escanear-lista-selector");
        let mut confirmar = false;
        let respuesta = Dialogo::new("escanear-lista").mostrar(ui.ctx(), |ui| {
            encabezado(ui, "Elegí la lista de precios", descripcion);
            ui.add_space(16.0);
            etiqueta_campo(ui, "Lista de precios");
            let mut indice = self.lista.and_then(|id| filas.iter().position(|f| f.lista_precio_id == id)).map_or(0, |i| i + 1);
            if selector_nativo(ui, id_selector, &mut indice, &opciones).changed() {
                self.lista = indice.checked_sub(1).map(|i| filas[i].lista_precio_id);
            }
            if self.enfocar {
                ui.memory_mut(|m| m.request_focus(id_selector));
                self.enfocar = false;
            }
            ui.add_space(16.0);
            etiqueta_campo(ui, if es_peso { "Ingrese el peso en gramos" } else { "Ingrese las unidades" });
            entrada_numero(ui, &mut self.cantidad, Id::new("escanear-lista-cantidad"), if es_peso { "Gramos" } else { "Unidades" });
            ui.add_space(16.0);
            let (agregar, cancelar) = pie(ui, cantidad_final > 0.0, false);
            confirmar = agregar;
            cancelar
        });
        if respuesta.interior || respuesta.cerrar {
            self.datos = None;
            return Salida::Cerrar;
        }
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            confirmar = true;
        }
        if !confirmar || cantidad_final <= 0.0 {
            return Salida::Nada;
        }
        let lista = fila.as_ref().map(|f| (f.lista_precio_id, Some(f.nombre_lista.clone())));
        let item = producto.item(cantidad_final, medida, precio, lista, &etiqueta);
        self.datos = None;
        Salida::Agregar(item)
    }
}

pub fn confirmar_borrado(ui: &mut Ui) -> Option<bool> {
    let respuesta = Dialogo::new("confirmar-borrar-orden").ancho(420.0).mostrar(ui.ctx(), |ui| {
        parrafo(ui, "¿Seguro que deseas eliminar esta pantalla?".into(), Tam::Base, Peso::Normal, color::TEXTO);
        ui.add_space(24.0);
        botonera(ui, |ui| {
            let aceptar = Boton::new("Aceptar").variante(Variante::Esmeralda).ui(ui).clicked();
            let cancelar = Boton::new("Cancelar").variante(Variante::Contorno).ui(ui).clicked();
            (aceptar, cancelar)
        })
    });
    let (aceptar, cancelar) = respuesta.interior;
    if aceptar || ui.input(|i| i.key_pressed(Key::Enter)) {
        Some(true)
    } else if cancelar || respuesta.cerrar {
        Some(false)
    } else {
        None
    }
}
