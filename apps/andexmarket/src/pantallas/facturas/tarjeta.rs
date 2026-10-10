use eframe::egui::{self, Align2, Color32, Rect, Ui, Vec2};
use nucleo::facturas::{self as logica, EstadoTarjeta};
use nucleo::formato;
use serde_json::Value;

use super::comun::{self, bloque, Boton, Insignia, Relleno};
use super::{id_texto, numero, texto};
use crate::ui::{self, color, paleta, Peso, Tam};

const BORDE: Color32 = color::BORDE_ENTRADA;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Accion {
    Ver,
    Editar,
    Descargar,
    Eliminar,
    VerPagos,
    AgregarPago,
}

fn insignia_tipo(tipo: &str) -> Insignia<'_> {
    match tipo.to_lowercase().as_str() {
        "remito" => Insignia::new("Remito", paleta::ORANGE_100, paleta::ORANGE_800, paleta::ORANGE_200),
        "a" => Insignia::new("Factura A", paleta::BLUE_100, paleta::BLUE_800, paleta::BLUE_200),
        "c" => Insignia::new("Factura C", paleta::PURPLE_100, paleta::PURPLE_800, paleta::PURPLE_200),
        _ => Insignia::new(tipo, paleta::GRAY_100, paleta::GRAY_800, paleta::GRAY_200),
    }
}

fn insignia_estado(estado: &EstadoTarjeta) -> Insignia<'_> {
    match estado {
        EstadoTarjeta::SegunBackend { pagada: true, .. } => {
            Insignia::new("Pagada", paleta::GRAY_100, paleta::GRAY_800, paleta::GRAY_200).icono("check-circle")
        }
        EstadoTarjeta::SegunBackend { texto, .. } => {
            Insignia::new(texto, paleta::GRAY_100, paleta::GRAY_800, paleta::GRAY_200).icono("clock")
        }
        EstadoTarjeta::Pagada => Insignia::new("Pagada", paleta::GREEN_100, paleta::GREEN_800, paleta::GREEN_200).icono("check-circle"),
        EstadoTarjeta::Parcial => Insignia::new("Pendiente", paleta::YELLOW_100, paleta::YELLOW_800, paleta::YELLOW_200).icono("clock"),
        EstadoTarjeta::Impaga => Insignia::new("Pendiente", paleta::RED_100, paleta::RED_800, paleta::RED_200).icono("xcircle"),
    }
}

pub fn alto(ui: &Ui, factura: &Value, ancho: f32) -> f32 {
    tarjeta(ui, factura, Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(ancho, 0.0)), false).0
}

pub fn pintar(ui: &Ui, factura: &Value, rect: Rect) -> Option<Accion> {
    tarjeta(ui, factura, rect, true).1
}

fn marco(ui: &Ui, rect: Rect) {
    ui.painter().add(ui::sombra_sm().as_shape(rect, 16));
    ui.painter().rect_filled(rect, 16, BORDE);
    let izquierda = Rect::from_min_max(rect.min, egui::pos2(rect.left() + 14.0, rect.bottom()));
    ui.painter().with_clip_rect(izquierda).rect_filled(rect, 16, paleta::EMERALD_800);
    let interior = Rect::from_min_max(rect.min + Vec2::new(4.0, 1.0), rect.max - Vec2::new(1.0, 1.0));
    ui.painter().rect_filled(interior, egui::CornerRadius { nw: 12, sw: 12, ne: 15, se: 15 }, color::FONDO);
}

fn tarjeta(ui: &Ui, factura: &Value, rect: Rect, dibujar: bool) -> (f32, Option<Accion>) {
    let id = id_texto(&factura["id"]);
    let ancho = rect.width() - 4.0 - 1.0 - 48.0;
    let x0 = rect.left() + 28.0;
    let mut y = rect.top() + 25.0;
    let mut accion = None;
    if dibujar {
        marco(ui, rect);
    }

    let numero_factura = texto(factura, "numero");
    let cliente = logica::nombre_o_razon(&factura["cliente"]);
    let nombre_cliente = cliente.clone().unwrap_or_else(|| "Cliente no especificado".into());
    let tipo = insignia_tipo(texto(factura, "tipoFactura"));
    let estado_tarjeta = logica::estado_tarjeta(factura);
    let estado = insignia_estado(&estado_tarjeta);
    let avatar: String = {
        let chars: Vec<char> = numero_factura.chars().collect();
        chars[chars.len().saturating_sub(2)..].iter().collect()
    };

    let titulo_nat = comun::ancho_texto(ui, numero_factura, Tam::Lg, Peso::Semi)
        .max(comun::ancho_texto(ui, &nombre_cliente, Tam::Sm, Peso::Normal));
    let titulo_min = comun::ancho_minimo(ui, numero_factura, Tam::Lg, Peso::Semi)
        .max(comun::ancho_minimo(ui, &nombre_cliente, Tam::Sm, Peso::Normal));
    let avatar_min = comun::ancho_texto(ui, &avatar, Tam::Base, Peso::Semi);
    let derecha_nat = tipo.ancho_natural(ui) + 8.0 + estado.ancho_natural(ui);
    let derecha_min = tipo.ancho_minimo(ui) + 8.0 + estado.ancho_minimo(ui);
    let [izq, der] = comun::encoger(&[52.0 + titulo_nat, derecha_nat], &[avatar_min + 12.0 + titulo_min, derecha_min], ancho)[..]
    else {
        unreachable!()
    };
    let [ancho_avatar, ancho_titulo] = comun::encoger(&[40.0, titulo_nat], &[avatar_min, titulo_min], izq - 12.0)[..] else {
        unreachable!()
    };
    let [ancho_tipo, ancho_estado] = comun::encoger(
        &[tipo.ancho_natural(ui), estado.ancho_natural(ui)],
        &[tipo.ancho_minimo(ui), estado.ancho_minimo(ui)],
        der - 8.0,
    )[..] else {
        unreachable!()
    };

    let titulo = bloque(ui, numero_factura, Tam::Lg, Peso::Semi, paleta::GRAY_900, ancho_titulo + 0.5);
    let subtitulo = bloque(ui, &nombre_cliente, Tam::Sm, Peso::Normal, paleta::GRAY_600, ancho_titulo + 0.5);
    let alto_titulos = titulo.alto() + subtitulo.alto();
    let alto_izq = alto_titulos.max(40.0);
    let alto_der = tipo.alto(ui, ancho_tipo).max(estado.alto(ui, ancho_estado));
    let alto_cabecera = alto_izq.max(alto_der);
    if dibujar {
        let avatar_rect = Rect::from_min_size(egui::pos2(x0, y + (alto_izq - 40.0) / 2.0), Vec2::new(ancho_avatar, 40.0));
        comun::pintar_relleno(ui, avatar_rect, avatar_rect.width().min(40.0) / 2.0, comun::esmeralda(), 1.0);
        comun::texto_en(ui, avatar_rect.center(), Align2::CENTER_CENTER, &avatar, Tam::Base, Peso::Semi, Color32::WHITE);
        let tx = x0 + ancho_avatar + 12.0;
        let ty = y + (alto_izq - alto_titulos) / 2.0;
        titulo.pintar(ui, egui::pos2(tx, ty), paleta::GRAY_900);
        subtitulo.pintar(ui, egui::pos2(tx, ty + titulo.alto()), paleta::GRAY_600);
        let bx = x0 + ancho - der;
        tipo.pintar(ui, Rect::from_min_size(egui::pos2(bx, y), Vec2::new(ancho_tipo, alto_der)));
        estado.pintar(ui, Rect::from_min_size(egui::pos2(bx + ancho_tipo + 8.0, y), Vec2::new(ancho_estado, alto_der)));
    }
    y += alto_cabecera + 16.0;

    let icono_texto = |y: f32, icono: &str, tam_icono: f32, contenido: &str, tam: Tam, peso: Peso, color: Color32, separacion: f32| {
        let alto = tam.alto_linea();
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x0 + tam_icono / 2.0, y + alto / 2.0), Vec2::splat(tam_icono)), icono, color);
        comun::texto_en(ui, egui::pos2(x0 + tam_icono + separacion, y + alto / 2.0), Align2::LEFT_CENTER, contenido, tam, peso, color)
    };
    if dibujar {
        icono_texto(y, "calendar", 16.0, &logica::fecha_es_ar(texto(factura, "fecha")), Tam::Sm, Peso::Normal, paleta::GRAY_600, 8.0);
    }
    y += 20.0 + 12.0;
    if dibujar {
        let total = formato::pesos(numero(factura, "total"));
        icono_texto(y, "dollar-sign", 20.0, &total, Tam::Lg, Peso::Semi, paleta::GRAY_900, 8.0);
    }
    y += 28.0 + 12.0;
    let saldo = logica::saldo_pendiente(factura);
    if logica::tiene_pagos(factura) {
        if dibujar {
            let pagado = logica::total_pagado(factura);
            let fila = |y: f32, etiqueta: &str, valor: f64, color_valor: Color32| {
                comun::texto_en(ui, egui::pos2(x0, y + 10.0), Align2::LEFT_CENTER, etiqueta, Tam::Sm, Peso::Normal, paleta::GRAY_600);
                comun::texto_en(
                    ui,
                    egui::pos2(x0 + ancho, y + 10.0),
                    Align2::RIGHT_CENTER,
                    &formato::pesos(valor),
                    Tam::Sm,
                    Peso::Medio,
                    color_valor,
                );
            };
            fila(y, "Total pagado:", pagado, paleta::GREEN_600);
            fila(y + 24.0, "Saldo pendiente:", saldo, if saldo > 0.0 { paleta::RED_600 } else { paleta::GREEN_600 });
        }
        y += 44.0 + 12.0;
    }
    if dibujar {
        let cantidad = factura["detalles"].as_array().map_or(0, Vec::len);
        let r = icono_texto(y, "user", 16.0, cliente.as_deref().unwrap_or("N/A"), Tam::Sm, Peso::Normal, paleta::GRAY_600, 4.0);
        let x = r.right() + 16.0;
        ui::pintar_icono_en(ui, Rect::from_center_size(egui::pos2(x + 8.0, y + 10.0), Vec2::splat(16.0)), "package", paleta::GRAY_600);
        let productos = format!("{cantidad} producto{}", if cantidad != 1 { "s" } else { "" });
        comun::texto_en(ui, egui::pos2(x + 20.0, y + 10.0), Align2::LEFT_CENTER, &productos, Tam::Sm, Peso::Normal, paleta::GRAY_600);
    }
    y += 20.0 + 16.0 + 8.0;

    if dibujar {
        let contorno = |texto: &'static str, icono: &'static str| {
            Boton::contorno(texto)
                .chico()
                .icono(icono)
                .separacion(12.0)
                .texto_tam(Tam::Xs)
                .borde(Some(BORDE))
                .radio(12.0)
        };
        let ancho_flexible = (ancho - 32.0 * 2.0 - 4.0 * 3.0) / 2.0;
        let mut x = x0;
        let caja = |x: f32, w: f32| Rect::from_min_size(egui::pos2(x, y), Vec2::new(w, 36.0));
        if contorno("Ver", "eye").en(ui, caja(x, ancho_flexible), (&id, "ver")).clicked() {
            accion = Some(Accion::Ver);
        }
        x += ancho_flexible + 4.0;
        if contorno("Editar", "edit").en(ui, caja(x, ancho_flexible), (&id, "editar")).clicked() {
            accion = Some(Accion::Editar);
        }
        x += ancho_flexible + 4.0;
        let solo_icono = |icono: &'static str, c: Color32, hover_c: Color32, hover_f: Color32| {
            Boton::contorno("")
                .chico()
                .icono(icono)
                .relleno_x(8.0)
                .color(c)
                .color_encima(hover_c)
                .encima(Relleno::Liso(hover_f))
                .borde(Some(BORDE))
                .radio(12.0)
        };
        if solo_icono("download", paleta::BLUE_600, paleta::BLUE_700, paleta::BLUE_50).en(ui, caja(x, 32.0), (&id, "pdf")).clicked() {
            accion = Some(Accion::Descargar);
        }
        x += 36.0;
        if solo_icono("trash-2", paleta::RED_600, paleta::RED_700, paleta::RED_50).en(ui, caja(x, 32.0), (&id, "borrar")).clicked() {
            accion = Some(Accion::Eliminar);
        }
    }
    y += 36.0 + 8.0;
    if dibujar {
        let mitad = if saldo > 0.0 { (ancho - 4.0) / 2.0 } else { ancho };
        let ver_pagos = Boton::contorno("Ver Pagos")
            .chico()
            .icono("dollar-sign")
            .separacion(12.0)
            .texto_tam(Tam::Xs)
            .borde(Some(paleta::GREEN_200))
            .color(paleta::GREEN_700)
            .encima(Relleno::Liso(paleta::GREEN_50))
            .radio(12.0);
        if ver_pagos.en(ui, Rect::from_min_size(egui::pos2(x0, y), Vec2::new(mitad, 36.0)), (&id, "pagos")).clicked() {
            accion = Some(Accion::VerPagos);
        }
        if saldo > 0.0 {
            let agregar = Boton::esmeralda("Agregar Pago")
                .chico()
                .icono("dollar-sign")
                .separacion(12.0)
                .texto_tam(Tam::Xs)
                .radio(12.0);
            let caja = Rect::from_min_size(egui::pos2(x0 + mitad + 4.0, y), Vec2::new(mitad, 36.0));
            if agregar.en(ui, caja, (&id, "agregar")).clicked() {
                accion = Some(Accion::AgregarPago);
            }
        }
    }
    y += 36.0 + 24.0 + 1.0;
    (y - rect.top(), accion)
}
