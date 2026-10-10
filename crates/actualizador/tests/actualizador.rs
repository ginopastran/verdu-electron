use std::io::Cursor;

use actualizador::{es_mas_nueva, leer_manifiesto, nombre_instalador, verificar, CLAVE_PUBLICA};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use minisign::KeyPair;
use serde_json::json;

const DATOS_FIRMADOS: &[u8] = b"AndexMarket: archivo de prueba para verificar la firma del actualizador.\n";
const FIRMA_CLAVE_REAL: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUanYyNzZPR2lJK2I0OW0zVGtVeml3a1JZSDZQazZuNjlFakVJVk00bC9XdFVKeUJsU3dMZ1ZIY293NWhreTdPY0FaTWl3dm9Ra2xiOUJFVzd6OWtjdUJOd0ZkUytSWkFJPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkxNjQ3NDU2CWZpbGU6cHJ1ZWJhLnR4dAoxSFZIUExHVHRCd0xZNzVmbHI0MFN2MjRjUUxZbWNMZ3BxK1AvUVZlUmVnaGFFRE5Mc1ZvTDhOWlhLdmVkbll3bldsenVKVnFEODVqYm1yUkF3ajhDdz09Cg==";

fn firmar_con_clave_nueva(datos: &[u8]) -> (String, String) {
    let par = KeyPair::generate_unencrypted_keypair().unwrap();
    let firma = minisign::sign(Some(&par.pk), &par.sk, Cursor::new(datos), Some("timestamp:0\tfile:x.exe"), None).unwrap();
    (STANDARD.encode(par.pk.to_box().unwrap().to_string()), STANDARD.encode(firma.to_string()))
}

#[test]
fn acepta_la_firma_hecha_con_la_clave_privada_de_andex() {
    assert_eq!(verificar(DATOS_FIRMADOS, FIRMA_CLAVE_REAL, CLAVE_PUBLICA), Ok(()));
}

#[test]
fn rechaza_datos_alterados() {
    let mut alterados = DATOS_FIRMADOS.to_vec();
    alterados[0] ^= 1;
    assert!(verificar(&alterados, FIRMA_CLAVE_REAL, CLAVE_PUBLICA).is_err());
}

#[test]
fn rechaza_instaladores_firmados_con_otra_clave() {
    let (clave_ajena, firma) = firmar_con_clave_nueva(DATOS_FIRMADOS);
    assert_eq!(verificar(DATOS_FIRMADOS, &firma, &clave_ajena), Ok(()));
    assert!(verificar(DATOS_FIRMADOS, &firma, CLAVE_PUBLICA).is_err());
}

#[test]
fn rechaza_firmas_mal_formadas() {
    assert!(verificar(DATOS_FIRMADOS, "no-es-base64", CLAVE_PUBLICA).is_err());
    assert!(verificar(DATOS_FIRMADOS, &STANDARD.encode("texto cualquiera"), CLAVE_PUBLICA).is_err());
    assert!(verificar(DATOS_FIRMADOS, "", CLAVE_PUBLICA).is_err());
}

#[test]
fn compara_versiones() {
    assert!(es_mas_nueva("4.1.0", "4.0.0"));
    assert!(es_mas_nueva("v4.0.1", "4.0.0"));
    assert!(es_mas_nueva("4.10.0", "4.9.9"));
    assert!(es_mas_nueva("5.0.0", "4.99.99"));
    assert!(es_mas_nueva("4.1.0", "4.1.0-beta.1"));
    assert!(!es_mas_nueva("4.0.0", "4.0.0"));
    assert!(!es_mas_nueva("3.9.9", "4.0.0"));
    assert!(!es_mas_nueva("4.1.0-beta.1", "4.1.0"));
    assert!(!es_mas_nueva("4.1", "4.0.0"));
    assert!(!es_mas_nueva("4.1.0.1", "4.0.0"));
    assert!(!es_mas_nueva("cuatro", "4.0.0"));
}

#[test]
fn lee_latest_json_de_tauri() {
    let manifiesto = json!({
        "version": "4.1.0",
        "notes": "AndexMarket 4.1.0",
        "pub_date": "2026-10-10T15:00:00.000Z",
        "platforms": {
            "windows-x86_64": {
                "signature": "firma",
                "url": "https://github.com/ginopastran/verdu-electron/releases/download/v4.1.0/AndexMarket_4.1.0_x64-setup.exe"
            }
        }
    });
    let publicacion = leer_manifiesto(&manifiesto, "4.0.0").unwrap().unwrap();
    assert_eq!(publicacion.version, "4.1.0");
    assert_eq!(publicacion.firma, "firma");
    assert_eq!(publicacion.notas.as_deref(), Some("AndexMarket 4.1.0"));
    assert_eq!(nombre_instalador(&publicacion), "AndexMarket_4.1.0_x64-setup.exe");
    assert_eq!(leer_manifiesto(&manifiesto, "4.1.0"), Ok(None));
    assert!(leer_manifiesto(&json!({ "version": "4.2.0", "platforms": {} }), "4.0.0").is_err());
    assert!(leer_manifiesto(&json!({ "version": "nueva" }), "4.0.0").is_err());
}
