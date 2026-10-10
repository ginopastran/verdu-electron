use eframe::egui::{self, Color32, Image, Response, Sense, Ui, Vec2};

const ICONOS: &[(&str, &[u8])] = &[
    ("alert-circle", include_bytes!("../../../../assets/iconos/alert-circle.svg")),
    ("alert-triangle", include_bytes!("../../../../assets/iconos/alert-triangle.svg")),
    ("arrow-left", include_bytes!("../../../../assets/iconos/arrow-left.svg")),
    ("arrow-right", include_bytes!("../../../../assets/iconos/arrow-right.svg")),
    ("bar-chart-3", include_bytes!("../../../../assets/iconos/bar-chart-3.svg")),
    ("building-2", include_bytes!("../../../../assets/iconos/building-2.svg")),
    ("building", include_bytes!("../../../../assets/iconos/building.svg")),
    ("calculator", include_bytes!("../../../../assets/iconos/calculator.svg")),
    ("calendar-icon", include_bytes!("../../../../assets/iconos/calendar-icon.svg")),
    ("calendar", include_bytes!("../../../../assets/iconos/calendar.svg")),
    ("check-circle", include_bytes!("../../../../assets/iconos/check-circle.svg")),
    ("check", include_bytes!("../../../../assets/iconos/check.svg")),
    ("chevron-down", include_bytes!("../../../../assets/iconos/chevron-down.svg")),
    ("chevron-left", include_bytes!("../../../../assets/iconos/chevron-left.svg")),
    ("chevron-right", include_bytes!("../../../../assets/iconos/chevron-right.svg")),
    ("chevron-up", include_bytes!("../../../../assets/iconos/chevron-up.svg")),
    ("circle", include_bytes!("../../../../assets/iconos/circle.svg")),
    ("clock", include_bytes!("../../../../assets/iconos/clock.svg")),
    ("credit-card", include_bytes!("../../../../assets/iconos/credit-card.svg")),
    ("dollar-sign", include_bytes!("../../../../assets/iconos/dollar-sign.svg")),
    ("dot", include_bytes!("../../../../assets/iconos/dot.svg")),
    ("download", include_bytes!("../../../../assets/iconos/download.svg")),
    ("edit", include_bytes!("../../../../assets/iconos/edit.svg")),
    ("eye", include_bytes!("../../../../assets/iconos/eye.svg")),
    ("file-text", include_bytes!("../../../../assets/iconos/file-text.svg")),
    ("grip-vertical", include_bytes!("../../../../assets/iconos/grip-vertical.svg")),
    ("hash", include_bytes!("../../../../assets/iconos/hash.svg")),
    ("history", include_bytes!("../../../../assets/iconos/history.svg")),
    ("home", include_bytes!("../../../../assets/iconos/home.svg")),
    ("loader-2", include_bytes!("../../../../assets/iconos/loader-2.svg")),
    ("log-out", include_bytes!("../../../../assets/iconos/log-out.svg")),
    ("minus", include_bytes!("../../../../assets/iconos/minus.svg")),
    ("moon", include_bytes!("../../../../assets/iconos/moon.svg")),
    ("more-horizontal", include_bytes!("../../../../assets/iconos/more-horizontal.svg")),
    ("package", include_bytes!("../../../../assets/iconos/package.svg")),
    ("panel-left", include_bytes!("../../../../assets/iconos/panel-left.svg")),
    ("percent", include_bytes!("../../../../assets/iconos/percent.svg")),
    ("plus-circle", include_bytes!("../../../../assets/iconos/plus-circle.svg")),
    ("plus", include_bytes!("../../../../assets/iconos/plus.svg")),
    ("printer", include_bytes!("../../../../assets/iconos/printer.svg")),
    ("qr-code", include_bytes!("../../../../assets/iconos/qr-code.svg")),
    ("receipt", include_bytes!("../../../../assets/iconos/receipt.svg")),
    ("refresh-cw", include_bytes!("../../../../assets/iconos/refresh-cw.svg")),
    ("save", include_bytes!("../../../../assets/iconos/save.svg")),
    ("scale", include_bytes!("../../../../assets/iconos/scale.svg")),
    ("search", include_bytes!("../../../../assets/iconos/search.svg")),
    ("settings", include_bytes!("../../../../assets/iconos/settings.svg")),
    ("shopping-cart", include_bytes!("../../../../assets/iconos/shopping-cart.svg")),
    ("store", include_bytes!("../../../../assets/iconos/store.svg")),
    ("sun", include_bytes!("../../../../assets/iconos/sun.svg")),
    ("trash-2", include_bytes!("../../../../assets/iconos/trash-2.svg")),
    ("user-cog", include_bytes!("../../../../assets/iconos/user-cog.svg")),
    ("user", include_bytes!("../../../../assets/iconos/user.svg")),
    ("users", include_bytes!("../../../../assets/iconos/users.svg")),
    ("wallet", include_bytes!("../../../../assets/iconos/wallet.svg")),
    ("x", include_bytes!("../../../../assets/iconos/x.svg")),
    ("xcircle", include_bytes!("../../../../assets/iconos/xcircle.svg")),
];

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
    ("andexmarket-logo.png", include_bytes!("../../../../assets/imagenes/andexmarket-logo.png")),
    ("andextech-black.png", include_bytes!("../../../../assets/imagenes/andextech-black.png")),
    ("andextech-logo.png", include_bytes!("../../../../assets/imagenes/andextech-logo.png")),
    ("icon.png", include_bytes!("../../../../assets/imagenes/icon.png")),
    ("iselin-logo.png", include_bytes!("../../../../assets/imagenes/iselin-logo.png")),
    ("mercadopago.webp", include_bytes!("../../../../assets/imagenes/mercadopago.webp")),
    ("teclas/F1.png", include_bytes!("../../../../assets/imagenes/teclas/F1.png")),
    ("teclas/F2.png", include_bytes!("../../../../assets/imagenes/teclas/F2.png")),
    ("teclas/F3.png", include_bytes!("../../../../assets/imagenes/teclas/F3.png")),
    ("teclas/F4.png", include_bytes!("../../../../assets/imagenes/teclas/F4.png")),
    ("teclas/F6.png", include_bytes!("../../../../assets/imagenes/teclas/F6.png")),
    ("teclas/F7.png", include_bytes!("../../../../assets/imagenes/teclas/F7.png")),
    ("teclas/F8.png", include_bytes!("../../../../assets/imagenes/teclas/F8.png")),
];

pub fn registrar(ctx: &egui::Context) {
    for (nombre, bytes) in ICONOS {
        let svg = String::from_utf8_lossy(bytes).replace("currentColor", "#ffffff");
        ctx.include_bytes(format!("bytes://icono/{nombre}.svg"), svg.into_bytes());
    }
    for (nombre, bytes) in ICONOS_SOLAR {
        let svg = String::from_utf8_lossy(bytes).replace("currentColor", "#ffffff");
        ctx.include_bytes(format!("bytes://solar/{nombre}.svg"), svg.into_bytes());
    }
    for (nombre, bytes) in IMAGENES {
        ctx.include_bytes(format!("bytes://imagen/{nombre}"), *bytes);
    }
}

pub fn icono(nombre: &str, tam: f32, color: Color32) -> Image<'static> {
    Image::new(format!("bytes://icono/{nombre}.svg")).fit_to_exact_size(Vec2::splat(tam)).tint(color)
}

pub fn imagen(nombre: &str) -> Image<'static> {
    Image::new(format!("bytes://imagen/{nombre}"))
}

pub fn pintar_icono(ui: &mut Ui, nombre: &str, tam: f32, color: Color32) -> Response {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(tam), Sense::hover());
    icono(nombre, tam, color).paint_at(ui, rect);
    respuesta
}

pub fn pintar_icono_en(ui: &Ui, rect: egui::Rect, nombre: &str, color: Color32) {
    icono(nombre, rect.width(), color).paint_at(ui, rect);
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
