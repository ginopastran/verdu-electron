mod imagen;
mod tablas;

use std::collections::HashMap;
use std::sync::LazyLock;

use unicode_normalization::UnicodeNormalization;

pub use imagen::{cargar_logo, Imagen};

const ESC: u8 = 0x1b;
const GS: u8 = 0x1d;

pub const IZQUIERDA: u8 = 0;
pub const CENTRO: u8 = 1;
pub const DERECHA: u8 = 2;

struct Codificacion {
    tablas: HashMap<u8, HashMap<char, u8>>,
    disponibles: HashMap<char, u8>,
}

static CODIFICACION: LazyLock<Codificacion> = LazyLock::new(|| {
    let mut tablas = HashMap::new();
    let mut disponibles = HashMap::new();
    for (numero, caracteres) in tablas::TABLAS {
        let mut mapa = HashMap::new();
        for (posicion, caracter) in caracteres.chars().enumerate() {
            if caracter == '\0' {
                continue;
            }
            mapa.insert(caracter, 128 + posicion as u8);
            disponibles.entry(caracter).or_insert(numero);
        }
        tablas.insert(numero, mapa);
    }
    Codificacion { tablas, disponibles }
});

pub struct Ticket {
    bytes: Vec<u8>,
    tabla: u8,
}

impl Ticket {
    pub fn new() -> Self {
        Ticket { bytes: vec![ESC, b'@'], tabla: 0 }
    }

    pub fn texto(&mut self, texto: &str) {
        let codificacion = &*CODIFICACION;
        let mut tabla = self.tabla;
        let mut bloque = Vec::new();
        for caracter in texto.nfc() {
            let coincide = !codificacion.disponibles.contains_key(&caracter)
                || codificacion.tablas.get(&tabla).is_some_and(|mapa| mapa.contains_key(&caracter));
            if coincide {
                bloque.push(caracter);
            } else {
                self.escribir_bloque(&bloque, tabla);
                tabla = codificacion.disponibles[&caracter];
                bloque = vec![caracter];
            }
        }
        if !bloque.is_empty() {
            self.escribir_bloque(&bloque, tabla);
        }
    }

    fn escribir_bloque(&mut self, bloque: &[char], tabla: u8) {
        let mapa = CODIFICACION.tablas.get(&tabla);
        let mut crudo = Vec::with_capacity(bloque.len());
        for &caracter in bloque {
            if let Some(&byte) = mapa.and_then(|mapa| mapa.get(&caracter)) {
                crudo.push(byte);
            } else if (' '..='~').contains(&caracter) || caracter == '\n' {
                crudo.push(caracter as u8);
            } else if caracter != '\r' {
                crudo.push(b'?');
            }
        }
        if self.tabla != tabla {
            self.bytes.extend([ESC, b't', tabla]);
            self.tabla = tabla;
        }
        self.bytes.extend(crudo);
    }

    pub fn negrita(&mut self, activa: bool) {
        self.bytes.extend([ESC, b'E', activa as u8]);
    }

    pub fn alinear(&mut self, alineacion: u8) {
        self.bytes.extend([ESC, b'a', alineacion]);
    }

    pub fn tamano(&mut self, ancho: u8, alto: u8) {
        self.bytes.extend([GS, b'!', 16 * (ancho - 1) + (alto - 1)]);
    }

    pub fn avanzar(&mut self, lineas: u8) {
        if lineas <= 1 {
            self.bytes.push(b'\n');
        } else {
            self.bytes.extend([ESC, b'd', lineas]);
        }
    }

    pub fn cortar(&mut self) {
        self.bytes.extend([GS, b'V', 65, 3]);
    }

    pub fn abrir_cajon(&mut self) {
        self.bytes.extend([ESC, b'p', b'0', 60, 120]);
    }

    pub fn qr(&mut self, contenido: &str, nivel: u8, tamano: u8) {
        if contenido.is_empty() {
            return;
        }
        self.dato_2d(65, &[50, 0], None);
        self.dato_2d(67, &[tamano], None);
        self.dato_2d(69, &[48 + nivel], None);
        self.dato_2d(80, contenido.as_bytes(), Some(b'0'));
        self.dato_2d(81, &[], Some(b'0'));
    }

    fn dato_2d(&mut self, funcion: u8, datos: &[u8], m: Option<u8>) {
        let largo = datos.len() + m.is_some() as usize + 2;
        self.bytes.extend([GS, b'(', b'k', (largo % 256) as u8, (largo / 256) as u8, b'1', funcion]);
        self.bytes.extend(m);
        self.bytes.extend(datos);
    }

    pub fn imagen(&mut self, imagen: &Imagen) {
        let ancho_bytes = imagen.ancho.div_ceil(8);
        self.bytes.extend([GS, b'v', b'0', 0]);
        for valor in [ancho_bytes, imagen.alto] {
            self.bytes.extend([(valor % 256) as u8, (valor / 256) as u8]);
        }
        for fila in imagen.negro.chunks(imagen.ancho) {
            for grupo in fila.chunks(8) {
                let byte = grupo
                    .iter()
                    .enumerate()
                    .filter(|(_, &negro)| negro)
                    .fold(0u8, |byte, (bit, _)| byte | (0x80 >> bit));
                self.bytes.push(byte);
            }
        }
    }

    pub fn bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codificar(texto: &str) -> Vec<u8> {
        let mut ticket = Ticket::new();
        ticket.texto(texto);
        ticket.bytes()[2..].to_vec()
    }

    #[test]
    fn ascii_pasa_igual_y_los_acentos_usan_cp437() {
        assert_eq!(codificar("Hola\n"), b"Hola\n");
        assert_eq!(codificar("¡Gracias!"), [0xad, b'G', b'r', b'a', b'c', b'i', b'a', b's', b'!']);
        assert_eq!(codificar("a\tb\r\n"), b"a?b\n");
    }

    #[test]
    fn cambia_de_tabla_solo_cuando_hace_falta() {
        assert_eq!(codificar("MÉTODO"), [b'M', 0x90, b'T', b'O', b'D', b'O']);
        assert_eq!(codificar("Ó é"), [ESC, b't', 2, 0xe0, b' ', 0x82]);
    }
}
