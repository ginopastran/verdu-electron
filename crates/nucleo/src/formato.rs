pub fn numero_con(valor: f64, minimo: usize, maximo: usize) -> String {
    if !valor.is_finite() {
        return "NaN".into();
    }
    let crudo = format!("{:.*}", maximo, valor.abs());
    let (entero, decimales) = crudo.split_once('.').unwrap_or((&crudo, ""));
    let mut decimales = decimales.trim_end_matches('0').to_string();
    while decimales.len() < minimo {
        decimales.push('0');
    }
    let mut agrupado = String::new();
    for (posicion, digito) in entero.chars().enumerate() {
        if posicion > 0 && (entero.len() - posicion) % 3 == 0 {
            agrupado.push('.');
        }
        agrupado.push(digito);
    }
    let cero = agrupado.chars().all(|c| c == '0' || c == '.') && decimales.chars().all(|c| c == '0');
    let signo = if valor < 0.0 && !cero { "-" } else { "" };
    if decimales.is_empty() {
        format!("{signo}{agrupado}")
    } else {
        format!("{signo}{agrupado},{decimales}")
    }
}

pub fn numero(valor: f64) -> String {
    numero_con(valor, 0, 3)
}

pub fn decimal2(valor: f64) -> String {
    numero_con(valor, 2, 2)
}

pub fn pesos(valor: f64) -> String {
    format!("${}", decimal2(valor))
}

pub fn pesos_sin_ceros(valor: f64) -> String {
    format!("${}", numero(valor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formato_argentino() {
        assert_eq!(numero(22449.5), "22.449,5");
        assert_eq!(numero(1234567.0), "1.234.567");
        assert_eq!(numero(0.1234), "0,123");
        assert_eq!(decimal2(1500.0), "1.500,00");
        assert_eq!(pesos(-150.256), "$-150,26");
        assert_eq!(pesos_sin_ceros(8500.0), "$8.500");
        assert_eq!(decimal2(-0.001), "0,00");
    }
}
