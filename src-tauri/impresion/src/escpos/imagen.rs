use std::io::Cursor;
use std::path::Path;

pub struct Imagen {
    pub ancho: usize,
    pub alto: usize,
    pub negro: Vec<bool>,
}

#[derive(Clone, Copy)]
struct Pixel {
    rojo: u8,
    verde: u8,
    azul: u8,
    alfa: u8,
}

struct Lienzo {
    ancho: usize,
    alto: usize,
    pixeles: Vec<Pixel>,
}

impl Lienzo {
    fn pixel(&self, x: usize, y: usize) -> Pixel {
        if x < self.ancho && y < self.alto {
            self.pixeles[y * self.ancho + x]
        } else {
            Pixel { rojo: 0, verde: 0, azul: 0, alfa: 0 }
        }
    }
}

pub fn cargar_logo(ruta: &Path, ancho_destino: usize) -> Option<Imagen> {
    let bytes = std::fs::read(ruta).ok()?;
    let original = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        decodificar_png(&bytes)?
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        decodificar_jpeg(&bytes)?
    } else {
        return None;
    };
    let alto_destino = (original.alto as f64 * (ancho_destino as f64 / original.ancho as f64)).floor() as usize;
    if alto_destino == 0 {
        return None;
    }
    let escalado = remuestrear(&original, ancho_destino, alto_destino);
    let negro = escalado
        .pixeles
        .iter()
        .map(|p| {
            let grisura = ((p.rojo as u32 + p.verde as u32 + p.azul as u32) / 3) >> 7;
            ((1 - grisura) >> (p.alfa >> 6)) == 1
        })
        .collect();
    Some(Imagen { ancho: ancho_destino, alto: alto_destino, negro })
}

fn alfa_gd(alfa_png: u8) -> u8 {
    127 - (alfa_png >> 1)
}

fn decodificar_png(bytes: &[u8]) -> Option<Lienzo> {
    let mut decodificador = png::Decoder::new(Cursor::new(bytes));
    decodificador.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut lector = decodificador.read_info().ok()?;
    let truecolor_sin_alfa = lector.info().color_type == png::ColorType::Rgb;
    let mut buffer = vec![0; lector.output_buffer_size()];
    let cuadro = lector.next_frame(&mut buffer).ok()?;
    let datos = &buffer[..cuadro.buffer_size()];
    let canales = cuadro.color_type.samples();
    let pixeles = datos
        .chunks_exact(canales)
        .map(|c| {
            let (rojo, verde, azul, alfa) = match c.len() {
                4 => (c[0], c[1], c[2], c[3]),
                3 => (c[0], c[1], c[2], 255),
                2 => (c[0], c[0], c[0], c[1]),
                _ => (c[0], c[0], c[0], 255),
            };
            let alfa = if truecolor_sin_alfa { 0 } else { alfa_gd(alfa) };
            Pixel { rojo, verde, azul, alfa }
        })
        .collect();
    Some(Lienzo { ancho: cuadro.width as usize, alto: cuadro.height as usize, pixeles })
}

fn decodificar_jpeg(bytes: &[u8]) -> Option<Lienzo> {
    let mut decodificador = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    let datos = decodificador.decode().ok()?;
    let info = decodificador.info()?;
    let pixeles = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => datos.iter().map(|&g| Pixel { rojo: g, verde: g, azul: g, alfa: 0 }).collect(),
        jpeg_decoder::PixelFormat::RGB24 => datos
            .chunks_exact(3)
            .map(|c| Pixel { rojo: c[0], verde: c[1], azul: c[2], alfa: 0 })
            .collect(),
        _ => return None,
    };
    Some(Lienzo { ancho: info.width as usize, alto: info.height as usize, pixeles })
}

fn piso_f32(valor: f64) -> f64 {
    (valor as f32).floor() as f64
}

fn truncar(valor: f64) -> f64 {
    valor as i64 as f64
}

fn remuestrear(origen: &Lienzo, ancho: usize, alto: usize) -> Lienzo {
    let (ancho_origen, alto_origen) = (origen.ancho as f64, origen.alto as f64);
    let mut pixeles = Vec::with_capacity(ancho * alto);
    for y in 0..alto {
        let sy1 = y as f64 * alto_origen / alto as f64;
        let sy2 = (y + 1) as f64 * alto_origen / alto as f64;
        for x in 0..ancho {
            let sx1 = x as f64 * ancho_origen / ancho as f64;
            let sx2 = (x + 1) as f64 * ancho_origen / ancho as f64;
            let (mut rojo, mut verde, mut azul, mut alfa) = (0.0, 0.0, 0.0, 0.0);
            let (mut suma_alfa, mut suma_contribucion, mut cantidad) = (0.0, 0.0, 0.0);
            let mut sy = sy1;
            loop {
                let porcion_y;
                if truncar(sy) == truncar(sy1) {
                    porcion_y = (1.0 - (sy - truncar(sy))).min(sy2 - sy1);
                    sy = truncar(sy);
                } else if sy == piso_f32(sy2) {
                    porcion_y = sy2 - truncar(sy2);
                } else {
                    porcion_y = 1.0;
                }
                let mut sx = sx1;
                loop {
                    let porcion_x;
                    if piso_f32(sx) == truncar(sx1) {
                        porcion_x = (1.0 - (sx - truncar(sx))).min(sx2 - sx1);
                        sx = truncar(sx);
                    } else if sx == piso_f32(sx2) {
                        porcion_x = sx2 - truncar(sx2);
                    } else {
                        porcion_x = 1.0;
                    }
                    let contribucion = porcion_x * porcion_y;
                    let p = origen.pixel(sx as usize, sy as usize);
                    let factor_alfa = (127.0 - p.alfa as f64) * contribucion;
                    rojo += p.rojo as f64 * factor_alfa;
                    verde += p.verde as f64 * factor_alfa;
                    azul += p.azul as f64 * factor_alfa;
                    alfa += p.alfa as f64 * contribucion;
                    suma_alfa += factor_alfa;
                    suma_contribucion += contribucion;
                    cantidad += porcion_x * porcion_y;
                    sx += 1.0;
                    if sx >= sx2 {
                        break;
                    }
                }
                sy += 1.0;
                if sy >= sy2 {
                    break;
                }
            }
            if cantidad != 0.0 {
                rojo /= cantidad;
                verde /= cantidad;
                azul /= cantidad;
                alfa /= cantidad;
            }
            if suma_alfa != 0.0 {
                if suma_contribucion != 0.0 {
                    suma_alfa /= suma_contribucion;
                }
                rojo /= suma_alfa;
                verde /= suma_alfa;
                azul /= suma_alfa;
            }
            let redondear = |valor: f64, maximo: f64| if valor >= maximo + 0.5 { maximo as u8 } else { (valor + 0.5) as u8 };
            pixeles.push(Pixel {
                rojo: redondear(rojo, 255.0),
                verde: redondear(verde, 255.0),
                azul: redondear(azul, 255.0),
                alfa: redondear(alfa, 127.0),
            });
        }
    }
    Lienzo { ancho, alto, pixeles }
}
