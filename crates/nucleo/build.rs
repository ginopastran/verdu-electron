fn main() {
    let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env.local");
    println!("cargo:rerun-if-changed={}", ruta.display());
    println!("cargo:rerun-if-env-changed=VITE_API_URL");
    let mut api_url = std::env::var("VITE_API_URL").unwrap_or_default();
    for linea in std::fs::read_to_string(&ruta).unwrap_or_default().lines().map(str::trim) {
        if linea.starts_with('#') {
            continue;
        }
        if let Some((clave, valor)) = linea.split_once('=') {
            if clave.starts_with("VITE_") {
                let (clave, valor) = (clave.trim(), valor.trim().trim_matches('"'));
                if clave == "VITE_API_URL" {
                    api_url = valor.to_string();
                }
                println!("cargo:rustc-env={clave}={valor}");
            }
        }
    }
    let es_release = std::env::var("PROFILE").as_deref() == Ok("release");
    let es_local = ["localhost", "127.0.0.1", "0.0.0.0"].iter().any(|host| api_url.contains(host));
    if es_release && (!api_url.starts_with("https://") || es_local) {
        panic!("Un build release necesita VITE_API_URL de producción (https) en .env.local; vale \"{api_url}\"");
    }
}
