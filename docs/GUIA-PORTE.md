# Guía para portar la UI React a egui

Objetivo: el POS AndexMarket pasa a ser 100% Rust (egui/eframe, como las apps craft de storytold) y tiene que **verse y comportarse igual** que la UI React de `src/` (Vite + Tailwind + shadcn). La UI React sigue en `src/` solo como referencia: no se toca y se borra al final.

## Reglas
- **Sin comentarios en el código** (regla del usuario: ni `//`, ni `///`, ni bloques). Los nombres explican. Lo que haya que explicar va en el reporte final.
- Nombres en español, como el resto del workspace.
- Cada área trabaja en **su propio worktree y rama** (ver tu prompt) y solo toca **sus archivos**. Archivos compartidos que NO se editan: `apps/andexmarket/src/ui/**`, `app.rs`, `tareas.rs`, `impresion.rs`, `lib.rs`, `main.rs`, `pantallas/mod.rs`, `crates/nucleo/src/{api,carrito,iva,entorno,lib}.rs`, `crates/hardware/**`, `crates/impresion/**`, `Cargo.toml` raíz. Si necesitás algo nuevo de uso general (un widget, un helper), creálo dentro de tu módulo y mencionalo en el reporte para promoverlo después. Si de verdad necesitás cambiar un archivo compartido, hacé el cambio mínimo y aditivo y reportalo.
- Las interfaces públicas de los stubs que te tocan (`abrir`, `abierto`, `mostrar`, `iniciar`, `ocupado`, enums de resultado) **se respetan** porque otras áreas las llaman. Podés agregar métodos, no cambiar las firmas existentes.
- Paridad de comportamiento: misma lógica que los hooks/TSX (validaciones, mensajes, toasts, llamadas a la API con los mismos endpoints, cuerpos y headers, atajos de teclado, foco). Si el TSX tiene un bug evidente, replicalo y mencionalo en el reporte.
- Nunca bloquear el hilo de la UI: toda llamada HTTP, impresión o I/O lenta va en `Tarea::lanzar` (ver abajo).

## Arquitectura
- `app::Contexto { global, servicios, egui }` llega a cada pantalla en `mostrar(ui, cx)`.
  - `cx.global`: ruta (`cx.global.ir(Ruta::X)`), `admin` (adminData), `negocio` (businessInfo, `serde_json::Value`), `usuario` (user), `carritos` (`nucleo::carrito::Carritos`, equivalente a `useCartState`), `peso` (último peso de la balanza, `Value`), `estado_balanza`, `en_linea`.
  - `cx.servicios` (Clone): `config` (lo que antes era `electronStore`: `get/set/has/delete` con las mismas claves), `balanza` (`hardware::balanza::Balanza`), `api` (`nucleo::api::Cliente`), `carpeta` (carpeta de datos: lo que antes iba a `localStorage` va a `config` o a un archivo en esta carpeta).
- HTTP: `nucleo::api::Cliente`. `pedir(metodo, ruta, cuerpo, encabezados)` devuelve `Respuesta { estado, cuerpo }` y no falla con códigos de error (para imitar `fetch` + `response.ok`). `get/post/put/delete` imitan `services/api.ts`. Las cookies de sesión se guardan solas (equivalente a `credentials: "include"`). Variables `VITE_*` en `nucleo::entorno`.
- Tareas: `crate::tareas::Tarea::lanzar(&cx.egui, move || { ... })` corre en otro hilo y repinta al terminar; en cada frame `tareas::esperar(&mut self.tarea)` devuelve el resultado cuando está listo. Cloná `cx.servicios` para moverlo al hilo.
- Impresión: `crate::impresion::imprimir(&servicios, Comprobante::{Venta,Factura,Afip,Cierre}, datos_json)` devuelve el mismo JSON que devolvía Electron (`{success, printerError, message}`). Va dentro de una `Tarea`.
- Toasts (sonner): `crate::ui::toast::{exito, error, info, aviso, con_detalle}(&cx.egui, "...")`.

## Kit visual (`crate::ui`)
- Tailwind → egui: 1px CSS = 1 punto. `ui::s(n)` = n·4 (spacing). Textos: `ui::texto(contenido, Tam::Sm, Peso::Medio, color)` (`Tam::{Xs,Sm,Base,Lg,Xl,X2..X6,Px(f32)}`, `Peso::{Normal,Medio,Semi,Negrita}` = 400/500/600/700, fuente Inter). Colores: `ui::paleta::EMERALD_500` etc. (paleta completa de Tailwind) y `ui::color::*` (variables de shadcn: FONDO, TEXTO, PRIMARIO, APAGADO_TEXTO, BORDE, BORDE_ENTRADA #A7A7A7, ESMERALDA_DESDE/HASTA del `bg-emerald-gradient`). Radios: `RADIO`=8 (rounded-lg), `RADIO_MD`=6, `RADIO_SM`=4, `RADIO_XL`=12. Sombras: `sombra_sm/md/lg/xl`.
- Componentes: `Boton` (variantes de shadcn + `Esmeralda` = bg-emerald-gradient, `Cancelar` = bg-cancel-gradient; tamaños; icono; cargando), `Entrada` (Input rounded-xl, también multilínea/contraseña), `casilla`, `interruptor`, `insignia`, `pestanas`, `selector` (Select), `menu`/`item_menu` (DropdownMenu), `Dialogo` (Dialog/AlertDialog con overlay negro 80%, X, Esc), `encabezado_dialogo`, `pie_dialogo`, `tarjeta()` (Card), `fila_tabla`/`celda_encabezado`, `alerta`, `cargando` (Loader2 girando), `separador`, `pintar_degradado`, `icono(nombre, tam, color)` / `pintar_icono` (íconos Lucide en kebab-case: `shopping-cart`, `loader-2`, `trash-2`...; están en `assets/iconos`), `imagen("andexmarket-logo.png")` (imágenes de `assets/imagenes`, teclas en `teclas/F1.png`).
- El `item_spacing` global es 0: el espaciado se pone explícito con `ui.add_space(..)` o `ui.spacing_mut().item_spacing`, igual que `gap-*`/`space-*`. Para medidas exactas usá `allocate_exact_size` y pintá con `ui.painter()`.

## Verificación visual (obligatoria)
- Capturas de referencia de la UI React: `C:\Users\GINO\Desktop\verdu-rust\docs\referencia\*.png` (1200×800), con `INDICE.md` y los datos mock en `mock.json`. Puede que todavía se estén generando: volvé a mirar la carpeta más tarde.
- Tus capturas: un test `apps/andexmarket/tests/captura_<area>.rs` que use `mod comun;` y `comun::capturar(nombre, global, preparar)` o `comun::capturar_con(nombre, global, preparar, pasos)` (pasos sobre el `Harness` de egui_kittest: clicks, teclas). Guardan en `docs/capturas/<nombre>.png` del worktree. Usá el **mismo nombre** que la referencia para comparar.
- Mirá ambas imágenes con la herramienta Read y ajustá hasta que sean equivalentes (layout, tamaños, colores, tipografía, espaciados, íconos). Iterá todas las veces que haga falta.
- Correr: `source /c/Users/GINO/AppData/Local/Temp/claude/C--Users-GINO-Desktop-verdulerias-soft/e27b9b79-5b9a-43b8-892b-fb2a1d1820e3/scratchpad/rustenv.sh; export CARGO_TARGET_DIR='D:\dev\rust\target-<area>'; cargo test -p andexmarket --test captura_<area>` (desde la raíz de tu worktree). Usá siempre tu propio `CARGO_TARGET_DIR`.
- Además de las capturas, `cargo build -p andexmarket` sin warnings y tests unitarios para la lógica no trivial (cálculos, armado de payloads).

## Entrega
- Commits en tu rama (mensajes en español, terminando con `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`). No push, no merge.
- Reporte final corto: qué portaste, capturas comparadas (nombre referencia ↔ nombre propio), diferencias visuales que quedan y por qué, cambios fuera de tus archivos, bugs del original replicados.
