# AndexMarket POS - Guía de Instalación

Esta guía te ayudará a instalar y configurar todos los componentes necesarios para ejecutar el sistema de punto de venta.

## Requisitos Previos

Desde la versión 4 el POS es una app Tauri: la interfaz React corre en el WebView2 de Windows y todo el lado de hardware (impresión ESC/POS, balanza, configuración y actualizaciones) está en Rust. El instalador pesa unos pocos MB y no trae Electron, Node ni PHP.

El instalador trae el driver CH340 para balanzas con adaptador USB-serie (en `src-tauri/drivers/`). Si no está cargado, al terminar la instalación Windows pide permiso de administrador una sola vez para instalarlo con `pnputil`. Otros drivers USB-serie (FTDI, PL2303) los instala Windows Update; para incluirlos, copiar la carpeta con el `.inf` firmado dentro de `src-tauri/drivers/`.

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
2. Sigue el asistente de instalación (se instala para el usuario actual, sin pedir administrador)
3. Al finalizar, se creará un acceso directo en el escritorio

Si la PC tenía la versión Electron (3.x), el instalador la desinstala y la app nueva toma su configuración (impresora, balanza, ancho de papel, logo del negocio) de `%APPDATA%\andex-market`. Lo que la versión vieja guardaba en el `localStorage` (credenciales recordadas y offline) no se migra: hay que iniciar sesión una vez.

## Desarrollo

Requisitos: Node 22, Rust (toolchain MSVC) y los Build Tools de C++ con el SDK de Windows.

```bash
npm install
npm run dev
```

- `npm test` corre los tests de Rust. Los tickets se comparan byte a byte contra lo que imprimía la versión PHP (`src-tauri/impresion/golden/`).
- `npm run release` arma el instalador firmado, genera `latest.json` (actualizador de Tauri) y `latest.yml` (para que las PCs con la versión Electron se actualicen a esta) y crea el release como borrador en GitHub. Ver `README-AUTOUPDATE.md`.

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
