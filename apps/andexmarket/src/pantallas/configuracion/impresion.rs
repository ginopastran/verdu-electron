use chrono::{SecondsFormat, Utc};
use eframe::egui::Ui;
use hardware::impresora;
use serde_json::{json, Value};

use super::{campo, encabezado, parrafo, selector, texto_valor};
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, toast, Boton, Peso, Tam, Variante};

const POR_DEFECTO: &str = "Compartida como TP806L (por defecto)";
const ANCHOS: [u8; 2] = [80, 58];

pub fn datos_prueba(nombre_negocio: Value, ahora: &str) -> Value {
    json!({
        "id": 0,
        "idReal": 0,
        "businessName": nombre_negocio,
        "fecha": ahora,
        "createdAt": ahora,
        "vendedor": "Prueba",
        "metodoPago": "efectivo",
        "items": [{ "nombre": "IMPRESION DE PRUEBA", "cantidad": 1, "precio": 0, "subtotal": 0 }],
        "subtotal": 0,
        "total": 0,
    })
}

pub fn nombre_del_negocio(negocio: Option<&Value>, admin: Option<&Value>) -> Value {
    [negocio.map(|n| &n["nombre"]), negocio.map(|n| &n["name"]), admin.map(|a| &a["businessName"])]
        .into_iter()
        .flatten()
        .find(|valor| nucleo::sesion::es_verdadero(valor))
        .cloned()
        .unwrap_or(Value::Null)
}

pub fn mensaje_prueba(resultado: &Value) -> Result<(), String> {
    if resultado["success"] != false {
        return Ok(());
    }
    let mensaje = ["message", "printerError", "error"].iter().find_map(|clave| resultado[*clave].as_str().filter(|m| !m.is_empty()));
    Err(mensaje.unwrap_or("No se pudo imprimir").to_string())
}

#[derive(Default)]
pub struct DialogoImpresion {
    abierto: bool,
    cargado: bool,
    ancho_papel: u8,
    tiene_logo: bool,
    impresora: Option<String>,
    recurso: String,
    impresoras: Vec<String>,
    listando: Option<Tarea<Vec<String>>>,
    eligiendo: Option<Tarea<(Option<String>, Value)>>,
    imprimiendo: Option<Tarea<Value>>,
}

impl DialogoImpresion {
    pub fn abrir(&mut self) {
        self.abierto = true;
        self.cargado = false;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    fn cargar(&mut self, cx: &Contexto) {
        let config = &cx.servicios.config;
        self.ancho_papel = config.ancho_papel();
        self.tiene_logo = config.ruta_logo_negocio().exists();
        self.impresora = config.get("impresoraWindows").as_str().map(str::to_string);
        self.recurso = impresora::recurso(config);
        self.listando = Some(Tarea::lanzar(&cx.egui, || {
            impresora::listar().as_array().map(|lista| lista.iter().filter_map(|p| p["name"].as_str().map(str::to_string)).collect()).unwrap_or_default()
        }));
        self.cargado = true;
    }

    fn recibir(&mut self, cx: &Contexto) {
        if let Some(impresoras) = esperar(&mut self.listando) {
            self.impresoras = impresoras;
        }
        if let Some((elegida, resultado)) = esperar(&mut self.eligiendo) {
            if resultado["ok"] == true {
                self.impresora = elegida;
                self.recurso = texto_valor(&resultado["recurso"]);
                toast::exito(&cx.egui, "Impresora guardada");
            } else {
                toast::error(&cx.egui, resultado["error"].as_str().unwrap_or("No se pudo elegir la impresora"));
            }
        }
        if let Some(resultado) = esperar(&mut self.imprimiendo) {
            match mensaje_prueba(&resultado) {
                Ok(()) => toast::exito(&cx.egui, "Impresión de prueba enviada"),
                Err(mensaje) => toast::error(&cx.egui, mensaje),
            }
        }
    }

    fn elegir_impresora(&mut self, cx: &Contexto, elegida: Option<String>) {
        let servicios: Servicios = cx.servicios.clone();
        self.eligiendo = Some(Tarea::lanzar(&cx.egui, move || {
            let valor = elegida.clone().map(Value::String).unwrap_or(Value::Null);
            let resultado = impresora::elegir(&servicios.config, &valor);
            (elegida, resultado)
        }));
    }

    fn imprimir_prueba(&mut self, cx: &Contexto) {
        let servicios = cx.servicios.clone();
        let nombre = nombre_del_negocio(cx.global.negocio.as_ref(), cx.global.admin.as_ref());
        let datos = datos_prueba(nombre, &Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));
        self.imprimiendo = Some(Tarea::lanzar(&cx.egui, move || imprimir(&servicios, Comprobante::Venta, datos)));
    }

    fn contenido(&mut self, ui: &mut Ui, cx: &mut Contexto) -> bool {
        encabezado(ui, "Impresión", "Impresora térmica, ancho del papel y prueba del encabezado.");
        ui.add_space(16.0);

        let mut valores: Vec<Option<String>> = vec![None];
        let mut etiquetas = vec![POR_DEFECTO.to_string()];
        for nombre in &self.impresoras {
            valores.push(Some(nombre.clone()));
            etiquetas.push(nombre.clone());
        }
        if let Some(actual) = self.impresora.as_ref().filter(|actual| !self.impresoras.contains(actual)) {
            valores.push(Some(actual.clone()));
            etiquetas.push(format!("{actual} (no conectada)"));
        }
        let seleccion = valores.iter().position(|valor| *valor == self.impresora);
        let elegida = campo(ui, "Impresora", |ui| selector(ui, "impresora", seleccion, &etiquetas, "", self.eligiendo.is_none()));
        if let Some(indice) = elegida {
            self.elegir_impresora(cx, valores[indice].clone());
        }
        ui.add_space(4.0);
        parrafo(ui, &format!("Se imprime en «{}».", self.recurso), Tam::Xs, Peso::Normal, color::APAGADO_TEXTO, 16.0);
        ui.add_space(16.0);

        let etiquetas_ancho: Vec<String> = ANCHOS.iter().map(|ancho| format!("{ancho} mm")).collect();
        let seleccion_ancho = ANCHOS.iter().position(|ancho| *ancho == self.ancho_papel);
        if let Some(indice) = campo(ui, "Ancho del papel", |ui| selector(ui, "ancho-papel", seleccion_ancho, &etiquetas_ancho, "", true)) {
            self.ancho_papel = ANCHOS[indice];
            cx.servicios.config.set("anchoPapel", json!(self.ancho_papel));
        }
        ui.add_space(16.0);

        let logo = if self.tiene_logo {
            "Se usa el logo del negocio cargado en Administración → Configuración."
        } else {
            "Sin logo del negocio: se imprime el logo por defecto. Cargalo en Administración → Configuración."
        };
        parrafo(ui, logo, Tam::Sm, Peso::Normal, color::APAGADO_TEXTO, 20.0);
        ui.add_space(16.0);

        let mut cerrar = false;
        ui::pie_dialogo(ui, |ui| {
            if Boton::new("Impresión de prueba").cargando(self.imprimiendo.is_some()).ui(ui).clicked() {
                self.imprimir_prueba(cx);
            }
            cerrar = Boton::new("Cerrar").variante(Variante::Contorno).ui(ui).clicked();
        });
        cerrar
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        self.recibir(cx);
        if !self.abierto {
            return;
        }
        if !self.cargado {
            self.cargar(cx);
        }
        let ctx = cx.egui.clone();
        let respuesta = ui::Dialogo::new("impresion").ancho(448.0).mostrar(&ctx, |ui| self.contenido(ui, cx));
        if respuesta.cerrar || respuesta.interior {
            self.abierto = false;
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_ticket_de_prueba_lleva_los_datos_del_original() {
        let datos = datos_prueba(json!("Lucía"), "2026-10-10T15:00:00.000Z");
        assert_eq!(datos["businessName"], "Lucía");
        assert_eq!(datos["fecha"], datos["createdAt"]);
        assert_eq!(datos["vendedor"], "Prueba");
        assert_eq!(datos["items"][0]["nombre"], "IMPRESION DE PRUEBA");
        assert_eq!(datos["total"], 0);
    }

    #[test]
    fn el_ticket_de_prueba_lleva_el_nombre_del_negocio() {
        let negocio = json!({ "nombre": "Verdulería La Huerta" });
        let admin = json!({ "nombre": "Roberto", "businessName": "La Huerta SRL" });
        assert_eq!(nombre_del_negocio(Some(&negocio), Some(&admin)), "Verdulería La Huerta");
        assert_eq!(nombre_del_negocio(Some(&json!({ "sistemaPago": "redondeo" })), Some(&admin)), "La Huerta SRL");
        assert_eq!(nombre_del_negocio(None, None), Value::Null);
    }

    #[test]
    fn el_error_de_impresion_muestra_el_motivo() {
        assert_eq!(mensaje_prueba(&json!({ "success": true, "printerError": null })), Ok(()));
        assert_eq!(
            mensaje_prueba(&json!({ "success": false, "printerError": "sin papel", "message": "Error de impresión: sin papel" })),
            Err("Error de impresión: sin papel".to_string())
        );
        assert_eq!(mensaje_prueba(&json!({ "success": false, "printerError": "sin papel" })), Err("sin papel".to_string()));
        assert_eq!(mensaje_prueba(&json!({ "success": false })), Err("No se pudo imprimir".to_string()));
    }
}
