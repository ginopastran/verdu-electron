use super::{largo, numero, rellenar_derecha_mb, rellenar_izquierda_mb};

const ANCHO_TOTAL: usize = 42;
const ANCHO_CANTIDAD: usize = 8;
const ANCHO_PRECIO: usize = 9;
const ANCHO_COLUMNA_TOTAL: usize = 13;
const ANCHO_PRODUCTO: usize = ANCHO_TOTAL - ANCHO_CANTIDAD - ANCHO_PRECIO - ANCHO_COLUMNA_TOTAL - 3;

pub(super) fn separador() -> String {
    "-".repeat(ANCHO_TOTAL)
}

pub(super) fn partir_texto(valor: &str, ancho: usize) -> Vec<String> {
    let normalizado = valor.split(char::is_whitespace).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ");
    let normalizado = normalizado.trim_matches([' ', '\t', '\n', '\r', '\0', '\x0b']);
    if normalizado.is_empty() {
        return vec![String::new()];
    }
    let mut lineas: Vec<String> = Vec::new();
    let mut actual = String::new();
    for palabra in normalizado.split(' ') {
        if actual.is_empty() {
            actual = palabra.to_string();
        } else if largo(&actual) + 1 + largo(palabra) <= ancho {
            actual = format!("{actual} {palabra}");
        } else {
            lineas.push(std::mem::replace(&mut actual, palabra.to_string()));
        }
    }
    if !actual.is_empty() {
        lineas.push(actual);
    }
    lineas
        .into_iter()
        .flat_map(|linea| {
            let caracteres: Vec<char> = linea.chars().collect();
            if caracteres.len() <= ancho {
                vec![linea]
            } else {
                caracteres.chunks(ancho).map(|trozo| trozo.iter().collect()).collect()
            }
        })
        .collect()
}

pub(super) fn lineas_item(nombre: &str, cantidad: f64, precio: f64, subtotal: f64) -> Vec<String> {
    let lineas = partir_texto(nombre, ANCHO_PRODUCTO);
    let ultima = lineas.len() - 1;
    let cantidad = rellenar_izquierda_mb(&numero(cantidad, 3), ANCHO_CANTIDAD);
    let precio = rellenar_izquierda_mb(&format!("${}", numero(precio, 2)), ANCHO_PRECIO);
    let subtotal = rellenar_izquierda_mb(&format!("${}", numero(subtotal, 2)), ANCHO_COLUMNA_TOTAL);
    lineas
        .into_iter()
        .enumerate()
        .map(|(indice, linea)| {
            if indice == ultima {
                format!("{} {cantidad} {precio} {subtotal}", rellenar_derecha_mb(&linea, ANCHO_PRODUCTO))
            } else {
                linea
            }
        })
        .collect()
}

pub(super) fn encabezado_columnas() -> String {
    format!(
        "{} {} {} {}",
        rellenar_derecha_mb("PRODUCTO", ANCHO_PRODUCTO),
        rellenar_izquierda_mb("CANT", ANCHO_CANTIDAD),
        rellenar_izquierda_mb("PRECIO", ANCHO_PRECIO),
        rellenar_izquierda_mb("TOTAL", ANCHO_COLUMNA_TOTAL)
    )
}
