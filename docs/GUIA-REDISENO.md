# Guía del rediseño (kit v2)

El POS pasa del look shadcn (`crate::ui`, kit v1) al diseño **Andex POS** (`design/redise-o-profesional-de-app/project/Andex POS.dc.html`). Esta guía junta los tokens, la API del kit nuevo (`crate::ui::v2`) y las reglas para portar cada pantalla.

Vitrinas de referencia (regenerar con `cargo test -p andexmarket --test captura_kit_v2`):
- `docs/capturas/00-kit-v2.png`: encabezado oscuro, campos, botones, teclas, chips, segmentados, opciones, avatares, shimmer, KPI y tabla.
- `docs/capturas/00-kit-v2-modal.png`: modal «Impresión» con desplegable abierto y toast.
- `docs/capturas/00-iconos-solar.png`: los 47 íconos Solar y el tinte con varios colores y tamaños.

## Reglas para portar una pantalla

1. **La lógica no se toca.** Estados, llamadas a la API, validaciones, atajos, foco y mensajes quedan igual. Solo cambia la presentación: qué widget se llama y con qué medidas y colores.
2. **Medidas exactas del HTML.** 1px CSS = 1 punto de egui. Buscá la pantalla en `Andex POS.dc.html` (los bloques están comentados: `CONFIGURACIÓN INICIAL`, `QUIÉN ESTÁ VENDIENDO`, `POS`, `CUENTA CORRIENTE`, `MODALES`) y copiá alto, relleno, radio, gap, tamaño y peso de letra. Los textos de la UI salen del HTML y del método `renderVals()` del script del final.
3. **Colores solo desde `ui::marca::color`.** Nada de `ui::paleta` ni `ui::color` en pantallas portadas. Si falta un color, agregalo a `marca.rs` con nombre semántico.
4. **Íconos solo Solar** (`icono_solar`, `pintar_icono_solar*`, o el nombre en los componentes v2).
5. **Toasts:** `ui::v2::toast::{exito, error, info, aviso, con_detalle}`. Hay un solo toast visible a la vez (reemplaza al anterior), 2,8 s, arriba al centro, como en el diseño. `app.rs` los pinta una vez por cuadro con `ui::v2::toast::pintar`.
6. **Sin comentarios en el código**, nombres en español (ver `docs/GUIA-PORTE.md`).
7. **El kit v1 ya no existe**: se borraron `controles`, `contenedores`, `degradado`, `paleta`, el toast viejo y los íconos Lucide. De `crate::ui` quedan `instalar`, `Peso`, `marca`, los íconos Solar, `imagen` y `v2`.
8. **Captura obligatoria:** cada pantalla portada tiene su test de captura y se compara contra el HTML abierto en el navegador (servir la carpeta del diseño por HTTP, por ejemplo `python -m http.server`, y abrir `Andex POS.dc.html`; la prop `pantallaInicial` del script elige `admin`, `usuarios`, `pos` o `cuenta`).
9. El `item_spacing` global es 0: los gaps del HTML se ponen con `ui.add_space` o `spacing_mut().item_spacing`.
10. **Filas horizontales:** dentro de un layout vertical usá `ui.horizontal(..)`. `ui.with_layout(Layout::left_to_right(Align::Center), ..)` ocupa todo el alto disponible.

## Tokens (`crate::ui::marca`)

### Colores (`marca::color`)

| Token | Hex | Uso en el diseño |
|---|---|---|
| `VERDE_PROFUNDO` | #0A3A2B | encabezado, tarjeta Total, hover del primario, texto sobre dorado |
| `VERDE` | #1F5444 | botón primario, íconos, foco, segmento de filtro activo |
| `VERDE_HOVER_OSCURO` | #134634 | hover de botones del encabezado, círculos de pasos |
| `VERDE_EXITO` | #14653F | montos pagados, «En línea», chip Pagada |
| `VERDE_PUNTO` | #22A06B | punto de estado conectado |
| `VERDE_CHIP` | #EAF2EE | fondo de chip de ícono, barras de progreso |
| `VERDE_ELEGIDO` | #F1F7F4 | opción elegida, fila resaltada, hover de resultados |
| `VERDE_EXITO_FONDO` | #E3F1EA | chip Pagada / Factura B / Efectivo |
| `VERDE_TOAST` | #6FC79F | ícono del toast de éxito |
| `DORADO` | #D39801 | acentos, avatar del usuario, ADMIN, íconos sobre verde |
| `DORADO_OSCURO` | #B07D00 | ícono sobre fondo dorado (modal de aviso) |
| `DORADO_TEXTO` | #7A5800 | texto de chips/avisos dorados, vuelto |
| `DORADO_FONDO` | #FDF4DC | chip Remito/Parcial, caja de vuelto, destello de fila nueva |
| `DORADO_HOVER` | #FDF8EC | botón dorado, hover de Reconfigurar |
| `DORADO_BORDE` | #E8C770 | tecla dentro del botón dorado |
| `DORADO_AVATAR` | #9A6F00 | tercer color de avatares |
| `FONDO_APP` | #F6F8F6 | fondo general, hover de ítems y botones de ícono |
| `BLANCO` | #FFFFFF | tarjetas, campos |
| `FONDO_ENCABEZADO_TABLA` | #FBFCFB | encabezados de tabla, fondo de teclas claras |
| `FILA_HOVER` | #F8FAF9 | hover de filas |
| `SHIMMER_BASE` / `SHIMMER_BRILLO` | #EDF2EF / #F7FAF8 | esqueletos de carga |
| `BORDE` | #DCE4DF | bordes de tarjetas, campos y botones secundarios |
| `BORDE_SUAVE` | #EDF2EF | divisores, bordes de filas |
| `BORDE_HOVER` | #B6C4BD | hover del desplegable, borde punteado «Nueva orden» |
| `INTERRUPTOR_APAGADO` | #C9D3CE | interruptor apagado |
| `TEXTO` | #101828 | texto principal |
| `TEXTO_SECUNDARIO` | #344054 | etiquetas de campos, botones secundarios |
| `TEXTO_APAGADO` | #667085 | subtítulos, encabezados de tabla |
| `TEXTO_TENUE` | #7C8B84 | íconos de campos, ayudas, PLU |
| `TEXTO_DESHABILITADO` | #98A2B3 | ícono de quitar, «Sin descuento», detalle del toast |
| `TEXTO_SUGERENCIA` | #757575 | placeholder (el del navegador) |
| `SOBRE_OSCURO` | #E3ECE7 | texto de botones del encabezado |
| `SOBRE_OSCURO_ICONO` | #B6C4BD | íconos del encabezado |
| `SOBRE_OSCURO_APAGADO` | #9DB8AC | reloj, subtítulos sobre verde |
| `SOBRE_OSCURO_TENUE` | #7FA897 | IVA incluido, versión |
| `SOBRE_OSCURO_PARRAFO` / `SOBRE_OSCURO_LISTA` | #B6CEC3 / #D0DDD6 | textos del panel de configuración inicial |
| `ROJO` | #B42318 | errores, saldo pendiente, acción peligrosa |
| `ROJO_INTENSO` | #D92D20 | botón peligro, ícono de modal peligroso |
| `ROJO_PESTANA` | #DC2626 | X de pestaña en hover (con `ROJO_12` de fondo) |
| `ROJO_ERROR_BORDE` | #F04438 | borde de campo con error |
| `ROJO_FONDO` / `ROJO_BORDE` | #FEF3F2 / #FECDCA | caja de error, hover de quitar |
| `ROJO_TOAST` | #F97066 | ícono del toast de error |
| `AZUL` / `AZUL_FONDO` | #1D4E89 / #E6EEF9 | Factura A, Tarjeta |
| `VIOLETA` / `VIOLETA_FONDO` | #5B3E96 / #EFEAF8 | Transferencia |
| `GRIS_AVATAR` | #3B4A43 | quinto color de avatares |
| `AVATARES` | [VERDE, VERDE_PROFUNDO, DORADO_AVATAR, VERDE_HOVER_OSCURO, GRIS_AVATAR] | `AV` del script, por índice de vendedor |
| `OVERLAY_MODAL` | rgba(10,24,18,.45) | velo de los modales |
| `ANILLO_FOCO` | rgba(31,84,68,.15) | anillo de foco de 3px |
| `ANILLO_BUSQUEDA` | rgba(211,152,1,.35) | anillo dorado de la búsqueda con texto |
| `BLANCO_10/12/18/20/25` | blanco con 10–25 % | teclas y badges sobre verde |

### Tipografía

- Inter: `marca::fuente(px, Peso)`, `marca::texto(..)` (RichText). `Peso::{Normal, Medio, Semi, Negrita}` = 400/500/600/700.
- Geist Mono: `marca::fuente_mono(px, Peso)`, `marca::texto_mono(..)`. Pesos 400/500/600 (`Negrita` usa 600). También es la `FontFamily::Monospace`.
- Espaciado de letras (`letter-spacing: -.02em`): `marca::em(px, -0.02)` o `v2::titulo(ui, texto, px, -0.02, color)`.
- Tamaños (`marca::tam`): `DIMINUTO` 11, `MINI` 11.5, `XS` 12, `CHICO` 12.5, `ETIQUETA` 13, `CONTROL` 13.5, `CUERPO` 14, `CUERPO_MAS` 14.5, `ENTRADA` 15, `BUSQUEDA` 15.5, `DESTACADO` 16, `FILA` 17, `TITULO_MODAL` 19, `GRANDE` 20, `TITULO_SECCION` 22, `MONTO_CHICO` 24, `KPI` 26, `MONTO_MEDIO` 32, `MONTO` 34, `TITULO_PANTALLA` 40, `TOTAL` 46. Otros valores del HTML (28, 30, 36, 42, 44, 48) van como número literal.
- egui no tiene `font-variant-numeric: tabular-nums`: los números de Inter son proporcionales. Si una columna de montos «baila», alineala a la derecha (ya lo hace `Columna::derecha()`).

### Radios (`marca::radio`)

`TECLA` 5 · `CHIP` 6 · `TECLA_CUADRADA` 7 · `ICONO` 8 · `CHIP_ICONO` 9 · `CONTROL` 10 · `BOTON_MODAL` 11 · `CAMPO` 12 · `POPOVER` 14 · `TARJETA` 16 · `PANEL` 18 · `MODAL` 20 · `AVATAR_GRANDE` 24 · `PASTILLA` 99.

### Altos (`marca::alto`) y espacios (`marca::espacio::E2..E40`)

`CAMPO` 46 · `CAMPO_GRANDE`/`BOTON_GRANDE` 48 · `BOTON` 44 · `BOTON_MODAL` 42 · `CONTROL` 40 · `BOTON_CHICO` 32 · `ENCABEZADO` 68 · `ENCABEZADO_TABLA` 42 · `FILA_TABLA` 62.

### Sombras (`marca::sombra`, tipo `Sombra`)

egui no admite spread negativo en `Shadow`; `Sombra::forma(rect, radio)` sí (achica el rectángulo antes de difuminar). Se pintan antes del fondo: `pintor.add(sombra::MODAL.forma(rect, 20))`.

| Token | CSS |
|---|---|
| `BOTON` | 0 8px 18px -10px rgba(10,58,43,.7) |
| `BOTON_COBRAR` | 0 10px 20px -12px rgba(10,58,43,.8) (botón F2) |
| `POPOVER` | 0 24px 48px -16px rgba(10,58,43,.35) (resultados, menú de usuario) |
| `DESPLEGABLE` | 0 20px 40px -12px rgba(10,58,43,.3) |
| `AUTOCOMPLETAR` | 0 18px 36px -12px rgba(10,58,43,.3) (clientes en factura) |
| `MODAL` | 0 30px 60px -20px rgba(10,58,43,.5) |
| `TOAST` | 0 16px 32px -12px rgba(0,0,0,.45) |
| `AVATAR` | 0 14px 28px -16px rgba(10,58,43,.6) |
| `SEGMENTO` | 0 1px 2px rgba(16,24,40,.1) |
| `ICONO_EXITO` | 0 12px 24px -8px rgba(10,58,43,.6) (check del QR) |

Anillos: `marca::pintar_anillo(pintor, rect, radio, color)` pinta el `box-shadow: 0 0 0 3px` por fuera.

Mezcla de colores (transiciones de hover): `marca::mezclar(desde, hasta, t)`.

## Componentes (`crate::ui::v2`)

Todos devuelven `egui::Response` salvo que se diga otra cosa. Importar con `use crate::ui::v2::{self, ..};`.

### Texto y utilidades (`v2::*`)
- `texto(ui, contenido, px, peso, color)`, `titulo(ui, contenido, px, espaciado_em, color)` (600 con letter-spacing), `parrafo(ui, contenido, px, color, interlineado, ancho)` (con `line-height`), `etiqueta(ui, contenido)` (13.5/500 #344054).
- `pintar_texto(ui, pos, Align2, contenido, px, peso, color) -> Rect`, `galeria(..)`, `galeria_espaciada(..)`, `ancho_texto(..)`.
- `separador(ui)` (1px #EDF2EF), `separador_vertical(ui, alto, color)`.
- `logo(ui, tam)` (andex-market-logo.png, 26/32), `logo_texto(ui, ancho)` (andex-market-text.png, 112/150).
- `pintar_icono(ui, centro, nombre, tam, color)`.

### Campo de texto (`campo.rs`)
- `CampoTexto::new(&mut String)` → `.ui(ui) -> RespuestaCampo { respuesta, accion }`. Por defecto 46px, radio 12, borde #DCE4DF, relleno 0 14, texto 15px, gap 10, foco con borde #1F5444 + anillo 3px rgba(31,84,68,.15).
  - `.icono("mail")` (18px #7C8B84), `.sugerencia(..)`, `.contrasena(bool)`, `.accion("eye")` (botón 34×34 a la derecha, relleno derecho 6; `accion` es true al clic), `.prefijo("$")`, `.sufijo("g")`, `.error(bool)` (borde #F04438), `.multilinea(alto_min)` (textarea), `.alto(..)`, `.ancho(..)`, `.radio(..)`, `.tam_texto(..)`, `.peso(..)`, `.centrado()`, `.id(Id)`, `.habilitado(bool)`, `.caracteres_maximos(n)`, `.relleno(izq, der)`.
  - Búsqueda del encabezado: `.sin_borde().anillo((!q.is_empty()).then_some(color::ANILLO_BUSQUEDA)).icono("barcode").tam_texto(tam::BUSQUEDA).accion("x")`.
- Variante «bare»: `Contenedor { alto, radio, fondo, borde, error, anillo, relleno_izquierdo, relleno_derecho, separacion, multilinea }.mostrar(ui, ancho, |ui| { ...; entrada_desnuda(&mut s).sugerencia(..).ui(ui) })`. El contenedor pinta borde y anillo según el foco de la `Response` que devuelve el closure; adentro va cualquier cosa (íconos, botones, sufijos, precio elegido).
- `entrada_desnuda(&mut String)`: `TextEdit` sin marco (`.sugerencia`, `.contrasena`, `.tam`, `.peso`, `.id`, `.centrada`, `.multilinea`, `.color`, `.caracteres_maximos`, `.habilitado`).

### Botones (`boton.rs`)
- `Boton::new(texto, Estilo)`, atajos `Boton::primario` (con `sombra::BOTON`), `Boton::secundario`, `Boton::peligro`.
- `Estilo`: `Primario` (#1F5444 → hover #0A3A2B, blanco 600 14.5), `Secundario` (blanco, borde #DCE4DF, 500 14; hover borde y texto #1F5444), `Peligro` (#D92D20), `PeligroContorno` (borde #FECDCA, texto #B42318, ícono #D92D20, hover #FEF3F2), `Dorado` (botón F5), `ContornoMarca` (borde y texto #1F5444, «Agregar»), `Fantasma`, `FantasmaOscuro` (botones del encabezado).
- Por defecto: alto 44, radio 12, relleno 18 (fuertes) o 16, ícono 17, gap 8. Modificadores: `.icono(n)`, `.color_icono(c)` (ej. dorado en «Agregar pago»), `.tecla("Enter")` (keycap a la derecha con el estilo que corresponde al botón), `.alto`, `.ancho`, `.lleno(ui)`, `.relleno`, `.radio`, `.tam_texto`, `.peso`, `.tam_icono`, `.separacion`, `.sombra(..)`/`.sin_sombra()`, `.habilitado(bool)` (opacidad .45), `.cargando(bool)` (loader girando, sin clic), `.activo(bool)` (fuerza el estado hover, ej. menú abierto), `.repartido()` (ícono+texto a la izquierda y tecla pegada a la derecha, como las acciones del panel lateral), `.modal()` (42px, radio 11, sin sombra: botones de pie de modal).
- `BotonIcono::new("history")`: 32×32, radio 8, ícono 17 #667085, hover #F6F8F6/#1F5444. `.tam`, `.tam_icono`, `.radio`, `.colores(color, fondo_hover, color_hover)`, `.habilitado`, `.cargando`. Presets: `BotonIcono::cerrar()` (X 20px del modal), `BotonIcono::quitar()` (papelera #98A2B3 → #FEF3F2/#B42318).

### Teclas, chips, estados y avatares (`insignias.rs`)
- `tecla(ui, "F7", &EstiloTecla::CLARA)`; `medir_tecla` y `pintar_tecla(ui, rect, texto, estilo, color_heredado)` para pintarla dentro de otra cosa. Estilos: `CLARA` (Geist Mono 12/600, borde inferior 2px, fondo #FBFCFB), `EN_LINEA` («Presioná Enter»), `CONTORNO` (F4 de cerrar sesión), `EN_BOTON` (dentro de primario/peligro), `DORADA`, `SOBRE_OSCURO` (F2 del botón grande), `CUADRADA` (24×24 de métodos de pago y cierre), `CUADRADA_OSCURA` (opción oscura del cierre), `NUMERO_AVATAR` (26×26 sobre el avatar del vendedor).
- `chip(ui, texto, Tono)` (11.5/600, radio 6: tipo de comprobante, lista de precios, ADMIN), `chip_estado` (12/600: Pagada/Parcial/Pendiente), `chip_metodo(ui, icono, texto, Tono)` (pastilla con ícono 13), `contador(ui, texto, fondo, color)` (cantidad en la pestaña de orden).
- `Tono`: `Marca`, `Exito`, `Aviso`, `Peligro`, `Azul`, `Violeta`, `Admin`; `tono.colores() -> (fondo, texto)`.
- `punto_estado(ui, texto, color_punto, color_texto, px, peso, latiendo)` (En línea, Balanza, «Leyendo balanza…» con `pulse`).
- `Avatar::new(iniciales)` (32×32 radio 9, dorado/verde profundo, 13/700), `Avatar::producto(..)` (36×36 radio 10, resultados de búsqueda), `Avatar::vendedor(iniciales, color::AVATARES[i])` (148×148 radio 24, 48px, con sombra). `.lado`, `.radio`, `.colores`, `.texto(px, peso)`, `.ui(ui)` o `.pintar(ui, rect)`. `iniciales("Lucía Fernández") == "LF"`.

### Segmentados y opciones (`segmentado.rs`)
- `segmentado(ui, &[Segmento::texto("80 mm"), Segmento::con_icono("Porcentaje", "percent")], &mut indice, ancho: Option<f32>, EstiloSegmentado::Claro)`. `Claro`: caja #F6F8F6 con borde, relleno 3, ítems de 36 radio 8, activo blanco #0A3A2B 600 con sombra. `Filtro`: caja blanca radio 10, activo #1F5444 blanco (Todas / Con saldo / Pagadas). Con `ancho = Some(..)` los ítems se reparten parejo (flex:1).
- `colores_eleccion(activo) -> ColoresEleccion { borde, fondo, texto }` = `choice()` del script.
- `opcion_eleccion(ui, tam, radio, activo, grosor_borde, |ui, rect, colores| ..)`: tarjeta elegible con borde 1.5px y hover #1F5444 (métodos de pago 124px, cierre 128px, listas de precios, presets, billetes, modos de balanza). El closure pinta el contenido.

### Popovers, desplegable y menú de usuario (`popover.rs`)
- `Popover::new(id, &respuesta_ancla)` → `.mostrar(ctx, |ui| ..) -> Option<R>`. Fondo blanco, borde, radio 12, sombra `DESPLEGABLE`, relleno 6, animación popIn (opacidad + escala .96→1, 140ms). Modificadores: `.ancho`, `.separacion` (6), `.a_la_derecha()` (alineado al borde derecho del ancla), `.encima()` (se abre arriba del ancla, como los productos del formulario de factura), `.radio`, `.sombra`, `.alto_maximo` (con scroll), `.visible(bool)` (abierto por estado propio, ej. resultados de búsqueda mientras hay texto; sin esto se abre/cierra con `Popover::alternar(ctx, id)` y se cierra solo con clic afuera o Esc), `.duracion`.
- Resultados del buscador: `Popover::new(id, &resp_campo).visible(hay_resultados).ancho(ancho_campo).separacion(8).radio(radio::POPOVER).sombra(sombra::POPOVER).alto_maximo(420.0)`.
- `desplegable(ui, id, &mut indice, &[String], sugerencia, ancho, alto)`: el `<select>` del diseño (borde #DCE4DF, abierto #1F5444, hover #B6C4BD, radio 10, 13.5px, chevron 16; opciones 32px radio 8, elegida #F1F7F4 600 con check, máx 260 con scroll). `changed()` al elegir.
- `item_menu(ui, &ItemMenu { icono, etiqueta, tecla, peligro })`.
- `menu_usuario(ui, id, &Usuario { nombre, email, detalle }, &[ItemMenu], compacto) -> Option<usize>`: botón del encabezado (avatar dorado, nombre, reloj, chevron; hover/abierto #134634) y menú de 250px alineado a la derecha. `compacto` = sin nombre (pantallas < 1240px, `wideHeader` del script). Devuelve el índice elegido.

### Tarjetas, tabla y carga (`tarjetas.rs`)
- `tarjeta_kpi(ui, &Kpi { etiqueta, valor, detalle, icono, fondo_icono, color_icono, color_valor }, ancho)`.
- `dato_compacto(ui, etiqueta, valor, ancho)` (estadísticas del modal de órdenes).
- Tabla: `marco_tabla()` (blanco, borde, radio 14) + `Tabla::new(&columnas)` con `Columna::fraccion(fr, minimo)` / `Columna::fija(px)` y `.derecha()`, igual que `grid-template-columns`. `.relleno(18)`, `.separacion(12)`, `.radio_superior(13)`. `tabla.encabezado(ui, &titulos, 42.0)` (#FBFCFB, 12.5/500 #667085) y `tabla.fila(ui, alto, fondo_opcional, |ui, celdas: &[Rect]| ..)` (borde inferior #EDF2EF, hover #F8FAF9). Dentro de la fila: `tarjetas::celda_texto(pintor, celda, texto, px, peso, color, derecha)` (trunca con «…») o `ui.new_child(UiBuilder::new().max_rect(celdas[i]))` para widgets.
- `shimmer(ui, tam, radio, animado)` y `tarjetas::pintar_shimmer(ui, rect, radio, animado)`: esqueleto #EDF2EF con el brillo que se desplaza (1.2s).

### Modal (`modal.rs`)
```rust
let r = v2::Modal::new("cobro", "Cobrar con factura AFIP")
    .subtitulo("Se emite Factura B. Presioná 1 a 4 para elegir.")
    .icono("wallet")
    .ancho(520.0)
    .mostrar(ctx, |ui| {
        v2::cuerpo_modal(ui, 20, |ui| { /* padding 20 24 0 */ });
        v2::pie_modal(ui, |ui| {
            v2::Boton::primario("Confirmar cobro").tecla("Enter").modal().sin_sombra().ui(ui);
            v2::Boton::secundario("Cancelar").modal().ui(ui);
        });
    });
if r.cerrar { /* misma acción que closeModal */ }
```
- Velo rgba(10,24,18,.45) con fadeIn 160ms, tarjeta blanca radio 20 con sombra `MODAL` y popIn 200ms. Encabezado: caja de ícono 42 radio 12 (`.tono_icono(fondo, color)`, atajos `.peligro()`, `.aviso()`, `.exito()`), título 19/600, subtítulo 13.5 #667085 con interlineado 1.45, X a la derecha (`.sin_cerrar()` la saca y desactiva Esc/clic afuera).
- `cerrar` es true con la X, Esc o clic en el velo.
- `pie_modal` acomoda de derecha a izquierda con gap 10: **agregá primero el botón principal**. Para pies centrados o con `space-between`, armá la fila a mano con `cuerpo_modal`.

### Toast (`toast.rs`)
`v2::toast::exito(ctx, "Pago registrado")`, `error`, `info`, `aviso`, `con_detalle(ctx, Tipo::Exito, titulo, detalle)`; `v2::toast::pintar(ctx)` una vez por cuadro, al final. Fondo #101828, radio 12, ícono 19 (`circle-check` #6FC79F, `alert` #F97066, `info` #D39801), texto 14/500 blanco, detalle 12.5 #98A2B3, entra con toastIn (260ms).

### Animaciones (`animacion.rs`)
- `girando(ui, rect, color)` / `cargando(ui, tam, color)`: ícono `loader` girando (900ms por vuelta).
- `pulso(ctx, periodo)`: opacidad 1 → .35 → 1 (`pulse`).
- `aparicion(ctx, id, visible, duracion)`: 0→1 al hacerse visible (para transiciones de estado).
- `apertura(ctx, id, duracion)`: 0→1 desde el primer cuadro en que se dibuja algo (lo usan modal y toast).
- `escalar_capa(ctx, capa, origen, escala)` + `escala_pop(avance)`: el `scale(.96)` de popIn.
- No portadas: `style-active` (scale .97/.98 al presionar), `rowIn` (destello dorado de la fila recién agregada: se puede hacer con `aparicion` mezclando `DORADO_FONDO` → transparente en 1.6s y la barra izquierda de 3px #D39801) y `translateY(-3px)` del hover de vendedor.

## Íconos Solar Bold Duotone

Archivos en `assets/iconos-solar/*.svg` (extraídos de `icons-solar.js`, `width/height` fijados en 24). Se registran como `bytes://solar/<nombre>.svg` con `currentColor` → blanco y se tiñen con `.tint(color)`; la capa secundaria conserva su `opacity=".5"`, así que el duotono sale del mismo color. API: `icono_solar(nombre, tam, color) -> Image`, `pintar_icono_solar(ui, nombre, tam, color)`, `pintar_icono_solar_en(ui, rect, nombre, color)`, `nombres_iconos_solar()`.

Nombres (los 47 que usa el POS): `alert`, `arrow-left`, `barcode`, `bill-check`, `calendar`, `card`, `cart`, `cart-plus`, `cash`, `check`, `chevron-down`, `chevron-left`, `chevron-right`, `circle-check`, `clock-time`, `doc`, `dollar-sign`, `download`, `eye`, `file-text`, `hand-coins`, `history`, `info`, `loader`, `lock`, `log-out`, `mail`, `moon`, `percent`, `plus`, `printer`, `qr`, `receipt`, `scale`, `search`, `settings`, `shield`, `split`, `square-pen`, `store`, `sun`, `tag`, `transfer`, `trash-2`, `user`, `wallet`, `x`.

Equivalencias con los Lucide del kit v1 (ya borrados), por si aparece un nombre viejo: `shopping-cart`→`cart`, `loader-2`→`loader`, `alert-circle`/`alert-triangle`→`alert`, `check-circle`→`circle-check`, `credit-card`→`card`, `qr-code`→`qr`, `edit`→`square-pen`, `clock`→`clock-time`, `xcircle`/`x`→`x`. Ojo: en Solar `check` es un cuadrado con tilde y `x` es un círculo con cruz (así está en el diseño).

Si una pantalla del diseño de Admin/Reportes necesita otro ícono del set, agregalo con el script de extracción (mismo reemplazo de `width="100%" height="100%"`) y sumalo a `ICONOS_SOLAR` en `ui/iconos.rs`.

## Fuentes e imágenes
- `assets/fuentes/GeistMono-{Regular,Medium,SemiBold}.ttf` (Geist v1.7.2, release oficial de vercel/geist-font) con `LICENSE-GeistMono.txt` (OFL 1.1). Familias: `Monospace` (400), `geist-mono-500`, `geist-mono-600`.
- `assets/imagenes/andex-market-logo.png` (29×30) y `andex-market-text.png` (230×28, texto blanco para fondo verde), registradas como `imagen("andex-market-logo.png")` / `imagen("andex-market-text.png")`.
