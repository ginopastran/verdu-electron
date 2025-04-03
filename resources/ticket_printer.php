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
        file_put_contents('php://stderr', "==== DEPURACIÓN AVANZADA LOGO ====\n");
        
        // Lista de posibles rutas para el logo
        $possibleLogoPaths = [
            __DIR__ . "/logo.png",
            __DIR__ . "/../resources/logo.png",
            __DIR__ . "/../logo.png",
            __DIR__ . "/../public/logo.png",
            __DIR__ . "/../../resources/logo.png",
            __DIR__ . "/../../public/logo.png",
            __DIR__ . "/../../logo.png"
        ];
        
        file_put_contents('php://stderr', "Directorio actual: " . __DIR__ . "\n");
        file_put_contents('php://stderr', "NODE_ENV: " . getenv('NODE_ENV') . "\n");
        
        // Mostrar información del entorno
        file_put_contents('php://stderr', "Memoria disponible: " . ini_get('memory_limit') . "\n");
        file_put_contents('php://stderr', "Extensiones GD cargadas: " . (extension_loaded('gd') ? 'SÍ' : 'NO') . "\n");
        
        // Probar cada ruta
        $logoPath = null;
        foreach ($possibleLogoPaths as $path) {
            file_put_contents('php://stderr', "Probando ruta: " . $path . "\n");
            if (file_exists($path)) {
                file_put_contents('php://stderr', "✅ Existe\n");
                
                // Verificar si es legible
                if (is_readable($path)) {
                    file_put_contents('php://stderr', "✅ Es legible\n");
                    $filesize = filesize($path);
                    file_put_contents('php://stderr', "✅ Tamaño: " . $filesize . " bytes\n");
                    
                    if ($filesize > 0) {
                        $logoPath = $path;
                        file_put_contents('php://stderr', "✅ Logo encontrado en: " . $logoPath . "\n");
                        break;
                    } else {
                        file_put_contents('php://stderr', "❌ Archivo de tamaño cero\n");
                    }
                } else {
                    file_put_contents('php://stderr', "❌ No es legible\n");
                }
            } else {
                file_put_contents('php://stderr', "❌ No existe\n");
            }
        }
        
        if (!$logoPath) {
            file_put_contents('php://stderr', "❌ No se encontró ningún logo válido\n");
        } else {
            // Intentar cargar la imagen
            try {
                file_put_contents('php://stderr', "Intentando cargar imagen desde: " . $logoPath . "\n");
                
                // Verificar el tipo de imagen
                $imageInfo = @getimagesize($logoPath);
                if ($imageInfo === false) {
                    file_put_contents('php://stderr', "❌ No es una imagen válida\n");
                } else {
                    file_put_contents('php://stderr', "✅ Información de imagen: " . print_r($imageInfo, true) . "\n");
                    
                    // Cargar la imagen dependiendo del tipo
                    switch ($imageInfo[2]) {
                        case IMAGETYPE_PNG:
                            file_put_contents('php://stderr', "Es una imagen PNG\n");
                            $originalImage = @imagecreatefrompng($logoPath);
                            break;
                        case IMAGETYPE_JPEG:
                            file_put_contents('php://stderr', "Es una imagen JPEG\n");
                            $originalImage = @imagecreatefromjpeg($logoPath);
                            break;
                        default:
                            file_put_contents('php://stderr', "Tipo de imagen no soportado\n");
                            $originalImage = false;
                    }
                    
                    if ($originalImage === false) {
                        file_put_contents('php://stderr', "❌ Error al cargar la imagen: " . error_get_last()['message'] . "\n");
                    } else {
                        file_put_contents('php://stderr', "✅ Imagen cargada correctamente\n");
                        
                        $originalWidth = imagesx($originalImage);
                        $originalHeight = imagesy($originalImage);
                        file_put_contents('php://stderr', "Dimensiones: " . $originalWidth . "x" . $originalHeight . "\n");
                        
                        // Calcular el nuevo tamaño manteniendo la proporción
                        $maxWidth = 556; // Ancho ajustado para mejor visualización
                        $newWidth = $maxWidth;
                        $newHeight = floor($originalHeight * ($maxWidth / $originalWidth));
                        file_put_contents('php://stderr', "Nuevas dimensiones: " . $newWidth . "x" . $newHeight . "\n");
                        
                        // Crear nueva imagen redimensionada
                        $newImage = imagecreatetruecolor($newWidth, $newHeight);
                        if (!$newImage) {
                            file_put_contents('php://stderr', "❌ Error al crear nueva imagen\n");
                        } else {
                            // Preservar transparencia
                            imagealphablending($newImage, false);
                            imagesavealpha($newImage, true);
                            
                            // Redimensionar
                            $result = imagecopyresampled(
                                $newImage, $originalImage,
                                0, 0, 0, 0,
                                $newWidth, $newHeight,
                                $originalWidth, $originalHeight
                            );
                            
                            if (!$result) {
                                file_put_contents('php://stderr', "❌ Error al redimensionar\n");
                            } else {
                                // Guardar temporalmente en directorio temporal del sistema
                                $tempPath = sys_get_temp_dir() . "/temp_logo_" . uniqid() . ".png";
                                file_put_contents('php://stderr', "Guardando en directorio temporal: " . $tempPath . "\n");
                                $saveResult = imagepng($newImage, $tempPath);
                                
                                if (!$saveResult) {
                                    file_put_contents('php://stderr', "❌ Error al guardar imagen temporal: " . error_get_last()['message'] . "\n");
                                } else {
                                    file_put_contents('php://stderr', "✅ Imagen guardada en: " . $tempPath . "\n");
                                    file_put_contents('php://stderr', "Tamaño del archivo: " . filesize($tempPath) . " bytes\n");
                                    
                                    // Liberar memoria
                                    imagedestroy($originalImage);
                                    imagedestroy($newImage);
                                    
                                    // Cargar y enviar a la impresora
                                    try {
                                        file_put_contents('php://stderr', "Cargando para la impresora\n");
                                        $logo = EscposImage::load($tempPath);
                                        $printer->bitImage($logo);
                                        unlink($tempPath);
                                        file_put_contents('php://stderr', "✅ Logo enviado a la impresora\n");
                                    } catch (Exception $e) {
                                        file_put_contents('php://stderr', "❌ Error al imprimir logo: " . $e->getMessage() . "\n");
                                    }
                                }
                            }
                        }
                    }
                }
            } catch (Exception $e) {
                file_put_contents('php://stderr', "❌ Error procesando imagen: " . $e->getMessage() . "\n");
                file_put_contents('php://stderr', "Traza: " . $e->getTraceAsString() . "\n");
            }
        }
        
        file_put_contents('php://stderr', "==== FIN DEPURACIÓN AVANZADA LOGO ====\n");
    } catch (Exception $e) {
        file_put_contents('php://stderr', "❌ Error general: " . $e->getMessage() . "\n");
        file_put_contents('php://stderr', "Traza: " . $e->getTraceAsString() . "\n");
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