use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use eframe::egui::{self, Align, Color32, CursorIcon, Id, Rect, Response, ScrollArea, Sense, Ui, Vec2};
use nucleo::cierre::{self as logica, Hora};
use serde_json::Value;

use super::dibujo::{caja_entrada, etiqueta, nota};
use super::{avisar_impresion, encabezados_app};
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{color, radio, tam};
use crate::ui::v2::{self, toast, Boton, BotonIcono, Popover};
use crate::ui::Peso;

type Listas = Result<(Option<Vec<Value>>, Option<Vec<Value>>), ()>;

const DIAS_SEMANA: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];

pub enum RespuestaManual {
    Hecho { cierre: Value, impresion: Value },
    Error(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Extremo {
    Inicio,
    Fin,
}

pub struct CierreManual {
    abierto: bool,
    inicio: NaiveDate,
    fin: NaiveDate,
    horas: [usize; 4],
    usuarios: Vec<Value>,
    sucursales: Vec<Value>,
    mes_visible: NaiveDate,
    carga: Option<Tarea<Listas>>,
    envio: Option<Tarea<RespuestaManual>>,
}

impl Default for CierreManual {
    fn default() -> Self {
        let mut manual = CierreManual {
            abierto: false,
            inicio: NaiveDate::MIN,
            fin: NaiveDate::MIN,
            horas: [0; 4],
            usuarios: Vec::new(),
            sucursales: Vec::new(),
            mes_visible: NaiveDate::MIN,
            carga: None,
            envio: None,
        };
        manual.valores_por_defecto(logica::ahora_argentina());
        manual
    }
}

fn lista_valida(valor: &Value) -> Option<Vec<Value>> {
    valor.as_array().cloned()
}

pub fn cargar_listas(servicios: &Servicios) -> Listas {
    let encabezados = encabezados_app();
    let usuarios = servicios.api.pedir("GET", "/api/usuarios", None, &encabezados).map_err(|_| ())?;
    let sucursales = servicios.api.pedir("GET", "/api/sucursales", None, &encabezados).map_err(|_| ())?;
    let usuarios = if usuarios.ok() {
        if !usuarios.cuerpo.is_array() {
            return Err(());
        }
        Some(logica::vendedores(&usuarios.cuerpo))
    } else {
        None
    };
    let sucursales = if sucursales.ok() { Some(lista_valida(&sucursales.cuerpo).unwrap_or_default()) } else { None };
    Ok((usuarios, sucursales))
}

pub fn crear_cierre(servicios: &Servicios, cuerpo: &Value) -> RespuestaManual {
    let Ok(respuesta) = servicios.api.pedir("POST", "/api/cierres", Some(cuerpo), &encabezados_app()) else {
        return RespuestaManual::Error("Failed to fetch".into());
    };
    if !respuesta.ok() {
        let mensaje = Some(&respuesta.cuerpo["message"]).filter(|m| logica::verdadero(Some(m))).map(logica::texto_valor);
        return RespuestaManual::Error(mensaje.unwrap_or_else(|| "Error al crear el cierre".into()));
    }
    let impresion = imprimir(servicios, Comprobante::Cierre, respuesta.cuerpo.clone());
    RespuestaManual::Hecho { cierre: respuesta.cuerpo, impresion }
}

pub fn fecha_con_dia(fecha: NaiveDate) -> String {
    format!(
        "{}, {} de {}",
        DIAS_SEMANA[fecha.weekday().num_days_from_monday() as usize],
        fecha.day(),
        logica::MESES[fecha.month0() as usize]
    )
}

pub fn resumen_periodo(inicio: NaiveDate, fin: NaiveDate, hora_inicio: &str, hora_fin: &str) -> String {
    let varios = if inicio != fin { " Es un cierre de varios días." } else { "" };
    format!("Desde el {} a las {hora_inicio} hasta el {} a las {hora_fin}.{varios}", fecha_con_dia(inicio), fecha_con_dia(fin))
}

impl CierreManual {
    pub fn abierto(&self) -> bool {
        self.abierto
    }

    fn valores_por_defecto(&mut self, ahora: NaiveDateTime) {
        self.inicio = ahora.date();
        self.fin = ahora.date();
        self.horas = [0, 0, ahora.hour() as usize, ahora.minute() as usize];
        self.mes_visible = ahora.date().with_day(1).expect("día 1");
    }

    pub fn abrir(&mut self, cx: &mut Contexto) {
        self.valores_por_defecto(logica::ahora_argentina());
        self.abierto = true;
        let servicios = cx.servicios.clone();
        self.carga = Some(Tarea::lanzar(&cx.egui, move || cargar_listas(&servicios)));
    }

    fn hora(&self, extremo: Extremo) -> Hora {
        let base = if extremo == Extremo::Inicio { 0 } else { 2 };
        Hora { hora: self.horas[base] as u32, minuto: self.horas[base + 1] as u32 }
    }

    fn ids(cx: &Contexto) -> (String, String) {
        let usuario = cx.global.usuario.as_ref();
        (
            logica::id_texto(usuario.and_then(|u| u.get("id"))),
            logica::id_texto(usuario.and_then(|u| u.get("sucursalId"))),
        )
    }

    fn recibir(&mut self, cx: &mut Contexto) -> Option<Value> {
        match esperar(&mut self.carga) {
            Some(Ok((usuarios, sucursales))) => {
                if let Some(usuarios) = usuarios {
                    self.usuarios = usuarios;
                }
                if let Some(sucursales) = sucursales {
                    self.sucursales = sucursales;
                }
            }
            Some(Err(())) => toast::error(&cx.egui, "Error al cargar datos para el formulario."),
            None => {}
        }
        match esperar(&mut self.envio)? {
            RespuestaManual::Hecho { cierre, impresion } => {
                avisar_impresion(&cx.egui, &impresion);
                toast::exito(&cx.egui, "Cierre creado correctamente");
                self.abierto = false;
                Some(cierre)
            }
            RespuestaManual::Error(mensaje) => {
                toast::error(&cx.egui, mensaje);
                None
            }
        }
    }

    fn crear(&mut self, cx: &mut Contexto) {
        let (vendedor, sucursal) = Self::ids(cx);
        let (hora_inicio, hora_fin) = (self.hora(Extremo::Inicio), self.hora(Extremo::Fin));
        match logica::validar_manual(self.inicio, self.fin, &vendedor, &sucursal, hora_inicio, hora_fin) {
            Err(mensaje) => toast::error(&cx.egui, mensaje),
            Ok(()) => {
                let (desde, hasta) = logica::rango_manual(self.inicio, hora_inicio, self.fin, hora_fin);
                let cuerpo = logica::cuerpo_manual(&vendedor, &sucursal, &desde, &hasta);
                let servicios = cx.servicios.clone();
                self.envio = Some(Tarea::lanzar(&cx.egui, move || crear_cierre(&servicios, &cuerpo)));
            }
        }
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) -> Option<Value> {
        let completado = self.recibir(cx);
        if !self.abierto {
            return completado;
        }
        let enviando = self.envio.is_some();
        let (vendedor_id, sucursal_id) = Self::ids(cx);
        let nombre_usuario = cx.global.usuario.as_ref().and_then(|u| u["nombre"].as_str()).filter(|n| !n.is_empty());
        let vendedor = logica::nombre_por_id(&self.usuarios, &vendedor_id)
            .or(nombre_usuario.map(str::to_string))
            .unwrap_or_else(|| "Cargando...".into());
        let sucursal = logica::nombre_por_id(&self.sucursales, &sucursal_id).unwrap_or_else(|| "Cargando...".into());
        let (hora_inicio, hora_fin) = (self.hora(Extremo::Inicio), self.hora(Extremo::Fin));
        let error = logica::validar_manual(self.inicio, self.fin, &vendedor_id, &sucursal_id, hora_inicio, hora_fin).err();
        let mut crear = false;
        let mut volver = false;
        let respuesta = v2::Modal::new("cierre-manual", "Cierre manual")
            .subtitulo("Elegí desde y hasta cuándo. Puede abarcar varios días.")
            .icono("clock-time")
            .ancho(560.0)
            .mostrar(&cx.egui, |ui| {
                v2::cuerpo_modal(ui, 20, |ui| {
                    let ancho = (ui.available_width() - 12.0) / 2.0;
                    fila(ui, ancho, |ui, columna| {
                        let (titulo, icono, valor) = if columna == 0 { ("Vendedor", "user", &vendedor) } else { ("Sucursal", "store", &sucursal) };
                        etiqueta(ui, titulo, tam::ETIQUETA, Peso::Medio);
                        ui.add_space(6.0);
                        caja_lectura(ui, ancho, icono, valor);
                    });
                    ui.add_space(14.0);
                    fila(ui, ancho, |ui, columna| {
                        let extremo = if columna == 0 { Extremo::Inicio } else { Extremo::Fin };
                        etiqueta(ui, if columna == 0 { "Desde" } else { "Hasta" }, tam::ETIQUETA, Peso::Medio);
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            self.selector_fecha(ui, ancho - 6.0 - 104.0, extremo);
                            self.selector_hora(ui, 104.0, extremo);
                        });
                    });
                    ui.add_space(14.0);
                    match error {
                        Some(mensaje) => nota(ui, "alert", 18.0, mensaje, tam::CONTROL, color::ROJO_FONDO, color::ROJO, color::ROJO),
                        None => {
                            let resumen = resumen_periodo(self.inicio, self.fin, &hora_inicio.texto(), &hora_fin.texto());
                            let (fondo, tinta) = if self.inicio != self.fin {
                                (color::DORADO_FONDO, color::DORADO_TEXTO)
                            } else {
                                (color::VERDE_ELEGIDO, color::VERDE_PROFUNDO)
                            };
                            nota(ui, "clock-time", 18.0, &resumen, tam::CONTROL, fondo, tinta, tinta);
                        }
                    }
                });
                v2::pie_modal(ui, |ui| {
                    ui.scope(|ui| {
                        if error.is_some() {
                            ui.set_opacity(0.45);
                        }
                        crear = Boton::primario("Crear cierre").modal().cargando(enviando).ui(ui).clicked();
                    });
                    volver = Boton::secundario("Volver").modal().ui(ui).clicked();
                });
            });
        if crear {
            self.crear(cx);
        }
        if volver || respuesta.cerrar {
            self.abierto = false;
        }
        None
    }

    fn selector_fecha(&mut self, ui: &mut Ui, ancho: f32, extremo: Extremo) {
        let valor = if extremo == Extremo::Inicio { self.inicio } else { self.fin };
        let id = Id::new("cierre-manual-fecha").with(extremo);
        let (rect, respuesta) = caja_entrada(ui, ancho, 42.0, Popover::abierto(ui.ctx(), id));
        v2::pintar_texto(ui, egui::pos2(rect.left() + 11.0, rect.center().y), egui::Align2::LEFT_CENTER, valor.format("%d/%m/%Y").to_string(), tam::CUERPO, Peso::Normal, color::TEXTO);
        v2::pintar_icono(ui, egui::pos2(rect.right() - 20.0, rect.center().y), "calendar", 16.0, color::TEXTO_SECUNDARIO);
        if respuesta.clicked() {
            if !Popover::abierto(ui.ctx(), id) {
                self.mes_visible = valor.with_day(1).expect("día 1");
            }
            Popover::alternar(ui.ctx(), id);
        }
        let elegido = Popover::new(id, &respuesta).ancho(7.0 * 36.0 + 24.0).radio(radio::CAMPO).mostrar(ui.ctx(), |ui| calendario(ui, &mut self.mes_visible, valor)).flatten();
        if let Some(dia) = elegido {
            if extremo == Extremo::Inicio {
                self.inicio = dia;
            } else {
                self.fin = dia;
            }
            Popover::cerrar(ui.ctx(), id);
        }
    }

    fn selector_hora(&mut self, ui: &mut Ui, ancho: f32, extremo: Extremo) {
        let base = if extremo == Extremo::Inicio { 0 } else { 2 };
        let id = Id::new("cierre-manual-hora").with(extremo);
        let (rect, respuesta) = caja_entrada(ui, ancho, 42.0, Popover::abierto(ui.ctx(), id));
        v2::pintar_texto(ui, egui::pos2(rect.left() + 9.0, rect.center().y), egui::Align2::LEFT_CENTER, self.hora(extremo).texto(), tam::CUERPO, Peso::Normal, color::TEXTO);
        v2::pintar_icono(ui, egui::pos2(rect.right() - 20.0, rect.center().y), "clock-time", 18.0, color::TEXTO_SECUNDARIO);
        let centrar = respuesta.clicked() && !Popover::abierto(ui.ctx(), id);
        if respuesta.clicked() {
            Popover::alternar(ui.ctx(), id);
        }
        let horas = &mut self.horas;
        Popover::new(id, &respuesta).ancho(2.0 * 52.0 + 4.0 + 12.0).mostrar(ui.ctx(), |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                columna_hora(ui, id.with("horas"), &mut horas[base], 24, centrar);
                columna_hora(ui, id.with("minutos"), &mut horas[base + 1], 60, centrar);
            });
        });
    }
}

fn columna_hora(ui: &mut Ui, id: Id, valor: &mut usize, cantidad: usize, centrar: bool) {
    ScrollArea::vertical().id_salt(id).max_height(216.0).show(ui, |ui| {
        ui.set_width(52.0);
        for numero in 0..cantidad {
            let elegido = *valor == numero;
            let (caja, respuesta) = ui.allocate_exact_size(Vec2::new(52.0, 30.0), Sense::click());
            let fondo = if elegido {
                color::VERDE
            } else if respuesta.hovered() {
                color::FONDO_APP
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect_filled(caja, radio::ICONO, fondo);
            let (tinta, peso) = if elegido { (color::BLANCO, Peso::Semi) } else { (color::TEXTO, Peso::Normal) };
            v2::pintar_texto(ui, caja.center(), egui::Align2::CENTER_CENTER, format!("{numero:02}"), tam::CUERPO, peso, tinta);
            if centrar && elegido {
                respuesta.scroll_to_me(Some(Align::Center));
            }
            if respuesta.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                *valor = numero;
            }
        }
    });
}

fn fila(ui: &mut Ui, ancho: f32, mut contenido: impl FnMut(&mut Ui, usize)) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        for columna in 0..2 {
            ui.vertical(|ui| {
                ui.set_width(ancho);
                contenido(ui, columna);
            });
        }
    });
}

fn caja_lectura(ui: &mut Ui, ancho: f32, icono: &str, valor: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 42.0), Sense::hover());
    ui.painter().rect(rect, radio::CONTROL, color::FONDO_APP, egui::Stroke::new(1.0, color::BORDE_SUAVE), egui::StrokeKind::Inside);
    v2::pintar_icono(ui, egui::pos2(rect.left() + 12.0 + 8.5, rect.center().y), icono, 17.0, color::TEXTO_TENUE);
    v2::pintar_texto(ui, egui::pos2(rect.left() + 12.0 + 17.0 + 8.0, rect.center().y), egui::Align2::LEFT_CENTER, valor, tam::CUERPO, Peso::Normal, color::TEXTO_SECUNDARIO);
}

fn nombre_mes(fecha: NaiveDate) -> String {
    let mes = logica::MESES[fecha.month0() as usize];
    let mut letras = mes.chars();
    let capital: String = letras.next().map(|c| c.to_uppercase().chain(letras).collect()).unwrap_or_default();
    format!("{capital} {}", fecha.year())
}

fn calendario(ui: &mut Ui, mes: &mut NaiveDate, elegida: NaiveDate) -> Option<NaiveDate> {
    let ancho = 36.0 * 7.0;
    let mut elegido = None;
    egui::Frame::NONE.inner_margin(egui::Margin::same(6)).show(ui, |ui| {
        ui.set_width(ancho);
        let hoy = logica::ahora_argentina().date();
        ui.horizontal(|ui| {
            if BotonIcono::new("chevron-left").tam(32.0).tam_icono(16.0).ui(ui).clicked() {
                *mes = (*mes - Duration::days(1)).with_day(1).expect("día 1");
            }
            let (centro, _) = ui.allocate_exact_size(Vec2::new(ancho - 64.0, 32.0), Sense::hover());
            v2::pintar_texto(ui, centro.center(), egui::Align2::CENTER_CENTER, nombre_mes(*mes), tam::CUERPO, Peso::Semi, color::TEXTO);
            if BotonIcono::new("chevron-right").tam(32.0).tam_icono(16.0).ui(ui).clicked() {
                *mes = (*mes + Duration::days(32)).with_day(1).expect("día 1");
            }
        });
        ui.add_space(8.0);
        let (dias, _) = ui.allocate_exact_size(Vec2::new(ancho, 20.0), Sense::hover());
        for (indice, nombre) in ["lu", "ma", "mi", "ju", "vi", "sá", "do"].iter().enumerate() {
            let centro = egui::pos2(dias.left() + 18.0 + 36.0 * indice as f32, dias.center().y);
            v2::pintar_texto(ui, centro, egui::Align2::CENTER_CENTER, *nombre, tam::XS, Peso::Medio, color::TEXTO_APAGADO);
        }
        let primero = mes.with_day(1).expect("día 1");
        let mut dia = primero - Duration::days(primero.weekday().num_days_from_monday() as i64);
        while dia.month() == primero.month() || dia < primero {
            ui.add_space(2.0);
            let (fila, _) = ui.allocate_exact_size(Vec2::new(ancho, 36.0), Sense::hover());
            for columna in 0..7 {
                let caja = Rect::from_min_size(egui::pos2(fila.left() + 36.0 * columna as f32, fila.top()), Vec2::splat(36.0)).shrink(1.0);
                if let Some(nuevo) = celda_dia(ui, caja, dia, primero, elegida, hoy) {
                    elegido = Some(nuevo);
                }
                dia += Duration::days(1);
            }
        }
    });
    elegido
}

fn celda_dia(ui: &Ui, caja: Rect, dia: NaiveDate, primero: NaiveDate, elegida: NaiveDate, hoy: NaiveDate) -> Option<NaiveDate> {
    let afuera = dia.month() != primero.month();
    let deshabilitado = dia > hoy;
    let respuesta: Response = ui.interact(caja, ui.id().with(("dia", dia)), if deshabilitado { Sense::hover() } else { Sense::click() });
    let (fondo, tinta, peso) = if elegida == dia {
        (color::VERDE, color::BLANCO, Peso::Semi)
    } else if dia == hoy {
        (color::VERDE_CHIP, color::VERDE, Peso::Semi)
    } else if respuesta.hovered() && !deshabilitado {
        (color::FONDO_APP, color::TEXTO, Peso::Normal)
    } else {
        (Color32::TRANSPARENT, if afuera { color::TEXTO_DESHABILITADO } else { color::TEXTO }, Peso::Normal)
    };
    let alfa = if deshabilitado { 0.45 } else { 1.0 };
    ui.painter().rect_filled(caja, radio::ICONO, fondo.gamma_multiply(alfa));
    v2::pintar_texto(ui, caja.center(), egui::Align2::CENTER_CENTER, dia.day().to_string(), tam::CONTROL, peso, tinta.gamma_multiply(alfa));
    (!deshabilitado && respuesta.on_hover_cursor(CursorIcon::PointingHand).clicked()).then_some(dia)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn resume_el_periodo_como_el_diseno() {
        let dia = |d| NaiveDate::from_ymd_opt(2026, 10, d).unwrap();
        assert_eq!(fecha_con_dia(dia(10)), "sábado, 10 de octubre");
        assert_eq!(
            resumen_periodo(dia(10), dia(10), "00:00", "23:59"),
            "Desde el sábado, 10 de octubre a las 00:00 hasta el sábado, 10 de octubre a las 23:59."
        );
        assert_eq!(
            resumen_periodo(dia(9), dia(10), "08:00", "23:59"),
            "Desde el viernes, 9 de octubre a las 08:00 hasta el sábado, 10 de octubre a las 23:59. Es un cierre de varios días."
        );
        assert_eq!(nombre_mes(dia(1)), "Octubre 2026");
    }
}
