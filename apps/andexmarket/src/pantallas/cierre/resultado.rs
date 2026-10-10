use eframe::egui::{self, Align2, Frame, Rect, ScrollArea, Sense, Stroke, Ui, Vec2};
use nucleo::cierre::{self as logica, numero, texto_valor};
use nucleo::formato;
use serde_json::Value;

use super::dibujo::{dinero, hora_corta, metodo_pago};
use crate::app::Contexto;
use crate::ui::marca::{color, fuente, radio, tam};
use crate::ui::v2::{self, tarjetas::celda_texto, Boton, Columna, Tabla};
use crate::ui::{pintar_icono_solar_en, Peso};

const COLORES_BARRA: [eframe::egui::Color32; 4] = [color::VERDE, color::VERDE_PROFUNDO, color::DORADO, color::SOBRE_OSCURO_TENUE];

#[derive(Default)]
pub struct DialogoResultadoCierre {
    abierto: bool,
    resultado: Option<Value>,
}

fn texto(valor: Option<&Value>) -> String {
    match valor {
        None | Some(Value::Null) | Some(Value::Bool(_)) => String::new(),
        Some(otro) => texto_valor(otro),
    }
}

fn importe(valor: Option<&Value>) -> f64 {
    let valor = match valor {
        Some(Value::Object(objeto)) => objeto.get("total"),
        otro => otro,
    };
    Some(numero(valor)).filter(|n| n.is_finite()).unwrap_or(0.0)
}

fn monto(valor: Option<&Value>) -> String {
    dinero(importe(valor))
}

fn conteo(valor: Option<&Value>) -> Option<f64> {
    let valor = match valor {
        Some(Value::Object(objeto)) => objeto.get("cantidad"),
        otro => otro,
    };
    Some(numero(valor)).filter(|n| n.is_finite())
}

fn cantidad(valor: Option<&Value>) -> String {
    conteo(valor).map(formato::numero).unwrap_or_else(|| "0".into())
}

fn periodo(valor: &Value) -> String {
    match valor.as_str() {
        Some("custom") => "Manual".into(),
        Some("todo") => "Todo el día".into(),
        Some("mañana") => "Turno mañana".into(),
        Some("tarde") => "Turno tarde".into(),
        _ => texto_valor(valor),
    }
}

fn momento(valor: &Value) -> String {
    logica::fecha_literal(valor.as_str().unwrap_or_default())
        .map(|fecha| format!("{} {}", fecha.format("%d/%m/%Y"), hora_corta(fecha)))
        .unwrap_or_else(|| "Fecha inválida".into())
}

fn subtitulo(resultado: &Value) -> String {
    format!("{} · {} a {}", periodo(&resultado["periodo"]), momento(&resultado["fechaInicio"]), momento(&resultado["fechaCierre"]))
}

fn ticket_promedio(resultado: &Value) -> String {
    match conteo(resultado.get("cantidadVentas")).filter(|n| *n > 0.0) {
        Some(ventas) => dinero(importe(resultado.get("totalVentas")) / ventas),
        None => "—".into(),
    }
}

fn estadisticas(ui: &mut Ui, resultado: &Value) {
    let datos = [
        ("Total ventas", monto(resultado.get("totalVentas")), color::VERDE),
        ("Cantidad de ventas", cantidad(resultado.get("cantidadVentas")), color::TEXTO),
        ("Ticket promedio", ticket_promedio(resultado), color::TEXTO),
    ];
    Frame::NONE.stroke(Stroke::new(1.0, color::BORDE_SUAVE)).corner_radius(radio::POPOVER).show(ui, |ui| {
        let ancho = (ui.available_width() - 2.0) / 3.0;
        ui.horizontal(|ui| {
            for (indice, (etiqueta, valor, tinta)) in datos.into_iter().enumerate() {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 76.0), Sense::hover());
                if indice > 0 {
                    ui.painter().vline(rect.left(), rect.y_range(), Stroke::new(1.0, color::BORDE_SUAVE));
                }
                v2::pintar_texto(ui, rect.min + Vec2::new(16.0, 14.0), Align2::LEFT_TOP, etiqueta, tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
                let g = v2::galeria_espaciada(ui, valor, fuente(tam::MONTO_CHICO, Peso::Semi), tinta, -0.01 * tam::MONTO_CHICO);
                ui.painter().galley(rect.min + Vec2::new(16.0, 33.0), g, tinta);
            }
        });
    });
}

fn metodos_de_pago(ui: &mut Ui, metodos: &serde_json::Map<String, Value>) {
    v2::texto(ui, "Por método de pago", tam::CUERPO, Peso::Semi, color::TEXTO);
    ui.add_space(6.0);
    let maximo = metodos.values().map(|v| importe(Some(v))).fold(1.0, f64::max);
    let ancho = ui.available_width();
    for (indice, (clave, valor)) in metodos.iter().enumerate() {
        ui.add_space(4.0);
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ancho, 30.0), Sense::hover());
        let (etiqueta, icono, _) = metodo_pago(clave);
        pintar_icono_solar_en(ui, Rect::from_center_size(egui::pos2(fila.left() + 8.0, fila.center().y), Vec2::splat(16.0)), icono, color::VERDE);
        v2::pintar_texto(ui, egui::pos2(fila.left() + 24.0, fila.center().y), Align2::LEFT_CENTER, etiqueta, tam::CONTROL, Peso::Normal, color::TEXTO);
        let barra = Rect::from_min_max(egui::pos2(fila.left() + 142.0, fila.center().y - 4.0), egui::pos2(fila.right() - 132.0, fila.center().y + 4.0));
        ui.painter().rect_filled(barra, radio::PASTILLA, color::VERDE_CHIP);
        let proporcion = (importe(Some(valor)) / maximo).clamp(0.0, 1.0) as f32;
        if proporcion > 0.0 {
            let lleno = Rect::from_min_size(barra.min, Vec2::new(barra.width() * proporcion, barra.height()));
            ui.painter().rect_filled(lleno, radio::PASTILLA, COLORES_BARRA[indice % COLORES_BARRA.len()]);
        }
        v2::pintar_texto(ui, fila.right_center(), Align2::RIGHT_CENTER, monto(Some(valor)), tam::CONTROL, Peso::Semi, color::TEXTO);
    }
}

fn vendedores(ui: &mut Ui, lista: &[Value]) {
    let columnas = [Columna::fraccion(1.0, 0.0), Columna::fija(80.0).derecha(), Columna::fija(130.0).derecha()];
    let tabla = Tabla::new(&columnas).relleno(16.0).radio_superior(radio::POPOVER - 1);
    Frame::NONE.stroke(Stroke::new(1.0, color::BORDE_SUAVE)).corner_radius(radio::POPOVER).show(ui, |ui| {
        ui.set_width(ui.available_width());
        tabla.encabezado(ui, &["Vendedor", "Ventas", "Total"], 38.0);
        for vendedor in lista {
            tabla.fila(ui, 48.0, None, |ui, celdas| {
                let pintor = ui.painter();
                let nombre = v2::galeria(ui, texto(vendedor.get("nombre")), fuente(tam::CONTROL, Peso::Medio), color::TEXTO);
                let correo = v2::galeria(ui, texto(vendedor.get("email")), fuente(tam::XS, Peso::Normal), color::TEXTO_TENUE);
                let y = celdas[0].center().y - (nombre.size().y + correo.size().y) / 2.0;
                let alto_nombre = nombre.size().y;
                pintor.with_clip_rect(celdas[0]).galley(egui::pos2(celdas[0].left(), y), nombre, color::TEXTO);
                pintor.with_clip_rect(celdas[0]).galley(egui::pos2(celdas[0].left(), y + alto_nombre), correo, color::TEXTO_TENUE);
                celda_texto(pintor, celdas[1], &cantidad(vendedor.get("cantidadVentas")), tam::CONTROL, Peso::Normal, color::TEXTO, true);
                celda_texto(pintor, celdas[2], &monto(vendedor.get("totalVentas")), tam::CONTROL, Peso::Semi, color::TEXTO, true);
            });
        }
    });
}

impl DialogoResultadoCierre {
    pub fn abrir(&mut self, resultado: Value) {
        self.resultado = Some(resultado);
        self.abierto = true;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        let Some(resultado) = self.resultado.as_ref().filter(|_| self.abierto) else {
            return;
        };
        let alto_maximo = (cx.egui.content_rect().height() - 48.0 - 86.0 - 82.0).max(160.0);
        let subtitulo = subtitulo(resultado);
        let mut cerrar = false;
        let respuesta = v2::Modal::new("cierre-resultado", "Resultado del cierre").subtitulo(&subtitulo).icono("bill-check").ancho(640.0).mostrar(&cx.egui, |ui| {
            ScrollArea::vertical().max_height(alto_maximo).auto_shrink([false, true]).show(ui, |ui| {
                v2::cuerpo_modal(ui, 18, |ui| {
                    estadisticas(ui, resultado);
                    if let Some(metodos) = resultado["ventasPorMetodo"].as_object().filter(|_| logica::verdadero(resultado.get("ventasPorMetodo"))) {
                        ui.add_space(14.0);
                        metodos_de_pago(ui, metodos);
                    }
                    if let Some(lista) = resultado["ventasPorVendedor"].as_array().filter(|v| !v.is_empty()) {
                        ui.add_space(14.0);
                        vendedores(ui, lista);
                    }
                });
            });
            v2::pie_modal(ui, |ui| cerrar = Boton::primario("Listo").modal().ui(ui).clicked());
        });
        if cerrar || respuesta.cerrar {
            self.abierto = false;
        }
    }
}

#[cfg(test)]
mod pruebas {
    use serde_json::json;

    use super::*;

    #[test]
    fn montos_y_cantidades_sin_nan() {
        assert_eq!(monto(Some(&json!(186450))), "$186.450");
        assert_eq!(monto(Some(&json!({ "total": 8500.5, "cantidad": 3 }))), "$8.500,5");
        assert_eq!(cantidad(Some(&json!({ "total": 8500.5, "cantidad": 3 }))), "3");
        assert_eq!(monto(None), "$0");
        assert_eq!(monto(Some(&json!("abc"))), "$0");
        assert_eq!(monto(Some(&json!({}))), "$0");
        assert_eq!(cantidad(Some(&json!(1234))), "1.234");
        assert_eq!(cantidad(None), "0");
        assert_eq!(periodo(&json!("custom")), "Manual");
        assert_eq!(periodo(&json!("mañana")), "Turno mañana");
        assert_eq!(periodo(&json!("todo")), "Todo el día");
    }

    #[test]
    fn subtitulo_y_promedio_como_el_diseno() {
        let cierre = json!({ "periodo": "todo", "fechaInicio": "2026-10-10T00:00:00.000Z", "fechaCierre": "2026-10-10T23:59:59.999Z", "totalVentas": 80750, "cantidadVentas": 11 });
        assert_eq!(subtitulo(&cierre), "Todo el día · 10/10/2026 12:00 a. m. a 10/10/2026 11:59 p. m.");
        assert_eq!(ticket_promedio(&cierre), "$7.340,91");
        assert_eq!(ticket_promedio(&json!({ "totalVentas": 0, "cantidadVentas": 0 })), "—");
    }
}
