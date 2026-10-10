use eframe::egui::{self, Color32, Image, Response, Sense, Ui, Vec2};

const ICONOS_SOLAR: &[(&str, &[u8])] = &[
    ("alert", include_bytes!("../../../../assets/iconos-solar/alert.svg")),
    ("arrow-left", include_bytes!("../../../../assets/iconos-solar/arrow-left.svg")),
    ("barcode", include_bytes!("../../../../assets/iconos-solar/barcode.svg")),
    ("bill-check", include_bytes!("../../../../assets/iconos-solar/bill-check.svg")),
    ("calendar", include_bytes!("../../../../assets/iconos-solar/calendar.svg")),
    ("card", include_bytes!("../../../../assets/iconos-solar/card.svg")),
    ("cart", include_bytes!("../../../../assets/iconos-solar/cart.svg")),
    ("cart-plus", include_bytes!("../../../../assets/iconos-solar/cart-plus.svg")),
    ("cash", include_bytes!("../../../../assets/iconos-solar/cash.svg")),
    ("check", include_bytes!("../../../../assets/iconos-solar/check.svg")),
    ("chevron-down", include_bytes!("../../../../assets/iconos-solar/chevron-down.svg")),
    ("chevron-left", include_bytes!("../../../../assets/iconos-solar/chevron-left.svg")),
    ("chevron-right", include_bytes!("../../../../assets/iconos-solar/chevron-right.svg")),
    ("circle-check", include_bytes!("../../../../assets/iconos-solar/circle-check.svg")),
    ("clock-time", include_bytes!("../../../../assets/iconos-solar/clock-time.svg")),
    ("doc", include_bytes!("../../../../assets/iconos-solar/doc.svg")),
    ("dollar-sign", include_bytes!("../../../../assets/iconos-solar/dollar-sign.svg")),
    ("download", include_bytes!("../../../../assets/iconos-solar/download.svg")),
    ("eye", include_bytes!("../../../../assets/iconos-solar/eye.svg")),
    ("file-text", include_bytes!("../../../../assets/iconos-solar/file-text.svg")),
    ("hand-coins", include_bytes!("../../../../assets/iconos-solar/hand-coins.svg")),
    ("history", include_bytes!("../../../../assets/iconos-solar/history.svg")),
    ("info", include_bytes!("../../../../assets/iconos-solar/info.svg")),
    ("loader", include_bytes!("../../../../assets/iconos-solar/loader.svg")),
    ("lock", include_bytes!("../../../../assets/iconos-solar/lock.svg")),
    ("log-out", include_bytes!("../../../../assets/iconos-solar/log-out.svg")),
    ("mail", include_bytes!("../../../../assets/iconos-solar/mail.svg")),
    ("moon", include_bytes!("../../../../assets/iconos-solar/moon.svg")),
    ("percent", include_bytes!("../../../../assets/iconos-solar/percent.svg")),
    ("plus", include_bytes!("../../../../assets/iconos-solar/plus.svg")),
    ("printer", include_bytes!("../../../../assets/iconos-solar/printer.svg")),
    ("qr", include_bytes!("../../../../assets/iconos-solar/qr.svg")),
    ("receipt", include_bytes!("../../../../assets/iconos-solar/receipt.svg")),
    ("scale", include_bytes!("../../../../assets/iconos-solar/scale.svg")),
    ("search", include_bytes!("../../../../assets/iconos-solar/search.svg")),
    ("settings", include_bytes!("../../../../assets/iconos-solar/settings.svg")),
    ("shield", include_bytes!("../../../../assets/iconos-solar/shield.svg")),
    ("split", include_bytes!("../../../../assets/iconos-solar/split.svg")),
    ("square-pen", include_bytes!("../../../../assets/iconos-solar/square-pen.svg")),
    ("store", include_bytes!("../../../../assets/iconos-solar/store.svg")),
    ("sun", include_bytes!("../../../../assets/iconos-solar/sun.svg")),
    ("tag", include_bytes!("../../../../assets/iconos-solar/tag.svg")),
    ("transfer", include_bytes!("../../../../assets/iconos-solar/transfer.svg")),
    ("trash-2", include_bytes!("../../../../assets/iconos-solar/trash-2.svg")),
    ("user", include_bytes!("../../../../assets/iconos-solar/user.svg")),
    ("wallet", include_bytes!("../../../../assets/iconos-solar/wallet.svg")),
    ("x", include_bytes!("../../../../assets/iconos-solar/x.svg")),
];

const IMAGENES: &[(&str, &[u8])] = &[
    ("andex-market-logo.png", include_bytes!("../../../../assets/imagenes/andex-market-logo.png")),
    ("andex-market-text.png", include_bytes!("../../../../assets/imagenes/andex-market-text.png")),
];

pub fn registrar(ctx: &egui::Context) {
    for (nombre, bytes) in ICONOS_SOLAR {
        let svg = String::from_utf8_lossy(bytes).replace("currentColor", "#ffffff");
        ctx.include_bytes(format!("bytes://solar/{nombre}.svg"), svg.into_bytes());
    }
    for (nombre, bytes) in IMAGENES {
        ctx.include_bytes(format!("bytes://imagen/{nombre}"), *bytes);
    }
}

pub fn imagen(nombre: &str) -> Image<'static> {
    Image::new(format!("bytes://imagen/{nombre}"))
}

pub fn icono_solar(nombre: &str, tam: f32, color: Color32) -> Image<'static> {
    Image::new(format!("bytes://solar/{nombre}.svg")).fit_to_exact_size(Vec2::splat(tam)).tint(color)
}

pub fn pintar_icono_solar(ui: &mut Ui, nombre: &str, tam: f32, color: Color32) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(tam), Sense::hover());
    icono_solar(nombre, tam, color).paint_at(ui, rect);
    respuesta
}

pub fn pintar_icono_solar_en(ui: &Ui, rect: egui::Rect, nombre: &str, color: Color32) {
    icono_solar(nombre, rect.width(), color).paint_at(ui, rect);
}

pub fn nombres_iconos_solar() -> impl Iterator<Item = &'static str> {
    ICONOS_SOLAR.iter().map(|(nombre, _)| *nombre)
}
