use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Map, Value};

pub const IMPRESORA_DEFAULT: &str = "TP806L";

pub struct Config {
    carpeta: PathBuf,
    datos: Mutex<Map<String, Value>>,
}

impl Config {
    pub fn cargar(carpeta: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&carpeta);
        if !carpeta.join("config.json").exists() {
            migrar_desde_electron(&carpeta);
        }
        let datos = std::fs::read_to_string(carpeta.join("config.json"))
            .ok()
            .and_then(|texto| serde_json::from_str(&texto).ok())
            .unwrap_or_default();
        Config { carpeta, datos: Mutex::new(datos) }
    }

    pub fn get(&self, clave: &str) -> Value {
        self.datos.lock().unwrap().get(clave).cloned().unwrap_or(Value::Null)
    }

    pub fn has(&self, clave: &str) -> bool {
        self.datos.lock().unwrap().contains_key(clave)
    }

    pub fn set(&self, clave: &str, valor: Value) {
        let mut datos = self.datos.lock().unwrap();
        datos.insert(clave.to_string(), valor);
        self.guardar(&datos);
    }

    pub fn delete(&self, clave: &str) {
        let mut datos = self.datos.lock().unwrap();
        datos.shift_remove(clave);
        self.guardar(&datos);
    }

    pub fn ruta_logo_negocio(&self) -> PathBuf {
        self.carpeta.join("logo-negocio.png")
    }

    pub fn ancho_papel(&self) -> u8 {
        if self.get("anchoPapel") == 58 {
            58
        } else {
            80
        }
    }

    fn guardar(&self, datos: &Map<String, Value>) {
        let ruta = self.carpeta.join("config.json");
        let temporal = ruta.with_extension("json.tmp");
        let contenido = serde_json::to_vec_pretty(datos).unwrap_or_default();
        if std::fs::write(&temporal, contenido).is_ok() {
            let _ = std::fs::rename(&temporal, &ruta);
        }
    }
}

fn migrar_desde_electron(carpeta: &Path) {
    let Some(appdata) = std::env::var_os("APPDATA") else {
        return;
    };
    let anterior = Path::new(&appdata).join("andex-market");
    for archivo in ["config.json", "logo-negocio.png"] {
        let origen = anterior.join(archivo);
        if origen.is_file() {
            let _ = std::fs::copy(&origen, carpeta.join(archivo));
        }
    }
}
