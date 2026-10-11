use chrono::{Duration, NaiveDate};
use serde_json::Value;

use super::*;
use crate::escpos::{Ticket, CENTRO, IZQUIERDA};

const SEPARADOR: &str = "-----------------------------\n";
const ANCHO_NEGOCIO: usize = 20;

fn o(valor: &Value, defecto: &str) -> String {
    match valor {
        Value::Null => defecto.into(),
        valor => texto(valor),
    }
}

fn vacio_texto(valor: &str) -> bool {
    valor.is_empty() || valor == "0"
}

fn configurado<'a>(datos: &'a Value, campo: &str) -> Option<&'a Value> {
    [&datos["configuracionAfip"][campo], &datos[campo]].into_iter().find(|valor| !vacio(valor))
}

fn capitalizar(valor: &str) -> String {
    let mut resultado = valor.to_ascii_lowercase();
    if let Some(inicial) = resultado.get_mut(0..1) {
        inicial.make_ascii_uppercase();
    }
    resultado
}

fn rellenar_derecha(valor: &str, ancho: usize) -> String {
    format!("{valor}{}", " ".repeat(ancho.saturating_sub(valor.len())))
}

fn lineas_negocio(nombre: &str) -> Vec<String> {
    if nombre.len() <= ANCHO_NEGOCIO {
        return vec![nombre.to_string()];
    }
    let recortar = |linea: &str| linea.trim_matches([' ', '\t', '\n', '\r', '\0', '\x0b']).to_string();
    let mut lineas = Vec::new();
    let mut actual = String::new();
    for palabra in nombre.split(' ') {
        if actual.len() + 1 + palabra.len() > ANCHO_NEGOCIO {
            if !vacio_texto(&actual) {
                lineas.push(recortar(&actual));
                actual = palabra.to_string();
            } else {
                lineas.push(palabra.to_string());
            }
        } else {
            if !vacio_texto(&actual) {
                actual.push(' ');
            }
            actual.push_str(palabra);
        }
    }
    if !vacio_texto(&actual) {
        lineas.push(recortar(&actual));
    }
    lineas
}

fn formatear_cuit(cuit: String) -> String {
    let digitos = cuit.strip_suffix('\n').unwrap_or(&cuit);
    if digitos.len() == 11 && digitos.bytes().all(|b| b.is_ascii_digit()) {
        format!("{}-{}-{}", &digitos[..2], &digitos[2..10], &digitos[10..])
    } else {
        cuit
    }
}

fn metodo_pago(metodo: &str, original: &str) -> String {
    match metodo.to_ascii_lowercase().as_str() {
        "tarjeta" => "TARJETA".into(),
        "transferencia" => "TRANSFERENCIA".into(),
        "efectivo" => "EFECTIVO".into(),
        "qr" => "QR / MERCADOPAGO".into(),
        _ => original.to_ascii_uppercase(),
    }
}

struct Lector<'a> {
    bytes: &'a [u8],
    posicion: usize,
}

impl Lector<'_> {
    fn digitos(&mut self, minimo: usize, maximo: usize) -> Option<(i64, usize)> {
        let cantidad = self.bytes[self.posicion..].iter().take(maximo).take_while(|b| b.is_ascii_digit()).count();
        if cantidad < minimo {
            return None;
        }
        let texto = std::str::from_utf8(&self.bytes[self.posicion..self.posicion + cantidad]).ok()?;
        self.posicion += cantidad;
        Some((texto.parse().ok()?, cantidad))
    }

    fn rango(&mut self, minimo: usize, maximo: usize, tope: i64) -> Option<i64> {
        self.digitos(minimo, maximo).map(|(valor, _)| valor).filter(|valor| *valor <= tope)
    }

    fn simbolo(&mut self, opciones: &[u8]) -> Option<u8> {
        let actual = *self.bytes.get(self.posicion)?;
        if opciones.contains(&actual) {
            self.posicion += 1;
            Some(actual)
        } else {
            None
        }
    }

    fn espacios(&mut self) -> usize {
        let cantidad = self.bytes[self.posicion..].iter().take_while(|b| matches!(b, b' ' | b'\t')).count();
        self.posicion += cantidad;
        cantidad
    }

    fn fin(&self) -> bool {
        self.posicion == self.bytes.len()
    }

    fn fecha(&mut self) -> Option<(i64, i64, i64)> {
        let inicio = self.posicion;
        if let Some((valor, 8)) = self.digitos(8, 8).filter(|_| !self.bytes.get(self.posicion).is_some_and(u8::is_ascii_digit)) {
            let (año, mes, dia) = (valor / 10000, valor / 100 % 100, valor % 100);
            return (mes <= 12 && dia <= 31).then_some((año, mes, dia));
        }
        self.posicion = inicio;
        if let Some(fecha) = self.fecha_iso() {
            return Some(fecha);
        }
        self.posicion = inicio;
        if let Some(fecha) = self.fecha_americana() {
            return Some(fecha);
        }
        self.posicion = inicio;
        self.fecha_europea()
    }

    fn fecha_iso(&mut self) -> Option<(i64, i64, i64)> {
        let (año, _) = self.digitos(4, 4)?;
        let separador = self.simbolo(b"-/")?;
        let mes = self.rango(1, 2, 12)?;
        self.simbolo(&[separador])?;
        let dia = self.rango(1, 2, 31)?;
        Some((año, mes, dia))
    }

    fn fecha_americana(&mut self) -> Option<(i64, i64, i64)> {
        let mes = self.rango(1, 2, 12)?;
        self.simbolo(b"/")?;
        let dia = self.rango(1, 2, 31)?;
        self.simbolo(b"/")?;
        let año = match self.digitos(2, 4)? {
            (año, 4) => año,
            (año, 2) if año < 70 => año + 2000,
            (año, 2) => año + 1900,
            _ => return None,
        };
        Some((año, mes, dia))
    }

    fn fecha_europea(&mut self) -> Option<(i64, i64, i64)> {
        let dia = self.rango(1, 2, 31)?;
        let separador = self.simbolo(b"-.")?;
        let mes = self.rango(1, 2, 12)?;
        self.simbolo(&[separador])?;
        let (año, _) = self.digitos(4, 4)?;
        Some((año, mes, dia))
    }

    fn hora(&mut self) -> Option<i64> {
        self.simbolo(b",");
        if self.simbolo(b"Tt").is_none() && self.espacios() == 0 {
            return None;
        }
        let horas = self.rango(1, 2, 24)?;
        self.simbolo(b":")?;
        let minutos = self.rango(2, 2, 59)?;
        let mut segundos = 0;
        if self.simbolo(b":").is_some() {
            segundos = self.rango(2, 2, 60)?;
            if self.simbolo(b".,").is_some() {
                self.digitos(1, usize::MAX)?;
            }
        }
        Some(horas * 3600 + minutos * 60 + segundos)
    }

    fn zona(&mut self) -> Option<i64> {
        self.espacios();
        if self.simbolo(b"Zz").is_some() {
            return Some(0);
        }
        let signo = if self.simbolo(b"+-")? == b'-' { -1 } else { 1 };
        let horas = self.rango(1, 2, 99)?;
        self.simbolo(b":");
        let minutos = self.digitos(2, 2).map_or(0, |(valor, _)| valor);
        Some(signo * (horas * 3600 + minutos * 60))
    }
}

fn strtotime(valor: &str) -> Option<NaiveDate> {
    let valor = valor.trim_matches([' ', '\t', '\n', '\r']);
    let mut lector = Lector { bytes: valor.as_bytes(), posicion: 0 };
    let (año, mes, dia) = lector.fecha()?;
    let mut segundos = 0;
    let mut zona = None;
    if !lector.fin() {
        segundos = lector.hora()?;
        if !lector.fin() {
            zona = Some(lector.zona()?);
        }
        lector.espacios();
        if !lector.fin() {
            return None;
        }
    }
    let primero_del_mes = NaiveDate::from_ymd_opt((año + (mes - 1).div_euclid(12)) as i32, ((mes - 1).rem_euclid(12) + 1) as u32, 1)?;
    let mut momento = primero_del_mes.and_hms_opt(0, 0, 0)? + Duration::days(dia - 1) + Duration::seconds(segundos);
    if let Some(desplazamiento) = zona {
        momento = momento - Duration::seconds(desplazamiento) - Duration::hours(3);
    }
    Some(momento.date())
}

fn vencimiento_cae(valor: &str) -> String {
    strtotime(valor).map_or("31/12/1969".into(), |fecha| fecha.format("%d/%m/%Y").to_string())
}

pub fn afip(datos: &Value, ahora: Ahora) -> Vec<u8> {
    let mut ticket = Ticket::new();
    ticket.alinear(CENTRO);
    let ancho_logo = if flotante(&datos["anchoPapel"]) == 58.0 { 370 } else { 556 };
    if let Some(imagen) = logo(datos, ancho_logo) {
        ticket.imagen(&imagen);
    }

    ticket.negrita(true);
    ticket.tamano(2, 2);
    let negocio = [&datos["businessName"], &datos["nombre"]]
        .into_iter()
        .find(|valor| !vacio(valor))
        .map_or("Comercio".into(), texto);
    let razon_social = configurado(datos, "razonSocial").map_or(negocio.clone(), texto);
    let cuit = configurado(datos, "cuit").map_or("00-00000000-0".into(), texto);
    for linea in lineas_negocio(&negocio.to_ascii_uppercase()) {
        ticket.texto(&format!("{linea}\n"));
    }
    ticket.negrita(false);
    ticket.tamano(1, 1);

    ticket.texto(&format!("Razón Social: {razon_social}\n"));
    ticket.texto(&format!("CUIT: {}\n", formatear_cuit(cuit)));
    let condicion_iva = configurado(datos, "condicionIva").map(texto);
    let condicion = condicion_iva.as_deref().unwrap_or("Responsable Inscripto");
    ticket.texto(&format!("Condición IVA: {}\n", capitalizar(condicion)));
    ticket.texto(&format!("Dirección: {}\n", o(&datos["direccion"], "Dirección no configurada")));
    ticket.texto(SEPARADOR);

    ticket.alinear(CENTRO);
    ticket.negrita(true);
    ticket.tamano(1, 2);
    let monotributo = condicion_iva.as_deref().is_some_and(|condicion| condicion.to_ascii_lowercase() == "monotributo");
    ticket.texto(if monotributo { "FACTURA C\n" } else { "FACTURA B\n" });
    ticket.negrita(false);
    ticket.tamano(1, 1);
    ticket.alinear(IZQUIERDA);

    let numero_factura = match primero([&datos["numeroFactura"], &datos["numero"]]) {
        Value::Null => "1".into(),
        valor => texto(valor),
    };
    let ceros = "0".repeat(8usize.saturating_sub(numero_factura.len()));
    ticket.texto(&format!("Nro: {}-{ceros}{numero_factura}\n", o(&datos["puntoVenta"], "0001")));
    let fecha = match &datos["fechaHora"] {
        Value::Null => ahora.format("%d/%m/%Y %H:%M:%S").to_string(),
        valor => texto(valor),
    };
    ticket.texto(&format!("Fecha: {fecha}\n"));
    ticket.texto(&format!("Vendedor: {}\n", o(primero([&datos["vendedor"], &datos["usuario"]]), "N/A")));
    let factura = &datos["factura"];
    if !vacio(&datos["idReal"]) {
        ticket.texto(&format!("Orden #{}\n", texto(&datos["idReal"])));
    } else if es_arreglo(factura) && !vacio(&factura["idReal"]) {
        ticket.texto(&format!("Orden #{}\n", texto(&factura["idReal"])));
    }
    ticket.texto(SEPARADOR);

    let condicion = condicion_iva.as_deref().map_or("Consumidor Final".into(), capitalizar);
    ticket.texto(&format!("Condición IVA: {condicion}\n"));
    ticket.texto(SEPARADOR);
    ticket.texto("PRODUCTO      CANT    PRECIO    TOTAL\n");
    ticket.texto(SEPARADOR);

    for item in elementos(&datos["items"]) {
        let nombre = texto(&item["nombre"]);
        let Ok(nombre) = std::str::from_utf8(&nombre.as_bytes()[..nombre.len().min(12)]) else {
            return Vec::new();
        };
        let cantidad = flotante(&item["cantidad"]);
        let subtotal = flotante(&item["subtotal"]);
        let precio = if !item["precio"].is_null() {
            flotante(&item["precio"])
        } else if !item["precioHistorico"].is_null() {
            flotante(&item["precioHistorico"])
        } else if cantidad > 0.0 {
            subtotal / cantidad
        } else {
            0.0
        };
        ticket.texto(&format!(
            "{} {} {} {}\n",
            rellenar_derecha(nombre, 12),
            rellenar_derecha(&numero(cantidad, 3), 8),
            rellenar_derecha(&format!("${}", numero(precio, 2)), 8),
            rellenar_derecha(&format!("${}", numero(subtotal, 2)), 8)
        ));
    }

    let total = flotante(&datos["total"]);
    let neto = total / 1.21;
    let iva = total - neto;
    ticket.texto(SEPARADOR);

    let mut descuento = None;
    let datos_descuento = &datos["discountData"];
    if es_arreglo(datos_descuento) {
        let monto = flotante(&datos_descuento["amount"]);
        if monto > 0.0 {
            let original = if flotante(&datos["subtotalSinDescuento"]) > 0.0 {
                flotante(&datos["subtotalSinDescuento"])
            } else if flotante(&datos["subtotal"]) > 0.0 {
                flotante(&datos["subtotal"])
            } else {
                total + monto
            };
            let tipo = match &datos_descuento["type"] {
                Value::Null => Value::String("fixed".into()),
                tipo => tipo.clone(),
            };
            let valor = match &datos_descuento["value"] {
                Value::Null => Value::from(0),
                valor => valor.clone(),
            };
            descuento = Some((original, monto, tipo, valor));
        }
    } else if !datos["subtotal"].is_null() && !datos["total"].is_null() && flotante(&datos["subtotal"]) > total {
        let subtotal = flotante(&datos["subtotal"]);
        descuento = Some((subtotal, subtotal - total, Value::String(String::new()), Value::from(0)));
    }

    if let Some((original, monto, tipo, valor)) = &descuento {
        let valor_numero = flotante(valor);
        if tipo == "amount" || tipo == "fixed" {
            ticket.texto(&format!("Subtotal sin descuento: ${}\n", numero(*original, 2)));
            let linea = if vacio(valor) || valor_numero <= 0.0 {
                format!("Descuento aplicado: -${}\n", numero(*monto, 2))
            } else if tipo == "fixed" {
                format!("Descuento fijo aplicado: -${}\n", numero(*monto, 2))
            } else {
                format!("Descuento (${}): -${}\n", numero(valor_numero, 2), numero(*monto, 2))
            };
            ticket.texto(&linea);
            ticket.texto(&format!("Total c/descuento: ${}\n", numero(total, 2)));
        } else {
            ticket.texto(&format!("Subtotal sin descuento: ${}\n", numero(original / 1.21, 2)));
            let monto = numero(monto / 1.21, 2);
            let linea = if !vacio(tipo) && valor_numero > 0.0 && tipo == "percentage" {
                format!("Descuento aplicado ({}%): -${monto}\n", numero(valor_numero, 1))
            } else {
                format!("Descuento aplicado: -${monto}\n")
            };
            ticket.texto(&linea);
            ticket.texto(&format!("Subtotal con descuento: ${}\n", numero(neto, 2)));
        }
        ticket.texto(SEPARADOR);
    }

    match &descuento {
        None => {
            ticket.texto(&format!("Subtotal: ${}\n", numero(neto, 2)));
            ticket.texto(&format!("IVA (21%): ${}\n", numero(iva, 2)));
        }
        Some((_, _, tipo, _)) if tipo == "percentage" => ticket.texto(&format!("IVA (21%): ${}\n", numero(iva, 2))),
        Some(_) => {}
    }

    ticket.negrita(true);
    ticket.texto(&format!("{}\n", rellenar_izquierda(&format!("TOTAL: ${}", numero(total, 2)), 32)));
    ticket.negrita(false);

    ticket.texto(SEPARADOR);
    let pagos = elementos(&datos["pagos"]);
    if !pagos.is_empty() {
        ticket.texto(if pagos.len() > 1 { "MÉTODOS DE PAGO:\n" } else { "MÉTODO DE PAGO:\n" });
        for pago in pagos {
            let metodo = &pago["metodoPago"];
            let nombre = metodo_pago(&texto(metodo), &o(metodo, "N/A"));
            ticket.texto(&format!("{nombre}: ${}\n", numero(flotante(&pago["monto"]), 2)));
        }
    } else {
        let metodo = o(&datos["metodoPago"], "N/A");
        let nombre = if metodo.eq_ignore_ascii_case("split") { "PAGO MIXTO".into() } else { metodo_pago(&metodo, &metodo) };
        ticket.texto(&format!("Método de pago: {nombre}\n"));
    }

    ticket.texto(SEPARADOR);
    ticket.alinear(CENTRO);
    ticket.texto("COMPROBANTE AUTORIZADO\n");
    ticket.negrita(true);
    ticket.texto(&format!("CAE: {}\n", o(&datos["cae"], "NO DISPONIBLE")));
    ticket.negrita(false);
    let vencimiento = match &datos["fechaVtoCae"] {
        Value::Null => ahora.format("%d/%m/%Y").to_string(),
        valor => vencimiento_cae(&texto(valor)),
    };
    ticket.texto(&format!("Fecha Vto CAE: {vencimiento}\n"));

    let qr = datos["afipQrUrl"].as_str().filter(|url| {
        !vacio(&datos["cae"]) && ["https://www.arca.gob.ar/fe/qr/?p=", "https://www.afip.gob.ar/fe/qr/?p="].iter().any(|prefijo| url.starts_with(prefijo))
    });
    if let Some(url) = qr {
        ticket.avanzar(1);
        ticket.alinear(CENTRO);
        ticket.qr(url, 1, 5);
        ticket.avanzar(1);
    }

    ticket.texto("\n¡Gracias por su compra!\n");
    ticket.texto("Conserve este comprobante\n");
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
    fn igual_que_afip_ticket_printer_php() {
        comparar("afip", |datos| afip(datos, ahora_fija()));
    }

    #[test]
    fn vencimiento_cae_como_strtotime_de_php() {
        let casos = std::fs::read_to_string(ruta("afip/fechas.txt")).unwrap();
        for linea in casos.lines() {
            let (entrada, esperado) = linea.rsplit_once('|').unwrap();
            assert_eq!(vencimiento_cae(entrada), esperado, "{entrada:?}");
        }
    }

    #[test]
    fn nombre_cortado_a_mitad_de_caracter_no_imprime_nada() {
        let datos = serde_json::json!({"total": 10, "items": [{"nombre": "Calabazas añejas", "cantidad": 1, "subtotal": 10}]});
        assert!(afip(&datos, ahora_fija()).is_empty());
    }
}
