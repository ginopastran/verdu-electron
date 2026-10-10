#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result {
    let icono = eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/imagenes/icon.png")).unwrap_or_default();
    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AndexMarket")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(icono),
        ..Default::default()
    };
    eframe::run_native("AndexMarket", opciones, Box::new(|cc| Ok(Box::new(andexmarket::app::App::new(&cc.egui_ctx)))))
}
