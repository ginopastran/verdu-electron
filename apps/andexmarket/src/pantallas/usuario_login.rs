use eframe::egui::{self, Align, Align2, CursorIcon, Event, Id, Key, Layout, Rect, Response, ScrollArea, Sense, Stroke, StrokeKind, TextureOptions, Ui, Vec2};
use serde_json::Value;

use super::sesion::{self, en, Parrafo};
use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::marca::{color, fuente, radio, sombra};
use crate::ui::v2::{self, cuerpo_modal, pie_modal, tarjetas, toast, Avatar, Boton, CampoTexto, EstiloTecla, Modal};
use crate::ui::Peso;

const ALTO_ENCABEZADO: f32 = 64.0;
const LADO_AVATAR: f32 = 148.0;
const SEPARACION_GRILLA: f32 = 28.0;
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
                Err(mensaje) => toast::con_detalle(&cx.egui, toast::Tipo::Error, "Error al iniciar sesión", mensaje),
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
        toast::con_detalle(&cx.egui, toast::Tipo::Info, "Reconfigurar negocio", "Limpiando configuración actual...");
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
        ui.painter().with_clip_rect(pantalla).rect_filled(pantalla, 0, color::FONDO_APP);
        let franja = Rect::from_min_size(pantalla.min, Vec2::new(pantalla.width(), ALTO_ENCABEZADO));
        if encabezado(ui, franja, cx) {
            self.dialogo_reconfigurar = true;
        }
        let cuerpo = Rect::from_min_max(egui::pos2(pantalla.left(), franja.bottom()), pantalla.max);
        let elegido = en(ui, cuerpo, |ui| {
            ScrollArea::vertical().id_salt("usuario-login-desplazamiento").auto_shrink(false).show(ui, |ui| self.cuerpo(ui, cuerpo.height())).inner
        });
        if let Some(vendedor) = elegido {
            self.elegir(vendedor);
        }

        if self.dialogo_contrasena && self.elegido.is_some() {
            self.dialogo_contrasena(ui, cx);
        }
        if self.dialogo_reconfigurar {
            self.dialogo_reconfigurar(ui, cx);
        }
    }

    fn cuerpo(&self, ui: &mut Ui, alto_visible: f32) -> Option<Value> {
        let id = Id::new("usuario-login-contenido");
        let ancho_total = ui.available_width();
        let alto = sesion::alto_medido(ui.ctx(), id).map_or(alto_visible, |a| (a + 96.0).max(alto_visible));
        let (zona, _) = ui.allocate_exact_size(Vec2::new(ancho_total, alto), Sense::hover());
        let zona = zona.shrink2(Vec2::new(32.0, 48.0));
        sesion::centrado(ui, id, zona, zona.width(), Align::Center, |ui| {
            v2::titulo(ui, "¿Quién está vendiendo?", 40.0, -0.03, color::TEXTO);
            ui.add_space(10.0);
            v2::texto(ui, "Elegí tu usuario o presioná su número en el teclado.", 15.5, Peso::Normal, color::TEXTO_APAGADO);
            ui.add_space(40.0);
            if self.cargando_vendedores {
                esqueleto(ui);
                None
            } else {
                grilla(ui, &self.vendedores, zona.width().min(1000.0))
            }
        })
    }

    fn dialogo_contrasena(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        let iniciando = self.inicio.is_some();
        let nombre = self.elegido.as_ref().map(|v| texto_de(&v["nombre"])).unwrap_or_default();
        let saludo = format!("Hola, {nombre}");
        let error = if self.intento_enviar { nucleo::sesion::validar_contrasena(&self.contrasena).err() } else { None };
        let mut enviar = false;
        let mut cancelar = false;
        let respuesta = Modal::new("contrasena-vendedor", "Ingresá tu contraseña").subtitulo(&saludo).icono("lock").ancho(440.0).mostrar(ui.ctx(), |ui| {
            cuerpo_modal(ui, 22, |ui| {
                v2::etiqueta(ui, "Contraseña");
                ui.add_space(8.0);
                let campo = CampoTexto::new(&mut self.contrasena)
                    .icono("lock")
                    .sugerencia("Ingresá tu contraseña")
                    .contrasena(true)
                    .alto(48.0)
                    .tam_texto(16.0)
                    .error(error.is_some())
                    .habilitado(!iniciando)
                    .id(id_contrasena())
                    .ui(ui)
                    .respuesta;
                if self.enfocar {
                    campo.request_focus();
                    self.enfocar = false;
                }
                if campo.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    enviar = true;
                }
                if let Some(error) = error {
                    ui.add_space(8.0);
                    v2::texto(ui, error, 13.0, Peso::Normal, color::ROJO);
                }
            });
            pie_modal(ui, |ui| {
                let principal = Boton::primario(if iniciando { "Ingresando…" } else { "Iniciar sesión" }).modal().tam_texto(14.0).cargando(iniciando);
                if principal.ui(ui).clicked() {
                    enviar = true;
                }
                if Boton::secundario("Cancelar").modal().habilitado(!iniciando).ui(ui).clicked() {
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
        let respuesta = Modal::new("reconfigurar-negocio", "¿Reconfigurar el punto de venta?")
            .subtitulo("Vas a volver a la configuración inicial")
            .icono("settings")
            .aviso()
            .ancho(460.0)
            .mostrar(ui.ctx(), |ui| {
                cuerpo_modal(ui, 16, |ui| {
                    let ancho = ui.available_width();
                    Parrafo::new(
                        ui,
                        "Se borra la configuración de esta caja y las credenciales guardadas para modo sin conexión. Vas a necesitar una cuenta de administrador.",
                        14.0,
                        Peso::Normal,
                        color::TEXTO_SECUNDARIO,
                        ancho,
                        1.55,
                    )
                    .ui(ui, ancho);
                });
                pie_modal(ui, |ui| {
                    if boton_aviso(ui, "Sí, reconfigurar").clicked() {
                        confirmar = true;
                    }
                    if Boton::secundario("Cancelar").modal().ui(ui).clicked() {
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

fn encabezado(ui: &mut Ui, franja: Rect, cx: &Contexto) -> bool {
    let pintor = ui.painter();
    pintor.rect_filled(franja, 0, color::BLANCO);
    pintor.hline(franja.x_range(), franja.bottom() - 0.5, Stroke::new(1.0, color::BORDE));
    let interior = Rect::from_min_max(franja.min + Vec2::new(28.0, 0.0), franja.max - Vec2::new(28.0, 1.0));
    en(ui, interior, |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            v2::logo(ui, 26.0);
            if let Some(admin) = &cx.global.admin {
                let negocio = admin["businessName"].as_str().filter(|n| !n.is_empty()).unwrap_or("Negocio");
                ui.add_space(10.0);
                v2::texto(ui, negocio, 15.0, Peso::Semi, color::VERDE_PROFUNDO);
                ui.add_space(10.0);
                v2::texto(ui, format!("· configurado por {}", texto_de(&admin["nombre"])), 13.0, Peso::Normal, color::TEXTO_TENUE);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let clic = boton_reconfigurar(ui).clicked();
                ui.add_space(12.0);
                if cx.global.en_linea {
                    v2::punto_estado(ui, "En línea", color::VERDE_PUNTO, color::VERDE_EXITO, 13.0, Peso::Medio, false);
                } else {
                    v2::punto_estado(ui, "Sin conexión", color::ROJO_INTENSO, color::ROJO, 13.0, Peso::Medio, false);
                }
                clic
            })
            .inner
        })
        .inner
    })
}

fn boton_reconfigurar(ui: &mut Ui) -> Response {
    let ancho_texto = v2::ancho_texto(ui, "Reconfigurar", 13.5, Peso::Medio);
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(12.0 + 17.0 + 8.0 + ancho_texto + 12.0, 36.0), Sense::click());
    let (fondo, borde, tinta) = if respuesta.hovered() {
        (color::DORADO_HOVER, color::DORADO, color::DORADO_TEXTO)
    } else {
        (color::BLANCO, color::BORDE, color::TEXTO_SECUNDARIO)
    };
    ui.painter().rect(rect, radio::CONTROL, fondo, Stroke::new(1.0, borde), StrokeKind::Inside);
    v2::pintar_icono(ui, egui::pos2(rect.left() + 12.0 + 8.5, rect.center().y), "settings", 17.0, tinta);
    v2::pintar_texto(ui, egui::pos2(rect.left() + 12.0 + 17.0 + 8.0, rect.center().y), Align2::LEFT_CENTER, "Reconfigurar", 13.5, Peso::Medio, tinta);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

fn boton_aviso(ui: &mut Ui, texto: &str) -> Response {
    let ancho_texto = v2::ancho_texto(ui, texto, 14.0, Peso::Semi);
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho_texto + 36.0, 42.0), Sense::click());
    ui.painter().rect_filled(rect, radio::BOTON_MODAL, color::DORADO_OSCURO);
    v2::pintar_texto(ui, rect.center(), Align2::CENTER_CENTER, texto, 14.0, Peso::Semi, color::BLANCO);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

fn esqueleto(ui: &mut Ui) {
    let cantidad = 5;
    let ancho = LADO_AVATAR * cantidad as f32 + SEPARACION_GRILLA * (cantidad - 1) as f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, LADO_AVATAR + 14.0 + 14.0), Sense::hover());
    for indice in 0..cantidad {
        let x = rect.left() + indice as f32 * (LADO_AVATAR + SEPARACION_GRILLA);
        let avatar = Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::splat(LADO_AVATAR));
        tarjetas::pintar_shimmer(ui, avatar, radio::AVATAR_GRANDE, true);
        let barra = Rect::from_center_size(egui::pos2(avatar.center().x, avatar.bottom() + 14.0 + 7.0), Vec2::new(96.0, 14.0));
        ui.painter().rect_filled(barra, radio::CHIP, color::SHIMMER_BASE);
    }
}

fn grilla(ui: &mut Ui, vendedores: &[Value], ancho_maximo: f32) -> Option<Value> {
    if vendedores.is_empty() {
        return None;
    }
    let nombres: Vec<String> = vendedores.iter().map(|v| texto_de(&v["nombre"])).collect();
    let anchos: Vec<f32> = nombres.iter().map(|n| v2::ancho_texto(ui, n, 16.0, Peso::Semi).max(LADO_AVATAR) + 16.0).collect();
    let alto_nombre = v2::galeria(ui, "Ág", fuente(16.0, Peso::Semi), color::TEXTO).size().y;
    let alto_tarjeta = 8.0 + LADO_AVATAR + 14.0 + alto_nombre + 8.0;
    let mut lineas: Vec<Vec<usize>> = vec![Vec::new()];
    let mut ancho_linea = 0.0;
    for (indice, ancho) in anchos.iter().enumerate() {
        let linea = lineas.last_mut().expect("siempre hay una línea");
        if !linea.is_empty() && ancho_linea + SEPARACION_GRILLA + ancho > ancho_maximo {
            lineas.push(vec![indice]);
            ancho_linea = *ancho;
        } else {
            ancho_linea += if linea.is_empty() { *ancho } else { SEPARACION_GRILLA + ancho };
            linea.push(indice);
        }
    }
    let ancho_de = |linea: &[usize]| linea.iter().map(|i| anchos[*i]).sum::<f32>() + SEPARACION_GRILLA * linea.len().saturating_sub(1) as f32;
    let ancho_total = lineas.iter().map(|l| ancho_de(l)).fold(0.0, f32::max);
    let alto_total = lineas.len() as f32 * alto_tarjeta + SEPARACION_GRILLA * (lineas.len() - 1) as f32;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho_total, alto_total), Sense::hover());
    let mut elegido = None;
    for (fila, linea) in lineas.iter().enumerate() {
        let mut x = (rect.center().x - ancho_de(linea) / 2.0).round();
        let y = rect.top() + fila as f32 * (alto_tarjeta + SEPARACION_GRILLA);
        for indice in linea {
            let celda = Rect::from_min_size(egui::pos2(x, y), Vec2::new(anchos[*indice], alto_tarjeta));
            if tarjeta_vendedor(ui, celda, &vendedores[*indice], &nombres[*indice], *indice) {
                elegido = Some(vendedores[*indice].clone());
            }
            x += anchos[*indice] + SEPARACION_GRILLA;
        }
    }
    elegido
}

fn tarjeta_vendedor(ui: &mut Ui, celda: Rect, vendedor: &Value, nombre: &str, indice: usize) -> bool {
    let respuesta = ui.interact(celda, Id::new(("usuario-login-vendedor", indice)), Sense::click());
    let alza = ui.ctx().animate_bool_with_time(respuesta.id.with("alza"), respuesta.hovered(), 0.16);
    let caja = celda.translate(Vec2::new(0.0, -3.0 * alza));
    let avatar = Rect::from_min_size(egui::pos2(caja.center().x - LADO_AVATAR / 2.0, caja.top() + 8.0), Vec2::splat(LADO_AVATAR));
    let textura = vendedor["avatar"].as_str().filter(|a| !a.is_empty()).and_then(|uri| {
        match ui.ctx().try_load_texture(uri, TextureOptions::default(), egui::SizeHint::default()) {
            Ok(egui::load::TexturePoll::Ready { texture }) => Some(texture),
            _ => None,
        }
    });
    match textura {
        Some(textura) => {
            ui.painter().add(sombra::AVATAR.forma(avatar, radio::AVATAR_GRANDE));
            egui::Image::from_texture(textura).corner_radius(radio::AVATAR_GRANDE).paint_at(ui, avatar);
        }
        None => Avatar::vendedor(v2::iniciales(nombre), color::AVATARES[indice % color::AVATARES.len()]).pintar(ui, avatar),
    }
    if indice < 9 {
        let numero = Rect::from_min_size(avatar.min + Vec2::splat(10.0), Vec2::splat(26.0));
        v2::pintar_tecla(ui, numero, &(indice + 1).to_string(), &EstiloTecla::NUMERO_AVATAR, color::BLANCO);
    }
    if nucleo::sesion::ruta_protegida_rechaza_admin(vendedor) {
        etiqueta_admin(ui, avatar);
    }
    v2::pintar_texto(ui, egui::pos2(caja.center().x, avatar.bottom() + 14.0), Align2::CENTER_TOP, nombre, 16.0, Peso::Semi, color::TEXTO);
    respuesta.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

fn etiqueta_admin(ui: &Ui, avatar: Rect) {
    let galeria = v2::galeria_espaciada(ui, "ADMIN", fuente(11.0, Peso::Semi), color::VERDE_PROFUNDO, 11.0 * 0.04);
    let tam = galeria.size() + Vec2::new(14.0, 6.0);
    let caja = Rect::from_min_size(avatar.max - Vec2::splat(10.0) - tam, tam);
    ui.painter().rect_filled(caja, radio::CHIP, color::DORADO);
    ui.painter().galley(caja.min + Vec2::new(7.0, 3.0), galeria, color::VERDE_PROFUNDO);
}
