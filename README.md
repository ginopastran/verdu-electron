# VerduSoft POS - Guía de Instalación

Esta guía te ayudará a instalar y configurar todos los componentes necesarios para ejecutar el sistema de punto de venta.

## Requisitos Previos

### 1. PHP y Composer

1. Descarga PHP desde [windows.php.net/download](https://windows.php.net/download/)
   - Selecciona la versión "VS16 x64 Thread Safe"
   - Descomprime el archivo en `C:\php`
2. Agrega PHP al PATH del sistema:
   - Abre el Panel de Control → Sistema → Configuración avanzada del sistema
   - Click en "Variables de entorno"
   - En "Variables del sistema", selecciona "Path" y click en "Editar"
   - Click en "Nuevo" y agrega `C:\php`
3. Descarga Composer desde [getcomposer.org](https://getcomposer.org/download/)
   - Ejecuta el instalador y sigue las instrucciones

### 2. Configuración de PHP

1. En la carpeta `C:\php`, copia el archivo `php.ini-development` y renómbralo a `php.ini`
2. Abre `php.ini` y habilita las siguientes extensiones (quita el ; del inicio de la línea):
   ```ini
   extension=fileinfo
   extension=gd
   extension=mbstring
   extension=openssl
   extension=pdo_mysql
   extension=sockets
   ```

### 3. Impresora Térmica

1. Instala los drivers de tu impresora térmica
   - Para impresoras Epson: Descarga "EPSON Advanced Printer Driver" desde la web oficial
   - Para otras marcas: Sigue las instrucciones del fabricante
2. Configura la impresora como predeterminada en Windows

### 4. Balanza Electrónica (si aplica)

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

### Error de "PHP no encontrado"

- Verifica que PHP esté correctamente instalado y en el PATH
- Reinicia la computadora después de instalar PHP
- Abre una terminal y ejecuta `php -v` para verificar la instalación

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
