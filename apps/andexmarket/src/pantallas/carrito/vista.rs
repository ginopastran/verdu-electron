use std::sync::Arc;

use chrono::{DateTime, Utc};
use eframe::egui::{
    Align, Color32, Context, CornerRadius, CursorIcon, Galley, Id, Layout, Painter, Pos2, Rect, Response, ScrollArea, Sense, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2,
};
use nucleo::busqueda::Resultado;
use nucleo::carrito::{Carritos, Producto};
use nucleo::formato;
use nucleo::iva::precio_visual_con_iva;
use serde_json::Value;

use super::diseno::{borde_punteado, celdas, galeria_centrada, galeria_cortada, Pista};
use super::fotos::{self, Fotos};
use super::productos::{monto, ProductoDisponible};
use crate::ui::marca::{self, color, fuente, fuente_mono, mezclar, radio, sombra, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, Avatar, Boton, BotonIcono, Estilo, EstiloTecla, ItemMenu, Popover, Usuario};
use crate::ui::{pintar_icono_solar_en, Peso};

pub const ALTO_CABECERA: f32 = 68.0;
pub const ANCHO_PANEL: f32 = 380.0;
const ANCHO_CABECERA_AMPLIA: f32 = 1240.0;
const ANCHO_MARCA: f32 = 26.0 + 10.0 + 112.0;
const ANCHO_BUSCADOR_MINIMO: f32 = 280.0;
const ANCHO_BUSCADOR_MAXIMO: f32 = 680.0;

fn galeria(ui: &Ui, contenido: impl Into<String>, px: f32, peso: Peso, tinta: Color32) -> Arc<Galley> {
    v2::galeria(ui, contenido, fuente(px, peso), tinta)
}

fn clic(ui: &Ui, rect: Rect, id: Id) -> Response {
    ui.interact(rect, id, Sense::click()).on_hover_cursor(CursorIcon::PointingHand)
}

pub fn precio_visual(precio: f64, iva_incluido: Option<bool>, iva_porcentaje: Option<f64>, negocio: Option<&Value>) -> f64 {
    match negocio {
        Some(n) => precio_visual_con_iva(precio, iva_incluido.unwrap_or(false), iva_porcentaje, n["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false)),
        None => precio,
    }
}

pub fn pesos(valor: f64) -> String {
    format!("${}", monto(valor))
}

pub fn texto_oferta(porcentaje: f64) -> String {
    format!("−{} %", formato::numero(porcentaje))
}

fn insignia_oferta(ui: &Ui, pintor: &Painter, derecha: f32, centro_y: f32, porcentaje: f64) -> f32 {
    let texto = galeria(ui, texto_oferta(porcentaje), tam::MINI, Peso::Semi, color::VERDE_EXITO);
    let caja = Rect::from_min_max(
        Pos2::new(derecha - texto.size().x - 12.0, centro_y - texto.size().y / 2.0 - 1.0),
        Pos2::new(derecha, centro_y + texto.size().y / 2.0 + 1.0),
    );
    pintor.rect_filled(caja, radio::CHIP, color::VERDE_EXITO_FONDO);
    pintor.galley(caja.center() - texto.size() / 2.0, texto, color::VERDE_EXITO);
    caja.left()
}

fn tachado(ui: &Ui, pintor: &Painter, derecha: f32, linea_base: f32, contenido: String) -> f32 {
    let g = galeria(ui, contenido, tam::XS, Peso::Normal, color::TEXTO_TENUE);
    let arriba = linea_base - g.size().y * 0.8;
    let izquierda = derecha - g.size().x;
    let medio = arriba + g.size().y * 0.55;
    pintor.galley(Pos2::new(izquierda, arriba), g, color::TEXTO_TENUE);
    pintor.hline(izquierda..=derecha, medio, Stroke::new(1.0, color::TEXTO_TENUE));
    izquierda
}

pub fn kilos(valor: f64) -> String {
    format!("{} kg", formato::numero_con(valor, 3, 3))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionCabecera {
    Ordenes,
    CuentaCorriente,
    Cierre,
    Impresion,
    Balanza,
    CerrarSesion,
}

pub struct Cabecera<'a> {
    pub facturacion: bool,
    pub cierre: bool,
    pub nombre: &'a str,
    pub email: &'a str,
    pub reloj: &'a str,
}

pub struct Disposicion {
    pub barra: Rect,
    pub buscador: Rect,
    derecha: Rect,
    amplia: bool,
}

impl Cabecera<'_> {
    fn botones(&self) -> Vec<(&'static str, &'static str, AccionCabecera)> {
        let mut botones = vec![("history", "Órdenes", AccionCabecera::Ordenes)];
        if self.facturacion {
            botones.push(("file-text", "Cuenta corriente", AccionCabecera::CuentaCorriente));
        }
        if self.cierre {
            botones.push(("store", "Cierre de caja", AccionCabecera::Cierre));
        }
        botones
    }

    fn ancho_derecha(&self, ui: &Ui, amplia: bool) -> f32 {
        let botones = self.botones();
        let ancho_botones: f32 = botones
            .iter()
            .map(|(_, etiqueta, _)| if amplia { 10.0 + 20.0 + 8.0 + v2::ancho_texto(ui, etiqueta, tam::CONTROL, Peso::Medio) + 10.0 } else { 40.0 })
            .sum::<f32>()
            + 6.0 * botones.len().saturating_sub(1) as f32;
        let textos = if amplia {
            v2::ancho_texto(ui, self.nombre, tam::CONTROL, Peso::Medio).max(v2::ancho_texto(ui, self.reloj, tam::MINI, Peso::Normal)) + 10.0
        } else {
            0.0
        };
        ancho_botones + 12.0 + 1.0 + 12.0 + 6.0 + 32.0 + 10.0 + textos + 16.0 + 10.0
    }

    pub fn disponer(&self, ui: &Ui, pantalla: Rect) -> Disposicion {
        let barra = Rect::from_min_size(pantalla.min, Vec2::new(pantalla.width(), ALTO_CABECERA));
        let interior = barra.shrink2(Vec2::new(20.0, 0.0));
        let disponible = interior.width() - ANCHO_MARCA - 40.0;
        let amplia = barra.width() >= ANCHO_CABECERA_AMPLIA && disponible - self.ancho_derecha(ui, true) >= ANCHO_BUSCADOR_MINIMO;
        let ancho_derecha = self.ancho_derecha(ui, amplia);
        let ancho_buscador = (disponible - ancho_derecha).clamp(0.0, ANCHO_BUSCADOR_MAXIMO);
        let buscador = Rect::from_min_size(Pos2::new(interior.left() + ANCHO_MARCA + 20.0, barra.center().y - 23.0), Vec2::new(ancho_buscador, 46.0));
        let derecha = Rect::from_min_max(Pos2::new(interior.right() - ancho_derecha, barra.top()), Pos2::new(interior.right(), barra.bottom()));
        Disposicion { barra, buscador, derecha, amplia }
    }

    pub fn pintar_fondo(ui: &mut Ui, disposicion: &Disposicion) {
        let barra = disposicion.barra;
        ui.painter().rect_filled(barra, 0, color::VERDE_PROFUNDO);
        let marca = Rect::from_min_size(Pos2::new(barra.left() + 20.0, barra.top()), Vec2::new(ANCHO_MARCA, barra.height()));
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(marca).layout(Layout::left_to_right(Align::Center)));
        hijo.spacing_mut().item_spacing.x = 10.0;
        v2::logo(&mut hijo, 26.0);
        v2::logo_texto(&mut hijo, 112.0);
    }

    pub fn acciones(&self, ui: &mut Ui, disposicion: &Disposicion) -> Option<AccionCabecera> {
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(disposicion.derecha).layout(Layout::right_to_left(Align::Center)));
        hijo.spacing_mut().item_spacing.x = 6.0;
        let items = [
            ItemMenu { icono: "printer", etiqueta: "Impresión", tecla: None, peligro: false },
            ItemMenu { icono: "scale", etiqueta: "Balanza", tecla: None, peligro: false },
            ItemMenu { icono: "log-out", etiqueta: "Cerrar sesión", tecla: Some("F4"), peligro: true },
        ];
        let usuario = Usuario { nombre: self.nombre, email: self.email, detalle: self.reloj };
        let mut accion = match v2::menu_usuario(&mut hijo, "menu-usuario-carrito", &usuario, &items, !disposicion.amplia) {
            Some(0) => Some(AccionCabecera::Impresion),
            Some(1) => Some(AccionCabecera::Balanza),
            Some(2) => Some(AccionCabecera::CerrarSesion),
            _ => None,
        };
        hijo.add_space(6.0);
        v2::separador_vertical(&mut hijo, 28.0, color::VERDE);
        hijo.add_space(6.0);
        for (icono, etiqueta, a) in self.botones().into_iter().rev() {
            let texto = if disposicion.amplia { etiqueta } else { "" };
            let respuesta = Boton::new(texto, Estilo::FantasmaOscuro).icono(icono).tam_icono(20.0).alto(40.0).relleno(10.0).radio(radio::CONTROL).tam_texto(tam::CONTROL).ui(&mut hijo);
            let respuesta = if disposicion.amplia { respuesta } else { respuesta.on_hover_text(etiqueta) };
            if respuesta.clicked() {
                accion = Some(a);
            }
        }
        accion
    }
}

pub struct Buscador {
    pub caja: Response,
    pub campo: Response,
    pub limpiar: bool,
}

pub fn buscador(ui: &mut Ui, rect: Rect, consulta: &mut String, id: Id) -> Buscador {
    let caja = ui.interact(rect, id.with("caja"), Sense::hover());
    let con_texto = !consulta.is_empty();
    if con_texto {
        marca::pintar_anillo(ui.painter(), rect, radio::CAMPO, color::ANILLO_BUSQUEDA);
    }
    ui.painter().rect_filled(rect, radio::CAMPO, color::BLANCO);
    let interior = Rect::from_min_max(Pos2::new(rect.left() + 14.0, rect.top()), Pos2::new(rect.right() - 6.0, rect.bottom()));
    let mut hijo = ui.new_child(UiBuilder::new().max_rect(interior).layout(Layout::left_to_right(Align::Center)));
    hijo.spacing_mut().item_spacing.x = 10.0;
    let (icono, _) = hijo.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
    pintar_icono_solar_en(&hijo, icono, "barcode", color::VERDE);
    let ancho = (hijo.available_width() - if con_texto { 42.0 } else { 0.0 }).max(10.0);
    let campo = hijo
        .allocate_ui_with_layout(Vec2::new(ancho, 44.0), Layout::left_to_right(Align::Center), |ui| {
            v2::entrada_desnuda(consulta).id(id).tam(tam::BUSQUEDA).sugerencia("Buscar producto, PLU o escanear código").ui(ui)
        })
        .inner;
    let limpiar = con_texto && BotonIcono::new("x").tam(32.0).tam_icono(18.0).colores(color::TEXTO_TENUE, color::FONDO_APP, color::TEXTO_TENUE).ui(&mut hijo).clicked();
    Buscador { caja, campo, limpiar }
}

pub struct Resultados<'a> {
    pub productos: &'a [ProductoDisponible],
    pub resultados: &'a [Resultado],
    pub otras_medidas: bool,
    pub no_encontrado: bool,
    pub seleccion: Option<usize>,
    pub mostrar_seleccion: bool,
    pub consulta: &'a str,
    pub negocio: Option<&'a Value>,
    pub fotos: &'a Fotos,
    pub ahora: DateTime<Utc>,
}

pub fn resultados(ctx: &Context, ancla: &Response, visible: bool, datos: &Resultados) -> (Option<usize>, Vec<usize>) {
    Popover::new(Id::new("resultados-busqueda"), ancla)
        .visible(visible)
        .ancho(ancla.rect.width())
        .separacion(8.0)
        .radio(radio::POPOVER)
        .sombra(sombra::POPOVER)
        .alto_maximo(420.0)
        .mostrar(ctx, |ui| {
            let ancho = ui.available_width();
            let pintor = ui.painter().clone();
            let mut elegido = None;
            let mut sin_foto = Vec::new();
            if datos.otras_medidas {
                let aviso = galeria_cortada(&pintor, &format!("No hay productos con esa medida para “{}”", datos.consulta), fuente(tam::CHICO, Peso::Normal), color::TEXTO_APAGADO, ancho - 24.0);
                let titulo = galeria(ui, "Otras medidas", tam::XS, Peso::Semi, color::DORADO_TEXTO);
                let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, 14.0 + aviso.size().y + 6.0 + titulo.size().y + 8.0), Sense::hover());
                let alto_aviso = aviso.size().y;
                pintor.galley(Pos2::new(caja.left() + 12.0, caja.top() + 14.0), aviso, color::TEXTO_APAGADO);
                pintor.galley(Pos2::new(caja.left() + 12.0, caja.top() + 14.0 + alto_aviso + 6.0), titulo, color::DORADO_TEXTO);
            }
            for (posicion, resultado) in datos.resultados.iter().enumerate() {
                let indice = resultado.indice;
                let producto = &datos.productos[indice];
                let (caja, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 56.0), Sense::click());
                let elegida = datos.seleccion == Some(posicion);
                if elegida && datos.mostrar_seleccion {
                    ui.scroll_to_rect(caja, None);
                }
                if elegida || respuesta.hovered() {
                    pintor.rect_filled(caja, radio::CONTROL, color::VERDE_ELEGIDO);
                }
                let avatar = Rect::from_min_size(Pos2::new(caja.left() + 12.0, caja.center().y - 18.0), Vec2::splat(36.0));
                let con_foto = datos.fotos.uri(producto).is_some_and(|uri| fotos::pintar(ui, avatar, uri, radio::CONTROL));
                if !con_foto {
                    Avatar::producto(producto.nombre.chars().next().map(String::from).unwrap_or_default()).pintar(ui, avatar);
                    if producto.foto.is_some() && ui.is_rect_visible(caja) {
                        sin_foto.push(indice);
                    }
                }
                let oferta = producto.oferta(datos.ahora);
                let precio = precio_visual(producto.precio_venta(datos.ahora), producto.iva_incluido, producto.iva_porcentaje, datos.negocio);
                let precio = galeria(ui, pesos(precio), tam::CUERPO_MAS, Peso::Semi, color::TEXTO);
                let unidad = galeria(ui, if producto.unidad == "Kg" { " /kg" } else { " /u" }, tam::XS, Peso::Normal, color::TEXTO_APAGADO);
                let x_unidad = caja.right() - 12.0 - unidad.size().x;
                let x_precio = x_unidad - precio.size().x;
                let y_precio = caja.center().y - precio.size().y / 2.0;
                let linea_base = y_precio + precio.size().y * 0.8;
                let mut limite = x_precio;
                if let Some(porcentaje) = oferta {
                    let original = precio_visual(producto.precio, producto.iva_incluido, producto.iva_porcentaje, datos.negocio);
                    let izquierda = tachado(ui, &pintor, x_precio - 8.0, linea_base, pesos(original));
                    limite = insignia_oferta(ui, &pintor, izquierda - 8.0, caja.center().y, porcentaje);
                }
                let ancho_texto = limite - 12.0 - (avatar.right() + 12.0);
                let nombre = galeria_cortada(&pintor, &producto.nombre, fuente(tam::CUERPO_MAS, Peso::Medio), color::TEXTO, ancho_texto);
                let codigos: Vec<String> = [producto.plu.as_ref().map(|plu| format!("PLU {plu}")), producto.codigo_barras.clone()]
                    .into_iter()
                    .flatten()
                    .filter(|c| !c.trim().is_empty())
                    .collect();
                let parecido = resultado.aproximado.then(|| galeria(ui, "Parecido", tam::XS, Peso::Semi, color::DORADO_TEXTO));
                let ancho_parecido = parecido.as_ref().map_or(0.0, |p| p.size().x + 8.0);
                let detalle = (!codigos.is_empty()).then(|| galeria_cortada(&pintor, &codigos.join(" · "), fuente_mono(tam::XS, Peso::Normal), color::TEXTO_TENUE, ancho_texto - ancho_parecido));
                let alto_detalle = detalle.as_ref().map(|d| d.size().y).into_iter().chain(parecido.as_ref().map(|p| p.size().y)).fold(0.0, f32::max);
                let alto = nombre.size().y + if alto_detalle > 0.0 { 1.0 + alto_detalle } else { 0.0 };
                let mut y = caja.center().y - alto / 2.0;
                let x = avatar.right() + 12.0;
                let alto_nombre = nombre.size().y;
                pintor.galley(Pos2::new(x, y), nombre, color::TEXTO);
                y += alto_nombre + 1.0;
                if let Some(parecido) = parecido {
                    pintor.galley(Pos2::new(x, y), parecido, color::DORADO_TEXTO);
                }
                if let Some(detalle) = detalle {
                    pintor.galley(Pos2::new(x + ancho_parecido, y), detalle, color::TEXTO_TENUE);
                }
                pintor.galley(Pos2::new(x_unidad, linea_base - unidad.size().y * 0.8), unidad, color::TEXTO_APAGADO);
                pintor.galley(Pos2::new(x_precio, y_precio), precio, color::TEXTO);
                if respuesta.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    elegido = Some(posicion);
                }
            }
            if datos.resultados.is_empty() {
                let mensaje = if datos.no_encontrado { format!("No encontrado: “{}”", datos.consulta) } else { format!("No hay productos que coincidan con “{}”", datos.consulta) };
                let mensaje = galeria_cortada(&pintor, &mensaje, fuente(tam::CUERPO, Peso::Normal), color::TEXTO_APAGADO, ancho - 24.0);
                let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, 36.0 + mensaje.size().y), Sense::hover());
                pintor.galley(caja.center() - mensaje.size() / 2.0, mensaje, color::TEXTO_APAGADO);
            }
            ui.add_space(4.0);
            let alto_ayuda = galeria(ui, "Esc", tam::XS, Peso::Normal, color::TEXTO_TENUE).size().y;
            let (pie, _) = ui.allocate_exact_size(Vec2::new(ancho, 8.0 + alto_ayuda + 4.0), Sense::hover());
            pintor.hline(pie.x_range(), pie.top() + 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            let mut x = pie.left() + 12.0;
            for ayuda in ["↑ ↓ para moverte", "Enter para agregar", "Esc para cerrar"] {
                let g = galeria(ui, ayuda, tam::XS, Peso::Normal, color::TEXTO_TENUE);
                let ancho_ayuda = g.size().x;
                pintor.galley(Pos2::new(x, pie.top() + 8.0), g, color::TEXTO_TENUE);
                x += ancho_ayuda + 14.0;
            }
            (elegido, sin_foto)
        })
        .unwrap_or_default()
}

pub enum AccionPestana {
    Elegir(usize),
    Agregar,
    Borrar(usize),
}

pub enum AccionIndicador {
    Balanza,
    Impresora,
}

pub struct Indicadores {
    pub peso: String,
    pub balanza_conectada: bool,
    pub impresora: String,
}

pub struct FilaPestanas {
    pub pestana: Option<AccionPestana>,
    pub indicador: Option<AccionIndicador>,
    pub alto: f32,
}

pub fn fila_pestanas(ui: &mut Ui, area: Rect, carritos: &Carritos, indicadores: &Indicadores) -> FilaPestanas {
    let pintor = ui.painter().clone();
    let mut pestana = None;
    let (mut x, mut y) = (area.left(), area.top());
    let mut colocar = |ancho: f32| {
        if x > area.left() && x + ancho > area.right() {
            x = area.left();
            y += 48.0;
        }
        let caja = Rect::from_min_size(Pos2::new(x, y), Vec2::new(ancho, 40.0));
        x += ancho + 8.0;
        caja
    };
    for (indice, pantalla) in carritos.screens.iter().enumerate() {
        let activa = carritos.active_screen == indice;
        let borrable = indice > 0;
        let (fondo, borde, tinta, fondo_contador, tinta_contador, tinta_cruz) = if activa {
            (color::VERDE_PROFUNDO, color::VERDE_PROFUNDO, color::BLANCO, color::DORADO, color::VERDE_PROFUNDO, color::SOBRE_OSCURO_APAGADO)
        } else {
            (color::BLANCO, color::BORDE, color::TEXTO_SECUNDARIO, color::VERDE_CHIP, color::VERDE, color::TEXTO_TENUE)
        };
        let texto = galeria(ui, format!("Orden {}", indice + 1), tam::CUERPO, if activa { Peso::Semi } else { Peso::Medio }, tinta);
        let cantidad = galeria(ui, pantalla.items.len().to_string(), tam::XS, Peso::Medio, tinta_contador);
        let ancho_contador = cantidad.size().x + 14.0;
        let ancho = 14.0 + texto.size().x + 10.0 + ancho_contador + if borrable { 10.0 + 24.0 + 6.0 } else { 14.0 };
        let caja = colocar(ancho);
        let respuesta = clic(ui, caja, Id::new("pestana-orden").with(indice));
        pintor.rect(caja, radio::BOTON_MODAL, fondo, Stroke::new(1.0, borde), StrokeKind::Inside);
        let ancho_texto = texto.size().x;
        pintor.galley(Pos2::new(caja.left() + 14.0, caja.center().y - texto.size().y / 2.0), texto, tinta);
        let contador = Rect::from_min_size(
            Pos2::new(caja.left() + 14.0 + ancho_texto + 10.0, caja.center().y - (cantidad.size().y + 2.0) / 2.0),
            Vec2::new(ancho_contador, cantidad.size().y + 2.0),
        );
        pintor.rect_filled(contador, radio::PASTILLA, fondo_contador);
        pintor.galley(contador.center() - cantidad.size() / 2.0, cantidad, tinta_contador);
        if respuesta.clicked() {
            pestana = Some(AccionPestana::Elegir(indice));
        }
        if borrable {
            let cruz = Rect::from_min_size(Pos2::new(contador.right() + 10.0, caja.center().y - 12.0), Vec2::splat(24.0));
            let respuesta = clic(ui, cruz, Id::new("borrar-orden").with(indice));
            let tinta_cruz = if respuesta.hovered() {
                pintor.rect_filled(cruz, radio::TECLA_CUADRADA, color::ROJO_12);
                color::ROJO_PESTANA
            } else {
                tinta_cruz
            };
            pintar_icono_solar_en(ui, Rect::from_center_size(cruz.center(), Vec2::splat(15.0)), "x", tinta_cruz);
            if respuesta.clicked() {
                pestana = Some(AccionPestana::Borrar(indice));
            }
        }
    }
    if carritos.screens.len() < 4 {
        let texto = galeria(ui, "Nueva orden", tam::CONTROL, Peso::Normal, color::TEXTO_SECUNDARIO);
        let caja = colocar(12.0 + 15.0 + 6.0 + texto.size().x + 12.0);
        let respuesta = clic(ui, caja, Id::new("agregar-orden"));
        let encima = respuesta.hovered();
        if encima {
            pintor.rect_filled(caja, radio::BOTON_MODAL, color::BLANCO);
        }
        let tinta = if encima { color::VERDE } else { color::TEXTO_SECUNDARIO };
        borde_punteado(&pintor, caja, radio::BOTON_MODAL as f32, if encima { color::VERDE } else { color::BORDE_HOVER });
        pintar_icono_solar_en(ui, Rect::from_min_size(Pos2::new(caja.left() + 12.0, caja.center().y - 7.5), Vec2::splat(15.0)), "plus", tinta);
        pintor.galley(Pos2::new(caja.left() + 33.0, caja.center().y - texto.size().y / 2.0), texto, tinta);
        if respuesta.clicked() {
            pestana = Some(AccionPestana::Agregar);
        }
    }
    let balanza = galeria(ui, format!("Balanza · {}", indicadores.peso), tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
    let impresora = galeria(ui, indicadores.impresora.as_str(), tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
    let ancho_indicadores = 13.0 + balanza.size().x + 14.0 + 13.0 + impresora.size().x;
    let alto_indicador = balanza.size().y.max(7.0);
    let (inicio, centro, alto) = if x + 8.0 + ancho_indicadores <= area.right() + 0.5 {
        (area.right() - ancho_indicadores, y + 20.0, y + 40.0 - area.top())
    } else {
        (area.left(), y + 48.0 + alto_indicador / 2.0, y + 48.0 + alto_indicador - area.top())
    };
    let mut indicador = None;
    let mut x = inicio;
    let punto_balanza = if indicadores.balanza_conectada { color::VERDE_PUNTO } else { color::TEXTO_DESHABILITADO };
    for (g, punto, accion, id) in [(balanza, punto_balanza, AccionIndicador::Balanza, "indicador-balanza"), (impresora, color::VERDE_PUNTO, AccionIndicador::Impresora, "indicador-impresora")] {
        let ancho = 13.0 + g.size().x;
        let caja = Rect::from_min_size(Pos2::new(x, centro - 9.0), Vec2::new(ancho, 18.0));
        pintor.circle_filled(Pos2::new(x + 3.5, centro), 3.5, punto);
        pintor.galley(Pos2::new(x + 13.0, centro - g.size().y / 2.0), g, color::TEXTO_APAGADO);
        if clic(ui, caja, Id::new(id)).clicked() {
            indicador = Some(accion);
        }
        x += ancho + 14.0;
    }
    FilaPestanas { pestana, indicador, alto }
}

pub enum AccionFila {
    Quitar(String),
    Editar(String),
}

pub struct Fila {
    pub cart_id: String,
    pub precio_libre: bool,
    pub nombre: String,
    pub por_kilo: bool,
    pub lista: Option<String>,
    pub cantidad: String,
    pub precio: String,
    pub precio_original: Option<String>,
    pub oferta: Option<f64>,
    pub subtotal: String,
}

pub fn fila_de_item(item: &Producto, negocio: Option<&Value>) -> Fila {
    let unitario = precio_visual(item.price_per_unit, item.iva_incluido, item.iva_porcentaje, negocio);
    let por_kilo = item.unit == "Kg";
    let lista_por_defecto = negocio.and_then(|n| n["listaPrecioPorDefectoId"].as_i64());
    let lista = item.lista_precio_id.filter(|id| Some(*id) != lista_por_defecto).map(|id| {
        let nombre = item.lista_precio_nombre.as_deref().map(str::trim).filter(|n| !n.is_empty());
        nombre.map(str::to_string).unwrap_or_else(|| format!("Lista #{id}"))
    });
    let oferta = item.oferta_porcentaje.zip(item.precio_sin_oferta);
    if item.precio_libre {
        return Fila {
            cart_id: item.cart_id.clone(),
            precio_libre: true,
            nombre: item.name.clone(),
            por_kilo: false,
            lista: Some("Precio a definir".into()),
            cantidad: "1 u".into(),
            precio: pesos(unitario),
            precio_original: None,
            oferta: None,
            subtotal: pesos(unitario),
        };
    }
    Fila {
        cart_id: item.cart_id.clone(),
        precio_libre: false,
        nombre: item.name.clone(),
        por_kilo,
        lista,
        cantidad: if por_kilo { kilos(item.quantity) } else { format!("{} u", formato::numero_con(item.quantity, 0, 2)) },
        precio: format!("{}{}", pesos(unitario), if por_kilo { " /kg" } else { " /u" }),
        precio_original: oferta.map(|(_, original)| pesos(precio_visual(original, item.iva_incluido, item.iva_porcentaje, negocio))),
        oferta: oferta.map(|(porcentaje, _)| porcentaje),
        subtotal: pesos(unitario * item.quantity),
    }
}

pub struct Destello<'a> {
    pub cart_id: &'a str,
    pub avance: f32,
    pub desplazar: bool,
}

pub fn tabla(ui: &mut Ui, caja: Rect, filas: &[Fila], permite_borrar: bool, destello: Option<&Destello>, id: Id) -> Option<AccionFila> {
    let pintor = ui.painter().clone();
    pintor.rect(caja, radio::TARJETA, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let interior = caja.shrink(1.0);
    let encabezado = Rect::from_min_size(interior.min, Vec2::new(interior.width(), 44.0));
    let esquina = radio::TARJETA - 1;
    pintor.rect_filled(encabezado, CornerRadius { nw: esquina, ne: esquina, sw: 0, se: 0 }, color::FONDO_ENCABEZADO_TABLA);
    pintor.hline(encabezado.x_range(), encabezado.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
    let estrecha = interior.width() < 700.0;
    let columnas = columnas_tabla(ui, filas, estrecha);
    let (relleno, separacion) = if estrecha { (16.0, 10.0) } else { (20.0, 14.0) };
    for (celda, (titulo, derecha)) in celdas(encabezado, &columnas, relleno, separacion).into_iter().zip([("Producto", false), ("Cantidad", true), ("Precio", true), ("Subtotal", true)]) {
        celda_texto(&pintor, celda, titulo, tam::CHICO, Peso::Medio, color::TEXTO_APAGADO, derecha);
    }
    let cuerpo = Rect::from_min_max(Pos2::new(interior.left(), encabezado.bottom()), Pos2::new(interior.right(), interior.bottom() - radio::TARJETA as f32 / 2.0));
    let mut hijo = ui.new_child(UiBuilder::new().max_rect(cuerpo).layout(Layout::top_down(Align::Min)));
    hijo.set_clip_rect(cuerpo.intersect(ui.clip_rect()));
    let mut accion = None;
    ScrollArea::vertical().id_salt(id).auto_shrink(false).show(&mut hijo, |ui| {
        let pintor = ui.painter().clone();
        for (posicion, fila) in filas.iter().enumerate() {
            let sentido = if fila.precio_libre { Sense::click() } else { Sense::hover() };
            let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 68.0), sentido);
            if fila.precio_libre && respuesta.on_hover_text("Doble clic para cambiar el monto").double_clicked() {
                accion = Some(AccionFila::Editar(fila.cart_id.clone()));
            }
            if let Some(d) = destello.filter(|d| d.cart_id == fila.cart_id) {
                if d.avance < 1.0 {
                    pintor.rect_filled(rect, 0, color::DORADO_FONDO.gamma_multiply(1.0 - d.avance));
                }
                pintor.rect_filled(Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())), 0, color::DORADO);
                if d.desplazar {
                    ui.scroll_to_rect(rect, None);
                }
            }
            pintor.hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, color::BORDE_SUAVE));
            let c = celdas(rect, &columnas, relleno, separacion);
            let numero = galeria(ui, (posicion + 1).to_string(), tam::CHICO, Peso::Normal, color::TEXTO_TENUE);
            pintor.galley(Pos2::new(c[0].left(), rect.center().y - numero.size().y / 2.0), numero, color::TEXTO_TENUE);
            let x = c[0].left() + 26.0 + 12.0;
            let nombre = galeria_cortada(&pintor, &fila.nombre, fuente(tam::FILA, Peso::Medio), color::TEXTO, c[0].right() - x);
            let unidad = galeria(ui, if fila.precio_libre { "Monto" } else if fila.por_kilo { "Por kilo" } else { "Por unidad" }, tam::XS, Peso::Normal, color::TEXTO_TENUE);
            let chip = fila.lista.as_ref().map(|l| galeria(ui, l.as_str(), tam::MINI, Peso::Semi, color::DORADO_TEXTO));
            let alto_detalle = unidad.size().y.max(chip.as_ref().map_or(0.0, |g| g.size().y + 2.0));
            let alto_nombre = nombre.size().y;
            let arriba = rect.center().y - (alto_nombre + 3.0 + alto_detalle) / 2.0;
            pintor.galley(Pos2::new(x, arriba), nombre, color::TEXTO);
            let centro_detalle = arriba + alto_nombre + 3.0 + alto_detalle / 2.0;
            let ancho_unidad = unidad.size().x;
            pintor.galley(Pos2::new(x, centro_detalle - unidad.size().y / 2.0), unidad, color::TEXTO_TENUE);
            let mut x_chip = x + ancho_unidad + 6.0;
            if let Some(chip) = chip {
                let pintor = pintor.with_clip_rect(c[0].intersect(pintor.clip_rect()));
                let caja_chip = Rect::from_center_size(Pos2::new(x_chip + (chip.size().x + 14.0) / 2.0, centro_detalle), chip.size() + Vec2::new(14.0, 2.0));
                pintor.rect_filled(caja_chip, radio::CHIP, color::DORADO_FONDO);
                pintor.galley(caja_chip.center() - chip.size() / 2.0, chip, color::DORADO_TEXTO);
                x_chip = caja_chip.right() + 6.0;
            }
            if let Some(porcentaje) = fila.oferta {
                let pintor = pintor.with_clip_rect(c[0].intersect(pintor.clip_rect()));
                let ancho_insignia = galeria(ui, texto_oferta(porcentaje), tam::MINI, Peso::Semi, color::VERDE_EXITO).size().x + 12.0;
                insignia_oferta(ui, &pintor, x_chip + ancho_insignia, centro_detalle, porcentaje);
            }
            celda_texto(&pintor, c[1], &fila.cantidad, tam::DESTACADO, Peso::Normal, color::TEXTO, true);
            match &fila.precio_original {
                Some(original) => {
                    let precio = galeria(ui, fila.precio.as_str(), tam::CUERPO_MAS, Peso::Normal, color::TEXTO_APAGADO);
                    let alto_original = galeria(ui, original.as_str(), tam::XS, Peso::Normal, color::TEXTO_TENUE).size().y;
                    let arriba = rect.center().y - (alto_original + 2.0 + precio.size().y) / 2.0;
                    tachado(ui, &pintor, c[2].right(), arriba + alto_original * 0.8, original.clone());
                    pintor.galley(Pos2::new(c[2].right() - precio.size().x, arriba + alto_original + 2.0), precio, color::TEXTO_APAGADO);
                }
                None => celda_texto(&pintor, c[2], &fila.precio, tam::CUERPO_MAS, Peso::Normal, color::TEXTO_APAGADO, true),
            }
            celda_texto(&pintor, c[3], &fila.subtotal, tam::FILA, Peso::Semi, color::TEXTO, true);
            if permite_borrar {
                let boton = Rect::from_min_size(Pos2::new(c[4].left(), rect.center().y - 18.0), Vec2::splat(36.0));
                let mut celda = ui.new_child(UiBuilder::new().id_salt(("quitar-item", &fila.cart_id)).max_rect(boton));
                if BotonIcono::quitar().tam(36.0).radio(radio::CONTROL).tam_icono(19.0).ui(&mut celda).on_hover_text("Quitar").clicked() {
                    accion = Some(AccionFila::Quitar(fila.cart_id.clone()));
                }
            }
        }
    });
    accion
}

fn columnas_tabla(ui: &Ui, filas: &[Fila], estrecha: bool) -> [Pista; 5] {
    let medir = |texto: &str, px: f32, peso: Peso| galeria(ui, texto, px, peso, color::TEXTO).size().x.ceil() + 2.0;
    let ancho = |titulo: &str, valor: fn(&Fila) -> &str, px: f32, peso: Peso| filas.iter().map(|f| medir(valor(f), px, peso)).fold(medir(titulo, tam::CHICO, Peso::Medio), f32::max);
    [
        Pista::Fraccion(2.2, 0.0),
        Pista::Fraccion(1.0, ancho("Cantidad", |f| &f.cantidad, tam::DESTACADO, Peso::Normal)),
        Pista::Fraccion(1.0, ancho("Precio", |f| &f.precio, tam::CUERPO_MAS, Peso::Normal)),
        Pista::Fraccion(1.0, ancho("Subtotal", |f| &f.subtotal, tam::FILA, Peso::Semi)),
        Pista::Fija(if estrecha { 36.0 } else { 44.0 }),
    ]
}

pub fn orden_vacia(ui: &Ui, caja: Rect, orden: usize) {
    let pintor = ui.painter();
    pintor.rect(caja, radio::TARJETA, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let titulo = v2::galeria_espaciada(ui, format!("Orden {orden} vacía"), fuente(tam::GRANDE, Peso::Semi), color::TEXTO, -0.01 * tam::GRANDE);
    let parrafo = galeria_centrada(
        pintor,
        "Escaneá un producto o buscalo arriba por nombre o PLU. Los productos por kilo toman el peso de la balanza.",
        fuente(tam::CUERPO_MAS, Peso::Normal),
        color::TEXTO_APAGADO,
        1.5,
        (caja.width() - 64.0).min(400.0),
    );
    let alto = 72.0 + 14.0 + titulo.size().y + 14.0 + parrafo.size().y;
    let centro_x = caja.center().x;
    let mut y = (caja.center().y - alto / 2.0).max(caja.top() + 32.0);
    let icono = Rect::from_min_size(Pos2::new(centro_x - 36.0, y), Vec2::splat(72.0));
    pintor.rect_filled(icono, radio::MODAL, color::VERDE_ELEGIDO);
    pintar_icono_solar_en(ui, Rect::from_center_size(icono.center(), Vec2::splat(38.0)), "cart", color::VERDE);
    y += 72.0 + 14.0;
    let alto_titulo = titulo.size().y;
    pintor.galley(Pos2::new(centro_x - titulo.size().x / 2.0, y), titulo, color::TEXTO);
    y += alto_titulo + 14.0;
    pintor.galley(Pos2::new(centro_x, y), parrafo, color::TEXTO_APAGADO);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionPanel {
    Cobrar,
    CobrarSinFactura,
    Descuento,
    DescuentoSinFactura,
    CuentaCorriente,
    Cancelar,
    CerrarSesion,
}

pub struct Panel {
    pub total: f64,
    pub productos: usize,
    pub orden: usize,
    pub afip: bool,
    pub facturacion: bool,
}

pub fn panel(ui: &mut Ui, area: Rect, datos: &Panel) -> Option<AccionPanel> {
    let pintor = ui.painter().clone();
    let titulo = v2::galeria_espaciada(ui, "TOTAL", fuente(tam::ETIQUETA, Peso::Medio), color::SOBRE_OSCURO_APAGADO, 0.06 * tam::ETIQUETA);
    let cantidad = galeria(ui, format!("{} producto{}", datos.productos, if datos.productos == 1 { "" } else { "s" }), tam::CHICO, Peso::Normal, color::SOBRE_OSCURO_APAGADO);
    let total = v2::galeria_espaciada(ui, pesos(datos.total), fuente(tam::TOTAL, Peso::Semi), color::BLANCO, -0.03 * tam::TOTAL);
    let iva = galeria(ui, format!("IVA incluido · Orden {}", datos.orden), tam::CHICO, Peso::Normal, color::SOBRE_OSCURO_TENUE);
    let alto_fila = titulo.size().y.max(cantidad.size().y);
    let alto_total = tam::TOTAL * 1.1;
    let tarjeta = Rect::from_min_size(area.min, Vec2::new(area.width(), 22.0 + alto_fila + 6.0 + alto_total + 6.0 + iva.size().y + 20.0));
    pintor.rect_filled(tarjeta, radio::PANEL, color::VERDE_PROFUNDO);
    let mut y = tarjeta.top() + 22.0;
    pintor.galley(Pos2::new(tarjeta.left() + 22.0, y + (alto_fila - titulo.size().y) / 2.0), titulo, color::SOBRE_OSCURO_APAGADO);
    pintor.galley(Pos2::new(tarjeta.right() - 22.0 - cantidad.size().x, y + (alto_fila - cantidad.size().y) / 2.0), cantidad, color::SOBRE_OSCURO_APAGADO);
    y += alto_fila + 6.0;
    pintor.galley(Pos2::new(tarjeta.left() + 22.0, y + (alto_total - total.size().y) / 2.0), total, color::BLANCO);
    y += alto_total + 6.0;
    pintor.galley(Pos2::new(tarjeta.left() + 22.0, y), iva, color::SOBRE_OSCURO_TENUE);

    let mut laterales = Vec::new();
    if datos.afip {
        laterales.push(("Cobrar orden sin factura", "receipt", "F3", AccionPanel::CobrarSinFactura, false));
    }
    laterales.push(("Cobrar con descuento", "percent", "F7", AccionPanel::Descuento, false));
    if datos.afip {
        laterales.push(("Descuento sin factura", "tag", "F8", AccionPanel::DescuentoSinFactura, false));
    }
    if datos.facturacion {
        laterales.push(("Cuenta corriente", "file-text", "F6", AccionPanel::CuentaCorriente, false));
    }
    laterales.push(("Cancelar orden", "trash-2", "F1", AccionPanel::Cancelar, true));
    let acciones = Rect::from_min_size(
        Pos2::new(area.left(), tarjeta.bottom() + 14.0),
        Vec2::new(area.width(), 14.0 + 68.0 + laterales.len() as f32 * 60.0 + 14.0),
    );
    pintor.rect(acciones, radio::PANEL, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
    let mut accion = None;
    let cobrar = Rect::from_min_size(acciones.min + Vec2::splat(14.0), Vec2::new(acciones.width() - 28.0, 68.0));
    if boton_cobrar(ui, cobrar, datos) {
        accion = Some(AccionPanel::Cobrar);
    }
    let mut y = cobrar.bottom() + 8.0;
    for (etiqueta, icono, tecla, a, peligro) in laterales {
        let caja = Rect::from_min_size(Pos2::new(cobrar.left(), y), Vec2::new(cobrar.width(), 52.0));
        if accion_lateral(ui, caja, etiqueta, icono, tecla, peligro) {
            accion = Some(a);
        }
        y += 60.0;
    }
    if cerrar_sesion(ui, area, acciones.bottom() + 14.0) {
        accion = Some(AccionPanel::CerrarSesion);
    }
    accion
}

fn boton_cobrar(ui: &mut Ui, rect: Rect, datos: &Panel) -> bool {
    let respuesta = clic(ui, rect, Id::new("accion-cobrar"));
    let avance = ui.ctx().animate_bool_with_time(respuesta.id.with("hover"), respuesta.hovered(), 0.15);
    let opacidad = if datos.productos > 0 { 1.0 } else { 0.55 };
    let tinte = |c: Color32| mezclar(color::BLANCO, c, opacidad);
    let mut sombra_apagada = ui.new_child(UiBuilder::new().max_rect(rect));
    sombra_apagada.set_opacity(opacidad);
    sombra_apagada.painter().add(sombra::BOTON_COBRAR.forma(rect, radio::POPOVER));
    let pintor = ui.painter().clone();
    pintor.rect_filled(rect, radio::POPOVER, tinte(mezclar(color::VERDE, color::VERDE_PROFUNDO, avance)));
    pintar_icono_solar_en(ui, Rect::from_min_size(Pos2::new(rect.left() + 18.0, rect.center().y - 13.0), Vec2::splat(26.0)), "wallet", tinte(color::DORADO));
    let (titulo, detalle) = if datos.afip { ("Cobrar con factura", None) } else { ("Cobrar", Some("Efectivo, tarjeta, QR o mixto")) };
    let titulo = galeria(ui, titulo, tam::FILA, Peso::Semi, color::BLANCO);
    let detalle = detalle.map(|d| galeria(ui, d, tam::XS, Peso::Normal, tinte(color::SOBRE_OSCURO_APAGADO)));
    let x = rect.left() + 18.0 + 26.0 + 14.0;
    let alto_detalle = detalle.as_ref().map_or(0.0, |d| 1.0 + d.size().y);
    let arriba = rect.center().y - (titulo.size().y + alto_detalle) / 2.0;
    let alto_titulo = titulo.size().y;
    pintor.galley(Pos2::new(x, arriba), titulo, color::BLANCO);
    if let Some(detalle) = detalle {
        pintor.galley(Pos2::new(x, arriba + alto_titulo + 1.0), detalle, tinte(color::SOBRE_OSCURO_APAGADO));
    }
    let tam_tecla = v2::medir_tecla(ui, "F2", &EstiloTecla::SOBRE_OSCURO);
    let tecla = Rect::from_min_size(Pos2::new(rect.right() - 18.0 - tam_tecla.x, rect.center().y - tam_tecla.y / 2.0), tam_tecla);
    v2::pintar_tecla(ui, tecla, "F2", &EstiloTecla::SOBRE_OSCURO, color::BLANCO);
    respuesta.clicked()
}

fn accion_lateral(ui: &mut Ui, rect: Rect, etiqueta: &str, icono: &str, tecla: &str, peligro: bool) -> bool {
    let respuesta = clic(ui, rect, Id::new("accion-lateral").with(tecla));
    let avance = ui.ctx().animate_bool_with_time(respuesta.id.with("hover"), respuesta.hovered(), 0.15);
    let (borde, tinta, tinta_icono, fondo_hover) =
        if peligro { (color::ROJO_BORDE, color::ROJO, color::ROJO_INTENSO, color::ROJO_FONDO) } else { (color::BORDE, color::TEXTO, color::VERDE, color::FONDO_APP) };
    let pintor = ui.painter();
    pintor.rect(rect, radio::CAMPO, mezclar(color::BLANCO, fondo_hover, avance), Stroke::new(1.0, borde), StrokeKind::Inside);
    pintar_icono_solar_en(ui, Rect::from_min_size(Pos2::new(rect.left() + 14.0, rect.center().y - 10.5), Vec2::splat(21.0)), icono, tinta_icono);
    let texto = galeria(ui, etiqueta, tam::CUERPO_MAS, Peso::Medio, tinta);
    pintor.galley(Pos2::new(rect.left() + 14.0 + 21.0 + 12.0, rect.center().y - texto.size().y / 2.0), texto, tinta);
    let tam_tecla = v2::medir_tecla(ui, tecla, &EstiloTecla::CLARA);
    let caja_tecla = Rect::from_min_size(Pos2::new(rect.right() - 14.0 - tam_tecla.x, rect.center().y - tam_tecla.y / 2.0), tam_tecla);
    v2::pintar_tecla(ui, caja_tecla, tecla, &EstiloTecla::CLARA, color::TEXTO_SECUNDARIO);
    respuesta.clicked()
}

fn cerrar_sesion(ui: &mut Ui, area: Rect, minimo_y: f32) -> bool {
    let texto = galeria(ui, "Cerrar sesión", tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
    let tam_tecla = v2::medir_tecla(ui, "F4", &EstiloTecla::CONTORNO);
    let ancho = 6.0 + 16.0 + 8.0 + texto.size().x + 8.0 + tam_tecla.x + 6.0;
    let alto = texto.size().y.max(tam_tecla.y).max(16.0) + 12.0;
    let y = (area.bottom() - alto).max(minimo_y);
    let rect = Rect::from_min_size(Pos2::new(area.center().x - ancho / 2.0, y), Vec2::new(ancho, alto));
    let respuesta = clic(ui, rect, Id::new("accion-cerrar-sesion"));
    let tinta = if respuesta.hovered() { color::ROJO } else { color::TEXTO_APAGADO };
    let mut x = rect.left() + 6.0;
    pintar_icono_solar_en(ui, Rect::from_min_size(Pos2::new(x, rect.center().y - 8.0), Vec2::splat(16.0)), "log-out", tinta);
    x += 16.0 + 8.0;
    let ancho_texto = texto.size().x;
    ui.painter().galley_with_override_text_color(Pos2::new(x, rect.center().y - texto.size().y / 2.0), texto, tinta);
    x += ancho_texto + 8.0;
    v2::pintar_tecla(ui, Rect::from_min_size(Pos2::new(x, rect.center().y - tam_tecla.y / 2.0), tam_tecla), "F4", &EstiloTecla::CONTORNO, tinta);
    respuesta.clicked()
}
