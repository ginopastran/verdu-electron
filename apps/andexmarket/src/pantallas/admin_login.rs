use eframe::egui::{self, Align, Key, Rect, StrokeKind, Ui, Vec2};
use serde_json::{json, Value};

use super::sesion::{self, en, linea, Parrafo};
use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::{self, color, paleta, toast, Boton, Direccion, Entrada, Peso, Tam, Variante};

const ANCHO_TARJETA: f32 = 448.0;
const RELLENO: f32 = 24.0;

#[derive(Default)]
pub struct AdminLogin {
    pub montado_en: Option<u64>,
    pub email: String,
    pub contrasena: String,
    pub recordar: bool,
    pub error: String,
    pub tarea: Option<Tarea<Result<Value, String>>>,
}

impl AdminLogin {
    fn montar(&mut self, cx: &Contexto) {
        self.email.clear();
        self.contrasena.clear();
        self.recordar = false;
        self.error.clear();
        let config = &cx.servicios.config;
        let (email, contrasena) = (config.get("adminEmail"), config.get("adminPassword"));
        if let (Some(email), Some(contrasena)) = (email.as_str(), contrasena.as_str()) {
            if !email.is_empty() && !contrasena.is_empty() && config.get("rememberAdminCredentials") == "true" {
                self.email = email.to_string();
                self.contrasena = contrasena.to_string();
                self.recordar = true;
            }
        }
    }

    fn enviar(&mut self, cx: &Contexto) {
        if self.email.is_empty() || self.contrasena.is_empty() {
            self.error = "Por favor, ingresa email y contraseña".into();
            return;
        }
        self.error.clear();
        let servicios = cx.servicios.clone();
        let (email, contrasena) = (self.email.clone(), self.contrasena.clone());
        self.tarea = Some(Tarea::lanzar(&cx.egui, move || nucleo::sesion::configurar_negocio(&servicios.api, &email, &contrasena)));
    }

    fn recibir(&mut self, cx: &mut Contexto) {
        let Some(resultado) = tareas::esperar(&mut self.tarea) else {
            return;
        };
        match resultado {
            Ok(admin) => {
                let nombre = admin["businessName"].as_str().unwrap_or_default().to_string();
                sesion::guardar_admin(cx, admin);
                self.guardar_credenciales(cx);
                toast::con_detalle(&cx.egui, toast::Tipo::Exito, "Configuración guardada", format!("Negocio: {nombre}"), 2.0);
                cx.global.ir(Ruta::UsuarioLogin);
            }
            Err(mensaje) => {
                self.error = mensaje.clone();
                toast::con_detalle(&cx.egui, toast::Tipo::Error, "Error al configurar", mensaje, 2.0);
            }
        }
    }

    fn guardar_credenciales(&self, cx: &Contexto) {
        let config = &cx.servicios.config;
        if self.recordar {
            config.set("adminEmail", json!(self.email));
            config.set("adminPassword", json!(self.contrasena));
            config.set("rememberAdminCredentials", json!("true"));
        } else {
            config.delete("adminEmail");
            config.delete("adminPassword");
            config.delete("rememberAdminCredentials");
        }
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if sesion::recien_montado(&mut self.montado_en, ui.ctx()) {
            self.montar(cx);
        }
        self.recibir(cx);
        let cargando = self.tarea.is_some();
        let pantalla = ui.ctx().content_rect();
        ui::pintar_degradado(&ui.painter().with_clip_rect(pantalla), pantalla, 0, paleta::EMERALD_50, paleta::BLUE_50, Direccion::AbajoDerecha);

        let interior = ANCHO_TARJETA - RELLENO * 2.0;
        let descripcion = Parrafo::new(
            ui,
            "Ingresa con una cuenta de administrador para configurar el negocio",
            Tam::Sm,
            Peso::Normal,
            paleta::GRAY_600,
            interior,
            20.0,
            Align::Center,
        );
        let alerta = (!self.error.is_empty())
            .then(|| Parrafo::new(ui, &self.error, Tam::Sm, Peso::Normal, color::DESTRUCTIVO, interior - 34.0, 20.0, Align::Min));
        let alto_alerta = alerta.as_ref().map_or(0.0, |a| a.alto() + 34.0 + 16.0);
        let alto = 1.0 + RELLENO + 48.0 + 16.0 + 6.0 + 32.0 + 6.0 + descripcion.alto() + RELLENO
            + alto_alerta + (24.0 + 8.0 + 40.0) * 2.0 + 16.0 + 16.0 + 20.0 + 16.0 + 40.0 + RELLENO + 1.0;
        let centro = pantalla.center().round();
        let tarjeta = Rect::from_center_size(centro, Vec2::new(ANCHO_TARJETA, alto.round()));
        ui.painter().add(ui::sombra_sm().as_shape(tarjeta, ui::RADIO));
        ui.painter().rect(tarjeta, ui::RADIO, color::FONDO, ui::borde(color::BORDE), StrokeKind::Inside);

        let x = tarjeta.left() + RELLENO;
        let fila = |y: f32, alto: f32| Rect::from_min_size(egui::pos2(x, y), Vec2::new(interior, alto));
        let mut y = tarjeta.top() + 1.0 + RELLENO;
        let circulo = egui::pos2(tarjeta.center().x, y + 24.0);
        ui.painter().circle_filled(circulo, 24.0, paleta::EMERALD_100);
        ui::pintar_icono_en(ui, Rect::from_center_size(circulo, Vec2::splat(24.0)), "user-cog", paleta::EMERALD_600);
        y += 48.0 + 16.0 + 6.0;
        Parrafo::ajustado(ui, "Configuración Inicial", Tam::X2, sesion::DEGRADADO_TEXTO, interior, 32.0, Align::Center).pintar(ui, fila(y, 32.0));
        y += 32.0 + 6.0;
        descripcion.pintar(ui, fila(y, descripcion.alto()));
        y += descripcion.alto() + RELLENO;

        if let Some(alerta) = &alerta {
            let caja = fila(y, alerta.alto() + 34.0);
            ui.painter().rect_stroke(caja, ui::RADIO, ui::borde(color::DESTRUCTIVO.gamma_multiply(0.5)), StrokeKind::Inside);
            alerta.pintar(ui, caja.shrink(17.0));
            y += caja.height() + 16.0;
        }

        let mut enviar = false;
        let mut campo = |ui: &mut Ui, y: &mut f32, etiqueta: &str, texto: &mut String, sugerencia: &str, contrasena: bool| {
            linea(ui, fila(*y, 24.0), etiqueta, Tam::Sm, Peso::Medio, color::TEXTO, Align::Min);
            *y += 24.0 + 8.0;
            let r = en(ui, fila(*y, 40.0), |ui| {
                Entrada::new(texto).sugerencia(sugerencia).contrasena(contrasena).habilitado(!cargando).tam(Tam::Sm).ui(ui)
            });
            if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                enviar = true;
            }
            *y += 40.0 + 16.0;
        };
        campo(ui, &mut y, "Email del Administrador", &mut self.email, "admin@empresa.com", false);
        campo(ui, &mut y, "Contraseña", &mut self.contrasena, "••••••••", true);

        en(ui, fila(y, 20.0), |ui| {
            ui.add_enabled_ui(!cargando, |ui| {
                ui.horizontal_centered(|ui| {
                    ui::casilla(ui, &mut self.recordar, "");
                    ui.add_space(8.0);
                    let texto = ui.add(
                        egui::Label::new(ui::texto("Recordar credenciales para próximas configuraciones", Tam::Sm, Peso::Normal, color::TEXTO))
                            .sense(egui::Sense::click()),
                    );
                    if texto.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.recordar = !self.recordar;
                    }
                });
            });
        });
        y += 20.0 + 16.0;

        let boton = en(ui, fila(y, 40.0), |ui| {
            let boton = Boton::new(if cargando { "Configurando..." } else { "Configurar Negocio" })
                .variante(Variante::Esmeralda)
                .ancho(interior)
                .cargando(cargando);
            if cargando { boton } else { boton.icono("building-2") }.ui(ui)
        });
        if (boton.clicked() || enviar) && !cargando {
            self.enviar(cx);
        }
    }
}
