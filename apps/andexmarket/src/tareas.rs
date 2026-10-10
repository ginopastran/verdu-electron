use std::sync::mpsc::{channel, Receiver, TryRecvError};

use eframe::egui::Context;

pub struct Tarea<T> {
    receptor: Receiver<T>,
}

impl<T: Send + 'static> Tarea<T> {
    pub fn lanzar(ctx: &Context, trabajo: impl FnOnce() -> T + Send + 'static) -> Self {
        let (emisor, receptor) = channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = emisor.send(trabajo());
            ctx.request_repaint();
        });
        Tarea { receptor }
    }

    pub fn listo(&mut self) -> Option<T> {
        match self.receptor.try_recv() {
            Ok(valor) => Some(valor),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }
}

pub fn esperar<T: Send + 'static>(tarea: &mut Option<Tarea<T>>) -> Option<T> {
    let resultado = tarea.as_mut()?.listo();
    if resultado.is_some() {
        *tarea = None;
    }
    resultado
}
