use eframe::egui::{self, Align, CursorIcon, Id, Key, Layout, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use nucleo::carrito::Producto;
use serde_json::Value;

use super::diseno::galeria_cortada;
use super::productos::{self, monto, numero_js, parse_float, FilaLista, ProductoDisponible};
use super::vista::{kilos, pesos, precio_visual};
use crate::app::Contexto;
use crate::tareas::{self, Tarea};
use crate::ui::degradado::mezclar;
use crate::ui::marca::{color, fuente, radio, sombra, tam};
use crate::ui::v2::{self, toast, Boton, Contenedor, Modal};
use crate::ui::{pintar_icono_solar_en, Peso};

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

fn etiqueta_campo(ui: &mut Ui, contenido: &str) {
    v2::texto(ui, contenido, tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO);
}

fn solo_numeros(texto: &mut String) {
    texto.retain(|c| c.is_ascii_digit() || c == '.' || c == '-');
}

fn valor_actual(texto: &str) -> f64 {
    let actual = parse_float(texto);
    if actual.is_nan() {
        0.0
    } else {
        actual
    }
}

fn tarjetas_listas(ui: &mut Ui, opciones: &[(String, String)], elegida: usize) -> Option<usize> {
    let ancho = ui.available_width();
    let columnas = (((ancho + 8.0) / 158.0).floor() as usize).clamp(1, opciones.len().max(1));
    let ancho_tarjeta = (ancho - 8.0 * (columnas - 1) as f32) / columnas as f32;
    let mut elegido = None;
    for (numero_fila, grupo) in opciones.chunks(columnas).enumerate() {
        if numero_fila > 0 {
            ui.add_space(8.0);
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (columna, (nombre, precio)) in grupo.iter().enumerate() {
                let indice = numero_fila * columnas + columna;
                let activa = indice == elegida;
                let respuesta = v2::opcion_eleccion(ui, Vec2::new(ancho_tarjeta, 60.0), radio::CAMPO, activa, 1.5, |ui, rect, _| {
                    let pintor = ui.painter();
                    let tinta_nombre = if activa { color::VERDE } else { color::TEXTO_APAGADO };
                    let nombre = galeria_cortada(pintor, nombre, fuente(tam::ETIQUETA, Peso::Medio), tinta_nombre, rect.width() - 27.0);
                    let precio = galeria_cortada(pintor, precio, fuente(tam::DESTACADO, Peso::Semi), color::TEXTO, rect.width() - 27.0);
                    let x = rect.left() + 13.5;
                    let alto_nombre = nombre.size().y;
                    pintor.galley(Pos2::new(x, rect.top() + 11.5), nombre, tinta_nombre);
                    pintor.galley(Pos2::new(x, rect.top() + 11.5 + alto_nombre + 2.0), precio, color::TEXTO);
                });
                if respuesta.clicked() {
                    elegido = Some(indice);
                }
            }
        });
    }
    elegido
}

fn interruptor(ui: &mut Ui, activo: &mut bool, etiqueta: &str) {
    let texto = v2::galeria(ui, etiqueta, fuente(tam::CUERPO, Peso::Normal), color::TEXTO_SECUNDARIO);
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(36.0 + 10.0 + texto.size().x, texto.size().y.max(20.0)), Sense::click());
    if respuesta.clicked() {
        *activo = !*activo;
    }
    let avance = ui.ctx().animate_bool_with_time(respuesta.id, *activo, 0.16);
    let pista = Rect::from_min_size(Pos2::new(rect.left(), rect.center().y - 10.0), Vec2::new(36.0, 20.0));
    let pintor = ui.painter();
    pintor.rect_filled(pista, radio::PASTILLA, mezclar(color::INTERRUPTOR_APAGADO, color::VERDE, avance));
    let perilla = Rect::from_min_size(Pos2::new(pista.left() + 2.0 + 16.0 * avance, pista.top() + 2.0), Vec2::splat(16.0));
    pintor.add(sombra::SEGMENTO.forma(perilla, radio::PASTILLA));
    pintor.rect_filled(perilla, radio::PASTILLA, color::BLANCO);
    pintor.galley(Pos2::new(pista.right() + 10.0, rect.center().y - texto.size().y / 2.0), texto, color::TEXTO_SECUNDARIO);
    respuesta.on_hover_cursor(CursorIcon::PointingHand);
}

fn tarjeta_peso(ui: &mut Ui, peso: &str, conectada: bool) {
    let ancho = ui.available_width();
    let estado = v2::galeria(ui, if conectada { "Peso estable" } else { "Balanza desconectada" }, fuente(tam::CHICO, Peso::Normal), color::SOBRE_OSCURO_APAGADO);
    let valor = v2::galeria_espaciada(ui, peso, fuente(44.0, Peso::Semi), color::BLANCO, -0.02 * 44.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 20.0 + estado.size().y + 6.0 + 44.0 + 20.0), Sense::hover());
    let pintor = ui.painter();
    pintor.rect_filled(rect, radio::TARJETA, color::VERDE_PROFUNDO);
    let centro_estado = rect.top() + 20.0 + estado.size().y / 2.0;
    pintor.circle_filled(Pos2::new(rect.left() + 22.0 + 3.5, centro_estado), 3.5, if conectada { color::VERDE_PUNTO } else { color::DORADO });
    let alto_estado = estado.size().y;
    pintor.galley(Pos2::new(rect.left() + 22.0 + 13.0, centro_estado - alto_estado / 2.0), estado, color::SOBRE_OSCURO_APAGADO);
    let arriba_valor = rect.top() + 20.0 + alto_estado + 6.0;
    pintor.galley(Pos2::new(rect.left() + 22.0, arriba_valor + (44.0 - valor.size().y) / 2.0), valor, color::BLANCO);
    let icono = Rect::from_min_size(Pos2::new(rect.right() - 22.0 - 34.0, rect.bottom() - 20.0 - 34.0), Vec2::splat(34.0));
    pintar_icono_solar_en(ui, icono, "scale", color::DORADO);
}

fn campo_gramos(ui: &mut Ui, texto: &mut String, id: Id) -> Response {
    let contenedor = Contenedor { alto: 52.0, separacion: 8.0, ..Default::default() };
    let respuesta = contenedor.mostrar(ui, ui.available_width(), |ui| {
        let ancho = ui.available_width() - 8.0 - v2::ancho_texto(ui, "g", tam::DESTACADO, Peso::Normal);
        let respuesta = ui
            .allocate_ui_with_layout(Vec2::new(ancho, 50.0), Layout::left_to_right(Align::Center), |ui| {
                v2::entrada_desnuda(texto).id(id).tam(tam::GRANDE).peso(Peso::Medio).sugerencia("Ej: 1250").ui(ui)
            })
            .inner;
        v2::texto(ui, "g", tam::DESTACADO, Peso::Normal, color::TEXTO_TENUE);
        respuesta
    });
    solo_numeros(texto);
    respuesta
}

fn boton_paso(ui: &mut Ui, simbolo: &str) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(52.0), Sense::click());
    let borde = if respuesta.hovered() { color::VERDE } else { color::BORDE };
    ui.painter().rect(rect, radio::CAMPO, color::BLANCO, Stroke::new(1.0, borde), StrokeKind::Inside);
    v2::pintar_texto(ui, rect.center(), egui::Align2::CENTER_CENTER, simbolo, 22.0, Peso::Normal, color::TEXTO_SECUNDARIO);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

fn selector_unidades(ui: &mut Ui, texto: &mut String, id: Id) {
    let ancho_campo = ui.available_width() - 52.0 * 2.0 - 16.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if boton_paso(ui, "−").clicked() {
            *texto = numero_js((valor_actual(texto).ceil() - 1.0).max(0.0));
            ui.memory_mut(|m| m.request_focus(id));
        }
        let contenedor = Contenedor { alto: 52.0, ..Default::default() };
        contenedor.mostrar(ui, ancho_campo, |ui| v2::entrada_desnuda(texto).id(id).tam(22.0).peso(Peso::Semi).centrada().ui(ui));
        if boton_paso(ui, "+").clicked() {
            *texto = numero_js(valor_actual(texto).floor() + 1.0);
            ui.memory_mut(|m| m.request_focus(id));
        }
    });
    solo_numeros(texto);
}

fn fila_subtotal(ui: &mut Ui, subtotal: Option<f64>) {
    let valor = v2::galeria(ui, subtotal.map_or_else(|| "—".to_string(), pesos), fuente(tam::GRANDE, Peso::Semi), color::TEXTO);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0 + valor.size().y), Sense::hover());
    ui.painter().rect_filled(rect, radio::CAMPO, color::FONDO_APP);
    v2::pintar_texto(ui, Pos2::new(rect.left() + 14.0, rect.center().y), egui::Align2::LEFT_CENTER, "Subtotal", tam::CUERPO, Peso::Normal, color::TEXTO_APAGADO);
    ui.painter().galley(Pos2::new(rect.right() - 14.0 - valor.size().x, rect.center().y - valor.size().y / 2.0), valor, color::TEXTO);
}

fn pie(ui: &mut Ui, habilitado: bool, agregando: bool) -> (bool, bool) {
    v2::pie_modal(ui, |ui| {
        let texto = if agregando { "Agregando..." } else { "Agregar a la orden" };
        let agregar = Boton::primario(texto)
            .icono("cart-plus")
            .color_icono(color::DORADO)
            .tam_icono(18.0)
            .tecla("Enter")
            .radio(radio::BOTON_MODAL)
            .sin_sombra()
            .habilitado(habilitado)
            .cargando(agregando)
            .ui(ui)
            .clicked();
        let cancelar = Boton::secundario("Cancelar").radio(radio::BOTON_MODAL).ui(ui).clicked();
        (agregar, cancelar)
    })
}

fn por_medida(medida: &str) -> &'static str {
    if medida == "Kg" {
        "por kilo"
    } else {
        "por unidad"
    }
}

fn subtitulo(producto: &ProductoDisponible, precio: f64, medida: &str) -> String {
    let precio = format!("{} {}", pesos(precio), por_medida(medida));
    match producto.plu.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        Some(plu) => format!("PLU {plu} · {precio}"),
        None => precio,
    }
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
        let permiso_manual = cx.global.usuario.as_ref().is_some_and(|u| u["permisos"]["pesoManualEnabled"] == Value::Bool(true));
        let id_cantidad = Id::new("agregar-producto-cantidad");
        let peso = cx.global.peso.clone();
        let conectada = cx.global.estado_balanza["conectada"] == Value::Bool(true);
        let cantidad_vista = if es_peso && !self.peso_manual {
            productos::gramos_de_peso(&peso) / 1000.0
        } else if es_peso {
            parse_float(&self.cantidad) / 1000.0
        } else {
            parse_float(&self.cantidad)
        };
        let subtotal = (cantidad_vista > 0.0).then(|| visual(precio) * cantidad_vista);
        let mut opciones = vec![(etiqueta.clone(), pesos(visual(producto.precio)))];
        opciones.extend(self.filas.iter().map(|f| (f.nombre_lista.clone(), pesos(visual(f.precio)))));
        let elegida = self.lista.and_then(|id| self.filas.iter().position(|f| f.lista_precio_id == id)).map_or(0, |i| i + 1);
        let subtitulo = subtitulo(&producto, visual(precio), &medida);
        let mut agregar = false;
        let respuesta = Modal::new("agregar-producto", &producto.nombre).subtitulo(&subtitulo).icono("cart-plus").ancho(520.0).mostrar(ui.ctx(), |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                if cargando {
                    v2::shimmer(ui, Vec2::new(120.0, 16.0), radio::CHIP, true);
                    ui.add_space(8.0);
                    v2::shimmer(ui, Vec2::new(ui.available_width(), 60.0), radio::CAMPO, true);
                    ui.add_space(16.0);
                } else if !self.filas.is_empty() {
                    etiqueta_campo(ui, "Lista de precios");
                    ui.add_space(8.0);
                    if let Some(indice) = tarjetas_listas(ui, &opciones, elegida) {
                        self.lista = indice.checked_sub(1).map(|i| self.filas[i].lista_precio_id);
                    }
                    ui.add_space(16.0);
                }
                if es_peso && permiso_manual {
                    interruptor(ui, &mut self.peso_manual, "Ingresar peso manualmente");
                    ui.add_space(16.0);
                }
                if es_peso && !self.peso_manual {
                    tarjeta_peso(ui, &kilos(productos::gramos_de_peso(&peso) / 1000.0), conectada);
                } else {
                    etiqueta_campo(ui, if es_peso { "Peso en gramos" } else { "Unidades" });
                    ui.add_space(6.0);
                    if es_peso {
                        campo_gramos(ui, &mut self.cantidad, id_cantidad);
                    } else {
                        selector_unidades(ui, &mut self.cantidad, id_cantidad);
                    }
                    if self.enfocar {
                        ui.memory_mut(|m| m.request_focus(id_cantidad));
                    }
                }
                self.enfocar = false;
                ui.add_space(16.0);
                fila_subtotal(ui, subtotal);
            });
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
                toast::error(&cx.egui, mensaje);
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
            if n.is_nan() || n <= 0.0 {
                0.0
            } else if es_peso {
                n / 1000.0
            } else {
                n
            }
        };
        let unidad = |m: &str| if m == "Kg" { " /kg" } else { " /u" };
        let mut opciones = vec![(etiqueta.clone(), format!("{}{}", pesos(visual(producto.precio)), unidad(&medida_producto)))];
        opciones.extend(filas.iter().map(|f| {
            let medida_fila = f.tipo_medida.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| medida_producto.clone());
            (f.nombre_lista.clone(), format!("${}{}", monto(visual(f.precio)), unidad(&medida_fila)))
        }));
        let elegida = self.lista.and_then(|id| filas.iter().position(|f| f.lista_precio_id == id)).map_or(0, |i| i + 1);
        let subtitulo = format!("{} · {}", producto.nombre, subtitulo(&producto, visual(precio), &medida));
        let subtotal = (cantidad_final > 0.0).then(|| visual(precio) * cantidad_final);
        let mut confirmar = false;
        let respuesta = Modal::new("escanear-lista", "Elegí la lista de precios").subtitulo(&subtitulo).icono("tag").ancho(520.0).mostrar(ui.ctx(), |ui| {
            v2::cuerpo_modal(ui, 20, |ui| {
                etiqueta_campo(ui, "Lista de precios");
                ui.add_space(8.0);
                if let Some(indice) = tarjetas_listas(ui, &opciones, elegida) {
                    self.lista = indice.checked_sub(1).map(|i| filas[i].lista_precio_id);
                }
                ui.add_space(16.0);
                etiqueta_campo(ui, if es_peso { "Peso en gramos" } else { "Unidades" });
                ui.add_space(6.0);
                let id_cantidad = Id::new("escanear-lista-cantidad");
                if es_peso {
                    campo_gramos(ui, &mut self.cantidad, id_cantidad);
                } else {
                    selector_unidades(ui, &mut self.cantidad, id_cantidad);
                }
                ui.add_space(16.0);
                fila_subtotal(ui, subtotal);
            });
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

pub fn confirmar_borrado(ui: &mut Ui, numero: usize, productos: usize) -> Option<bool> {
    let plural = if productos == 1 { "" } else { "s" };
    let titulo = format!("¿Cerrar la Orden {numero}?");
    let subtitulo = format!("Tiene {productos} producto{plural} cargado{plural}");
    let respuesta = Modal::new("confirmar-borrar-orden", &titulo).subtitulo(&subtitulo).icono("x").peligro().ancho(440.0).mostrar(ui.ctx(), |ui| {
        v2::cuerpo_modal(ui, 16, |ui| {
            let ancho = ui.available_width();
            v2::parrafo(ui, "Los productos de esta orden se descartan.", tam::CUERPO, color::TEXTO_SECUNDARIO, 1.55, ancho);
        });
        v2::pie_modal(ui, |ui| {
            let aceptar = Boton::peligro("Cerrar orden").tecla("Enter").modal().ui(ui).clicked();
            let cancelar = Boton::secundario("Volver").modal().ui(ui).clicked();
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
