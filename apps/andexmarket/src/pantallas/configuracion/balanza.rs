use eframe::egui::{self, Frame, Margin, Sense, Ui, Vec2};
use hardware::balanza;
use serde_json::{json, Value};

use super::texto_valor;
use crate::pantallas::cierre::dibujo::etiqueta;
use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, fuente, fuente_mono, radio, tam};
use crate::ui::v2::{self, desplegable, opcion_eleccion, punto_estado, toast, Boton};
use crate::ui::Peso;

type Puertos = Vec<(String, Option<String>)>;

const ORIGENES: [(&str, &str, &str, bool); 2] = [("serie", "Puerto serie", "Croma / RS-232", false), ("archivo", "Archivo", "C:\\Peso\\peso.json", true)];
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

fn elegir(ui: &mut Ui, id: &str, actual: &Value, opciones: &[(String, String)], sugerencia: &str, alto: f32) -> Option<String> {
    let actual = texto_valor(actual);
    let mut seleccion = opciones.iter().position(|(valor, _)| *valor == actual).unwrap_or(usize::MAX);
    let etiquetas: Vec<String> = opciones.iter().map(|(_, etiqueta)| etiqueta.clone()).collect();
    let ancho = ui.available_width();
    desplegable(ui, id, &mut seleccion, &etiquetas, sugerencia, ancho, alto).changed().then(|| opciones[seleccion].0.clone())
}

fn pares(opciones: &[(&str, &str)]) -> Vec<(String, String)> {
    opciones.iter().map(|(valor, etiqueta)| (valor.to_string(), etiqueta.to_string())).collect()
}

fn origenes(ui: &mut Ui, config: &mut Value) {
    etiqueta(ui, "Origen del peso", tam::ETIQUETA, Peso::Semi);
    ui.add_space(6.0);
    let ancho = (ui.available_width() - 8.0) / 2.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for (modo, titulo, detalle, mono) in ORIGENES {
            let activo = config["modo"] == modo;
            let respuesta = opcion_eleccion(ui, Vec2::new(ancho, 60.0), radio::CAMPO, activo, 1.5, |ui, rect, _| {
                let fuente_detalle = if mono { fuente_mono(tam::XS, Peso::Normal) } else { fuente(tam::XS, Peso::Normal) };
                let g_titulo = v2::galeria(ui, titulo, fuente(tam::CUERPO, Peso::Semi), color::TEXTO);
                let g_detalle = v2::galeria(ui, detalle, fuente_detalle, color::TEXTO_APAGADO);
                let y = rect.center().y - (g_titulo.size().y + 2.0 + g_detalle.size().y) / 2.0;
                let alto_titulo = g_titulo.size().y;
                ui.painter().galley(egui::pos2(rect.left() + 15.0, y), g_titulo, color::TEXTO);
                ui.painter().galley(egui::pos2(rect.left() + 15.0, y + alto_titulo + 2.0), g_detalle, color::TEXTO_APAGADO);
            });
            if respuesta.clicked() {
                config["modo"] = json!(modo);
            }
        }
    });
}

fn cuadricula(ui: &mut Ui, config: &mut Value) {
    let ancho = (ui.available_width() - 3.0 * 8.0) / 4.0;
    let mut cambios: Vec<(&str, Value)> = Vec::new();
    let velocidades: Vec<(String, String)> = VELOCIDADES.iter().map(|v| (v.to_string(), v.to_string())).collect();
    let formatos: Vec<(String, String)> = FORMATOS.iter().map(|f| (f.to_string(), f.to_string())).collect();
    let selectores: [(&str, &str, &str, Vec<(String, String)>); 4] = [
        ("Velocidad", "velocidad", "baudRate", velocidades),
        ("Formato", "formato", "formato", formatos),
        ("Lectura", "lectura", "consulta", pares(&LECTURAS)),
        ("Sin decimales", "unidad", "unidadEntera", pares(&UNIDADES)),
    ];
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for (titulo, id, clave, opciones) in &selectores {
            ui.allocate_ui_with_layout(Vec2::new(ancho, 62.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(ancho);
                etiqueta(ui, titulo, tam::CHICO, Peso::Medio);
                ui.add_space(6.0);
                if let Some(valor) = elegir(ui, id, &config[*clave], opciones, "", 40.0) {
                    let valor = if *clave == "baudRate" { json!(valor.parse::<u32>().unwrap_or(9600)) } else { json!(valor) };
                    cambios.push((clave, valor));
                }
            });
        }
    });
    for (clave, valor) in cambios {
        config[clave] = valor;
    }
}

fn estado_conexion(ui: &mut Ui, estado: &Value, puerto: &str) {
    if estado["conectada"] == true {
        punto_estado(ui, &format!("Conectada en {puerto}"), color::VERDE_PUNTO, color::VERDE_EXITO, tam::CONTROL, Peso::Medio, false);
        return;
    }
    let texto = match estado["error"].as_str().filter(|e| !e.is_empty()) {
        Some(error) => format!("Desconectada ({error})"),
        None => "Desconectada".to_string(),
    };
    punto_estado(ui, &texto, color::ROJO_INTENSO, color::ROJO, tam::CONTROL, Peso::Medio, false);
}

fn peso_actual(ui: &mut Ui, peso: &Value) {
    Frame::NONE.fill(color::VERDE_PROFUNDO).corner_radius(radio::TARJETA).inner_margin(Margin::symmetric(20, 18)).show(ui, |ui| {
        let ancho = ui.available_width();
        ui.set_width(ancho);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                v2::texto(ui, "Peso actual", tam::CHICO, Peso::Normal, color::SOBRE_OSCURO_APAGADO);
                ui.add_space(4.0);
                v2::titulo(ui, peso_en_kilos(peso), 36.0, -0.02, color::BLANCO);
            });
            let (resto, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 63.0), Sense::hover());
            v2::pintar_icono(ui, egui::pos2(resto.right() - 16.0, resto.center().y), "scale", 32.0, color::DORADO);
        });
    });
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
        let mut cerrar = false;
        v2::cuerpo_modal(ui, 20, |ui| {
            let Some(config) = self.config.as_mut() else {
                return;
            };
            origenes(ui, config);
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
                etiqueta(ui, "Puerto", tam::ETIQUETA, Peso::Semi);
                ui.add_space(6.0);
                if let Some(puerto) = elegir(ui, "puerto", &config["puerto"], &puertos, sugerencia, 42.0) {
                    config["puerto"] = json!(puerto);
                }
                ui.add_space(16.0);
                cuadricula(ui, config);
                ui.add_space(16.0);
                estado_conexion(ui, &cx.global.estado_balanza, &texto_valor(&config["puerto"]));
            }
            ui.add_space(16.0);
            peso_actual(ui, &cx.global.peso);
        });
        v2::pie_modal(ui, |ui| {
            if Boton::primario("Guardar").modal().habilitado(self.config.is_some()).ui(ui).clicked() {
                self.guardar(cx);
            }
            cerrar = Boton::secundario("Cerrar").modal().ui(ui).clicked();
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
        let respuesta = v2::Modal::new("balanza", "Balanza")
            .subtitulo("Origen del peso para los productos por kilo")
            .icono("scale")
            .ancho(560.0)
            .mostrar(&ctx, |ui| self.contenido(ui, cx));
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
