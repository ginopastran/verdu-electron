use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

use eframe::egui::{self, Context, Ui};
use hardware::balanza::{Balanza, Evento};
use hardware::config::Config;
use nucleo::api::Cliente;
use nucleo::carrito::Carritos;
use serde_json::Value;

use crate::pantallas::Pantallas;
use crate::ui;

#[derive(Clone, Debug, PartialEq)]
pub enum Ruta {
    Inicio,
    AdminLogin,
    UsuarioLogin,
    MensajeAdmin,
    Carrito,
    Facturas,
    FacturaDetalle(String),
}

#[derive(Clone)]
pub struct Servicios {
    pub config: Arc<Config>,
    pub balanza: Balanza,
    pub api: Cliente,
    pub carpeta: PathBuf,
}

pub struct Global {
    pub ruta: Ruta,
    pub admin: Option<Value>,
    pub negocio: Option<Value>,
    pub negocio_cargando: bool,
    pub usuario: Option<Value>,
    pub carritos: Carritos,
    pub peso: Value,
    pub estado_balanza: Value,
    pub en_linea: bool,
}

impl Global {
    pub fn ir(&mut self, ruta: Ruta) {
        self.ruta = ruta;
    }

    pub fn negocio_bool(&self, clave: &str) -> bool {
        self.negocio.as_ref().is_some_and(|n| n[clave].as_bool().unwrap_or(false))
    }

    pub fn business_id(&self) -> Option<i64> {
        self.admin.as_ref().and_then(|a| a["businessId"].as_i64()).filter(|id| *id > 0)
    }
}

pub struct Contexto<'a> {
    pub global: &'a mut Global,
    pub servicios: &'a Servicios,
    pub egui: Context,
}

pub struct App {
    servicios: Servicios,
    global: Global,
    eventos_balanza: Receiver<Evento>,
    pantallas: Pantallas,
    instalado: bool,
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        ui::instalar(ctx);
        let carpeta = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("com.andextech.andexmarket");
        let config = Arc::new(Config::cargar(carpeta.clone()));
        let (emisor, eventos_balanza) = channel();
        let repintar = ctx.clone();
        let balanza = Balanza::new(move |evento| {
            let _ = emisor.send(evento);
            repintar.request_repaint();
        });
        balanza.iniciar(&config);
        let admin = Some(config.get("adminData")).filter(|a| a["businessId"].as_i64().is_some_and(|id| id > 0));
        let global = Global {
            ruta: Ruta::Inicio,
            admin,
            negocio: None,
            negocio_cargando: false,
            usuario: None,
            carritos: Carritos::cargar(carpeta.join("carrito.json")),
            peso: Value::from(0),
            estado_balanza: balanza.estado(),
            en_linea: true,
        };
        let servicios = Servicios { config, balanza, api: Cliente::default(), carpeta };
        App { servicios, global, eventos_balanza, pantallas: Pantallas::default(), instalado: true }
    }

    pub fn de_prueba(global: Global, servicios: Servicios) -> Self {
        let (_, eventos_balanza) = channel();
        App { servicios, global, eventos_balanza, pantallas: Pantallas::default(), instalado: false }
    }

    pub fn global(&mut self) -> &mut Global {
        &mut self.global
    }

    pub fn pantallas(&mut self) -> &mut Pantallas {
        &mut self.pantallas
    }

    pub fn mostrar(&mut self, ui: &mut Ui) {
        if !self.instalado {
            ui::instalar(ui.ctx());
            self.instalado = true;
            ui.ctx().request_repaint();
            return;
        }
        while let Ok(evento) = self.eventos_balanza.try_recv() {
            match evento {
                Evento::Peso(peso) => self.global.peso = peso,
                Evento::Estado(estado) => self.global.estado_balanza = estado,
            }
        }
        let mut contexto = Contexto { global: &mut self.global, servicios: &self.servicios, egui: ui.ctx().clone() };
        self.pantallas.mostrar(ui, &mut contexto);
        ui::toast::pintar(ui.ctx());
        ui::v2::toast::pintar(ui.ctx());
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::new().fill(ui::color::FONDO).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            self.mostrar(ui);
        });
    }
}
