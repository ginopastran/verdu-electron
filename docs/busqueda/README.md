# Búsqueda de productos (KAN-19 + KAN-20)

Un mismo criterio en la web (`src/lib/busqueda/productos.ts`) y en el POS (`crates/nucleo/src/busqueda.rs`). Los dos pasan los casos de `casos.json`. Si se cambia una regla, se cambia acá, en los dos lados y en los casos.

## Normalización (`normalizar`)

1. Minúsculas y sin tildes (NFD, sacar marcas). `ñ` → `n`.
2. Coma decimal entre dígitos → punto: `2,5` → `2.5`.
3. Dimensiones: `(\d) *[x×*] *(\d)` → `$1x$2`, repetido (`6 x 40 x 2` → `6x40x2`).
4. Fracciones: `(\d) */ *(\d)` → `$1/$2`.
5. Una `x` que no viene después de un dígito y va antes de un número significa "por" y se borra: `(^|[^0-9])x *(?=\d)` → `$1 ` (`x 500 ml` → ` 500 ml`, `x41grs` → ` 41grs`).
6. Unidad pegada al número: `(\d) +(mm|cm|m|mts|kg|g|gr|grs|l|lt|lts|ml|cc|v|w|a|hp|pulg)\b` → `$1$2`.
7. Tokens: `\d+(?:[./]\d+)*(?:x\d+(?:[./]\d+)*)*[a-z]*` o `[a-z0-9]+`. Todo lo demás separa.

Un token que empieza con dígito es **numérico**: núcleo = la parte numérica (`6x40`, `1/2`, `2.5`, `012`) y unidad = las letras finales (`mm`, `w`, vacía). El resto son tokens de **texto**.

Clave fonética de un token de texto (solo para comparar aproximado): `ll`→`y`, `v`→`b`, `z`→`s`, `ce`→`se`, `ci`→`si`, `h` inicial se borra.

## Qué se compara de cada producto

Texto: `nombre` + cada alias. Códigos: `codigoBarras` y `plu`, tal cual (con ceros iniciales).

## Reglas

- **Código exacto:** si la consulta sin espacios es igual a `codigoBarras` o `plu`, ese producto va primero (puntaje 1000). Si además la consulta es solo dígitos y tiene 8 o más, es un código de barras: se devuelven solo las coincidencias exactas, sin aproximar.
- **Prefijo de código:** consulta de 3 o más dígitos con la que empieza `codigoBarras` o `plu`: puntaje 100.
- **Numéricos, estrictos:** cada token numérico de la consulta tiene que coincidir con un token numérico del producto. Coincide si el núcleo es igual, o si el producto tiene una dimensión (`6x40`) y el núcleo de la consulta es una de sus partes. Si los dos tienen unidad, tiene que ser la misma. `8` no coincide con `80`, ni `1/2` con `1/4` o `12`.
- **Texto, tolerante:** cada token de texto de la consulta tiene que coincidir con algún token de texto del producto, en cualquier orden:
  - igual: 3 puntos;
  - el token del producto empieza con el de la consulta: 2;
  - aproximado: 1. Es la distancia Damerau-Levenshtein entre claves fonéticas, con un máximo que depende del largo del token de la consulta: 0 si tiene hasta 3 letras, 1 si tiene 4 o 5, y 2 si tiene 6 o más. Una coincidencia que solo pasa por fonética o distancia marca el resultado como `aproximado`.
- Cada numérico coincidente suma 2. Si el nombre empieza con el primer token de texto coincidente, suma 1.
- **Orden:** puntaje descendente; a igual puntaje, nombre más corto y después alfabético.
- **Otras medidas:** si la consulta tiene numéricos y no hay resultados, se repite sin los numéricos. Lo que aparece va en `otrasMedidas` y nunca en `resultados`.

## Casos

`casos.json` tiene un catálogo chico y las consultas con lo esperado:

- `incluye`: ids que tienen que estar;
- `excluye`: ids que no;
- `primero`: el id del primer resultado;
- `exactamente`: la lista completa;
- `aproximado`: si el primer resultado es aproximado;
- `otrasMedidas`: ids que tienen que aparecer en otras medidas.
