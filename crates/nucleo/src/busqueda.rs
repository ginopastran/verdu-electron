use std::cell::RefCell;

use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

const UNIDADES: [&str; 18] = ["mm", "cm", "m", "mts", "kg", "g", "gr", "grs", "l", "lt", "lts", "ml", "cc", "v", "w", "a", "hp", "pulg"];

#[derive(Clone, Debug, Default, PartialEq)]
struct Numerico {
    nucleo: String,
    unidad: String,
    partes: Vec<String>,
}

#[derive(Default)]
struct Tokens {
    texto: Vec<String>,
    numeros: Vec<Numerico>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemBusqueda {
    texto: Vec<String>,
    fonetica: Vec<String>,
    numeros: Vec<Numerico>,
    primero: String,
    nombre: String,
    plegado: String,
    largo: usize,
    codigo_barras: Option<String>,
    plu: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resultado {
    pub indice: usize,
    pub puntaje: i32,
    pub aproximado: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Busqueda {
    pub resultados: Vec<Resultado>,
    pub otras_medidas: Vec<Resultado>,
}

impl ItemBusqueda {
    pub fn nuevo(nombre: &str, alias: &[String], codigo_barras: Option<&str>, plu: Option<&str>) -> Self {
        let del_nombre = tokenizar(nombre);
        let mut texto = del_nombre.texto;
        let mut numeros = del_nombre.numeros;
        let primero = texto.first().cloned().unwrap_or_default();
        for a in alias {
            let t = tokenizar(a);
            texto.extend(t.texto);
            numeros.extend(t.numeros);
        }
        ItemBusqueda {
            fonetica: texto.iter().map(|t| fonetica(t)).collect(),
            texto,
            numeros,
            primero,
            nombre: nombre.to_string(),
            plegado: sin_tildes(nombre),
            largo: nombre.encode_utf16().count(),
            codigo_barras: codigo_barras.map(str::to_string),
            plu: plu.map(str::to_string),
        }
    }

    pub fn de_json(p: &Value) -> Self {
        let codigo = |v: &Value| match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        };
        let alias: Vec<String> = p["alias"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_string)).collect();
        ItemBusqueda::nuevo(p["nombre"].as_str().unwrap_or_default(), &alias, codigo(&p["codigoBarras"]).as_deref(), codigo(&p["plu"]).as_deref())
    }
}

impl AsRef<ItemBusqueda> for ItemBusqueda {
    fn as_ref(&self) -> &ItemBusqueda {
        self
    }
}

fn sin_tildes(texto: &str) -> String {
    texto.to_lowercase().nfd().filter(|c| !('\u{300}'..='\u{36f}').contains(c)).collect()
}

fn digito(c: Option<&char>) -> bool {
    c.is_some_and(|c| c.is_ascii_digit())
}

fn saltar_espacios(c: &[char], mut i: usize) -> usize {
    while c.get(i) == Some(&' ') {
        i += 1;
    }
    i
}

fn unir_numeros(c: &[char], separadores: &[char], union: char) -> Vec<char> {
    let mut salida = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        if c[i].is_ascii_digit() {
            let j = saltar_espacios(c, i + 1);
            if c.get(j).is_some_and(|s| separadores.contains(s)) {
                let k = saltar_espacios(c, j + 1);
                if digito(c.get(k)) {
                    salida.extend([c[i], union, c[k]]);
                    i = k + 1;
                    continue;
                }
            }
        }
        salida.push(c[i]);
        i += 1;
    }
    salida
}

fn x_por_numero(c: &[char], i: usize) -> Option<usize> {
    (c.get(i) == Some(&'x')).then(|| saltar_espacios(c, i + 1)).filter(|&k| digito(c.get(k)))
}

fn sin_por(c: &[char]) -> Vec<char> {
    let mut salida = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        if i == 0 {
            if let Some(k) = x_por_numero(c, 0) {
                salida.push(' ');
                i = k;
                continue;
            }
        }
        if !c[i].is_ascii_digit() {
            if let Some(k) = x_por_numero(c, i + 1) {
                salida.extend([c[i], ' ']);
                i = k;
                continue;
            }
        }
        salida.push(c[i]);
        i += 1;
    }
    salida
}

fn palabra(c: Option<&char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
}

fn pegar_unidades(c: &[char]) -> Vec<char> {
    let mut salida = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        if c[i].is_ascii_digit() && c.get(i + 1) == Some(&' ') {
            let j = saltar_espacios(c, i + 1);
            let unidad = UNIDADES.iter().find(|u| {
                let fin = j + u.len();
                fin <= c.len() && c[j..fin].iter().copied().eq(u.chars()) && !palabra(c.get(fin))
            });
            if let Some(u) = unidad {
                salida.push(c[i]);
                salida.extend(u.chars());
                i = j + u.len();
                continue;
            }
        }
        salida.push(c[i]);
        i += 1;
    }
    salida
}

pub fn normalizar(texto: &str) -> String {
    let c: Vec<char> = sin_tildes(texto).chars().collect();
    let mut c = {
        let mut salida = Vec::with_capacity(c.len());
        let mut i = 0;
        while i < c.len() {
            if c[i].is_ascii_digit() && c.get(i + 1) == Some(&',') && digito(c.get(i + 2)) {
                salida.extend([c[i], '.', c[i + 2]]);
                i += 3;
            } else {
                salida.push(c[i]);
                i += 1;
            }
        }
        salida
    };
    loop {
        let siguiente = unir_numeros(&c, &['x', '×', '*'], 'x');
        if siguiente == c {
            break;
        }
        c = siguiente;
    }
    let c = unir_numeros(&c, &['/'], '/');
    let c = sin_por(&c);
    pegar_unidades(&c).into_iter().collect()
}

fn digitos(c: &[u8], mut i: usize) -> usize {
    while i < c.len() && c[i].is_ascii_digit() {
        i += 1;
    }
    i
}

fn decimales(c: &[u8], mut i: usize) -> usize {
    while i + 1 < c.len() && matches!(c[i], b'.' | b'/') && c[i + 1].is_ascii_digit() {
        i = digitos(c, i + 1);
    }
    i
}

fn tokenizar(texto: &str) -> Tokens {
    let normal = normalizar(texto);
    let c = normal.as_bytes();
    let mut tokens = Tokens::default();
    let mut i = 0;
    while i < c.len() {
        if c[i].is_ascii_digit() {
            let mut fin = decimales(c, digitos(c, i));
            while fin + 1 < c.len() && c[fin] == b'x' && c[fin + 1].is_ascii_digit() {
                fin = decimales(c, digitos(c, fin + 1));
            }
            let corte = fin;
            while fin < c.len() && c[fin].is_ascii_lowercase() {
                fin += 1;
            }
            let nucleo = normal[i..corte].to_string();
            let partes = if nucleo.contains('x') { nucleo.split('x').map(str::to_string).collect() } else { Vec::new() };
            tokens.numeros.push(Numerico { nucleo, unidad: normal[corte..fin].to_string(), partes });
            i = fin;
        } else if c[i].is_ascii_lowercase() {
            let mut fin = i;
            while fin < c.len() && (c[fin].is_ascii_lowercase() || c[fin].is_ascii_digit()) {
                fin += 1;
            }
            tokens.texto.push(normal[i..fin].to_string());
            i = fin;
        } else {
            i += 1;
        }
    }
    tokens
}

fn fonetica(token: &str) -> String {
    let t = token.strip_prefix('h').unwrap_or(token).replace("ll", "y").replace('v', "b").replace('z', "s");
    t.replace("ce", "se").replace("ci", "si")
}

thread_local! {
    static TABLA_DISTANCIA: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

fn distancia(a: &[u8], b: &[u8]) -> usize {
    let ancho = b.len() + 1;
    TABLA_DISTANCIA.with_borrow_mut(|d| {
        d.clear();
        d.resize((a.len() + 1) * ancho, 0);
        for i in 0..=a.len() {
            d[i * ancho] = i;
        }
        for j in 0..=b.len() {
            d[j] = j;
        }
        for i in 1..=a.len() {
            for j in 1..=b.len() {
                let costo = usize::from(a[i - 1] != b[j - 1]);
                let mut v = (d[(i - 1) * ancho + j] + 1).min(d[i * ancho + j - 1] + 1).min(d[(i - 1) * ancho + j - 1] + costo);
                if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                    v = v.min(d[(i - 2) * ancho + j - 2] + 1);
                }
                d[i * ancho + j] = v;
            }
        }
        d[a.len() * ancho + b.len()]
    })
}

fn tolerancia(largo: usize) -> usize {
    match largo {
        0..=3 => 0,
        4..=5 => 1,
        _ => 2,
    }
}

fn coincide_numero(q: &Numerico, p: &Numerico) -> bool {
    let nucleo = q.nucleo == p.nucleo || p.partes.contains(&q.nucleo);
    nucleo && (q.unidad.is_empty() || p.unidad.is_empty() || q.unidad == p.unidad)
}

struct Consulta {
    texto: Vec<(String, String)>,
    numeros: Vec<Numerico>,
}

fn puntaje_texto<'a>(q: &str, qf: &str, p: &'a ItemBusqueda) -> Option<(i32, bool, &'a str)> {
    let maximo = tolerancia(q.len());
    let mut mejor: Option<(i32, bool, &str)> = None;
    for (t, f) in p.texto.iter().zip(&p.fonetica) {
        let candidato = if t == q {
            Some((3, false))
        } else if t.starts_with(q) {
            Some((2, false))
        } else if maximo > 0 && f.len().abs_diff(qf.len()) <= maximo && distancia(qf.as_bytes(), f.as_bytes()) <= maximo {
            Some((1, true))
        } else {
            None
        };
        if let Some((puntos, aproximado)) = candidato {
            if mejor.is_none_or(|m| puntos > m.0) {
                mejor = Some((puntos, aproximado, t));
            }
        }
    }
    mejor
}

fn puntuar(consulta: &Consulta, p: &ItemBusqueda, con_numeros: bool) -> Option<(i32, bool)> {
    let mut puntaje = 0;
    let mut aproximado = false;
    let mut primer_token = None;
    for (q, qf) in &consulta.texto {
        let (puntos, aprox, token) = puntaje_texto(q, qf, p)?;
        puntaje += puntos;
        aproximado |= aprox;
        primer_token.get_or_insert(token);
    }
    if con_numeros {
        for q in &consulta.numeros {
            if !p.numeros.iter().any(|n| coincide_numero(q, n)) {
                return None;
            }
            puntaje += 2;
        }
    }
    if primer_token.is_some_and(|t| t == p.primero) {
        puntaje += 1;
    }
    Some((puntaje, aproximado))
}

fn ordenar<T: AsRef<ItemBusqueda>>(productos: &[T], lista: &mut [Resultado]) {
    lista.sort_by(|a, b| {
        let (pa, pb) = (productos[a.indice].as_ref(), productos[b.indice].as_ref());
        b.puntaje.cmp(&a.puntaje).then(pa.largo.cmp(&pb.largo)).then_with(|| pa.plegado.cmp(&pb.plegado)).then_with(|| pa.nombre.cmp(&pb.nombre))
    });
}

pub fn buscar<T: AsRef<ItemBusqueda>>(productos: &[T], consulta: &str) -> Busqueda {
    let compacta: String = consulta.split_whitespace().collect();
    if compacta.is_empty() {
        return Busqueda::default();
    }
    let es_codigo = |p: &ItemBusqueda| p.codigo_barras.as_deref() == Some(compacta.as_str()) || p.plu.as_deref() == Some(compacta.as_str());
    let mut resultados: Vec<Resultado> =
        productos.iter().enumerate().filter(|(_, p)| es_codigo(p.as_ref())).map(|(indice, _)| Resultado { indice, puntaje: 1000, aproximado: false }).collect();
    let solo_digitos = compacta.bytes().all(|b| b.is_ascii_digit());
    if solo_digitos && compacta.len() >= 8 {
        return Busqueda { resultados, otras_medidas: Vec::new() };
    }
    let tokens = tokenizar(consulta);
    let consulta = Consulta { texto: tokens.texto.into_iter().map(|t| { let f = fonetica(&t); (t, f) }).collect(), numeros: tokens.numeros };
    let prefijo_codigo = solo_digitos && compacta.len() >= 3;
    let vacia = consulta.texto.is_empty() && consulta.numeros.is_empty();
    for (indice, p) in productos.iter().enumerate() {
        let p = p.as_ref();
        if es_codigo(p) {
            continue;
        }
        let empieza = |c: &Option<String>| c.as_deref().is_some_and(|c| c.starts_with(compacta.as_str()));
        if prefijo_codigo && (empieza(&p.codigo_barras) || empieza(&p.plu)) {
            resultados.push(Resultado { indice, puntaje: 100, aproximado: false });
            continue;
        }
        if vacia {
            continue;
        }
        if let Some((puntaje, aproximado)) = puntuar(&consulta, p, true) {
            resultados.push(Resultado { indice, puntaje, aproximado });
        }
    }
    let mut otras_medidas = Vec::new();
    if resultados.is_empty() && !consulta.numeros.is_empty() && !consulta.texto.is_empty() {
        otras_medidas = productos
            .iter()
            .enumerate()
            .filter_map(|(indice, p)| puntuar(&consulta, p.as_ref(), false).map(|(puntaje, aproximado)| Resultado { indice, puntaje, aproximado }))
            .collect();
        ordenar(productos, &mut otras_medidas);
    }
    ordenar(productos, &mut resultados);
    Busqueda { resultados, otras_medidas }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use serde_json::json;

    fn ids(productos: &[Value], lista: &[Resultado]) -> Vec<i64> {
        lista.iter().map(|r| productos[r.indice]["id"].as_i64().unwrap()).collect()
    }

    fn lista(valor: &Value) -> Vec<i64> {
        valor.as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect()
    }

    #[test]
    fn casos_de_la_spec() {
        let casos: Value = serde_json::from_str(include_str!("../../../docs/busqueda/casos.json")).unwrap();
        let productos = casos["productos"].as_array().unwrap();
        let items: Vec<ItemBusqueda> = productos.iter().map(ItemBusqueda::de_json).collect();
        let mut fallas = Vec::new();
        for caso in casos["consultas"].as_array().unwrap() {
            let q = caso["q"].as_str().unwrap();
            let b = buscar(&items, q);
            let obtenidos = ids(productos, &b.resultados);
            let otras = ids(productos, &b.otras_medidas);
            let mut fallar = |motivo: String| fallas.push(format!("{q:?}: {motivo} (resultados {obtenidos:?}, otras {otras:?})"));
            if let Some(v) = caso.get("incluye") {
                for id in lista(v).into_iter().filter(|id| !obtenidos.contains(id)) {
                    fallar(format!("falta {id}"));
                }
            }
            if let Some(v) = caso.get("excluye") {
                for id in lista(v).into_iter().filter(|id| obtenidos.contains(id)) {
                    fallar(format!("sobra {id}"));
                }
            }
            if let Some(v) = caso.get("primero") {
                if obtenidos.first() != v.as_i64().as_ref() {
                    fallar(format!("primero esperado {v}"));
                }
            }
            if let Some(v) = caso.get("exactamente") {
                let (mut a, mut e) = (obtenidos.clone(), lista(v));
                a.sort();
                e.sort();
                if a != e {
                    fallar(format!("exactamente {e:?}"));
                }
            }
            if let Some(v) = caso.get("aproximado") {
                if b.resultados.first().map(|r| r.aproximado) != v.as_bool() {
                    fallar(format!("aproximado esperado {v}"));
                }
            }
            if let Some(v) = caso.get("otrasMedidas") {
                for id in lista(v).into_iter().filter(|id| !otras.contains(id)) {
                    fallar(format!("falta {id} en otras medidas"));
                }
                if otras.iter().any(|id| obtenidos.contains(id)) {
                    fallar("otras medidas repite resultados".into());
                }
            }
        }
        assert!(fallas.is_empty(), "{}", fallas.join("\n"));
    }

    #[test]
    fn normaliza_como_la_spec() {
        assert_eq!(normalizar("Cable 2,5 MM"), "cable 2.5mm");
        assert_eq!(normalizar("6 x 40 x 2"), "6x40x2");
        assert_eq!(normalizar("tornillo 6×40"), "tornillo 6x40");
        assert_eq!(normalizar("1 / 2\""), "1/2\"");
        assert_eq!(normalizar("x 500 ml"), " 500ml");
        assert_eq!(normalizar("Nesquik x41grs"), "nesquik  41grs");
        assert_eq!(normalizar("Ñandú 5 mts"), "nandu 5mts");
        assert_eq!(normalizar("5 mtsx"), "5 mtsx");
        let t = tokenizar("Mecha 1/2\" 6x40mm abc12 2.5.");
        assert_eq!(t.texto, ["mecha", "abc12"]);
        let nucleos: Vec<(&str, &str)> = t.numeros.iter().map(|n| (n.nucleo.as_str(), n.unidad.as_str())).collect();
        assert_eq!(nucleos, [("1/2", ""), ("6x40", "mm"), ("2.5", "")]);
        assert_eq!(fonetica("hallazgo"), "ayasgo");
        assert_eq!(fonetica("cerveza"), "serbesa");
        assert_eq!(distancia(b"bnana", b"banana"), 1);
        assert_eq!(distancia(b"ab", b"ba"), 1);
    }

    #[test]
    fn de_json_lee_alias_y_codigos_numericos() {
        let item = ItemBusqueda::de_json(&json!({ "nombre": "Precinto", "alias": ["Brida", 3], "codigoBarras": 7791234000010i64, "plu": null }));
        assert_eq!(item.texto, ["precinto", "brida"]);
        assert_eq!(item.codigo_barras.as_deref(), Some("7791234000010"));
        assert!(buscar(&[item], "   ").resultados.is_empty());
    }

    #[test]
    fn veinte_mil_productos_en_menos_de_50_ms() {
        let nombres = ["Tornillo autoperforante", "Tarugo nylon", "Mecha acero rápido", "Lámpara LED", "Cable unipolar", "Banana Ecuador", "Salsa de soja", "Precinto plástico"];
        let inicio_carga = std::time::Instant::now();
        let items: Vec<ItemBusqueda> = (0..20_000)
            .map(|i| {
                let nombre = format!("{} {}x{} mm modelo {}", nombres[i % nombres.len()], i % 13 + 2, i % 97 + 10, i);
                ItemBusqueda::nuevo(&nombre, &[format!("alias{}", i % 50)], Some(&format!("779{i:010}")), Some(&format!("{:03}", i % 1000)))
            })
            .collect();
        let carga = inicio_carga.elapsed();
        let consultas = ["torniyo 6x40", "lampara", "mecha 1/2", "bnana", "cable 2,5", "tornillo 6x45", "precinto plastico 200", "7790000012345", "012"];
        let mut peor = std::time::Duration::ZERO;
        for q in consultas {
            let inicio = std::time::Instant::now();
            std::hint::black_box(buscar(&items, q));
            peor = peor.max(inicio.elapsed());
        }
        eprintln!("carga de 20000: {carga:?}, peor consulta: {peor:?}");
        if !cfg!(debug_assertions) {
            assert!(peor < std::time::Duration::from_millis(50), "peor consulta {peor:?}");
        }
    }
}
