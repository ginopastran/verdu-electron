use std::path::PathBuf;
use std::sync::Arc;

use actualizador::Publicacion;
use eframe::egui::{self, Align2, Color32, Context, Galley, Id, Order, Pos2, Rect, Sense, Ui, UiBuilder, Vec2};

use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::{self, color, paleta, toast, Boton, Peso, Tam, Variante};

const PRIMERA_BUSQUEDA: f64 = 5.0;
const INTERVALO_BUSQUEDA: f64 = 10.0 * 60.0;
const ESPERA_REINICIO: f64 = 1.0;
const DURACION_AVISO: f64 = 2.0;
const ICONO_LISTO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.801 10A10 10 0 1 1 17 3.335"/><path d="m9 11 3 3L22 4"/></svg>"##;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Estado {
    #[default]
    Inactivo,
    Disponible,
    Descargando,
    Descargado(PathBuf),
}

struct AvisoConAccion {
    titulo: String,
    detalle: String,
    accion: Accion,
    desde: f64,
}

#[derive(Default)]
pub struct Actualizacion {
    pub estado: Estado,
    publicacion: Option<Publicacion>,
    proxima_busqueda: Option<f64>,
    busquedas: u32,
    buscando: Option<Tarea<Result<Option<Publicacion>, String>>>,
    descargando: Option<Tarea<Result<PathBuf, String>>>,
    instalando: Option<Tarea<Result<(), String>>>,
    reiniciar_en: Option<f64>,
    aviso: Option<AvisoConAccion>,
    icono_registrado: bool,
}

pub fn mensaje_error_busqueda(error: &str) -> &'static str {
    if error.contains("Timeout") {
        "Tiempo de espera agotado verificando actualizaciones"
    } else {
        "Error al verificar actualizaciones"
    }
}

fn galeria(ctx: &Context, contenido: &str, tam: f32, peso: Peso, alto_linea: f32, espaciado: f32, color: Color32, ancho: f32) -> Arc<Galley> {
    let mut trabajo = super::trabajo(contenido, Tam::Px(tam), peso, color, alto_linea, espaciado);
    trabajo.wrap.max_width = ancho;
    ctx.fonts_mut(|fuentes| fuentes.layout_job(trabajo))
}

impl Actualizacion {
    pub fn ofrecer(&mut self, ctx: &Context, publicacion: Publicacion) {
        let ahora = ctx.input(|i| i.time);
        self.aviso = Some(AvisoConAccion {
            titulo: "¡Nueva versión disponible!".into(),
            detalle: format!("Versión {} está lista para descargar", publicacion.version),
            accion: Accion::Descargar,
            desde: ahora,
        });
        self.publicacion = Some(publicacion);
        self.estado = Estado::Disponible;
        ctx.request_repaint();
    }

    pub fn al_descargar(&mut self, ctx: &Context, resultado: Result<PathBuf, String>) {
        match resultado {
            Ok(ruta) => {
                self.estado = Estado::Descargado(ruta);
                self.aviso = Some(AvisoConAccion {
                    titulo: "Actualización descargada".into(),
                    detalle: "La actualización se instalará al reiniciar la aplicación".into(),
                    accion: Accion::Instalar,
                    desde: ctx.input(|i| i.time),
                });
            }
            Err(error) => {
                self.estado = Estado::Disponible;
                toast::error(ctx, format!("Error al descargar la actualización: {error}"));
            }
        }
        ctx.request_repaint();
    }

    fn descargar(&mut self, ctx: &Context) {
        let (Estado::Disponible, Some(publicacion)) = (&self.estado, self.publicacion.clone()) else {
            return;
        };
        self.estado = Estado::Descargando;
        self.descargando = Some(Tarea::lanzar(ctx, move || actualizador::descargar(&publicacion)));
    }

    fn instalar(&mut self, ctx: &Context) {
        if !matches!(self.estado, Estado::Descargado(_)) || self.reiniciar_en.is_some() || self.instalando.is_some() {
            return;
        }
        toast::con_detalle(ctx, toast::Tipo::Info, "Reiniciando aplicación...", "La aplicación se cerrará y se aplicará la actualización", DURACION_AVISO);
        self.reiniciar_en = Some(ctx.input(|i| i.time) + ESPERA_REINICIO);
    }

    pub fn reiniciando(&self) -> bool {
        self.reiniciar_en.is_some() || self.instalando.is_some()
    }

    fn cancelar(&mut self, ctx: &Context) {
        self.estado = Estado::Inactivo;
        self.publicacion = None;
        toast::info(ctx, "Actualización cancelada");
    }

    fn programar(&mut self, ctx: &Context, ahora: f64) {
        if cfg!(debug_assertions) {
            return;
        }
        let proxima = *self.proxima_busqueda.get_or_insert(ahora + PRIMERA_BUSQUEDA);
        if ahora < proxima {
            ctx.request_repaint_after_secs((proxima - ahora) as f32);
            return;
        }
        self.proxima_busqueda = Some(ahora + INTERVALO_BUSQUEDA);
        ctx.request_repaint_after_secs(INTERVALO_BUSQUEDA as f32);
        if self.estado == Estado::Inactivo && self.buscando.is_none() {
            self.buscando = Some(Tarea::lanzar(ctx, || actualizador::buscar(env!("CARGO_PKG_VERSION"))));
        }
    }

    fn recibir(&mut self, ctx: &Context, ahora: f64) {
        if let Some(resultado) = esperar(&mut self.buscando) {
            let primera = self.busquedas == 0;
            self.busquedas += 1;
            match resultado {
                Ok(Some(publicacion)) if self.estado == Estado::Inactivo => self.ofrecer(ctx, publicacion),
                Ok(_) => {}
                Err(error) if primera => toast::error(ctx, mensaje_error_busqueda(&error)),
                Err(_) => {}
            }
        }
        if let Some(resultado) = esperar(&mut self.descargando) {
            self.al_descargar(ctx, resultado);
        }
        if let Some(momento) = self.reiniciar_en {
            if ahora >= momento {
                self.reiniciar_en = None;
                if let (Estado::Descargado(ruta), Some(publicacion)) = (&self.estado, &self.publicacion) {
                    let (ruta, firma) = (ruta.clone(), publicacion.firma.clone());
                    self.instalando = Some(Tarea::lanzar(ctx, move || actualizador::instalar(&ruta, &firma)));
                }
            } else {
                ctx.request_repaint_after_secs((momento - ahora) as f32);
            }
        }
        if let Some(resultado) = esperar(&mut self.instalando) {
            match resultado {
                Ok(()) => std::process::exit(0),
                Err(error) => toast::error(ctx, format!("No se pudo instalar la actualización: {error}")),
            }
        }
    }

    fn tarjeta(&mut self, ctx: &Context) {
        if self.estado == Estado::Inactivo {
            return;
        }
        let (icono, tam_icono, titulo, descripcion) = match self.estado {
            Estado::Descargado(_) => ("circle-check-big", 20.0, "Listo para instalar", "La actualización se instalará al reiniciar"),
            Estado::Descargando => ("loader-2", 20.0, "Descargando...", "Descargando la nueva versión..."),
            _ => ("download", 16.0, "Actualización disponible", "Una nueva versión está disponible"),
        };
        let version = self.publicacion.as_ref().map(|p| format!("v{}", p.version));
        let insignia = version
            .as_ref()
            .map(|v| galeria(ctx, v, 12.0, Peso::Semi, 16.0, 0.0, paleta::EMERALD_700, f32::INFINITY));
        let ancho_insignia = insignia.as_ref().map_or(0.0, |g| g.size().x + 22.0);
        let interior = 320.0 - 2.0 - 48.0;
        let ancho_titulo = interior - ancho_insignia - tam_icono - 8.0;
        let galeria_titulo = galeria(ctx, titulo, 24.0, Peso::Semi, 24.0, -0.6, paleta::EMERALD_800, ancho_titulo);
        let alto_titulo = galeria_titulo.size().y;
        let alto = 2.0 + 24.0 + alto_titulo + 6.0 + 20.0 + 12.0 + 40.0 + 24.0;
        let pantalla = ctx.content_rect();
        let caja = Rect::from_min_size(Pos2::new(pantalla.right() - 16.0 - 320.0, pantalla.bottom() - 16.0 - alto), Vec2::new(320.0, alto));
        let mut accion = None;
        egui::Area::new(Id::new("tarjeta-actualizacion"))
            .order(Order::Foreground)
            .fixed_pos(caja.min)
            .show(ctx, |ui| {
                ui.allocate_rect(caja, Sense::hover());
                let pintor = ui.painter();
                pintor.add(ui::sombra_lg().as_shape(caja, ui::RADIO));
                pintor.rect(caja, ui::RADIO, paleta::EMERALD_50.gamma_multiply(0.95), ui::borde(paleta::EMERALD_200), egui::StrokeKind::Inside);
                let izquierda = caja.left() + 25.0;
                let arriba = caja.top() + 25.0;
                let centro_titulo = arriba + alto_titulo / 2.0;
                let caja_icono = Rect::from_center_size(Pos2::new(izquierda + tam_icono / 2.0, centro_titulo), Vec2::splat(tam_icono));
                if self.estado == Estado::Descargando {
                    ui::girando(ui, caja_icono, paleta::EMERALD_800);
                } else {
                    ui::pintar_icono_en(ui, caja_icono, icono, paleta::EMERALD_800);
                }
                let ajuste = super::media_interlinea(ctx, Tam::X2, Peso::Semi, 24.0);
                ui.painter().galley(Pos2::new(izquierda + tam_icono + 8.0, arriba + ajuste), galeria_titulo, paleta::EMERALD_800);
                if let Some(insignia) = insignia {
                    let rect = Rect::from_min_size(
                        Pos2::new(caja.right() - 25.0 - ancho_insignia, centro_titulo - 11.0),
                        Vec2::new(ancho_insignia, 22.0),
                    );
                    ui.painter().rect_filled(rect, 11.0, paleta::EMERALD_100);
                    ui.painter().galley(rect.center() - insignia.size() / 2.0, insignia, paleta::EMERALD_700);
                }
                let y_descripcion = arriba + alto_titulo + 6.0;
                ui.painter().text(
                    Pos2::new(izquierda, y_descripcion + 10.0),
                    Align2::LEFT_CENTER,
                    descripcion,
                    ui::fuente(Tam::Sm, Peso::Normal),
                    paleta::EMERALD_600,
                );
                let fila = Rect::from_min_size(Pos2::new(izquierda, y_descripcion + 20.0 + 12.0), Vec2::new(interior, 40.0));
                accion = self.botones(ui, fila);
            });
        match accion {
            Some(Accion::Descargar) => self.descargar(ctx),
            Some(Accion::MasTarde) => self.cancelar(ctx),
            Some(Accion::Instalar) => self.instalar(ctx),
            None => {}
        }
    }

    fn botones(&self, ui: &mut Ui, fila: Rect) -> Option<Accion> {
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(fila).layout(egui::Layout::left_to_right(egui::Align::Center)));
        let ui = &mut hijo;
        match self.estado {
            Estado::Descargado(_) => Boton::new("Reiniciar e instalar")
                .icono("refresh-cw")
                .colores(paleta::EMERALD_600, Color32::WHITE)
                .ancho(fila.width())
                .ui(ui)
                .clicked()
                .then_some(Accion::Instalar),
            Estado::Descargando => {
                let galeria_texto = ui.painter().layout_no_wrap("Descargando...".into(), ui::fuente(Tam::Base, Peso::Normal), paleta::EMERALD_600);
                let ancho = 16.0 + 8.0 + galeria_texto.size().x;
                let inicio = fila.center().x - ancho / 2.0;
                ui::girando(ui, Rect::from_center_size(Pos2::new(inicio + 8.0, fila.center().y), Vec2::splat(16.0)), paleta::EMERALD_600);
                ui.painter().galley(Pos2::new(inicio + 24.0, fila.center().y - galeria_texto.size().y / 2.0), galeria_texto, paleta::EMERALD_600);
                None
            }
            _ => {
                ui.spacing_mut().item_spacing.x = 8.0;
                let mas_tarde = Boton::new("Más tarde")
                    .variante(Variante::Contorno)
                    .colores(color::FONDO, paleta::EMERALD_600)
                    .color_borde(paleta::EMERALD_300)
                    .ui(ui)
                    .clicked();
                let restante = ui.available_width();
                let descargar = Boton::new("Descargar")
                    .icono("download")
                    .colores(paleta::EMERALD_600, Color32::WHITE)
                    .ancho(restante)
                    .ui(ui)
                    .clicked();
                if mas_tarde {
                    Some(Accion::MasTarde)
                } else {
                    descargar.then_some(Accion::Descargar)
                }
            }
        }
    }

    fn pintar_aviso(&mut self, ctx: &Context, ahora: f64) {
        let Some(aviso) = &self.aviso else {
            return;
        };
        let transcurrido = ahora - aviso.desde;
        if transcurrido > DURACION_AVISO + 0.2 {
            self.aviso = None;
            return;
        }
        ctx.request_repaint();
        let opacidad = if transcurrido > DURACION_AVISO {
            (1.0 - (transcurrido - DURACION_AVISO) / 0.2) as f32
        } else {
            (transcurrido / 0.15).min(1.0) as f32
        };
        let (fondo, texto_color, trazo) = (
            Color32::from_rgb(0xec, 0xfd, 0xf3),
            Color32::from_rgb(0x00, 0x8a, 0x2e),
            Color32::from_rgb(0xbf, 0xfc, 0xd9),
        );
        let ancho = 356.0;
        let pantalla = ctx.content_rect();
        let izquierda = pantalla.center().x - ancho / 2.0;
        let galeria_boton = galeria(ctx, aviso.accion.texto_aviso(), 12.0, Peso::Medio, 16.0, 0.0, Color32::WHITE, f32::INFINITY);
        let ancho_boton = galeria_boton.size().x + 16.0;
        let x_boton = izquierda + ancho - 17.0 - ancho_boton;
        let ancho_texto = x_boton - 6.0 - (izquierda + 40.0);
        let titulo = galeria(ctx, &aviso.titulo, 13.0, Peso::Medio, 20.0, 0.0, texto_color, ancho_texto);
        let detalle = galeria(ctx, &aviso.detalle, 13.0, Peso::Normal, 18.5, 0.0, texto_color, ancho_texto);
        let alto = 34.0 + titulo.size().y + detalle.size().y;
        let caja = Rect::from_min_size(Pos2::new(izquierda, pantalla.top() + 32.0), Vec2::new(ancho, alto));
        let accion = aviso.accion;
        let mut cerrar = false;
        egui::Area::new(Id::new("aviso-actualizacion"))
            .order(Order::Tooltip)
            .fixed_pos(caja.min)
            .show(ctx, |ui| {
                ui.allocate_rect(caja, Sense::hover());
                let pintor = ui.painter();
                pintor.add(ui::sombra_md().as_shape(caja, 8));
                pintor.rect(caja, 8, fondo.gamma_multiply(opacidad), ui::borde(trazo.gamma_multiply(opacidad)), egui::StrokeKind::Inside);
                let caja_icono = Rect::from_min_size(Pos2::new(izquierda + 14.0, caja.center().y - 8.0), Vec2::splat(16.0));
                ui::pintar_icono_en(ui, caja_icono, "check-circle", texto_color.gamma_multiply(opacidad));
                let y_texto = caja.center().y - (titulo.size().y + detalle.size().y) / 2.0;
                let alto_titulo = titulo.size().y;
                let ajuste_titulo = super::media_interlinea(ctx, Tam::Px(13.0), Peso::Medio, 20.0);
                let ajuste_detalle = super::media_interlinea(ctx, Tam::Px(13.0), Peso::Normal, 18.5);
                pintor.galley(Pos2::new(izquierda + 40.0, y_texto + ajuste_titulo), titulo, texto_color.gamma_multiply(opacidad));
                pintor.galley(Pos2::new(izquierda + 40.0, y_texto + alto_titulo + ajuste_detalle), detalle, texto_color.gamma_multiply(opacidad));
                let boton = Rect::from_min_size(Pos2::new(x_boton, caja.center().y - 12.0), Vec2::new(ancho_boton, 24.0));
                let respuesta = ui.interact(boton, Id::new("aviso-actualizacion-accion"), Sense::click());
                pintor.rect_filled(boton, ui::RADIO_SM, color::PRIMARIO.gamma_multiply(opacidad));
                pintor.galley(boton.center() - galeria_boton.size() / 2.0, galeria_boton, Color32::WHITE.gamma_multiply(opacidad));
                cerrar = respuesta.on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
            });
        if cerrar {
            self.aviso = None;
            match accion {
                Accion::Descargar => self.descargar(ctx),
                Accion::Instalar => self.instalar(ctx),
                Accion::MasTarde => {}
            }
        }
    }

    pub fn mostrar(&mut self, _ui: &mut Ui, cx: &mut Contexto) {
        let ctx = cx.egui.clone();
        if !self.icono_registrado {
            ctx.include_bytes("bytes://icono/circle-check-big.svg", ICONO_LISTO.as_bytes());
            self.icono_registrado = true;
        }
        let ahora = ctx.input(|i| i.time);
        self.programar(&ctx, ahora);
        self.recibir(&ctx, ahora);
        self.tarjeta(&ctx);
        self.pintar_aviso(&ctx, ahora);
    }
}

#[derive(Clone, Copy)]
enum Accion {
    Descargar,
    MasTarde,
    Instalar,
}

impl Accion {
    fn texto_aviso(self) -> &'static str {
        match self {
            Accion::Instalar => "Reiniciar ahora",
            _ => "Descargar",
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn distingue_el_tiempo_agotado_de_otros_errores() {
        assert_eq!(mensaje_error_busqueda("Timeout verificando actualizaciones"), "Tiempo de espera agotado verificando actualizaciones");
        assert_eq!(mensaje_error_busqueda("No se pudo leer latest.json (HTTP 404)"), "Error al verificar actualizaciones");
    }
}
