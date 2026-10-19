use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cobros::redondear;
use crate::iva::precio_visual_con_iva;

pub fn fecha_utc(valor: &Value) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(valor.as_str()?).ok().map(|fecha| fecha.with_timezone(&Utc))
}

pub fn oferta_vigente(porcentaje: Option<f64>, hasta: Option<DateTime<Utc>>, ahora: DateTime<Utc>) -> Option<f64> {
    let porcentaje = porcentaje.filter(|p| *p > 0.0 && *p <= 100.0)?;
    (hasta? >= ahora).then_some(porcentaje)
}

pub fn oferta_de(producto: &Value, ahora: DateTime<Utc>) -> Option<f64> {
    oferta_vigente(producto["ofertaPorcentaje"].as_f64(), fecha_utc(&producto["ofertaHasta"]), ahora)
}

pub fn precio_con_oferta(precio: f64, porcentaje: f64) -> f64 {
    redondear(precio * (1.0 - porcentaje / 100.0), 2)
}

pub fn precio_de_venta(producto: &Value, ahora: DateTime<Utc>) -> f64 {
    let precio = producto["precio"].as_f64().unwrap_or(0.0);
    oferta_de(producto, ahora).map_or(precio, |p| precio_con_oferta(precio, p))
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Producto {
    pub id: i64,
    pub cart_id: String,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub price_per_unit: f64,
    pub subtotal: f64,
    pub costo: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iva_incluido: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iva_porcentaje: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lista_precio_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lista_precio_nombre: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precio_sin_oferta: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oferta_porcentaje: Option<f64>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub precio_libre: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PantallaCarrito {
    pub id: usize,
    pub items: Vec<Producto>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Carritos {
    pub screens: Vec<PantallaCarrito>,
    pub active_screen: usize,
    #[serde(skip)]
    archivo: Option<PathBuf>,
}

impl Default for Carritos {
    fn default() -> Self {
        Carritos { screens: vec![PantallaCarrito::default()], active_screen: 0, archivo: None }
    }
}

impl Carritos {
    pub fn cargar(archivo: PathBuf) -> Self {
        let mut carritos: Carritos = std::fs::read_to_string(&archivo)
            .ok()
            .and_then(|texto| serde_json::from_str(&texto).ok())
            .filter(|c: &Carritos| !c.screens.is_empty())
            .unwrap_or_default();
        if carritos.active_screen >= carritos.screens.len() {
            carritos.active_screen = 0;
        }
        carritos.numerar();
        carritos.archivo = Some(archivo);
        carritos
    }

    fn numerar(&mut self) {
        for (indice, pantalla) in self.screens.iter_mut().enumerate() {
            pantalla.id = indice;
        }
    }

    fn guardar(&self) {
        if let Some(archivo) = &self.archivo {
            if let Ok(texto) = serde_json::to_string(self) {
                let _ = std::fs::write(archivo, texto);
            }
        }
    }

    pub fn activa(&self) -> &PantallaCarrito {
        &self.screens[self.active_screen]
    }

    pub fn items(&self) -> &[Producto] {
        &self.activa().items
    }

    pub fn total(&self) -> f64 {
        self.items().iter().map(|item| item.price_per_unit * item.quantity).sum()
    }

    pub fn total_sin_iva(&self) -> f64 {
        self.total()
    }

    pub fn total_con_iva(&self, iva_incluido_en_precios: Option<bool>) -> f64 {
        self.items()
            .iter()
            .map(|item| {
                let unitario = match iva_incluido_en_precios {
                    Some(en_precios) => precio_visual_con_iva(
                        item.price_per_unit,
                        item.iva_incluido.unwrap_or(false),
                        item.iva_porcentaje,
                        en_precios,
                    ),
                    None => item.price_per_unit,
                };
                unitario * item.quantity
            })
            .sum()
    }

    pub fn agregar(&mut self, nuevo: Producto) {
        let pantalla = &mut self.screens[self.active_screen];
        match pantalla.items.iter_mut().find(|item| item.cart_id == nuevo.cart_id) {
            Some(existente) => {
                existente.quantity += nuevo.quantity;
                existente.subtotal += nuevo.subtotal;
            }
            None => pantalla.items.push(nuevo),
        }
        self.guardar();
    }

    pub fn fijar_monto(&mut self, cart_id: &str, monto: f64) {
        if let Some(item) = self.screens[self.active_screen].items.iter_mut().find(|item| item.cart_id == cart_id && item.precio_libre) {
            item.quantity = 1.0;
            item.price_per_unit = monto;
            item.subtotal = redondear(monto, 2);
            self.guardar();
        }
    }

    pub fn quitar(&mut self, cart_id: &str) {
        self.screens[self.active_screen].items.retain(|item| item.cart_id != cart_id);
        self.guardar();
    }

    pub fn vaciar(&mut self) {
        self.screens[self.active_screen].items.clear();
        self.guardar();
    }

    pub fn vaciar_todo(&mut self) {
        self.screens = vec![PantallaCarrito::default()];
        self.active_screen = 0;
        if let Some(archivo) = &self.archivo {
            let _ = std::fs::remove_file(archivo);
        }
    }

    pub fn elegir(&mut self, indice: usize) {
        if indice < self.screens.len() {
            self.active_screen = indice;
            self.guardar();
        }
    }

    pub fn agregar_pantalla(&mut self) -> bool {
        if self.screens.len() >= 4 {
            return false;
        }
        let id = self.screens.len();
        self.screens.push(PantallaCarrito { id, items: Vec::new() });
        self.active_screen = id;
        self.guardar();
        true
    }

    pub fn borrar_pantalla(&mut self, indice: usize) -> bool {
        if self.screens.len() <= 1 || indice >= self.screens.len() {
            return false;
        }
        self.screens.remove(indice);
        self.numerar();
        if self.active_screen == indice {
            self.active_screen = 0;
        } else if self.active_screen > indice {
            self.active_screen -= 1;
        }
        self.guardar();
        true
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn con_items(carritos: &mut Carritos, nombre: &str) {
        carritos.agregar(Producto { cart_id: nombre.into(), name: nombre.into(), ..Default::default() });
    }

    #[test]
    fn pestanas_quedan_consistentes_al_borrar_una_del_medio() {
        let mut carritos = Carritos::default();
        con_items(&mut carritos, "a");
        carritos.agregar_pantalla();
        con_items(&mut carritos, "b");
        carritos.agregar_pantalla();
        con_items(&mut carritos, "c");
        assert!(carritos.borrar_pantalla(1));
        assert_eq!(carritos.active_screen, 1);
        assert_eq!(carritos.items()[0].name, "c");
        assert!(carritos.agregar_pantalla());
        let ids: Vec<usize> = carritos.screens.iter().map(|p| p.id).collect();
        assert_eq!(ids, [0, 1, 2]);
        assert_eq!(carritos.active_screen, 2);
        assert!(carritos.items().is_empty());
        carritos.elegir(1);
        assert_eq!(carritos.items()[0].name, "c");
        assert!(carritos.borrar_pantalla(1));
        assert_eq!(carritos.active_screen, 0);
        assert_eq!(carritos.items()[0].name, "a");
        assert!(!carritos.borrar_pantalla(5));
    }

    #[test]
    fn oferta_vigente_solo_con_porcentaje_y_fecha_futura() {
        let ahora = fecha_utc(&serde_json::json!("2026-10-14T15:00:00.000Z")).unwrap();
        let producto = |porcentaje: Value, hasta: Value| serde_json::json!({ "precio": 2890, "ofertaPorcentaje": porcentaje, "ofertaHasta": hasta });
        let vigente = producto(serde_json::json!(20), serde_json::json!("2026-10-15T02:59:59.000Z"));
        assert_eq!(oferta_de(&vigente, ahora), Some(20.0));
        assert_eq!(precio_de_venta(&vigente, ahora), 2312.0);
        assert_eq!(oferta_de(&producto(serde_json::json!(20), serde_json::json!("2026-10-14T15:00:00.000Z")), ahora), Some(20.0));
        let vencida = producto(serde_json::json!(20), serde_json::json!("2026-10-14T14:59:59.000Z"));
        assert_eq!(oferta_de(&vencida, ahora), None);
        assert_eq!(precio_de_venta(&vencida, ahora), 2890.0);
        assert_eq!(oferta_de(&producto(serde_json::json!(0), serde_json::json!("2027-01-01T00:00:00.000Z")), ahora), None);
        assert_eq!(oferta_de(&producto(Value::Null, serde_json::json!("2027-01-01T00:00:00.000Z")), ahora), None);
        assert_eq!(oferta_de(&producto(serde_json::json!(15), Value::Null), ahora), None);
        assert_eq!(precio_de_venta(&serde_json::json!({ "precio": 1150 }), ahora), 1150.0);
        assert_eq!(precio_con_oferta(1250.0, 15.0), 1062.5);
        assert_eq!(precio_con_oferta(999.99, 33.0), 669.99);
    }

    #[test]
    fn carrito_guardado_sin_campos_de_oferta_sigue_cargando() {
        let item: Producto = serde_json::from_str(r#"{"id":1,"cartId":"1-a","name":"Tomate","quantity":1,"unit":"Kg","pricePerUnit":100,"subtotal":100,"costo":50}"#).unwrap();
        assert_eq!(item.precio_sin_oferta, None);
        assert!(!serde_json::to_string(&item).unwrap().contains("oferta"));
    }

    #[test]
    fn linea_de_precio_libre_cambia_solo_el_monto() {
        let mut carritos = Carritos::default();
        carritos.agregar(Producto { cart_id: "libre".into(), quantity: 1.0, price_per_unit: 1500.0, subtotal: 1500.0, precio_libre: true, ..Default::default() });
        carritos.agregar(Producto { cart_id: "comun".into(), quantity: 2.0, price_per_unit: 100.0, subtotal: 200.0, ..Default::default() });
        carritos.fijar_monto("libre", 2750.5);
        carritos.fijar_monto("comun", 9999.0);
        let items = carritos.items();
        assert_eq!((items[0].quantity, items[0].price_per_unit, items[0].subtotal), (1.0, 2750.5, 2750.5));
        assert_eq!((items[1].quantity, items[1].price_per_unit, items[1].subtotal), (2.0, 100.0, 200.0));
        assert_eq!(carritos.total(), 2950.5);
        let guardado = serde_json::to_string(&items[0]).unwrap();
        assert!(guardado.contains("\"precioLibre\":true"), "{guardado}");
        assert!(!serde_json::to_string(&items[1]).unwrap().contains("precioLibre"));
        let leido: Producto = serde_json::from_str(&guardado).unwrap();
        assert!(leido.precio_libre);
    }

    #[test]
    fn cargar_renumera_ids_repetidos() {
        let archivo = std::env::temp_dir().join(format!("carritos-ids-{}.json", std::process::id()));
        std::fs::write(&archivo, r#"{"screens":[{"id":0,"items":[]},{"id":2,"items":[]},{"id":2,"items":[]}],"activeScreen":2}"#).unwrap();
        let carritos = Carritos::cargar(archivo.clone());
        let _ = std::fs::remove_file(archivo);
        assert_eq!(carritos.screens.iter().map(|p| p.id).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(carritos.active_screen, 2);
    }
}
