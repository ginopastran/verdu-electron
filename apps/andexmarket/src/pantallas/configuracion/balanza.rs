use eframe::egui::{self, Ui, Vec2};
use hardware::balanza;
use serde_json::{json, Value};

use super::{campo, encabezado, parrafo, selector, texto_valor};
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, toast, Boton, Peso, Tam, Variante};

type Puertos = Vec<(String, Option<String>)>;

const ORIGENES: [(&str, &str); 2] = [("archivo", "Archivo C:\\Peso\\peso.json"), ("serie", "Puerto serie (Croma / RS-232)")];
const VELOCIDADES: [u32; 6] = [1200, 2400, 4800, 9600, 19200, 115200];
const FORMATOS: [&str; 4] = ["8N1", "7E1", "7O1", "8E1"];
const LECTURAS: [(&str, &str); 2] = [("enq", "Por consulta"), ("continuo", "Continua")];
const UNIDADES: [(&str, &str); 2] = [("gramos", "Gramos"), ("kilos", "Kilos")];

pub fn peso_en_kilos(peso: &Value) -> String {
    let gramos = match peso {
        Value::Number(numero) => numero.as_f64().unwrap_or(0.0),
        Value::String(texto) => texto.trim().parse().unwrap_or(f64::NAN),
        Value::Bool(verdadero) => f64::from(u8::from(*verdadero)),
        _ => 0.0,
    };
    format!("{} kg", nucleo::formato::numero_con(gramos / 1000.0, 3, 3))
}

pub fn texto_puerto((ruta, fabricante): &(String, Option<String>)) -> String {
    match fabricante {
        Some(fabricante) if !fabricante.is_empty() => format!("{ruta} · {fabricante}"),
        _ => ruta.clone(),
    }
}

fn elegir(ui: &mut Ui, id: &str, actual: &Value, opciones: &[(String, String)], sugerencia: &str) -> Option<String> {
    let actual = texto_valor(actual);
    let seleccion = opciones.iter().position(|(valor, _)| *valor == actual);
    let etiquetas: Vec<String> = opciones.iter().map(|(_, etiqueta)| etiqueta.clone()).collect();
    selector(ui, id, seleccion, &etiquetas, sugerencia, true).map(|indice| opciones[indice].0.clone())
}

fn pares(opciones: &[(&str, &str)]) -> Vec<(String, String)> {
    opciones.iter().map(|(valor, etiqueta)| (valor.to_string(), etiqueta.to_string())).collect()
}

fn cuadricula(ui: &mut Ui, config: &mut Value) {
    let ancho = (ui.available_width() - 3.0 * 8.0) / 4.0;
    let mut cambios: Vec<(&str, Value)> = Vec::new();
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let columna = |ui: &mut Ui, contenido: &mut dyn FnMut(&mut Ui)| {
            ui.allocate_ui_with_layout(Vec2::new(ancho, 64.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(ancho);
                contenido(ui);
            });
        };
        let velocidades: Vec<(String, String)> = VELOCIDADES.iter().map(|v| (v.to_string(), v.to_string())).collect();
        columna(ui, &mut |ui| {
            if let Some(v) = campo(ui, "Velocidad", |ui| elegir(ui, "velocidad", &config["baudRate"], &velocidades, "")) {
                cambios.push(("baudRate", json!(v.parse::<u32>().unwrap_or(9600))));
            }
        });
        let formatos: Vec<(String, String)> = FORMATOS.iter().map(|f| (f.to_string(), f.to_string())).collect();
        columna(ui, &mut |ui| {
            if let Some(v) = campo(ui, "Formato", |ui| elegir(ui, "formato", &config["formato"], &formatos, "")) {
                cambios.push(("formato", json!(v)));
            }
        });
        columna(ui, &mut |ui| {
            if let Some(v) = campo(ui, "Lectura", |ui| elegir(ui, "lectura", &config["consulta"], &pares(&LECTURAS), "")) {
                cambios.push(("consulta", json!(v)));
            }
        });
        columna(ui, &mut |ui| {
            if let Some(v) = campo(ui, "Sin decimales", |ui| elegir(ui, "unidad", &config["unidadEntera"], &pares(&UNIDADES), "")) {
                cambios.push(("unidadEntera", json!(v)));
            }
        });
    });
    for (clave, valor) in cambios {
        config[clave] = valor;
    }
}

#[derive(Default)]
pub struct DialogoBalanza {
    abierto: bool,
    config: Option<Value>,
    puertos: Puertos,
    buscando_puertos: Option<Tarea<Puertos>>,
}

impl DialogoBalanza {
    pub fn abrir(&mut self) {
        self.abierto = true;
        self.config = None;
    }

    pub fn abierto(&self) -> bool {
        self.abierto
    }

    pub fn fijar_puertos(&mut self, puertos: Puertos) {
        self.buscando_puertos = None;
        self.puertos = puertos;
    }

    fn guardar(&self, cx: &Contexto) {
        let Some(config) = &self.config else {
            return;
        };
        cx.servicios.config.set("balanza", config.clone());
        cx.servicios.balanza.reiniciar_lectura();
        cx.servicios.balanza.iniciar(&cx.servicios.config);
        toast::exito(&cx.egui, "Configuración de balanza guardada");
    }

    fn contenido(&mut self, ui: &mut Ui, cx: &mut Contexto) -> bool {
        encabezado(ui, "Balanza", "Origen del peso para los productos por kilo.");
        ui.add_space(16.0);
        if let Some(config) = self.config.as_mut() {
            if let Some(modo) = campo(ui, "Origen", |ui| elegir(ui, "origen", &config["modo"], &pares(&ORIGENES), "")) {
                config["modo"] = json!(modo);
            }
            if config["modo"] == "serie" {
                ui.add_space(16.0);
                let puertos: Vec<(String, String)> = self.puertos.iter().map(|p| (p.0.clone(), texto_puerto(p))).collect();
                let sugerencia = if !texto_valor(&config["puerto"]).is_empty() {
                    ""
                } else if puertos.is_empty() {
                    "No se encontraron puertos"
                } else {
                    "Elegí un puerto"
                };
                if let Some(puerto) = campo(ui, "Puerto", |ui| elegir(ui, "puerto", &config["puerto"], &puertos, sugerencia)) {
                    config["puerto"] = json!(puerto);
                }
                ui.add_space(16.0);
                cuadricula(ui, config);
                ui.add_space(16.0);
                let estado = &cx.global.estado_balanza;
                let conectada = estado["conectada"] == true;
                let mut trabajo = egui::text::LayoutJob::default();
                let formato = |c| egui::TextFormat { line_height: Some(20.0), ..egui::TextFormat::simple(ui::fuente(Tam::Sm, Peso::Normal), c) };
                trabajo.append("Estado: ", 0.0, formato(color::TEXTO));
                trabajo.append(
                    if conectada { "conectada" } else { "desconectada" },
                    0.0,
                    formato(if conectada { paleta::EMERALD_600 } else { paleta::RED_600 }),
                );
                if let Some(error) = estado["error"].as_str().filter(|e| !e.is_empty()) {
                    trabajo.append(&format!(" ({error})"), 0.0, formato(color::TEXTO));
                }
                ui.add(egui::Label::new(trabajo).wrap());
            }
            ui.add_space(16.0);
            ui::marco()
                .stroke(ui::borde(color::BORDE))
                .corner_radius(ui::RADIO)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        parrafo(ui, "Peso actual", Tam::Xs, Peso::Normal, color::APAGADO_TEXTO, 16.0);
                        parrafo(ui, &peso_en_kilos(&cx.global.peso), Tam::X3, Peso::Negrita, color::TEXTO, 36.0);
                    });
                });
            ui.add_space(16.0);
        }
        let mut cerrar = false;
        ui::pie_dialogo(ui, |ui| {
            if Boton::new("Guardar").habilitado(self.config.is_some()).ui(ui).clicked() {
                self.guardar(cx);
            }
            cerrar = Boton::new("Cerrar").variante(Variante::Contorno).ui(ui).clicked();
        });
        cerrar
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        if let Some(puertos) = esperar(&mut self.buscando_puertos) {
            self.puertos = puertos;
        }
        if !self.abierto {
            return;
        }
        if self.config.is_none() {
            self.config = Some(balanza::config_balanza(&cx.servicios.config));
            self.buscando_puertos = Some(Tarea::lanzar(&cx.egui, balanza::puertos));
        }
        let ctx = cx.egui.clone();
        let respuesta = ui::Dialogo::new("balanza").ancho(448.0).mostrar(&ctx, |ui| self.contenido(ui, cx));
        if respuesta.cerrar || respuesta.interior {
            self.abierto = false;
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn muestra_el_peso_en_kilos_con_tres_decimales() {
        assert_eq!(peso_en_kilos(&json!(1245)), "1,245 kg");
        assert_eq!(peso_en_kilos(&json!(1234567)), "1.234,567 kg");
        assert_eq!(peso_en_kilos(&json!(0)), "0,000 kg");
        assert_eq!(peso_en_kilos(&json!("750")), "0,750 kg");
        assert_eq!(peso_en_kilos(&json!(12.5)), "0,013 kg");
        assert_eq!(peso_en_kilos(&json!("abc")), "NaN kg");
    }

    #[test]
    fn rotula_los_puertos_como_el_original() {
        assert_eq!(texto_puerto(&("COM3".into(), Some("Prolific".into()))), "COM3 · Prolific");
        assert_eq!(texto_puerto(&("COM4".into(), None)), "COM4");
        assert_eq!(texto_puerto(&("COM5".into(), Some(String::new()))), "COM5");
    }
}
