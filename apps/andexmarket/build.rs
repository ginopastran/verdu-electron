fn main() {
    println!("cargo:rerun-if-changed=../../empaquetado/icono.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut recurso = winresource::WindowsResource::new();
    recurso
        .set_icon("../../empaquetado/icono.ico")
        .set("ProductName", "AndexMarket")
        .set("FileDescription", "AndexMarket")
        .set("CompanyName", "AndexTech")
        .set("LegalCopyright", "AndexTech")
        .set("OriginalFilename", "AndexMarket.exe");
    recurso.compile().expect("no se pudo compilar el ícono y los datos de versión del ejecutable");
}
