use chrono::{DateTime, Utc};
use nucleo::api::Cliente;
use nucleo::busqueda::ItemBusqueda;
use nucleo::carrito::{self, Producto};
use nucleo::{entorno, formato};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProductoDisponible {
    pub id: i64,
    pub nombre: String,
    pub precio: f64,
    pub unidad: String,
    pub costo: f64,
    pub codigo_barras: Option<String>,
    pub plu: Option<String>,
    pub iva_incluido: Option<bool>,
    pub iva_porcentaje: Option<f64>,
    pub oferta_porcentaje: Option<f64>,
    pub oferta_hasta: Option<DateTime<Utc>>,
    pub foto: Option<String>,
    pub precio_libre: bool,
    pub alias: Vec<String>,
    pub busqueda: ItemBusqueda,
}

impl AsRef<ItemBusqueda> for ProductoDisponible {
    fn as_ref(&self) -> &ItemBusqueda {
        &self.busqueda
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilaLista {
    pub lista_precio_id: i64,
    pub precio: f64,
    pub nombre_lista: String,
    pub tipo_medida: Option<String>,
}

fn numero(valor: &Value) -> f64 {
    match valor {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse().unwrap_or(f64::NAN),
        Value::Bool(b) => f64::from(u8::from(*b)),
        _ => 0.0,
    }
}

fn texto_opcional(valor: &Value) -> Option<String> {
    match valor {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

impl ProductoDisponible {
    pub fn desde_backend(p: &Value) -> Self {
        let producto = ProductoDisponible {
            id: p["id"].as_i64().unwrap_or(0),
            nombre: p["nombre"].as_str().unwrap_or_default().to_string(),
            precio: numero(&p["precio"]),
            unidad: p["tipoMedida"].as_str().unwrap_or_default().to_string(),
            costo: numero(&p["costo"]),
            codigo_barras: texto_opcional(&p["codigoBarras"]),
            plu: texto_opcional(&p["plu"]),
            iva_incluido: p["ivaIncluido"].as_bool(),
            iva_porcentaje: p["ivaPorcentaje"].as_f64(),
            oferta_porcentaje: p["ofertaPorcentaje"].as_f64(),
            oferta_hasta: carrito::fecha_utc(&p["ofertaHasta"]),
            foto: p["fotoActualizadaEn"].as_str().filter(|f| !f.is_empty()).map(str::to_string),
            precio_libre: p["precioLibre"].as_bool().unwrap_or(false),
            alias: p["alias"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_string)).collect(),
            busqueda: ItemBusqueda::default(),
        };
        producto.indexado()
    }

    pub fn indexado(mut self) -> Self {
        self.busqueda = ItemBusqueda::nuevo(&self.nombre, &self.alias, self.codigo_barras.as_deref(), self.plu.as_deref());
        self
    }

    pub fn oferta(&self, ahora: DateTime<Utc>) -> Option<f64> {
        carrito::oferta_vigente(self.oferta_porcentaje, self.oferta_hasta, ahora)
    }

    pub fn precio_venta(&self, ahora: DateTime<Utc>) -> f64 {
        self.oferta(ahora).map_or(self.precio, |p| carrito::precio_con_oferta(self.precio, p))
    }

    pub fn unidad_o_defecto(&self) -> String {
        if self.unidad.is_empty() { "Unidad".to_string() } else { self.unidad.clone() }
    }

    pub fn item(&self, cantidad: f64, unidad: String, precio: f64, lista: Option<(i64, Option<String>)>, etiqueta_base: &str) -> Producto {
        let oferta = self.oferta(Utc::now()).filter(|_| lista.is_none() && precio != self.precio);
        let (lista_precio_id, lista_precio_nombre) = match lista {
            Some((id, nombre)) => (Some(id), nombre),
            None => (None, Some(etiqueta_base.to_string())),
        };
        Producto {
            id: self.id,
            cart_id: id_unico(self.id),
            name: self.nombre.clone(),
            quantity: cantidad,
            unit: unidad,
            price_per_unit: precio,
            subtotal: redondear(precio * cantidad, 2),
            costo: self.costo,
            iva_incluido: self.iva_incluido,
            iva_porcentaje: self.iva_porcentaje,
            lista_precio_id,
            lista_precio_nombre,
            precio_sin_oferta: oferta.map(|_| self.precio),
            oferta_porcentaje: oferta,
            precio_libre: false,
        }
    }

    pub fn item_libre(&self, monto: f64) -> Producto {
        Producto {
            id: self.id,
            cart_id: id_unico(self.id),
            name: self.nombre.clone(),
            quantity: 1.0,
            unit: "Unidad".into(),
            price_per_unit: monto,
            subtotal: redondear(monto, 2),
            costo: self.costo,
            iva_incluido: Some(true),
            iva_porcentaje: self.iva_porcentaje,
            precio_libre: true,
            ..Default::default()
        }
    }
}

pub fn redondear(valor: f64, decimales: i32) -> f64 {
    let factor = 10f64.powi(decimales);
    (valor * factor).round() / factor
}

pub fn id_unico(producto: i64) -> String {
    use std::hash::{BuildHasher, Hasher};
    let milis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let mut azar = std::collections::hash_map::RandomState::new().build_hasher();
    azar.write_u128(milis);
    let mut sufijo = String::new();
    let mut n = azar.finish();
    for _ in 0..8 {
        sufijo.push(char::from_digit((n % 36) as u32, 36).unwrap_or('0'));
        n /= 36;
    }
    format!("{producto}-{milis}-{sufijo}")
}

fn encabezados() -> Vec<(&'static str, &'static str)> {
    entorno::APP_ID.map(|id| vec![("X-App-ID", id)]).unwrap_or_default()
}

pub fn descargar_todos(api: &Cliente) -> Result<Vec<ProductoDisponible>, String> {
    let limite = 150;
    let mut pagina = 1;
    let mut todos = Vec::new();
    loop {
        let respuesta = api
            .pedir("GET", &format!("/api/productos?limit={limite}&page={pagina}"), None, &encabezados())
            .map_err(|_| "Failed to fetch".to_string())?;
        if respuesta.estado == 401 || respuesta.estado == 403 {
            return Err("No autorizado: inicia sesión nuevamente".into());
        }
        if !respuesta.ok() {
            return Err(format!("HTTP {}", respuesta.estado));
        }
        let datos = &respuesta.cuerpo;
        let pagina_actual = datos["productos"].as_array().or(datos["data"].as_array());
        todos.extend(pagina_actual.into_iter().flatten().map(ProductoDisponible::desde_backend));
        if !datos["pagination"]["hasNext"].as_bool().unwrap_or(false) {
            return Ok(todos);
        }
        pagina += 1;
    }
}

pub fn listas_de_precios(api: &Cliente, producto: i64) -> Option<Vec<FilaLista>> {
    let mut encabezados = encabezados();
    encabezados.insert(0, ("Content-Type", "application/json"));
    let respuesta = api.pedir("GET", &format!("/api/productos/{producto}/listas-precios"), None, &encabezados).ok()?;
    if !respuesta.ok() {
        return None;
    }
    let filas = respuesta.cuerpo["productos"].as_array().cloned().unwrap_or_default();
    Some(
        filas
            .iter()
            .filter(|x| x["activa"].as_bool().unwrap_or(false) && x["listaPrecio"]["activa"].as_bool().unwrap_or(false))
            .map(|x| {
                let id = x["listaPrecioId"].as_i64().unwrap_or(0);
                FilaLista {
                    lista_precio_id: id,
                    precio: numero(&x["precio"]),
                    nombre_lista: x["listaPrecio"]["nombre"].as_str().map(str::to_string).unwrap_or_else(|| format!("Lista #{id}")),
                    tipo_medida: x["listaPrecio"]["tipoMedida"].as_str().or(x["tipoMedida"].as_str()).map(str::to_string),
                }
            })
            .collect(),
    )
}

pub const TOPE_MONTO_LIBRE: f64 = 10_000_000.0;
pub const RAFAGA_LECTOR: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RevisionMonto {
    Valido(f64),
    Invalido,
    Codigo,
    ConfirmarTope(f64),
}

pub fn revisar_monto(texto: &str, rafaga: bool, tope_confirmado: bool) -> RevisionMonto {
    let monto = parse_float(texto);
    if rafaga && texto.len() >= 8 && texto.bytes().all(|b| b.is_ascii_digit()) {
        return RevisionMonto::Codigo;
    }
    if !monto.is_finite() || monto <= 0.0 {
        return RevisionMonto::Invalido;
    }
    let monto = redondear(monto, 2);
    if monto <= 0.0 {
        RevisionMonto::Invalido
    } else if monto > TOPE_MONTO_LIBRE && !tope_confirmado {
        RevisionMonto::ConfirmarTope(monto)
    } else {
        RevisionMonto::Valido(monto)
    }
}

pub fn es_codigo_directo(valor: &str) -> bool {
    (8..=13).contains(&valor.len()) && valor.bytes().all(|b| b.is_ascii_digit())
}

pub fn interpretar_codigo(codigo: &str, productos: &[ProductoDisponible]) -> Option<(usize, f64)> {
    if codigo.is_empty() || productos.is_empty() {
        return None;
    }
    let solo_digitos = codigo.bytes().all(|b| b.is_ascii_digit());
    if solo_digitos && (8..=15).contains(&codigo.len()) {
        let exacto = productos.iter().position(|p| p.codigo_barras.as_deref() == Some(codigo));
        let parecido = || {
            productos.iter().position(|p| {
                p.codigo_barras.as_deref().is_some_and(|c| !c.is_empty() && (c.contains(codigo) || codigo.contains(c)))
            })
        };
        if let Some(indice) = exacto.or_else(parecido) {
            return Some((indice, 1.0));
        }
    }
    if !solo_digitos {
        return None;
    }
    let (plu, gramos) = match codigo.len() {
        12 => (&codigo[0..3], &codigo[3..11]),
        13 if codigo.starts_with('0') => (&codigo[1..4], &codigo[4..12]),
        _ => return None,
    };
    let kilos = gramos.parse::<f64>().ok()? / 1000.0;
    if !(0.001..=999.999).contains(&kilos) {
        return None;
    }
    let plu: f64 = plu.parse().ok()?;
    let indice = productos.iter().position(|p| p.plu.as_deref().and_then(numero_de_texto) == Some(plu))?;
    Some((indice, kilos))
}

fn numero_de_texto(texto: &str) -> Option<f64> {
    let limpio = texto.trim();
    if limpio.is_empty() { Some(0.0) } else { limpio.parse().ok() }
}

pub fn monto(valor: f64) -> String {
    formato::numero_con(valor, 0, 2)
}

pub fn numero_js(valor: f64) -> String {
    if valor.is_finite() && valor == valor.trunc() && valor.abs() < 1e21 {
        format!("{}", valor as i64)
    } else {
        format!("{valor}")
    }
}

pub fn texto_peso(peso: &Value) -> String {
    match peso {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.as_f64().map(formato::numero).unwrap_or_else(|| n.to_string()),
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        otro => otro.to_string(),
    }
}

pub fn gramos_de_peso(peso: &Value) -> f64 {
    match peso {
        Value::String(s) if s.trim().is_empty() => 0.0,
        Value::Null => 0.0,
        otro => numero(otro),
    }
}

pub fn parse_float(texto: &str) -> f64 {
    let t = texto.trim_start();
    let mut fin = 0;
    let mut punto = false;
    let mut digitos = false;
    for (i, c) in t.char_indices() {
        match c {
            '+' | '-' if i == 0 => {}
            '0'..='9' => digitos = true,
            '.' if !punto => punto = true,
            _ => break,
        }
        fin = i + c.len_utf8();
    }
    if !digitos {
        return f64::NAN;
    }
    t[..fin].trim_end_matches('.').parse().unwrap_or(f64::NAN)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn producto(id: i64, nombre: &str, plu: Option<&str>, codigo: Option<&str>) -> ProductoDisponible {
        ProductoDisponible {
            id,
            nombre: nombre.into(),
            precio: 1000.0,
            unidad: "Kg".into(),
            plu: plu.map(Into::into),
            codigo_barras: codigo.map(Into::into),
            ..Default::default()
        }
        .indexado()
    }

    fn catalogo() -> Vec<ProductoDisponible> {
        vec![
            producto(1, "Tomate perito", Some("001"), None),
            producto(5, "Manzana roja", Some("005"), None),
            producto(7, "Mandarina", Some("007"), None),
            producto(21, "Huevos blancos maple x30", None, Some("7791234500033")),
            producto(20, "Ananá", None, Some("7791234500026")),
            producto(13, "Batata", Some("013"), None),
        ]
    }

    #[test]
    fn el_catalogo_queda_indexado_con_alias() {
        let p = ProductoDisponible::desde_backend(&serde_json::json!({ "id": 9, "nombre": "Precinto plástico 200 mm", "codigoBarras": "0077900123456", "alias": ["Brida", "Sujetacables"] }));
        assert_eq!(p.alias, ["Brida", "Sujetacables"]);
        let lista = [p, ProductoDisponible::desde_backend(&serde_json::json!({ "id": 4, "nombre": "Papa negra", "plu": "103" }))];
        let ids = |q: &str| nucleo::busqueda::buscar(&lista, q).resultados.iter().map(|r| lista[r.indice].id).collect::<Vec<_>>();
        assert_eq!(ids("sujetacables"), [9]);
        assert_eq!(ids("precinto 200"), [9]);
        assert_eq!(ids("0077900123456"), [9]);
        assert_eq!(ids("103"), [4]);
        assert!(ProductoDisponible::desde_backend(&serde_json::json!({ "id": 1, "nombre": "Sin alias" })).alias.is_empty());
    }

    #[test]
    fn codigos_de_barras_y_plu_con_peso() {
        let p = catalogo();
        assert_eq!(interpretar_codigo("7791234500026", &p), Some((4, 1.0)));
        assert_eq!(interpretar_codigo("005000013552", &p), Some((1, 1.355)));
        assert_eq!(interpretar_codigo("0005000013552", &p), Some((1, 1.355)));
        assert_eq!(interpretar_codigo("013000000002", &p), None);
        assert_eq!(interpretar_codigo("999000013552", &p), None);
        assert_eq!(interpretar_codigo("tomate", &p), None);
        assert!(es_codigo_directo("77912345"));
        assert!(!es_codigo_directo("7791234500026123"));
    }

    #[test]
    fn oferta_del_listado_baja_el_precio_base_y_queda_en_el_item() {
        let p = ProductoDisponible::desde_backend(&serde_json::json!({
            "id": 3, "nombre": "Tomate", "precio": 2890, "tipoMedida": "Kg",
            "ofertaPorcentaje": 20, "ofertaHasta": "2999-01-01T00:00:00.000Z", "fotoActualizadaEn": "2026-10-10T12:00:00.000Z"
        }));
        let ahora = Utc::now();
        assert_eq!(p.oferta(ahora), Some(20.0));
        assert_eq!(p.precio_venta(ahora), 2312.0);
        assert_eq!(p.foto.as_deref(), Some("2026-10-10T12:00:00.000Z"));
        let item = p.item(2.0, "Kg".into(), p.precio_venta(ahora), None, "Precio catálogo");
        assert_eq!((item.price_per_unit, item.subtotal, item.precio_sin_oferta, item.oferta_porcentaje), (2312.0, 4624.0, Some(2890.0), Some(20.0)));
        let con_lista = p.item(1.0, "Kg".into(), 2500.0, Some((4, Some("Mayorista".into()))), "Precio catálogo");
        assert_eq!((con_lista.price_per_unit, con_lista.precio_sin_oferta, con_lista.oferta_porcentaje), (2500.0, None, None));
        let viejo = ProductoDisponible::desde_backend(&serde_json::json!({ "id": 4, "nombre": "Papa", "precio": 1150 }));
        assert_eq!((viejo.precio_venta(ahora), viejo.foto.clone()), (1150.0, None));
        assert_eq!(viejo.item(1.0, "Kg".into(), 1150.0, None, "Precio catálogo").precio_sin_oferta, None);
    }

    #[test]
    fn precio_libre_arma_una_linea_de_una_unidad_con_el_monto() {
        let p = ProductoDisponible::desde_backend(&serde_json::json!({
            "id": 9, "nombre": "Varios", "precio": 0, "tipoMedida": "Kg", "precioLibre": true,
            "ofertaPorcentaje": 20, "ofertaHasta": "2999-01-01T00:00:00.000Z", "ivaIncluido": false, "ivaPorcentaje": 21
        }));
        assert!(p.precio_libre);
        let item = p.item_libre(1234.5);
        assert_eq!((item.quantity, item.price_per_unit, item.subtotal, item.unit.as_str()), (1.0, 1234.5, 1234.5, "Unidad"));
        assert!(item.precio_libre);
        assert_eq!((item.precio_sin_oferta, item.oferta_porcentaje, item.lista_precio_id), (None, None, None));
        let comun = ProductoDisponible::desde_backend(&serde_json::json!({ "id": 4, "nombre": "Papa", "precio": 1150 }));
        assert!(!comun.precio_libre);
        assert!(!comun.item(1.0, "Kg".into(), 1150.0, None, "Precio catálogo").precio_libre);
    }

    #[test]
    fn el_monto_libre_rechaza_codigos_escaneados_y_pide_confirmar_montos_enormes() {
        assert_eq!(revisar_monto("2500.556", false, false), RevisionMonto::Valido(2500.56));
        assert_eq!(revisar_monto("7791234000010", true, false), RevisionMonto::Codigo);
        assert_eq!(revisar_monto("7791234000010", false, false), RevisionMonto::ConfirmarTope(7791234000010.0));
        assert_eq!(revisar_monto("7791234000010", false, true), RevisionMonto::Valido(7791234000010.0));
        assert_eq!(revisar_monto("1500", true, false), RevisionMonto::Valido(1500.0));
        assert_eq!(revisar_monto("12000000.50", true, false), RevisionMonto::ConfirmarTope(12000000.5));
        assert_eq!(revisar_monto("10000000", false, false), RevisionMonto::Valido(10000000.0));
        assert_eq!(revisar_monto("0", false, false), RevisionMonto::Invalido);
        assert_eq!(revisar_monto("0.001", false, false), RevisionMonto::Invalido);
        assert_eq!(revisar_monto("", false, false), RevisionMonto::Invalido);
    }

    #[test]
    fn numeros_en_formato_argentino() {
        assert_eq!(monto(22449.5), "22.449,5");
        assert_eq!(monto(2652.0), "2.652");
        assert_eq!(monto(1234.5675), "1.234,57");
        assert_eq!(texto_peso(&serde_json::json!(1245)), "1.245");
        assert_eq!(numero_js(1.0), "1");
        assert_eq!(numero_js(0.415), "0.415");
        assert_eq!(parse_float("1750"), 1750.0);
        assert_eq!(parse_float("3abc"), 3.0);
        assert!(parse_float("").is_nan());
    }
}
