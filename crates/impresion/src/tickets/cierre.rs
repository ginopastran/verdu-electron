use std::path::Path;

use chrono::{Duration, NaiveDate, NaiveDateTime};
use serde_json::Value;

use super::*;
use crate::escpos::{Ticket, CENTRO, IZQUIERDA};

const SEPARADOR: &str = "-----------------------------\n";
const PRINCIPALES: [&str; 3] = ["qr", "tarjeta", "efectivo"];

#[derive(Clone, Copy)]
enum Numero {
    Entero(i64),
    Real(f64),
}

impl Numero {
    fn real(self) -> f64 {
        match self {
            Numero::Entero(entero) => entero as f64,
            Numero::Real(real) => real,
        }
    }
}

fn espacio(byte: &u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn analizar_numero(cadena: &str) -> Option<(Numero, bool)> {
    let bytes = cadena.as_bytes();
    let digitos = |desde: usize| bytes[desde..].iter().take_while(|b| b.is_ascii_digit()).count();
    let inicio = bytes.iter().take_while(|b| espacio(b)).count();
    let mut fin = inicio + usize::from(matches!(bytes.get(inicio), Some(b'+' | b'-')));
    let enteros = digitos(fin);
    fin += enteros;
    let mut real = false;
    if bytes.get(fin) == Some(&b'.') {
        let decimales = digitos(fin + 1);
        if enteros + decimales > 0 {
            fin += 1 + decimales;
            real = true;
        }
    }
    if enteros == 0 && !real {
        return None;
    }
    if matches!(bytes.get(fin), Some(b'e' | b'E')) {
        let signo = usize::from(matches!(bytes.get(fin + 1), Some(b'+' | b'-')));
        let exponente = digitos(fin + 1 + signo);
        if exponente > 0 {
            fin += 1 + signo + exponente;
            real = true;
        }
    }
    let literal = &cadena[inicio..fin];
    let numero = match literal.parse::<i64>() {
        Ok(entero) if !real => Numero::Entero(entero),
        _ => Numero::Real(literal.parse().ok()?),
    };
    Some((numero, bytes[fin..].iter().all(espacio)))
}

fn operando(valor: &Value) -> Option<Numero> {
    match valor {
        Value::Null => Some(Numero::Entero(0)),
        Value::Bool(b) => Some(Numero::Entero(*b as i64)),
        Value::Number(n) => Some(n.as_i64().map_or_else(|| Numero::Real(n.as_f64().unwrap_or(0.0)), Numero::Entero)),
        Value::String(s) => analizar_numero(s).map(|(numero, _)| numero),
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn parametro_real(valor: &Value) -> Option<f64> {
    match valor {
        Value::String(s) => match analizar_numero(s)? {
            (numero, true) => Some(numero.real()),
            _ => None,
        },
        otro => operando(otro).map(Numero::real),
    }
}

fn formato(valor: &Value) -> Option<String> {
    parametro_real(valor).map(|real| numero(real, 2))
}

fn mayor_que_cero(valor: &Value) -> bool {
    match valor {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|real| real > 0.0),
        Value::String(s) => match analizar_numero(s) {
            Some((numero, true)) => numero.real() > 0.0,
            _ => s.as_bytes() > b"0".as_slice(),
        },
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn es_58(valor: &Value) -> bool {
    match valor {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() == Some(58.0),
        Value::String(s) => matches!(analizar_numero(s), Some((numero, true)) if numero.real() == 58.0),
        _ => false,
    }
}

fn real_a_entero(real: f64) -> i64 {
    if !real.is_finite() {
        return 0;
    }
    if (-9.223372036854775808e18..9.223372036854775808e18).contains(&real) {
        return real as i64;
    }
    let dos_a_la_64 = 2f64.powi(64);
    let mut resto = real % dos_a_la_64;
    if resto < 0.0 {
        resto += dos_a_la_64;
    }
    if resto > i64::MAX as f64 {
        resto -= dos_a_la_64;
    }
    resto as i64
}

fn entero(valor: &Value) -> i64 {
    match valor {
        Value::Number(n) => n.as_i64().unwrap_or_else(|| real_a_entero(n.as_f64().unwrap_or(0.0))),
        Value::String(s) => match analizar_numero(s) {
            Some((Numero::Entero(entero), _)) => entero,
            Some((Numero::Real(real), _)) => real as i64,
            None => 0,
        },
        otro => flotante(otro) as i64,
    }
}

fn sumar_texto(valor: &Value, otro: i64) -> Option<String> {
    let suma = match operando(valor)? {
        Numero::Entero(entero) => entero.checked_add(otro).map_or(Numero::Real(entero as f64 + otro as f64), Numero::Entero),
        Numero::Real(real) => Numero::Real(real + otro as f64),
    };
    Some(match suma {
        Numero::Entero(entero) => entero.to_string(),
        Numero::Real(real) => texto(&Value::from(real)),
    })
}

fn mayusculas_estricta(valor: &Value) -> Option<String> {
    (!es_arreglo(valor)).then(|| mayusculas(valor))
}

fn capitalizar(valor: &str) -> String {
    let mut resultado = valor.to_string();
    if let Some(primera) = resultado.get_mut(0..1) {
        primera.make_ascii_uppercase();
    }
    resultado
}

fn rellenar_derecha(valor: &str, ancho: usize) -> String {
    format!("{valor}{}", " ".repeat(ancho.saturating_sub(valor.len())))
}

fn pares(valor: &Value) -> Vec<(String, &Value)> {
    match valor {
        Value::Array(a) => a.iter().enumerate().map(|(indice, v)| (indice.to_string(), v)).collect(),
        Value::Object(o) => o.iter().map(|(clave, v)| (clave.clone(), v)).collect(),
        _ => Vec::new(),
    }
}

fn buscar<'a>(lista: &[(String, &'a Value)], clave: &str) -> Option<&'a Value> {
    lista.iter().find(|(k, _)| k == clave).map(|(_, v)| *v).filter(|v| !v.is_null())
}

fn fecha_php(valor: &Value, ahora: Ahora) -> Option<NaiveDateTime> {
    if es_arreglo(valor) {
        return None;
    }
    let cadena = texto(valor);
    let cadena = cadena.trim_matches([' ', '\t', '\n', '\r']);
    if cadena.is_empty() || cadena.eq_ignore_ascii_case("now") {
        return Some(ahora.naive_local());
    }
    let bytes = cadena.as_bytes();
    let mut i = 0;
    let leer = |i: &mut usize, minimo: usize, maximo: usize| -> Option<u32> {
        let largo = bytes[*i..].iter().take(maximo).take_while(|b| b.is_ascii_digit()).count();
        if largo < minimo {
            return None;
        }
        *i += largo;
        cadena[*i - largo..*i].parse().ok()
    };
    let simbolo = |i: &mut usize, aceptados: &[u8]| -> bool {
        let coincide = bytes.get(*i).is_some_and(|b| aceptados.contains(b));
        *i += coincide as usize;
        coincide
    };
    let anio = leer(&mut i, 4, 4)?;
    simbolo(&mut i, b"-").then_some(())?;
    let mes = leer(&mut i, 1, 2).filter(|m| (1..=12).contains(m))?;
    simbolo(&mut i, b"-").then_some(())?;
    let dia = leer(&mut i, 1, 2).filter(|d| *d <= 31)?;
    let fecha_completa = i == 10;
    let mut segundos = 0;
    let t = matches!(bytes.get(i), Some(b'T' | b't'));
    let separadores = if t { 1 } else { bytes[i..].iter().take_while(|b| matches!(b, b' ' | b'\t')).count() };
    if separadores > 0 && bytes.get(i + separadores).is_some_and(u8::is_ascii_digit) {
        i += separadores;
        let inicio_hora = i;
        let hora = leer(&mut i, 1, 2).filter(|h| *h <= 24)?;
        simbolo(&mut i, b":").then_some(())?;
        let minuto = leer(&mut i, 1, 2).filter(|m| *m <= 59)?;
        let mut segundo = 0;
        if simbolo(&mut i, b":") {
            segundo = leer(&mut i, 1, 2).filter(|s| *s <= 60)?;
            let hora_completa = fecha_completa && i - inicio_hora == 8;
            if (!t || hora_completa) && simbolo(&mut i, b".") {
                leer(&mut i, 1, 9)?;
                i += bytes[i..].iter().take_while(|b| b.is_ascii_digit()).count();
            }
        }
        segundos = (hora * 3600 + minuto * 60 + segundo) as i64;
    }
    i += bytes[i..].iter().take_while(|b| espacio(b)).count();
    let resto = &cadena[i..];
    if resto.eq_ignore_ascii_case("utc") || resto.eq_ignore_ascii_case("gmt") {
        i = bytes.len();
    } else if !simbolo(&mut i, b"Zz") && simbolo(&mut i, b"+-") {
        leer(&mut i, 2, 2)?;
        simbolo(&mut i, b":");
        if bytes.get(i).is_some_and(u8::is_ascii_digit) {
            leer(&mut i, 2, 2)?;
        }
    }
    if !bytes[i..].iter().all(espacio) {
        return None;
    }
    let base = NaiveDate::from_ymd_opt(anio as i32, mes, 1)?.and_hms_opt(0, 0, 0)?;
    Some(base + Duration::days(dia as i64 - 1) + Duration::seconds(segundos))
}

fn formatear_fecha(fecha: NaiveDateTime) -> String {
    fecha.format("%d/%m/%Y %H:%M").to_string()
}

fn linea_metodo(ticket: &mut Ticket, nombre: &str, monto: &Value) -> Option<()> {
    ticket.texto(&rellenar_derecha(nombre, 15));
    ticket.texto(&format!("{}\n", rellenar_izquierda(&format!("${}", formato(monto)?), 17)));
    Some(())
}

fn linea_total(ticket: &mut Ticket, valor: &str) {
    ticket.texto(&format!("{}\n", rellenar_izquierda(valor, 32)));
}

fn metodos_generales(datos: &Value) -> Value {
    let por_metodo = &datos["ventasPorMetodo"];
    let mut metodos = if !por_metodo["ventasPorMetodo"].is_null() {
        match &por_metodo["ventasPorMetodo"] {
            Value::String(s) => serde_json::from_str(s).unwrap_or(Value::Null),
            otro => otro.clone(),
        }
    } else if es_arreglo(por_metodo) {
        por_metodo.clone()
    } else {
        Value::Null
    };
    let respaldo = &datos["_debug"]["originalData"]["ventasPorMetodo"];
    if vacio(&metodos) && !respaldo.is_null() {
        metodos = respaldo.clone();
    }
    metodos
}

fn vendedor_cuenta_corriente(ticket: &mut Ticket, vendedor: &Value) -> Option<()> {
    ticket.negrita(true);
    ticket.texto(&format!("{}\n", mayusculas_estricta(&vendedor["nombre"])?));
    ticket.negrita(false);
    if !vacio(&vendedor["email"]) {
        ticket.texto(&format!("Email: {}\n", texto(&vendedor["email"])));
    }
    let metodos = pares(&vendedor["metodosPago"]);
    if metodos.iter().any(|(_, monto)| mayor_que_cero(monto)) {
        for (metodo, monto) in metodos.iter().filter(|(_, monto)| mayor_que_cero(monto)) {
            ticket.texto(&format!("{}: ${}\n", capitalizar(&metodo.replace('_', " ")), numero(flotante(monto), 2)));
        }
    }
    ticket.texto(&format!("Total: ${}\n", formato(&vendedor["totalVentas"])?));
    if !vendedor["cantidadVentas"].is_null() {
        ticket.texto(&format!("Cantidad: {}\n", texto(&vendedor["cantidadVentas"])));
    }
    ticket.texto(SEPARADOR);
    Some(())
}

fn vendedor_general(ticket: &mut Ticket, vendedor: &Value) -> Option<()> {
    ticket.negrita(true);
    ticket.texto(&format!("{}\n", mayusculas_estricta(&vendedor["nombre"])?));
    ticket.negrita(false);
    if !vacio(&vendedor["email"]) {
        ticket.texto(&format!("Email: {}\n", texto(&vendedor["email"])));
    }
    let decodificado;
    let mut metodos = Vec::new();
    let mut tiene_metodos = false;
    if !vendedor["metodosPago"].is_null() {
        let mut origen = &vendedor["metodosPago"];
        if let Some(cadena) = origen.as_str().filter(|s| s.starts_with('{')) {
            decodificado = serde_json::from_str(cadena).unwrap_or(Value::Null);
            origen = &decodificado;
        }
        if es_arreglo(origen) {
            metodos = pares(origen);
            tiene_metodos = metodos.iter().any(|(_, monto)| mayor_que_cero(monto));
        }
    } else {
        for clave in PRINCIPALES {
            if mayor_que_cero(&vendedor[clave]) {
                metodos.push((clave.to_string(), &vendedor[clave]));
                tiene_metodos = true;
            }
        }
    }
    if tiene_metodos && !metodos.is_empty() {
        for (clave, nombre) in [("qr", "QR"), ("tarjeta", "Tarjeta"), ("efectivo", "Efectivo")] {
            if let Some(monto) = buscar(&metodos, clave).filter(|monto| mayor_que_cero(monto)) {
                ticket.texto(&format!("{nombre}: ${}\n", numero(flotante(monto), 2)));
            }
        }
        for (metodo, monto) in &metodos {
            if !PRINCIPALES.contains(&metodo.as_str()) && mayor_que_cero(monto) {
                ticket.texto(&format!("{}: ${}\n", capitalizar(&metodo.replace('_', " ")), numero(flotante(monto), 2)));
            }
        }
    }
    let total = &vendedor["totalVentas"];
    if !total.is_null() {
        ticket.texto(&format!("Total: ${}\n", numero(flotante(total), 2)));
    }
    let total_con_cuenta_corriente = match &vendedor["totalVentasConCuentaCorriente"] {
        Value::Null => flotante(total),
        valor => flotante(valor),
    };
    ticket.texto(&format!("Total con CC: ${}\n", numero(total_con_cuenta_corriente, 2)));
    if !vendedor["cantidadVentas"].is_null() {
        ticket.texto(&format!("Cantidad: {}\n", texto(&vendedor["cantidadVentas"])));
    }
    ticket.texto(SEPARADOR);
    Some(())
}

fn imprimir(datos: &Value, logo_defecto: &Path, ahora: Ahora) -> Option<Vec<u8>> {
    let mut ticket = Ticket::new();
    let ancho_logo = if es_58(&datos["anchoPapel"]) { 370 } else { 556 };
    if let Some(imagen) = logo(datos, logo_defecto, ancho_logo) {
        ticket.imagen(&imagen);
    }

    ticket.alinear(CENTRO);
    ticket.negrita(true);
    ticket.tamano(2, 2);
    ticket.texto("Cierre de Caja\n");
    ticket.tamano(1, 1);
    let periodo = &datos["periodo"];
    let manual = periodo.as_str() == Some("custom");
    let periodo_texto = if manual { "MANUAL".to_string() } else { mayusculas_estricta(periodo)? };
    ticket.texto(&format!("Periodo: {periodo_texto}\n\n"));

    ticket.alinear(IZQUIERDA);
    let inicio = fecha_php(&datos["fechaInicio"], ahora)?;
    let fin = fecha_php(&datos["fechaCierre"], ahora)?;
    if manual {
        ticket.texto(&format!("Desde: {}\n", formatear_fecha(inicio)));
        ticket.texto(&format!("Hasta: {}\n", formatear_fecha(fin)));
    } else {
        ticket.texto(&format!("Fecha inicio: {}\n", formatear_fecha(inicio)));
        ticket.texto(&format!("Fecha cierre: {}\n", formatear_fecha(fin)));
    }
    ticket.texto(SEPARADOR);

    let metodos_origen = metodos_generales(datos);
    let origen = pares(&metodos_origen);
    let cero = Value::from(0);
    let mut metodos: Vec<(String, &Value)> =
        ["efectivo", "tarjeta", "qr"].map(|clave| (clave.to_string(), buscar(&origen, clave).unwrap_or(&cero))).into();
    for (clave, monto) in &origen {
        if !metodos.iter().any(|(k, _)| k == clave) {
            metodos.push((clave.clone(), monto));
        }
    }
    for (_, monto) in &metodos {
        operando(monto)?;
    }
    operando(&datos["totalVentas"])?;

    if metodos.iter().any(|(_, monto)| mayor_que_cero(monto)) {
        ticket.texto("VENTAS POR MÉTODO DE PAGO:\n");
        ticket.texto(SEPARADOR);
        for clave in PRINCIPALES {
            let monto = buscar(&metodos, clave).unwrap_or(&cero);
            if mayor_que_cero(monto) {
                linea_metodo(&mut ticket, &capitalizar(clave), monto)?;
            }
        }
        for (clave, monto) in &metodos {
            if !PRINCIPALES.contains(&clave.as_str()) && mayor_que_cero(monto) {
                linea_metodo(&mut ticket, &capitalizar(clave), monto)?;
            }
        }
    }

    let cuenta_corriente_ventas = &datos["ventas"]["cuentaCorriente"];
    let cuenta_corriente_raiz = &datos["cuentaCorriente"];
    let mut metodos_cuenta_corriente = &Value::Null;
    let cuenta_corriente = if es_arreglo(cuenta_corriente_ventas) {
        cuenta_corriente_ventas
    } else if es_arreglo(cuenta_corriente_raiz) {
        cuenta_corriente_raiz
    } else {
        &datos["ventasCuentaCorriente"]
    };
    let total_cuenta_corriente = flotante(&cuenta_corriente["total"]);
    let cantidad_cuenta_corriente = entero(&cuenta_corriente["cantidad"]);
    if es_arreglo(cuenta_corriente_ventas) || es_arreglo(cuenta_corriente_raiz) {
        if es_arreglo(&cuenta_corriente["ventasPorMetodo"]) {
            metodos_cuenta_corriente = &cuenta_corriente["ventasPorMetodo"];
        }
    }

    ticket.texto(SEPARADOR);
    ticket.negrita(true);
    linea_total(&mut ticket, &format!("TOTAL VENTAS NORMALES: ${}", formato(&datos["totalVentas"])?));
    linea_total(&mut ticket, &format!("CANT. VENTAS NORMALES: {}", texto(&datos["cantidadVentas"])));
    ticket.negrita(false);

    if total_cuenta_corriente > 0.0 && cantidad_cuenta_corriente > 0 {
        ticket.texto(SEPARADOR);
        ticket.alinear(CENTRO);
        ticket.negrita(true);
        ticket.texto("VENTAS POR CUENTA CORRIENTE\n");
        ticket.negrita(false);
        ticket.alinear(IZQUIERDA);
        ticket.texto(SEPARADOR);

        let metodos = pares(metodos_cuenta_corriente);
        if metodos.iter().any(|(_, monto)| mayor_que_cero(monto)) {
            ticket.texto("MÉTODOS DE PAGO:\n");
            for (metodo, monto) in metodos.iter().filter(|(_, monto)| mayor_que_cero(monto)) {
                linea_metodo(&mut ticket, &capitalizar(&metodo.replace('_', " ")), monto)?;
            }
            ticket.texto(SEPARADOR);
        }

        let vendedores_ventas = &cuenta_corriente_ventas["ventasPorVendedor"];
        let vendedores = if es_arreglo(vendedores_ventas) { vendedores_ventas } else { &cuenta_corriente_raiz["ventasPorVendedor"] };
        let validos: Vec<&Value> = elementos(vendedores)
            .into_iter()
            .filter(|vendedor| !vacio(&vendedor["nombre"]) && flotante(&vendedor["totalVentas"]) > 0.0)
            .collect();
        if !validos.is_empty() {
            ticket.texto("VENTAS POR VENDEDOR:\n");
            for vendedor in validos {
                vendedor_cuenta_corriente(&mut ticket, vendedor)?;
            }
        }

        linea_total(&mut ticket, &format!("TOTAL CUENTA CORRIENTE: ${}", numero(total_cuenta_corriente, 2)));
        linea_total(&mut ticket, &format!("CANT. VENTAS CC: {cantidad_cuenta_corriente}"));
        ticket.texto(SEPARADOR);
    }

    if total_cuenta_corriente > 0.0 {
        let resumen = primero([&datos["ventas"]["resumen"]["totalCombinado"], &datos["resumen"]["totalCombinado"]]);
        let total_combinado = match resumen {
            Value::Null => operando(&datos["totalVentas"])?.real() + total_cuenta_corriente,
            valor => flotante(valor),
        };
        let cantidad = primero([&datos["ventas"]["resumen"]["cantidadTotalCombinada"], &datos["resumen"]["cantidadTotalCombinada"]]);
        let cantidad_combinada = match cantidad {
            Value::Null => sumar_texto(&datos["cantidadVentas"], cantidad_cuenta_corriente)?,
            valor => entero(valor).to_string(),
        };
        ticket.texto(SEPARADOR);
        ticket.alinear(CENTRO);
        ticket.negrita(true);
        ticket.texto("TOTAL GENERAL\n");
        ticket.negrita(false);
        ticket.alinear(IZQUIERDA);
        ticket.texto("(Cuenta Corriente + Ventas Normales)\n");
        ticket.texto(SEPARADOR);
        ticket.negrita(true);
        linea_total(&mut ticket, &format!("TOTAL GENERAL: ${}", numero(total_combinado, 2)));
        linea_total(&mut ticket, &format!("CANT. TOTAL: {cantidad_combinada}"));
        ticket.negrita(false);
    }

    let vendedores = &datos["ventasPorVendedor"];
    let validos: Vec<&Value> = elementos(vendedores)
        .into_iter()
        .filter(|vendedor| {
            !vacio(&vendedor["nombre"]) && !vendedor["totalVentas"].is_null() && flotante(&vendedor["totalVentas"]) >= 0.0
        })
        .collect();
    if !validos.is_empty() {
        ticket.texto("\n\n");
        ticket.alinear(CENTRO);
        ticket.negrita(true);
        ticket.texto("VENTAS POR VENDEDOR\n");
        ticket.negrita(false);
        ticket.texto("=============================\n\n");
        ticket.alinear(IZQUIERDA);
        for vendedor in validos {
            vendedor_general(&mut ticket, vendedor)?;
        }
    }

    ticket.alinear(CENTRO);
    ticket.texto(&format!("\n\n{}\n", ahora.format("%d/%m/%Y %H:%M:%S")));
    ticket.texto("Gracias por su trabajo\n");
    ticket.avanzar(3);
    ticket.cortar();
    Some(ticket.bytes())
}

pub fn cierre(datos: &Value, logo_defecto: &Path, ahora: Ahora) -> Vec<u8> {
    imprimir(datos, logo_defecto, ahora).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::pruebas::*;
    use super::*;

    #[test]
    fn sin_ticket_cuando_php_corta() {
        let casos = [
            json!({"periodo": "todo", "fechaInicio": "basura", "fechaCierre": "", "totalVentas": 1}),
            json!({"periodo": "todo", "fechaInicio": "2025-13-01", "fechaCierre": "", "totalVentas": 1}),
            json!({"periodo": "todo", "fechaInicio": "2025-1-5T07:08:09.5Z", "fechaCierre": "", "totalVentas": 1}),
            json!({"periodo": ["x"], "fechaInicio": "", "fechaCierre": "", "totalVentas": 1}),
            json!({"periodo": "todo", "fechaInicio": "", "fechaCierre": "", "totalVentas": "abc"}),
            json!({"periodo": "todo", "fechaInicio": "", "fechaCierre": "", "totalVentas": 1, "ventasPorMetodo": {"qr": "12abc"}}),
            json!({"periodo": "todo", "fechaInicio": "", "fechaCierre": "", "totalVentas": 1, "ventasPorVendedor": [{"nombre": ["x"], "totalVentas": 1}]}),
        ];
        for datos in casos {
            assert!(cierre(&datos, Path::new("no-existe.png"), ahora_fija()).is_empty(), "{datos}");
        }
    }

    #[test]
    fn fecha_inicio_con_la_hora_real_del_cierre() {
        let datos = json!({"periodo": "tarde", "fechaInicio": "2026-10-10T09:37:00.619Z", "fechaCierre": "2026-10-10T20:15:00.000Z", "totalVentas": 1, "cantidadVentas": 1});
        let texto = String::from_utf8_lossy(&cierre(&datos, Path::new("no-existe.png"), ahora_fija())).into_owned();
        assert!(texto.contains("Fecha inicio: 10/10/2026 09:37\n"), "{texto}");
        assert!(texto.contains("Fecha cierre: 10/10/2026 20:15\n"), "{texto}");
    }

    #[test]
    fn igual_que_closing_printer_php() {
        comparar("cierre", |datos| cierre(datos, &logo_defecto(), ahora_fija()));
    }
}
