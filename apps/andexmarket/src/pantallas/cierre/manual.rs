use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, Id, Popup, PopupCloseBehavior, Rect, RectAlign, Sense, StrokeKind, Ui, Vec2,
};
use nucleo::cierre::{self as logica, Hora};
use serde_json::Value;

use super::dibujo::{self, color_degradado_texto};
use super::{avisar_impresion, encabezados_app};
use crate::app::{Contexto, Servicios};
use crate::impresion::{imprimir, Comprobante};
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, fuente, paleta, toast, Boton, Dialogo, Peso, Tam, Variante, RADIO, RADIO_MD};

type Listas = Result<(Option<Vec<Value>>, Option<Vec<Value>>), ()>;

pub enum RespuestaManual {
    Hecho(Value),
    Error(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Extremo {
    Inicio,
    Fin,
}

pub struct CierreManual {
    abierto: bool,
    fechas_listas: bool,
    inicio: Option<NaiveDateTime>,
    fin: Option<NaiveDateTime>,
    horas: [usize; 4],
    usuarios: Vec<Value>,
    sucursales: Vec<Value>,
    mes_visible: NaiveDate,
    carga: Option<Tarea<Listas>>,
    envio: Option<Tarea<RespuestaManual>>,
}

impl Default for CierreManual {
    fn default() -> Self {
        CierreManual {
            abierto: false,
            fechas_listas: false,
            inicio: None,
            fin: None,
            horas: [0, 0, 23, 59],
            usuarios: Vec::new(),
            sucursales: Vec::new(),
            mes_visible: Local::now().date_naive(),
            carga: None,
            envio: None,
        }
    }
}

fn opciones(cantidad: usize) -> Vec<String> {
    (0..cantidad).map(|i| format!("{i:02}")).collect()
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
    RespuestaManual::Hecho(imprimir(servicios, Comprobante::Cierre, respuesta.cuerpo))
}

impl CierreManual {
    pub fn abierto(&self) -> bool {
        self.abierto
    }

    fn preparar_fechas(&mut self) {
        if !self.fechas_listas {
            let ahora = Local::now().naive_local();
            self.inicio = Some(ahora);
            self.fin = Some(ahora);
            self.fechas_listas = true;
        }
    }

    pub fn abrir(&mut self, cx: &mut Contexto) {
        self.preparar_fechas();
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

    fn recibir(&mut self, cx: &mut Contexto) {
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
        match esperar(&mut self.envio) {
            Some(RespuestaManual::Hecho(impresion)) => {
                avisar_impresion(&cx.egui, &impresion);
                toast::exito(&cx.egui, "Cierre creado correctamente");
                self.abierto = false;
                let ahora = Local::now().naive_local();
                self.inicio = Some(ahora);
                self.fin = Some(ahora);
                self.horas = [0, 0, 23, 59];
            }
            Some(RespuestaManual::Error(mensaje)) => toast::error(&cx.egui, mensaje),
            None => {}
        }
    }

    fn crear(&mut self, cx: &mut Contexto) {
        let (vendedor, sucursal) = Self::ids(cx);
        let (hora_inicio, hora_fin) = (self.hora(Extremo::Inicio), self.hora(Extremo::Fin));
        match logica::validar_manual(self.inicio, self.fin, &vendedor, &sucursal, hora_inicio, hora_fin) {
            Err(mensaje) => toast::error(&cx.egui, mensaje),
            Ok((dia_inicio, dia_fin)) => {
                let (desde, hasta) = logica::rango_manual(&Local, dia_inicio, hora_inicio, dia_fin, hora_fin);
                let cuerpo = logica::cuerpo_manual(&vendedor, &sucursal, &desde, &hasta);
                let servicios = cx.servicios.clone();
                self.envio = Some(Tarea::lanzar(&cx.egui, move || crear_cierre(&servicios, &cuerpo)));
            }
        }
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        self.preparar_fechas();
        self.recibir(cx);
        if !self.abierto {
            return;
        }
        let enviando = self.envio.is_some();
        let (vendedor_id, sucursal_id) = Self::ids(cx);
        let nombre_usuario = cx.global.usuario.as_ref().and_then(|u| u["nombre"].as_str()).filter(|n| !n.is_empty());
        let vendedor = logica::nombre_por_id(&self.usuarios, &vendedor_id)
            .or(nombre_usuario.map(str::to_string))
            .unwrap_or_else(|| "Cargando...".into());
        let sucursal = logica::nombre_por_id(&self.sucursales, &sucursal_id).unwrap_or_else(|| "Cargando...".into());
        let mut crear = false;
        let mut cancelar = false;
        let respuesta = Dialogo::new("cierre-manual").ancho(448.0).mostrar(&cx.egui, |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
            ui::texto_en(ui, rect.left_center(), Align2::LEFT_CENTER, "Crear Cierre Manual", Tam::Xl, Peso::Semi, color_degradado_texto());
            ui.add_space(6.0);
            dibujo::parrafo(
                ui,
                "Selecciona las fechas y horarios para generar el cierre de caja manual. Puedes seleccionar fechas diferentes para cierres que abarquen múltiples días.",
                Tam::Sm,
                Peso::Normal,
                color::APAGADO_TEXTO,
                20.0,
            );
            ui.add_space(32.0);
            let ancho = (ui.available_width() - 16.0) / 2.0;
            fila(ui, ancho, |ui, columna| {
                let (titulo, icono, valor) = if columna == 0 { ("Vendedor", "user", &vendedor) } else { ("Sucursal", "store", &sucursal) };
                etiqueta(ui, titulo);
                caja_lectura(ui, ancho, icono, valor);
            });
            ui.add_space(16.0);
            fila(ui, ancho, |ui, columna| {
                let extremo = if columna == 0 { Extremo::Inicio } else { Extremo::Fin };
                etiqueta(ui, if columna == 0 { "Fecha de inicio" } else { "Fecha de fin" });
                self.selector_fecha(ui, ancho, extremo);
            });
            ui.add_space(16.0);
            fila(ui, ancho, |ui, columna| {
                etiqueta(ui, if columna == 0 { "Hora de inicio" } else { "Hora de fin" });
                self.selector_hora(ui, ancho, columna * 2);
            });
            if let (Some(inicio), Some(fin)) = (self.inicio, self.fin) {
                ui.add_space(16.0);
                let horas = (self.hora(Extremo::Inicio).texto(), self.hora(Extremo::Fin).texto());
                resumen(ui, inicio, fin, &horas.0, &horas.1);
            }
            ui.add_space(32.0);
            ui::pie_dialogo(ui, |ui| {
                let texto = if enviando { "Creando..." } else { "Crear Cierre" };
                crear = Boton::new(texto).variante(Variante::Esmeralda).habilitado(!enviando).ui(ui).clicked();
                cancelar = Boton::new("Cancelar").variante(Variante::Contorno).ui(ui).clicked();
            });
        });
        if crear {
            self.crear(cx);
        }
        if cancelar || respuesta.cerrar {
            self.abierto = false;
        }
    }

    fn selector_fecha(&mut self, ui: &mut Ui, ancho: f32, extremo: Extremo) {
        let valor = if extremo == Extremo::Inicio { self.inicio } else { self.fin };
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, 40.0), Sense::click());
        let fondo = if respuesta.hovered() { color::ACENTO } else { color::FONDO };
        ui.painter().rect(rect, RADIO, fondo, ui::borde(color::BORDE), StrokeKind::Inside);
        let icono = Rect::from_min_size(egui::pos2(rect.left() + 17.0, rect.center().y - 8.0), Vec2::splat(16.0));
        ui::pintar_icono_en(ui, icono, "calendar", color::TEXTO);
        let texto = match valor {
            Some(fecha) => logica::fecha_larga(fecha.date()),
            None => if extremo == Extremo::Inicio { "Elige fecha inicio" } else { "Elige fecha fin" }.to_string(),
        };
        ui.painter().with_clip_rect(ui.ctx().content_rect()).text(
            egui::pos2(icono.right() + 16.0, rect.center().y),
            Align2::LEFT_CENTER,
            texto,
            fuente(Tam::Sm, Peso::Normal),
            color::TEXTO,
        );
        let id = Id::new("cierre-manual-calendario").with(extremo == Extremo::Inicio);
        if respuesta.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            if !Popup::is_id_open(ui.ctx(), id) {
                self.mes_visible = Local::now().date_naive().with_day(1).expect("día 1");
            }
            Popup::toggle_id(ui.ctx(), id);
        }
        let mut elegido = None;
        Popup::new(id, ui.ctx().clone(), rect, ui.layer_id())
            .open_memory(None)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .align(RectAlign::BOTTOM)
            .gap(5.0)
            .frame(
                ui::marco()
                    .fill(color::FONDO)
                    .stroke(ui::borde(color::BORDE))
                    .corner_radius(RADIO_MD)
                    .shadow(ui::sombra_md())
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(|ui| {
                elegido = calendario(ui, &mut self.mes_visible, valor.map(|v| v.date()));
            });
        if let Some(dia) = elegido {
            let nuevo = if valor.map(|v| v.date()) == Some(dia) { None } else { dia.and_hms_opt(0, 0, 0) };
            if extremo == Extremo::Inicio {
                self.inicio = nuevo;
            } else {
                self.fin = nuevo;
            }
        }
    }

    fn selector_hora(&mut self, ui: &mut Ui, ancho: f32, base: usize) {
        let ancho_dos_puntos = ui.ctx().fonts_mut(|f| f.layout_no_wrap(":".into(), fuente(Tam::Base, Peso::Normal), color::TEXTO).size().x);
        let ancho_selector = (ancho - 16.0 - ancho_dos_puntos) / 2.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui::selector(ui, ("cierre-hora", base), &mut self.horas[base], &opciones(24), "", Some(ancho_selector));
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho_dos_puntos, 40.0), Sense::hover());
            ui::texto_en(ui, rect.center(), Align2::CENTER_CENTER, ":", Tam::Base, Peso::Normal, color::APAGADO_TEXTO);
            ui::selector(ui, ("cierre-hora", base + 1), &mut self.horas[base + 1], &opciones(60), "", Some(ancho_selector));
        });
    }
}

fn fila(ui: &mut Ui, ancho: f32, mut contenido: impl FnMut(&mut Ui, usize)) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        for columna in 0..2 {
            ui.vertical(|ui| {
                ui.set_width(ancho);
                contenido(ui, columna);
            });
        }
    });
}

fn etiqueta(ui: &mut Ui, texto: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0), Sense::hover());
    ui::texto_en(ui, rect.left_center(), Align2::LEFT_CENTER, texto, Tam::Sm, Peso::Medio, color::TEXTO);
    ui.add_space(8.0);
}

fn caja_lectura(ui: &mut Ui, ancho: f32, icono: &str, valor: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 46.0), Sense::hover());
    ui.painter().rect(rect, RADIO_MD, paleta::GRAY_50, ui::borde(paleta::GRAY_200), StrokeKind::Inside);
    let caja = Rect::from_min_size(egui::pos2(rect.left() + 13.0, rect.center().y - 8.0), Vec2::splat(16.0));
    ui::pintar_icono_en(ui, caja, icono, paleta::GRAY_500);
    ui::texto_en(ui, egui::pos2(caja.right() + 8.0, rect.center().y), Align2::LEFT_CENTER, valor, Tam::Sm, Peso::Normal, paleta::GRAY_700);
}

fn linea_mixta(ui: &mut Ui, partes: &[(&str, Peso)], tam: Tam, color: Color32, alto: f32) {
    let mut trabajo = egui::text::LayoutJob::default();
    for (texto, peso) in partes {
        trabajo.append(texto, 0.0, egui::TextFormat { font_id: fuente(tam, *peso), color, ..Default::default() });
    }
    let galeria = ui.ctx().fonts_mut(|f| f.layout_job(trabajo));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), alto), Sense::hover());
    ui.painter().galley(egui::pos2(rect.left(), rect.center().y - galeria.size().y / 2.0), galeria, color);
}

fn resumen(ui: &mut Ui, inicio: NaiveDateTime, fin: NaiveDateTime, hora_inicio: &str, hora_fin: &str) {
    ui::marco()
        .fill(paleta::BLUE_50)
        .stroke(ui::borde(paleta::BLUE_200))
        .corner_radius(RADIO)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 20.0), Sense::hover());
                ui::pintar_icono_en(ui, Rect::from_center_size(rect.center(), Vec2::splat(16.0)), "clock", paleta::BLUE_600);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
                ui::texto_en(ui, rect.left_center(), Align2::LEFT_CENTER, "Período seleccionado:", Tam::Sm, Peso::Medio, paleta::BLUE_800);
            });
            ui.add_space(8.0);
            let texto_inicio = format!(" {} a las {hora_inicio}", logica::fecha_larga(inicio.date()));
            linea_mixta(ui, &[("Inicio:", Peso::Negrita), (&texto_inicio, Peso::Normal)], Tam::Sm, paleta::BLUE_700, 20.0);
            ui.add_space(4.0);
            let texto_fin = format!(" {} a las {hora_fin}", logica::fecha_larga(fin.date()));
            linea_mixta(ui, &[("Fin:", Peso::Negrita), (&texto_fin, Peso::Normal)], Tam::Sm, paleta::BLUE_700, 20.0);
            if inicio != fin {
                ui.add_space(4.0);
                linea_mixta(ui, &[("⚠️ Cierre de múltiples días", Peso::Normal)], Tam::Xs, paleta::BLUE_600, 16.0);
            }
        });
}

fn nombre_mes(fecha: NaiveDate) -> String {
    format!("{} {}", logica::MESES[fecha.month0() as usize], fecha.year())
}

fn boton_navegacion(ui: &Ui, rect: Rect, icono: &str, id: Id) -> bool {
    let respuesta = ui.interact(rect, id, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    let opacidad = if respuesta.hovered() { 1.0 } else { 0.5 };
    ui.painter().rect_stroke(rect, RADIO, ui::borde(color::BORDE.gamma_multiply(opacidad)), StrokeKind::Inside);
    ui::pintar_icono_en(ui, Rect::from_center_size(rect.center(), Vec2::splat(16.0)), icono, color::TEXTO.gamma_multiply(opacidad));
    respuesta.clicked()
}

fn calendario(ui: &mut Ui, mes: &mut NaiveDate, elegida: Option<NaiveDate>) -> Option<NaiveDate> {
    let ancho = 36.0 * 7.0;
    ui.set_width(ancho);
    let hoy = Local::now().date_naive();
    let (cabecera, _) = ui.allocate_exact_size(Vec2::new(ancho, 28.0), Sense::hover());
    ui::texto_en(ui, cabecera.center() + Vec2::new(0.0, 2.0), Align2::CENTER_CENTER, &nombre_mes(*mes), Tam::Sm, Peso::Medio, color::TEXTO);
    let anterior = Rect::from_min_size(egui::pos2(cabecera.left() + 4.0, cabecera.top()), Vec2::splat(28.0));
    let siguiente = Rect::from_min_size(egui::pos2(cabecera.right() - 32.0, cabecera.top()), Vec2::splat(28.0));
    if boton_navegacion(ui, anterior, "chevron-left", ui.id().with("mes-anterior")) {
        *mes = (*mes - Duration::days(1)).with_day(1).expect("día 1");
    }
    if boton_navegacion(ui, siguiente, "chevron-right", ui.id().with("mes-siguiente")) {
        *mes = (*mes + Duration::days(32)).with_day(1).expect("día 1");
    }
    ui.add_space(16.0);
    let (dias, _) = ui.allocate_exact_size(Vec2::new(ancho, 19.0), Sense::hover());
    for (indice, nombre) in ["lu", "ma", "mi", "ju", "vi", "sá", "do"].iter().enumerate() {
        let centro = egui::pos2(dias.left() + 18.0 + 36.0 * indice as f32, dias.center().y);
        ui::texto_en(ui, centro, Align2::CENTER_CENTER, nombre, Tam::Px(12.8), Peso::Normal, color::APAGADO_TEXTO);
    }
    let primero = mes.with_day(1).expect("día 1");
    let mut dia = primero - Duration::days(primero.weekday().num_days_from_monday() as i64);
    let mut elegido = None;
    while dia.month() == primero.month() || dia < primero {
        ui.add_space(8.0);
        let (fila, _) = ui.allocate_exact_size(Vec2::new(ancho, 36.0), Sense::hover());
        for columna in 0..7 {
            let caja = Rect::from_min_size(egui::pos2(fila.left() + 36.0 * columna as f32, fila.top()), Vec2::splat(36.0));
            let afuera = dia.month() != primero.month();
            let deshabilitado = dia > hoy;
            let seleccionado = elegida == Some(dia);
            let sentido = if deshabilitado { Sense::hover() } else { Sense::click() };
            let respuesta = ui.interact(caja, ui.id().with(("dia", dia)), sentido);
            let (fondo, texto) = if seleccionado && !afuera {
                (color::PRIMARIO, color::PRIMARIO_TEXTO)
            } else if seleccionado {
                (color::ACENTO.gamma_multiply(0.5), color::APAGADO_TEXTO)
            } else if dia == hoy {
                (color::ACENTO, color::ACENTO_TEXTO)
            } else if respuesta.hovered() && !deshabilitado {
                (color::ACENTO, if afuera { color::APAGADO_TEXTO } else { color::ACENTO_TEXTO })
            } else {
                (Color32::TRANSPARENT, if afuera { color::APAGADO_TEXTO } else { color::TEXTO })
            };
            let opacidad = if deshabilitado { 0.5 } else { 1.0 };
            ui.painter().rect_filled(caja, RADIO, fondo.gamma_multiply(opacidad));
            let texto = if deshabilitado { color::APAGADO_TEXTO } else { texto };
            ui::texto_en(ui, caja.center(), Align2::CENTER_CENTER, &dia.day().to_string(), Tam::Sm, Peso::Normal, texto.gamma_multiply(opacidad));
            if !deshabilitado && respuesta.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                elegido = Some(dia);
            }
            dia += Duration::days(1);
        }
    }
    elegido
}
