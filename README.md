# VerduSoft POS - Guía de Instalación

Esta guía te ayudará a instalar y configurar todos los componentes necesarios para ejecutar el sistema de punto de venta.

## Requisitos Previos

El instalador trae todo lo necesario: PHP 8.3 con sus extensiones (`gd`, `mbstring`, `intl`) en `resources/php`, las librerías de impresión y el driver CH340 para balanzas con adaptador USB-serie. No hace falta instalar PHP, Composer ni tocar el PATH.

Para actualizar el PHP incluido: bajar el zip NTS x64 de windows.php.net y reemplazar en `resources/php` los archivos `php.exe`, `php8.dll`, `icu*.dll`, `ext/php_gd.dll`, `ext/php_mbstring.dll` y `ext/php_intl.dll`.

Otros drivers USB-serie (FTDI, PL2303) los instala Windows Update. Para incluirlos en el instalador, copiar la carpeta con el `.inf` firmado dentro de `resources/drivers/`: el instalador carga con `pnputil` todos los `.inf` que encuentre ahí.

### 1. Impresora Térmica

1. Instala los drivers de tu impresora térmica (dependen de la marca).
2. En el POS, menú de usuario → Impresión, elegí la impresora. El programa la comparte sola en Windows.
   - Si no se elige ninguna, imprime en la impresora compartida como `TP806L`, como en las versiones anteriores.

### 2. Balanza Electrónica (si aplica)

1. Conecta la balanza a un puerto COM de la computadora
2. Anota el número de puerto COM asignado (lo necesitarás más adelante)
   - Puedes verificarlo en el Administrador de dispositivos → Puertos (COM y LPT)

## Instalación del Programa

1. Ejecuta el instalador `Verdulería Setup 1.0.0.exe`
2. Sigue el asistente de instalación
3. Al finalizar, se creará un acceso directo en el escritorio

## Configuración Inicial

1. La primera vez que ejecutes el programa, te pedirá:
   - Configurar la impresora térmica
   - El puerto COM de la balanza (si aplica)
   - Las credenciales de acceso proporcionadas por el administrador

## Solución de Problemas Comunes

### Error de Impresión

- Verifica que la impresora esté encendida y conectada
- Comprueba que sea la impresora predeterminada
- Reinicia la impresora y el programa

### Error de Balanza

- Verifica que la balanza esté encendida y conectada
- Comprueba que el puerto COM configurado sea el correcto
- Reinicia la balanza y el programa

### Aviso "La impresión de tickets no va a funcionar" al abrir

- El PHP incluido no pudo arrancar. Reinstalá AndexMarket.

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
