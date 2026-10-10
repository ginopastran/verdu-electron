use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};
use serde_json::{json, Map, Value};

use crate::api::{codificar, Cliente};
use crate::cierre::argentina;
use crate::formato;
use crate::iva::precio_visual_con_iva;

pub fn total_pagado(factura: &Value) -> f64 {
    redondear2(
        factura["pagos"]
            .as_array()
            .map_or(0.0, |pagos| pagos.iter().fold(0.0, |suma, pago| suma + pago["monto"].as_f64().unwrap_or(0.0))),
    )
}

pub fn saldo_pendiente(factura: &Value) -> f64 {
    redondear2(factura["total"].as_f64().unwrap_or(0.0) - total_pagado(factura))
}

pub fn porcentaje_pagado(factura: &Value) -> f64 {
    let total = factura["total"].as_f64().unwrap_or(0.0);
    if total > 0.0 {
        total_pagado(factura) / total * 100.0
    } else {
        0.0
    }
}

pub fn tiene_pagos(factura: &Value) -> bool {
    factura["pagos"].as_array().is_some_and(|p| !p.is_empty())
}

pub fn saldo_para_pago(factura: &Value) -> f64 {
    match factura["saldoPendiente"].as_f64() {
        Some(saldo) if saldo.is_finite() => redondear2(saldo).max(0.0),
        _ => saldo_pendiente(factura).max(0.0),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EstadoTarjeta {
    SegunBackend { pagada: bool, texto: String },
    Pagada,
    Parcial,
    Impaga,
}

pub fn estado_tarjeta(factura: &Value) -> EstadoTarjeta {
    if !tiene_pagos(factura) {
        let estado = factura["estado"].as_str().unwrap_or_default().to_string();
        return EstadoTarjeta::SegunBackend { pagada: estado == "pagada", texto: estado };
    }
    let pagado = total_pagado(factura);
    if saldo_pendiente(factura) <= 0.0 {
        EstadoTarjeta::Pagada
    } else if pagado > 0.0 {
        EstadoTarjeta::Parcial
    } else {
        EstadoTarjeta::Impaga
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EstadoHistorial {
    Pagada,
    Parcial,
    Pendiente,
}

pub fn estado_historial(factura: &Value) -> EstadoHistorial {
    if saldo_pendiente(factura) <= 0.0 {
        EstadoHistorial::Pagada
    } else if total_pagado(factura) > 0.0 {
        EstadoHistorial::Parcial
    } else {
        EstadoHistorial::Pendiente
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResumenCuenta {
    pub saldo: f64,
    pub con_saldo: usize,
    pub cobrado_mes: f64,
}

pub fn resumen_cuenta(facturas: &[Value], hoy: NaiveDate) -> ResumenCuenta {
    let mut resumen = ResumenCuenta::default();
    for factura in facturas.iter().filter(|f| f["estado"] != "anulada") {
        let saldo = saldo_pendiente(factura);
        if saldo > 0.0 {
            resumen.saldo += saldo;
            resumen.con_saldo += 1;
        }
        for pago in factura["pagos"].as_array().into_iter().flatten() {
            let dia = pago["fecha"].as_str().and_then(dia_argentino);
            if dia.is_some_and(|d| d.year() == hoy.year() && d.month() == hoy.month()) {
                resumen.cobrado_mes += pago["monto"].as_f64().unwrap_or(0.0);
            }
        }
    }
    resumen.saldo = redondear2(resumen.saldo);
    resumen.cobrado_mes = redondear2(resumen.cobrado_mes);
    resumen
}

pub fn nombre_cliente(cliente: &Value) -> String {
    match cliente["razonSocial"].as_str().filter(|r| !r.is_empty()) {
        Some(razon) => razon.to_string(),
        None => format!(
            "{} {}",
            cliente["nombre"].as_str().unwrap_or("undefined"),
            cliente["apellido"].as_str().unwrap_or_default()
        )
        .trim()
        .to_string(),
    }
}

pub fn nombre_o_razon(cliente: &Value) -> Option<String> {
    cliente["nombre"]
        .as_str()
        .filter(|n| !n.is_empty())
        .or(cliente["razonSocial"].as_str().filter(|r| !r.is_empty()))
        .map(str::to_string)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Detalle {
    pub producto_id: i64,
    pub producto: Option<Value>,
    pub descripcion: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

impl Detalle {
    pub fn nuevo(producto_id: i64, producto: Option<Value>, descripcion: String, cantidad: f64, precio: f64) -> Self {
        Detalle { producto_id, producto, descripcion, cantidad, precio_unitario: precio, subtotal: precio * cantidad }
    }

    pub fn recalcular(&mut self) {
        self.subtotal = self.cantidad * self.precio_unitario;
    }

    pub fn a_json(&self) -> Value {
        let mut mapa = Map::new();
        mapa.insert("productoId".into(), json!(self.producto_id));
        if let Some(producto) = &self.producto {
            mapa.insert("producto".into(), producto.clone());
        }
        mapa.insert("descripcion".into(), json!(self.descripcion));
        mapa.insert("cantidad".into(), json!(self.cantidad));
        mapa.insert("precioUnitario".into(), json!(self.precio_unitario));
        mapa.insert("subtotal".into(), json!(self.subtotal));
        Value::Object(mapa)
    }
}

pub fn iva_visible(tipo_factura: &str, iva_incluido_en_precios: Option<bool>) -> Option<bool> {
    iva_incluido_en_precios.filter(|_| tipo_factura != "remito")
}

pub fn precio_con_iva(producto: Option<&Value>, precio: f64, iva_incluido_en_precios: Option<bool>) -> f64 {
    match iva_incluido_en_precios {
        Some(en_precios) => precio_visual_con_iva(
            precio,
            producto.is_some_and(|p| p["ivaIncluido"].as_bool().unwrap_or(false)),
            producto.and_then(|p| p["ivaPorcentaje"].as_f64()),
            en_precios,
        ),
        None => precio,
    }
}

pub fn precio_sin_iva(producto: Option<&Value>, precio_con_iva_visible: f64, iva_incluido_en_precios: Option<bool>) -> f64 {
    precio_con_iva_visible / precio_con_iva(producto, 1.0, iva_incluido_en_precios)
}

pub fn texto_precio_con_iva(producto: Option<&Value>, precio: f64, iva_incluido_en_precios: Option<bool>) -> String {
    texto_entrada(redondear2(precio_con_iva(producto, precio, iva_incluido_en_precios)))
}

pub fn precio_con_lista(producto: &Value, precio_lista: Option<f64>) -> f64 {
    precio_lista
        .filter(|precio| *precio != 0.0 && precio.is_finite())
        .unwrap_or_else(|| producto["precio"].as_f64().unwrap_or(0.0))
}

pub fn cuerpo_edicion(factura: &Value, estado: &str, observaciones: &str) -> Option<Value> {
    let mut cuerpo = Map::new();
    if estado != factura["estado"].as_str().unwrap_or_default() {
        cuerpo.insert("estado".into(), json!(estado));
    }
    if observaciones != factura["observaciones"].as_str().unwrap_or_default() {
        cuerpo.insert("observaciones".into(), json!(observaciones));
    }
    (!cuerpo.is_empty()).then_some(Value::Object(cuerpo))
}

pub fn nombre_pdf(factura: &Value) -> String {
    let limpiar = |texto: &str| -> String {
        texto.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
    };
    let numero = match &factura["numero"] {
        Value::String(numero) => numero.clone(),
        Value::Null => String::new(),
        otro => otro.to_string(),
    };
    format!("factura-{}-{}.pdf", limpiar(factura["tipoFactura"].as_str().unwrap_or_default()), limpiar(&numero))
}

pub fn descargar_pdf(api: &Cliente, factura: &Value, carpeta: &Path) -> Result<PathBuf, String> {
    let id = match &factura["id"] {
        Value::String(id) => id.clone(),
        otro => otro.to_string(),
    };
    let (estado, bytes) = api.bytes(&format!("/api/facturas/generar-pdf?id={}", codificar(&id))).map_err(|e| e.mensaje)?;
    if !(200..300).contains(&estado) || !bytes.starts_with(b"%PDF") {
        let error = serde_json::from_slice::<Value>(&bytes).ok().and_then(|v| v["error"].as_str().map(str::to_string));
        return Err(error.unwrap_or_else(|| format!("el servidor respondió {estado}")));
    }
    std::fs::create_dir_all(carpeta).map_err(|e| e.to_string())?;
    let ruta = carpeta.join(nombre_pdf(factura));
    std::fs::write(&ruta, bytes).map_err(|e| e.to_string())?;
    Ok(ruta)
}

pub fn redondear_js(valor: f64) -> f64 {
    let piso = valor.floor();
    if valor - piso >= 0.5 {
        piso + 1.0
    } else {
        piso
    }
}

pub fn redondear2(valor: f64) -> f64 {
    redondear_js(valor * 100.0) / 100.0
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Totales {
    pub subtotal: f64,
    pub impuestos: f64,
    pub total: f64,
}

pub fn calcular_totales(detalles: &[Detalle], tipo_factura: &str, iva_incluido_en_precios: bool) -> Totales {
    let es_remito = tipo_factura == "remito";
    let mut subtotal_sin_iva = 0.0;
    let mut total_iva = 0.0;
    for detalle in detalles {
        if es_remito {
            subtotal_sin_iva += detalle.subtotal;
            continue;
        }
        let producto = detalle.producto.as_ref();
        let porcentaje = producto.and_then(|p| p["ivaPorcentaje"].as_f64()).unwrap_or(21.0);
        let iva_incluido =
            producto.is_some_and(|p| p["ivaIncluido"].as_bool().unwrap_or(false)) || iva_incluido_en_precios;
        if iva_incluido {
            let base = redondear2(detalle.subtotal / (1.0 + porcentaje / 100.0));
            subtotal_sin_iva += base;
            total_iva += redondear2(detalle.subtotal - base);
        } else {
            subtotal_sin_iva += detalle.subtotal;
            total_iva += redondear2(detalle.subtotal * (porcentaje / 100.0));
        }
    }
    let subtotal = redondear2(subtotal_sin_iva);
    let impuestos = if es_remito { 0.0 } else { redondear2(total_iva) };
    Totales { subtotal, impuestos, total: redondear2(subtotal + impuestos) }
}

pub struct DatosFactura<'a> {
    pub cliente_id: Value,
    pub tipo_factura: &'a str,
    pub observaciones: &'a str,
    pub lista_precio_id: Option<i64>,
    pub pago_inicial: Option<f64>,
    pub fecha_vencimiento: Option<&'a str>,
    pub detalles: &'a [Detalle],
}

pub fn usar_afip(negocio: &Value, tipo_factura: &str) -> bool {
    negocio["afipHabilitado"].as_bool().unwrap_or(false) && tipo_factura != "remito"
}

fn verdadero(valor: &Value) -> bool {
    match valor {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

pub fn cuerpo_factura(datos: &DatosFactura, afip: bool, totales: Totales, negocio: &Value) -> (String, Value) {
    let mut mapa = Map::new();
    if !datos.cliente_id.is_null() {
        mapa.insert("clienteId".into(), datos.cliente_id.clone());
    }
    mapa.insert("tipoFactura".into(), json!(datos.tipo_factura));
    mapa.insert("observaciones".into(), json!(datos.observaciones));
    if let Some(id) = datos.lista_precio_id {
        mapa.insert("listaPrecioId".into(), json!(id));
    }
    let pago_inicial = datos.pago_inicial.filter(|p| *p != 0.0 && !p.is_nan()).unwrap_or(0.0);
    mapa.insert("pagoInicial".into(), json!(pago_inicial));
    let ruta = if afip {
        mapa.insert("esConsumidorFinal".into(), json!(false));
        let sucursal = &negocio["sucursalId"];
        mapa.insert("sucursalId".into(), if verdadero(sucursal) { sucursal.clone() } else { json!(1) });
        let productos: Vec<Value> = datos
            .detalles
            .iter()
            .map(|d| json!({ "productoId": d.producto_id, "cantidad": d.cantidad, "precio": d.precio_unitario }))
            .collect();
        mapa.insert("productos".into(), Value::Array(productos));
        "/api/facturas/crear-afip"
    } else {
        mapa.insert("subtotal".into(), json!(totales.subtotal));
        mapa.insert("impuestos".into(), json!(totales.impuestos));
        mapa.insert("total".into(), json!(totales.total));
        let detalles: Vec<Value> = datos
            .detalles
            .iter()
            .map(|d| {
                json!({
                    "productoId": d.producto_id,
                    "descripcion": d.descripcion,
                    "cantidad": d.cantidad,
                    "precioUnitario": d.precio_unitario,
                    "subtotal": d.subtotal,
                })
            })
            .collect();
        mapa.insert("detalles".into(), Value::Array(detalles));
        "/api/facturas"
    };
    mapa.insert("esCuentaCorriente".into(), json!(true));
    if let Some(fecha) = datos.fecha_vencimiento.filter(|f| !f.is_empty()) {
        mapa.insert("fechaVencimiento".into(), json!(fecha));
    }
    (ruta.to_string(), Value::Object(mapa))
}

pub fn nombre_negocio_ticket(negocio: &Value, factura: &Value) -> String {
    [
        &negocio["nombre"],
        &negocio["name"],
        &negocio["razonSocial"],
        &factura["businessName"],
        &factura["sucursal"],
        &factura["business"]["nombre"],
        &factura["business"]["name"],
        &factura["business"]["razonSocial"],
    ]
    .into_iter()
    .find(|v| verdadero(v))
    .map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string()))
    .unwrap_or_else(|| "Mi Negocio".to_string())
}

pub fn datos_impresion(factura: &Value, negocio: &Value) -> Value {
    let nombre = nombre_negocio_ticket(negocio, factura);
    let mut mapa = factura.as_object().cloned().unwrap_or_default();
    let primero = |a: &Value, b: Value| if verdadero(a) { a.clone() } else { b };
    mapa.insert("businessName".into(), json!(nombre));
    mapa.insert("nombre".into(), primero(&negocio["nombre"], json!(nombre)));
    mapa.insert("razonSocial".into(), primero(&negocio["razonSocial"], json!(nombre)));
    mapa.insert("cuit".into(), primero(&negocio["cuit"], factura["cuit"].clone()));
    mapa.insert("telefono".into(), primero(&negocio["telefono"], factura["telefono"].clone()));
    Value::Object(mapa)
}

pub fn cuerpo_pago(factura_id: &Value, monto: f64, metodo: &str, fecha: &str, referencia: &str) -> Value {
    let mut cuerpo = json!({ "facturaId": factura_id, "monto": monto, "metodoPago": metodo, "fecha": fecha });
    if !referencia.is_empty() {
        cuerpo["referencia"] = json!(referencia);
    }
    cuerpo
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorPago {
    MontoInvalido,
    MontoMayor(f64),
}

impl std::fmt::Display for ErrorPago {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorPago::MontoInvalido => f.write_str("El monto debe ser un número válido mayor a 0"),
            ErrorPago::MontoMayor(maximo) => {
                write!(f, "El monto no puede ser mayor al saldo pendiente ({})", formato::pesos(*maximo))
            }
        }
    }
}

pub fn validar_pago(monto: &str, maximo: f64) -> Result<f64, ErrorPago> {
    let monto = parsear_numero(monto).filter(|m| *m > 0.0).ok_or(ErrorPago::MontoInvalido)?;
    if monto > maximo {
        return Err(ErrorPago::MontoMayor(maximo));
    }
    Ok(monto)
}

pub fn parsear_float(texto: &str) -> Option<f64> {
    let texto = texto.trim_start();
    let mut fin = 0;
    let bytes = texto.as_bytes();
    let mut punto = false;
    let mut digitos = false;
    if fin < bytes.len() && (bytes[fin] == b'-' || bytes[fin] == b'+') {
        fin += 1;
    }
    while fin < bytes.len() {
        match bytes[fin] {
            b'0'..=b'9' => digitos = true,
            b'.' if !punto => punto = true,
            _ => break,
        }
        fin += 1;
    }
    if !digitos {
        return None;
    }
    let mut numero = &texto[..fin];
    if fin < bytes.len() && (bytes[fin] == b'e' || bytes[fin] == b'E') {
        let mut exponente = fin + 1;
        if exponente < bytes.len() && (bytes[exponente] == b'-' || bytes[exponente] == b'+') {
            exponente += 1;
        }
        let inicio = exponente;
        while exponente < bytes.len() && bytes[exponente].is_ascii_digit() {
            exponente += 1;
        }
        if exponente > inicio {
            numero = &texto[..exponente];
        }
    }
    numero.parse().ok()
}

pub fn paginas_visibles(pagina: u32, total_paginas: u32) -> Vec<u32> {
    if total_paginas <= 5 {
        (1..=total_paginas).collect()
    } else if pagina <= 3 {
        (1..=5).collect()
    } else if pagina >= total_paginas - 2 {
        (total_paginas - 4..=total_paginas).collect()
    } else {
        (pagina - 2..=pagina + 2).collect()
    }
}

fn plural(n: usize, singular: &str, plural: &str) -> String {
    if n == 1 { singular } else { plural }.to_string()
}

pub fn resumen_clientes(cargando: bool, buscando: bool, total: usize) -> String {
    if cargando {
        return "Cargando...".into();
    }
    if buscando {
        format!("{total} resultado{} encontrado{}", plural(total, "", "s"), plural(total, "", "s"))
    } else {
        format!("{total} cliente{} total{}", plural(total, "", "s"), plural(total, "", "es"))
    }
}

pub fn resumen_productos(cargando: bool, buscando: bool, total: usize) -> String {
    if cargando {
        return "Cargando...".into();
    }
    let s = plural(total, "", "s");
    if buscando {
        format!("{total} producto{s} encontrado{s}")
    } else {
        format!("{total} producto{s} disponible{s}")
    }
}

pub fn filtrar_productos(productos: &[Value], busqueda: &str, excluidos: &[i64]) -> Vec<Value> {
    let busqueda = busqueda.trim().to_lowercase();
    productos
        .iter()
        .filter(|p| !excluidos.contains(&p["id"].as_i64().unwrap_or(i64::MIN)))
        .filter(|p| {
            busqueda.is_empty()
                || p["nombre"].as_str().unwrap_or_default().to_lowercase().contains(&busqueda)
                || p["plu"].as_str().is_some_and(|plu| !plu.is_empty() && plu.to_lowercase().contains(&busqueda))
        })
        .cloned()
        .collect()
}

pub fn parsear_numero(texto: &str) -> Option<f64> {
    if texto.contains(',') {
        parsear_float(&texto.replace('.', "").replace(',', "."))
    } else {
        parsear_float(texto)
    }
}

pub fn texto_entrada(valor: f64) -> String {
    if valor.is_finite() {
        format!("{valor}").replace('.', ",")
    } else {
        String::new()
    }
}

pub fn dia_argentino(texto: &str) -> Option<NaiveDate> {
    if let Ok(dia) = NaiveDate::parse_from_str(texto.trim(), "%Y-%m-%d") {
        return Some(dia);
    }
    let instante = DateTime::parse_from_rfc3339(texto.trim()).ok()?.with_timezone(&Utc);
    if Some(instante.time()) == NaiveTime::from_hms_opt(0, 0, 0) {
        return Some(instante.date_naive());
    }
    Some(instante.with_timezone(&argentina()).date_naive())
}

pub fn fecha_es_ar(texto: &str) -> String {
    dia_argentino(texto).map_or_else(|| "-".into(), |dia| dia.format("%d/%m/%Y").to_string())
}

pub fn hoy_argentina() -> NaiveDate {
    Utc::now().with_timezone(&argentina()).date_naive()
}

pub fn fecha_iso_desde_input(texto: &str) -> Option<String> {
    let fecha = NaiveDate::parse_from_str(texto, "%Y-%m-%d").ok()?;
    Some(format!("{}T00:00:00.000Z", fecha.format("%Y-%m-%d")))
}

pub fn fecha_dia_mes_anio(texto: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(texto.trim(), "%d/%m/%Y").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factura_con_pagos(total: f64, montos: &[f64]) -> Value {
        let pagos: Vec<Value> = montos.iter().map(|m| json!({ "monto": m })).collect();
        json!({ "total": total, "estado": "pendiente", "pagos": pagos })
    }

    #[test]
    fn saldos_y_estado_toleran_errores_de_coma_flotante() {
        let factura = factura_con_pagos(63925.62, &[40000.0, 23925.62]);
        assert_eq!(saldo_pendiente(&factura), 0.0);
        assert_eq!(estado_tarjeta(&factura), EstadoTarjeta::Pagada);
        assert_eq!(estado_historial(&factura), EstadoHistorial::Pagada);
        assert_eq!(formato::pesos(saldo_pendiente(&factura)), "$0,00");
        let decimos = factura_con_pagos(0.3, &[0.1, 0.2]);
        assert_eq!(saldo_pendiente(&decimos), 0.0);
        assert_eq!(estado_tarjeta(&decimos), EstadoTarjeta::Pagada);
        let pagada = factura_con_pagos(18600.0, &[18600.0]);
        assert_eq!(estado_tarjeta(&pagada), EstadoTarjeta::Pagada);
        assert_eq!(estado_historial(&pagada), EstadoHistorial::Pagada);
        let sin_pagos = json!({ "total": 100, "estado": "pendiente", "pagos": [] });
        assert_eq!(estado_tarjeta(&sin_pagos), EstadoTarjeta::SegunBackend { pagada: false, texto: "pendiente".into() });
        assert_eq!(estado_historial(&sin_pagos), EstadoHistorial::Pendiente);
        let parcial = factura_con_pagos(86500.0, &[30000.0]);
        assert_eq!(saldo_pendiente(&parcial), 56500.0);
        assert_eq!(formato::numero_con(porcentaje_pagado(&parcial), 1, 1), "34,7");
    }

    #[test]
    fn resumen_de_cuenta_suma_saldos_y_cobros_del_mes() {
        let hoy = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        let facturas = vec![
            json!({ "total": 1000, "estado": "pendiente", "pagos": [{ "monto": 400, "fecha": "2026-10-02T15:00:00.000Z" }, { "monto": 100, "fecha": "2026-09-28T15:00:00.000Z" }] }),
            json!({ "total": 500, "estado": "pagada", "pagos": [{ "monto": 500, "fecha": "2026-10-01T02:00:00.000Z" }] }),
            json!({ "total": 300, "estado": "pendiente", "pagos": [] }),
            json!({ "total": 900, "estado": "anulada", "pagos": [{ "monto": 50, "fecha": "2026-10-05T15:00:00.000Z" }] }),
        ];
        let resumen = resumen_cuenta(&facturas, hoy);
        assert_eq!(resumen, ResumenCuenta { saldo: 800.0, con_saldo: 2, cobrado_mes: 400.0 });
        assert_eq!(resumen_cuenta(&[], hoy), ResumenCuenta::default());
    }

    #[test]
    fn saldo_para_pago_respeta_el_saldo_del_backend_aunque_sea_cero() {
        assert_eq!(saldo_para_pago(&json!({ "total": 100, "saldoPendiente": 40 })), 40.0);
        assert_eq!(saldo_para_pago(&json!({ "total": 100, "saldoPendiente": 0 })), 0.0);
        assert_eq!(saldo_para_pago(&json!({ "total": 100, "saldoPendiente": 1e-12 })), 0.0);
        assert_eq!(saldo_para_pago(&json!({ "total": 100 })), 100.0);
        assert_eq!(saldo_para_pago(&json!({ "total": 100, "pagos": [{ "monto": 30 }] })), 70.0);
        assert_eq!(validar_pago("1", 0.0), Err(ErrorPago::MontoMayor(0.0)));
    }

    #[test]
    fn precio_de_lista_o_base() {
        let producto = json!({ "id": 5, "precio": 2600 });
        assert_eq!(precio_con_lista(&producto, Some(2450.0)), 2450.0);
        assert_eq!(precio_con_lista(&producto, Some(0.0)), 2600.0);
        assert_eq!(precio_con_lista(&producto, None), 2600.0);
    }

    #[test]
    fn edicion_manda_solo_lo_que_cambio() {
        let factura = json!({ "estado": "vencida", "observaciones": null });
        assert_eq!(cuerpo_edicion(&factura, "vencida", ""), None);
        assert_eq!(cuerpo_edicion(&factura, "vencida", "Entregar lunes"), Some(json!({ "observaciones": "Entregar lunes" })));
        assert_eq!(cuerpo_edicion(&factura, "pagada", ""), Some(json!({ "estado": "pagada" })));
    }

    #[test]
    fn nombre_del_pdf_como_el_backend() {
        assert_eq!(nombre_pdf(&json!({ "tipoFactura": "A", "numero": "0002-00000013" })), "factura-A-0002-00000013.pdf");
        assert_eq!(nombre_pdf(&json!({ "tipoFactura": "remito", "numero": "R/1:2" })), "factura-remito-R_1_2.pdf");
    }

    #[test]
    fn totales_de_remito_sin_iva_y_de_factura_con_iva() {
        let tomate = json!({ "ivaIncluido": true, "ivaPorcentaje": 10.5 });
        let detalles = vec![
            Detalle::nuevo(1, Some(tomate.clone()), "Tomate".into(), 1.245, 2400.0),
            Detalle::nuevo(12, Some(tomate), "Pimiento".into(), 0.415, 4500.0),
        ];
        let remito = calcular_totales(&detalles, "remito", false);
        assert_eq!(remito, Totales { subtotal: 4855.5, impuestos: 0.0, total: 4855.5 });
        let factura = calcular_totales(&detalles, "A", false);
        assert_eq!(factura.subtotal, 4394.12);
        assert_eq!(factura.impuestos, 461.38);
        assert_eq!(factura.total, redondear2(factura.subtotal + factura.impuestos));
        let sin_producto = vec![Detalle::nuevo(9, None, "X".into(), 1.0, 100.0)];
        assert_eq!(calcular_totales(&sin_producto, "A", false), Totales { subtotal: 100.0, impuestos: 21.0, total: 121.0 });
        assert_eq!(calcular_totales(&sin_producto, "C", true), Totales { subtotal: 82.64, impuestos: 17.36, total: 100.0 });
    }

    #[test]
    fn precios_visibles_con_iva_y_vuelta() {
        let manzana = json!({ "ivaIncluido": false, "ivaPorcentaje": 10.5 });
        let incluido = json!({ "ivaIncluido": true, "ivaPorcentaje": 21.0 });
        assert_eq!(precio_con_iva(Some(&manzana), 1000.0, Some(false)), 1105.0);
        assert_eq!(precio_con_iva(Some(&manzana), 1000.0, Some(true)), 1000.0);
        assert_eq!(precio_con_iva(Some(&incluido), 1000.0, Some(false)), 1000.0);
        assert_eq!(precio_con_iva(None, 100.0, Some(false)), 121.0);
        assert_eq!(precio_con_iva(Some(&manzana), 1000.0, None), 1000.0);
        assert!((precio_sin_iva(Some(&manzana), 1105.0, Some(false)) - 1000.0).abs() < 1e-9);
        assert!((precio_sin_iva(None, 121.0, Some(false)) - 100.0).abs() < 1e-9);
        assert_eq!(precio_sin_iva(Some(&incluido), 1000.0, Some(false)), 1000.0);
        assert_eq!(texto_precio_con_iva(Some(&manzana), 82.5, Some(false)), "91,16");
        assert_eq!(texto_precio_con_iva(None, 0.1 * 3.0, Some(true)), "0,3");
    }

    #[test]
    fn filas_con_iva_suman_el_total_de_la_factura() {
        let sin_incluir = json!({ "ivaIncluido": false, "ivaPorcentaje": 10.5 });
        let detalles = vec![
            Detalle::nuevo(1, Some(sin_incluir.clone()), "Tomate".into(), 1.245, 2400.0),
            Detalle::nuevo(2, Some(sin_incluir), "Pimiento".into(), 0.415, 4500.0),
            Detalle::nuevo(3, None, "Varios".into(), 3.0, 333.33),
        ];
        for tipo in ["A", "C"] {
            let totales = calcular_totales(&detalles, tipo, false);
            let visibles: f64 = detalles.iter().map(|d| redondear2(precio_con_iva(d.producto.as_ref(), d.subtotal, Some(false)))).sum();
            assert!((visibles - totales.total).abs() <= 0.01 * detalles.len() as f64, "{tipo}: {visibles} vs {}", totales.total);
            assert_eq!(totales.total, redondear2(totales.subtotal + totales.impuestos));
        }
        let mut editado = detalles[0].clone();
        editado.precio_unitario = precio_sin_iva(editado.producto.as_ref(), 2652.0, Some(false));
        editado.recalcular();
        assert!((precio_con_iva(editado.producto.as_ref(), editado.subtotal, Some(false)) - 2652.0 * 1.245).abs() < 1e-6);
        assert!((editado.precio_unitario - 2400.0).abs() < 1e-9);
    }

    #[test]
    fn remito_muestra_sin_iva_y_las_filas_suman_su_total() {
        let sin_incluir = json!({ "ivaIncluido": false, "ivaPorcentaje": 10.5 });
        let detalles = vec![
            Detalle::nuevo(1, Some(sin_incluir.clone()), "Tomate".into(), 1.245, 2400.0),
            Detalle::nuevo(2, Some(sin_incluir.clone()), "Pimiento".into(), 0.415, 4500.0),
            Detalle::nuevo(3, None, "Varios".into(), 3.0, 333.33),
        ];
        let iva = iva_visible("remito", Some(false));
        assert_eq!(iva, None);
        assert_eq!(precio_con_iva(Some(&sin_incluir), 100.0, iva), 100.0);
        assert_eq!(precio_con_iva(None, 100.0, iva), 100.0);
        assert_eq!(precio_sin_iva(Some(&sin_incluir), 100.0, iva), 100.0);
        assert_eq!(texto_precio_con_iva(Some(&sin_incluir), 2400.0, iva), "2400");
        let totales = calcular_totales(&detalles, "remito", false);
        let visibles: f64 = detalles.iter().map(|d| redondear2(precio_con_iva(d.producto.as_ref(), d.subtotal, iva))).sum();
        assert!((visibles - totales.total).abs() <= 0.01 * detalles.len() as f64, "{visibles} vs {}", totales.total);
    }

    #[test]
    fn facturas_siguen_mostrando_con_iva() {
        let sin_incluir = json!({ "ivaIncluido": false, "ivaPorcentaje": 21.0 });
        for tipo in ["A", "B", "C"] {
            assert_eq!(iva_visible(tipo, Some(false)), Some(false));
            assert_eq!(iva_visible(tipo, Some(true)), Some(true));
            assert_eq!(precio_con_iva(Some(&sin_incluir), 100.0, iva_visible(tipo, Some(false))), 121.0);
        }
        assert_eq!(iva_visible("B", None), None);
        assert_eq!(iva_visible("remito", Some(true)), None);
    }

    #[test]
    fn redondeo_como_math_round() {
        assert_eq!(redondear_js(2.5), 3.0);
        assert_eq!(redondear_js(-2.5), -2.0);
        assert_eq!(redondear2(1.005), 1.0);
        assert_eq!(redondear2(0.125), 0.13);
    }

    #[test]
    fn cuerpo_local_y_afip() {
        let detalles = vec![Detalle::nuevo(3, None, "Cebolla".into(), 2.0, 1100.0)];
        let datos = DatosFactura {
            cliente_id: json!("c1"),
            tipo_factura: "remito",
            observaciones: "",
            lista_precio_id: None,
            pago_inicial: None,
            fecha_vencimiento: None,
            detalles: &detalles,
        };
        let totales = calcular_totales(&detalles, "remito", true);
        let (ruta, cuerpo) = cuerpo_factura(&datos, false, totales, &json!({}));
        assert_eq!(ruta, "/api/facturas");
        assert_eq!(
            cuerpo.to_string(),
            r#"{"clienteId":"c1","tipoFactura":"remito","observaciones":"","pagoInicial":0.0,"subtotal":2200.0,"impuestos":0.0,"total":2200.0,"detalles":[{"productoId":3,"descripcion":"Cebolla","cantidad":2.0,"precioUnitario":1100.0,"subtotal":2200.0}],"esCuentaCorriente":true}"#
        );
        let datos = DatosFactura { tipo_factura: "A", lista_precio_id: Some(1), pago_inicial: Some(500.0), fecha_vencimiento: Some("2026-11-01T00:00:00.000Z"), ..datos };
        let (ruta, cuerpo) = cuerpo_factura(&datos, true, totales, &json!({ "sucursalId": 2 }));
        assert_eq!(ruta, "/api/facturas/crear-afip");
        assert_eq!(
            cuerpo.to_string(),
            r#"{"clienteId":"c1","tipoFactura":"A","observaciones":"","listaPrecioId":1,"pagoInicial":500.0,"esConsumidorFinal":false,"sucursalId":2,"productos":[{"productoId":3,"cantidad":2.0,"precio":1100.0}],"esCuentaCorriente":true,"fechaVencimiento":"2026-11-01T00:00:00.000Z"}"#
        );
        assert!(usar_afip(&json!({ "afipHabilitado": true }), "A"));
        assert!(!usar_afip(&json!({ "afipHabilitado": true }), "remito"));
    }

    #[test]
    fn pagos_validacion_y_cuerpo() {
        assert_eq!(validar_pago("abc", 100.0), Err(ErrorPago::MontoInvalido));
        assert_eq!(validar_pago("0", 100.0), Err(ErrorPago::MontoInvalido));
        assert_eq!(validar_pago("150", 100.0), Err(ErrorPago::MontoMayor(100.0)));
        assert_eq!(validar_pago("99.5", 100.0), Ok(99.5));
        assert_eq!(
            ErrorPago::MontoMayor(56500.0).to_string(),
            "El monto no puede ser mayor al saldo pendiente ($56.500,00)"
        );
        assert_eq!(
            cuerpo_pago(&json!("f1"), 10.0, "efectivo", "2026-10-10", "").to_string(),
            r#"{"facturaId":"f1","monto":10.0,"metodoPago":"efectivo","fecha":"2026-10-10"}"#
        );
        assert_eq!(cuerpo_pago(&json!("f1"), 10.0, "tarjeta", "2026-10-10", "Visa")["referencia"], "Visa");
    }

    #[test]
    fn formatos_numericos() {
        assert_eq!(texto_entrada(1.245), "1,245");
        assert_eq!(texto_entrada(2400.0), "2400");
        assert_eq!(texto_entrada(f64::NAN), "");
        assert_eq!(parsear_numero("1,245"), Some(1.245));
        assert_eq!(parsear_numero("1.245"), Some(1.245));
        assert_eq!(parsear_numero("56.500,50"), Some(56500.5));
        assert_eq!(parsear_numero(&texto_entrada(0.415)), Some(0.415));
        assert_eq!(parsear_float("12.5abc"), Some(12.5));
        assert_eq!(parsear_float(""), None);
        assert_eq!(parsear_float("1e3"), Some(1000.0));
    }

    #[test]
    fn paginacion_y_resumenes() {
        assert_eq!(paginas_visibles(1, 4), vec![1, 2, 3, 4]);
        assert_eq!(paginas_visibles(2, 9), vec![1, 2, 3, 4, 5]);
        assert_eq!(paginas_visibles(8, 9), vec![5, 6, 7, 8, 9]);
        assert_eq!(paginas_visibles(5, 9), vec![3, 4, 5, 6, 7]);
        assert_eq!(resumen_clientes(false, false, 6), "6 clientes totales");
        assert_eq!(resumen_clientes(false, true, 1), "1 resultado encontrado");
        assert_eq!(resumen_productos(false, false, 19), "19 productos disponibles");
        let productos = vec![json!({ "id": 1, "nombre": "Tomate", "plu": "001" }), json!({ "id": 2, "nombre": "Papa", "plu": null })];
        assert_eq!(filtrar_productos(&productos, "00", &[]).len(), 1);
        assert_eq!(filtrar_productos(&productos, "", &[1]).len(), 1);
        assert_eq!(filtrar_productos(&productos, " PA ", &[]).len(), 1);
    }

    #[test]
    fn nombres_y_fechas() {
        assert_eq!(nombre_cliente(&json!({ "nombre": "María", "apellido": "Quiroga", "razonSocial": null })), "María Quiroga");
        assert_eq!(nombre_cliente(&json!({ "nombre": "X", "razonSocial": "El Mesón S.A." })), "El Mesón S.A.");
        assert_eq!(fecha_es_ar("no"), "-");
        assert_eq!(fecha_es_ar("2026-10-09T15:35:37.370Z"), "09/10/2026");
        assert_eq!(fecha_es_ar("2026-10-10T01:30:00.000Z"), "09/10/2026");
        assert_eq!(fecha_es_ar("2026-10-10T02:59:59.999-03:00"), "10/10/2026");
        assert_eq!(fecha_es_ar("2026-10-10"), "10/10/2026");
        assert_eq!(fecha_es_ar("2026-10-10T00:00:00.000Z"), "10/10/2026");
        assert_eq!(fecha_iso_desde_input("2026-10-15").as_deref(), Some("2026-10-15T00:00:00.000Z"));
        assert_eq!(fecha_dia_mes_anio("10/10/2026"), NaiveDate::from_ymd_opt(2026, 10, 10));
    }

    #[test]
    fn impresion_completa_datos_del_negocio() {
        let factura = json!({ "numero": "R-1", "total": 10 });
        let datos = datos_impresion(&factura, &json!({ "nombre": "La Huerta", "cuit": "30-1" }));
        assert_eq!(datos["businessName"], "La Huerta");
        assert_eq!(datos["razonSocial"], "La Huerta");
        assert_eq!(datos["cuit"], "30-1");
        assert_eq!(datos_impresion(&factura, &Value::Null)["businessName"], "Mi Negocio");
    }
}
