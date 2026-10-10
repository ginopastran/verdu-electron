use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{json, Map, Value};

use crate::cierre::{argentina, miles, numero, parsear_fecha, texto_valor, verdadero};

pub const POR_PAGINA: usize = 6;

fn primero_verdadero<'a>(objeto: &'a Value, claves: &[&str]) -> Option<&'a Value> {
    claves.iter().map(|clave| objeto.get(*clave)).find(|valor| verdadero(*valor)).flatten()
}

fn lista(valor: &Value) -> Result<Vec<Value>, String> {
    match valor {
        Value::Null | Value::Bool(false) => Ok(Vec::new()),
        Value::Array(elementos) => Ok(elementos.clone()),
        Value::Number(n) if n.as_f64() == Some(0.0) => Ok(Vec::new()),
        Value::String(s) if s.is_empty() => Ok(Vec::new()),
        _ => Err("(ordenes || []).map is not a function".into()),
    }
}

fn con(objeto: &Value, campos: &[(&str, Value)]) -> Value {
    let mut mapa = objeto.as_object().cloned().unwrap_or_else(Map::new);
    for (clave, valor) in campos {
        mapa.insert((*clave).into(), valor.clone());
    }
    Value::Object(mapa)
}

fn fecha_de(transaccion: &Value) -> Option<DateTime<Utc>> {
    transaccion.get("fecha").and_then(Value::as_str).and_then(parsear_fecha)
}

pub fn unir_transacciones(ordenes: &Value, facturas: Option<&Value>) -> Result<Vec<Value>, String> {
    let mut todas: Vec<Value> = lista(ordenes)?.iter().map(|o| con(o, &[("tipo", json!("orden"))])).collect();
    if let Some(facturas) = facturas {
        for factura in lista(facturas)? {
            let fecha = primero_verdadero(&factura, &["fecha", "fechaEmision", "fechaFactura"])
                .or(factura.get("createdAt"))
                .cloned()
                .unwrap_or(Value::Null);
            let metodo = primero_verdadero(&factura, &["metodoPago", "formaPago"]).cloned().unwrap_or(json!("N/A"));
            todas.push(con(&factura, &[("tipo", json!("facturaB")), ("fecha", fecha), ("metodoPago", metodo)]));
        }
    }
    todas.sort_by(|a, b| {
        let clave = |t: &Value| fecha_de(t).map(|f| f.timestamp_millis());
        match (clave(a), clave(b)) {
            (Some(x), Some(y)) => y.cmp(&x),
            _ => std::cmp::Ordering::Equal,
        }
    });
    Ok(todas)
}

pub fn dia_argentina(fecha: &DateTime<Utc>) -> NaiveDate {
    fecha.with_timezone(&argentina()).date_naive()
}

pub fn del_dia(transacciones: Vec<Value>, ahora: &DateTime<Utc>) -> Result<Vec<Value>, String> {
    let hoy = dia_argentina(ahora);
    let mut filtradas = Vec::new();
    for transaccion in transacciones {
        let fecha = fecha_de(&transaccion).ok_or_else(|| "Invalid time value".to_string())?;
        if dia_argentina(&fecha) == hoy {
            filtradas.push(transaccion);
        }
    }
    Ok(filtradas)
}

pub fn texto_tipo(transaccion: &Value) -> &'static str {
    if transaccion["tipo"] == "facturaB" {
        "Factura B"
    } else {
        "Orden"
    }
}

pub fn capitalizar(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len());
    let mut inicio = true;
    for c in texto.chars() {
        if inicio && c.is_alphabetic() {
            salida.extend(c.to_uppercase());
            inicio = false;
        } else {
            salida.push(c);
            if c.is_whitespace() {
                inicio = true;
            } else if c.is_alphanumeric() {
                inicio = false;
            }
        }
    }
    salida
}

pub fn texto_metodo(transaccion: &Value) -> String {
    match transaccion.get("metodoPago") {
        Some(Value::String(m)) if m == "split" => "Mixto".into(),
        Some(Value::Null) => String::new(),
        None => String::new(),
        Some(otro) => capitalizar(&texto_valor(otro)),
    }
}

pub fn texto_js(valor: Option<&Value>) -> String {
    match valor {
        None => "undefined".into(),
        Some(v) => texto_valor(v),
    }
}

pub fn insignia_descuento(transaccion: &Value) -> Option<(String, Option<String>)> {
    if !verdadero(transaccion.get("tieneDescuento")) {
        return None;
    }
    let tipo = &transaccion["tipoDescuento"];
    let insignia = if tipo == "percentage" || tipo == "porcentual" {
        format!("{}%", texto_js(transaccion.get("valorDescuento")))
    } else {
        format!("${}", miles(numero(transaccion.get("valorDescuento"))))
    };
    let subtotal = verdadero(transaccion.get("subtotalSinDescuento"))
        .then(|| format!("Subtotal: ${}", miles(numero(transaccion.get("subtotalSinDescuento")))));
    Some((insignia, subtotal))
}

pub fn moneda(valor: f64) -> String {
    if valor.is_nan() {
        return "NaN".into();
    }
    let texto = format!("{:.2}", valor.abs());
    let (entera, decimal) = texto.split_once('.').unwrap_or((&texto, "00"));
    let mut agrupada = String::new();
    for (indice, digito) in entera.chars().enumerate() {
        if indice > 0 && (entera.len() - indice) % 3 == 0 {
            agrupada.push('.');
        }
        agrupada.push(digito);
    }
    let signo = if valor < 0.0 && texto.trim_start_matches(['0', '.']).len() > 0 { "-" } else { "" };
    format!("{signo}{agrupada},{decimal}")
}

pub fn total_mostrado(transaccion: &Value) -> String {
    let con_descuento = ["tieneDescuento", "subtotalSinDescuento", "montoDescuento"].iter().all(|c| verdadero(transaccion.get(*c)));
    let total = if con_descuento {
        numero(transaccion.get("subtotalSinDescuento")) - numero(transaccion.get("montoDescuento"))
    } else {
        numero(transaccion.get("total"))
    };
    format!("${}", moneda(total))
}

pub fn linea_descuento(transaccion: &Value) -> Option<String> {
    (verdadero(transaccion.get("tieneDescuento")) && verdadero(transaccion.get("montoDescuento")))
        .then(|| format!("Descuento: -${}", moneda(numero(transaccion.get("montoDescuento")))))
}

pub fn numeros_pagina(actual: usize, total: usize) -> Vec<usize> {
    let maximo = 5usize;
    let mut desde = actual.saturating_sub(maximo / 2).max(1);
    let hasta = total.min(desde + maximo - 1);
    if hasta + 1 < desde + maximo {
        desde = (hasta + 1).saturating_sub(maximo).max(1);
    }
    (desde..=hasta).collect()
}

pub fn total_paginas(cantidad: usize) -> usize {
    cantidad.div_ceil(POR_PAGINA)
}

fn normalizar_item(item: &Value, con_detalle: bool) -> Value {
    let anidado = |padre: &str| item.get(padre).and_then(|p| p.get("nombre")).filter(|v| verdadero(Some(v)));
    let nombre = primero_verdadero(item, &["nombre"])
        .or_else(|| anidado("producto"))
        .or_else(|| primero_verdadero(item, &["producto"]))
        .or_else(|| if con_detalle { anidado("detalle") } else { None })
        .cloned()
        .unwrap_or(json!("Producto"));
    let cantidad = primero_verdadero(item, &["cantidad", "qty", "cantidadProducto"]).cloned().unwrap_or(json!(0));
    let precio = primero_verdadero(item, &["precioHistorico", "precio", "price", "precioUnitario"]).cloned().unwrap_or(json!(0));
    let subtotal = primero_verdadero(item, &["subtotal"]).cloned().unwrap_or_else(|| {
        let producto = numero(Some(&cantidad)) * numero(Some(&precio));
        if producto.is_finite() {
            json!(producto)
        } else {
            Value::Null
        }
    });
    json!({ "nombre": nombre, "cantidad": cantidad, "precioHistorico": precio, "subtotal": subtotal })
}

fn corregir_total_por_descuento(orden: &mut Value) {
    let Some(descuento) = orden.get("discountData").filter(|d| d.is_object()).cloned() else {
        return;
    };
    let valor = numero(descuento.get("value"));
    if !verdadero(descuento.get("type")) || !(valor > 0.0) {
        return;
    }
    let subtotal = if numero(orden.get("subtotal")) > 0.0 {
        numero(orden.get("subtotal"))
    } else if numero(orden.get("subtotalSinDescuento")) > 0.0 {
        numero(orden.get("subtotalSinDescuento"))
    } else {
        numero(orden.get("total"))
    };
    let tipo = &descuento["type"];
    let mut monto = if tipo == "percentage" || tipo == "porcentual" { subtotal * valor / 100.0 } else { valor };
    if numero(descuento.get("amount")) > 0.0 {
        monto = numero(descuento.get("amount"));
    }
    let calculado = subtotal - monto;
    if !((numero(orden.get("total")) - calculado).abs() <= 0.01) {
        orden["total"] = json!(calculado);
    }
}

pub fn para_reimprimir(orden: &Value, nombre_negocio: &str) -> Value {
    let vendedor = match orden.get("vendedor") {
        Some(Value::String(v)) => json!(v),
        Some(Value::Object(v)) => primero_verdadero(&Value::Object(v.clone()), &["nombre", "name"]).cloned().unwrap_or(json!("N/A")),
        _ => json!("N/A"),
    };
    let items: Vec<Value> = match (orden.get("items"), orden.get("detalles")) {
        (Some(Value::Array(items)), _) => items.iter().map(|i| normalizar_item(i, true)).collect(),
        (_, Some(Value::Array(detalles))) => detalles.iter().map(|d| normalizar_item(d, false)).collect(),
        _ => Vec::new(),
    };
    let negocio = primero_verdadero(orden, &["businessName"]).cloned().unwrap_or(json!(nombre_negocio));
    let mut enriquecida = con(orden, &[("vendedor", vendedor), ("items", json!(items)), ("businessName", negocio)]);
    corregir_total_por_descuento(&mut enriquecida);
    enriquecida
}

pub fn nombre_negocio(admin: Option<&Value>) -> String {
    admin
        .and_then(|a| a.get("businessName"))
        .filter(|n| verdadero(Some(n)))
        .map(texto_valor)
        .unwrap_or_else(|| "Verdulería".into())
}

pub fn rango_mostrado(pagina: usize, cantidad: usize) -> String {
    let desde = (pagina - 1) * POR_PAGINA;
    format!("Mostrando {} a {} de {} transacciones", desde + 1, (desde + POR_PAGINA).min(cantidad), cantidad)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn orden(id: i64, fecha: &str) -> Value {
        json!({ "id": id, "fecha": fecha, "metodoPago": "efectivo", "total": 100 })
    }

    #[test]
    fn une_ordenes_y_facturas_ordenadas_por_fecha() {
        let ordenes = json!([orden(1, "2026-10-10T12:00:00.000Z"), orden(2, "2026-10-10T15:00:00.000Z")]);
        let facturas = json!([{ "id": 9, "fechaEmision": "2026-10-10T13:00:00.000Z", "formaPago": "tarjeta" }, { "id": 10, "createdAt": "2026-10-10T11:00:00.000Z" }]);
        let todas = unir_transacciones(&ordenes, Some(&facturas)).unwrap();
        let ids: Vec<i64> = todas.iter().map(|t| t["id"].as_i64().unwrap()).collect();
        assert_eq!(ids, vec![2, 9, 1, 10]);
        assert_eq!(todas[1]["tipo"], "facturaB");
        assert_eq!(todas[1]["metodoPago"], "tarjeta");
        assert_eq!(todas[3]["metodoPago"], "N/A");
        assert_eq!(todas[0]["tipo"], "orden");
    }

    #[test]
    fn respuesta_que_no_es_lista_es_error() {
        assert!(unir_transacciones(&json!({ "error": "x" }), None).is_err());
        assert!(unir_transacciones(&Value::Null, None).unwrap().is_empty());
    }

    #[test]
    fn filtra_por_dia_argentino() {
        let ahora = parsear_fecha("2026-10-10T15:36:00.000Z").unwrap();
        let todas = vec![orden(1, "2026-10-10T02:59:00.000Z"), orden(2, "2026-10-10T03:00:00.000Z"), orden(3, "2026-10-11T02:59:00.000Z")];
        let hoy = del_dia(todas, &ahora).unwrap();
        let ids: Vec<i64> = hoy.iter().map(|t| t["id"].as_i64().unwrap()).collect();
        assert_eq!(ids, vec![2, 3]);
        assert!(del_dia(vec![json!({ "id": 4 })], &ahora).is_err());
    }

    #[test]
    fn textos_de_la_tabla() {
        let con_descuento = json!({ "metodoPago": "efectivo", "total": 7650, "tieneDescuento": true, "tipoDescuento": "porcentual", "valorDescuento": 10, "montoDescuento": 850, "subtotalSinDescuento": 8500 });
        assert_eq!(insignia_descuento(&con_descuento), Some(("10%".into(), Some("Subtotal: $8,500".into()))));
        assert_eq!(total_mostrado(&con_descuento), "$7.650,00");
        assert_eq!(linea_descuento(&con_descuento).as_deref(), Some("Descuento: -$850,00"));
        let fijo = json!({ "tieneDescuento": true, "tipoDescuento": "cantidad", "valorDescuento": 500, "total": 23900 });
        assert_eq!(insignia_descuento(&fijo), Some(("$500".into(), None)));
        assert_eq!(total_mostrado(&fijo), "$23.900,00");
        assert_eq!(texto_metodo(&json!({ "metodoPago": "split" })), "Mixto");
        assert_eq!(texto_metodo(&json!({ "metodoPago": "qr" })), "Qr");
        assert_eq!(texto_metodo(&json!({ "metodoPago": "mercado pago" })), "Mercado Pago");
        assert_eq!(texto_tipo(&json!({ "tipo": "facturaB" })), "Factura B");
        assert_eq!(moneda(1234567.891), "1.234.567,89");
        assert_eq!(moneda(0.0), "0,00");
    }

    #[test]
    fn paginacion() {
        assert_eq!(total_paginas(6), 1);
        assert_eq!(total_paginas(13), 3);
        assert_eq!(numeros_pagina(1, 3), vec![1, 2, 3]);
        assert_eq!(numeros_pagina(1, 9), vec![1, 2, 3, 4, 5]);
        assert_eq!(numeros_pagina(5, 9), vec![3, 4, 5, 6, 7]);
        assert_eq!(numeros_pagina(9, 9), vec![5, 6, 7, 8, 9]);
        assert_eq!(rango_mostrado(2, 8), "Mostrando 7 a 8 de 8 transacciones");
    }

    #[test]
    fn arma_la_orden_para_reimprimir() {
        let original = json!({
            "id": 9021,
            "total": 7650,
            "vendedor": { "nombre": "Lucía Fernández" },
            "items": [
                { "producto": { "nombre": "Tomate" }, "cantidad": 1.25, "precio": 2400 },
                { "nombre": "Huevos", "qty": 1, "precioHistorico": 5500, "subtotal": 5500 }
            ]
        });
        let lista = para_reimprimir(&original, "Verdulería La Huerta");
        assert_eq!(lista["vendedor"], "Lucía Fernández");
        assert_eq!(lista["businessName"], "Verdulería La Huerta");
        assert_eq!(lista["items"][0], json!({ "nombre": "Tomate", "cantidad": 1.25, "precioHistorico": 2400, "subtotal": 3000.0 }));
        assert_eq!(lista["items"][1], json!({ "nombre": "Huevos", "cantidad": 1, "precioHistorico": 5500, "subtotal": 5500 }));
        assert_eq!(lista["id"], 9021);
        let sin_vendedor = para_reimprimir(&json!({ "detalles": [{ "producto": "Papa", "cantidad": 2, "precioUnitario": 900 }], "businessName": "Otra" }), "X");
        assert_eq!(sin_vendedor["vendedor"], "N/A");
        assert_eq!(sin_vendedor["businessName"], "Otra");
        assert_eq!(sin_vendedor["items"][0]["nombre"], "Papa");
        assert_eq!(sin_vendedor["items"][0]["subtotal"], 1800.0);
    }

    #[test]
    fn corrige_el_total_si_el_descuento_no_esta_aplicado() {
        let orden = json!({ "total": 1000, "subtotal": 1000, "discountData": { "type": "percentage", "value": 10, "amount": 0 } });
        assert_eq!(para_reimprimir(&orden, "X")["total"], 900.0);
        let aplicada = json!({ "total": 900, "subtotal": 1000, "discountData": { "type": "percentage", "value": 10, "amount": 100 } });
        assert_eq!(para_reimprimir(&aplicada, "X")["total"], 900);
    }

    #[test]
    fn nombre_del_negocio_para_el_ticket() {
        assert_eq!(nombre_negocio(Some(&json!({ "businessName": "Andex" }))), "Andex");
        assert_eq!(nombre_negocio(None), "Verdulería");
    }
}
