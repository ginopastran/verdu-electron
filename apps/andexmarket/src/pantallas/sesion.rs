use std::sync::Arc;

use eframe::egui::text::LayoutJob;
use eframe::egui::{self, Align, Align2, Color32, Context, Galley, Id, Rect, Ui, UiBuilder};
use hardware::config::Config;
use nucleo::sesion as logica;
use serde_json::{json, Value};

use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::{fuente, Peso, Tam};

const CONTRASENA_ADMIN_PLANA: &str = "adminPassword";
const CONTRASENA_ADMIN: &str = "adminPasswordCifrada";

pub const DEGRADADO_TEXTO: Color32 = Color32::from_rgb(0x15, 0x47, 0x38);

#[derive(Default)]
pub struct Sesion {
    pub iniciada: bool,
    pub negocio_de: Option<i64>,
    pub carga_negocio: Option<Tarea<(i64, Result<Value, String>)>>,
}

impl Sesion {
    pub fn actualizar(&mut self, cx: &mut Contexto) {
        if !self.iniciada {
            self.iniciada = true;
            if cx.global.admin.is_none() && cx.servicios.config.has("adminData") {
                cx.servicios.config.delete("adminData");
            }
            migrar_contrasena_admin(&cx.servicios.config);
        }
        self.proteger_rutas(cx);
        self.seguir_negocio(cx);
    }

    fn proteger_rutas(&self, cx: &mut Contexto) {
        if !matches!(cx.global.ruta, Ruta::Carrito | Ruta::Facturas | Ruta::FacturaDetalle(_)) {
            return;
        }
        match &cx.global.usuario {
            None => cx.global.ir(Ruta::UsuarioLogin),
            Some(usuario) if logica::ruta_protegida_rechaza_admin(usuario) => cx.global.ir(Ruta::MensajeAdmin),
            Some(_) => {}
        }
    }

    fn seguir_negocio(&mut self, cx: &mut Contexto) {
        let id = cx.global.business_id();
        if id != self.negocio_de {
            self.negocio_de = id;
            self.carga_negocio = None;
            match id {
                None => {
                    cx.global.negocio = None;
                    cx.global.negocio_cargando = false;
                }
                Some(id) if cx.global.negocio.as_ref().is_some_and(|n| n["id"].as_i64() == Some(id)) => {
                    cx.global.negocio_cargando = false;
                }
                Some(id) => {
                    cx.global.negocio = None;
                    cx.global.negocio_cargando = true;
                    let guardado = cx.servicios.config.get(&logica::clave_negocio(id));
                    if !guardado.is_null() {
                        cx.global.negocio = Some(guardado);
                        cx.global.negocio_cargando = false;
                    }
                    self.carga_negocio = Some(cargar_negocio(cx, id));
                }
            }
        }
        if refresco_pedido(&cx.egui) {
            if let Some(id) = self.negocio_de {
                cx.global.negocio_cargando = true;
                self.carga_negocio = Some(cargar_negocio(cx, id));
            }
        }
        if let Some((id, resultado)) = tareas::esperar(&mut self.carga_negocio) {
            if Some(id) != self.negocio_de {
                return;
            }
            match resultado {
                Ok(negocio) => cx.global.negocio = Some(negocio),
                Err(_) => {
                    let guardado = cx.servicios.config.get(&logica::clave_negocio(id));
                    if !guardado.is_null() {
                        cx.global.negocio = Some(guardado);
                    } else if cx.global.negocio.is_none() {
                        cx.global.negocio = Some(json!({ "sistemaPago": "redondeo" }));
                    }
                }
            }
            cx.global.negocio_cargando = false;
        }
    }
}

fn cargar_negocio(cx: &Contexto, id: i64) -> Tarea<(i64, Result<Value, String>)> {
    let servicios = cx.servicios.clone();
    Tarea::lanzar(&cx.egui, move || {
        let resultado = logica::pedir_negocio(&servicios.api, id);
        if let Ok(negocio) = &resultado {
            servicios.config.set(&logica::clave_negocio(id), negocio.clone());
        }
        (id, resultado)
    })
}

fn id_refresco() -> Id {
    Id::new("sesion-refrescar-negocio")
}

pub fn refrescar_negocio(ctx: &Context) {
    ctx.data_mut(|d| d.insert_temp(id_refresco(), true));
    ctx.request_repaint();
}

fn refresco_pedido(ctx: &Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(id_refresco())).unwrap_or(false)
}

pub fn guardar_admin(cx: &mut Contexto, admin: Value) {
    cx.servicios.config.set("adminData", admin.clone());
    cx.global.admin = Some(admin);
}

pub fn borrar_admin(cx: &mut Contexto) {
    cx.global.admin = None;
    cx.global.negocio = None;
    cx.servicios.config.delete("adminData");
}

pub fn migrar_contrasena_admin(config: &Config) {
    if !config.has(CONTRASENA_ADMIN_PLANA) {
        return;
    }
    let plana = config.get(CONTRASENA_ADMIN_PLANA);
    config.delete(CONTRASENA_ADMIN_PLANA);
    if let Some(cifrada) = plana.as_str().filter(|p| !p.is_empty()).and_then(|p| logica::cifrar(p).ok()) {
        config.set(CONTRASENA_ADMIN, json!(cifrada));
    }
}

pub fn contrasena_admin(config: &Config) -> String {
    migrar_contrasena_admin(config);
    config.get(CONTRASENA_ADMIN).as_str().and_then(|c| logica::descifrar(c).ok()).unwrap_or_default()
}

pub fn recordar_contrasena_admin(config: &Config, contrasena: Option<&str>) {
    match contrasena.map(logica::cifrar) {
        Some(Ok(cifrada)) => config.set(CONTRASENA_ADMIN, json!(cifrada)),
        _ => config.delete(CONTRASENA_ADMIN),
    }
    config.delete(CONTRASENA_ADMIN_PLANA);
}

pub fn iniciar_sesion(cx: &mut Contexto, usuario: Value, email: &str) {
    cx.servicios.config.set("offlineCredentials", json!({ "email": email, "user": usuario }));
    cx.global.usuario = Some(usuario);
}

pub fn cerrar_sesion(cx: &mut Contexto) {
    cx.servicios.config.delete("offlineCredentials");
    cx.global.usuario = None;
    cx.global.ir(Ruta::UsuarioLogin);
}

pub fn destino_tras_login(usuario: &Value) -> Ruta {
    if logica::es_admin(usuario) {
        Ruta::MensajeAdmin
    } else {
        Ruta::Carrito
    }
}

pub fn en<R>(ui: &mut Ui, rect: Rect, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope_builder(UiBuilder::new().max_rect(rect), contenido).inner
}

pub fn linea(ui: &Ui, caja: Rect, contenido: &str, tam: Tam, peso: Peso, color: Color32, horizontal: Align) -> Rect {
    let x = match horizontal {
        Align::Min => caja.left(),
        Align::Center => caja.center().x,
        Align::Max => caja.right(),
    };
    ui.painter().text(egui::pos2(x, caja.center().y), Align2([horizontal, Align::Center]), contenido, fuente(tam, peso), color)
}

pub struct Parrafo {
    galeria: Arc<Galley>,
    ajuste: f32,
    alineacion: Align,
}

impl Parrafo {
    pub fn new(ui: &Ui, contenido: &str, tam: Tam, peso: Peso, color: Color32, ancho: f32, alto_linea: f32, alineacion: Align) -> Self {
        Self::con_espaciado(ui, contenido, tam, peso, color, ancho, alto_linea, alineacion, 0.0)
    }

    pub fn ajustado(ui: &Ui, contenido: &str, tam: Tam, color: Color32, ancho: f32, alto_linea: f32, alineacion: Align) -> Self {
        Self::con_espaciado(ui, contenido, tam, Peso::Negrita, color, ancho, alto_linea, alineacion, -0.025 * tam.px())
    }

    pub fn con_espaciado(
        ui: &Ui,
        contenido: &str,
        tam: Tam,
        peso: Peso,
        color: Color32,
        ancho: f32,
        alto_linea: f32,
        alineacion: Align,
        espaciado: f32,
    ) -> Self {
        let fuente_texto = fuente(tam, peso);
        let fila = ui.ctx().fonts_mut(|f| f.row_height(&fuente_texto));
        let mut trabajo = LayoutJob::simple(contenido.to_string(), fuente_texto, color, ancho);
        trabajo.halign = alineacion;
        trabajo.sections[0].format.line_height = Some(alto_linea);
        trabajo.sections[0].format.extra_letter_spacing = espaciado;
        let galeria = ui.ctx().fonts_mut(|f| f.layout_job(trabajo));
        Parrafo { galeria, ajuste: ((alto_linea - fila) / 2.0).max(0.0), alineacion }
    }

    pub fn alto(&self) -> f32 {
        self.galeria.size().y
    }

    pub fn pintar(&self, ui: &Ui, caja: Rect) {
        let x = match self.alineacion {
            Align::Min => caja.left(),
            Align::Center => caja.center().x,
            Align::Max => caja.right(),
        };
        ui.painter().galley(egui::pos2(x, caja.top() + self.ajuste), self.galeria.clone(), Color32::PLACEHOLDER);
    }
}

pub fn recien_montado(montado_en: &mut Option<u64>, ctx: &Context) -> bool {
    let pase = ctx.cumulative_pass_nr();
    let nuevo = !montado_en.is_some_and(|anterior| anterior.saturating_add(1) >= pase);
    *montado_en = Some(pase);
    nuevo
}
