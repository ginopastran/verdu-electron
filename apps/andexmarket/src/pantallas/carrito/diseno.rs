use std::sync::Arc;

use eframe::egui::{Color32, Galley, Painter, Pos2};

use crate::ui::{fuente, Peso, Tam};

pub fn encoger(bases: &[f32], minimos: &[f32], disponible: f32) -> Vec<f32> {
    let mut tamanos = bases.to_vec();
    if bases.iter().sum::<f32>() <= disponible {
        return tamanos;
    }
    let mut congelado = vec![false; bases.len()];
    loop {
        let fijo: f32 = (0..bases.len()).filter(|&i| congelado[i]).map(|i| tamanos[i]).sum();
        let peso: f32 = (0..bases.len()).filter(|&i| !congelado[i]).map(|i| bases[i]).sum();
        if peso <= 0.0 {
            return tamanos;
        }
        let libre = disponible - fijo - peso;
        let mut violo = false;
        let libres: Vec<usize> = (0..bases.len()).filter(|&i| !congelado[i]).collect();
        for i in libres {
            tamanos[i] = bases[i] + libre * bases[i] / peso;
            if tamanos[i] < minimos[i] {
                tamanos[i] = minimos[i];
                congelado[i] = true;
                violo = true;
            }
        }
        if !violo {
            return tamanos;
        }
    }
}

pub struct Texto {
    pub contenido: String,
    pub tam: Tam,
    pub peso: Peso,
    pub color: Color32,
}

impl Texto {
    pub fn new(contenido: impl Into<String>, tam: Tam, peso: Peso, color: Color32) -> Self {
        Texto { contenido: contenido.into(), tam, peso, color }
    }

    pub fn maximo(&self, pintor: &Painter) -> f32 {
        pintor.layout_no_wrap(self.contenido.clone(), fuente(self.tam, self.peso), self.color).size().x
    }

    pub fn minimo(&self, pintor: &Painter) -> f32 {
        self.contenido
            .split(' ')
            .map(|palabra| pintor.layout_no_wrap(palabra.to_string(), fuente(self.tam, self.peso), self.color).size().x)
            .fold(0.0, f32::max)
    }

    pub fn galeria(&self, pintor: &Painter, ancho: f32) -> Arc<Galley> {
        pintor.layout(self.contenido.clone(), fuente(self.tam, self.peso), self.color, ancho + 0.5)
    }

    pub fn alto(&self, galeria: &Galley) -> f32 {
        galeria.rows.len().max(1) as f32 * self.tam.alto_linea()
    }

    pub fn pintar(&self, pintor: &Painter, galeria: &Galley, origen: Pos2) {
        let alto_linea = self.tam.alto_linea();
        for (i, fila) in galeria.rows.iter().enumerate() {
            let linea: String = fila.row.glyphs.iter().map(|g| g.chr).collect();
            let renglon = pintor.layout_no_wrap(linea.trim_end().to_string(), fuente(self.tam, self.peso), self.color);
            let y = origen.y + i as f32 * alto_linea + (alto_linea - renglon.size().y) / 2.0;
            pintor.galley(Pos2::new(origen.x, y), renglon, self.color);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::encoger;

    #[test]
    fn encoge_como_flexbox() {
        assert_eq!(encoger(&[100.0, 50.0], &[10.0, 10.0], 200.0), vec![100.0, 50.0]);
        let t = encoger(&[300.0, 100.0], &[0.0, 0.0], 200.0);
        assert!((t[0] - 150.0).abs() < 0.01 && (t[1] - 50.0).abs() < 0.01);
        let t = encoger(&[300.0, 100.0], &[0.0, 90.0], 200.0);
        assert!((t[0] - 110.0).abs() < 0.01 && (t[1] - 90.0).abs() < 0.01);
        let t = encoger(&[300.0, 100.0], &[250.0, 90.0], 200.0);
        assert_eq!(t, vec![250.0, 90.0]);
    }
}
