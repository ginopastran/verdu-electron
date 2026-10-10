use eframe::egui::epaint::{Mesh, RectShape, TessellationOptions, Tessellator};
use eframe::egui::{Color32, CornerRadius, Painter, Rect, Shape, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direccion {
    Derecha,
    Abajo,
    AbajoDerecha,
}

pub fn pintar_degradado(
    pintor: &Painter,
    rect: Rect,
    radio: impl Into<CornerRadius>,
    desde: Color32,
    hasta: Color32,
    direccion: Direccion,
) {
    let mut malla = Mesh::default();
    let mut teselador = Tessellator::new(
        pintor.pixels_per_point(),
        TessellationOptions::default(),
        [1, 1],
        Vec::new(),
    );
    teselador.tessellate_rect(&RectShape::filled(rect, radio, Color32::WHITE), &mut malla);
    let eje = match direccion {
        Direccion::Derecha => Vec2::new(1.0, 0.0),
        Direccion::Abajo => Vec2::new(0.0, 1.0),
        Direccion::AbajoDerecha => Vec2::new(rect.width(), rect.height()).normalized(),
    };
    let inicio = rect.min.to_vec2().dot(eje);
    let largo = (rect.max - rect.min).dot(eje).max(1.0);
    for vertice in &mut malla.vertices {
        let t = ((vertice.pos.to_vec2().dot(eje) - inicio) / largo).clamp(0.0, 1.0);
        let alfa = vertice.color.a() as f32 / 255.0;
        vertice.color = mezclar(desde, hasta, t).gamma_multiply(alfa);
    }
    pintor.add(Shape::mesh(malla));
}

pub fn mezclar(desde: Color32, hasta: Color32, t: f32) -> Color32 {
    let canal = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        canal(desde.r(), hasta.r()),
        canal(desde.g(), hasta.g()),
        canal(desde.b(), hasta.b()),
        canal(desde.a(), hasta.a()),
    )
}
