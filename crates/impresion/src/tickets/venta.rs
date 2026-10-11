use serde_json::Value;

use super::formato::*;
use super::*;
use crate::escpos::{Ticket, CENTRO, IZQUIERDA};

fn nombre_item(item: &Value) -> String {
    let producto = &item["producto"];
    if !item["nombre"].is_null() {
        texto(&item["nombre"])
    } else if es_arreglo(producto) && !producto["nombre"].is_null() {
        texto(&producto["nombre"])
    } else if let Some(nombre) = producto.as_str() {
        nombre.to_string()
    } else {
        "Producto".into()
    }
}

fn vendedor(datos: &Value) -> String {
    match &datos["vendedor"] {
        Value::Null => "N/A".into(),
        valor if es_arreglo(valor) => match primero([&valor["nombre"], &valor["name"]]) {
            Value::Null => "N/A".into(),
            nombre => texto(nombre),
        },
        valor => texto(valor),
    }
}

fn metodo_pago(metodo: &str) -> String {
    match metodo.to_ascii_lowercase().as_str() {
        "tarjeta" => "TARJETA".into(),
        "transferencia" => "TRANSFERENCIA".into(),
        "efectivo" => "EFECTIVO".into(),
        "qr" => "QR / MERCADOPAGO".into(),
        "split" => "PAGO MIXTO".into(),
        _ => metodo.to_ascii_uppercase(),
    }
}

pub fn venta(datos: &Value, ahora: Ahora) -> Vec<u8> {
    let mut ticket = Ticket::new();
    ticket.alinear(CENTRO);
    let ancho_logo = if flotante(&datos["anchoPapel"]) == 58.0 { 370 } else { 556 };
    if let Some(imagen) = logo(datos, ancho_logo) {
        ticket.imagen(&imagen);
    }

    ticket.negrita(true);
    ticket.tamano(2, 2);
    let negocio = if !vacio(&datos["businessName"]) {
        texto(&datos["businessName"])
    } else if !vacio(&datos["sucursal"]) {
        texto(&datos["sucursal"])
    } else {
        "Verdulería".into()
    };
    ticket.texto(&format!("{}\n", negocio.to_ascii_uppercase()));
    ticket.negrita(false);
    ticket.tamano(1, 1);
    ticket.texto(&format!("Vendedor: {}\n", vendedor(datos)));
    ticket.texto(&format!("{}\n", ahora.format("%Y-%m-%d %H:%M:%S")));
    if !vacio(&datos["idReal"]) {
        ticket.texto(&format!("Orden #{}\n", texto(&datos["idReal"])));
    } else if !vacio(&datos["id"]) {
        ticket.texto(&format!("Orden #{}\n", texto(&datos["id"])));
    }
    ticket.texto(&format!("{}\n", separador()));

    ticket.alinear(IZQUIERDA);
    ticket.texto(&format!("{}\n", encabezado_columnas()));
    ticket.texto(&format!("{}\n", separador()));

    let items = elementos(&datos["items"]);
    if items.is_empty() {
        ticket.texto("No hay productos para mostrar\n");
    }
    for item in items {
        let cantidad = flotante(primero([&item["cantidad"], &item["qty"]]));
        let precio = flotante(primero([&item["precioHistorico"], &item["precio"], &item["price"]]));
        let subtotal = match &item["subtotal"] {
            Value::Null => cantidad * precio,
            valor => flotante(valor),
        };
        for linea in lineas_item(&nombre_item(item), cantidad, precio, subtotal) {
            ticket.texto(&format!("{linea}\n"));
        }
    }

    ticket.texto(&format!("{}\n", separador()));
    let total = flotante(&datos["total"]);
    let subtotal = flotante(&datos["subtotal"]);
    let mut descuento = None;
    let datos_descuento = &datos["discountData"];
    if es_arreglo(datos_descuento) {
        let monto = flotante(&datos_descuento["amount"]);
        if !datos_descuento["amount"].is_null() && monto > 0.0 {
            let sin_descuento = flotante(&datos["subtotalSinDescuento"]);
            let original = if !datos["subtotalSinDescuento"].is_null() && sin_descuento > 0.0 {
                sin_descuento
            } else if !datos["subtotal"].is_null() && subtotal > 0.0 {
                subtotal
            } else {
                total + monto
            };
            let tipo = match &datos_descuento["type"] {
                Value::Null => Value::String("fixed".into()),
                tipo => tipo.clone(),
            };
            descuento = Some((original, monto, tipo, flotante(&datos_descuento["value"])));
        }
    } else if !datos["subtotal"].is_null() && !datos["total"].is_null() && subtotal > total {
        descuento = Some((subtotal, subtotal - total, Value::String(String::new()), 0.0));
    }

    if let Some((original, monto, tipo, valor)) = descuento {
        ticket.texto(&format!("Subtotal sin descuento: ${}\n", numero(original, 2)));
        let linea = match tipo.as_str() {
            _ if vacio(&tipo) || valor <= 0.0 => format!("Descuento aplicado: -${}\n", numero(monto, 2)),
            Some("percentage") => format!("Descuento aplicado ({}%): -${}\n", numero(valor, 1), numero(monto, 2)),
            Some("fixed") => format!("Descuento fijo aplicado: -${}\n", numero(monto, 2)),
            _ => format!("Descuento aplicado: -${}\n", numero(monto, 2)),
        };
        ticket.texto(&linea);
        ticket.texto(&format!("Subtotal con descuento: ${}\n", numero(total, 2)));
        ticket.texto("-----------------------------\n");
    }

    ticket.negrita(true);
    ticket.texto(&format!("{}\n", rellenar_izquierda(&format!("TOTAL: ${}", numero(total, 2)), 32)));
    ticket.negrita(false);

    let pagos = elementos(&datos["pagos"]);
    if pagos.len() > 1 {
        ticket.texto("MÉTODOS DE PAGO:\n");
        for pago in pagos {
            ticket.texto(&format!("{}: ${}\n", mayusculas(&pago["metodoPago"]), numero(flotante(&pago["monto"]), 2)));
        }
    } else {
        let metodo = match &datos["metodoPago"] {
            Value::Null => "N/A".to_string(),
            metodo => texto(metodo),
        };
        ticket.texto(&format!("Método de pago: {}\n", metodo_pago(&metodo)));
    }

    ticket.alinear(CENTRO);
    ticket.texto("\n¡Gracias por su compra!\n");
    ticket.avanzar(3);
    ticket.cortar();
    ticket.abrir_cajon();
    ticket.bytes()
}

#[cfg(test)]
mod tests {
    use super::super::pruebas::*;
    use super::*;

    #[test]
    fn igual_que_ticket_printer_php() {
        comparar("venta", |datos| venta(datos, ahora_fija()));
    }
}

