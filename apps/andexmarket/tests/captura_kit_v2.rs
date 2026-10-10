mod comun;

use andexmarket::ui::marca::{color, tam};
use andexmarket::ui::v2::{
    self, chip, chip_estado, chip_metodo, contador, desplegable, menu_usuario, opcion_eleccion, pintar_tecla, punto_estado, segmentado, tarjeta_kpi,
    tecla, Avatar, Boton, BotonIcono, CampoTexto, Columna, Estilo, EstiloSegmentado, EstiloTecla, ItemMenu, Kpi, Modal, Segmento, Tabla, Tono,
    Usuario,
};
use andexmarket::ui::{self, Peso};
use eframe::egui::{self, Align, Id, Layout, Pos2, Rect, Ui, Vec2};
use egui_kittest::Harness;

struct Vitrina {
    cuadro: u32,
    busqueda: String,
    email: String,
    clave: String,
    vacio: String,
    monto: String,
    motivo: String,
    error: String,
    ancho_papel: usize,
    filtro: usize,
    descuento: usize,
    impresora: usize,
    metodo: usize,
    fila_hover: Option<Pos2>,
}

impl Default for Vitrina {
    fn default() -> Self {
        Vitrina {
            cuadro: 0,
            busqueda: "toma".into(),
            email: "admin@donjose.com.ar".into(),
            clave: "secreta".into(),
            vacio: String::new(),
            monto: "12.500".into(),
            motivo: "El cliente cambió de opinión".into(),
            error: "abc".into(),
            ancho_papel: 0,
            filtro: 1,
            descuento: 0,
            impresora: 0,
            metodo: 2,
            fila_hover: None,
        }
    }
}

fn arrancar(ui: &mut Ui, v: &mut Vitrina) -> bool {
    v.cuadro += 1;
    if v.cuadro == 1 {
        ui::instalar(ui.ctx());
        ui.ctx().request_repaint();
        return false;
    }
    let pantalla = ui.ctx().content_rect();
    ui.painter().with_clip_rect(pantalla).rect_filled(pantalla, 0, color::FONDO_APP);
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    true
}

fn fila(ui: &mut Ui, separacion: f32, contenido: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = separacion;
        contenido(ui);
    });
}

fn titulo_seccion(ui: &mut Ui, texto: &str) {
    v2::texto(ui, texto.to_uppercase(), tam::XS, Peso::Semi, color::TEXTO_TENUE);
    ui.add_space(8.0);
}

fn encabezado(ui: &mut Ui, v: &mut Vitrina) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 68.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0, color::VERDE_PROFUNDO);
    let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(20.0, 0.0))).layout(Layout::left_to_right(Align::Center)));
    let ui = &mut hijo;
    ui.spacing_mut().item_spacing.x = 10.0;
    v2::logo(ui, 26.0);
    v2::logo_texto(ui, 112.0);
    ui.add_space(10.0);
    let anillo = (!v.busqueda.is_empty()).then_some(color::ANILLO_BUSQUEDA);
    CampoTexto::new(&mut v.busqueda)
        .ancho(520.0)
        .icono("barcode")
        .sugerencia("Buscar producto, PLU o escanear código")
        .tam_texto(tam::BUSQUEDA)
        .sin_borde()
        .anillo(anillo)
        .accion("x")
        .ui(ui);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let items = [
            ItemMenu { icono: "printer", etiqueta: "Impresión", tecla: None, peligro: false },
            ItemMenu { icono: "scale", etiqueta: "Balanza", tecla: None, peligro: false },
            ItemMenu { icono: "log-out", etiqueta: "Cerrar sesión", tecla: Some("F4"), peligro: true },
        ];
        if v.cuadro == 3 {
            egui::Popup::open_id(ui.ctx(), ui.make_persistent_id("usuario"));
        }
        menu_usuario(ui, "usuario", &Usuario { nombre: "Lucía Fernández", email: "lucia@donjose.com.ar", detalle: "vie 10 oct · 17:40" }, &items, false);
        v2::separador_vertical(ui, 28.0, color::VERDE);
        for (icono, etiqueta) in [("store", "Cierre de caja"), ("file-text", "Cuenta corriente"), ("history", "Órdenes")] {
            Boton::new(etiqueta, Estilo::FantasmaOscuro).icono(icono).tam_icono(20.0).alto(40.0).relleno(10.0).radio(10).tam_texto(tam::CONTROL).ui(ui);
        }
    });
}

fn campos(ui: &mut Ui, v: &mut Vitrina) {
    titulo_seccion(ui, "Campos");
    v2::etiqueta(ui, "Email del administrador");
    ui.add_space(6.0);
    CampoTexto::new(&mut v.email).icono("mail").sugerencia("admin@negocio.com").id(Id::new("campo-foco")).ui(ui);
    if v.cuadro == 2 {
        ui.memory_mut(|m| m.request_focus(Id::new("campo-foco")));
    }
    ui.add_space(12.0);
    v2::etiqueta(ui, "Contraseña");
    ui.add_space(6.0);
    CampoTexto::new(&mut v.clave).icono("lock").contrasena(true).accion("eye").ui(ui);
    ui.add_space(12.0);
    CampoTexto::new(&mut v.vacio).icono("search").sugerencia("Buscar por cliente o número").alto(40.0).radio(10).tam_texto(tam::CUERPO).ui(ui);
    ui.add_space(12.0);
    fila(ui, 10.0, |ui| {
        CampoTexto::new(&mut v.monto).prefijo("$").ancho(170.0).alto(48.0).tam_texto(18.0).ui(ui);
        CampoTexto::new(&mut v.error).ancho(170.0).alto(48.0).error(true).sufijo("g").ui(ui);
    });
    ui.add_space(4.0);
    v2::texto(ui, "Ingresá el peso en gramos.", tam::ETIQUETA, Peso::Normal, color::ROJO);
    ui.add_space(12.0);
    CampoTexto::new(&mut v.motivo).multilinea(96.0).tam_texto(tam::CUERPO).caracteres_maximos(500).ui(ui);
}

fn botones(ui: &mut Ui) {
    titulo_seccion(ui, "Botones");
    fila(ui, 10.0, |ui| {
        Boton::primario("Configurar negocio").icono("store").alto(48.0).tam_texto(tam::ENTRADA).tam_icono(19.0).ui(ui);
        Boton::primario("Configurando…").cargando(true).alto(48.0).tam_texto(tam::ENTRADA).tam_icono(19.0).ui(ui);
        Boton::primario("Agregar a la orden").icono("cart-plus").color_icono(color::DORADO).tecla("Enter").sin_sombra().radio(11).ui(ui);
    });
    ui.add_space(10.0);
    fila(ui, 10.0, |ui| {
        Boton::secundario("Cancelar").modal().ui(ui);
        Boton::secundario("Reimprimir").icono("printer").color_icono(color::VERDE).ui(ui);
        Boton::peligro("Sí, cancelar").tecla("Enter").modal().ui(ui);
        Boton::new("Descuento efectivo 10%", Estilo::Dorado).tecla("F5").radio(11).ui(ui);
        Boton::new("Agregar", Estilo::ContornoMarca).icono("plus").alto(38.0).tam_icono(14.0).relleno(12.0).radio(9).tam_texto(tam::CONTROL).ui(ui);
        Boton::primario("Deshabilitado").habilitado(false).sin_sombra().modal().ui(ui);
    });
    ui.add_space(10.0);
    fila(ui, 8.0, |ui| {
        Boton::new("Cobrar orden sin factura", Estilo::Secundario).icono("receipt").color_icono(color::VERDE).tecla("F3").alto(52.0).ancho(330.0).relleno(14.0).separacion(12.0).tam_icono(21.0).tam_texto(tam::CUERPO_MAS).repartido().ui(ui);
        Boton::new("Cancelar orden", Estilo::PeligroContorno).icono("trash-2").tecla("F1").alto(52.0).ancho(250.0).relleno(14.0).separacion(12.0).tam_icono(21.0).tam_texto(tam::CUERPO_MAS).repartido().ui(ui);
        BotonIcono::quitar().tam(36.0).tam_icono(19.0).radio(10).ui(ui);
        BotonIcono::cerrar().ui(ui);
        BotonIcono::new("history").colores(color::VERDE, color::VERDE_CHIP, color::VERDE).ui(ui);
        BotonIcono::new("square-pen").ui(ui);
    });
}

fn insignias(ui: &mut Ui) {
    titulo_seccion(ui, "Teclas, chips y estados");
    fila(ui, 8.0, |ui| {
        tecla(ui, "F7", &EstiloTecla::CLARA);
        tecla(ui, "Enter", &EstiloTecla::EN_LINEA);
        tecla(ui, "F4", &EstiloTecla::CONTORNO);
        tecla(ui, "1", &EstiloTecla::CUADRADA);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(96.0, 34.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 10, color::VERDE);
        let mut hijo = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(5.0)).layout(Layout::left_to_right(Align::Center)));
        hijo.spacing_mut().item_spacing.x = 6.0;
        tecla(&mut hijo, "F2", &EstiloTecla::SOBRE_OSCURO);
        tecla(&mut hijo, "4", &EstiloTecla::CUADRADA_OSCURA);
        ui.add_space(8.0);
        chip(ui, "Remito", Tono::Aviso);
        chip(ui, "Factura A", Tono::Azul);
        chip(ui, "Factura B", Tono::Exito);
        chip(ui, "Mayorista", Tono::Aviso);
        chip(ui, "ADMIN", Tono::Admin);
    });
    ui.add_space(10.0);
    fila(ui, 8.0, |ui| {
        chip_estado(ui, "Pagada", Tono::Exito);
        chip_estado(ui, "Parcial", Tono::Aviso);
        chip_estado(ui, "Pendiente", Tono::Peligro);
        chip_metodo(ui, "cash", "Efectivo", Tono::Exito);
        chip_metodo(ui, "card", "Tarjeta", Tono::Azul);
        chip_metodo(ui, "qr", "Transferencia", Tono::Violeta);
        chip_metodo(ui, "split", "Mixto", Tono::Aviso);
        contador(ui, "3", color::VERDE_CHIP, color::VERDE);
        contador(ui, "12", color::DORADO, color::VERDE_PROFUNDO);
        ui.add_space(6.0);
        punto_estado(ui, "En línea", color::VERDE_PUNTO, color::VERDE_EXITO, tam::ETIQUETA, Peso::Medio, false);
        punto_estado(ui, "Balanza · 1,245 kg", color::VERDE_PUNTO, color::TEXTO_APAGADO, tam::CHICO, Peso::Normal, false);
        punto_estado(ui, "Leyendo balanza…", color::DORADO, color::TEXTO_APAGADO, tam::CHICO, Peso::Normal, true);
    });
}

fn selectores(ui: &mut Ui, v: &mut Vitrina) {
    titulo_seccion(ui, "Segmentados, desplegable y opciones");
    fila(ui, 12.0, |ui| {
        segmentado(ui, &[Segmento::texto("80 mm"), Segmento::texto("58 mm")], &mut v.ancho_papel, Some(220.0), EstiloSegmentado::Claro);
        segmentado(
            ui,
            &[Segmento::con_icono("Porcentaje", "percent"), Segmento::con_icono("Monto fijo", "dollar-sign")],
            &mut v.descuento,
            Some(300.0),
            EstiloSegmentado::Claro,
        );
        segmentado(ui, &[Segmento::texto("Todas"), Segmento::texto("Con saldo"), Segmento::texto("Pagadas")], &mut v.filtro, None, EstiloSegmentado::Filtro);
        let opciones = ["Todos los tipos", "Remito", "Factura A", "Factura B"].map(String::from);
        desplegable(ui, "tipo", &mut v.impresora, &opciones, "Elegí", 170.0, 40.0);
    });
    ui.add_space(12.0);
    fila(ui, 10.0, |ui| {
        let metodos = [("qr", "Transferencia / QR"), ("card", "Tarjeta"), ("cash", "Efectivo"), ("split", "Pago mixto")];
        for (indice, (icono, etiqueta)) in metodos.iter().enumerate() {
            let r = opcion_eleccion(ui, Vec2::new(150.0, 124.0), 16, v.metodo == indice, 1.5, |ui, rect, _| {
                let caja = Rect::from_min_size(rect.min + Vec2::splat(10.0), Vec2::splat(24.0));
                pintar_tecla(ui, caja, &(indice + 1).to_string(), &EstiloTecla::CUADRADA, color::TEXTO_SECUNDARIO);
                v2::pintar_icono(ui, rect.center() - Vec2::new(0.0, 14.0), icono, 36.0, color::VERDE);
                v2::pintar_texto(ui, rect.center() + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, *etiqueta, tam::ENTRADA, Peso::Semi, color::TEXTO);
            });
            if r.clicked() {
                v.metodo = indice;
            }
        }
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            fila(ui, 8.0, |ui| {
                Avatar::new("LF").ui(ui);
                Avatar::producto("T").ui(ui);
                Avatar::new("DR").colores(color::VERDE_PROFUNDO, color::BLANCO).ui(ui);
            });
            v2::shimmer(ui, Vec2::new(148.0, 72.0), 24, true);
            v2::shimmer(ui, Vec2::new(96.0, 14.0), 6, false);
        });
        Avatar::vendedor("LF", color::AVATARES[0]).ui(ui);
    });
}

fn tarjetas(ui: &mut Ui, v: &mut Vitrina) {
    titulo_seccion(ui, "Tarjetas KPI y tabla");
    fila(ui, 14.0, |ui| {
        tarjeta_kpi(
            ui,
            &Kpi {
                etiqueta: "Saldo pendiente",
                valor: "$412.380",
                detalle: "Total a cobrar entre todos los clientes",
                icono: "wallet",
                fondo_icono: color::ROJO_FONDO,
                color_icono: color::ROJO,
                color_valor: color::ROJO,
            },
            300.0,
        );
        tarjeta_kpi(
            ui,
            &Kpi {
                etiqueta: "Cobrado este mes",
                valor: "$212.000",
                detalle: "Pagos registrados",
                icono: "hand-coins",
                fondo_icono: color::VERDE_CHIP,
                color_icono: color::VERDE,
                color_valor: color::VERDE_EXITO,
            },
            300.0,
        );
        v2::dato_compacto(ui, "Ticket promedio", "$6.420", 200.0);
    });
    ui.add_space(14.0);
    let columnas = [
        Columna::fraccion(1.2, 170.0),
        Columna::fraccion(1.4, 170.0),
        Columna::fija(100.0),
        Columna::fija(120.0).derecha(),
        Columna::fija(110.0),
        Columna::fija(120.0),
    ];
    let tabla = Tabla::new(&columnas);
    v2::marco_tabla().show(ui, |ui| {
        ui.set_width(ui.available_width());
        tabla.encabezado(ui, &["Comprobante", "Cliente", "Fecha", "Saldo", "Estado", ""], 42.0);
        let filas = [
            ("R 0003-00000114", "Remito", Tono::Aviso, "Restaurante La Huerta", "12/09/2026", "$18.250", "Parcial", Tono::Aviso),
            ("A 0003-00000213", "Factura A", Tono::Azul, "Hotel Aconcagua", "18/09/2026", "$0", "Pagada", Tono::Exito),
            ("B 0003-00001846", "Factura B", Tono::Exito, "Comedor San José", "09/10/2026", "$41.640", "Pendiente", Tono::Peligro),
        ];
        for (indice, (numero, tipo, tono_tipo, cliente, fecha, saldo, estado, tono_estado)) in filas.into_iter().enumerate() {
            let r = tabla.fila(ui, 62.0, None, |ui, celdas| {
                let mut c0 = ui.new_child(egui::UiBuilder::new().max_rect(celdas[0]).layout(Layout::top_down(Align::Min)));
                c0.add_space(12.0);
                c0.label(andexmarket::ui::marca::texto_mono(numero, tam::ETIQUETA, Peso::Medio, color::TEXTO));
                c0.add_space(3.0);
                chip(&mut c0, tipo, tono_tipo);
                v2::tarjetas::celda_texto(ui.painter(), celdas[1], cliente, tam::CUERPO, Peso::Medio, color::TEXTO, false);
                v2::tarjetas::celda_texto(ui.painter(), celdas[2], fecha, tam::CUERPO, Peso::Normal, color::TEXTO_SECUNDARIO, false);
                let color_saldo = if saldo == "$0" { color::VERDE_EXITO } else { color::ROJO };
                v2::tarjetas::celda_texto(ui.painter(), celdas[3], saldo, tam::CUERPO, Peso::Semi, color_saldo, true);
                let mut c4 = ui.new_child(egui::UiBuilder::new().max_rect(celdas[4]).layout(Layout::left_to_right(Align::Center)));
                chip_estado(&mut c4, estado, tono_estado);
                let mut c5 = ui.new_child(egui::UiBuilder::new().max_rect(celdas[5]).layout(Layout::right_to_left(Align::Center)));
                c5.spacing_mut().item_spacing.x = 4.0;
                BotonIcono::quitar().ui(&mut c5);
                BotonIcono::new("download").ui(&mut c5);
                BotonIcono::new("history").colores(color::VERDE, color::VERDE_CHIP, color::VERDE).ui(&mut c5);
            });
            if indice == 1 {
                v.fila_hover = Some(r.rect.center());
            }
        }
    });
}

#[test]
fn vitrina_kit_v2() {
    let mut harness = Harness::builder().with_size([1440.0, 1200.0]).wgpu().build_ui_state(
        |ui, v: &mut Vitrina| {
            if !arrancar(ui, v) {
                return;
            }
            encabezado(ui, v);
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(24, 20)).show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 32.0;
                    ui.vertical(|ui| {
                        ui.set_width(380.0);
                        campos(ui, v);
                    });
                    ui.vertical(|ui| {
                        botones(ui);
                        ui.add_space(22.0);
                        insignias(ui);
                    });
                });
                ui.add_space(22.0);
                selectores(ui, v);
                ui.add_space(22.0);
                tarjetas(ui, v);
            });
        },
        Vitrina::default(),
    );
    harness.run_steps(4);
    if let Some(pos) = harness.state().fila_hover {
        harness.hover_at(pos);
    }
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join("00-kit-v2.png")).expect("guardar");
}

#[test]
fn vitrina_modal_v2() {
    let mut harness = Harness::builder().with_size([1200.0, 800.0]).wgpu().build_ui_state(
        |ui, v: &mut Vitrina| {
            if !arrancar(ui, v) {
                return;
            }
            encabezado(ui, v);
            let r = Modal::new("impresion", "Impresión")
                .subtitulo("Impresora térmica y ancho del papel")
                .icono("printer")
                .ancho(480.0)
                .mostrar(ui.ctx(), |ui| {
                    v2::cuerpo_modal(ui, 20, |ui| {
                        v2::texto(ui, "Impresora", tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO);
                        ui.add_space(6.0);
                        let opciones = ["Compartida como TP806L (por defecto)", "EPSON TM-T20III", "XPrinter XP-80C"].map(String::from);
                        if v.cuadro == 3 {
                            egui::Popup::open_id(ui.ctx(), ui.make_persistent_id("impresora"));
                        }
                        let ancho = ui.available_width();
                        desplegable(ui, "impresora", &mut v.impresora, &opciones, "Elegí", ancho, 44.0);
                        ui.add_space(6.0);
                        v2::texto(ui, "Se imprime en el recurso compartido «TP806L».", tam::CHICO, Peso::Normal, color::TEXTO_APAGADO);
                        ui.add_space(16.0);
                        v2::texto(ui, "Ancho del papel", tam::ETIQUETA, Peso::Semi, color::TEXTO_SECUNDARIO);
                        ui.add_space(6.0);
                        segmentado(ui, &[Segmento::texto("80 mm"), Segmento::texto("58 mm")], &mut v.ancho_papel, Some(ancho), EstiloSegmentado::Claro);
                    });
                    v2::pie_modal(ui, |ui| {
                        Boton::primario("Impresión de prueba").icono("printer").modal().ui(ui);
                        Boton::secundario("Cerrar").modal().ui(ui);
                    });
                });
            let _ = r.cerrar;
            v2::toast::pintar(ui.ctx());
        },
        Vitrina::default(),
    );
    harness.run_steps(2);
    v2::toast::con_detalle(&harness.ctx, v2::toast::Tipo::Exito, "Tomate redondo agregado", "Código 7790001000213");
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join("00-kit-v2-modal.png")).expect("guardar");
}

#[test]
fn vitrina_iconos_solar() {
    let mut harness = Harness::builder().with_size([1200.0, 720.0]).wgpu().build_ui_state(
        |ui, v: &mut Vitrina| {
            if !arrancar(ui, v) {
                return;
            }
            egui::Frame::NONE.inner_margin(egui::Margin::same(24)).show(ui, |ui| {
                v2::titulo(ui, "Íconos Solar Bold Duotone", tam::TITULO_SECCION, -0.015, color::TEXTO);
                ui.add_space(16.0);
                let nombres: Vec<&str> = ui::nombres_iconos_solar().collect();
                for grupo in nombres.chunks(10) {
                    fila(ui, 8.0, |ui| {
                        for nombre in grupo {
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(106.0, 84.0), egui::Sense::hover());
                            ui.painter().rect(rect, 12, color::BLANCO, egui::Stroke::new(1.0, color::BORDE), egui::StrokeKind::Inside);
                            v2::pintar_icono(ui, rect.center() - Vec2::new(0.0, 12.0), nombre, 28.0, color::VERDE);
                            v2::pintar_texto(ui, rect.center() + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, *nombre, tam::XS, Peso::Normal, color::TEXTO_APAGADO);
                        }
                    });
                    ui.add_space(8.0);
                }
                ui.add_space(8.0);
                fila(ui, 14.0, |ui| {
                    for (tam_icono, tinte) in [(16.0, color::TEXTO_TENUE), (20.0, color::ROJO), (24.0, color::DORADO), (34.0, color::VERDE_PROFUNDO), (46.0, color::AZUL)] {
                        ui::pintar_icono_solar(ui, "wallet", tam_icono, tinte);
                        ui::pintar_icono_solar(ui, "x", tam_icono, tinte);
                    }
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(160.0, 56.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 12, color::VERDE_PROFUNDO);
                    for (i, nombre) in ["wallet", "store", "scale", "history"].iter().enumerate() {
                        v2::pintar_icono(ui, egui::pos2(rect.left() + 26.0 + i as f32 * 36.0, rect.center().y), nombre, 24.0, if i == 0 { color::DORADO } else { color::SOBRE_OSCURO_ICONO });
                    }
                    v2::cargando(ui, 24.0, color::VERDE);
                });
            });
        },
        Vitrina::default(),
    );
    harness.run_steps(6);
    let imagen = harness.render().expect("render");
    imagen.save(comun::carpeta_capturas().join("00-iconos-solar.png")).expect("guardar");
}
