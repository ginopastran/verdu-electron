use chrono::{DateTime, Local, Utc};
use serde_json::{json, Map, Value};

use crate::carrito::Producto;
use crate::formato;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TipoDescuento {
    Porcentaje,
    Fijo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Descuento {
    pub tipo: TipoDescuento,
    pub valor: f64,
    pub monto: f64,
}

impl Descuento {
    pub fn porcentaje(valor: f64, monto: f64) -> Self {
        Descuento { tipo: TipoDescuento::Porcentaje, valor, monto }
    }

    pub fn fijo(valor: f64, monto: f64) -> Self {
        Descuento { tipo: TipoDescuento::Fijo, valor, monto }
    }

    pub fn con_monto(self, monto: f64) -> Self {
        Descuento { monto, ..self }
    }

    pub fn tipo_api(&self) -> &'static str {
        match self.tipo {
            TipoDescuento::Porcentaje => "porcentual",
            TipoDescuento::Fijo => "cantidad",
        }
    }

    pub fn tipo_js(&self) -> &'static str {
        match self.tipo {
            TipoDescuento::Porcentaje => "percentage",
            TipoDescuento::Fijo => "fixed",
        }
    }

    pub fn datos(&self) -> Value {
        json!({ "type": self.tipo_js(), "value": self.valor, "amount": self.monto })
    }

    pub fn etiqueta(&self) -> String {
        match self.tipo {
            TipoDescuento::Porcentaje => format!("{}%", formato::numero(self.valor)),
            TipoDescuento::Fijo => formato::pesos(self.valor),
        }
    }
}

pub fn verdad(valor: &Value) -> bool {
    match valor {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

pub fn primero<'a>(valores: impl IntoIterator<Item = &'a Value>) -> Option<&'a Value> {
    valores.into_iter().find(|v| verdad(v))
}

pub fn numero(valor: &Value) -> f64 {
    match valor {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse().unwrap_or(f64::NAN),
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::Null => 0.0,
        _ => f64::NAN,
    }
}

fn redondear_digitos(x: f64, decimales: usize) -> (bool, String, String) {
    let negativo = x.is_sign_negative();
    let exacto = format!("{:.*}", decimales + 30, x.abs());
    let (entero, fraccion) = exacto.split_once('.').unwrap_or((&exacto, ""));
    let mut digitos: Vec<u8> = entero.bytes().chain(fraccion.bytes().take(decimales)).map(|b| b - b'0').collect();
    if fraccion.as_bytes().get(decimales).is_some_and(|b| *b >= b'5') {
        let mut i = digitos.len();
        loop {
            if i == 0 {
                digitos.insert(0, 1);
                break;
            }
            i -= 1;
            if digitos[i] == 9 {
                digitos[i] = 0;
            } else {
                digitos[i] += 1;
                break;
            }
        }
    }
    let texto: String = digitos.iter().map(|d| (d + b'0') as char).collect();
    let corte = texto.len() - decimales;
    (negativo, texto[..corte].to_string(), texto[corte..].to_string())
}

pub fn fijo(x: f64, decimales: usize) -> String {
    if !x.is_finite() || x.abs() >= 1e21 {
        return numero_js(x);
    }
    let (_, entero, fraccion) = redondear_digitos(x, decimales);
    let signo = if x < 0.0 { "-" } else { "" };
    if decimales == 0 {
        format!("{signo}{entero}")
    } else {
        format!("{signo}{entero}.{fraccion}")
    }
}

pub fn redondear(x: f64, decimales: usize) -> f64 {
    fijo(x, decimales).parse().unwrap_or(x)
}

pub fn numero_js(x: f64) -> String {
    if x.is_nan() {
        "NaN".into()
    } else if x.is_infinite() {
        if x > 0.0 { "Infinity".into() } else { "-Infinity".into() }
    } else if x == 0.0 {
        "0".into()
    } else {
        format!("{x}")
    }
}

pub fn parse_float(texto: &str) -> Option<f64> {
    let prefijo: String = texto
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E'))
        .collect();
    (1..=prefijo.len())
        .rev()
        .filter(|n| prefijo[..*n].bytes().any(|b| b.is_ascii_digit()))
        .find_map(|n| prefijo[..n].parse::<f64>().ok())
}

pub fn redondear_a_50(monto: f64) -> f64 {
    let fijado = redondear(monto, 2);
    let entero = fijado.floor();
    let resto = entero % 50.0;
    let decimal = fijado - entero;
    if resto == 0.0 && decimal == 0.0 {
        return fijado;
    }
    entero - resto
}

pub fn descuento_calculado(subtotal: f64, porcentaje: bool, texto: &str) -> f64 {
    if texto.is_empty() {
        return 0.0;
    }
    match parse_float(texto) {
        Some(p) if porcentaje && p > 0.0 && p <= 100.0 => subtotal * (p / 100.0),
        Some(m) if !porcentaje && m > 0.0 => m.min(subtotal),
        _ => 0.0,
    }
}

pub fn montos_efectivo(total: f64, descuento: Option<&Descuento>, descuento_efectivo: Option<f64>, pago_exacto: bool) -> (f64, f64) {
    let original = redondear(total, 2);
    let final_ = match (descuento, descuento_efectivo) {
        (Some(d), _) if d.tipo == TipoDescuento::Porcentaje => original - original * d.valor / 100.0,
        (Some(d), _) => original - d.monto,
        (None, Some(p)) => original - original * p / 100.0,
        (None, None) => original,
    };
    if pago_exacto {
        (original, final_)
    } else {
        (original, redondear_a_50(final_))
    }
}

pub fn descuento_pago_efectivo(descuento: Option<Descuento>, aplicando: bool, porcentaje: Option<f64>, monto: f64) -> Option<Descuento> {
    match (descuento, aplicando, porcentaje) {
        (Some(d), _, _) => Some(d.con_monto(monto)),
        (None, true, Some(p)) => Some(Descuento::porcentaje(p, monto)),
        _ => None,
    }
}

pub fn descuento_efectivo(negocio: &Value) -> Option<f64> {
    verdad(&negocio["descuentoEfectivo"]).then(|| numero(&negocio["descuentoEfectivo"]))
}

pub fn minimo_descuento_efectivo(negocio: &Value) -> Option<f64> {
    Some(numero(&negocio["descuentoEfectivoMinimo"])).filter(|minimo| *minimo > 0.0)
}

pub fn alcanza_minimo_efectivo(negocio: &Value, total: f64) -> bool {
    minimo_descuento_efectivo(negocio).is_none_or(|minimo| total >= minimo)
}

pub fn vuelto(texto: &str, total: f64) -> (f64, Option<&'static str>) {
    match parse_float(texto) {
        Some(pagado) if pagado > 0.0 => {
            let cambio = pagado - total;
            (cambio, (cambio < 0.0).then_some("El monto ingresado es insuficiente"))
        }
        _ => (0.0, None),
    }
}

pub fn validar_pago_exacto(texto: &str, total: f64) -> Result<f64, &'static str> {
    match parse_float(texto) {
        Some(pagado) if pagado > 0.0 => {
            if pagado < total {
                Err("El monto pagado debe ser mayor o igual al total")
            } else {
                Ok(pagado)
            }
        }
        _ => Err("Ingresa un monto válido"),
    }
}

pub fn mixto_restante(total: f64, efectivo: &str) -> f64 {
    match parse_float(efectivo) {
        Some(monto) if !efectivo.is_empty() => total - monto.min(total),
        _ => total,
    }
}

pub fn mixto_habilitado(total: f64, efectivo: &str, procesando: bool) -> bool {
    !procesando && !efectivo.is_empty() && parse_float(efectivo).is_some_and(|m| m > 0.0 && m < total)
}

pub const SONDEO_MAXIMO_ERRORES: u32 = 5;

pub fn sondeo_vencido(inicio: f64, ahora: f64, afip: bool) -> bool {
    ahora - inicio >= if afip { 300.0 } else { 600.0 }
}

pub fn clave_manual_valida(ingresada: &str, configurada: Option<&str>) -> bool {
    configurada.is_some_and(|clave| !clave.is_empty() && clave == ingresada)
}

pub fn estado_qr(status: &Value) -> String {
    let estado = status.as_str().unwrap_or("").to_uppercase();
    match estado.as_str() {
        "PENDING" => "PENDIENTE".into(),
        "APPROVED" | "COMPLETED" => "COMPLETADA".into(),
        "CANCELLED" | "CANCELED" | "REJECTED" => "CANCELADA".into(),
        _ => estado,
    }
}

pub fn fecha_iso(ahora: DateTime<Utc>) -> String {
    ahora.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn fecha_es_ar(ahora: DateTime<Local>) -> String {
    ahora.format("%d/%m/%Y %H:%M:%S").to_string()
}

pub fn items_orden(items: &[Producto], con_nombre_lista: bool) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            let mut fila = json!({
                "productoId": item.id,
                "cantidad": item.quantity,
                "subtotal": redondear(item.subtotal, 2),
                "precioHistorico": item.price_per_unit,
                "costo": item.costo,
                "nombre": item.name,
                "listaPrecioId": item.lista_precio_id,
            });
            if let (true, Some(nombre)) = (con_nombre_lista, &item.lista_precio_nombre) {
                fila["listaPrecioNombre"] = json!(nombre);
            }
            fila
        })
        .collect()
}

pub fn items_afip(items: &[Producto]) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            json!({
                "productoId": item.id,
                "cantidad": item.quantity,
                "precio": item.price_per_unit,
                "subtotal": redondear(item.subtotal, 2),
                "nombre": item.name,
            })
        })
        .collect()
}

pub fn productos_cancelacion(items: &[Producto]) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            json!({
                "productoId": item.id,
                "nombreProducto": item.name,
                "cantidad": item.quantity,
                "precioUnitario": item.price_per_unit,
                "subtotal": redondear(item.subtotal, 2),
            })
        })
        .collect()
}

pub struct Venta<'a> {
    pub usuario: &'a Value,
    pub negocio_nombre: &'a str,
    pub creado: &'a str,
}

impl Venta<'_> {
    fn base(&self, mapa: &mut Map<String, Value>) {
        mapa.insert("vendedorId".into(), self.usuario["id"].clone());
        mapa.insert("sucursalId".into(), self.usuario["sucursalId"].clone());
        mapa.insert("vendedor".into(), self.usuario["nombre"].clone());
        mapa.insert("businessName".into(), json!(self.negocio_nombre));
    }
}

pub fn campos_descuento(descuento: Option<&Descuento>, monto: f64, subtotal_sin: f64, subtotal: Option<f64>) -> Map<String, Value> {
    let mut mapa = Map::new();
    match descuento {
        Some(d) => {
            mapa.insert("tieneDescuento".into(), json!(true));
            mapa.insert("tipoDescuento".into(), json!(d.tipo_api()));
            mapa.insert("valorDescuento".into(), json!(d.valor));
            mapa.insert("montoDescuento".into(), json!(monto));
            mapa.insert("subtotalSinDescuento".into(), json!(subtotal_sin));
            mapa.insert("discountData".into(), d.datos());
            if let Some(s) = subtotal {
                mapa.insert("subtotal".into(), json!(s));
            }
        }
        None => {
            mapa.insert("tieneDescuento".into(), json!(false));
        }
    }
    mapa
}

pub fn orden(
    venta: &Venta,
    metodo: Option<&str>,
    total: f64,
    items: Vec<Value>,
    estado: Option<&str>,
    pagos: Option<Value>,
    descuento: Map<String, Value>,
) -> Value {
    let mut mapa = Map::new();
    if let Some(m) = metodo {
        mapa.insert("metodoPago".into(), json!(m));
    }
    mapa.insert("total".into(), json!(total));
    mapa.insert("items".into(), Value::Array(items));
    venta.base(&mut mapa);
    if let Some(e) = estado {
        mapa.insert("estado".into(), json!(e));
    }
    mapa.insert("createdAt".into(), json!(venta.creado));
    if let Some(p) = pagos {
        mapa.insert("pagos".into(), p);
    }
    mapa.extend(descuento);
    Value::Object(mapa)
}

pub fn pagos_mixtos(efectivo: f64, segundo: &str, monto_segundo: f64) -> Value {
    json!([
        { "metodoPago": "efectivo", "monto": efectivo },
        { "metodoPago": segundo, "monto": monto_segundo },
    ])
}

pub fn con_id(mut orden: Value, resultado: &Value) -> Value {
    let id_real = primero([&resultado["idReal"], &resultado["id"]]).cloned().unwrap_or(Value::Null);
    let id = primero([&resultado["id"]]).cloned().unwrap_or(Value::Null);
    orden["idReal"] = id_real;
    orden["id"] = id;
    orden
}

fn numero_o_none(v: &Value) -> Option<f64> {
    v.as_f64()
}

pub fn payload_valido(payload: &Value) -> Result<Value, String> {
    let items: Vec<Value> = payload["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|raw| {
                    let producto_id = [&raw["productoId"], &raw["id"]].into_iter().find(|v| v.is_number())?.clone();
                    let cantidad = numero_o_none(&raw["cantidad"]).or(numero_o_none(&raw["quantity"])).unwrap_or(0.0);
                    let mut precio = numero_o_none(&raw["precioHistorico"]).or(numero_o_none(&raw["pricePerUnit"]));
                    let mut subtotal = numero_o_none(&raw["subtotal"]);
                    if precio.is_none() && cantidad != 0.0 {
                        if let Some(s) = subtotal {
                            precio = Some(redondear(s / cantidad, 2));
                        }
                    }
                    if subtotal.is_none_or(|s| s == 0.0) {
                        if let Some(p) = precio {
                            subtotal = Some(redondear(p * cantidad, 2));
                        }
                    }
                    let costo = numero_o_none(&raw["costo"]).unwrap_or(0.0);
                    if cantidad == 0.0 || cantidad.is_nan() {
                        return None;
                    }
                    let (precio, subtotal) = (precio?, subtotal?);
                    let lista = match &raw["listaPrecioId"] {
                        Value::Null => Value::Null,
                        Value::String(s) if s.is_empty() => Value::Null,
                        otro => {
                            let n = numero(otro);
                            if n.is_finite() { json!(n) } else { Value::Null }
                        }
                    };
                    let mut fila = Map::new();
                    fila.insert("productoId".into(), producto_id);
                    fila.insert("cantidad".into(), json!(cantidad));
                    fila.insert("subtotal".into(), json!(subtotal));
                    fila.insert("costo".into(), json!(costo));
                    fila.insert("precioHistorico".into(), json!(precio));
                    if let Some(nombre) = raw.get("nombre") {
                        fila.insert("nombre".into(), nombre.clone());
                    }
                    fila.insert("listaPrecioId".into(), lista);
                    if let Some(nombre) = raw["listaPrecioNombre"].as_str().filter(|s| !s.is_empty()) {
                        fila.insert("listaPrecioNombre".into(), json!(nombre));
                    }
                    Some(Value::Object(fila))
                })
                .collect()
        })
        .unwrap_or_default();
    if items.is_empty() {
        return Err("No valid items to create order payload".into());
    }
    let calculado = redondear(items.iter().map(|i| numero(&i["subtotal"])).sum(), 2);
    let total_payload = payload["total"].as_f64();
    let total = total_payload.unwrap_or(calculado);
    let tiene_descuento = verdad(&payload["tieneDescuento"])
        || (!payload["montoDescuento"].is_null() && numero(&payload["montoDescuento"]) > 0.0)
        || verdad(&payload["discountData"]);
    let final_ = match total_payload {
        Some(t) if tiene_descuento => t,
        _ if (total - calculado).abs() < 0.01 => calculado,
        _ => total,
    };
    let mut pagos: Vec<Value> = payload["pagos"].as_array().cloned().unwrap_or_default();
    if pagos.is_empty() {
        let metodo = primero([&payload["metodoPago"]]).cloned().unwrap_or(json!("efectivo"));
        pagos = vec![json!({ "metodoPago": metodo, "monto": final_ })];
    }
    let pagado: f64 = pagos.iter().map(|p| numero(&p["monto"])).sum();
    if (pagado - final_).abs() >= 0.01 {
        let primero_monto = numero(&pagos[0]["monto"]);
        pagos[0]["monto"] = json!(redondear(primero_monto + (final_ - pagado), 2));
    }
    let mut salida = payload.as_object().cloned().unwrap_or_default();
    salida.insert("items".into(), Value::Array(items));
    salida.insert("total".into(), json!(final_));
    salida.insert("pagos".into(), Value::Array(pagos));
    Ok(Value::Object(salida))
}

pub fn ajustar_total_ticket(datos: &mut Value) {
    let descuento = &datos["discountData"];
    if !descuento.is_object() {
        return;
    }
    let tipo = descuento["type"].as_str().unwrap_or("").to_string();
    let valor = numero(&descuento["value"]);
    if tipo.is_empty() || !(valor > 0.0) {
        return;
    }
    let monto = numero(&descuento["amount"]);
    let subtotal = numero(&datos["subtotal"]);
    let sin = numero(&datos["subtotalSinDescuento"]);
    let total = numero(&datos["total"]);
    let base = if subtotal > 0.0 {
        subtotal
    } else if sin > 0.0 {
        sin
    } else {
        total
    };
    let mut descontado = if tipo == "percentage" || tipo == "porcentual" { base * valor / 100.0 } else { valor };
    if monto > 0.0 {
        descontado = monto;
    }
    let calculado = base - descontado;
    if (total - calculado).abs() > 0.01 {
        datos["total"] = json!(calculado);
    }
}

pub struct Qr<'a> {
    pub monto: f64,
    pub descripcion: String,
    pub usuario: &'a Value,
    pub items: Vec<Value>,
    pub afip: bool,
    pub mixto: bool,
    pub efectivo: f64,
}

impl Qr<'_> {
    pub fn payload(self, descuento: Map<String, Value>) -> Value {
        let mut mapa = Map::new();
        mapa.insert("monto".into(), json!(self.monto));
        mapa.insert("descripcion".into(), json!(self.descripcion));
        mapa.insert("vendedorId".into(), self.usuario["id"].clone());
        mapa.insert("sucursalId".into(), self.usuario["sucursalId"].clone());
        mapa.insert("items".into(), Value::Array(self.items));
        mapa.insert("requiresAfipInvoice".into(), json!(self.afip));
        mapa.insert("isSplitPayment".into(), json!(self.mixto));
        mapa.insert("cashAmount".into(), json!(self.efectivo));
        mapa.insert("afipData".into(), Value::Null);
        mapa.extend(descuento);
        Value::Object(mapa)
    }
}

pub fn descuento_qr(descuento: Option<&Descuento>, subtotal_sin: f64) -> Map<String, Value> {
    let mut mapa = Map::new();
    match descuento {
        Some(d) => {
            mapa.insert("tieneDescuento".into(), json!(true));
            mapa.insert("tipoDescuento".into(), json!(d.tipo_api()));
            mapa.insert("valorDescuento".into(), json!(d.valor));
            mapa.insert("subtotalSinDescuento".into(), json!(subtotal_sin));
        }
        None => {
            mapa.insert("tieneDescuento".into(), json!(false));
        }
    }
    mapa
}

pub fn descuento_qr_afip(descuento: Option<&Descuento>, subtotal: f64, monto: f64, con_subtotal: bool) -> Map<String, Value> {
    let mut mapa = descuento_qr(descuento, subtotal);
    if let Some(d) = descuento {
        mapa.insert("montoDescuento".into(), json!(monto));
        mapa.insert("discountData".into(), d.con_monto(monto).datos());
        if con_subtotal {
            mapa.insert("subtotal".into(), json!(subtotal));
        }
        mapa.insert(
            "descuento".into(),
            json!({ "tipoDescuento": d.tipo_api(), "valorDescuento": d.valor, "montoDescuento": monto }),
        );
    }
    mapa
}

pub fn subtotal_mixto_afip(total: f64, descuento: Option<&Descuento>) -> f64 {
    match descuento {
        Some(d) if d.tipo == TipoDescuento::Porcentaje => total / (1.0 - d.valor / 100.0),
        Some(d) => total + d.monto,
        None => total,
    }
}

pub struct Afip<'a> {
    pub negocio: &'a Value,
    pub usuario: &'a Value,
    pub metodo: &'a str,
    pub items: Vec<Value>,
    pub total: f64,
    pub subtotal_iva: f64,
    pub descuento: Option<Descuento>,
    pub mixto: Option<(f64, &'a str, f64)>,
}

impl Afip<'_> {
    pub fn payload(&self) -> Value {
        let mut mapa = Map::new();
        let tipo = primero([&self.negocio["configuracionAfip"]["tipoFactura"]]).cloned().unwrap_or(json!("C"));
        mapa.insert("tipoFactura".into(), tipo);
        mapa.insert("esConsumidorFinal".into(), json!(true));
        mapa.insert("clienteId".into(), Value::Null);
        mapa.insert("productos".into(), Value::Array(self.items.clone()));
        mapa.insert("metodoPago".into(), json!(self.metodo));
        mapa.insert("vendedorId".into(), self.usuario["id"].clone());
        mapa.insert("sucursalId".into(), self.usuario["sucursalId"].clone());
        mapa.insert("observaciones".into(), json!("Factura generada desde Electron"));
        match &self.descuento {
            Some(d) => {
                mapa.insert("tieneDescuento".into(), json!(true));
                mapa.insert("tipoDescuento".into(), json!(d.tipo_api()));
                mapa.insert("valorDescuento".into(), json!(d.valor));
                mapa.insert("montoDescuento".into(), json!(d.monto));
                mapa.insert("subtotalSinDescuento".into(), json!(redondear(self.subtotal_iva, 2)));
                mapa.insert("total".into(), json!(redondear(self.total, 2)));
            }
            None => {
                mapa.insert("tieneDescuento".into(), json!(false));
            }
        }
        if let (true, Some((efectivo, segundo, monto))) = (self.metodo == "split", self.mixto) {
            mapa.insert("pagos".into(), pagos_mixtos(efectivo, segundo, monto));
        }
        Value::Object(mapa)
    }

    pub fn impresion(&self, resultado: &Value, fecha_hora: &str) -> Value {
        let n = self.negocio;
        let u = self.usuario;
        let r = resultado;
        let cadena = |valores: &[&Value], defecto: Value| primero(valores.iter().copied()).cloned().unwrap_or(defecto);
        let mut mapa = r.as_object().cloned().unwrap_or_default();
        let mut poner = |clave: &str, valor: Value| {
            if !valor.is_null() || clave == "afipQrUrl" || clave == "idReal" || clave == "configuracionAfip" {
                mapa.insert(clave.into(), valor);
            } else {
                mapa.remove(clave);
            }
        };
        poner("metodoPago", json!(self.metodo));
        poner("items", Value::Array(self.items.clone()));
        poner("total", json!(self.total));
        poner("usuario", cadena(&[&u["nombre"]], json!("Vendedor")));
        poner("fechaHora", json!(fecha_hora));
        poner(
            "businessName",
            cadena(
                &[&n["nombre"], &n["name"], &n["razonSocial"], &r["business"]["name"], &r["business"]["razonSocial"], &u["business"]["name"], &u["business"]["razonSocial"], &u["nombre"]],
                json!("Comercio"),
            ),
        );
        poner(
            "razonSocial",
            cadena(
                &[
                    &n["configuracionAfip"]["razonSocial"],
                    &n["razonSocial"],
                    &n["nombre"],
                    &n["name"],
                    &r["business"]["razonSocial"],
                    &r["business"]["name"],
                    &r["razonSocial"],
                    &r["empresa"]["razonSocial"],
                    &r["empresa"]["nombre"],
                    &u["business"]["razonSocial"],
                    &u["business"]["name"],
                    &u["razonSocial"],
                    &u["empresa"],
                    &u["nombre"],
                ],
                json!("Comercio"),
            ),
        );
        poner(
            "cuit",
            cadena(
                &[&n["configuracionAfip"]["cuit"], &n["cuit"], &n["CUIT"], &r["business"]["cuit"], &r["cuit"], &r["empresa"]["cuit"], &u["business"]["cuit"], &u["cuit"], &u["empresa"]["cuit"]],
                json!("00-00000000-0"),
            ),
        );
        poner(
            "condicionIva",
            cadena(
                &[&n["configuracionAfip"]["condicionIva"], &n["condicionIva"], &r["business"]["condicionIva"], &r["condicionIva"], &r["empresa"]["condicionIva"], &u["business"]["condicionIva"], &u["condicionIva"]],
                json!("Responsable Inscripto"),
            ),
        );
        poner("configuracionAfip", cadena(&[&n["configuracionAfip"]], Value::Null));
        poner(
            "direccion",
            cadena(
                &[
                    &n["sucursalActiva"]["ubicacion"],
                    &n["direccion"],
                    &r["business"]["sucursalActiva"]["ubicacion"],
                    &r["business"]["ubicacion"],
                    &r["direccion"],
                    &r["empresa"]["ubicacion"],
                    &u["business"]["sucursalActiva"]["ubicacion"],
                    &u["business"]["ubicacion"],
                    &u["direccion"],
                ],
                json!("Dirección no configurada"),
            ),
        );
        poner(
            "telefono",
            cadena(
                &[&n["sucursalActiva"]["telefono"], &n["telefono"], &r["business"]["sucursalActiva"]["telefono"], &r["business"]["telefono"], &r["telefono"]],
                json!(""),
            ),
        );
        poner("cae", cadena(&[&r["afip"]["cae"], &r["cae"]], Value::Null));
        poner("afipQrUrl", cadena(&[&r["factura"]["afipQrUrl"], &r["afipQrUrl"]], Value::Null));
        poner("fechaVtoCae", cadena(&[&r["afip"]["vencimientoCae"], &r["afip"]["fechaVtoCae"], &r["fechaVtoCae"]], Value::Null));
        poner("puntoVenta", cadena(&[&r["factura"]["puntoVenta"], &r["afip"]["puntoVenta"], &r["puntoVenta"], &r["punto_venta"]], json!("0001")));
        poner(
            "numeroFactura",
            cadena(
                &[&r["factura"]["numeroFactura"], &r["afip"]["numeroFactura"], &r["numeroFactura"], &r["numero_factura"], &r["factura"]["numero"], &r["numero"]],
                Value::Null,
            ),
        );
        poner("tipoFactura", cadena(&[&r["afip"]["tipoFactura"], &r["tipoFactura"], &r["tipo_factura"]], json!("FACTURA B")));
        poner("vendedor", cadena(&[&u["nombre"], &u["name"]], json!("Vendedor")));
        poner("idReal", cadena(&[&r["factura"]["idReal"], &r["idReal"]], Value::Null));
        let factura = &r["factura"];
        if verdad(&factura["tieneDescuento"]) {
            for clave in ["tieneDescuento", "tipoDescuento", "valorDescuento", "montoDescuento", "subtotalSinDescuento", "subtotal", "total", "impuestos"] {
                poner(clave, factura[clave].clone());
            }
            let tipo = if factura["tipoDescuento"] == "porcentual" { "percentage" } else { "fixed" };
            poner(
                "discountData",
                json!({ "type": tipo, "value": factura["valorDescuento"], "amount": factura["montoDescuento"] }),
            );
        } else if let Some(d) = &self.descuento {
            let subtotal = redondear(self.subtotal_iva, 2);
            poner("tieneDescuento", json!(true));
            poner("tipoDescuento", json!(d.tipo_api()));
            poner("valorDescuento", json!(d.valor));
            poner("montoDescuento", json!(d.monto));
            poner("subtotalSinDescuento", json!(subtotal));
            poner("subtotal", json!(subtotal));
            poner("discountData", d.datos());
        } else {
            poner("tieneDescuento", json!(false));
        }
        if factura["pagos"].as_array().is_some_and(|p| !p.is_empty()) {
            poner("pagos", factura["pagos"].clone());
        } else if let (true, Some((efectivo, segundo, monto))) = (self.metodo == "split", self.mixto) {
            poner("pagos", pagos_mixtos(efectivo, segundo, monto));
        }
        Value::Object(mapa)
    }
}

pub fn cancelacion(
    negocio: &Value,
    usuario: &Value,
    referencia: &str,
    motivo: &str,
    monto: f64,
    productos: Vec<Value>,
) -> Value {
    let negocio_id = primero([&negocio["id"], &usuario["businessId"]]).cloned().unwrap_or(json!(1));
    let sucursal = primero([&usuario["sucursalId"]]).cloned().unwrap_or(json!(1));
    json!({
        "businessId": negocio_id,
        "tipo": "orden",
        "referenciaId": referencia,
        "motivo": motivo.trim(),
        "monto": monto,
        "vendedorId": usuario["id"],
        "sucursalId": sucursal,
        "productos": productos,
    })
}

pub fn validar_motivo(motivo: &str) -> Result<(), &'static str> {
    let recortado = motivo.trim();
    if recortado.is_empty() {
        Err("Debes ingresar un motivo para la cancelación")
    } else if recortado.chars().count() < 3 {
        Err("El motivo debe tener al menos 3 caracteres")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use chrono::TimeZone;

    fn producto(id: i64, nombre: &str, cantidad: f64, precio: f64) -> Producto {
        Producto {
            id,
            cart_id: format!("{id}-x"),
            name: nombre.into(),
            quantity: cantidad,
            unit: "Kg".into(),
            price_per_unit: precio,
            subtotal: (precio * cantidad * 100.0).round() / 100.0,
            costo: precio * 0.62,
            iva_incluido: Some(true),
            iva_porcentaje: Some(10.5),
            lista_precio_id: None,
            lista_precio_nombre: Some("Precio catálogo".into()),
            ..Default::default()
        }
    }

    fn usuario() -> Value {
        json!({ "id": 11, "nombre": "Lucía Fernández", "sucursalId": 1 })
    }

    #[test]
    fn precio_libre_viaja_como_una_unidad_al_monto_cargado() {
        let libre = Producto { id: 9, cart_id: "9-x".into(), name: "Varios".into(), quantity: 1.0, unit: "Unidad".into(), price_per_unit: 2750.5, subtotal: 2750.5, precio_libre: true, ..Default::default() };
        let orden = &items_orden(&[libre.clone()], true)[0];
        assert_eq!((orden["cantidad"].as_f64(), orden["precioHistorico"].as_f64(), orden["subtotal"].as_f64()), (Some(1.0), Some(2750.5), Some(2750.5)));
        let afip = &items_afip(&[libre])[0];
        assert_eq!((afip["cantidad"].as_f64(), afip["precio"].as_f64(), afip["subtotal"].as_f64()), (Some(1.0), Some(2750.5), Some(2750.5)));
        assert!(orden.get("precioLibre").is_none() && afip.get("precioLibre").is_none());
    }

    #[test]
    fn formatos_como_javascript() {
        assert_eq!(fijo(1122.475, 2), "1122.47");
        assert_eq!(fijo(22449.5 * 0.05, 2), "1122.48");
        assert_eq!(fijo(0.125, 2), "0.13");
        assert_eq!(fijo(1.005, 2), "1.00");
        assert_eq!(fijo(27550.5, 2), "27550.50");
        assert_eq!(fijo(-0.001, 2), "-0.00");
        assert_eq!(fijo(9.995, 2), "9.99");
        assert_eq!(fijo(99.996, 2), "100.00");
        assert_eq!(numero_js(5.0), "5");
        assert_eq!(numero_js(10.5), "10.5");
        assert_eq!(numero_js(-0.0), "0");
        assert_eq!(parse_float("10abc"), Some(10.0));
        assert_eq!(parse_float("  1.5e2"), Some(150.0));
        assert_eq!(parse_float("1e"), Some(1.0));
        assert_eq!(parse_float("-"), None);
        assert_eq!(parse_float(""), None);
        assert_eq!(parse_float(".5"), Some(0.5));
    }

    #[test]
    fn redondeo_a_50_hacia_abajo() {
        assert_eq!(redondear_a_50(22449.5), 22400.0);
        assert_eq!(redondear_a_50(22400.0), 22400.0);
        assert_eq!(redondear_a_50(20204.55), 20200.0);
        assert_eq!(redondear_a_50(49.99), 0.0);
        assert_eq!(redondear_a_50(150.004), 150.0);
        assert_eq!(redondear_a_50(150.01), 150.0);
    }

    #[test]
    fn efectivo_segun_sistema_de_pago() {
        assert_eq!(montos_efectivo(22449.5, None, None, false), (22449.5, 22400.0));
        assert_eq!(montos_efectivo(22449.5, None, Some(10.0), false), (22449.5, 20200.0));
        assert_eq!(montos_efectivo(22449.5, None, None, true), (22449.5, 22449.5));
        let d = Descuento::porcentaje(10.0, 2244.95);
        let (o, f) = montos_efectivo(22449.5, Some(&d), Some(10.0), true);
        assert_eq!(o, 22449.5);
        assert!((f - 20204.55).abs() < 1e-9);
        let fijo_ = Descuento::fijo(1500.0, 1500.0);
        assert_eq!(montos_efectivo(22449.5, Some(&fijo_), None, false), (22449.5, 20900.0));
        assert_eq!(descuento_efectivo(&json!({ "descuentoEfectivo": 10 })), Some(10.0));
        assert_eq!(descuento_efectivo(&json!({ "descuentoEfectivo": "5" })), Some(5.0));
        assert_eq!(descuento_efectivo(&json!({ "descuentoEfectivo": 0 })), None);
        assert_eq!(descuento_efectivo(&json!({})), None);
    }

    #[test]
    fn descuento_en_efectivo_desde_un_minimo() {
        let con_minimo = json!({ "descuentoEfectivo": 10, "descuentoEfectivoMinimo": 20000 });
        assert_eq!(minimo_descuento_efectivo(&con_minimo), Some(20000.0));
        assert!(!alcanza_minimo_efectivo(&con_minimo, 19999.99));
        assert!(alcanza_minimo_efectivo(&con_minimo, 20000.0));
        for sin_minimo in [json!({ "descuentoEfectivo": 10 }), json!({ "descuentoEfectivoMinimo": null }), json!({ "descuentoEfectivoMinimo": 0 })] {
            assert_eq!(minimo_descuento_efectivo(&sin_minimo), None);
            assert!(alcanza_minimo_efectivo(&sin_minimo, 1.0));
        }
    }

    #[test]
    fn vuelto_y_validacion() {
        assert_eq!(vuelto("50000", 22449.5), (27550.5, None));
        assert_eq!(vuelto("100", 22449.5).1, Some("El monto ingresado es insuficiente"));
        assert_eq!(vuelto("", 22449.5), (0.0, None));
        assert_eq!(vuelto("0", 22449.5), (0.0, None));
        assert_eq!(validar_pago_exacto("22449.5", 22449.5), Ok(22449.5));
        assert_eq!(validar_pago_exacto("abc", 10.0), Err("Ingresa un monto válido"));
        assert_eq!(validar_pago_exacto("5", 10.0), Err("El monto pagado debe ser mayor o igual al total"));
    }

    #[test]
    fn division_mixta() {
        assert_eq!(mixto_restante(22449.5, "10000"), 12449.5);
        assert_eq!(mixto_restante(22449.5, ""), 22449.5);
        assert_eq!(mixto_restante(22449.5, "99999"), 0.0);
        assert!(mixto_habilitado(22449.5, "10000", false));
        assert!(!mixto_habilitado(22449.5, "10000", true));
        assert!(!mixto_habilitado(22449.5, "22449.5", false));
        assert!(!mixto_habilitado(22449.5, "0", false));
        assert!(!mixto_habilitado(22449.5, "", false));
    }

    #[test]
    fn descuentos_del_dialogo() {
        assert!((descuento_calculado(22449.5, true, "10") - 2244.95).abs() < 1e-9);
        assert_eq!(descuento_calculado(22449.5, true, "101"), 0.0);
        assert_eq!(descuento_calculado(22449.5, true, "0"), 0.0);
        assert_eq!(descuento_calculado(22449.5, false, "1500"), 1500.0);
        assert_eq!(descuento_calculado(1000.0, false, "1500"), 1000.0);
        assert_eq!(descuento_calculado(1000.0, false, ""), 0.0);
        let d = Descuento::porcentaje(5.0, 1122.475);
        assert_eq!(d.etiqueta(), "5%");
        assert_eq!(Descuento::fijo(1500.0, 1500.0).etiqueta(), "$1.500,00");
        assert_eq!(Descuento::porcentaje(10.5, 0.0).etiqueta(), "10,5%");
        assert_eq!(d.datos(), json!({ "type": "percentage", "value": 5.0, "amount": 1122.475 }));
    }

    #[test]
    fn payload_de_orden_con_descuento() {
        let items = vec![producto(1, "Tomate perito", 1.245, 2400.0), producto(19, "Palta Hass", 3.0, 1500.0)];
        let u = usuario();
        let venta = Venta { usuario: &u, negocio_nombre: "Verdulería La Huerta", creado: "2026-10-10T12:00:00.000Z" };
        let d = Descuento::porcentaje(10.0, 748.8);
        let orden = orden(&venta, Some("tarjeta"), 6739.2, items_orden(&items, true), None, None, campos_descuento(Some(&d), 748.8, 7488.0, Some(7488.0)));
        assert_eq!(orden["metodoPago"], "tarjeta");
        assert_eq!(orden["vendedorId"], 11);
        assert_eq!(orden["items"][0]["subtotal"], 2988.0);
        assert_eq!(orden["items"][0]["listaPrecioNombre"], "Precio catálogo");
        assert_eq!(orden["items"][0]["listaPrecioId"], Value::Null);
        assert_eq!(orden["tipoDescuento"], "porcentual");
        assert_eq!(orden["discountData"]["type"], "percentage");
        assert_eq!(orden["subtotal"], 7488.0);
        let valido = payload_valido(&orden).unwrap();
        assert_eq!(valido["total"], 6739.2);
        assert_eq!(valido["pagos"], json!([{ "metodoPago": "tarjeta", "monto": 6739.2 }]));
        assert_eq!(valido["items"][1]["precioHistorico"], 1500.0);
    }

    #[test]
    fn payload_valido_ajusta_pagos_y_rechaza_vacios() {
        let sin_descuento = json!({
            "total": 100.004,
            "items": [{ "productoId": 1, "cantidad": 2, "subtotal": 100.0, "precioHistorico": 50.0, "costo": 30 }],
            "pagos": [{ "metodoPago": "efectivo", "monto": 40.0 }, { "metodoPago": "tarjeta", "monto": 50.0 }],
            "tieneDescuento": false,
        });
        let valido = payload_valido(&sin_descuento).unwrap();
        assert_eq!(valido["total"], 100.0);
        assert_eq!(valido["pagos"][0]["monto"], 50.0);
        let sin_precio = json!({ "items": [{ "productoId": 1, "cantidad": 4, "subtotal": 10.0 }] });
        assert_eq!(payload_valido(&sin_precio).unwrap()["items"][0]["precioHistorico"], 2.5);
        let vacio = json!({ "items": [{ "cantidad": 1, "subtotal": 10.0 }] });
        assert_eq!(payload_valido(&vacio).unwrap_err(), "No valid items to create order payload");
    }

    #[test]
    fn ticket_corrige_total_con_descuento() {
        let mut datos = json!({ "total": 22400.0, "subtotal": 22449.5, "discountData": { "type": "percentage", "value": 10.0, "amount": 49.5 } });
        ajustar_total_ticket(&mut datos);
        assert_eq!(datos["total"], 22400.0);
        let mut datos = json!({ "total": 22449.5, "subtotalSinDescuento": 22449.5, "discountData": { "type": "percentage", "value": 10.0, "amount": 0 } });
        ajustar_total_ticket(&mut datos);
        assert!((numero(&datos["total"]) - 20204.55).abs() < 1e-9);
    }

    #[test]
    fn payloads_de_qr() {
        let items = vec![producto(1, "Tomate perito", 1.0, 1000.0)];
        let u = usuario();
        let d = Descuento::porcentaje(10.0, 100.0);
        let qr = Qr { monto: 900.0, descripcion: "Compra de 1 productos".into(), usuario: &u, items: items_orden(&items, false), afip: false, mixto: false, efectivo: 0.0 }
            .payload(descuento_qr(Some(&d), 1000.0));
        assert_eq!(qr["requiresAfipInvoice"], false);
        assert_eq!(qr["afipData"], Value::Null);
        assert_eq!(qr["subtotalSinDescuento"], 1000.0);
        assert!(qr["items"][0].get("listaPrecioNombre").is_none());
        let afip = descuento_qr_afip(Some(&d), 1000.0, 100.0, true);
        assert_eq!(afip["descuento"]["montoDescuento"], 100.0);
        assert_eq!(afip["subtotal"], 1000.0);
        assert!((subtotal_mixto_afip(900.0, Some(&d)) - 1000.0).abs() < 1e-9);
        assert_eq!(subtotal_mixto_afip(900.0, Some(&Descuento::fijo(100.0, 100.0))), 1000.0);
        assert_eq!(estado_qr(&json!("approved")), "COMPLETADA");
        assert_eq!(estado_qr(&json!("REJECTED")), "CANCELADA");
        assert_eq!(estado_qr(&json!("pending")), "PENDIENTE");
        assert_eq!(estado_qr(&json!("en_proceso")), "EN_PROCESO");
    }

    #[test]
    fn factura_afip_y_datos_de_impresion() {
        let items = vec![producto(1, "Tomate perito", 1.0, 1000.0)];
        let negocio = json!({ "nombre": "Verdulería La Huerta", "configuracionAfip": { "tipoFactura": "B", "razonSocial": "La Huerta S.R.L.", "cuit": "30-71234567-8" } });
        let u = usuario();
        let afip = Afip {
            negocio: &negocio,
            usuario: &u,
            metodo: "split",
            items: items_afip(&items),
            total: 1000.0,
            subtotal_iva: 1000.0,
            descuento: None,
            mixto: Some((400.0, "tarjeta", 600.0)),
        };
        let payload = afip.payload();
        assert_eq!(payload["tipoFactura"], "B");
        assert_eq!(payload["productos"][0]["precio"], 1000.0);
        assert_eq!(payload["pagos"][1], json!({ "metodoPago": "tarjeta", "monto": 600.0 }));
        let resultado = json!({
            "message": "Factura AFIP creada correctamente",
            "factura": { "id": 88, "numero": "0003-00000045", "numeroFactura": "45", "puntoVenta": "0003", "vencimientoCae": "2026-10-20T03:00:00.000Z" },
            "afip": { "cae": "75412369899123", "vencimientoCae": "20261020" },
        });
        let impresion = afip.impresion(&resultado, "10/10/2026 12:00:00");
        assert_eq!(impresion["cae"], "75412369899123");
        assert_eq!(impresion["razonSocial"], "La Huerta S.R.L.");
        assert_eq!(impresion["businessName"], "Verdulería La Huerta");
        assert_eq!(impresion["numeroFactura"], "45");
        assert_eq!(impresion["puntoVenta"], "0003");
        assert_eq!(impresion["fechaVtoCae"], "20261020");
        assert_eq!(impresion["tipoFactura"], "FACTURA B");
        assert_eq!(impresion["direccion"], "Dirección no configurada");
        assert_eq!(impresion["tieneDescuento"], false);
        assert_eq!(impresion["afipQrUrl"], Value::Null);
        let vieja = afip.impresion(&json!({ "afip": { "cae": "1", "fechaVtoCae": "20261101" } }), "");
        assert_eq!(vieja["fechaVtoCae"], "20261101");
        assert!(afip.impresion(&json!({ "afip": { "cae": "1" } }), "").get("fechaVtoCae").is_none());
        assert_eq!(impresion["pagos"][0]["monto"], 400.0);
    }

    #[test]
    fn descuento_de_pago_en_efectivo() {
        let elegido = Descuento::porcentaje(5.0, 1122.48);
        assert_eq!(descuento_pago_efectivo(Some(elegido), false, None, 1149.5), Some(elegido.con_monto(1149.5)));
        assert_eq!(descuento_pago_efectivo(Some(elegido), true, Some(10.0), 1149.5), Some(elegido.con_monto(1149.5)));
        assert_eq!(descuento_pago_efectivo(None, true, Some(10.0), 2249.5), Some(Descuento::porcentaje(10.0, 2249.5)));
        assert_eq!(descuento_pago_efectivo(None, false, Some(10.0), 49.5), None);
        assert_eq!(descuento_pago_efectivo(None, true, None, 49.5), None);
    }

    #[test]
    fn sondeo_con_limite_de_tiempo() {
        assert!(!sondeo_vencido(100.0, 699.9, false));
        assert!(sondeo_vencido(100.0, 700.0, false));
        assert!(!sondeo_vencido(100.0, 399.9, true));
        assert!(sondeo_vencido(100.0, 400.0, true));
    }

    #[test]
    fn clave_manual() {
        assert!(clave_manual_valida("secreta", Some("secreta")));
        assert!(!clave_manual_valida("otra", Some("secreta")));
        assert!(!clave_manual_valida("", Some("secreta")));
        assert!(!clave_manual_valida("", Some("")));
        assert!(!clave_manual_valida("cualquiera", None));
    }

    #[test]
    fn cancelacion_y_motivo() {
        let items = vec![producto(1, "Tomate perito", 1.245, 2400.0)];
        let cuerpo = cancelacion(&json!({ "id": 7 }), &usuario(), "TEMP-1", "  cliente se fue ", 2988.0, productos_cancelacion(&items));
        assert_eq!(cuerpo["businessId"], 7);
        assert_eq!(cuerpo["motivo"], "cliente se fue");
        assert_eq!(cuerpo["productos"][0]["nombreProducto"], "Tomate perito");
        assert_eq!(cancelacion(&json!({}), &json!({ "id": 3 }), "T", "abc", 1.0, vec![])["sucursalId"], 1);
        assert_eq!(validar_motivo("  "), Err("Debes ingresar un motivo para la cancelación"));
        assert_eq!(validar_motivo("ab "), Err("El motivo debe tener al menos 3 caracteres"));
        assert_eq!(validar_motivo("abc"), Ok(()));
    }

    #[test]
    fn fechas() {
        let utc = DateTime::parse_from_rfc3339("2026-01-05T09:03:07.120Z").unwrap().with_timezone(&Utc);
        assert_eq!(fecha_iso(utc), "2026-01-05T09:03:07.120Z");
        let local_ = Local.with_ymd_and_hms(2026, 1, 5, 9, 3, 7).unwrap();
        assert_eq!(fecha_es_ar(local_), "05/01/2026 09:03:07");
    }
}
