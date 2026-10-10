use eframe::egui::{
    self, Align, Area, Color32, CursorIcon, Id, Layout, Order, Pos2, Rect, Response, Sense, StrokeKind, Ui, UiBuilder, Vec2,
};
use nucleo::carrito::{Carritos, Producto};
use nucleo::iva::precio_visual_con_iva;
use serde_json::Value;

use super::diseno::{encoger, Texto};
use super::productos::{formato_local, numero_js, ProductoDisponible};
use crate::ui::{self, borde, color, fuente, paleta, Boton, Direccion, Peso, Tam, TamBoton, Variante};

pub const ANCHO_BARRA: f32 = 384.0;

pub fn en_rect<R>(ui: &mut Ui, rect: Rect, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    let mut hijo = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Min)));
    contenido(&mut hijo)
}

pub fn ancho_texto(ui: &Ui, contenido: &str, tam: Tam, peso: Peso) -> f32 {
    ui.painter().layout_no_wrap(contenido.to_string(), fuente(tam, peso), color::TEXTO).size().x
}

pub fn precio_visual(precio: f64, iva_incluido: Option<bool>, iva_porcentaje: Option<f64>, negocio: Option<&Value>) -> f64 {
    match negocio {
        Some(n) => precio_visual_con_iva(precio, iva_incluido.unwrap_or(false), iva_porcentaje, n["ivaIncluidoEnPrecios"].as_bool().unwrap_or(false)),
        None => precio,
    }
}

pub fn atajos(negocio: Option<&Value>) -> Vec<(&'static str, &'static str)> {
    let bandera = |clave: &str| negocio.is_some_and(|n| n[clave].as_bool().unwrap_or(false));
    let afip = bandera("afipHabilitado");
    let facturacion = bandera("facturacionHabilitada");
    [
        ("F1", "CANCELAR", true),
        ("F2", if afip { "PAGAR CON AFIP" } else { "PAGAR" }, true),
        ("F7", if afip { "PAGAR CON DESCUENTO AFIP" } else { "PAGAR CON DESCUENTO" }, true),
        ("F3", "PAGAR ORDEN", afip),
        ("F8", "PAGAR CON DESCUENTO ORDEN", afip),
        ("F6", "CUENTA CORRIENTE", facturacion),
        ("F4", "CERRAR SESIÓN", true),
    ]
    .into_iter()
    .filter(|(_, _, visible)| *visible)
    .map(|(tecla, descripcion, _)| (tecla, descripcion))
    .collect()
}

pub fn barra_lateral(ui: &mut Ui, rect: Rect, atajos: &[(&str, &str)]) -> Rect {
    ui::pintar_degradado(ui.painter(), rect, 0, color::ESMERALDA_DESDE, color::ESMERALDA_HASTA, Direccion::AbajoDerecha);
    let alto_logo = 192.0 * 26.0 / 228.0;
    let logo = Rect::from_min_size(rect.min + Vec2::new(24.0, 24.0), Vec2::new(192.0, alto_logo));
    ui::imagen("andexmarket-logo.png").paint_at(ui, logo);
    let filas = atajos.len() as f32;
    let alto_panel = 8.0 + filas * 40.0 + (filas - 1.0).max(0.0) * 6.0;
    let panel = Rect::from_min_max(Pos2::new(rect.left() + 8.0, rect.bottom() - 8.0 - alto_panel.max(128.0)), Pos2::new(rect.right() - 8.0, rect.bottom() - 8.0));
    ui.painter().rect_filled(panel, ui::RADIO, paleta::GRAY_50);
    let mut y = panel.top() + 4.0;
    for (tecla, descripcion) in atajos {
        let imagen = Rect::from_min_size(Pos2::new(panel.left() + 4.0, y), Vec2::splat(40.0));
        ui::imagen(&format!("teclas/{tecla}.png")).paint_at(ui, imagen);
        ui.painter().text(
            Pos2::new(imagen.right() + 8.0, imagen.center().y),
            egui::Align2::LEFT_CENTER,
            *descripcion,
            fuente(Tam::Base, Peso::Normal),
            color::APAGADO_TEXTO,
        );
        y += 46.0;
    }
    Rect::from_min_size(Pos2::new(rect.left() + 16.0, logo.bottom() + 24.0 + 16.0), Vec2::new(rect.width() - 32.0, 40.0))
}

pub fn resultados(
    ctx: &egui::Context,
    origen: Pos2,
    ancho: f32,
    productos: &[ProductoDisponible],
    indices: &[usize],
    seleccion: Option<usize>,
    negocio: Option<&Value>,
) -> Option<usize> {
    let mut elegido = None;
    Area::new(Id::new("resultados-busqueda")).order(Order::Foreground).fixed_pos(origen).show(ctx, |ui| {
        let pintor = ui.painter().clone();
        let ancho_item = ancho - 18.0;
        let disponible = ancho_item - 24.0;
        let filas: Vec<_> = indices
            .iter()
            .enumerate()
            .map(|(posicion, &indice)| {
                let p = &productos[indice];
                let elegida = seleccion == Some(posicion);
                let peso = if elegida { Peso::Medio } else { Peso::Normal };
                let color_nombre = if elegida { paleta::EMERALD_900 } else { color::TEXTO };
                let insignia = p.plu.as_ref().map(|plu| Texto::new(format!("PLU: {plu}"), Tam::Xs, Peso::Medio, paleta::BLUE_800));
                let ancho_insignia = insignia.as_ref().map_or(0.0, |t| t.maximo(&pintor) + 16.0 + 8.0);
                let nombre = Texto::new(p.nombre.clone(), Tam::Base, peso, color_nombre);
                let precio = format!("${}/{}", formato_local(precio_visual(p.precio, p.iva_incluido, p.iva_porcentaje, negocio)), p.unidad);
                let precio = Texto::new(precio, Tam::Base, peso, paleta::EMERALD_600);
                let ancho_precio = precio.maximo(&pintor);
                let tamanos = encoger(
                    &[ancho_insignia + nombre.maximo(&pintor), ancho_precio],
                    &[ancho_insignia + nombre.minimo(&pintor), ancho_precio],
                    disponible,
                );
                let galeria_nombre = nombre.galeria(&pintor, tamanos[0] - ancho_insignia);
                let alto = nombre.alto(&galeria_nombre).max(24.0) + 24.0;
                (elegida, insignia, ancho_insignia, nombre, galeria_nombre, precio, ancho_precio, alto)
            })
            .collect();
        let alto_total = filas.iter().map(|f| f.7).sum::<f32>() + 18.0;
        let (tarjeta, _) = ui.allocate_exact_size(Vec2::new(ancho, alto_total), Sense::hover());
        ui.painter().add(ui::sombra_lg().as_shape(tarjeta, ui::RADIO_XL));
        ui.painter().rect(tarjeta, ui::RADIO_XL, color::FONDO, borde(color::BORDE), StrokeKind::Inside);
        let mut y = tarjeta.top() + 9.0;
        for (posicion, (elegida, insignia, ancho_insignia, nombre, galeria_nombre, precio, ancho_precio, alto)) in filas.into_iter().enumerate() {
            let caja = Rect::from_min_size(Pos2::new(tarjeta.left() + 9.0, y), Vec2::new(ancho_item, alto));
            let r = ui.interact(caja, Id::new("resultado").with(posicion), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            if elegida {
                ui.painter().rect_filled(caja, ui::RADIO, paleta::EMERALD_100);
            } else if r.hovered() {
                ui.painter().rect_filled(caja, ui::RADIO, color::APAGADO);
            }
            let centro = caja.center().y;
            let mut x = caja.left() + 12.0;
            if let Some(insignia) = insignia {
                let caja_insignia = Rect::from_min_size(Pos2::new(x, centro - 12.0), Vec2::new(ancho_insignia - 8.0, 24.0));
                ui.painter().rect_filled(caja_insignia, ui::RADIO_SM, paleta::BLUE_100);
                let g = insignia.galeria(&pintor, f32::INFINITY);
                insignia.pintar(&pintor, &g, Pos2::new(x + 8.0, centro - 8.0));
                x += ancho_insignia;
            }
            let alto_nombre = nombre.alto(&galeria_nombre);
            nombre.pintar(&pintor, &galeria_nombre, Pos2::new(x, centro - alto_nombre / 2.0));
            let g = precio.galeria(&pintor, f32::INFINITY);
            precio.pintar(&pintor, &g, Pos2::new(caja.right() - 12.0 - ancho_precio, centro - 12.0));
            if r.clicked() {
                elegido = Some(posicion);
            }
            y += alto;
        }
    });
    elegido
}

pub enum AccionPestana {
    Elegir(usize),
    Agregar,
    Borrar(usize),
}

struct Pestana {
    id: usize,
    caja: Rect,
    borrar: Option<Rect>,
}

fn disponer_pestanas(ui: &Ui, carritos: &Carritos, origen: Pos2) -> (Vec<Pestana>, Option<Rect>, f32) {
    let mut x = origen.x;
    let mut pestanas = Vec::new();
    for pantalla in &carritos.screens {
        if pantalla.id > 0 {
            x += 16.0;
        }
        let activa = carritos.active_screen == pantalla.id;
        let ancho = 64.0 + ancho_texto(ui, &format!("Orden {}", pantalla.id + 1), Tam::Sm, Peso::Medio) + if activa { 0.0 } else { 2.0 };
        let caja = Rect::from_min_size(Pos2::new(x, origen.y - 20.0), Vec2::new(ancho, 40.0));
        x += ancho;
        let borrar = (pantalla.id > 0).then(|| {
            let r = Rect::from_min_size(Pos2::new(x, origen.y - 16.0), Vec2::new(28.0, 32.0));
            x += 28.0;
            r
        });
        pestanas.push(Pestana { id: pantalla.id, caja, borrar });
    }
    let mas = (carritos.screens.len() < 4).then(|| {
        x += 4.0;
        let r = Rect::from_min_size(Pos2::new(x, origen.y - 16.0), Vec2::new(34.0, 32.0));
        x += 34.0;
        r
    });
    (pestanas, mas, x - origen.x)
}

pub fn ancho_pestanas(ui: &Ui, carritos: &Carritos) -> f32 {
    disponer_pestanas(ui, carritos, Pos2::ZERO).2
}

pub fn pestanas(ui: &mut Ui, carritos: &Carritos, origen: Pos2) -> Option<AccionPestana> {
    let (lista, mas, _) = disponer_pestanas(ui, carritos, origen);
    let mut accion = None;
    for pestana in lista {
        let activa = carritos.active_screen == pestana.id;
        let r = ui.interact(pestana.caja, Id::new("pestana-orden").with(pestana.id), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let (fondo, texto_color, trazo) = match (activa, r.hovered()) {
            (true, false) => (Color32::BLACK, Color32::WHITE, None),
            (true, true) => (color::PRIMARIO.gamma_multiply(0.9), Color32::WHITE, None),
            (false, false) => (paleta::GRAY_300, Color32::WHITE, Some(color::BORDE)),
            (false, true) => (color::ACENTO, color::ACENTO_TEXTO, Some(color::BORDE)),
        };
        ui.painter().rect(pestana.caja, ui::RADIO, fondo, trazo.map_or(egui::Stroke::NONE, borde), StrokeKind::Inside);
        ui.painter().text(pestana.caja.center(), egui::Align2::CENTER_CENTER, format!("Orden {}", pestana.id + 1), fuente(Tam::Sm, Peso::Medio), texto_color);
        if r.clicked() {
            accion = Some(AccionPestana::Elegir(pestana.id));
        }
        if let Some(caja) = pestana.borrar {
            let r = ui.interact(caja, Id::new("borrar-orden").with(pestana.id), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            let icono_color = if r.hovered() {
                ui.painter().rect_filled(caja, ui::RADIO, paleta::RED_100);
                paleta::RED_600
            } else {
                color::TEXTO
            };
            ui::pintar_icono_en(ui, Rect::from_center_size(caja.center(), Vec2::splat(16.0)), "x", icono_color);
            if r.clicked() {
                accion = Some(AccionPestana::Borrar(pestana.id));
            }
        }
    }
    if let Some(caja) = mas {
        let r = ui.interact(caja, Id::new("agregar-orden"), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let fondo = if r.hovered() { color::ACENTO } else { color::FONDO };
        ui.painter().rect(caja, ui::RADIO, fondo, borde(Color32::BLACK), StrokeKind::Inside);
        ui::pintar_icono_en(ui, Rect::from_center_size(caja.center(), Vec2::splat(16.0)), "plus", Color32::BLACK);
        if r.clicked() {
            accion = Some(AccionPestana::Agregar);
        }
    }
    accion
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
    pub usuario: Option<(&'a str, &'a str)>,
}

impl Cabecera<'_> {
    fn botones(&self) -> Vec<(&'static str, &'static str, AccionCabecera)> {
        let mut botones = vec![("history", "Órdenes", AccionCabecera::Ordenes)];
        if self.facturacion {
            botones.push(("receipt", "Cuenta Corriente", AccionCabecera::CuentaCorriente));
        }
        if self.cierre {
            botones.push(("store", "Cierre de caja", AccionCabecera::Cierre));
        }
        botones
    }

    fn ancho_boton(ui: &Ui, texto: &str) -> f32 {
        32.0 + 24.0 + 8.0 + ancho_texto(ui, texto, Tam::Base, Peso::Medio)
    }

    pub fn ancho(&self, ui: &Ui) -> f32 {
        let botones = self.botones();
        let mut ancho: f32 = botones.iter().map(|(_, t, _)| Self::ancho_boton(ui, t)).sum::<f32>() + 16.0 * (botones.len() as f32 - 1.0);
        if let Some((nombre, _)) = self.usuario {
            ancho += 16.0 + ancho_texto(ui, nombre, Tam::Base, Peso::Medio) + 8.0 + 40.0;
        }
        ancho
    }

    pub fn mostrar(&self, ui: &mut Ui, derecha: f32, centro_y: f32) -> Option<AccionCabecera> {
        let mut accion = None;
        let mut x = derecha;
        if let Some((nombre, email)) = self.usuario {
            let caja = Rect::from_min_size(Pos2::new(x - 40.0, centro_y - 20.0), Vec2::splat(40.0));
            let r = en_rect(ui, caja, |ui| {
                Boton::new("").variante(Variante::Esmeralda).tam(TamBoton::Icono).icono("user").tam_icono(24.0).ui(ui)
            });
            x -= 48.0;
            let ancho_nombre = ancho_texto(ui, nombre, Tam::Base, Peso::Medio);
            ui.painter().text(Pos2::new(x, centro_y), egui::Align2::RIGHT_CENTER, nombre, fuente(Tam::Base, Peso::Medio), color::TEXTO);
            x -= ancho_nombre + 16.0;
            if let Some(Some(a)) = menu_usuario(&r, nombre, email) {
                accion = Some(a);
            }
        }
        for (icono, texto, a) in self.botones().into_iter().rev() {
            let ancho = Self::ancho_boton(ui, texto);
            let caja = Rect::from_min_size(Pos2::new(x - ancho, centro_y - 20.0), Vec2::new(ancho, 40.0));
            let r = en_rect(ui, caja, |ui| {
                Boton::new(texto).variante(Variante::Esmeralda).icono(icono).tam_icono(24.0).texto_tam(Tam::Base).ancho(ancho).ui(ui)
            });
            if r.clicked() {
                accion = Some(a);
            }
            x -= ancho + 16.0;
        }
        accion
    }
}

fn menu_usuario(boton: &Response, nombre: &str, email: &str) -> Option<Option<AccionCabecera>> {
    ui::menu(boton, 213.0, |ui| {
        let (caja, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 42.0), Sense::hover());
        ui.painter().text(caja.left_top() + Vec2::new(8.0, 6.0), egui::Align2::LEFT_TOP, nombre, fuente(Tam::Sm, Peso::Medio), color::TEXTO);
        ui.painter().text(caja.left_top() + Vec2::new(8.0, 24.0), egui::Align2::LEFT_TOP, email, fuente(Tam::Xs, Peso::Normal), color::APAGADO_TEXTO);
        ui::separador_menu(ui);
        let mut accion = None;
        if ui::item_menu(ui, Some("printer"), "Impresión").clicked() {
            accion = Some(AccionCabecera::Impresion);
        }
        if ui::item_menu(ui, Some("scale"), "Balanza").clicked() {
            accion = Some(AccionCabecera::Balanza);
        }
        if ui::item_menu_colores(ui, Some("log-out"), "Cerrar sesión", Some((paleta::RED_700, paleta::RED_800))).clicked() {
            accion = Some(AccionCabecera::CerrarSesion);
        }
        if accion.is_some() {
            ui.close();
        }
        accion
    })
}

pub fn item_carrito(ui: &mut Ui, item: &Producto, negocio: Option<&Value>, permite_borrar: bool) -> bool {
    let ancho = ui.available_width();
    let pintor = ui.painter().clone();
    let unitario = precio_visual(item.price_per_unit, item.iva_incluido, item.iva_porcentaje, negocio);
    let subtotal = unitario * item.quantity;
    let lista_por_defecto = negocio.and_then(|n| n["listaPrecioPorDefectoId"].as_i64());
    let insignia = item.lista_precio_id.filter(|id| Some(*id) != lista_por_defecto).map(|id| {
        let nombre = item.lista_precio_nombre.as_deref().map(str::trim).filter(|n| !n.is_empty());
        Texto::new(nombre.map(str::to_string).unwrap_or_else(|| format!("Lista #{id}")), Tam::Xs, Peso::Normal, color::SECUNDARIO_TEXTO)
    });
    let ancho_insignia = insignia.as_ref().map_or(0.0, |t| t.maximo(&pintor) + 20.0 + 2.0);
    let nombre = Texto::new(item.name.clone(), Tam::X2, Peso::Medio, color::TEXTO);
    let etiqueta_precio = if item.unit == "Kg" { "$/Kg" } else { "$/U" };
    let detalles = [
        Texto::new(format!("Cantidad: {} {}", numero_js(item.quantity), item.unit), Tam::Base, Peso::Normal, color::APAGADO_TEXTO),
        Texto::new(format!("{etiqueta_precio}: ${}", formato_local(unitario)), Tam::Base, Peso::Normal, color::APAGADO_TEXTO),
        Texto::new(format!("SUBTOTAL: ${}", formato_local(subtotal)), Tam::Base, Peso::Normal, color::APAGADO_TEXTO),
    ];
    let maximo_nombre = nombre.maximo(&pintor) + if ancho_insignia > 0.0 { 8.0 + ancho_insignia } else { 0.0 };
    let minimo_nombre = nombre.minimo(&pintor).max(ancho_insignia);
    let mut bases = vec![maximo_nombre];
    let mut minimos = vec![minimo_nombre];
    for d in &detalles {
        bases.push(d.maximo(&pintor));
        minimos.push(d.minimo(&pintor));
    }
    let separacion = 40.0 * 3.0;
    let interior = ancho - 34.0;
    let ancho_boton = if permite_borrar { 32.0 } else { 0.0 };
    let bloque = encoger(
        &[bases.iter().sum::<f32>() + separacion, ancho_boton],
        &[minimos.iter().sum::<f32>() + separacion, if permite_borrar { 24.0 } else { 0.0 }],
        interior,
    );
    let columnas = encoger(&bases, &minimos, bloque[0] - separacion);
    let galeria_nombre = nombre.galeria(&pintor, columnas[0]);
    let insignia_en_linea = columnas[0] + 0.5 >= maximo_nombre;
    let alto_nombre = match &insignia {
        Some(_) if !insignia_en_linea => nombre.alto(&galeria_nombre) + 8.0 + 22.0,
        _ => nombre.alto(&galeria_nombre),
    };
    let galerias: Vec<_> = detalles.iter().zip(&columnas[1..]).map(|(d, w)| d.galeria(&pintor, *w)).collect();
    let altos: Vec<f32> = detalles.iter().zip(&galerias).map(|(d, g)| d.alto(g)).collect();
    let alto_bloque = altos.iter().copied().fold(alto_nombre, f32::max);
    let alto_contenido = alto_bloque.max(32.0);
    let (caja, _) = ui.allocate_exact_size(Vec2::new(ancho, alto_contenido + 34.0), Sense::hover());
    ui.painter().add(ui::sombra_sm().as_shape(caja, ui::RADIO_XL));
    ui.painter().rect(caja, ui::RADIO_XL, color::FONDO, borde(color::BORDE_ENTRADA), StrokeKind::Inside);
    let contenido = caja.shrink(17.0);
    let base_y = contenido.center().y + alto_bloque / 2.0;
    let mut x = contenido.left();
    let y_nombre = base_y - alto_nombre;
    nombre.pintar(&pintor, &galeria_nombre, Pos2::new(x, y_nombre));
    if let Some(insignia) = &insignia {
        let ancho_linea = galeria_nombre.rows.last().map_or(0.0, |f| f.rect().width());
        let (ix, centro) = if insignia_en_linea {
            (x + ancho_linea + 8.0, y_nombre + nombre.alto(&galeria_nombre) - 16.0)
        } else {
            (x, base_y - 11.0)
        };
        let r = Rect::from_min_size(Pos2::new(ix, centro - 11.0), Vec2::new(ancho_insignia, 22.0));
        ui.painter().rect(r, 11.0, color::SECUNDARIO, egui::Stroke::NONE, StrokeKind::Inside);
        let g = insignia.galeria(&pintor, f32::INFINITY);
        insignia.pintar(&pintor, &g, Pos2::new(r.left() + 11.0, r.center().y - 8.0));
    }
    x += columnas[0] + 40.0;
    for ((d, g), (w, alto)) in detalles.iter().zip(&galerias).zip(columnas[1..].iter().zip(&altos)) {
        d.pintar(&pintor, g, Pos2::new(x, base_y - alto));
        x += w + 40.0;
    }
    if !permite_borrar {
        return false;
    }
    let boton = Rect::from_min_size(Pos2::new(contenido.right() - bloque[1], contenido.center().y - 16.0), Vec2::new(bloque[1], 32.0));
    en_rect(ui, boton, |ui| {
        Boton::new("").variante(Variante::Cancelar).tam(TamBoton::Icono).icono("trash-2").tam_icono(16.0).ancho(bloque[1]).alto(32.0).ui(ui)
    })
    .on_hover_text("Eliminar producto")
    .clicked()
}

pub struct Resumen {
    pub alto: f32,
    ancho_cancelar: f32,
    ancho_pagar: f32,
    ancho_izquierda: f32,
    ancho_total: f32,
    texto: Texto,
}

impl Resumen {
    pub fn medir(ui: &Ui, total: f64, ancho: f32) -> Self {
        let pintor = ui.painter().clone();
        let ancho_cancelar = 112.0 + ancho_texto(ui, "CANCELAR", Tam::Xl, Peso::Medio);
        let ancho_pagar = 112.0 + ancho_texto(ui, "PAGAR", Tam::Xl, Peso::Medio);
        let texto = Texto::new(format!("TOTAL: ${}", formato_local(total)), Tam::X4, Peso::Semi, color::TEXTO);
        let anchos = encoger(
            &[ancho_cancelar + 8.0 + ancho_pagar, texto.maximo(&pintor) + 66.0],
            &[ancho_cancelar.max(ancho_pagar), texto.minimo(&pintor) + 66.0],
            ancho,
        );
        let partido = anchos[0] + 0.5 < ancho_cancelar + 8.0 + ancho_pagar;
        let lineas = texto.galeria(&pintor, anchos[1] - 66.0).rows.len().max(1) as f32;
        let alto_izquierda: f32 = if partido { 129.0 } else { 64.0 };
        Resumen {
            alto: alto_izquierda.max(lineas * 40.0 + 50.0),
            ancho_cancelar,
            ancho_pagar,
            ancho_izquierda: anchos[0],
            ancho_total: anchos[1],
            texto,
        }
    }

    pub fn mostrar(&self, ui: &mut Ui, caja: Rect) -> (bool, bool) {
        let partido = self.ancho_izquierda + 0.5 < self.ancho_cancelar + 8.0 + self.ancho_pagar;
        let alto_izquierda = if partido { 129.0 } else { 64.0 };
        let arriba = caja.center().y - alto_izquierda / 2.0;
        let cancelar_caja = Rect::from_min_size(Pos2::new(caja.left(), arriba), Vec2::new(self.ancho_cancelar, 64.0));
        let pagar_caja = if partido {
            Rect::from_min_size(Pos2::new(caja.left() + 8.0, arriba + 65.0), Vec2::new(self.ancho_pagar, 64.0))
        } else {
            Rect::from_min_size(Pos2::new(cancelar_caja.right() + 8.0, arriba), Vec2::new(self.ancho_pagar, 64.0))
        };
        let boton = |ui: &mut Ui, texto: &str, variante: Variante, ancho: f32| {
            Boton::new(texto).variante(variante).texto_tam(Tam::Xl).radio(ui::RADIO_XL).ancho(ancho).alto(64.0).ui(ui).clicked()
        };
        let cancelar = en_rect(ui, cancelar_caja, |ui| boton(ui, "CANCELAR", Variante::Cancelar, self.ancho_cancelar));
        let pagar = en_rect(ui, pagar_caja, |ui| boton(ui, "PAGAR", Variante::Esmeralda, self.ancho_pagar));
        let pintor = ui.painter().clone();
        let galeria = self.texto.galeria(&pintor, self.ancho_total - 66.0);
        let alto = self.texto.alto(&galeria) + 50.0;
        let total = Rect::from_min_size(Pos2::new(caja.right() - self.ancho_total, caja.center().y - alto / 2.0), Vec2::new(self.ancho_total, alto));
        pintor.add(ui::sombra_sm().as_shape(total, 16));
        pintor.rect(total, 16, color::FONDO, borde(color::BORDE_ENTRADA), StrokeKind::Inside);
        self.texto.pintar(&pintor, &galeria, total.min + Vec2::new(33.0, 25.0));
        (cancelar, pagar)
    }
}
