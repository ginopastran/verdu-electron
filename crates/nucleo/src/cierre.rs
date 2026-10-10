use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde_json::{json, Map, Value};

pub const MESES: [&str; 12] = [
    "enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre",
];

pub fn argentina() -> FixedOffset {
    FixedOffset::west_opt(3 * 3600).expect("UTC-3")
}

pub fn hora_argentina<Tz: TimeZone>(instante: &DateTime<Tz>) -> NaiveDateTime {
    instante.with_timezone(&argentina()).naive_local()
}

pub fn ahora_argentina() -> NaiveDateTime {
    hora_argentina(&Utc::now())
}

pub fn rango_periodo<Tz: TimeZone>(periodo: &str, ahora: &DateTime<Tz>) -> (NaiveDateTime, NaiveDateTime) {
    let fin = hora_argentina(ahora);
    let hora_inicio = if periodo == "tarde" { 12 } else { 0 };
    let mut inicio = fin.date().and_hms_opt(hora_inicio, 0, 0).expect("hora válida");
    if inicio >= fin {
        inicio -= Duration::days(1);
    }
    (inicio, fin)
}

pub fn iso(fecha: &NaiveDateTime) -> String {
    fecha.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn cuerpo_cierre(usuario: &Value, periodo: &str, inicio: &NaiveDateTime, fin: &NaiveDateTime) -> Value {
    let mut cuerpo = Map::new();
    for (clave, origen) in [("vendedorId", "id"), ("sucursalId", "sucursalId")] {
        if let Some(valor) = usuario.get(origen) {
            cuerpo.insert(clave.into(), valor.clone());
        }
    }
    cuerpo.insert("fechaInicio".into(), json!(iso(inicio)));
    cuerpo.insert("fechaCierre".into(), json!(iso(fin)));
    cuerpo.insert("periodo".into(), json!(periodo));
    Value::Object(cuerpo)
}

pub fn id_texto(valor: Option<&Value>) -> String {
    match valor {
        Some(Value::Number(n)) => texto_numero(n.as_f64().unwrap_or(f64::NAN)),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

pub fn texto_numero(valor: f64) -> String {
    if valor.is_nan() {
        "NaN".into()
    } else if valor.fract() == 0.0 && valor.abs() < 1e21 {
        format!("{}", valor as i64)
    } else {
        format!("{valor}")
    }
}

fn numero_js(texto: &str) -> Value {
    let recortado = texto.trim();
    let valor = if recortado.is_empty() { Some(0.0) } else { recortado.parse::<f64>().ok() };
    match valor {
        Some(v) if v.is_finite() && v.fract() == 0.0 => json!(v as i64),
        Some(v) if v.is_finite() => json!(v),
        _ => Value::Null,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hora {
    pub hora: u32,
    pub minuto: u32,
}

impl Hora {
    pub fn texto(self) -> String {
        format!("{:02}:{:02}", self.hora, self.minuto)
    }
}

pub fn validar_manual(
    inicio: NaiveDate,
    fin: NaiveDate,
    vendedor_id: &str,
    sucursal_id: &str,
    hora_inicio: Hora,
    hora_fin: Hora,
) -> Result<(), &'static str> {
    if vendedor_id.is_empty() || sucursal_id.is_empty() {
        return Err("Por favor, completa todos los campos.");
    }
    if fin < inicio {
        return Err("La fecha de fin no puede ser anterior a la fecha de inicio.");
    }
    if inicio == fin && hora_inicio >= hora_fin {
        return Err("La hora de fin debe ser posterior a la hora de inicio.");
    }
    Ok(())
}

pub fn rango_manual(inicio: NaiveDate, hora_inicio: Hora, fin: NaiveDate, hora_fin: Hora) -> (NaiveDateTime, NaiveDateTime) {
    let desde = inicio.and_hms_milli_opt(hora_inicio.hora, hora_inicio.minuto, 0, 0).expect("hora válida");
    let hasta = fin.and_hms_milli_opt(hora_fin.hora, hora_fin.minuto, 59, 999).expect("hora válida");
    (desde, hasta)
}

pub fn cuerpo_manual(vendedor_id: &str, sucursal_id: &str, inicio: &NaiveDateTime, fin: &NaiveDateTime) -> Value {
    json!({
        "vendedorId": numero_js(vendedor_id),
        "sucursalId": numero_js(sucursal_id),
        "fechaInicio": iso(inicio),
        "fechaCierre": iso(fin),
        "periodo": "custom",
    })
}

pub fn vendedores(usuarios: &Value) -> Vec<Value> {
    usuarios
        .as_array()
        .map(|lista| lista.iter().filter(|u| u["rol"]["nombre"] == "VENDEDOR").cloned().collect())
        .unwrap_or_default()
}

pub fn nombre_por_id(lista: &[Value], id: &str) -> Option<String> {
    lista
        .iter()
        .find(|item| id_texto(item.get("id")) == id)
        .and_then(|item| item["nombre"].as_str())
        .filter(|nombre| !nombre.is_empty())
        .map(str::to_string)
}

pub fn fecha_larga(fecha: NaiveDate) -> String {
    format!("{} de {} de {}", fecha.day(), MESES[fecha.month0() as usize], fecha.year())
}

pub fn fecha_literal(texto: &str) -> Option<NaiveDateTime> {
    if let Ok(fecha) = DateTime::parse_from_rfc3339(texto) {
        return Some(fecha.naive_local());
    }
    for formato in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M:%S%.f"] {
        if let Ok(fecha) = NaiveDateTime::parse_from_str(texto, formato) {
            return Some(fecha);
        }
    }
    NaiveDate::parse_from_str(texto, "%Y-%m-%d").ok().and_then(|dia| dia.and_hms_opt(0, 0, 0))
}

pub fn fecha_argentina(texto: &str) -> String {
    fecha_literal(texto).map(|fecha| fecha.format("%d/%m/%Y %H:%M").to_string()).unwrap_or_else(|| "Fecha inválida".into())
}

pub fn numero(valor: Option<&Value>) -> f64 {
    match valor {
        None => f64::NAN,
        Some(Value::Null) => 0.0,
        Some(Value::Bool(b)) => f64::from(u8::from(*b)),
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::String(s)) => {
            let recortado = s.trim();
            if recortado.is_empty() {
                0.0
            } else {
                recortado.parse().unwrap_or(f64::NAN)
            }
        }
        Some(_) => f64::NAN,
    }
}

pub fn verdadero(valor: Option<&Value>) -> bool {
    match valor {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|v| v != 0.0 && !v.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvisoImpresion {
    Exito,
    ErrorImpresora(String),
    ErrorGeneral(String),
}

pub fn aviso_impresion(resultado: &Value) -> AvisoImpresion {
    let error_impresora = verdadero(resultado.get("printerError"));
    if verdadero(resultado.get("success")) && !error_impresora {
        AvisoImpresion::Exito
    } else if error_impresora {
        AvisoImpresion::ErrorImpresora(texto_valor(&resultado["printerError"]))
    } else {
        let mensaje = Some(&resultado["message"]).filter(|m| verdadero(Some(m))).map(texto_valor);
        AvisoImpresion::ErrorGeneral(mensaje.unwrap_or_else(|| "Desconocido".into()))
    }
}

pub fn texto_valor(valor: &Value) -> String {
    match valor {
        Value::String(s) => s.clone(),
        Value::Number(n) => texto_numero(n.as_f64().unwrap_or(f64::NAN)),
        Value::Null => "null".into(),
        otro => otro.to_string(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FalloCierre {
    MananaRequerida,
    Mensaje(String),
}

pub fn fallo_cierre(estado_ok: bool, cuerpo: &Value) -> Option<FalloCierre> {
    if estado_ok {
        return None;
    }
    if cuerpo["error"] == "ERROR_CIERRE_MAÑANA_REQUERIDO" {
        return Some(FalloCierre::MananaRequerida);
    }
    let mensaje = Some(&cuerpo["message"]).filter(|m| verdadero(Some(m))).map(texto_valor);
    Some(FalloCierre::Mensaje(mensaje.unwrap_or_else(|| "Error al registrar el cierre".into())))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn en_argentina(texto: &str) -> DateTime<FixedOffset> {
        argentina().from_local_datetime(&NaiveDateTime::parse_from_str(texto, "%Y-%m-%d %H:%M:%S%.f").unwrap()).unwrap()
    }

    fn en_utc(texto: &str) -> DateTime<Utc> {
        Utc.from_utc_datetime(&NaiveDateTime::parse_from_str(texto, "%Y-%m-%d %H:%M:%S%.f").unwrap())
    }

    #[test]
    fn periodo_manana_va_de_medianoche_a_ahora_en_hora_argentina() {
        let (inicio, fin) = rango_periodo("mañana", &en_argentina("2026-10-10 09:37:00.619"));
        assert_eq!(iso(&inicio), "2026-10-10T00:00:00.000Z");
        assert_eq!(iso(&fin), "2026-10-10T09:37:00.619Z");
    }

    #[test]
    fn el_rango_no_depende_del_huso_de_la_pc() {
        let argentina = rango_periodo("mañana", &en_argentina("2026-10-10 09:37:00.619"));
        assert_eq!(rango_periodo("mañana", &en_utc("2026-10-10 12:37:00.619")), argentina);
        let madrid = FixedOffset::east_opt(2 * 3600).unwrap();
        assert_eq!(rango_periodo("mañana", &en_utc("2026-10-10 12:37:00.619").with_timezone(&madrid)), argentina);
        let (inicio, fin) = rango_periodo("todo", &en_utc("2026-10-11 01:30:00.000"));
        assert_eq!(iso(&inicio), "2026-10-10T00:00:00.000Z");
        assert_eq!(iso(&fin), "2026-10-10T22:30:00.000Z");
    }

    #[test]
    fn periodo_tarde_arranca_al_mediodia() {
        let (inicio, fin) = rango_periodo("tarde", &en_argentina("2026-10-10 18:05:10.000"));
        assert_eq!(iso(&inicio), "2026-10-10T12:00:00.000Z");
        assert_eq!(iso(&fin), "2026-10-10T18:05:10.000Z");
    }

    #[test]
    fn periodo_todo_usa_medianoche() {
        let (inicio, _) = rango_periodo("todo", &en_argentina("2026-10-10 20:00:00.000"));
        assert_eq!(iso(&inicio), "2026-10-10T00:00:00.000Z");
    }

    #[test]
    fn tarde_antes_del_mediodia_corre_el_inicio_al_dia_anterior() {
        let (inicio, fin) = rango_periodo("tarde", &en_argentina("2026-10-10 10:00:00.000"));
        assert_eq!(iso(&inicio), "2026-10-09T12:00:00.000Z");
        assert_eq!(iso(&fin), "2026-10-10T10:00:00.000Z");
    }

    #[test]
    fn cuerpo_del_cierre_igual_al_del_hook() {
        let usuario = json!({ "id": 11, "nombre": "Lucía", "sucursalId": 1 });
        let (inicio, fin) = rango_periodo("mañana", &en_argentina("2026-10-10 09:37:00.619"));
        let cuerpo = cuerpo_cierre(&usuario, "mañana", &inicio, &fin);
        assert_eq!(
            cuerpo.to_string(),
            r#"{"vendedorId":11,"sucursalId":1,"fechaInicio":"2026-10-10T00:00:00.000Z","fechaCierre":"2026-10-10T09:37:00.619Z","periodo":"mañana"}"#
        );
    }

    #[test]
    fn cuerpo_del_cierre_omite_lo_que_el_usuario_no_tiene() {
        let (inicio, fin) = rango_periodo("todo", &en_argentina("2026-10-10 09:00:00.000"));
        let cuerpo = cuerpo_cierre(&json!({ "id": 3 }), "todo", &inicio, &fin);
        assert!(cuerpo.get("sucursalId").is_none());
        assert_eq!(cuerpo["vendedorId"], 3);
    }

    #[test]
    fn rango_manual_en_hora_argentina_y_cierra_al_segundo_59() {
        let dia = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        let (inicio, fin) = rango_manual(dia, Hora { hora: 0, minuto: 0 }, dia, Hora { hora: 23, minuto: 59 });
        assert_eq!(iso(&inicio), "2026-10-10T00:00:00.000Z");
        assert_eq!(iso(&fin), "2026-10-10T23:59:59.999Z");
        let cuerpo = cuerpo_manual("11", "1", &inicio, &fin);
        assert_eq!(
            cuerpo.to_string(),
            r#"{"vendedorId":11,"sucursalId":1,"fechaInicio":"2026-10-10T00:00:00.000Z","fechaCierre":"2026-10-10T23:59:59.999Z","periodo":"custom"}"#
        );
    }

    #[test]
    fn validaciones_del_cierre_manual() {
        let hoy = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        let ayer = hoy - Duration::days(1);
        let cero = Hora { hora: 0, minuto: 0 };
        let ahora = Hora { hora: 15, minuto: 42 };
        let fin = Hora { hora: 23, minuto: 59 };
        assert_eq!(validar_manual(hoy, hoy, "11", "", cero, fin), Err("Por favor, completa todos los campos."));
        assert_eq!(validar_manual(hoy, ayer, "11", "1", cero, fin), Err("La fecha de fin no puede ser anterior a la fecha de inicio."));
        assert_eq!(validar_manual(hoy, hoy, "11", "1", fin, cero), Err("La hora de fin debe ser posterior a la hora de inicio."));
        assert_eq!(validar_manual(hoy, hoy, "11", "1", cero, cero), Err("La hora de fin debe ser posterior a la hora de inicio."));
        assert_eq!(validar_manual(hoy, hoy, "11", "1", Hora { hora: 15, minuto: 43 }, ahora), Err("La hora de fin debe ser posterior a la hora de inicio."));
        assert_eq!(validar_manual(hoy, hoy, "11", "1", cero, ahora), Ok(()));
        assert_eq!(validar_manual(ayer, hoy, "11", "1", fin, cero), Ok(()));
    }

    #[test]
    fn filtra_vendedores_y_busca_nombres() {
        let usuarios = json!([
            { "id": 11, "nombre": "Lucía", "rol": { "nombre": "VENDEDOR" } },
            { "id": 1, "nombre": "Roberto", "rol": { "nombre": "ADMIN" } }
        ]);
        let lista = vendedores(&usuarios);
        assert_eq!(lista.len(), 1);
        assert_eq!(nombre_por_id(&lista, "11").as_deref(), Some("Lucía"));
        assert_eq!(nombre_por_id(&lista, "1"), None);
    }

    #[test]
    fn fechas_en_castellano_y_en_formato_argentino() {
        assert_eq!(fecha_larga(NaiveDate::from_ymd_opt(2026, 10, 10).unwrap()), "10 de octubre de 2026");
        assert_eq!(fecha_argentina("2026-10-10T15:23:37.370Z"), "10/10/2026 15:23");
        assert_eq!(fecha_argentina("2026-10-10T01:05:00.000Z"), "10/10/2026 01:05");
        assert_eq!(fecha_argentina("2026-10-10"), "10/10/2026 00:00");
        assert_eq!(fecha_argentina("basura"), "Fecha inválida");
    }

    #[test]
    fn avisos_de_impresion() {
        assert_eq!(aviso_impresion(&json!({ "success": true, "printerError": null })), AvisoImpresion::Exito);
        assert_eq!(
            aviso_impresion(&json!({ "success": false, "printerError": "sin papel", "message": "x" })),
            AvisoImpresion::ErrorImpresora("sin papel".into())
        );
        assert_eq!(aviso_impresion(&json!({ "success": false })), AvisoImpresion::ErrorGeneral("Desconocido".into()));
    }

    #[test]
    fn errores_del_servidor() {
        assert_eq!(fallo_cierre(true, &json!({})), None);
        assert_eq!(fallo_cierre(false, &json!({ "error": "ERROR_CIERRE_MAÑANA_REQUERIDO" })), Some(FalloCierre::MananaRequerida));
        assert_eq!(fallo_cierre(false, &json!({ "message": "Ya existe" })), Some(FalloCierre::Mensaje("Ya existe".into())));
        assert_eq!(fallo_cierre(false, &json!({})), Some(FalloCierre::Mensaje("Error al registrar el cierre".into())));
    }
}
