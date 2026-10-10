mod trama;

use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use serialport::{DataBits, Parity, SerialPort, SerialPortType, StopBits};
use tauri::{AppHandle, Emitter, Manager};

use crate::config::Config;
use trama::{peso_de_trama, Crc, FiltroEstable, LectorTramas, Trama};

const ARCHIVO_PESO: &str = r"C:\Peso\peso.json";
const ENQ: u8 = 0x05;
const ACK: u8 = 0x06;
const NACK: u8 = 0x15;
const INTERVALO_CONSULTA: Duration = Duration::from_millis(300);
const SILENCIO_FIN_TRAMA: Duration = Duration::from_millis(60);
const REINTENTO: Duration = Duration::from_secs(5);
const INTERVALO_ARCHIVO: Duration = Duration::from_millis(500);

pub struct Balanza {
    peso_serie: Mutex<i64>,
    estado: Mutex<Value>,
    detener: Mutex<Arc<AtomicBool>>,
}

impl Balanza {
    pub fn new() -> Self {
        Balanza {
            peso_serie: Mutex::new(0),
            estado: Mutex::new(json!({ "conectada": false })),
            detener: Mutex::new(Arc::new(AtomicBool::new(false))),
        }
    }

    pub fn peso_serie(&self) -> i64 {
        *self.peso_serie.lock().unwrap()
    }

    pub fn estado(&self) -> Value {
        self.estado.lock().unwrap().clone()
    }

    pub fn reiniciar_lectura(&self) {
        *self.peso_serie.lock().unwrap() = 0;
        *self.estado.lock().unwrap() = json!({ "conectada": false });
    }
}

pub fn config_balanza(config: &Config) -> Value {
    let mut resultado = json!({
        "modo": "archivo",
        "baudRate": 9600,
        "formato": "8N1",
        "consulta": "enq",
        "unidadEntera": "gramos",
    });
    if let Value::Object(guardada) = config.get("balanza") {
        for (clave, valor) in guardada {
            resultado[clave] = valor;
        }
    }
    resultado
}

pub fn iniciar(app: &AppHandle) {
    let balanza = app.state::<Balanza>();
    let detener = Arc::new(AtomicBool::new(false));
    let anterior = std::mem::replace(&mut *balanza.detener.lock().unwrap(), detener.clone());
    anterior.store(true, Ordering::Relaxed);
    let config = config_balanza(&app.state::<Config>());
    let app = app.clone();
    if config["modo"] == "serie" {
        thread::spawn(move || leer_serie(app, config, detener));
    } else {
        thread::spawn(move || vigilar_archivo(app, detener));
    }
}

pub fn leer_peso(config: &Config, balanza: &Balanza) -> Value {
    if config_balanza(config)["modo"] == "serie" {
        return json!({ "success": true, "peso": balanza.peso_serie() });
    }
    if !Path::new(ARCHIVO_PESO).exists() {
        return json!({ "error": "Archivo no encontrado", "peso": 0 });
    }
    match peso_del_archivo() {
        Ok(Some(peso)) => json!({ "success": true, "peso": peso }),
        Ok(None) => json!({ "error": "Archivo vacío", "peso": 0 }),
        Err(error) => json!({ "error": error, "peso": 0 }),
    }
}

pub fn leer_peso_simple() -> Value {
    match peso_del_archivo() {
        Ok(Some(peso)) => peso,
        _ => json!(0),
    }
}

fn peso_del_archivo() -> Result<Option<Value>, String> {
    let datos = std::fs::read_to_string(ARCHIVO_PESO).map_err(|e| e.to_string())?;
    let limpio = datos.trim_start_matches('\u{feff}').trim();
    if limpio.is_empty() {
        return Ok(None);
    }
    let valor: Value = serde_json::from_str(limpio).map_err(|e| e.to_string())?;
    let peso = match &valor["peso"] {
        Value::Null | Value::Bool(false) => json!(0),
        Value::Number(n) if n.as_f64() == Some(0.0) => json!(0),
        Value::String(s) if s.is_empty() => json!(0),
        peso => peso.clone(),
    };
    Ok(Some(peso))
}

pub fn puertos() -> Value {
    let lista = serialport::available_ports().unwrap_or_default();
    Value::Array(
        lista
            .into_iter()
            .map(|puerto| match puerto.port_type {
                SerialPortType::UsbPort(info) => json!({ "path": puerto.port_name, "manufacturer": info.manufacturer }),
                _ => json!({ "path": puerto.port_name }),
            })
            .collect(),
    )
}

fn vigilar_archivo(app: AppHandle, detener: Arc<AtomicBool>) {
    let mut ultimo: Option<Value> = None;
    while !detener.load(Ordering::Relaxed) {
        if let Ok(Some(peso)) = peso_del_archivo() {
            if ultimo.as_ref() != Some(&peso) {
                let _ = app.emit("peso-updated", &peso);
                ultimo = Some(peso);
            }
        }
        thread::sleep(INTERVALO_ARCHIVO);
    }
}

fn informar_estado(app: &AppHandle, conectada: bool, error: Option<&str>) {
    let estado = match error {
        Some(error) => json!({ "conectada": conectada, "error": error }),
        None => json!({ "conectada": conectada }),
    };
    *app.state::<Balanza>().estado.lock().unwrap() = estado.clone();
    let _ = app.emit("balanza-estado", estado);
}

fn informar_peso(app: &AppHandle, peso: i64) {
    let balanza = app.state::<Balanza>();
    let mut actual = balanza.peso_serie.lock().unwrap();
    if *actual != peso {
        *actual = peso;
        let _ = app.emit("peso-updated", peso);
    }
}

fn abrir(puerto: &str, config: &Value) -> Result<Box<dyn SerialPort>, String> {
    let formato = config["formato"].as_str().unwrap_or("8N1").as_bytes();
    let bits = if formato.first() == Some(&b'7') { DataBits::Seven } else { DataBits::Eight };
    let paridad = match formato.get(1) {
        Some(b'E') => Parity::Even,
        Some(b'O') => Parity::Odd,
        _ => Parity::None,
    };
    let parada = if formato.get(2) == Some(&b'2') { StopBits::Two } else { StopBits::One };
    let velocidad = config["baudRate"].as_u64().unwrap_or(9600) as u32;
    serialport::new(puerto, velocidad)
        .data_bits(bits)
        .parity(paridad)
        .stop_bits(parada)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|e| e.to_string())
}

fn esperar(detener: &AtomicBool, tiempo: Duration) {
    let fin = Instant::now() + tiempo;
    while Instant::now() < fin && !detener.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }
}

fn leer_serie(app: AppHandle, config: Value, detener: Arc<AtomicBool>) {
    let Some(puerto) = config["puerto"].as_str().filter(|p| !p.is_empty()).map(str::to_string) else {
        informar_estado(&app, false, Some("Sin puerto configurado"));
        return;
    };
    let consulta = config["consulta"] == "enq";
    let kilos = config["unidadEntera"] == "kilos";
    let mut estable = FiltroEstable::new();
    while !detener.load(Ordering::Relaxed) {
        match abrir(&puerto, &config) {
            Ok(mut conexion) => {
                informar_estado(&app, true, None);
                let resultado = escuchar(&app, conexion.as_mut(), consulta, kilos, &mut estable, &detener);
                if resultado.is_ok() {
                    return;
                }
                informar_estado(&app, false, Some("Puerto cerrado"));
            }
            Err(error) => informar_estado(&app, false, Some(&error)),
        }
        esperar(&detener, REINTENTO);
    }
}

fn escuchar(
    app: &AppHandle,
    conexion: &mut dyn SerialPort,
    consulta: bool,
    kilos: bool,
    estable: &mut FiltroEstable,
    detener: &AtomicBool,
) -> Result<(), ()> {
    let mut lector = LectorTramas::default();
    let mut ultima_consulta = Instant::now();
    let mut corte: Option<Instant> = None;
    let mut bloque = [0u8; 256];
    let mut procesar = |conexion: &mut dyn SerialPort, tramas: Vec<Trama>| -> Result<(), ()> {
        for trama in tramas {
            if trama.crc != Crc::Sin && consulta {
                conexion.write_all(&[if trama.crc == Crc::Ok { ACK } else { NACK }]).map_err(|_| ())?;
            }
            if trama.crc == Crc::Mal {
                continue;
            }
            if let Some(valor) = peso_de_trama(&trama.texto, kilos).and_then(|gramos| estable.leer(gramos)) {
                informar_peso(app, valor);
            }
        }
        Ok(())
    };
    while !detener.load(Ordering::Relaxed) {
        if consulta && ultima_consulta.elapsed() >= INTERVALO_CONSULTA {
            conexion.write_all(&[ENQ]).map_err(|_| ())?;
            ultima_consulta = Instant::now();
        }
        match conexion.read(&mut bloque) {
            Ok(leidos) if leidos > 0 => {
                let tramas = lector.leer(&bloque[..leidos], false);
                procesar(conexion, tramas)?;
                corte = Some(Instant::now() + SILENCIO_FIN_TRAMA);
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::TimedOut => {}
            Err(_) => return Err(()),
        }
        if corte.is_some_and(|momento| Instant::now() >= momento) {
            corte = None;
            let tramas = lector.leer(&[], true);
            procesar(conexion, tramas)?;
        }
    }
    Ok(())
}
