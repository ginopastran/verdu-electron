use std::collections::HashMap;
use std::path::Path;

use eframe::egui::{Context, Image, Pos2, Rect, Ui, Vec2};
use nucleo::api::Cliente;

use super::productos::ProductoDisponible;
use crate::app::Contexto;
use crate::tareas::Tarea;

type Cargada = (i64, String, Option<Vec<u8>>);

#[derive(Default)]
pub struct Fotos {
    uris: HashMap<i64, String>,
    pedidas: HashMap<i64, String>,
    tareas: Vec<Tarea<Cargada>>,
}

pub fn version(foto: &str) -> String {
    foto.chars().filter(char::is_ascii_digit).collect()
}

pub fn es_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

pub fn cargar(api: &Cliente, carpeta: &Path, id: i64, version: &str) -> Option<Vec<u8>> {
    let carpeta = carpeta.join("fotos");
    let archivo = carpeta.join(format!("{id}-{version}.webp"));
    if let Some(bytes) = std::fs::read(&archivo).ok().filter(|b| es_webp(b)) {
        return Some(bytes);
    }
    let (estado, bytes) = api.bytes(&format!("/api/productos/{id}/foto?v={version}")).ok()?;
    if !(200..300).contains(&estado) || !es_webp(&bytes) {
        return None;
    }
    let _ = std::fs::create_dir_all(&carpeta);
    let prefijo = format!("{id}-");
    for viejo in std::fs::read_dir(&carpeta).into_iter().flatten().flatten() {
        if viejo.file_name().to_string_lossy().starts_with(&prefijo) {
            let _ = std::fs::remove_file(viejo.path());
        }
    }
    let _ = std::fs::write(&archivo, &bytes);
    Some(bytes)
}

impl Fotos {
    pub fn uri(&self, producto: &ProductoDisponible) -> Option<&str> {
        producto.foto.as_ref()?;
        self.uris.get(&producto.id).map(String::as_str)
    }

    pub fn pedir(&mut self, cx: &Contexto, producto: &ProductoDisponible) {
        let Some(foto) = producto.foto.as_deref() else { return };
        let version = version(foto);
        if self.pedidas.get(&producto.id) == Some(&version) {
            return;
        }
        self.pedidas.insert(producto.id, version.clone());
        let (api, carpeta, id) = (cx.servicios.api.clone(), cx.servicios.carpeta.clone(), producto.id);
        self.tareas.push(Tarea::lanzar(&cx.egui, move || {
            let bytes = cargar(&api, &carpeta, id, &version);
            (id, version, bytes)
        }));
    }

    pub fn recibir(&mut self, ctx: &Context) {
        let mut listas = Vec::new();
        self.tareas.retain_mut(|t| match t.listo() {
            Some(cargada) => {
                listas.push(cargada);
                false
            }
            None => true,
        });
        for (id, version, bytes) in listas {
            if self.pedidas.get(&id) != Some(&version) {
                continue;
            }
            match bytes {
                Some(bytes) => {
                    let uri = format!("bytes://foto-producto/{id}-{version}.webp");
                    ctx.include_bytes(uri.clone(), bytes);
                    self.uris.insert(id, uri);
                }
                None => {
                    self.uris.remove(&id);
                }
            }
        }
    }
}

pub fn pintar(ui: &Ui, rect: Rect, uri: &str, radio: u8) -> bool {
    let imagen = Image::new(uri.to_string()).corner_radius(radio);
    let Ok(eframe::egui::load::TexturePoll::Ready { texture }) = imagen.load_for_size(ui.ctx(), rect.size()) else {
        return false;
    };
    let lado = texture.size.x.min(texture.size.y).max(1.0);
    let recorte = Vec2::new(lado / texture.size.x.max(1.0), lado / texture.size.y.max(1.0));
    imagen.uv(Rect::from_center_size(Pos2::new(0.5, 0.5), recorte)).paint_at(ui, rect);
    true
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn cache_por_version_y_limpia_la_vieja() {
        let carpeta = std::env::temp_dir().join(format!("fotos-pos-{}", std::process::id()));
        let fotos = carpeta.join("fotos");
        std::fs::create_dir_all(&fotos).unwrap();
        let webp = b"RIFF\x10\x00\x00\x00WEBPVP8 ".to_vec();
        std::fs::write(fotos.join("7-1.webp"), &webp).unwrap();
        std::fs::write(fotos.join("7-0.webp"), &webp).unwrap();
        let sin_red = Cliente::new("http://127.0.0.1:9");
        assert_eq!(cargar(&sin_red, &carpeta, 7, "1"), Some(webp));
        assert_eq!(cargar(&sin_red, &carpeta, 7, "2"), None);
        assert_eq!(version("2026-10-10T12:00:00.000Z"), "20261010120000000");
        assert!(!es_webp(b"\x89PNG\r\n\x1a\n0000"));
        let _ = std::fs::remove_dir_all(carpeta);
    }
}
