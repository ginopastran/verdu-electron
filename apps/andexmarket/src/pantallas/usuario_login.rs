use eframe::egui::{self, Align, Color32, Event, Id, Key, Rect, Sense, StrokeKind, TextureOptions, Ui, Vec2};
use serde_json::Value;

use super::sesion::{self, en, linea, Parrafo};
use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::{self, color, paleta, toast, Boton, Direccion, Dialogo, Entrada, Peso, Tam, Variante};

const TECLAS: [Key; 9] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];

pub struct UsuarioLogin {
    pub montado_en: Option<u64>,
    pub clave_carga: Option<(i64, bool)>,
    pub cargando_vendedores: bool,
    pub vendedores: Vec<Value>,
    pub carga_vendedores: Option<Tarea<Result<Vec<Value>, String>>>,
    pub elegido: Option<Value>,
    pub dialogo_contrasena: bool,
    pub contrasena: String,
    pub intento_enviar: bool,
    pub enfocar: bool,
    pub inicio: Option<Tarea<Result<Value, String>>>,
    pub dialogo_reconfigurar: bool,
}

impl Default for UsuarioLogin {
    fn default() -> Self {
        UsuarioLogin {
            montado_en: None,
            clave_carga: None,
            cargando_vendedores: true,
            vendedores: Vec::new(),
            carga_vendedores: None,
            elegido: None,
            dialogo_contrasena: false,
            contrasena: String::new(),
            intento_enviar: false,
            enfocar: false,
            inicio: None,
            dialogo_reconfigurar: false,
        }
    }
}

fn id_contrasena() -> Id {
    Id::new("usuario-login-contrasena")
}

impl UsuarioLogin {
    fn seguir_vendedores(&mut self, cx: &mut Contexto) {
        let Some(business_id) = cx.global.business_id() else {
            return;
        };
        let clave = Some((business_id, cx.global.en_linea));
        if clave == self.clave_carga {
            return;
        }
        self.clave_carga = clave;
        self.cargando_vendedores = true;
        if !cx.global.en_linea {
            toast::error(&cx.egui, "No hay conexión a internet");
            self.cargando_vendedores = false;
            return;
        }
        let servicios = cx.servicios.clone();
        self.carga_vendedores = Some(Tarea::lanzar(&cx.egui, move || nucleo::sesion::vendedores(&servicios.api, business_id)));
    }

    fn recibir(&mut self, cx: &mut Contexto) {
        if let Some(resultado) = tareas::esperar(&mut self.carga_vendedores) {
            match resultado {
                Ok(vendedores) => self.vendedores = vendedores,
                Err(_) => toast::con_detalle(
                    &cx.egui,
                    toast::Tipo::Error,
                    "Error al cargar vendedores",
                    "Por favor, intenta nuevamente más tarde.",
                    2.0,
                ),
            }
            self.cargando_vendedores = false;
        }
        if let Some(resultado) = tareas::esperar(&mut self.inicio) {
            match resultado {
                Ok(usuario) => {
                    let email = self.elegido.as_ref().and_then(|v| v["email"].as_str()).unwrap_or_default().to_string();
                    let destino = sesion::destino_tras_login(&usuario);
                    sesion::iniciar_sesion(cx, usuario, &email);
                    self.dialogo_contrasena = false;
                    cx.global.ir(destino);
                }
                Err(mensaje) => toast::con_detalle(&cx.egui, toast::Tipo::Error, "Error al iniciar sesión", mensaje, 2.0),
            }
        }
    }

    fn elegir(&mut self, vendedor: Value) {
        self.elegido = Some(vendedor);
        self.dialogo_contrasena = true;
        self.contrasena.clear();
        self.intento_enviar = false;
        self.enfocar = true;
    }

    fn enviar_contrasena(&mut self, cx: &mut Contexto) {
        self.intento_enviar = true;
        if nucleo::sesion::validar_contrasena(&self.contrasena).is_err() {
            return;
        }
        let Some(vendedor) = self.elegido.clone() else {
            return;
        };
        let email = vendedor["email"].as_str().unwrap_or_default().to_string();
        if !cx.global.en_linea {
            let guardadas = cx.servicios.config.get("offlineCredentials");
            if !guardadas.is_null() && guardadas["email"] == email.as_str() {
                let usuario = guardadas["user"].clone();
                let destino = sesion::destino_tras_login(&usuario);
                cx.global.usuario = Some(usuario);
                cx.global.ir(destino);
            } else {
                toast::error(&cx.egui, "No hay credenciales guardadas para modo offline");
            }
            return;
        }
        let servicios = cx.servicios.clone();
        let contrasena = self.contrasena.clone();
        self.inicio = Some(Tarea::lanzar(&cx.egui, move || nucleo::sesion::iniciar_vendedor(&servicios.api, &email, &contrasena)));
    }

    fn reconfigurar(&mut self, cx: &mut Contexto) {
        self.dialogo_reconfigurar = false;
        toast::con_detalle(&cx.egui, toast::Tipo::Info, "Reconfigurar negocio", "Limpiando configuración actual...", 2.0);
        sesion::borrar_admin(cx);
        cx.global.usuario = None;
        cx.servicios.config.delete("offlineCredentials");
        cx.global.ir(Ruta::AdminLogin);
    }

    fn atajos(&mut self, ui: &mut Ui) {
        if ui.ctx().egui_wants_keyboard_input() || self.dialogo_contrasena || self.dialogo_reconfigurar {
            return;
        }
        let Some(indice) = TECLAS.iter().position(|tecla| ui.input(|i| i.key_pressed(*tecla))) else {
            return;
        };
        let digito = (indice + 1).to_string();
        ui.input_mut(|i| i.events.retain(|e| !matches!(e, Event::Text(t) if *t == digito)));
        if let Some(vendedor) = self.vendedores.get(indice).cloned() {
            self.elegir(vendedor);
        }
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if sesion::recien_montado(&mut self.montado_en, ui.ctx()) {
            *self = UsuarioLogin { montado_en: self.montado_en, ..UsuarioLogin::default() };
        }
        self.seguir_vendedores(cx);
        self.recibir(cx);
        if cx.global.ruta != Ruta::UsuarioLogin {
            return;
        }
        self.atajos(ui);

        let pantalla = ui.ctx().content_rect();
        ui.painter().with_clip_rect(pantalla).rect_filled(pantalla, 0, crate::ui::color::FONDO);
        let ancho = 1024.0_f32.min(pantalla.width()) - 48.0;
        let izquierda = (pantalla.center().x - ancho / 2.0).round();
        let mut y = pantalla.top() + 32.0 + 24.0;

        let boton = Rect::from_min_size(egui::pos2(izquierda + ancho - 137.0, y + 6.0), Vec2::new(137.0, 36.0));
        let respuesta = en(ui, boton, |ui| boton_reconfigurar(ui));
        if respuesta.on_hover_text("Limpiar configuración actual y volver al login de administrador").clicked() {
            self.dialogo_reconfigurar = true;
        }
        let titulo = Rect::from_min_size(egui::pos2(izquierda, y), Vec2::new(ancho - boton.width(), 48.0));
        linea(ui, titulo, "¿Quién está vendiendo?", Tam::X5, Peso::Negrita, sesion::DEGRADADO_TEXTO, Align::Center);
        y += 48.0 + 32.0;

        if let Some(admin) = &cx.global.admin {
            let banda = Rect::from_min_size(egui::pos2(izquierda, y), Vec2::new(ancho, 58.0));
            ui.painter().rect(banda, ui::RADIO, paleta::EMERALD_50, ui::borde(paleta::EMERALD_200), StrokeKind::Inside);
            let icono = Rect::from_min_size(egui::pos2(banda.left() + 17.0, banda.center().y - 8.0), Vec2::splat(16.0));
            ui::pintar_icono_en(ui, icono, "building-2", paleta::EMERALD_700);
            let negocio = admin["businessName"].as_str().filter(|n| !n.is_empty()).unwrap_or("Negocio");
            let nombre = admin["nombre"].as_str().map(str::to_string).unwrap_or_else(|| texto_de(&admin["nombre"]));
            let texto = Rect::from_min_max(egui::pos2(icono.right() + 8.0, banda.top()), banda.max);
            linea(ui, texto, &format!("{negocio} - Configurado por {nombre}"), Tam::Base, Peso::Medio, paleta::EMERALD_700, Align::Min);
            y += 58.0 + 24.0;
        }

        if self.cargando_vendedores {
            let zona = Rect::from_min_size(egui::pos2(izquierda, y), Vec2::new(ancho, 300.0));
            ui::girando(ui, Rect::from_center_size(zona.center(), Vec2::splat(48.0)), paleta::EMERALD_400);
        } else {
            let columnas = 5usize;
            let ancho_columna = (ancho - 32.0 * (columnas - 1) as f32) / columnas as f32;
            let filas = self.vendedores.len().div_ceil(columnas);
            let mut elegido = None;
            for (indice, vendedor) in self.vendedores.iter().enumerate() {
                let (fila, columna) = (indice / columnas, indice % columnas);
                let celda = Rect::from_min_size(
                    egui::pos2(izquierda + columna as f32 * (ancho_columna + 32.0), y + fila as f32 * (196.0 + 32.0)),
                    Vec2::new(ancho_columna, 196.0),
                );
                if tarjeta_vendedor(ui, celda, vendedor, indice + 1) {
                    elegido = Some(vendedor.clone());
                }
            }
            if let Some(vendedor) = elegido {
                self.elegir(vendedor);
            }
            y += filas as f32 * 196.0 + filas.saturating_sub(1) as f32 * 32.0 + 32.0;
            let ayuda = Rect::from_min_size(egui::pos2(izquierda, y), Vec2::new(ancho, 24.0));
            linea(
                ui,
                ayuda,
                "También puedes presionar las teclas numéricas 1-9 para seleccionar un vendedor",
                Tam::Base,
                Peso::Normal,
                paleta::GRAY_500,
                Align::Center,
            );
        }

        if self.dialogo_contrasena && self.elegido.is_some() {
            self.dialogo_contrasena(ui, cx);
        }
        if self.dialogo_reconfigurar {
            self.dialogo_reconfigurar(ui, cx);
        }
    }

    fn dialogo_contrasena(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        let iniciando = self.inicio.is_some();
        let nombre = self.elegido.as_ref().map(|v| texto_de(&v["nombre"])).unwrap_or_default();
        let error = if self.intento_enviar { nucleo::sesion::validar_contrasena(&self.contrasena).err() } else { None };
        let mut enviar = false;
        let mut cancelar = false;
        let respuesta = Dialogo::new("contrasena-vendedor").ancho(448.0).mostrar(ui.ctx(), |ui| {
            let ancho = ui.available_width();
            let titulo = Parrafo::ajustado(ui, &format!("Contraseña para {nombre}"), Tam::X2, sesion::DEGRADADO_TEXTO, ancho, 32.0, Align::Min);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, titulo.alto()), Sense::hover());
            titulo.pintar(ui, rect);
            ui.add_space(16.0 + 16.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 24.0), Sense::hover());
            let color_etiqueta = if error.is_some() { color::DESTRUCTIVO } else { color::TEXTO };
            linea(ui, rect, "Contraseña", Tam::Sm, Peso::Medio, color_etiqueta, Align::Min);
            ui.add_space(8.0);
            let campo = Entrada::new(&mut self.contrasena)
                .sugerencia("Ingresa tu contraseña")
                .contrasena(true)
                .habilitado(!iniciando)
                .tam(Tam::Sm)
                .id(id_contrasena())
                .ui(ui);
            if self.enfocar {
                campo.request_focus();
                self.enfocar = false;
            }
            if campo.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                enviar = true;
            }
            if let Some(error) = error {
                ui.add_space(8.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 20.0), Sense::hover());
                linea(ui, rect, error, Tam::Sm, Peso::Medio, color::DESTRUCTIVO, Align::Min);
            }
            ui.add_space(24.0);
            ui::pie_dialogo(ui, |ui| {
                let principal = Boton::new(if iniciando { "Iniciando sesión..." } else { "Iniciar Sesión" })
                    .variante(Variante::Esmeralda)
                    .cargando(iniciando);
                if principal.ui(ui).clicked() {
                    enviar = true;
                }
                if Boton::new("Cancelar").variante(Variante::Contorno).habilitado(!iniciando).ui(ui).clicked() {
                    cancelar = true;
                }
            });
        });
        if respuesta.cerrar || cancelar {
            self.dialogo_contrasena = false;
        }
        if enviar && !iniciando {
            self.enviar_contrasena(cx);
        }
    }

    fn dialogo_reconfigurar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        let mut confirmar = false;
        let mut cancelar = false;
        let respuesta = Dialogo::new("reconfigurar-negocio").ancho(448.0).mostrar(ui.ctx(), |ui| {
            let ancho = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, 28.0), Sense::hover());
            ui::pintar_icono_en(ui, Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 10.0), Vec2::splat(20.0)), "settings", paleta::ORANGE_600);
            let titulo = Parrafo::ajustado(ui, "Confirmar Reconfiguración", Tam::Xl, paleta::ORANGE_600, ancho - 28.0, 28.0, Align::Min);
            titulo.pintar(ui, Rect::from_min_max(egui::pos2(rect.left() + 28.0, rect.top()), rect.max));
            ui.add_space(16.0 + 16.0);
            let pregunta = Parrafo::new(
                ui,
                "¿Estás seguro de que deseas reconfigurar el punto de venta?",
                Tam::Base,
                Peso::Normal,
                paleta::GRAY_700,
                ancho,
                24.0,
                Align::Min,
            );
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, pregunta.alto()), Sense::hover());
            pregunta.pintar(ui, rect);
            ui.add_space(12.0);
            let detalle = Parrafo::new(
                ui,
                "Esta acción limpiará la configuración actual y te redirigirá al login de administrador.",
                Tam::Sm,
                Peso::Normal,
                paleta::GRAY_500,
                ancho,
                20.0,
                Align::Min,
            );
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, detalle.alto()), Sense::hover());
            detalle.pintar(ui, rect);
            ui.add_space(16.0 + 16.0);
            ui::pie_dialogo(ui, |ui| {
                let si = Boton::new("Sí, Reconfigurar").colores(paleta::ORANGE_600, Color32::WHITE);
                if si.ui(ui).clicked() {
                    confirmar = true;
                }
                if Boton::new("Cancelar").variante(Variante::Contorno).ui(ui).clicked() {
                    cancelar = true;
                }
            });
        });
        if respuesta.cerrar || cancelar {
            self.dialogo_reconfigurar = false;
        }
        if confirmar {
            self.reconfigurar(cx);
        }
    }
}

fn texto_de(valor: &Value) -> String {
    match valor {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        otro => otro.to_string(),
    }
}

fn boton_reconfigurar(ui: &mut Ui) -> egui::Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(137.0, 36.0), Sense::click());
    let encima = respuesta.hovered();
    let (fondo, trazo, texto) = if encima {
        (paleta::ORANGE_50, paleta::ORANGE_300, paleta::ORANGE_700)
    } else {
        (color::FONDO, color::BORDE, color::TEXTO)
    };
    ui.painter().rect(rect, ui::RADIO, fondo, ui::borde(trazo), StrokeKind::Inside);
    let icono = Rect::from_min_size(egui::pos2(rect.left() + 13.0, rect.center().y - 8.0), Vec2::splat(16.0));
    ui::pintar_icono_en(ui, icono, "settings", texto);
    let resto = Rect::from_min_max(egui::pos2(icono.right() + 8.0, rect.top()), rect.max);
    linea(ui, resto, "Reconfigurar", Tam::Sm, Peso::Medio, texto, Align::Min);
    respuesta.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn tarjeta_vendedor(ui: &mut Ui, celda: Rect, vendedor: &Value, numero: usize) -> bool {
    let base = Rect::from_center_size(egui::pos2(celda.center().x, celda.top() + 80.0), Vec2::splat(160.0));
    let respuesta = ui.interact(base, ui.id().with(("vendedor", numero)), Sense::click());
    let escala = ui.ctx().animate_bool_with_time(respuesta.id, respuesta.hovered(), 0.2);
    let caja = Rect::from_center_size(base.center(), base.size() * (1.0 + 0.05 * escala));
    ui.painter().add(ui::sombra_lg().as_shape(caja, ui::RADIO));
    let nombre = texto_de(&vendedor["nombre"]);
    let avatar = vendedor["avatar"].as_str().filter(|a| !a.is_empty()).and_then(|uri| {
        match ui.ctx().try_load_texture(uri, TextureOptions::default(), egui::SizeHint::default()) {
            Ok(egui::load::TexturePoll::Ready { texture }) => Some(texture),
            _ => None,
        }
    });
    ui::pintar_degradado(ui.painter(), caja, ui::RADIO, color::ESMERALDA_DESDE, color::ESMERALDA_HASTA, Direccion::AbajoDerecha);
    match avatar {
        Some(textura) => egui::Image::from_texture(textura).corner_radius(ui::RADIO).paint_at(ui, caja),
        None => {
            let inicial: String = nombre.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
            linea(ui, caja, &inicial, Tam::X5, Peso::Negrita, Color32::WHITE, Align::Center);
        }
    }
    if numero <= 9 {
        let insignia = egui::pos2(caja.left() + 20.0, caja.top() + 20.0);
        ui.painter().circle_filled(insignia, 12.0, Color32::from_black_alpha(102));
        linea(ui, Rect::from_center_size(insignia, Vec2::splat(24.0)), &numero.to_string(), Tam::Base, Peso::Negrita, Color32::WHITE, Align::Center);
    }
    let pie = Rect::from_min_size(egui::pos2(celda.left() - 20.0, base.bottom() + 8.0), Vec2::new(celda.width() + 40.0, 28.0));
    linea(ui, pie, &nombre, Tam::Lg, Peso::Negrita, sesion::DEGRADADO_TEXTO, Align::Center);
    respuesta.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}
