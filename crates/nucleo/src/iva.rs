pub fn precio_visual_con_iva(precio: f64, iva_incluido: bool, iva_porcentaje: Option<f64>, iva_incluido_en_precios: bool) -> f64 {
    if iva_incluido_en_precios || iva_incluido {
        return precio;
    }
    precio + precio * (iva_porcentaje.unwrap_or(21.0) / 100.0)
}
