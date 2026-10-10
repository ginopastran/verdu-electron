mod comun;

use andexmarket::app::Ruta;
use andexmarket::ui::{self, Boton, Entrada, TamBoton, Variante, VarianteInsignia};
use eframe::egui;
use egui_kittest::Harness;

#[test]
fn arranque() {
    comun::capturar("00-arranque", comun::global(Ruta::AdminLogin), |_| {});
}

struct Vitrina {
    instalado: bool,
    texto: String,
    vacio: String,
    marcada: bool,
    activo: bool,
    pestana: usize,
    opcion: usize,
}

#[test]
fn vitrina_kit() {
    let estado = Vitrina {
        instalado: false,
        texto: "Tomate perita".into(),
        vacio: String::new(),
        marcada: true,
        activo: true,
        pestana: 0,
        opcion: 1,
    };
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(
        |ui, v: &mut Vitrina| {
            if !v.instalado {
                ui::instalar(ui.ctx());
                v.instalado = true;
                ui.ctx().request_repaint();
                return;
            }
            egui::Frame::new().fill(ui::paleta::GRAY_50).inner_margin(24).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    Boton::new("Primario").ui(ui);
                    Boton::new("Destructivo").variante(Variante::Destructivo).ui(ui);
                    Boton::new("Contorno").variante(Variante::Contorno).ui(ui);
                    Boton::new("Secundario").variante(Variante::Secundario).ui(ui);
                    Boton::new("Fantasma").variante(Variante::Fantasma).ui(ui);
                    Boton::new("Enlace").variante(Variante::Enlace).ui(ui);
                    Boton::new("Esmeralda").variante(Variante::Esmeralda).icono("shopping-cart").ui(ui);
                    Boton::new("Cancelar").variante(Variante::Cancelar).icono("x").ui(ui);
                    Boton::new("").variante(Variante::Contorno).tam(TamBoton::Icono).icono("user").ui(ui);
                    Boton::new("Guardando").cargando(true).ui(ui);
                    Boton::new("Deshabilitado").habilitado(false).ui(ui);
                });
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.vertical(|ui| {
                        ui::etiqueta(ui, "Producto");
                        ui.add_space(8.0);
                        Entrada::new(&mut v.texto).ancho(300.0).ui(ui);
                        ui.add_space(8.0);
                        Entrada::new(&mut v.vacio).ancho(300.0).sugerencia("Buscar producto...").icono("search").ui(ui);
                    });
                    ui.vertical(|ui| {
                        ui::casilla(ui, &mut v.marcada, "Recordar credenciales");
                        ui.add_space(12.0);
                        ui::interruptor(ui, &mut v.activo);
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui::insignia(ui, "Primaria", VarianteInsignia::Primaria);
                            ui::insignia(ui, "Secundaria", VarianteInsignia::Secundaria);
                            ui::insignia(ui, "Destructiva", VarianteInsignia::Destructiva);
                            ui::insignia(ui, "Contorno", VarianteInsignia::Contorno);
                        });
                    });
                    ui.vertical(|ui| {
                        ui::pestanas(ui, &["Orden 1", "Orden 2", "Orden 3"], &mut v.pestana, None);
                        ui.add_space(12.0);
                        let opciones = vec!["58 mm".to_string(), "80 mm".to_string()];
                        ui::selector(ui, "papel", &mut v.opcion, &opciones, "Elegí", Some(200.0));
                    });
                });
                ui.add_space(16.0);
                ui::tarjeta().inner_margin(24).show(ui, |ui| {
                    ui.set_width(360.0);
                    ui::titulo_tarjeta(ui, "Total");
                    ui.add_space(6.0);
                    ui::descripcion(ui, "Resumen de la orden actual");
                    ui.add_space(16.0);
                    ui.label(ui::texto("$12.345,67", ui::Tam::X4, ui::Peso::Negrita, ui::color::TEXTO));
                });
                ui.add_space(16.0);
                let mut x = 0.0;
                ui.horizontal(|ui| {
                    ui::pintar_degradado(
                        ui.painter(),
                        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(384.0, 120.0)),
                        8,
                        ui::color::ESMERALDA_DESDE,
                        ui::color::ESMERALDA_HASTA,
                        ui::Direccion::AbajoDerecha,
                    );
                    x += 1.0;
                });
            });
            let r = ui::Dialogo::new("demo").mostrar(ui.ctx(), |ui| {
                ui::encabezado_dialogo(ui, "Confirmar pago", Some("Revisá el total antes de cobrar."));
                ui.add_space(16.0);
                ui.label(ui::texto("Total a cobrar: $4.500,00", ui::Tam::Base, ui::Peso::Normal, ui::color::TEXTO));
                ui.add_space(16.0);
                ui::pie_dialogo(ui, |ui| {
                    Boton::new("Cobrar").variante(Variante::Esmeralda).ui(ui);
                    Boton::new("Cancelar").variante(Variante::Contorno).ui(ui);
                });
            });
            let _ = r.cerrar;
            ui::toast::pintar(ui.ctx());
        },
        estado,
    );
    harness.run_steps(2);
    ui::toast::exito(&harness.ctx, "Venta registrada correctamente");
    ui::toast::error(&harness.ctx, "Error al imprimir el ticket");
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join("00-vitrina-kit.png")).expect("guardar");
}
