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
    // Debug avanzado del proceso de impresión
    file_put_contents('php://stderr', "====== INICIO DEBUG IMPRESIÓN TICKET ======\n");
    file_put_contents('php://stderr', "🚀 Iniciando proceso de impresión...\n");
    file_put_contents('php://stderr', "📅 Timestamp: " . date('Y-m-d H:i:s') . "\n");
    file_put_contents('php://stderr', "🔧 PHP Version: " . phpversion() . "\n");
    file_put_contents('php://stderr', "💾 Memory Limit: " . ini_get('memory_limit') . "\n");
    file_put_contents('php://stderr', "⚙️ NODE_ENV: " . (getenv('NODE_ENV') ?: 'not_set') . "\n");
    
    $orderDataPath = $argv[1];
    file_put_contents('php://stderr', "📁 Ruta del archivo de datos: " . $orderDataPath . "\n");
    
    if (!file_exists($orderDataPath)) {
        throw new Exception("Archivo de datos no encontrado: " . $orderDataPath);
    }
    
    $fileSize = filesize($orderDataPath);
    file_put_contents('php://stderr', "📊 Tamaño del archivo: " . $fileSize . " bytes\n");
    
    file_put_contents('php://stderr', "📖 Leyendo datos de orden...\n");
    $rawData = file_get_contents($orderDataPath);
    file_put_contents('php://stderr', "📋 Datos RAW recibidos: " . substr($rawData, 0, 200) . "...\n");
    
    $orderData = json_decode($rawData, true);
    if (json_last_error() !== JSON_ERROR_NONE) {
        throw new Exception("Error al decodificar JSON: " . json_last_error_msg());
    }
    
    // Debug de la estructura de datos
    file_put_contents('php://stderr', "🔍 ESTRUCTURA DE DATOS PROCESADA:\n");
    file_put_contents('php://stderr', "- ID: " . ($orderData['idReal'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Total: " . ($orderData['total'] ?? 'N/A') . "\n");
    // Debug del vendedor con manejo de array
    $vendedorDebug = 'N/A';
    if (isset($orderData['vendedor'])) {
        if (is_array($orderData['vendedor'])) {
            $vendedorDebug = $orderData['vendedor']['nombre'] ?? $orderData['vendedor']['name'] ?? 'Array sin nombre';
        } else {
            $vendedorDebug = $orderData['vendedor'];
        }
    }
    file_put_contents('php://stderr', "- Vendedor: " . $vendedorDebug . "\n");
    file_put_contents('php://stderr', "- Fecha: " . ($orderData['createdAt'] ?? $orderData['fecha'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Business Name: " . ($orderData['businessName'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Sucursal: " . ($orderData['sucursal'] ?? 'NO DEFINIDO') . "\n");
    // Debug detallado de items
    if (isset($orderData['items']) && is_array($orderData['items'])) {
        file_put_contents('php://stderr', "- Cantidad de items: " . count($orderData['items']) . "\n");
        file_put_contents('php://stderr', "- Estructura del primer item: " . json_encode(array_slice($orderData['items'], 0, 1)) . "\n");
    } else {
        file_put_contents('php://stderr', "- Cantidad de items: N/A (no es array o no existe)\n");
    }
    file_put_contents('php://stderr', "- Método de pago: " . ($orderData['metodoPago'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Pagos múltiples: " . (isset($orderData['pagos']) ? 'SÍ (' . count($orderData['pagos']) . ')' : 'NO') . "\n");
    
    // Debug de descuentos
    file_put_contents('php://stderr', "🔍 DEBUG DESCUENTOS:\n");
    if (isset($orderData['discountData'])) {
        file_put_contents('php://stderr', "- discountData existe: SÍ\n");
        file_put_contents('php://stderr', "- discountData completa: " . json_encode($orderData['discountData']) . "\n");
    } else {
        file_put_contents('php://stderr', "- discountData: NO EXISTE\n");
    }
    if (isset($orderData['subtotal']) && isset($orderData['total'])) {
        $descuentoCalculado = $orderData['subtotal'] - $orderData['total'];
        file_put_contents('php://stderr', "- Descuento calculado (subtotal - total): -$" . number_format($descuentoCalculado, 2) . "\n");
    }
    if (isset($orderData['descuentoAplicado'])) {
        file_put_contents('php://stderr', "- descuentoAplicado: " . json_encode($orderData['descuentoAplicado']) . "\n");
    }
    file_put_contents('php://stderr', "\n");

    $nombre_impresora = "TP806L";
    file_put_contents('php://stderr', "Conectando a impresora: " . $nombre_impresora . "\n");
    
    try {
        // Intentar conectar a la impresora - esto fallará si no existe
        $connector = new WindowsPrintConnector($nombre_impresora);
        
        // Si llegamos aquí, la conexión fue exitosa
        file_put_contents('php://stderr', "Conexión exitosa a la impresora\n");
    } catch (Exception $e) {
        // Error específico de la impresora - reportarlo pero no interrumpir el proceso
        file_put_contents('php://stderr', "Error: " . $e->getMessage() . "\n");
        exit(1);
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

    // Encabezado dinámico con nombre del business
    $printer->setEmphasis(true);
    $printer->setTextSize(2, 2);
    
    // Determinar el nombre del business de manera dinámica
    $businessName = "Verdulería"; // Valor por defecto
    if (isset($orderData['businessName']) && !empty($orderData['businessName'])) {
        $businessName = $orderData['businessName'];
        file_put_contents('php://stderr', "✅ Usando nombre del business desde orderData: " . $businessName . "\n");
    } elseif (isset($orderData['sucursal']) && !empty($orderData['sucursal'])) {
        $businessName = $orderData['sucursal'];
        file_put_contents('php://stderr', "✅ Usando nombre de sucursal: " . $businessName . "\n");
    } else {
        file_put_contents('php://stderr', "⚠️ Usando nombre por defecto: " . $businessName . "\n");
        file_put_contents('php://stderr', "🔍 Datos disponibles en orderData: " . json_encode(array_keys($orderData)) . "\n");
    }
    
    $printer->text(strtoupper($businessName) . "\n");
    $printer->setEmphasis(false);
    $printer->setTextSize(1, 1);
    // Manejar vendedor que puede venir como string o como array
    $vendedorText = 'N/A';
    if (isset($orderData['vendedor'])) {
        if (is_array($orderData['vendedor'])) {
            // Si es array, usar la propiedad 'nombre' si existe
            $vendedorText = $orderData['vendedor']['nombre'] ?? $orderData['vendedor']['name'] ?? 'N/A';
        } else {
            // Si es string, usarlo directamente
            $vendedorText = $orderData['vendedor'];
        }
    }
    $printer->text("Vendedor: " . $vendedorText . "\n");
    date_default_timezone_set('America/Argentina/Buenos_Aires');
    $printer->text(date("Y-m-d H:i:s") . "\n");
    
    // Añadir ID real de la orden si está disponible
    if (isset($orderData['idReal']) && !empty($orderData['idReal'])) {
        $printer->text("Orden #" . $orderData['idReal'] . "\n");
        file_put_contents('php://stderr', "✅ ID Real de la orden encontrado: " . $orderData['idReal'] . "\n");
    } elseif (isset($orderData['id']) && !empty($orderData['id'])) {
        $printer->text("Orden #" . $orderData['id'] . "\n");
        file_put_contents('php://stderr', "⚠️ Usando ID regular de la orden: " . $orderData['id'] . "\n");
    } else {
        file_put_contents('php://stderr', "❌ No se encontró ID de orden (idReal o id)\n");
        file_put_contents('php://stderr', "🔍 Claves disponibles: " . json_encode(array_keys($orderData)) . "\n");
    }
    
    $printer->text("-----------------------------\n");

    // Detalles de productos
    $printer->setJustification(Printer::JUSTIFY_LEFT);
    $printer->text("PRODUCTO      CANT    PRECIO    TOTAL\n");
    $printer->text("-----------------------------\n");

    // Verificar si hay items para imprimir
    if (isset($orderData['items']) && is_array($orderData['items']) && count($orderData['items']) > 0) {
        foreach ($orderData['items'] as $item) {
            // Manejar diferentes estructuras de nombre del producto
            $nombreProducto = '';
            if (isset($item['nombre'])) {
                $nombreProducto = $item['nombre'];
            } elseif (isset($item['producto']) && is_array($item['producto']) && isset($item['producto']['nombre'])) {
                $nombreProducto = $item['producto']['nombre'];
            } elseif (isset($item['producto']) && is_string($item['producto'])) {
                $nombreProducto = $item['producto'];
            } else {
                $nombreProducto = 'Producto';
            }
            
            // Manejar diferentes estructuras de cantidad
            $cantidad = 0;
            if (isset($item['cantidad'])) {
                $cantidad = $item['cantidad'];
            } elseif (isset($item['qty'])) {
                $cantidad = $item['qty'];
            }
            
            // Manejar diferentes estructuras de precio
            $precio = 0;
            if (isset($item['precioHistorico'])) {
                $precio = $item['precioHistorico'];
            } elseif (isset($item['precio'])) {
                $precio = $item['precio'];
            } elseif (isset($item['price'])) {
                $precio = $item['price'];
            }
            
            // Manejar diferentes estructuras de subtotal
            $subtotal = 0;
            if (isset($item['subtotal'])) {
                $subtotal = $item['subtotal'];
            } else {
                $subtotal = $cantidad * $precio;
            }
            
            $nombre = str_pad(substr($nombreProducto, 0, 12), 12);
            $cantidadFormateada = str_pad(number_format($cantidad, 3), 8);
            $precioFormateado = str_pad('$' . number_format($precio, 2), 8);
            $subtotalFormateado = str_pad('$' . number_format($subtotal, 2), 8);
            
            $printer->text("$nombre $cantidadFormateada $precioFormateado $subtotalFormateado\n");
        }
    } else {
        file_put_contents('php://stderr', "⚠️ No hay items para imprimir o items está vacío\n");
        $printer->text("No hay productos para mostrar\n");
    }

    // Total
    $printer->text("-----------------------------\n");
    
    // Mostrar subtotal y descuentos si existen
    $hasDiscount = false;
    $subtotalOriginal = 0;
    $descuentoMonto = 0;
    $tipoDescuento = '';
    $valorDescuento = 0;
    
    // Verificar si hay información de descuentos
    if (isset($orderData['discountData']) && is_array($orderData['discountData'])) {
        $discountData = $orderData['discountData'];
        if (isset($discountData['amount']) && $discountData['amount'] > 0) {
            $hasDiscount = true;
            $descuentoMonto = $discountData['amount'];
            $tipoDescuento = $discountData['type'] ?? 'fixed';
            $valorDescuento = $discountData['value'] ?? 0;
            
            // ✅ CORRECCIÓN CRÍTICA: Usar el subtotal que viene del frontend
            // El frontend ya calcula correctamente el subtotal original antes del descuento
            if (isset($orderData['subtotal']) && $orderData['subtotal'] > 0) {
                $subtotalOriginal = $orderData['subtotal'];
            } else {
                // Fallback: calcular sumando total + descuento solo si no viene subtotal
                $subtotalOriginal = $orderData['total'] + $descuentoMonto;
            }
            
            // Debug para verificar cálculos
            file_put_contents('php://stderr', "🔍 DESCUENTO DEBUG:\n");
            file_put_contents('php://stderr', "- Subtotal original (del frontend): $" . number_format($subtotalOriginal, 2) . "\n");
            file_put_contents('php://stderr', "- Descuento aplicado: -$" . number_format($descuentoMonto, 2) . "\n");
            file_put_contents('php://stderr', "- Total final: $" . number_format($orderData['total'], 2) . "\n");
            file_put_contents('php://stderr', "- Tipo descuento: " . $tipoDescuento . "\n");
            file_put_contents('php://stderr', "- Valor descuento: " . $valorDescuento . "\n");
            file_put_contents('php://stderr', "- Verificación: $" . number_format($subtotalOriginal, 2) . " - $" . number_format($descuentoMonto, 2) . " = $" . number_format($subtotalOriginal - $descuentoMonto, 2) . "\n");
        }
    } elseif (isset($orderData['subtotal']) && isset($orderData['total']) && $orderData['subtotal'] > $orderData['total']) {
        // Calcular descuento basado en subtotal y total (fallback)
        $hasDiscount = true;
        $subtotalOriginal = $orderData['subtotal'];
        $descuentoMonto = $subtotalOriginal - $orderData['total'];
        
        file_put_contents('php://stderr', "🔍 DESCUENTO FALLBACK:\n");
        file_put_contents('php://stderr', "- Subtotal original: $" . number_format($subtotalOriginal, 2) . "\n");
        file_put_contents('php://stderr', "- Descuento calculado: -$" . number_format($descuentoMonto, 2) . "\n");
    }
    
    // Mostrar desglose si hay descuento
    if ($hasDiscount) {
        $printer->text("Subtotal: $" . number_format($subtotalOriginal, 2) . "\n");
        
        // Mostrar información del descuento
        if (!empty($tipoDescuento) && $valorDescuento > 0) {
            if ($tipoDescuento === 'percentage') {
                $printer->text("Descuento (" . number_format($valorDescuento, 1) . "%): -$" . number_format($descuentoMonto, 2) . "\n");
            } else {
                $printer->text("Descuento: -$" . number_format($descuentoMonto, 2) . "\n");
            }
        } else {
            $printer->text("Descuento aplicado: -$" . number_format($descuentoMonto, 2) . "\n");
        }
        $printer->text("-----------------------------\n");
    }
    
    $printer->setEmphasis(true);
    $printer->text(str_pad("TOTAL: $" . number_format($orderData['total'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    $printer->setEmphasis(false);

    // Método de pago
    // Verificar si es un pago con múltiples métodos
    if (isset($orderData['pagos']) && is_array($orderData['pagos']) && count($orderData['pagos']) > 1) {
        $printer->text("MÉTODOS DE PAGO:\n");
        foreach ($orderData['pagos'] as $pago) {
            $metodoPago = strtoupper($pago['metodoPago']);
            $monto = number_format($pago['monto'], 2);
            $printer->text("$metodoPago: $$monto\n");
        }
    } else {
        // Para pagos con un solo método, mantener el comportamiento actual
        $printer->text("Método de pago: " . strtoupper($orderData['metodoPago']) . "\n");
    }

    // Pie de página
    $printer->setJustification(Printer::JUSTIFY_CENTER);
    $printer->text("\n¡Gracias por su compra!\n");

    // Información adicional del ticket (IDs removidos según solicitud)
    
    $printer->feed(3);
    $printer->cut();
    $printer->pulse();
    $printer->close();
    
    // Debug final
    file_put_contents('php://stderr', "✅ Impresión completada exitosamente\n");
    file_put_contents('php://stderr', "📊 ESTADÍSTICAS FINALES:\n");
    file_put_contents('php://stderr', "- Items procesados: " . (isset($orderData['items']) ? count($orderData['items']) : 0) . "\n");
    file_put_contents('php://stderr', "- Total impreso: $" . number_format($orderData['total'], 2) . "\n");
    file_put_contents('php://stderr', "- Business mostrado: " . $businessName . "\n");
    file_put_contents('php://stderr', "====== FIN DEBUG IMPRESIÓN TICKET ======\n");

} catch (Exception $e) {
    file_put_contents('php://stderr', "Error: " . $e->getMessage() . "\n");
    exit(1);
}
?>