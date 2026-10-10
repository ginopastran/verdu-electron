# Actualizaciones automáticas

El POS usa el actualizador de Tauri. Cada instalador va firmado con una clave privada y la app solo acepta instaladores firmados con esa clave (la pública está en `src-tauri/tauri.conf.json`).

## Cómo lo ve el usuario

- A los 3 segundos de abrir y después cada 10 minutos, la app consulta `latest.json` del último release publicado en `ginopastran/verdu-electron`.
- Si hay una versión nueva, pregunta "Descargar / Más tarde". Al aceptar baja el instalador, lo verifica con la firma y lo corre en modo pasivo: solo se ve la barra de progreso y la app se vuelve a abrir.
- La UI (`useAutoUpdater`) puede hacer lo mismo a mano con buscar → descargar.

## Clave de firma

- Privada: `D:\dev\claves\andex-market.key` (sin contraseña). Se puede usar otra ruta con la variable `ANDEX_CLAVE_ACTUALIZACIONES`.
- **Guardá una copia fuera de esta PC.** Si se pierde, las PCs instaladas no van a aceptar ninguna actualización nueva y habrá que reinstalar a mano en cada una.

## Publicar una versión

1. Subir `version` en `package.json`.
2. `npm run release`. Compila, firma y deja en `<CARGO_TARGET_DIR>\release\bundle\nsis\`:
   - `AndexMarket_<versión>_x64-setup.exe` y su `.sig`
   - `latest.json`, que lo lee el actualizador de Tauri
   - `latest.yml`, que lo lee el `electron-updater` de las PCs que siguen con la versión Electron 3.x
   
   Después crea el release `v<versión>` como **borrador** con esos tres archivos.
3. Probar el instalador y recién entonces publicar el borrador desde GitHub.

`npm run release -- --local` hace todo menos subir el release.

Una vez publicada una 4.x, no publiques más releases de la rama Electron (3.x): GitHub toma el último release publicado como `latest`, las PCs con Tauri dejarían de encontrar su `latest.json` y las que siguen en Electron bajarían esa 3.x en lugar de pasar a Tauri.

## Paso de Electron (3.x) a Tauri (4.x)

Las PCs con la versión Electron ven el release 4.x por su `latest.yml`, descargan el instalador nuevo y lo ejecutan al aceptar. El instalador (`src-tauri/instalador.nsh`):

1. cierra la app vieja y la desinstala en silencio, conservando sus datos;
2. instala la versión nueva para el usuario actual;
3. carga el driver CH340 si todavía no está.

Al abrir por primera vez, la app copia `config.json` y `logo-negocio.png` desde `%APPDATA%\andex-market`.
