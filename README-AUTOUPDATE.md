# Actualizaciones automáticas

El POS tiene su propio actualizador (crate `crates/actualizador`). Cada instalador va firmado con una clave privada (formato minisign, el mismo que usaba Tauri) y la app solo ejecuta instaladores cuya firma coincide con la clave pública que lleva compilada (`CLAVE_PUBLICA` en `crates/actualizador/src/lib.rs`, la misma que tenía `tauri.conf.json` en la v4).

## Cómo lo ve el usuario

- A los 5 segundos de abrir y después cada 10 minutos, la app consulta `latest.json` del último release publicado en `ginopastran/verdu-electron` (solo en el build de release; en `cargo run` no busca).
- Si hay una versión más nueva que la instalada, muestra el aviso "¡Nueva versión disponible!" y la tarjeta "Actualización disponible" abajo a la derecha, con **Más tarde** / **Descargar**.
- **Descargar** baja el instalador, verifica la firma y pasa a "Listo para instalar". Si la firma no coincide, no se guarda nada y aparece el error.
- **Reiniciar e instalar** vuelve a verificar la firma del archivo descargado, ejecuta el instalador en modo pasivo (`/P /R`: solo la barra de progreso, sin preguntas) y cierra la app. El instalador vuelve a abrir AndexMarket al terminar.
- **Más tarde** descarta el aviso; a los 10 minutos se vuelve a ofrecer.

## Clave de firma

- Privada: `D:\dev\claves\andex-market.key` (sin contraseña). Se puede usar otra ruta con la variable `ANDEX_CLAVE_ACTUALIZACIONES`.
- **Guardá una copia fuera de esta PC.** Si se pierde, las PCs instaladas no van a aceptar ninguna actualización nueva y habrá que reinstalar a mano en cada una.
- Para firmar se usa `cargo run -p actualizador --features firmar --bin firmar -- <clave> <archivo>`, que genera `<archivo>.sig` igual que `tauri signer sign` y se niega a firmar si la clave privada no corresponde a la clave pública de la app.

## Publicar una versión

1. Subir `version` en `package.json` **y** en `apps/andexmarket/Cargo.toml` (el script corta si no coinciden: la app compara su propia versión con la de `latest.json`).
2. `npm run release`. Compila en release, arma el instalador con NSIS (`empaquetado/instalador.nsi`), lo firma y deja en `<CARGO_TARGET_DIR>\release\instalador\`:
   - `AndexMarket_<versión>_x64-setup.exe` y su `.sig`
   - `latest.json`, que lo lee el actualizador de la app
   - `latest.yml`, que lo lee el `electron-updater` de las PCs que siguen con la versión Electron 3.x

   Después crea el release `v<versión>` como **borrador** con el instalador, `latest.json` y `latest.yml`.
3. Probar el instalador y recién entonces publicar el borrador desde GitHub.

`npm run release -- --local` hace todo menos subir el release.

NSIS: el script usa `MAKENSIS` si está definida; si no, el que bajó Tauri en `%LOCALAPPDATA%\tauri\NSIS\makensis.exe` o el de `Program Files\NSIS`.

Una vez publicada una versión egui, no publiques más releases de la rama Electron (3.x): GitHub toma el último release publicado como `latest`, las PCs nuevas dejarían de encontrar su `latest.json` y las que siguen en Electron bajarían esa 3.x.

## El instalador

`empaquetado/instalador.nsi` (NSIS, en español, sin pedir administrador):

- instala para el usuario actual en `%LOCALAPPDATA%\AndexMarket` (`AndexMarket.exe`, el driver CH340 en `drivers\ch340` y `Desinstalar AndexMarket.exe`);
- crea accesos directos en el escritorio y en el menú inicio y registra la desinstalación en "Aplicaciones instaladas";
- si encuentra la versión Electron 3.x, la cierra y la desinstala en silencio conservando sus datos;
- si el driver CH340 no está cargado, lo instala con `pnputil` (Windows pide administrador una sola vez);
- argumentos: `/S` silencioso, `/P` pasivo (solo progreso), `/R` reabre la app al terminar; `--updated` (con el que lo corre `electron-updater`) equivale a `/P /R`, y `--force-run` a `/R`.

El desinstalador borra el programa, los accesos directos y la entrada del registro, pero no los datos (`%APPDATA%\com.andextech.andexmarket`).

## Paso de Electron (3.x) a esta versión

Las PCs con la versión Electron ven el release nuevo por su `latest.yml`, descargan el instalador y lo ejecutan al aceptar. El instalador quita la versión vieja, instala la nueva y la abre. Al abrir por primera vez, la app copia `config.json` y `logo-negocio.png` desde `%APPDATA%\andex-market`.
