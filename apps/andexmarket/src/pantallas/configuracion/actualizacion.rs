use std::path::PathBuf;

use actualizador::Publicacion;
use eframe::egui::{self, Align, Align2, Context, Id, Layout, Order, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2};

use crate::app::Contexto;
use crate::tareas::{esperar, Tarea};
use crate::ui::marca::{alto, color, fuente, radio, sombra, tam};
use crate::ui::v2::animacion::{salida_suave, DURACION_TOAST};
use crate::ui::v2::{self, chip as chip_v2, toast, Boton, Tono};
use crate::ui::{pintar_icono_solar_en, Peso};

const PRIMERA_BUSQUEDA: f64 = 5.0;
const INTERVALO_BUSQUEDA: f64 = 10.0 * 60.0;
const ESPERA_REINICIO: f64 = 1.0;
const DURACION_AVISO: f64 = 2.0;
const ANCHO_TARJETA: f32 = 340.0;

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
}

pub fn mensaje_error_busqueda(error: &str) -> &'static str {
    if error.contains("Timeout") {
        "Tiempo de espera agotado verificando actualizaciones"
    } else {
        "Error al verificar actualizaciones"
    }
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
        toast::mostrar_aviso(ctx, toast::Tipo::Info, "Reiniciando aplicación...", Some("La aplicación se cerrará y se aplicará la actualización".into()), DURACION_AVISO);
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
        let (icono, titulo, descripcion) = match self.estado {
            Estado::Descargado(_) => ("circle-check", "Listo para instalar", "La actualización se instalará al reiniciar"),
            Estado::Descargando => ("loader", "Descargando...", "Descargando la nueva versión..."),
            _ => ("download", "Actualización disponible", "Una nueva versión está disponible"),
        };
        let version = self.publicacion.as_ref().map(|p| format!("v{}", p.version));
        let pantalla = ctx.content_rect();
        let alto = 20.0 + 36.0 + 12.0 + 20.0 + 16.0 + alto::BOTON_MODAL + 20.0;
        let caja = Rect::from_min_size(Pos2::new(pantalla.right() - 20.0 - ANCHO_TARJETA, pantalla.bottom() - 20.0 - alto), Vec2::new(ANCHO_TARJETA, alto));
        let mut accion = None;
        egui::Area::new(Id::new("tarjeta-actualizacion"))
            .order(Order::Foreground)
            .fixed_pos(caja.min)
            .show(ctx, |ui| {
                ui.allocate_rect(caja, Sense::hover());
                let pintor = ui.painter();
                pintor.add(sombra::POPOVER.forma(caja, radio::TARJETA));
                pintor.rect(caja, radio::TARJETA, color::BLANCO, Stroke::new(1.0, color::BORDE), StrokeKind::Inside);
                let interior = caja.shrink(20.0);
                let chip = Rect::from_min_size(interior.min, Vec2::splat(36.0));
                pintor.rect_filled(chip, radio::CONTROL, color::VERDE_CHIP);
                let caja_icono = Rect::from_center_size(chip.center(), Vec2::splat(20.0));
                if self.estado == Estado::Descargando {
                    v2::girando(ui, caja_icono, color::VERDE);
                } else {
                    pintar_icono_solar_en(ui, caja_icono, icono, color::VERDE);
                }
                let mut derecha = interior.right();
                if let Some(version) = &version {
                    let mut insignia = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(Pos2::new(chip.right(), chip.top()), Pos2::new(interior.right(), chip.bottom()))).layout(Layout::right_to_left(Align::Center)));
                    derecha = chip_v2(&mut insignia, version, Tono::Marca).rect.left() - 8.0;
                }
                let g = v2::galeria_espaciada(ui, titulo, fuente(tam::DESTACADO, Peso::Semi), color::TEXTO, -0.01 * tam::DESTACADO);
                ui.painter().with_clip_rect(Rect::from_min_max(caja.min, Pos2::new(derecha, caja.bottom()))).galley(Pos2::new(chip.right() + 12.0, chip.center().y - g.size().y / 2.0), g, color::TEXTO);
                let y_descripcion = chip.bottom() + 12.0;
                v2::pintar_texto(ui, Pos2::new(interior.left(), y_descripcion + 10.0), Align2::LEFT_CENTER, descripcion, tam::ETIQUETA, Peso::Normal, color::TEXTO_APAGADO);
                let fila = Rect::from_min_size(Pos2::new(interior.left(), y_descripcion + 20.0 + 16.0), Vec2::new(interior.width(), alto::BOTON_MODAL));
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
        let mut hijo = ui.new_child(UiBuilder::new().max_rect(fila).layout(Layout::left_to_right(Align::Center)));
        let ui = &mut hijo;
        match self.estado {
            Estado::Descargado(_) => Boton::primario("Reiniciar e instalar")
                .icono("loader")
                .modal()
                .ancho(fila.width())
                .ui(ui)
                .clicked()
                .then_some(Accion::Instalar),
            Estado::Descargando => {
                let g = v2::galeria(ui, "Descargando...", fuente(tam::CUERPO, Peso::Medio), color::VERDE);
                let inicio = fila.center().x - (16.0 + 8.0 + g.size().x) / 2.0;
                v2::girando(ui, Rect::from_center_size(Pos2::new(inicio + 8.0, fila.center().y), Vec2::splat(16.0)), color::VERDE);
                ui.painter().galley(Pos2::new(inicio + 24.0, fila.center().y - g.size().y / 2.0), g, color::VERDE);
                None
            }
            _ => {
                ui.spacing_mut().item_spacing.x = 10.0;
                let mas_tarde = Boton::secundario("Más tarde").modal().ui(ui).clicked();
                let restante = ui.available_width();
                let descargar = Boton::primario("Descargar").icono("download").modal().ancho(restante).ui(ui).clicked();
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
        let entrada = salida_suave(transcurrido as f32 / DURACION_TOAST);
        let opacidad = if transcurrido > DURACION_AVISO { (1.0 - (transcurrido - DURACION_AVISO) / 0.2) as f32 } else { entrada };
        let pantalla = ctx.content_rect();
        let accion = aviso.accion;
        let mut cerrar = false;
        egui::Area::new(Id::new("aviso-actualizacion")).order(Order::Tooltip).fixed_pos(pantalla.min).show(ctx, |ui| {
            let titulo = v2::galeria(ui, aviso.titulo.clone(), fuente(tam::CUERPO, Peso::Medio), color::BLANCO);
            let detalle = v2::galeria(ui, aviso.detalle.clone(), fuente(tam::CHICO, Peso::Normal), color::TEXTO_DESHABILITADO);
            let texto_boton = v2::galeria(ui, accion.texto_aviso(), fuente(tam::CHICO, Peso::Semi), color::VERDE_PROFUNDO);
            let ancho_boton = texto_boton.size().x + 24.0;
            let ancho_texto = titulo.size().x.max(detalle.size().x);
            let alto_texto = titulo.size().y + 1.0 + detalle.size().y;
            let tam_caja = Vec2::new(13.0 + 19.0 + 10.0 + ancho_texto + 16.0 + ancho_boton + 10.0, alto_texto + 22.0);
            let y = pantalla.top() + 18.0 - 10.0 * (1.0 - entrada);
            let caja = Rect::from_min_size(Pos2::new(pantalla.center().x - tam_caja.x / 2.0, y), tam_caja);
            let pintor = ui.painter();
            pintor.add(sombra::TOAST.con_opacidad(opacidad).forma(caja, radio::CAMPO));
            pintor.rect_filled(caja, radio::CAMPO, color::TEXTO.gamma_multiply(opacidad));
            let caja_icono = Rect::from_center_size(Pos2::new(caja.left() + 13.0 + 9.5, caja.center().y), Vec2::splat(19.0));
            pintar_icono_solar_en(ui, caja_icono, "circle-check", color::VERDE_TOAST.gamma_multiply(opacidad));
            let x = caja_icono.right() + 10.0;
            let y_texto = caja.center().y - alto_texto / 2.0;
            let alto_titulo = titulo.size().y;
            pintor.galley_with_override_text_color(Pos2::new(x, y_texto), titulo, color::BLANCO.gamma_multiply(opacidad));
            pintor.galley_with_override_text_color(Pos2::new(x, y_texto + alto_titulo + 1.0), detalle, color::TEXTO_DESHABILITADO.gamma_multiply(opacidad));
            let boton = Rect::from_min_size(Pos2::new(caja.right() - 10.0 - ancho_boton, caja.center().y - 15.0), Vec2::new(ancho_boton, 30.0));
            let respuesta = ui.interact(boton, Id::new("aviso-actualizacion-accion"), Sense::click());
            let fondo = if respuesta.hovered() { color::DORADO_BORDE } else { color::DORADO };
            pintor.rect_filled(boton, radio::ICONO, fondo.gamma_multiply(opacidad));
            pintor.galley_with_override_text_color(boton.center() - texto_boton.size() / 2.0, texto_boton, color::VERDE_PROFUNDO.gamma_multiply(opacidad));
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
