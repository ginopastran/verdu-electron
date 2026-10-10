use eframe::egui::{self, Align, Align2, CursorIcon, Frame, Id, Key, Margin, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use serde_json::{json, Value};

use super::sesion::{self, Parrafo};
use crate::app::{Contexto, Ruta};
use crate::tareas::{self, Tarea};
use crate::ui::marca::{color, fuente, radio};
use crate::ui::v2::{self, Boton, CampoTexto, EstiloTecla};
use crate::ui::{self, toast, Peso};

const ANCHO_FORMULARIO: f32 = 400.0;
const PASOS: [&str; 3] = [
    "Vinculá la caja con tu cuenta de administrador",
    "Cada vendedor elige su usuario e ingresa",
    "Configurá la impresora y la balanza desde el menú",
];

pub fn id_email() -> Id {
    Id::new("admin-login-email")
}

pub fn id_contrasena() -> Id {
    Id::new("admin-login-contrasena")
}

#[derive(Default)]
pub struct AdminLogin {
    pub montado_en: Option<u64>,
    pub email: String,
    pub contrasena: String,
    pub recordar: bool,
    pub ver_contrasena: bool,
    pub error: String,
    pub tarea: Option<Tarea<Result<Value, String>>>,
}

impl AdminLogin {
    fn montar(&mut self, cx: &Contexto) {
        self.email.clear();
        self.contrasena.clear();
        self.recordar = false;
        self.ver_contrasena = false;
        self.error.clear();
        let config = &cx.servicios.config;
        let contrasena = sesion::contrasena_admin(config);
        if let Some(email) = config.get("adminEmail").as_str() {
            if !email.is_empty() && !contrasena.is_empty() && config.get("rememberAdminCredentials") == "true" {
                self.email = email.to_string();
                self.contrasena = contrasena;
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
            sesion::recordar_contrasena_admin(config, Some(&self.contrasena));
            config.set("rememberAdminCredentials", json!("true"));
        } else {
            config.delete("adminEmail");
            sesion::recordar_contrasena_admin(config, None);
            config.delete("rememberAdminCredentials");
        }
    }

    pub fn mostrar(&mut self, ui: &mut Ui, cx: &mut Contexto) {
        if sesion::recien_montado(&mut self.montado_en, ui.ctx()) {
            self.montar(cx);
        }
        self.recibir(cx);
        let pantalla = ui.ctx().content_rect();
        let mitad = (pantalla.left() + pantalla.width() / 2.0).round();
        let izquierda = Rect::from_min_max(pantalla.min, egui::pos2(mitad, pantalla.bottom()));
        let derecha = Rect::from_min_max(egui::pos2(mitad, pantalla.top()), pantalla.max);
        let pintor = ui.painter().with_clip_rect(pantalla);
        pintor.rect_filled(izquierda, 0, color::VERDE_PROFUNDO);
        pintor.rect_filled(derecha, 0, color::FONDO_APP);
        panel_marca(ui, izquierda);
        let zona = derecha.shrink(40.0);
        let ancho = ANCHO_FORMULARIO.min(zona.width());
        let enviar = sesion::centrado(ui, Id::new("admin-login-formulario"), zona, ancho, Align::Min, |ui| self.formulario(ui, ancho));
        if enviar && self.tarea.is_none() {
            self.enviar(cx);
        }
    }

    fn formulario(&mut self, ui: &mut Ui, ancho: f32) -> bool {
        let cargando = self.tarea.is_some();
        let mut enviar = false;
        let (caja, _) = ui.allocate_exact_size(Vec2::splat(48.0), Sense::hover());
        ui.painter().rect_filled(caja, radio::POPOVER, color::VERDE_CHIP);
        v2::pintar_icono(ui, caja.center(), "shield", 26.0, color::VERDE);
        ui.add_space(22.0);
        v2::titulo(ui, "Configuración inicial", 26.0, -0.02, color::TEXTO);
        ui.add_space(6.0);
        Parrafo::new(
            ui,
            "Ingresá con una cuenta de administrador para vincular esta caja al negocio.",
            14.5,
            Peso::Normal,
            color::TEXTO_APAGADO,
            ancho,
            1.5,
        )
        .ui(ui, ancho);
        ui.add_space(22.0);

        if !self.error.is_empty() {
            caja_error(ui, &self.error, ancho);
            ui.add_space(22.0);
        }

        v2::etiqueta(ui, "Email del administrador");
        ui.add_space(6.0);
        let email = CampoTexto::new(&mut self.email)
            .icono("mail")
            .sugerencia("admin@negocio.com")
            .id(id_email())
            .habilitado(!cargando)
            .ancho(ancho)
            .ui(ui);
        enviar |= email.respuesta.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        ui.add_space(22.0);

        v2::etiqueta(ui, "Contraseña");
        ui.add_space(6.0);
        let contrasena = CampoTexto::new(&mut self.contrasena)
            .icono("lock")
            .sugerencia("••••••••")
            .contrasena(!self.ver_contrasena)
            .accion("eye")
            .id(id_contrasena())
            .habilitado(!cargando)
            .ancho(ancho)
            .ui(ui);
        enviar |= contrasena.respuesta.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if contrasena.accion {
            self.ver_contrasena = !self.ver_contrasena;
        }
        ui.add_space(22.0);

        if casilla(ui, self.recordar, "Recordar credenciales para próximas configuraciones", !cargando).clicked() {
            self.recordar = !self.recordar;
        }
        ui.add_space(22.0);

        let boton = ui
            .scope(|ui| {
                if cargando {
                    ui.set_opacity(0.75);
                }
                let boton = Boton::primario(if cargando { "Configurando…" } else { "Configurar negocio" });
                if cargando { boton.sin_sombra() } else { boton }
                    .icono("store")
                    .alto(48.0)
                    .ancho(ancho)
                    .tam_texto(15.0)
                    .tam_icono(19.0)
                    .separacion(10.0)
                    .cargando(cargando)
                    .ui(ui)
            })
            .inner;
        ui.add_space(22.0);
        presiona_enter(ui, ancho);
        enviar || boton.clicked()
    }
}

fn panel_marca(ui: &mut Ui, panel: Rect) {
    let interior = Rect::from_min_max(panel.min + Vec2::new(56.0, 48.0), panel.max - Vec2::new(56.0, 48.0));
    let logo = Rect::from_min_size(interior.min, Vec2::splat(32.0));
    ui::imagen("andex-market-logo.png").paint_at(ui, logo);
    let alto_texto = 150.0 * 28.0 / 230.0;
    let texto_logo = Rect::from_min_size(egui::pos2(logo.right() + 12.0, logo.center().y - alto_texto / 2.0), Vec2::new(150.0, alto_texto));
    ui::imagen("andex-market-text.png").paint_at(ui, texto_logo);

    let ancho = interior.width().min(440.0);
    let etiqueta = v2::galeria_espaciada(ui, "PUNTO DE VENTA", fuente(13.0, Peso::Semi), color::DORADO, 13.0 * 0.08);
    let titulo = Parrafo::con(ui, "Configurá esta caja una sola vez.", 40.0, Peso::Semi, color::BLANCO, ancho, 1.1, Align::Min, -0.025);
    let bajada = Parrafo::new(
        ui,
        "Ingresá con la cuenta de administrador del negocio. Después, cada vendedor entra con su propio usuario.",
        16.0,
        Peso::Normal,
        color::SOBRE_OSCURO_PARRAFO,
        ancho,
        1.55,
    );
    let version = v2::galeria(ui, format!("Andex Market POS · v{}", env!("CARGO_PKG_VERSION")), fuente(12.5, Peso::Normal), color::SOBRE_OSCURO_TENUE);
    let alto_medio = etiqueta.size().y + 18.0 + titulo.alto() + 18.0 + bajada.alto();
    let alto_pie = 28.0 * PASOS.len() as f32 + 12.0 * (PASOS.len() - 1) as f32 + 12.0 + 12.0 + version.size().y;
    let libre = (interior.height() - 32.0 - alto_medio - alto_pie).max(0.0);
    let x = interior.left();
    let mut y = (interior.top() + 32.0 + libre / 2.0).round();
    let alto_etiqueta = etiqueta.size().y;
    ui.painter().galley(egui::pos2(x, y), etiqueta, color::DORADO);
    y += alto_etiqueta + 18.0;
    titulo.pintar(ui, Rect::from_min_size(egui::pos2(x, y), Vec2::new(ancho, titulo.alto())));
    y += titulo.alto() + 18.0;
    bajada.pintar(ui, Rect::from_min_size(egui::pos2(x, y), Vec2::new(ancho, bajada.alto())));

    let mut y = (interior.bottom() - alto_pie).round();
    for (indice, paso) in PASOS.iter().enumerate() {
        let circulo = egui::pos2(x + 14.0, y + 14.0);
        ui.painter().circle_filled(circulo, 14.0, color::VERDE_HOVER_OSCURO);
        v2::pintar_texto(ui, circulo, Align2::CENTER_CENTER, (indice + 1).to_string(), 13.0, Peso::Semi, color::DORADO);
        v2::pintar_texto(ui, egui::pos2(x + 40.0, circulo.y), Align2::LEFT_CENTER, *paso, 14.0, Peso::Normal, color::SOBRE_OSCURO_LISTA);
        y += 28.0 + 12.0;
    }
    ui.painter().galley(egui::pos2(x, y + 12.0), version, color::SOBRE_OSCURO_TENUE);
}

fn caja_error(ui: &mut Ui, mensaje: &str, ancho: f32) {
    let interior = ancho - 28.0 - 2.0;
    Frame::NONE
        .fill(color::ROJO_FONDO)
        .stroke(Stroke::new(1.0, color::ROJO_BORDE))
        .corner_radius(radio::CAMPO)
        .inner_margin(Margin { left: 14, right: 14, top: 12, bottom: 12 })
        .show(ui, |ui| {
            ui.set_width(interior);
            ui.horizontal_top(|ui| {
                let (icono, _) = ui.allocate_exact_size(Vec2::new(18.0 + 10.0, 18.0), Sense::hover());
                ui::pintar_icono_solar_en(ui, Rect::from_min_size(icono.min, Vec2::splat(18.0)), "alert", color::ROJO);
                Parrafo::new(ui, mensaje, 13.5, Peso::Normal, color::ROJO, interior - 28.0, 1.45).ui(ui, interior - 28.0);
            });
        });
}

fn casilla(ui: &mut Ui, marcada: bool, texto: &str, habilitada: bool) -> egui::Response {
    let galeria = v2::galeria(ui, texto, fuente(14.0, Peso::Normal), color::TEXTO_SECUNDARIO);
    let tam = Vec2::new(18.0 + 10.0 + galeria.size().x, galeria.size().y.max(18.0));
    let (rect, respuesta) = ui.allocate_exact_size(tam, if habilitada { Sense::click() } else { Sense::hover() });
    let caja = Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 9.0), Vec2::splat(18.0));
    let (borde, fondo) = if marcada { (color::VERDE, color::VERDE) } else { (color::BORDE_HOVER, color::BLANCO) };
    ui.painter().rect(caja, radio::TECLA, fondo, Stroke::new(1.5, borde), StrokeKind::Inside);
    if marcada {
        v2::pintar_icono(ui, caja.center(), "check", 14.0, color::BLANCO);
    }
    let alto_texto = galeria.size().y;
    ui.painter().galley(egui::pos2(caja.right() + 10.0, rect.center().y - alto_texto / 2.0), galeria, color::TEXTO_SECUNDARIO);
    if habilitada {
        respuesta.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        respuesta
    }
}

fn presiona_enter(ui: &mut Ui, ancho: f32) {
    let antes = v2::galeria(ui, "Presioná", fuente(13.0, Peso::Normal), color::TEXTO_TENUE);
    let despues = v2::galeria(ui, "para continuar", fuente(13.0, Peso::Normal), color::TEXTO_TENUE);
    let tecla = v2::medir_tecla(ui, "Enter", &EstiloTecla::EN_LINEA);
    let espacio = v2::ancho_texto(ui, "a b", 13.0, Peso::Normal) - v2::ancho_texto(ui, "ab", 13.0, Peso::Normal);
    let (ancho_antes, ancho_despues, alto_texto) = (antes.size().x, despues.size().x, antes.size().y);
    let total = ancho_antes + espacio + tecla.x + espacio + ancho_despues;
    let (fila, _) = ui.allocate_exact_size(Vec2::new(ancho, alto_texto), Sense::hover());
    let y = fila.center().y;
    let mut x = (fila.center().x - total / 2.0).round();
    ui.painter().galley(egui::pos2(x, y - alto_texto / 2.0), antes, color::TEXTO_TENUE);
    x += ancho_antes + espacio;
    v2::pintar_tecla(ui, Rect::from_min_size(egui::pos2(x, (y - tecla.y / 2.0).round()), tecla), "Enter", &EstiloTecla::EN_LINEA, color::TEXTO_SECUNDARIO);
    x += tecla.x + espacio;
    ui.painter().galley(egui::pos2(x, y - alto_texto / 2.0), despues, color::TEXTO_TENUE);
}
