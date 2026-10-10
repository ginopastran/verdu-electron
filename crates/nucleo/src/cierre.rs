use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde_json::{json, Map, Value};

pub const MESES: [&str; 12] = [
    "enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre",
];

pub fn argentina() -> FixedOffset {
    FixedOffset::west_opt(3 * 3600).expect("UTC-3")
}

fn tres_horas() -> Duration {
    Duration::hours(3)
}

fn local_a_utc<Tz: TimeZone>(zona: &Tz, fecha: NaiveDateTime) -> DateTime<Utc> {
    zona.from_local_datetime(&fecha)
        .earliest()
        .unwrap_or_else(|| zona.from_utc_datetime(&fecha))
        .with_timezone(&Utc)
}

pub fn rango_periodo<Tz: TimeZone>(periodo: &str, ahora: &DateTime<Tz>) -> (DateTime<Utc>, DateTime<Utc>) {
    let fin = ahora.with_timezone(&Utc) - tres_horas();
    let hora_inicio = if periodo == "tarde" { 12 } else { 0 };
    let dia = ahora.date_naive().and_hms_opt(hora_inicio, 0, 0).expect("hora válida");
    let mut inicio = local_a_utc(&ahora.timezone(), dia) - tres_horas();
    if inicio >= fin {
        inicio -= Duration::days(1);
    }
    (inicio, fin)
}

pub fn iso(fecha: &DateTime<Utc>) -> String {
    fecha.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn cuerpo_cierre(usuario: &Value, periodo: &str, inicio: &DateTime<Utc>, fin: &DateTime<Utc>) -> Value {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    inicio: Option<NaiveDateTime>,
    fin: Option<NaiveDateTime>,
    vendedor_id: &str,
    sucursal_id: &str,
    hora_inicio: Hora,
    hora_fin: Hora,
) -> Result<(NaiveDate, NaiveDate), &'static str> {
    let (Some(inicio), Some(fin)) = (inicio, fin) else {
        return Err("Por favor, completa todos los campos.");
    };
    if vendedor_id.is_empty() || sucursal_id.is_empty() {
        return Err("Por favor, completa todos los campos.");
    }
    if fin < inicio {
        return Err("La fecha de fin no puede ser anterior a la fecha de inicio.");
    }
    if inicio == fin && (hora_inicio.hora, hora_inicio.minuto) >= (hora_fin.hora, hora_fin.minuto) {
        return Err("La hora de fin debe ser posterior a la hora de inicio.");
    }
    Ok((inicio.date(), fin.date()))
}

pub fn rango_manual<Tz: TimeZone>(
    zona: &Tz,
    inicio: NaiveDate,
    hora_inicio: Hora,
    fin: NaiveDate,
    hora_fin: Hora,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let desde = inicio.and_hms_milli_opt(hora_inicio.hora, hora_inicio.minuto, 0, 0).expect("hora válida");
    let hasta = fin.and_hms_milli_opt(hora_fin.hora, hora_fin.minuto, 59, 999).expect("hora válida");
    (local_a_utc(zona, desde) - tres_horas(), local_a_utc(zona, hasta) - tres_horas())
}

pub fn cuerpo_manual(vendedor_id: &str, sucursal_id: &str, inicio: &DateTime<Utc>, fin: &DateTime<Utc>) -> Value {
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

pub fn parsear_fecha(texto: &str) -> Option<DateTime<Utc>> {
    if let Ok(fecha) = DateTime::parse_from_rfc3339(texto) {
        return Some(fecha.with_timezone(&Utc));
    }
    for formato in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M:%S%.f"] {
        if let Ok(fecha) = NaiveDateTime::parse_from_str(texto, formato) {
            return Some(local_a_utc(&chrono::Local, fecha));
        }
    }
    NaiveDate::parse_from_str(texto, "%Y-%m-%d")
        .ok()
        .map(|dia| Utc.from_utc_datetime(&dia.and_hms_opt(0, 0, 0).expect("medianoche")))
}

pub fn fecha_argentina_en<Tz: TimeZone>(fecha: &DateTime<Utc>, zona: &Tz) -> String {
    let corrida = (*fecha + tres_horas()).with_timezone(zona);
    corrida.naive_local().format("%d/%m/%Y, %H:%M").to_string()
}

pub fn fecha_argentina(texto: &str) -> String {
    parsear_fecha(texto)
        .map(|fecha| fecha_argentina_en(&fecha, &chrono::Local))
        .unwrap_or_else(|| "Invalid Date".into())
}

pub fn miles(valor: f64) -> String {
    if valor.is_nan() {
        return "NaN".into();
    }
    if valor.is_infinite() {
        return if valor > 0.0 { "∞".into() } else { "-∞".into() };
    }
    let redondeado = (valor.abs() * 1000.0).round() / 1000.0;
    let texto = format!("{redondeado:.3}");
    let (entera, decimal) = texto.split_once('.').unwrap_or((&texto, ""));
    let decimal = decimal.trim_end_matches('0');
    let mut agrupada = String::new();
    for (indice, digito) in entera.chars().enumerate() {
        if indice > 0 && (entera.len() - indice) % 3 == 0 {
            agrupada.push(',');
        }
        agrupada.push(digito);
    }
    let signo = if valor < 0.0 && redondeado != 0.0 { "-" } else { "" };
    if decimal.is_empty() {
        format!("{signo}{agrupada}")
    } else {
        format!("{signo}{agrupada}.{decimal}")
    }
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

    #[test]
    fn periodo_manana_va_de_medianoche_a_ahora_corridos_tres_horas() {
        let (inicio, fin) = rango_periodo("mañana", &en_argentina("2026-10-10 09:37:00.619"));
        assert_eq!(iso(&inicio), "2026-10-10T00:00:00.000Z");
        assert_eq!(iso(&fin), "2026-10-10T09:37:00.619Z");
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
    fn rango_manual_resta_tres_horas_y_cierra_al_segundo_59() {
        let dia = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        let (inicio, fin) = rango_manual(&argentina(), dia, Hora { hora: 0, minuto: 0 }, dia, Hora { hora: 23, minuto: 59 });
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
        let ahora = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap().and_hms_opt(15, 0, 0).unwrap();
        let ayer = ahora - Duration::days(1);
        let cero = Hora { hora: 0, minuto: 0 };
        let fin = Hora { hora: 23, minuto: 59 };
        assert_eq!(validar_manual(None, Some(ahora), "11", "1", cero, fin), Err("Por favor, completa todos los campos."));
        assert_eq!(validar_manual(Some(ahora), Some(ahora), "11", "", cero, fin), Err("Por favor, completa todos los campos."));
        assert_eq!(
            validar_manual(Some(ahora), Some(ayer), "11", "1", cero, fin),
            Err("La fecha de fin no puede ser anterior a la fecha de inicio.")
        );
        assert_eq!(
            validar_manual(Some(ahora), Some(ahora), "11", "1", fin, cero),
            Err("La hora de fin debe ser posterior a la hora de inicio.")
        );
        assert_eq!(validar_manual(Some(ahora), Some(ahora), "11", "1", cero, cero), Err("La hora de fin debe ser posterior a la hora de inicio."));
        assert!(validar_manual(Some(ayer), Some(ahora), "11", "1", fin, cero).is_ok());
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
        let fecha = parsear_fecha("2026-10-10T15:23:37.370Z").unwrap();
        assert_eq!(fecha_argentina_en(&fecha, &argentina()), "10/10/2026, 15:23");
    }

    #[test]
    fn miles_imita_to_locale_string() {
        assert_eq!(miles(186450.0), "186,450");
        assert_eq!(miles(8500.0), "8,500");
        assert_eq!(miles(1234.5678), "1,234.568");
        assert_eq!(miles(-500.0), "-500");
        assert_eq!(miles(0.0), "0");
        assert_eq!(miles(f64::NAN), "NaN");
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
