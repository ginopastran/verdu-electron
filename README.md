# AndexMarket POS - Guía de Instalación

Esta guía te ayudará a instalar y configurar todos los componentes necesarios para ejecutar el sistema de punto de venta.

## Requisitos Previos

El POS es una app 100% Rust: la interfaz está hecha con egui y el hardware (impresión ESC/POS, balanza, configuración y actualizaciones) también es Rust. Es un solo `AndexMarket.exe`; el instalador pesa pocos MB y no trae Electron, Node, WebView2 ni PHP.

El instalador trae el driver CH340 para balanzas con adaptador USB-serie (en `drivers/ch340/`). Si no está cargado, al terminar la instalación Windows pide permiso de administrador una sola vez para instalarlo con `pnputil`. Otros drivers USB-serie (FTDI, PL2303) los instala Windows Update; para incluirlos hay que copiar la carpeta con el `.inf` firmado dentro de `drivers/` y agregarla en `empaquetado/instalador.nsi`.

### 1. Impresora Térmica

1. Instala los drivers de tu impresora térmica (dependen de la marca).
2. En el POS, menú de usuario → Impresión, elegí la impresora. Se imprime directo en la cola de Windows, sin compartirla en red.
   - Si no se elige ninguna, usa la impresora compartida como `TP806L` (o la que se llame así), como en las versiones anteriores.

### 2. Balanza Electrónica (si aplica)

1. Conecta la balanza a un puerto COM de la computadora
2. Anota el número de puerto COM asignado (lo necesitarás más adelante)
   - Puedes verificarlo en el Administrador de dispositivos → Puertos (COM y LPT)

## Instalación del Programa

1. Ejecuta el instalador `AndexMarket_<versión>_x64-setup.exe`
2. Sigue el asistente de instalación (se instala para el usuario actual en `%LOCALAPPDATA%\AndexMarket`, sin pedir administrador)
3. Al finalizar, se crean accesos directos en el escritorio y en el menú inicio

Si la PC tenía la versión Electron (3.x), el instalador la desinstala y la app nueva toma su configuración (impresora, balanza, ancho de papel, logo del negocio) de `%APPDATA%\andex-market`. Lo que la versión vieja guardaba en el `localStorage` (credenciales recordadas y offline) no se migra: hay que iniciar sesión una vez.

## Desarrollo

Requisitos: Rust (toolchain MSVC), los Build Tools de C++ con el SDK de Windows, NSIS 3 (sirve el que bajó Tauri en `%LOCALAPPDATA%\tauri\NSIS`) y Node 22 para los scripts de npm.

```bash
npm run dev
```

- `npm run dev` es `cargo run -p andexmarket`; `npm run build` compila `andexmarket.exe` en release.
- `npm test` corre todos los tests de Rust (`cargo test --workspace`). Los tickets se comparan byte a byte contra lo que imprimía la versión PHP.
- `npm run release` compila, arma el instalador NSIS (`empaquetado/instalador.nsi`), lo firma, genera `latest.json` (actualizador de la app) y `latest.yml` (para que las PCs con la versión Electron se actualicen a esta) y crea el release como borrador en GitHub. `npm run release -- --local` hace lo mismo sin subir nada. Ver `README-AUTOUPDATE.md`.

## Solución de Problemas Comunes

### Error de Impresión

- Verifica que la impresora esté encendida y conectada
- Comprueba en menú de usuario → Impresión que esté elegida la impresora correcta
- Reinicia la impresora y el programa

### Error de Balanza

- Verifica que la balanza esté encendida y conectada
- Comprueba que el puerto COM configurado sea el correcto
- Reinicia la balanza y el programa

## Soporte

Si encuentras algún problema durante la instalación o uso del programa:

1. Contacta al soporte técnico al [número de teléfono]
2. Envía un correo a [correo de soporte]
3. Visita [sitio web de soporte] para más información

## Notas Importantes

- El programa requiere conexión a internet para funcionar en modo online
- Se recomienda tener Windows 10 o superior
- Asegúrate de tener todos los drivers actualizados
- Realiza copias de seguridad periódicas de tus datos
