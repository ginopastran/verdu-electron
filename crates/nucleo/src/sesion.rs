use serde_json::{json, Map, Value};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB};

use crate::api::{Cliente, Respuesta};
use crate::entorno;

pub const LARGO_MINIMO_CONTRASENA: usize = 4;
pub const ERROR_CONTRASENA_CORTA: &str = "La contraseña debe tener al menos 4 caracteres.";
pub const ERROR_RED: &str = "No se pudo conectar con el servidor";

pub fn clave_negocio(business_id: i64) -> String {
    format!("businessInfo_{business_id}")
}

pub fn validar_contrasena(contrasena: &str) -> Result<(), &'static str> {
    if contrasena.encode_utf16().count() < LARGO_MINIMO_CONTRASENA {
        Err(ERROR_CONTRASENA_CORTA)
    } else {
        Ok(())
    }
}

pub fn es_verdadero(valor: &Value) -> bool {
    match valor {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

pub fn es_admin(usuario: &Value) -> bool {
    usuario["rol"]["nombre"] == "ADMIN"
}

pub fn ruta_protegida_rechaza_admin(usuario: &Value) -> bool {
    usuario["role"] == "ADMIN" || es_admin(usuario)
}

fn dpapi(datos: &[u8], cifrar: bool) -> Result<Vec<u8>, String> {
    let entrada = CRYPT_INTEGER_BLOB { cbData: datos.len() as u32, pbData: datos.as_ptr() as *mut u8 };
    let mut salida = CRYPT_INTEGER_BLOB::default();
    unsafe {
        if cifrar {
            CryptProtectData(&entrada, PCWSTR::null(), None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut salida)
        } else {
            CryptUnprotectData(&entrada, None, None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut salida)
        }
        .map_err(|e| e.to_string())?;
        let bytes = std::slice::from_raw_parts(salida.pbData, salida.cbData as usize).to_vec();
        LocalFree(Some(HLOCAL(salida.pbData.cast())));
        Ok(bytes)
    }
}

pub fn cifrar(texto: &str) -> Result<String, String> {
    Ok(dpapi(texto.as_bytes(), true)?.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn descifrar(cifrado: &str) -> Result<String, String> {
    let bytes = (0..cifrado.len())
        .step_by(2)
        .map(|i| cifrado.get(i..i + 2).and_then(|par| u8::from_str_radix(par, 16).ok()))
        .collect::<Option<Vec<u8>>>()
        .ok_or("Contraseña cifrada inválida")?;
    String::from_utf8(dpapi(&bytes, false)?).map_err(|e| e.to_string())
}

pub fn normalizar_usuario(mut usuario: Value) -> Value {
    let Some(objeto) = usuario.as_object_mut() else {
        return usuario;
    };
    if !objeto.get("permisos").is_some_and(es_verdadero) {
        objeto.insert("permisos".into(), Value::Object(Map::new()));
    }
    for permiso in ["pesoManualEnabled", "cierreDeCajaEnabled"] {
        if let Some(valor) = objeto.shift_remove(permiso) {
            if let Some(permisos) = objeto.get_mut("permisos").and_then(Value::as_object_mut) {
                permisos.insert(permiso.into(), valor);
            }
        }
    }
    usuario
}

pub fn business_id_de_admin(sesion: &Value) -> Result<Value, String> {
    let usuario = &sesion["user"];
    if !es_verdadero(usuario) || !es_admin(usuario) {
        return Err("Solo los administradores pueden configurar la aplicación".into());
    }
    [&usuario["businessId"], &usuario["sucursal"]["businessId"]]
        .into_iter()
        .find(|v| es_verdadero(v))
        .cloned()
        .ok_or_else(|| "No se pudo obtener el ID del negocio del administrador".into())
}

pub fn nombre_negocio(negocio: &Value) -> String {
    [&negocio["nombre"], &negocio["name"]]
        .into_iter()
        .find(|v| es_verdadero(v))
        .map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string()))
        .unwrap_or_else(|| "Negocio".into())
}

pub fn armar_admin(usuario: &Value, business_id: Value, business_name: &str) -> Value {
    json!({
        "id": usuario["id"],
        "nombre": usuario["nombre"],
        "email": usuario["email"],
        "businessId": business_id,
        "businessName": business_name,
    })
}

pub fn enriquecer_negocio(mut negocio: Value) -> Value {
    let principal = negocio["sucursales"].as_array().filter(|s| !s.is_empty()).map(|sucursales| {
        sucursales.iter().find(|s| es_verdadero(&s["esPrincipal"])).unwrap_or(&sucursales[0]).clone()
    });
    if let (Some(principal), Some(objeto)) = (principal, negocio.as_object_mut()) {
        for campo in ["direccion", "telefono"] {
            if es_verdadero(&principal[campo]) {
                objeto.insert(campo.into(), principal[campo].clone());
            }
        }
        objeto.insert("sucursalActiva".into(), principal);
    }
    if !es_verdadero(&negocio["sistemaPago"]) {
        if let Some(objeto) = negocio.as_object_mut() {
            objeto.insert("sistemaPago".into(), json!("redondeo"));
        }
    }
    negocio
}

fn mensaje_o(cuerpo: &Value, defecto: &str) -> String {
    match &cuerpo["message"] {
        Value::String(s) if !s.is_empty() => s.clone(),
        otro if es_verdadero(otro) => otro.to_string(),
        _ => defecto.to_string(),
    }
}

fn pedir(api: &Cliente, metodo: &str, ruta: &str, cuerpo: Option<&Value>, encabezados: &[(&str, String)]) -> Result<Respuesta, String> {
    let mut todos: Vec<(&str, &str)> = encabezados.iter().map(|(c, v)| (*c, v.as_str())).collect();
    if let Some(app_id) = entorno::APP_ID {
        todos.push(("X-App-ID", app_id));
    }
    api.pedir(metodo, ruta, cuerpo, &todos).map_err(|_| ERROR_RED.to_string())
}

pub fn iniciar(api: &Cliente, email: &str, contrasena: &str) -> Result<String, String> {
    let respuesta = pedir(api, "POST", "/api/login", Some(&json!({ "email": email, "password": contrasena })), &[])?;
    if !respuesta.ok() {
        return Err(mensaje_o(&respuesta.cuerpo, "Error en el login"));
    }
    match &respuesta.cuerpo["token"] {
        Value::String(token) if !token.is_empty() => Ok(token.clone()),
        _ => Err("No se recibió el token de autenticación".into()),
    }
}

pub fn autorizar_admin(base: &str, email: &str, contrasena: &str, business_id: i64) -> Result<(), String> {
    let aparte = Cliente::new(base);
    let respuesta = pedir(&aparte, "POST", "/api/login", Some(&json!({ "email": email.trim(), "password": contrasena })), &[])?;
    if !respuesta.ok() {
        return Err(mensaje_o(&respuesta.cuerpo, "Credenciales incorrectas"));
    }
    let usuario = &respuesta.cuerpo["user"];
    if !es_admin(usuario) {
        return Err("El usuario no es administrador".into());
    }
    if usuario["businessId"].as_i64() != Some(business_id) {
        return Err("El administrador no pertenece a este negocio".into());
    }
    Ok(())
}

pub fn pedir_sesion(api: &Cliente, token: &str) -> Result<Value, String> {
    let encabezados = [("Authorization", format!("Bearer {token}")), ("Cache-Control", "no-cache".to_string())];
    let respuesta = pedir(api, "GET", "/api/auth/session", None, &encabezados)?;
    if !respuesta.ok() {
        return Err(mensaje_o(&respuesta.cuerpo, "Error al obtener la sesión"));
    }
    Ok(respuesta.cuerpo)
}

pub fn configurar_negocio(api: &Cliente, email: &str, contrasena: &str) -> Result<Value, String> {
    let token = iniciar(api, email, contrasena)?;
    let sesion = pedir_sesion(api, &token)?;
    let business_id = business_id_de_admin(&sesion)?;
    let ruta = format!("/api/business/{}", business_id.as_str().map(str::to_string).unwrap_or_else(|| business_id.to_string()));
    let respuesta = pedir(api, "GET", &ruta, None, &[("Authorization", format!("Bearer {token}"))])?;
    let nombre = if respuesta.ok() { nombre_negocio(&respuesta.cuerpo) } else { "Negocio".to_string() };
    Ok(armar_admin(&sesion["user"], business_id, &nombre))
}

pub fn iniciar_vendedor(api: &Cliente, email: &str, contrasena: &str) -> Result<Value, String> {
    let token = iniciar(api, email, contrasena)?;
    let sesion = pedir_sesion(api, &token)?;
    let usuario = &sesion["user"];
    if !es_verdadero(usuario) {
        return Err("No se pudo obtener la información del usuario".into());
    }
    if !es_verdadero(&usuario["rol"]) {
        return Err("El usuario no tiene un rol asignado".into());
    }
    Ok(normalizar_usuario(usuario.clone()))
}

pub fn vendedores(api: &Cliente, business_id: i64) -> Result<Vec<Value>, String> {
    let respuesta = pedir(api, "GET", &format!("/api/usuarios/vendedores?businessId={business_id}"), None, &[])?;
    if !respuesta.ok() {
        return Err("Error al obtener vendedores".into());
    }
    Ok(respuesta.cuerpo.as_array().cloned().unwrap_or_default())
}

pub fn pedir_negocio(api: &Cliente, business_id: i64) -> Result<Value, String> {
    let ruta = format!("/api/business/{business_id}?include=sucursales,configuracionAfip");
    let respuesta = pedir(api, "GET", &ruta, None, &[])?;
    if !respuesta.ok() {
        return Err("Error al cargar información del negocio".into());
    }
    Ok(enriquecer_negocio(respuesta.cuerpo))
}

#[cfg(test)]
mod pruebas {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn contrasena_minima_de_cuatro() {
        assert_eq!(validar_contrasena("12"), Err(ERROR_CONTRASENA_CORTA));
        assert_eq!(validar_contrasena("ñañ"), Err(ERROR_CONTRASENA_CORTA));
        assert_eq!(validar_contrasena("1234"), Ok(()));
    }

    #[test]
    fn admin_rechazado_por_role_o_por_rol_nombre() {
        assert!(ruta_protegida_rechaza_admin(&json!({ "role": "ADMIN" })));
        assert!(ruta_protegida_rechaza_admin(&json!({ "rol": { "nombre": "ADMIN" } })));
        assert!(!ruta_protegida_rechaza_admin(&json!({ "rol": { "nombre": "VENDEDOR" } })));
    }

    #[test]
    fn la_contrasena_se_cifra_con_dpapi_y_vuelve() {
        let cifrada = cifrar("clave-ñandú").unwrap();
        assert!(!cifrada.contains("clave"));
        assert_eq!(descifrar(&cifrada).unwrap(), "clave-ñandú");
        assert!(descifrar("zz").is_err());
        assert!(descifrar("abc").is_err());
    }

    #[test]
    fn permisos_se_mueven_adentro() {
        let usuario = normalizar_usuario(json!({ "id": 1, "pesoManualEnabled": true, "cierreDeCajaEnabled": null, "rol": { "nombre": "VENDEDOR" } }));
        assert_eq!(usuario["permisos"], json!({ "pesoManualEnabled": true, "cierreDeCajaEnabled": null }));
        assert!(usuario.get("pesoManualEnabled").is_none());
        assert!(usuario.get("cierreDeCajaEnabled").is_none());
        let sin_permisos = normalizar_usuario(json!({ "permisos": { "pesoManualEnabled": false } }));
        assert_eq!(sin_permisos["permisos"], json!({ "pesoManualEnabled": false }));
    }

    #[test]
    fn admin_valida_rol_y_negocio() {
        let vendedor = json!({ "user": { "rol": { "nombre": "VENDEDOR" }, "businessId": 7 } });
        assert_eq!(business_id_de_admin(&vendedor).unwrap_err(), "Solo los administradores pueden configurar la aplicación");
        let sin_negocio = json!({ "user": { "rol": { "nombre": "ADMIN" }, "businessId": 0 } });
        assert_eq!(business_id_de_admin(&sin_negocio).unwrap_err(), "No se pudo obtener el ID del negocio del administrador");
        let por_sucursal = json!({ "user": { "rol": { "nombre": "ADMIN" }, "sucursal": { "businessId": 9 } } });
        assert_eq!(business_id_de_admin(&por_sucursal).unwrap(), json!(9));
        assert_eq!(nombre_negocio(&json!({ "name": "X" })), "X");
        assert_eq!(nombre_negocio(&json!({ "nombre": "" })), "Negocio");
    }

    #[test]
    fn negocio_toma_la_sucursal_principal() {
        let negocio = enriquecer_negocio(json!({
            "direccion": "vieja",
            "sucursales": [
                { "id": 1, "direccion": "uno", "telefono": "" },
                { "id": 2, "direccion": "dos", "telefono": "222", "esPrincipal": true }
            ]
        }));
        assert_eq!(negocio["direccion"], "dos");
        assert_eq!(negocio["telefono"], "222");
        assert_eq!(negocio["sucursalActiva"]["id"], 2);
        assert_eq!(negocio["sistemaPago"], "redondeo");
        let sin_principal = enriquecer_negocio(json!({ "telefono": "fijo", "sistemaPago": "pago-exacto", "sucursales": [{ "id": 1, "telefono": "" }] }));
        assert_eq!(sin_principal["telefono"], "fijo");
        assert_eq!(sin_principal["sucursalActiva"]["id"], 1);
        assert_eq!(sin_principal["sistemaPago"], "pago-exacto");
    }

    fn servidor(respuestas: Vec<(u16, Value)>) -> (String, mpsc::Receiver<String>) {
        let oyente = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", oyente.local_addr().unwrap());
        let (emisor, receptor) = mpsc::channel();
        std::thread::spawn(move || {
            for (estado, cuerpo) in respuestas {
                let (mut conexion, _) = oyente.accept().unwrap();
                let mut pedido = Vec::new();
                let mut bufer = [0u8; 4096];
                loop {
                    let leidos = conexion.read(&mut bufer).unwrap();
                    pedido.extend_from_slice(&bufer[..leidos]);
                    let texto = String::from_utf8_lossy(&pedido).to_string();
                    if let Some(fin) = texto.find("\r\n\r\n") {
                        let largo = texto
                            .lines()
                            .find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap()))
                            .unwrap_or(0);
                        if pedido.len() >= fin + 4 + largo || leidos == 0 {
                            break;
                        }
                    }
                }
                emisor.send(String::from_utf8_lossy(&pedido).to_string()).unwrap();
                let texto = cuerpo.to_string();
                let respuesta = format!(
                    "HTTP/1.1 {estado} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}",
                    texto.len()
                );
                conexion.write_all(respuesta.as_bytes()).unwrap();
            }
        });
        (base, receptor)
    }

    #[test]
    fn configurar_negocio_sigue_los_tres_pedidos() {
        let (base, pedidos) = servidor(vec![
            (200, json!({ "token": "t-1" })),
            (200, json!({ "user": { "id": 1, "nombre": "Roberto", "email": "a@b.c", "rol": { "nombre": "ADMIN" }, "businessId": 7 } })),
            (200, json!({ "id": 7, "nombre": "La Huerta" })),
        ]);
        let admin = configurar_negocio(&Cliente::new(&base), "a@b.c", "secreto").unwrap();
        assert_eq!(admin, json!({ "id": 1, "nombre": "Roberto", "email": "a@b.c", "businessId": 7, "businessName": "La Huerta" }));
        let login = pedidos.recv().unwrap();
        assert!(login.starts_with("POST /api/login "));
        assert!(login.ends_with(r#"{"email":"a@b.c","password":"secreto"}"#));
        let sesion = pedidos.recv().unwrap().to_lowercase();
        assert!(sesion.starts_with("get /api/auth/session "));
        assert!(sesion.contains("authorization: bearer t-1") && sesion.contains("cache-control: no-cache"));
        let negocio = pedidos.recv().unwrap().to_lowercase();
        assert!(negocio.starts_with("get /api/business/7 ") && negocio.contains("authorization: bearer t-1"));
    }

    #[test]
    fn autorizar_admin_valida_rol_y_negocio() {
        let admin = |rol: &str, negocio: i64| (200, json!({ "token": "t-admin", "user": { "id": 1, "rol": { "nombre": rol }, "businessId": negocio } }));
        let (base, pedidos) = servidor(vec![admin("ADMIN", 7), admin("VENDEDOR", 7), admin("ADMIN", 9), (401, json!({ "message": "Credenciales incorrectas" }))]);
        assert_eq!(autorizar_admin(&base, " jefe@b.c ", "secreto", 7), Ok(()));
        let login = pedidos.recv().unwrap();
        assert!(login.starts_with("POST /api/login "));
        assert!(login.ends_with(r#"{"email":"jefe@b.c","password":"secreto"}"#));
        assert_eq!(autorizar_admin(&base, "v@b.c", "x", 7).unwrap_err(), "El usuario no es administrador");
        assert_eq!(autorizar_admin(&base, "otro@b.c", "x", 7).unwrap_err(), "El administrador no pertenece a este negocio");
        assert_eq!(autorizar_admin(&base, "jefe@b.c", "mal", 7).unwrap_err(), "Credenciales incorrectas");
    }

    #[test]
    fn login_rechazado_usa_el_mensaje_del_servidor() {
        let (base, _pedidos) = servidor(vec![(401, json!({ "message": "Credenciales inválidas" }))]);
        assert_eq!(iniciar_vendedor(&Cliente::new(&base), "x@y.z", "1234").unwrap_err(), "Credenciales inválidas");
        let (base, _pedidos) = servidor(vec![(500, json!({ "error": "Error interno" }))]);
        assert_eq!(vendedores(&Cliente::new(&base), 7).unwrap_err(), "Error al obtener vendedores");
    }
}
