use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use super::formato::*;
use super::*;
use crate::escpos::{Ticket, CENTRO, IZQUIERDA};

const LINEA_CORTA: &str = "-----------------------------\n";
const ESPACIOS_PHP: [char; 6] = [' ', '\t', '\n', '\r', '\x0b', '\x0c'];

fn numerico(texto: &str) -> Option<f64> {
    let recortado = texto.trim_matches(ESPACIOS_PHP);
    let bytes = recortado.as_bytes();
    let digitos = |desde: usize| bytes[desde..].iter().take_while(|b| b.is_ascii_digit()).count();
    let mut fin = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let enteros = digitos(fin);
    fin += enteros;
    let mut decimales = 0;
    if bytes.get(fin) == Some(&b'.') {
        decimales = digitos(fin + 1);
        fin += 1 + decimales;
    }
    if enteros + decimales == 0 {
        return None;
    }
    if matches!(bytes.get(fin), Some(b'e' | b'E')) {
        let signo = usize::from(matches!(bytes.get(fin + 1), Some(b'+' | b'-')));
        let exponente = digitos(fin + 1 + signo);
        if exponente == 0 {
            return None;
        }
        fin += 1 + signo + exponente;
    }
    (fin == bytes.len()).then(|| recortado.parse().ok()).flatten()
}

fn igual_a(valor: &Value, objetivo: f64) -> bool {
    match valor {
        Value::Number(n) => n.as_f64() == Some(objetivo),
        Value::String(s) => numerico(s) == Some(objetivo),
        Value::Bool(b) => *b == (objetivo != 0.0),
        _ => false,
    }
}

fn mayor_que_cero(valor: &Value) -> bool {
    match valor {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|n| n > 0.0),
        Value::String(s) => match numerico(s) {
            Some(n) => n > 0.0,
            None => s.as_str() > "0",
        },
        Value::Array(_) | Value::Object(_) => true,
    }
}

struct Lector<'a> {
    bytes: &'a [u8],
    posicion: usize,
}

impl Lector<'_> {
    fn fin(&self) -> bool {
        self.posicion == self.bytes.len()
    }

    fn siguiente(&self) -> Option<u8> {
        self.bytes.get(self.posicion).copied()
    }

    fn tomar(&mut self, opciones: &[u8]) -> Option<u8> {
        let byte = self.siguiente().filter(|b| opciones.contains(b))?;
        self.posicion += 1;
        Some(byte)
    }

    fn espacios(&mut self) -> usize {
        let desde = self.posicion;
        while self.tomar(b" \t").is_some() {}
        self.posicion - desde
    }

    fn numero(&mut self, minimo: usize, maximo: usize) -> Option<i64> {
        let cantidad = self.bytes[self.posicion..].iter().take(maximo).take_while(|b| b.is_ascii_digit()).count();
        if cantidad < minimo {
            return None;
        }
        let digitos = &self.bytes[self.posicion..self.posicion + cantidad];
        self.posicion += cantidad;
        Some(digitos.iter().fold(0, |total, d| total * 10 + i64::from(d - b'0')))
    }

    fn zona(&mut self) -> Option<i64> {
        let resto = &self.bytes[self.posicion..];
        if resto.len() >= 3 && (resto[..3].eq_ignore_ascii_case(b"UTC") || resto[..3].eq_ignore_ascii_case(b"GMT")) {
            self.posicion += 3;
            return Some(0);
        }
        if self.tomar(b"Zz").is_some() {
            return Some(0);
        }
        let signo = if self.tomar(b"+-")? == b'-' { -1 } else { 1 };
        let desde = self.posicion;
        let (horas, minutos) = match self.numero(1, 4)? {
            valor if self.posicion - desde == 4 => (valor / 100, valor % 100),
            valor if self.posicion - desde <= 2 => (valor, if self.tomar(b":").is_some() { self.numero(2, 2)? } else { 0 }),
            _ => return None,
        };
        (horas <= 24 && minutos <= 59).then_some(signo * (horas * 3600 + minutos * 60))
    }
}

fn dias_desde_epoca(anio: i64, mes: i64, dia: i64) -> i64 {
    let anio = anio + (mes - 1).div_euclid(12);
    let mes = (mes - 1).rem_euclid(12) + 1;
    let anio_marzo = if mes <= 2 { anio - 1 } else { anio };
    let era = anio_marzo.div_euclid(400);
    let anio_de_era = anio_marzo - era * 400;
    let dia_del_anio = (153 * ((mes + 9) % 12) + 2) / 5;
    let dia_de_era = anio_de_era * 365 + anio_de_era / 4 - anio_de_era / 100 + dia_del_anio;
    era * 146097 + dia_de_era - 719468 + (dia - 1)
}

fn strtotime(valor: &str, zona: FixedOffset) -> Option<i64> {
    let mut lector = Lector { bytes: valor.trim_matches(ESPACIOS_PHP).as_bytes(), posicion: 0 };
    let anio = lector.numero(4, 4)?;
    let separador_fecha = lector.tomar(b"-/")?;
    let mes = lector.numero(1, 2)?;
    lector.tomar(&[separador_fecha])?;
    let dia = lector.numero(1, 2)?;
    let (mut hora, mut minuto, mut segundo) = (0, 0, 0);
    let mut desfase = i64::from(zona.local_minus_utc());
    if !lector.fin() {
        let con_t = lector.tomar(b"Tt").is_some();
        if !con_t && lector.espacios() == 0 {
            return None;
        }
        hora = lector.numero(1, 2)?;
        if lector.tomar(b":").is_some() {
            minuto = lector.numero(1, 2)?;
            if lector.tomar(b":").is_some() {
                segundo = lector.numero(1, 2)?;
                if lector.tomar(b".").is_some() {
                    lector.numero(0, usize::MAX);
                }
            }
        } else if !con_t {
            return None;
        }
        lector.espacios();
        if !lector.fin() {
            desfase = lector.zona()?;
        }
        if !lector.fin() {
            return None;
        }
    }
    if mes > 12 || dia > 31 || hora > 24 || minuto > 59 || segundo > 60 {
        return None;
    }
    Some(dias_desde_epoca(anio, mes, dia) * 86400 + hora * 3600 + minuto * 60 + segundo - desfase)
}

fn fecha(datos: &Value, ahora: Ahora) -> String {
    let zona = FixedOffset::west_opt(3 * 3600).unwrap();
    let instante = match &datos["fechaEmision"] {
        Value::Null => ahora.with_timezone(&zona),
        valor => {
            let marca = strtotime(&texto(valor), zona).unwrap_or(0);
            DateTime::from_timestamp(marca, 0).unwrap_or_default().with_timezone(&zona)
        }
    };
    instante.format("%d/%m/%Y %H:%M").to_string()
}

fn nombre_detalle(detalle: &Value) -> String {
    let producto = &detalle["producto"];
    if !detalle["descripcion"].is_null() {
        texto(&detalle["descripcion"])
    } else if es_arreglo(producto) && !producto["nombre"].is_null() {
        texto(&producto["nombre"])
    } else if let Some(nombre) = producto.as_str() {
        nombre.to_string()
    } else {
        "Producto".into()
    }
}

fn es_qr_arca(url: &str) -> bool {
    ["https://www.arca.gob.ar/fe/qr/?p=", "https://www.afip.gob.ar/fe/qr/?p="].iter().any(|prefijo| url.starts_with(prefijo))
}

fn monto(etiqueta: &str, valor: f64) -> String {
    format!("{}\n", rellenar_izquierda(&format!("{etiqueta}: ${}", numero(valor, 2)), 32))
}

pub fn factura(datos: &Value, ahora: Ahora) -> Vec<u8> {
    let mut ticket = Ticket::new();
    ticket.alinear(CENTRO);
    let ancho_logo = if igual_a(&datos["anchoPapel"], 58.0) { 370 } else { 556 };
    if let Some(imagen) = logo(datos, ancho_logo) {
        ticket.imagen(&imagen);
    }

    ticket.negrita(true);
    ticket.tamano(2, 2);
    let negocio = ["businessName", "nombre", "razonSocial", "sucursal"]
        .into_iter()
        .map(|clave| &datos[clave])
        .find(|valor| !vacio(valor))
        .map_or_else(|| "Mi Negocio".to_string(), texto);
    ticket.texto(&format!("{}\n", negocio.to_ascii_uppercase()));
    ticket.negrita(false);
    ticket.tamano(1, 1);

    ticket.negrita(true);
    let tipo = match &datos["tipoFactura"] {
        Value::Null => "DOCUMENTO".to_string(),
        tipo => mayusculas(tipo),
    };
    ticket.texto(&format!("{tipo}\n"));
    ticket.negrita(false);

    if !vacio(&datos["numero"]) {
        ticket.texto(&format!("Nº {}\n", texto(&datos["numero"])));
    } else if !vacio(&datos["id"]) {
        ticket.texto(&format!("ID: {}\n", texto(&datos["id"])));
    }
    ticket.texto(&format!("Fecha: {}\n", fecha(datos, ahora)));
    ticket.texto(LINEA_CORTA);

    ticket.alinear(IZQUIERDA);
    ticket.negrita(true);
    ticket.texto("DATOS DEL CLIENTE\n");
    ticket.negrita(false);
    let cliente = &datos["cliente"];
    if !cliente.is_null() {
        if !vacio(&cliente["razonSocial"]) {
            ticket.texto(&format!("Razón Social: {}\n", texto(&cliente["razonSocial"])));
        } else {
            let completo = format!("{} {}", texto(&cliente["nombre"]), texto(&cliente["apellido"]));
            let completo = completo.trim_matches([' ', '\t', '\n', '\r', '\0', '\x0b']);
            if !completo.is_empty() && completo != "0" {
                ticket.texto(&format!("Cliente: {completo}\n"));
            }
        }
        if !vacio(&cliente["cuit"]) {
            ticket.texto(&format!("CUIT: {}\n", texto(&cliente["cuit"])));
        }
    }
    ticket.texto(&format!("{}\n", separador()));

    ticket.alinear(IZQUIERDA);
    ticket.texto(&format!("{}\n", encabezado_columnas()));
    ticket.texto(&format!("{}\n", separador()));

    let detalles = elementos(&datos["detalles"]);
    if detalles.is_empty() {
        ticket.texto("No hay productos para mostrar\n");
    }
    for detalle in detalles {
        let cantidad = flotante(&detalle["cantidad"]);
        let precio = flotante(primero([&detalle["precioUnitario"], &detalle["precio"]]));
        let subtotal = match &detalle["subtotal"] {
            Value::Null => cantidad * precio,
            valor => flotante(valor),
        };
        for linea in lineas_item(&nombre_detalle(detalle), cantidad, precio, subtotal) {
            ticket.texto(&format!("{linea}\n"));
        }
    }

    ticket.texto(&format!("{}\n", separador()));
    if !datos["subtotal"].is_null() {
        ticket.texto(&monto("SUBTOTAL", flotante(&datos["subtotal"])));
    }
    if mayor_que_cero(&datos["impuestos"]) {
        ticket.texto(&monto("IVA", flotante(&datos["impuestos"])));
    }
    let total = flotante(&datos["total"]);
    ticket.negrita(true);
    ticket.texto(&monto("TOTAL", total));
    ticket.negrita(false);

    if mayor_que_cero(&datos["pagoInicial"]) {
        let pago_inicial = flotante(&datos["pagoInicial"]);
        ticket.texto(LINEA_CORTA);
        ticket.texto(&monto("PAGO INICIAL", pago_inicial));
        ticket.negrita(true);
        ticket.texto(&monto("SALDO", total - pago_inicial));
        ticket.negrita(false);
    }

    if !vacio(&datos["observaciones"]) {
        ticket.texto(LINEA_CORTA);
        ticket.texto("OBSERVACIONES:\n");
        ticket.texto(&format!("{}\n", texto(&datos["observaciones"])));
    }

    let afip = &datos["afip"];
    if !afip.is_null() {
        ticket.texto(LINEA_CORTA);
        ticket.negrita(true);
        ticket.texto("INFORMACIÓN AFIP\n");
        ticket.negrita(false);
        if !afip["cae"].is_null() {
            ticket.texto(&format!("CAE: {}\n", texto(&afip["cae"])));
        }
        if !afip["vencimientoCae"].is_null() {
            ticket.texto(&format!("Venc. CAE: {}\n", texto(&afip["vencimientoCae"])));
        }
        let qr = primero([&datos["afipQrUrl"], &afip["afipQrUrl"]]);
        if let Some(url) = qr.as_str().filter(|url| !vacio(&afip["cae"]) && es_qr_arca(url)) {
            ticket.avanzar(1);
            ticket.alinear(CENTRO);
            ticket.qr(url, 1, 5);
            ticket.avanzar(1);
        }
    }

    ticket.alinear(CENTRO);
    ticket.texto("\n¡Gracias por su compra!\n");
    ticket.avanzar(3);
    ticket.cortar();
    ticket.abrir_cajon();
    ticket.bytes()
}

#[cfg(test)]
mod tests {
    use super::super::pruebas::*;
    use super::*;

    #[test]
    fn igual_que_factura_ticket_printer_php() {
        comparar("factura", |datos| factura(datos, ahora_fija()));
    }
}
