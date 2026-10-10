const STX: u8 = 0x02;
const ETX: u8 = 0x03;
const CR: u8 = 0x0d;
const LF: u8 = 0x0a;
const CRC_PARA_CONFIRMAR: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Crc {
    Ok,
    Mal,
    Sin,
}

#[derive(Debug, PartialEq)]
pub struct Trama {
    pub texto: String,
    pub crc: Crc,
}

pub fn peso_de_trama(trama: &str, unidad_entera_kilos: bool) -> Option<i64> {
    let bytes = trama.as_bytes();
    let digitos = |desde: usize| bytes[desde..].iter().take_while(|b| b.is_ascii_digit()).count();
    let inicio = (0..bytes.len()).find(|&i| {
        bytes[i].is_ascii_digit() || (matches!(bytes[i], b'+' | b'-') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
    })?;
    let mut fin = inicio + usize::from(!bytes[inicio].is_ascii_digit());
    fin += digitos(fin);
    if matches!(bytes.get(fin), Some(b'.' | b',')) && bytes.get(fin + 1).is_some_and(u8::is_ascii_digit) {
        fin += 1 + digitos(fin + 1);
    }
    let texto = trama[inicio..fin].replacen(',', ".", 1);
    let valor: f64 = texto.parse().ok()?;
    let en_kilos = texto.contains('.') || unidad_entera_kilos;
    Some((if en_kilos { valor * 1000.0 } else { valor } + 0.5).floor() as i64)
}

pub struct FiltroEstable {
    ultimas: Vec<i64>,
}

impl FiltroEstable {
    pub fn new() -> Self {
        FiltroEstable { ultimas: Vec::new() }
    }

    pub fn leer(&mut self, gramos: i64) -> Option<i64> {
        self.ultimas.push(gramos);
        if self.ultimas.len() > 3 {
            self.ultimas.remove(0);
        }
        if self.ultimas.len() < 3 {
            return None;
        }
        let maximo = self.ultimas.iter().max()?;
        let minimo = self.ultimas.iter().min()?;
        (maximo - minimo <= 2).then_some(gramos)
    }
}

fn sin_control(bytes: &[u8]) -> String {
    bytes.iter().filter(|&&b| b >= 0x20).map(|&b| b as char).collect()
}

#[derive(Default)]
pub struct LectorTramas {
    buffer: Vec<u8>,
    crc_seguidos: u32,
}

impl LectorTramas {
    pub fn leer(&mut self, bloque: &[u8], cortar: bool) -> Vec<Trama> {
        self.buffer.extend_from_slice(bloque);
        let mut tramas = Vec::new();
        while let Some(fin) = self.buffer.iter().position(|&b| b == ETX || b == CR || b == LF) {
            if self.buffer[fin] != ETX {
                tramas.push(Trama { texto: sin_control(&self.buffer[..fin]), crc: Crc::Sin });
                self.buffer.drain(..=fin);
                continue;
            }
            if fin + 1 >= self.buffer.len() && !cortar {
                break;
            }
            let inicio = self.buffer[..=fin].iter().rposition(|&b| b == STX);
            let texto = sin_control(&self.buffer[inicio.map_or(0, |i| i + 1)..fin]);
            let crc = self.buffer[inicio.unwrap_or(0)..=fin].iter().fold(0, |a, b| a ^ b);
            if self.buffer.get(fin + 1) == Some(&crc) {
                self.crc_seguidos += 1;
                tramas.push(Trama { texto, crc: Crc::Ok });
                self.buffer.drain(..fin + 2);
            } else if self.crc_seguidos >= CRC_PARA_CONFIRMAR {
                tramas.push(Trama { texto, crc: Crc::Mal });
                self.buffer.drain(..(fin + 2).min(self.buffer.len()));
            } else {
                self.crc_seguidos = 0;
                tramas.push(Trama { texto, crc: Crc::Sin });
                self.buffer.drain(..=fin);
            }
        }
        if cortar && !self.buffer.is_empty() {
            tramas.push(Trama { texto: sin_control(&self.buffer), crc: Crc::Sin });
            self.buffer.clear();
        }
        tramas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trama_systel(peso: &str) -> Vec<u8> {
        let mut cuerpo = vec![STX];
        cuerpo.extend(peso.bytes());
        cuerpo.push(ETX);
        let crc = cuerpo.iter().fold(0, |a, b| a ^ b);
        cuerpo.push(crc);
        cuerpo
    }

    fn trama(texto: &str, crc: Crc) -> Trama {
        Trama { texto: texto.into(), crc }
    }

    #[test]
    fn lee_gramos_enteros_y_kilos_con_decimales() {
        assert_eq!(peso_de_trama("001250\u{13}", false), Some(1250));
        assert_eq!(peso_de_trama("ST,GS,+00012.50kg", false), Some(12500));
        assert_eq!(peso_de_trama(" 1,235 kg", false), Some(1235));
        assert_eq!(peso_de_trama("000125", true), Some(125000));
        assert_eq!(peso_de_trama("\u{2}\u{3}", false), None);
    }

    #[test]
    fn solo_informa_el_peso_cuando_3_lecturas_seguidas_difieren_2_g_o_menos() {
        let mut estable = FiltroEstable::new();
        assert_eq!(estable.leer(1000), None);
        assert_eq!(estable.leer(1010), None);
        assert_eq!(estable.leer(1012), None);
        assert_eq!(estable.leer(1011), Some(1011));
        assert_eq!(estable.leer(1500), None);
    }

    #[test]
    fn lee_tramas_systel_y_detecta_las_corruptas_una_vez_aprendido_el_crc() {
        let mut lector = LectorTramas::default();
        for _ in 0..3 {
            assert_eq!(lector.leer(&trama_systel("01.250"), false), [trama("01.250", Crc::Ok)]);
        }
        let mut corrupta = trama_systel("01.250");
        *corrupta.last_mut().unwrap() ^= 0xff;
        assert_eq!(lector.leer(&corrupta, false), [trama("01.250", Crc::Mal)]);
        let partida = trama_systel("00.800");
        assert_eq!(lector.leer(&partida[..3], false), []);
        assert_eq!(lector.leer(&partida[3..], false), [trama("00.800", Crc::Ok)]);
    }

    #[test]
    fn lee_balanzas_sin_crc_stx_etx_sueltos_lineas_y_tramas_cortadas_por_silencio() {
        let mut lector = LectorTramas::default();
        assert_eq!(lector.leer(b"\x0200.500\x03\x0200.600\x03", false), [trama("00.500", Crc::Sin)]);
        assert_eq!(lector.leer(&[], true), [trama("00.600", Crc::Sin)]);
        let textos: Vec<String> = lector.leer(b"ST,GS,+00012.50kg\r\n", false).into_iter().map(|t| t.texto).collect();
        assert_eq!(textos, ["ST,GS,+00012.50kg", ""]);
        assert_eq!(lector.leer(b"001250", false), []);
        assert_eq!(lector.leer(&[], true), [trama("001250", Crc::Sin)]);
    }
}
