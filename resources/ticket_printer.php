<?php
$autoloaderPath = __DIR__ . '/vendor/autoload.php';
if (!file_exists($autoloaderPath)) {
    die("Error: No se encuentra el autoloader en: " . $autoloaderPath);
}

require $autoloaderPath;

// Verificar que la clase existe
if (!class_exists('Mike42\Escpos\PrintConnectors\WindowsPrintConnector')) {
    die("Error: No se encuentra la clase WindowsPrintConnector");
}

use Mike42\Escpos\Printer;
use Mike42\Escpos\EscposImage;
use Mike42\Escpos\PrintConnectors\WindowsPrintConnector;

try {
    // Agregar logs
    file_put_contents('php://stderr', "Iniciando proceso de impresión...\n");
    
    $orderDataPath = $argv[1];
    if (!file_exists($orderDataPath)) {
        throw new Exception("Archivo de datos no encontrado: " . $orderDataPath);
    }
    
    file_put_contents('php://stderr', "Leyendo datos de orden...\n");
    $orderData = json_decode(file_get_contents($orderDataPath), true);
    if (json_last_error() !== JSON_ERROR_NONE) {
        throw new Exception("Error al decodificar JSON: " . json_last_error_msg());
    }

    $nombre_impresora = "TP806L";
    file_put_contents('php://stderr', "Conectando a impresora: " . $nombre_impresora . "\n");
    
    try {
        $connector = new WindowsPrintConnector($nombre_impresora);
        file_put_contents('php://stderr', "Conexión exitosa\n");
    } catch (Exception $e) {
        throw new Exception("Error al conectar con la impresora: " . $e->getMessage());
    }

    $printer = new Printer($connector);
    file_put_contents('php://stderr', "Impresora inicializada\n");

    // Configuración inicial
    $printer->setJustification(Printer::JUSTIFY_CENTER);

    // Logo (opcional)
    try {
        file_put_contents('php://stderr', "==== DIAGNÓSTICO DEL LOGO ====\n");
        
        // Definir posibles ubicaciones del logo
        $possibleLogoPaths = [
            __DIR__ . "/logo.png",                   // Mismo directorio que el script
            __DIR__ . "/../public/logo.png",         // Directorio public (desarrollo)
            __DIR__ . "/../resources/logo.png",      // Directorio resources
            __DIR__ . "/../../public/logo.png",      // Un nivel más arriba
            __DIR__ . "/../../resources/logo.png",   // Un nivel más arriba en resources
        ];
        
        // Variable para almacenar la ruta válida
        $logoPath = null;
        
        // Imprimir directorio actual para diagnóstico
        file_put_contents('php://stderr', "Directorio actual: " . __DIR__ . "\n");
        file_put_contents('php://stderr', "NODE_ENV: " . getenv('NODE_ENV') . "\n");
        
        // Verificar si existen los directorios principales
        file_put_contents('php://stderr', "Verificando directorios principales:\n");
        $dirsToCheck = [
            __DIR__,
            __DIR__ . "/../public",
            __DIR__ . "/../resources", 
            __DIR__ . "/../../public",
            __DIR__ . "/../../resources"
        ];
        
        foreach ($dirsToCheck as $dir) {
            if (is_dir($dir)) {
                file_put_contents('php://stderr', "✓ $dir existe\n");
                
                // Listar archivos en este directorio
                $files = scandir($dir);
                file_put_contents('php://stderr', "   Contenido: " . implode(", ", $files) . "\n");
            } else {
                file_put_contents('php://stderr', "✗ $dir no existe\n");
            }
        }
        
        // Buscar la primera ruta válida
        foreach ($possibleLogoPaths as $path) {
            file_put_contents('php://stderr', "Verificando logo en: " . $path . "\n");
            if (file_exists($path)) {
                $logoPath = $path;
                file_put_contents('php://stderr', "✓ Logo encontrado en: " . $logoPath . "\n");
                
                // Verificar que es una imagen PNG válida
                if (getimagesize($logoPath)) {
                    file_put_contents('php://stderr', "✓ El archivo es una imagen válida\n");
                    
                    // Verificar tamaño del archivo
                    $size = filesize($logoPath);
                    file_put_contents('php://stderr', "✓ Tamaño del archivo: " . $size . " bytes\n");
                } else {
                    file_put_contents('php://stderr', "✗ El archivo no es una imagen válida\n");
                }
                break;
            } else {
                file_put_contents('php://stderr', "✗ No existe en esta ubicación\n");
            }
        }
        
        if ($logoPath && file_exists($logoPath)) {
            try {
                // Cargar la imagen original
                file_put_contents('php://stderr', "Intentando cargar la imagen PNG...\n");
                
                // Obtener información sobre la imagen
                $imageInfo = getimagesize($logoPath);
                file_put_contents('php://stderr', "Información de imagen: " . print_r($imageInfo, true) . "\n");
                $mimeType = $imageInfo['mime'] ?? '';
                file_put_contents('php://stderr', "Tipo MIME: $mimeType\n");
                
                // Cargar la imagen según su tipo
                $originalImage = null;
                if ($mimeType === 'image/png') {
                    $originalImage = @imagecreatefrompng($logoPath);
                } elseif ($mimeType === 'image/jpeg') {
                    $originalImage = @imagecreatefromjpeg($logoPath);
                } else {
                    // Intentar con PNG por defecto
                    $originalImage = @imagecreatefrompng($logoPath);
                    
                    // Si falla, intentar con JPEG
                    if (!$originalImage) {
                        file_put_contents('php://stderr', "Intentando cargar como JPEG...\n");
                        $originalImage = @imagecreatefromjpeg($logoPath);
                    }
                }
                
                if (!$originalImage) {
                    throw new Exception("No se pudo crear la imagen: " . error_get_last()['message']);
                }
                
                file_put_contents('php://stderr', "✓ Imagen cargada correctamente\n");
                
                $originalWidth = imagesx($originalImage);
                $originalHeight = imagesy($originalImage);
                file_put_contents('php://stderr', "Dimensiones originales: {$originalWidth}x{$originalHeight}\n");
                
                // Calcular el nuevo tamaño manteniendo la proporción
                $maxWidth = 556; // Ancho ajustado para mejor visualización
                $newWidth = $maxWidth;
                $newHeight = floor($originalHeight * ($maxWidth / $originalWidth));
                file_put_contents('php://stderr', "Nuevas dimensiones: {$newWidth}x{$newHeight}\n");
                
                // Crear nueva imagen redimensionada
                file_put_contents('php://stderr', "Creando imagen redimensionada...\n");
                $newImage = imagecreatetruecolor($newWidth, $newHeight);
                
                // Configuración para manejar transparencia
                imagealphablending($newImage, false);
                imagesavealpha($newImage, true);
                
                // Redimensionar
                file_put_contents('php://stderr', "Redimensionando imagen...\n");
                $resizeResult = imagecopyresampled(
                    $newImage, $originalImage,
                    0, 0, 0, 0,
                    $newWidth, $newHeight,
                    $originalWidth, $originalHeight
                );
                
                if (!$resizeResult) {
                    throw new Exception("Error al redimensionar imagen");
                }
                
                // Guardar temporalmente
                file_put_contents('php://stderr', "Guardando imagen temporal...\n");
                $tempPath = __DIR__ . "/temp_logo.png";
                $saveResult = imagepng($newImage, $tempPath);
                
                if (!$saveResult) {
                    throw new Exception("Error al guardar imagen temporal: " . error_get_last()['message']);
                }
                
                // Verificar que el archivo temporal fue creado
                if (!file_exists($tempPath)) {
                    throw new Exception("El archivo temporal no fue creado");
                }
                
                file_put_contents('php://stderr', "✓ Imagen temporal guardada en: " . $tempPath . "\n");
                file_put_contents('php://stderr', "✓ Tamaño del archivo temporal: " . filesize($tempPath) . " bytes\n");
                
                // Liberar memoria
                imagedestroy($originalImage);
                imagedestroy($newImage);
                
                // Cargar y enviar a la impresora
                file_put_contents('php://stderr', "Cargando imagen para la impresora...\n");
                $logo = EscposImage::load($tempPath);
                file_put_contents('php://stderr', "Enviando imagen a la impresora...\n");
                $printer->bitImage($logo);
                
                // Eliminar archivo temporal
                unlink($tempPath);
                
                file_put_contents('php://stderr', "✓ Logo redimensionado y enviado a la impresora\n");
                file_put_contents('php://stderr', "==== FIN DIAGNÓSTICO DEL LOGO ====\n");
            } catch (Exception $e) {
                file_put_contents('php://stderr', "✗ Error procesando la imagen: " . $e->getMessage() . "\n");
                file_put_contents('php://stderr', "==== FIN DIAGNÓSTICO DEL LOGO (ERROR) ====\n");
            }
        } else {
            file_put_contents('php://stderr', "✗ No se encontró el logo en ninguna ubicación\n");
            file_put_contents('php://stderr', "==== FIN DIAGNÓSTICO DEL LOGO (NO ENCONTRADO) ====\n");
        }
    } catch (Exception $e) {
        file_put_contents('php://stderr', "✗ Error al cargar el logo: " . $e->getMessage() . "\n");
        file_put_contents('php://stderr', "Traza: " . $e->getTraceAsString() . "\n");
        file_put_contents('php://stderr', "==== FIN DIAGNÓSTICO DEL LOGO (ERROR GENERAL) ====\n");
    }

    // Encabezado
    $printer->setEmphasis(true);
    $printer->setTextSize(1, 1);
    $printer->text("Iselín II\n");
    $printer->setEmphasis(false);
    $printer->setTextSize(1, 1);
    $printer->text("Vendedor: " . $orderData['vendedor'] . "\n");
    date_default_timezone_set('America/Argentina/Buenos_Aires');
    $printer->text(date("Y-m-d H:i:s") . "\n");
    $printer->text("-----------------------------\n");

    // Detalles de productos
    $printer->setJustification(Printer::JUSTIFY_LEFT);
    $printer->text("PRODUCTO      CANT    PRECIO    TOTAL\n");
    $printer->text("-----------------------------\n");

    foreach ($orderData['items'] as $item) {
        $nombre = str_pad(substr($item['nombre'], 0, 12), 12);
        $cantidad = str_pad(number_format($item['cantidad'], 3), 8);
        $precio = str_pad('$' . number_format($item['precioHistorico'], 2), 8);
        $subtotal = str_pad('$' . number_format($item['subtotal'], 2), 8);
        
        $printer->text("$nombre $cantidad $precio $subtotal\n");
    }

    // Total
    $printer->text("-----------------------------\n");
    $printer->setEmphasis(true);
    $printer->text(str_pad("TOTAL: $" . number_format($orderData['total'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    $printer->setEmphasis(false);

    // Método de pago
    $printer->text("Método de pago: " . strtoupper($orderData['metodoPago']) . "\n");

    // Pie de página
    $printer->setJustification(Printer::JUSTIFY_CENTER);
    $printer->text("\n¡Gracias por su compra!\n");

    $printer->feed(3);
    $printer->cut();
    $printer->pulse();
    $printer->close();
    file_put_contents('php://stderr', "Impresión completada exitosamente\n");

} catch (Exception $e) {
    file_put_contents('php://stderr', "Error: " . $e->getMessage() . "\n");
    exit(1);
}
?> 