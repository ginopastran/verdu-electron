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
    file_put_contents('php://stderr', "====== INICIO DEBUG IMPRESIÓN FACTURA ======\n");
    file_put_contents('php://stderr', "🚀 Iniciando proceso de impresión de factura...\n");
    file_put_contents('php://stderr', "📅 Timestamp: " . date('Y-m-d H:i:s') . "\n");
    file_put_contents('php://stderr', "🔧 PHP Version: " . phpversion() . "\n");
    file_put_contents('php://stderr', "💾 Memory Limit: " . ini_get('memory_limit') . "\n");
    file_put_contents('php://stderr', "⚙️ NODE_ENV: " . (getenv('NODE_ENV') ?: 'not_set') . "\n");
    
    $facturaDataPath = $argv[1];
    file_put_contents('php://stderr', "📁 Ruta del archivo de datos: " . $facturaDataPath . "\n");
    
    if (!file_exists($facturaDataPath)) {
        throw new Exception("Archivo de datos no encontrado: " . $facturaDataPath);
    }
    
    $fileSize = filesize($facturaDataPath);
    file_put_contents('php://stderr', "📊 Tamaño del archivo: " . $fileSize . " bytes\n");
    
    file_put_contents('php://stderr', "📖 Leyendo datos de factura...\n");
    $rawData = file_get_contents($facturaDataPath);
    file_put_contents('php://stderr', "📋 Datos RAW recibidos: " . substr($rawData, 0, 200) . "...\n");
    
    $facturaData = json_decode($rawData, true);
    if (json_last_error() !== JSON_ERROR_NONE) {
        throw new Exception("Error al decodificar JSON: " . json_last_error_msg());
    }
    
    // Debug de la estructura de datos
    file_put_contents('php://stderr', "🔍 ESTRUCTURA DE DATOS PROCESADA:\n");
    file_put_contents('php://stderr', "- ID: " . ($facturaData['id'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Número: " . ($facturaData['numero'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Tipo: " . ($facturaData['tipoFactura'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Total: " . ($facturaData['total'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Subtotal: " . ($facturaData['subtotal'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- IVA: " . ($facturaData['impuestos'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Cliente: " . ($facturaData['cliente']['nombre'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Business Name: " . ($facturaData['businessName'] ?? 'NO DEFINIDO') . "\n");
    
    // Debug detallado de detalles
    if (isset($facturaData['detalles']) && is_array($facturaData['detalles'])) {
        file_put_contents('php://stderr', "- Cantidad de detalles: " . count($facturaData['detalles']) . "\n");
        file_put_contents('php://stderr', "- Estructura del primer detalle: " . json_encode(array_slice($facturaData['detalles'], 0, 1)) . "\n");
    } else {
        file_put_contents('php://stderr', "- Cantidad de detalles: N/A (no es array o no existe)\n");
    }

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
                                $tempPath = sys_get_temp_dir() . "/temp_logo_factura_" . uniqid() . ".png";
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
    if (isset($facturaData['businessName']) && !empty($facturaData['businessName'])) {
        $businessName = $facturaData['businessName'];
        file_put_contents('php://stderr', "✅ Usando nombre del business desde facturaData: " . $businessName . "\n");
    } elseif (isset($facturaData['sucursal']) && !empty($facturaData['sucursal'])) {
        $businessName = $facturaData['sucursal'];
        file_put_contents('php://stderr', "✅ Usando nombre de sucursal: " . $businessName . "\n");
    } else {
        file_put_contents('php://stderr', "⚠️ Usando nombre por defecto: " . $businessName . "\n");
        file_put_contents('php://stderr', "🔍 Datos disponibles en facturaData: " . json_encode(array_keys($facturaData)) . "\n");
    }
    
    $printer->text(strtoupper($businessName) . "\n");
    $printer->setEmphasis(false);
    $printer->setTextSize(1, 1);
    
    // Tipo de documento
    $printer->setEmphasis(true);
    $tipoDocumento = strtoupper($facturaData['tipoFactura'] ?? 'DOCUMENTO');
    $printer->text($tipoDocumento . "\n");
    $printer->setEmphasis(false);
    
    // Número de factura si está disponible
    if (isset($facturaData['numero']) && !empty($facturaData['numero'])) {
        $printer->text("Nº " . $facturaData['numero'] . "\n");
    } elseif (isset($facturaData['id']) && !empty($facturaData['id'])) {
        $printer->text("ID: " . $facturaData['id'] . "\n");
    }
    
    // Fecha
    date_default_timezone_set('America/Argentina/Buenos_Aires');
    $fechaFactura = isset($facturaData['fechaEmision']) ? $facturaData['fechaEmision'] : date("Y-m-d H:i:s");
    $printer->text("Fecha: " . date("d/m/Y H:i", strtotime($fechaFactura)) . "\n");
    
    $printer->text("-----------------------------\n");
    
    // Datos del cliente
    $printer->setJustification(Printer::JUSTIFY_LEFT);
    $printer->setEmphasis(true);
    $printer->text("DATOS DEL CLIENTE\n");
    $printer->setEmphasis(false);
    
    if (isset($facturaData['cliente'])) {
        $cliente = $facturaData['cliente'];
        
        // Nombre completo o razón social
        if (isset($cliente['razonSocial']) && !empty($cliente['razonSocial'])) {
            $printer->text("Razón Social: " . $cliente['razonSocial'] . "\n");
        } else {
            $nombreCompleto = trim(($cliente['nombre'] ?? '') . ' ' . ($cliente['apellido'] ?? ''));
            if (!empty($nombreCompleto)) {
                $printer->text("Cliente: " . $nombreCompleto . "\n");
            }
        }
        
        // CUIT si está disponible
        if (isset($cliente['cuit']) && !empty($cliente['cuit'])) {
            $printer->text("CUIT: " . $cliente['cuit'] . "\n");
        }
        
        // Condición fiscal
        if (isset($cliente['condicionFiscal']) && !empty($cliente['condicionFiscal'])) {
            $printer->text("Cond. Fiscal: " . $cliente['condicionFiscal'] . "\n");
        }
        
        // Dirección si está disponible
        if (isset($cliente['direccion']) && !empty($cliente['direccion'])) {
            $printer->text("Dirección: " . $cliente['direccion'] . "\n");
        }
    }
    
    $printer->text("-----------------------------\n");

    // Detalles de productos
    $printer->setJustification(Printer::JUSTIFY_LEFT);
    $printer->text("PRODUCTO      CANT    PRECIO    TOTAL\n");
    $printer->text("-----------------------------\n");

    // Verificar si hay detalles para imprimir
    if (isset($facturaData['detalles']) && is_array($facturaData['detalles']) && count($facturaData['detalles']) > 0) {
        foreach ($facturaData['detalles'] as $detalle) {
            // Manejar diferentes estructuras de nombre del producto
            $nombreProducto = '';
            if (isset($detalle['descripcion'])) {
                $nombreProducto = $detalle['descripcion'];
            } elseif (isset($detalle['producto']) && is_array($detalle['producto']) && isset($detalle['producto']['nombre'])) {
                $nombreProducto = $detalle['producto']['nombre'];
            } elseif (isset($detalle['producto']) && is_string($detalle['producto'])) {
                $nombreProducto = $detalle['producto'];
            } else {
                $nombreProducto = 'Producto';
            }
            
            // Manejar diferentes estructuras de cantidad
            $cantidad = 0;
            if (isset($detalle['cantidad'])) {
                $cantidad = $detalle['cantidad'];
            }
            
            // Manejar diferentes estructuras de precio
            $precio = 0;
            if (isset($detalle['precioUnitario'])) {
                $precio = $detalle['precioUnitario'];
            } elseif (isset($detalle['precio'])) {
                $precio = $detalle['precio'];
            }
            
            // Manejar diferentes estructuras de subtotal
            $subtotal = 0;
            if (isset($detalle['subtotal'])) {
                $subtotal = $detalle['subtotal'];
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
        file_put_contents('php://stderr', "⚠️ No hay detalles para imprimir o detalles está vacío\n");
        $printer->text("No hay productos para mostrar\n");
    }

    // Totales
    $printer->text("-----------------------------\n");
    
    // Subtotal (sin IVA)
    if (isset($facturaData['subtotal'])) {
        $printer->text(str_pad("SUBTOTAL: $" . number_format($facturaData['subtotal'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    }
    
    // IVA
    if (isset($facturaData['impuestos']) && $facturaData['impuestos'] > 0) {
        $printer->text(str_pad("IVA: $" . number_format($facturaData['impuestos'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    }
    
    // Total
    $printer->setEmphasis(true);
    $printer->text(str_pad("TOTAL: $" . number_format($facturaData['total'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    $printer->setEmphasis(false);
    
    // Pago inicial si es cuenta corriente
    if (isset($facturaData['pagoInicial']) && $facturaData['pagoInicial'] > 0) {
        $printer->text("-----------------------------\n");
        $printer->text(str_pad("PAGO INICIAL: $" . number_format($facturaData['pagoInicial'], 2), 32, " ", STR_PAD_LEFT) . "\n");
        $saldo = $facturaData['total'] - $facturaData['pagoInicial'];
        $printer->setEmphasis(true);
        $printer->text(str_pad("SALDO: $" . number_format($saldo, 2), 32, " ", STR_PAD_LEFT) . "\n");
        $printer->setEmphasis(false);
    }

    // Observaciones si existen
    if (isset($facturaData['observaciones']) && !empty($facturaData['observaciones'])) {
        $printer->text("-----------------------------\n");
        $printer->text("OBSERVACIONES:\n");
        $printer->text($facturaData['observaciones'] . "\n");
    }
    
    // Información AFIP si está disponible
    if (isset($facturaData['afip'])) {
        $printer->text("-----------------------------\n");
        $printer->setEmphasis(true);
        $printer->text("INFORMACIÓN AFIP\n");
        $printer->setEmphasis(false);
        
        if (isset($facturaData['afip']['cae'])) {
            $printer->text("CAE: " . $facturaData['afip']['cae'] . "\n");
        }
        
        if (isset($facturaData['afip']['vencimientoCae'])) {
            $printer->text("Venc. CAE: " . $facturaData['afip']['vencimientoCae'] . "\n");
        }
    }

    // Pie de página
    $printer->setJustification(Printer::JUSTIFY_CENTER);
    $printer->text("\n¡Gracias por su compra!\n");
    
    $printer->feed(3);
    $printer->cut();
    $printer->pulse();
    $printer->close();
    
    // Debug final
    file_put_contents('php://stderr', "✅ Impresión de factura completada exitosamente\n");
    file_put_contents('php://stderr', "📊 ESTADÍSTICAS FINALES:\n");
    file_put_contents('php://stderr', "- Detalles procesados: " . (isset($facturaData['detalles']) ? count($facturaData['detalles']) : 0) . "\n");
    file_put_contents('php://stderr', "- Total impreso: $" . number_format($facturaData['total'], 2) . "\n");
    file_put_contents('php://stderr', "- Tipo documento: " . ($facturaData['tipoFactura'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Business mostrado: " . $businessName . "\n");
    file_put_contents('php://stderr', "====== FIN DEBUG IMPRESIÓN FACTURA ======\n");

} catch (Exception $e) {
    file_put_contents('php://stderr', "Error: " . $e->getMessage() . "\n");
    exit(1);
}
?>