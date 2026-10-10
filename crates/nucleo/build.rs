fn main() {
    let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env.local");
    println!("cargo:rerun-if-changed={}", ruta.display());
    let Ok(contenido) = std::fs::read_to_string(&ruta) else {
        return;
    };
    for linea in contenido.lines().map(str::trim) {
        if linea.starts_with('#') {
            continue;
        }
        if let Some((clave, valor)) = linea.split_once('=') {
            if clave.starts_with("VITE_") {
                println!("cargo:rustc-env={}={}", clave.trim(), valor.trim().trim_matches('"'));
            }
        }
    }
}
