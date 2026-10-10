use eframe::egui::{
    self, Align, Align2, Color32, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    WidgetText,
};

use super::{borde, color, fuente, paleta, pintar_degradado, Direccion, Peso, Tam, RADIO, RADIO_MD, RADIO_SM, RADIO_XL};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variante {
    Primario,
    Destructivo,
    Contorno,
    Secundario,
    Fantasma,
    Enlace,
    Esmeralda,
    Cancelar,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TamBoton {
    Normal,
    Chico,
    Grande,
    Icono,
}

pub struct Boton<'a> {
    texto: String,
    variante: Variante,
    tam: TamBoton,
    icono: Option<&'a str>,
    icono_derecha: Option<&'a str>,
    tam_icono: f32,
    ancho: Option<f32>,
    alto: Option<f32>,
    habilitado: bool,
    cargando: bool,
    tam_texto: Tam,
    peso: Peso,
    radio: u8,
    fondo: Option<Color32>,
    color_texto: Option<Color32>,
    color_borde: Option<Color32>,
    degradado: Option<(Color32, Color32, Direccion)>,
}

impl<'a> Boton<'a> {
    pub fn new(texto: impl Into<String>) -> Self {
        Boton {
            texto: texto.into(),
            variante: Variante::Primario,
            tam: TamBoton::Normal,
            icono: None,
            icono_derecha: None,
            tam_icono: 16.0,
            ancho: None,
            alto: None,
            habilitado: true,
            cargando: false,
            tam_texto: Tam::Sm,
            peso: Peso::Medio,
            radio: RADIO,
            fondo: None,
            color_texto: None,
            color_borde: None,
            degradado: None,
        }
    }

    pub fn variante(mut self, variante: Variante) -> Self {
        self.variante = variante;
        self
    }

    pub fn tam(mut self, tam: TamBoton) -> Self {
        self.tam = tam;
        self
    }

    pub fn icono(mut self, icono: &'a str) -> Self {
        self.icono = Some(icono);
        self
    }

    pub fn icono_derecha(mut self, icono: &'a str) -> Self {
        self.icono_derecha = Some(icono);
        self
    }

    pub fn tam_icono(mut self, tam: f32) -> Self {
        self.tam_icono = tam;
        self
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = Some(ancho);
        self
    }

    pub fn lleno(self, ui: &Ui) -> Self {
        let ancho = ui.available_width();
        self.ancho(ancho)
    }

    pub fn alto(mut self, alto: f32) -> Self {
        self.alto = Some(alto);
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.habilitado = habilitado;
        self
    }

    pub fn cargando(mut self, cargando: bool) -> Self {
        self.cargando = cargando;
        self
    }

    pub fn texto_tam(mut self, tam: Tam) -> Self {
        self.tam_texto = tam;
        self
    }

    pub fn peso(mut self, peso: Peso) -> Self {
        self.peso = peso;
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn colores(mut self, fondo: Color32, texto: Color32) -> Self {
        self.fondo = Some(fondo);
        self.color_texto = Some(texto);
        self
    }

    pub fn color_borde(mut self, borde: Color32) -> Self {
        self.color_borde = Some(borde);
        self
    }

    pub fn degradado(mut self, desde: Color32, hasta: Color32, direccion: Direccion) -> Self {
        self.degradado = Some((desde, hasta, direccion));
        self
    }

    fn medidas(&self) -> (f32, f32) {
        match self.tam {
            TamBoton::Normal => (40.0, 16.0),
            TamBoton::Chico => (36.0, 12.0),
            TamBoton::Grande => (44.0, 32.0),
            TamBoton::Icono => (40.0, 0.0),
        }
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let (alto_base, relleno) = self.medidas();
        let alto = self.alto.unwrap_or(alto_base);
        let texto_color = self.color_texto.unwrap_or(match self.variante {
            Variante::Primario => color::PRIMARIO_TEXTO,
            Variante::Destructivo | Variante::Esmeralda | Variante::Cancelar => Color32::WHITE,
            Variante::Contorno | Variante::Fantasma => color::TEXTO,
            Variante::Secundario => color::SECUNDARIO_TEXTO,
            Variante::Enlace => color::PRIMARIO,
        });
        let galeria = (!self.texto.is_empty()).then(|| {
            ui.painter().layout_no_wrap(self.texto.clone(), fuente(self.tam_texto, self.peso), texto_color)
        });
        let iconos = [self.icono.or(self.cargando.then_some("loader-2")), self.icono_derecha];
        let cantidad_iconos = iconos.iter().flatten().count();
        let ancho_texto = galeria.as_ref().map_or(0.0, |g| g.size().x);
        let piezas = cantidad_iconos + usize::from(galeria.is_some());
        let contenido =
            ancho_texto + cantidad_iconos as f32 * self.tam_icono + piezas.saturating_sub(1) as f32 * 8.0;
        let ancho = self.ancho.unwrap_or(if self.tam == TamBoton::Icono { alto } else { contenido + relleno * 2.0 });
        let sentido = if self.habilitado && !self.cargando { Sense::click() } else { Sense::hover() };
        let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(ancho, alto), sentido);
        if !ui.is_rect_visible(rect) {
            return respuesta;
        }
        let encima = respuesta.hovered() && self.habilitado && !self.cargando;
        let pintor = ui.painter().with_clip_rect(ui.clip_rect());
        let opacidad = if self.habilitado && !self.cargando { 1.0 } else { 0.5 };
        let fondo = |c: Color32| c.gamma_multiply(opacidad);
        match (self.degradado, self.variante) {
            (Some((desde, hasta, direccion)), _) => {
                pintar_degradado(&pintor, rect, self.radio, fondo(desde), fondo(hasta), direccion);
            }
            (None, Variante::Esmeralda) => pintar_degradado(
                &pintor,
                rect,
                self.radio,
                fondo(color::ESMERALDA_DESDE),
                fondo(color::ESMERALDA_HASTA),
                Direccion::AbajoDerecha,
            ),
            (None, Variante::Cancelar) => {
                let (desde, hasta) = if encima {
                    (paleta::RED_600, paleta::RED_500)
                } else {
                    (paleta::RED_700, paleta::RED_800)
                };
                pintar_degradado(&pintor, rect, self.radio, fondo(desde), fondo(hasta), Direccion::Derecha);
            }
            (None, variante) => {
                let base = self.fondo.unwrap_or(match variante {
                    Variante::Primario => color::PRIMARIO,
                    Variante::Destructivo => color::DESTRUCTIVO,
                    Variante::Contorno => color::FONDO,
                    Variante::Secundario => color::SECUNDARIO,
                    _ => Color32::TRANSPARENT,
                });
                let relleno_final = match (variante, encima) {
                    (_, false) => base,
                    (Variante::Primario | Variante::Destructivo, true) if self.fondo.is_none() => {
                        base.gamma_multiply(0.9)
                    }
                    (Variante::Secundario, true) if self.fondo.is_none() => base.gamma_multiply(0.8),
                    (Variante::Contorno | Variante::Fantasma, true) if self.fondo.is_none() => color::ACENTO,
                    (_, true) => base.gamma_multiply(0.9),
                };
                let trazo = match (variante, self.color_borde) {
                    (_, Some(c)) => borde(c),
                    (Variante::Contorno, None) => borde(color::BORDE),
                    _ => Stroke::NONE,
                };
                pintor.rect(rect, self.radio, fondo(relleno_final), trazo, StrokeKind::Inside);
            }
        }
        let color_contenido = texto_color.gamma_multiply(opacidad);
        let mut x = rect.center().x - contenido / 2.0;
        if let Some(nombre) = iconos[0] {
            let caja = Rect::from_center_size(
                egui::pos2(x + self.tam_icono / 2.0, rect.center().y),
                Vec2::splat(self.tam_icono),
            );
            if self.cargando {
                girando(ui, caja, color_contenido);
            } else {
                super::pintar_icono_en(ui, caja, nombre, color_contenido);
            }
            x += self.tam_icono + 8.0;
        }
        if let Some(galeria) = galeria {
            let pos = egui::pos2(x, rect.center().y - galeria.size().y / 2.0);
            if self.variante == Variante::Enlace && encima {
                let y = pos.y + galeria.size().y;
                pintor.line_segment([egui::pos2(x, y), egui::pos2(x + ancho_texto, y)], Stroke::new(1.0, color_contenido));
            }
            pintor.galley(pos, galeria, color_contenido);
            x += ancho_texto + 8.0;
        }
        if let Some(nombre) = iconos[1] {
            let caja = Rect::from_center_size(
                egui::pos2(x + self.tam_icono / 2.0, rect.center().y),
                Vec2::splat(self.tam_icono),
            );
            super::pintar_icono_en(ui, caja, nombre, color_contenido);
        }
        if self.habilitado && !self.cargando {
            respuesta.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            respuesta
        }
    }
}

pub fn girando(ui: &Ui, caja: Rect, color: Color32) {
    let angulo = ui.input(|i| i.time) as f32 * std::f32::consts::TAU;
    super::icono("loader-2", caja.width(), color).rotate(angulo, Vec2::splat(0.5)).paint_at(ui, caja);
    ui.ctx().request_repaint();
}

pub fn cargando(ui: &mut Ui, tam: f32, color: Color32) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(tam), Sense::hover());
    girando(ui, rect, color);
    respuesta
}

pub struct Entrada<'a> {
    texto: &'a mut String,
    sugerencia: String,
    ancho: Option<f32>,
    alto: f32,
    contrasena: bool,
    tam: Tam,
    id: Option<Id>,
    radio: u8,
    color_borde: Color32,
    icono: Option<&'a str>,
    habilitado: bool,
    alineacion: Align,
    multilinea: bool,
    peso: Peso,
}

impl<'a> Entrada<'a> {
    pub fn new(texto: &'a mut String) -> Self {
        Entrada {
            texto,
            sugerencia: String::new(),
            ancho: None,
            alto: 40.0,
            contrasena: false,
            tam: Tam::Base,
            id: None,
            radio: RADIO_XL,
            color_borde: color::BORDE_ENTRADA,
            icono: None,
            habilitado: true,
            alineacion: Align::Min,
            multilinea: false,
            peso: Peso::Normal,
        }
    }

    pub fn sugerencia(mut self, sugerencia: impl Into<String>) -> Self {
        self.sugerencia = sugerencia.into();
        self
    }

    pub fn ancho(mut self, ancho: f32) -> Self {
        self.ancho = Some(ancho);
        self
    }

    pub fn alto(mut self, alto: f32) -> Self {
        self.alto = alto;
        self
    }

    pub fn contrasena(mut self, contrasena: bool) -> Self {
        self.contrasena = contrasena;
        self
    }

    pub fn tam(mut self, tam: Tam) -> Self {
        self.tam = tam;
        self
    }

    pub fn peso(mut self, peso: Peso) -> Self {
        self.peso = peso;
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn radio(mut self, radio: u8) -> Self {
        self.radio = radio;
        self
    }

    pub fn color_borde(mut self, color_borde: Color32) -> Self {
        self.color_borde = color_borde;
        self
    }

    pub fn icono(mut self, icono: &'a str) -> Self {
        self.icono = Some(icono);
        self
    }

    pub fn habilitado(mut self, habilitado: bool) -> Self {
        self.habilitado = habilitado;
        self
    }

    pub fn centrado(mut self) -> Self {
        self.alineacion = Align::Center;
        self
    }

    pub fn derecha(mut self) -> Self {
        self.alineacion = Align::Max;
        self
    }

    pub fn multilinea(mut self, alto_minimo: f32) -> Self {
        self.multilinea = true;
        self.alto = alto_minimo;
        self.radio = RADIO_MD;
        self.color_borde = color::BORDE;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> Response {
        let ancho = self.ancho.unwrap_or_else(|| ui.available_width());
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ancho, self.alto), Sense::hover());
        let opacidad = if self.habilitado { 1.0 } else { 0.5 };
        ui.painter().rect(
            rect,
            self.radio,
            color::FONDO.gamma_multiply(opacidad),
            borde(self.color_borde.gamma_multiply(opacidad)),
            StrokeKind::Inside,
        );
        let mut interior = rect.shrink2(Vec2::new(12.0, if self.multilinea { 8.0 } else { 0.0 }));
        if let Some(nombre) = self.icono {
            let caja = Rect::from_center_size(egui::pos2(interior.left() + 8.0, rect.center().y), Vec2::splat(16.0));
            super::pintar_icono_en(ui, caja, nombre, color::APAGADO_TEXTO);
            interior.min.x += 24.0;
        }
        let sugerencia = WidgetText::from(
            egui::RichText::new(self.sugerencia).font(fuente(self.tam, self.peso)).color(color::APAGADO_TEXTO),
        );
        let editor = if self.multilinea { TextEdit::multiline(self.texto) } else { TextEdit::singleline(self.texto) };
        let mut editor = editor
            .frame(egui::Frame::NONE)
            .hint_text(sugerencia)
            .password(self.contrasena)
            .font(fuente(self.tam, self.peso))
            .text_color(color::TEXTO)
            .horizontal_align(self.alineacion)
            .vertical_align(if self.multilinea { Align::Min } else { Align::Center })
            .margin(egui::Margin::ZERO)
            .desired_width(interior.width())
            .interactive(self.habilitado);
        if let Some(id) = self.id {
            editor = editor.id(id);
        }
        ui.put(interior, editor)
    }
}

pub fn etiqueta(ui: &mut Ui, contenido: &str) -> Response {
    ui.label(super::texto(contenido, Tam::Sm, Peso::Medio, color::TEXTO))
}

pub fn separador(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::BORDE);
}

pub fn separador_vertical(ui: &mut Ui, alto: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, alto), Sense::hover());
    ui.painter().rect_filled(rect, 0, color::BORDE);
}

pub fn casilla(ui: &mut Ui, marcada: &mut bool, texto: &str) -> Response {
    let galeria = (!texto.is_empty())
        .then(|| ui.painter().layout_no_wrap(texto.to_string(), fuente(Tam::Sm, Peso::Medio), color::TEXTO));
    let ancho = 16.0 + galeria.as_ref().map_or(0.0, |g| 8.0 + g.size().x);
    let alto = galeria.as_ref().map_or(16.0, |g| g.size().y.max(16.0));
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(ancho, alto), Sense::click());
    if respuesta.clicked() {
        *marcada = !*marcada;
        respuesta.mark_changed();
    }
    let caja = Rect::from_min_size(egui::pos2(rect.left(), rect.center().y - 8.0), Vec2::splat(16.0));
    let relleno = if *marcada { color::PRIMARIO } else { color::FONDO };
    ui.painter().rect(caja, RADIO_SM, relleno, borde(color::PRIMARIO), StrokeKind::Inside);
    if *marcada {
        super::pintar_icono_en(ui, caja.shrink(1.0), "check", color::PRIMARIO_TEXTO);
    }
    if let Some(galeria) = galeria {
        ui.painter().galley(egui::pos2(caja.right() + 8.0, rect.center().y - galeria.size().y / 2.0), galeria, color::TEXTO);
    }
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn interruptor(ui: &mut Ui, activo: &mut bool) -> Response {
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(44.0, 24.0), Sense::click());
    if respuesta.clicked() {
        *activo = !*activo;
        respuesta.mark_changed();
    }
    let avance = ui.ctx().animate_bool_with_time(respuesta.id, *activo, 0.15);
    let fondo = super::degradado::mezclar(color::BORDE, color::PRIMARIO, avance);
    ui.painter().rect_filled(rect, 12, fondo);
    let x = rect.left() + 12.0 + avance * 20.0;
    let perilla = Rect::from_center_size(egui::pos2(x, rect.center().y), Vec2::splat(20.0));
    ui.painter().add(super::sombra_lg().as_shape(perilla, 10));
    ui.painter().rect_filled(perilla, 10, color::FONDO);
    respuesta.on_hover_cursor(CursorIcon::PointingHand)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VarianteInsignia {
    Primaria,
    Secundaria,
    Destructiva,
    Contorno,
}

pub fn insignia(ui: &mut Ui, contenido: &str, variante: VarianteInsignia) -> Response {
    let (fondo, texto_color, trazo) = match variante {
        VarianteInsignia::Primaria => (color::PRIMARIO, color::PRIMARIO_TEXTO, Color32::TRANSPARENT),
        VarianteInsignia::Secundaria => (color::SECUNDARIO, color::SECUNDARIO_TEXTO, Color32::TRANSPARENT),
        VarianteInsignia::Destructiva => (color::DESTRUCTIVO, color::DESTRUCTIVO_TEXTO, Color32::TRANSPARENT),
        VarianteInsignia::Contorno => (Color32::TRANSPARENT, color::TEXTO, color::BORDE),
    };
    insignia_colores(ui, contenido, fondo, texto_color, trazo)
}

pub fn insignia_colores(ui: &mut Ui, contenido: &str, fondo: Color32, texto_color: Color32, trazo: Color32) -> Response {
    let galeria = ui.painter().layout_no_wrap(contenido.to_string(), fuente(Tam::Xs, Peso::Semi), texto_color);
    let tam = galeria.size() + Vec2::new(20.0, 4.0);
    let (rect, respuesta) = ui.allocate_exact_size(tam, Sense::hover());
    ui.painter().rect(rect, rect.height() / 2.0, fondo, borde(trazo), StrokeKind::Inside);
    ui.painter().galley(rect.center() - galeria.size() / 2.0, galeria, texto_color);
    respuesta
}

pub fn pestanas(ui: &mut Ui, opciones: &[&str], activa: &mut usize, ancho: Option<f32>) -> Response {
    let fuente_pestana = fuente(Tam::Sm, Peso::Medio);
    let galerias: Vec<_> = opciones
        .iter()
        .map(|o| ui.painter().layout_no_wrap(o.to_string(), fuente_pestana.clone(), color::TEXTO))
        .collect();
    let natural: f32 = galerias.iter().map(|g| g.size().x + 24.0).sum::<f32>() + 8.0;
    let ancho_total = ancho.unwrap_or(natural);
    let (rect, mut respuesta) = ui.allocate_exact_size(Vec2::new(ancho_total, 40.0), Sense::hover());
    ui.painter().rect_filled(rect, RADIO_MD, color::APAGADO);
    let interior = rect.shrink(4.0);
    let mut x = interior.left();
    for (indice, galeria) in galerias.into_iter().enumerate() {
        let ancho_pestana = if ancho.is_some() {
            interior.width() / opciones.len() as f32
        } else {
            galeria.size().x + 24.0
        };
        let caja = Rect::from_min_size(egui::pos2(x, interior.top()), Vec2::new(ancho_pestana, interior.height()));
        let r = ui.interact(caja, respuesta.id.with(indice), Sense::click());
        if r.clicked() && *activa != indice {
            *activa = indice;
            respuesta.mark_changed();
        }
        let seleccionada = *activa == indice;
        if seleccionada {
            ui.painter().add(super::sombra_sm().as_shape(caja, RADIO_SM));
            ui.painter().rect_filled(caja, RADIO_SM, color::FONDO);
        }
        let texto_color = if seleccionada { color::TEXTO } else { color::APAGADO_TEXTO };
        ui.painter().galley(caja.center() - galeria.size() / 2.0, galeria, texto_color);
        x += ancho_pestana;
    }
    respuesta
}

pub fn texto_en(ui: &Ui, pos: egui::Pos2, ancla: Align2, contenido: &str, tam: Tam, peso: Peso, color: Color32) -> Rect {
    ui.painter().text(pos, ancla, contenido, fuente(tam, peso), color)
}
