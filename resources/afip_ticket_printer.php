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
    // Debug avanzado del proceso de impresión AFIP
    file_put_contents('php://stderr', "====== INICIO DEBUG IMPRESIÓN TICKET AFIP ======\n");
    file_put_contents('php://stderr', "🧾 Iniciando proceso de impresión AFIP...\n");
    file_put_contents('php://stderr', "📅 Timestamp: " . date('Y-m-d H:i:s') . "\n");
    file_put_contents('php://stderr', "🔧 PHP Version: " . phpversion() . "\n");
    file_put_contents('php://stderr', "💾 Memory Limit: " . ini_get('memory_limit') . "\n");
    
    $afipDataPath = $argv[1];
    file_put_contents('php://stderr', "📁 Ruta del archivo de datos AFIP: " . $afipDataPath . "\n");
    
    if (!file_exists($afipDataPath)) {
        throw new Exception("Archivo de datos AFIP no encontrado: " . $afipDataPath);
    }
    
    $fileSize = filesize($afipDataPath);
    file_put_contents('php://stderr', "📊 Tamaño del archivo: " . $fileSize . " bytes\n");
    
    file_put_contents('php://stderr', "📖 Leyendo datos de factura AFIP...\n");
    $rawData = file_get_contents($afipDataPath);
    file_put_contents('php://stderr', "📋 Datos RAW recibidos: " . substr($rawData, 0, 200) . "...\n");
    
    $afipData = json_decode($rawData, true);
    if (json_last_error() !== JSON_ERROR_NONE) {
        throw new Exception("Error al decodificar JSON: " . json_last_error_msg());
    }
    
    // Debug de la estructura de datos AFIP
    file_put_contents('php://stderr', "🔍 ESTRUCTURA DE DATOS AFIP PROCESADA:\n");
    file_put_contents('php://stderr', "- CAE: " . ($afipData['cae'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Fecha Vto CAE: " . ($afipData['fechaVtoCae'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Número Factura: " . ($afipData['numeroFactura'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Total: " . ($afipData['total'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Tipo Factura: " . ($afipData['tipoFactura'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Business Name: " . ($afipData['businessName'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Condición IVA: " . ($afipData['condicionIva'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "- Cantidad de items: " . (isset($afipData['items']) ? count($afipData['items']) : 'N/A') . "\n");
    
    // 🆕 DEBUG ESPECÍFICO DE CONFIGURACIÓN AFIP
    file_put_contents('php://stderr', "\n🔍 DEBUG CONFIGURACIÓN AFIP:\n");
    if (isset($afipData['configuracionAfip'])) {
        file_put_contents('php://stderr', "- configuracionAfip existe: SÍ\n");
        file_put_contents('php://stderr', "- configuracionAfip completa: " . json_encode($afipData['configuracionAfip']) . "\n");
        if (isset($afipData['configuracionAfip']['condicionIva'])) {
            file_put_contents('php://stderr', "- configuracionAfip.condicionIva: " . $afipData['configuracionAfip']['condicionIva'] . "\n");
        } else {
            file_put_contents('php://stderr', "- configuracionAfip.condicionIva: NO EXISTE\n");
        }
        if (isset($afipData['configuracionAfip']['tipoFactura'])) {
            file_put_contents('php://stderr', "- configuracionAfip.tipoFactura: " . $afipData['configuracionAfip']['tipoFactura'] . "\n");
        } else {
            file_put_contents('php://stderr', "- configuracionAfip.tipoFactura: NO EXISTE\n");
        }
    } else {
        file_put_contents('php://stderr', "- configuracionAfip: NO EXISTE\n");
    }
    file_put_contents('php://stderr', "\n");

    $nombre_impresora = "TP806L";
    file_put_contents('php://stderr', "Conectando a impresora AFIP: " . $nombre_impresora . "\n");
    
    try {
        // Intentar conectar a la impresora - esto fallará si no existe
        $connector = new WindowsPrintConnector($nombre_impresora);
        
        // Si llegamos aquí, la conexión fue exitosa
        file_put_contents('php://stderr', "Conexión exitosa a la impresora AFIP\n");
    } catch (Exception $e) {
        // Error específico de la impresora - reportarlo pero no interrumpir el proceso
        file_put_contents('php://stderr', "Error: " . $e->getMessage() . "\n");
        exit(1);
    }

    $printer = new Printer($connector);
    file_put_contents('php://stderr', "Impresora AFIP inicializada\n");

    // Configuración inicial
    $printer->setJustification(Printer::JUSTIFY_CENTER);

    // Logo (opcional) - usar el mismo sistema que el ticket normal
    try {
        file_put_contents('php://stderr', "==== LOGO PARA FACTURA AFIP ====\n");
        
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
        
        $logoPath = null;
        foreach ($possibleLogoPaths as $path) {
            if (file_exists($path) && is_readable($path) && filesize($path) > 0) {
                $logoPath = $path;
                file_put_contents('php://stderr', "✅ Logo AFIP encontrado en: " . $logoPath . "\n");
                break;
            }
        }
        
        if ($logoPath) {
            $imageInfo = @getimagesize($logoPath);
            if ($imageInfo !== false) {
                switch ($imageInfo[2]) {
                    case IMAGETYPE_PNG:
                        $originalImage = @imagecreatefrompng($logoPath);
                        break;
                    case IMAGETYPE_JPEG:
                        $originalImage = @imagecreatefromjpeg($logoPath);
                        break;
                    default:
                        $originalImage = false;
                }
                
                if ($originalImage !== false) {
                    $originalWidth = imagesx($originalImage);
                    $originalHeight = imagesy($originalImage);
                    
                    // Calcular el nuevo tamaño manteniendo la proporción
                    $maxWidth = 556;
                    $newWidth = $maxWidth;
                    $newHeight = floor($originalHeight * ($maxWidth / $originalWidth));
                    
                    // Crear nueva imagen redimensionada
                    $newImage = imagecreatetruecolor($newWidth, $newHeight);
                    if ($newImage) {
                        imagealphablending($newImage, false);
                        imagesavealpha($newImage, true);
                        
                        if (imagecopyresampled($newImage, $originalImage, 0, 0, 0, 0, $newWidth, $newHeight, $originalWidth, $originalHeight)) {
                            // Guardar temporalmente
                            $tempPath = sys_get_temp_dir() . "/temp_afip_logo_" . uniqid() . ".png";
                            if (imagepng($newImage, $tempPath)) {
                                try {
                                    $logo = EscposImage::load($tempPath);
                                    $printer->bitImage($logo);
                                    unlink($tempPath);
                                    file_put_contents('php://stderr', "✅ Logo AFIP enviado a la impresora\n");
                                } catch (Exception $e) {
                                    file_put_contents('php://stderr', "❌ Error al imprimir logo AFIP: " . $e->getMessage() . "\n");
                                }
                            }
                        }
                        imagedestroy($newImage);
                    }
                    imagedestroy($originalImage);
                }
            }
        }
    } catch (Exception $e) {
        file_put_contents('php://stderr', "❌ Error procesando logo AFIP: " . $e->getMessage() . "\n");
    }

    // Encabezado con información del business
    $printer->setEmphasis(true);
    $printer->setTextSize(2, 2);
    
    // Determinar el nombre del business de manera dinámica
    $businessName = "Comercio"; // Valor por defecto más genérico
    $razonSocial = "Comercio";
    
    // Buscar el nombre del business en diferentes ubicaciones posibles
    if (isset($afipData['businessName']) && !empty($afipData['businessName'])) {
        $businessName = $afipData['businessName'];
        file_put_contents('php://stderr', "✅ Usando nombre del business desde afipData.businessName: " . $businessName . "\n");
    } elseif (isset($afipData['nombre']) && !empty($afipData['nombre'])) {
        $businessName = $afipData['nombre'];
        file_put_contents('php://stderr', "✅ Usando nombre del business desde afipData.nombre: " . $businessName . "\n");
    } else {
        file_put_contents('php://stderr', "⚠️ Usando nombre por defecto: " . $businessName . "\n");
    }
    
    // Buscar razón social en la nueva estructura configuracionAfip
    if (isset($afipData['configuracionAfip']) && is_array($afipData['configuracionAfip']) && 
        isset($afipData['configuracionAfip']['razonSocial']) && !empty($afipData['configuracionAfip']['razonSocial'])) {
        $razonSocial = $afipData['configuracionAfip']['razonSocial'];
        file_put_contents('php://stderr', "✅ Usando razón social desde configuracionAfip.razonSocial: " . $razonSocial . "\n");
    } elseif (isset($afipData['razonSocial']) && !empty($afipData['razonSocial'])) {
        $razonSocial = $afipData['razonSocial'];
        file_put_contents('php://stderr', "✅ Usando razón social desde afipData.razonSocial: " . $razonSocial . "\n");
    } else {
        $razonSocial = $businessName;
        file_put_contents('php://stderr', "⚠️ Usando businessName como razón social: " . $razonSocial . "\n");
    }
    
    // Imprimir el nombre del business en grande arriba con salto de línea inteligente
    $businessNameUpper = strtoupper($businessName);
    $maxCharsPerLine = 20; // Máximo de caracteres por línea para texto grande (aumentado)
    
    if (strlen($businessNameUpper) <= $maxCharsPerLine) {
        // Si cabe en una línea, imprimir normalmente
        $printer->text($businessNameUpper . "\n");
    } else {
        // Si no cabe, dividir por palabras sin cortar
        $words = explode(' ', $businessNameUpper);
        $currentLine = '';
        
        foreach ($words as $word) {
            // Si agregar esta palabra excede el límite
            if (strlen($currentLine . ' ' . $word) > $maxCharsPerLine) {
                // Imprimir la línea actual si no está vacía
                if (!empty($currentLine)) {
                    $printer->text(trim($currentLine) . "\n");
                    $currentLine = $word;
                } else {
                    // Si una sola palabra es muy larga, la imprimimos completa
                    $printer->text($word . "\n");
                }
            } else {
                // Agregar la palabra a la línea actual
                $currentLine .= (empty($currentLine) ? '' : ' ') . $word;
            }
        }
        
        // Imprimir la última línea si no está vacía
        if (!empty($currentLine)) {
            $printer->text(trim($currentLine) . "\n");
        }
    }
    $printer->setEmphasis(false);
    $printer->setTextSize(1, 1);

    // Información AFIP requerida
    $printer->text("Razón Social: " . $razonSocial . "\n");

    // Formatear CUIT si viene sin guiones (11 dígitos)
    $cuitValue = $afipData['cuit'] ?? '00-00000000-0';
    if (preg_match('/^\d{11}$/', $cuitValue)) {
        $cuitValue = substr($cuitValue, 0, 2) . '-' . substr($cuitValue, 2, 8) . '-' . substr($cuitValue, 10, 1);
    }
    $printer->text("CUIT: " . $cuitValue . "\n");
    
    // Obtener condición IVA desde configuracionAfip si está disponible
    $condicionIva = 'Responsable Inscripto'; // Valor por defecto
    if (isset($afipData['configuracionAfip']) && is_array($afipData['configuracionAfip']) && 
        isset($afipData['configuracionAfip']['condicionIva']) && !empty($afipData['configuracionAfip']['condicionIva'])) {
        $condicionIva = $afipData['configuracionAfip']['condicionIva'];
        file_put_contents('php://stderr', "✅ Usando condición IVA desde configuracionAfip: " . $condicionIva . "\n");
    } elseif (isset($afipData['condicionIva']) && !empty($afipData['condicionIva'])) {
        $condicionIva = $afipData['condicionIva'];
        file_put_contents('php://stderr', "✅ Usando condición IVA desde afipData: " . $condicionIva . "\n");
    } else {
        file_put_contents('php://stderr', "⚠️ Usando condición IVA por defecto: " . $condicionIva . "\n");
    }
    
    // Capitalizar primera letra de condición IVA
    $condicionIva = ucfirst(strtolower($condicionIva));
    
    $printer->text("Condición IVA: " . $condicionIva . "\n");
    $printer->text("Dirección: " . ($afipData['direccion'] ?? 'Dirección no configurada') . "\n");
    
    $printer->text("-----------------------------\n");
    
    // Tipo de comprobante centrado y destacado
    $printer->setJustification(Printer::JUSTIFY_CENTER);
    $printer->setEmphasis(true);
    $printer->setTextSize(1, 2);
    
    // Determinar tipo de factura basado en la condición IVA del negocio
    $tipoFactura = 'B'; // Valor por defecto
    
    // Obtener condición IVA para determinar el tipo de factura
    $condicionIvaParaTipo = '';
    if (isset($afipData['configuracionAfip']) && is_array($afipData['configuracionAfip']) && 
        isset($afipData['configuracionAfip']['condicionIva']) && !empty($afipData['configuracionAfip']['condicionIva'])) {
        $condicionIvaParaTipo = strtolower($afipData['configuracionAfip']['condicionIva']);
    } elseif (isset($afipData['condicionIva']) && !empty($afipData['condicionIva'])) {
        $condicionIvaParaTipo = strtolower($afipData['condicionIva']);
    }
    
    // Si la condición IVA es monotributo, usar factura C, sino B
    if ($condicionIvaParaTipo === 'monotributo') {
        $tipoFactura = 'C';
        file_put_contents('php://stderr', "✅ Negocio es Monotributo - Usando FACTURA C\n");
    } else {
        $tipoFactura = 'B';
        file_put_contents('php://stderr', "✅ Negocio no es Monotributo (" . $condicionIvaParaTipo . ") - Usando FACTURA B\n");
    }
    
    // Verificar si el valor ya contiene 'FACTURA' para evitar duplicación
    if (strpos($tipoFactura, 'FACTURA') !== false) {
        $tipoFacturaCompleto = $tipoFactura;
    } else {
        $tipoFacturaCompleto = 'FACTURA ' . $tipoFactura;
    }
    
    $printer->text("$tipoFacturaCompleto\n");
    $printer->setEmphasis(false);
    $printer->setTextSize(1, 1);
    $printer->setJustification(Printer::JUSTIFY_LEFT);
    
    // Número de factura y fecha
    $printer->text("Nro: " . ($afipData['puntoVenta'] ?? '0001') . "-" . str_pad($afipData['numeroFactura'] ?? '1', 8, '0', STR_PAD_LEFT) . "\n");
    date_default_timezone_set('America/Argentina/Buenos_Aires');
    $fechaFactura = $afipData['fechaHora'] ?? date("d/m/Y H:i:s");
    $printer->text("Fecha: " . $fechaFactura . "\n");
    $printer->text("Vendedor: " . ($afipData['vendedor'] ?? $afipData['usuario'] ?? 'N/A') . "\n");

    // Añadir ID real de la orden/factura si está disponible
    if (isset($afipData['idReal']) && !empty($afipData['idReal'])) {
        $printer->text("Orden #" . $afipData['idReal'] . "\n");
        file_put_contents('php://stderr', "✅ ID Real encontrado: " . $afipData['idReal'] . "\n");
    } elseif (isset($afipData['factura']) && is_array($afipData['factura']) && isset($afipData['factura']['idReal']) && !empty($afipData['factura']['idReal'])) {
        $printer->text("Orden #" . $afipData['factura']['idReal'] . "\n");
        file_put_contents('php://stderr', "✅ ID Real encontrado en factura: " . $afipData['factura']['idReal'] . "\n");
    } else {
        file_put_contents('php://stderr', "⚠️ No se encontró ID Real en los datos AFIP\n");
    }

    $printer->text("-----------------------------\n");

    // Obtener condición IVA desde configuracionAfip si está disponible
    $condicionIva = 'Consumidor Final'; // Valor por defecto
    if (isset($afipData['configuracionAfip']) && is_array($afipData['configuracionAfip']) && 
        isset($afipData['configuracionAfip']['condicionIva']) && !empty($afipData['configuracionAfip']['condicionIva'])) {
        $condicionIva = ucfirst(strtolower($afipData['configuracionAfip']['condicionIva']));
        file_put_contents('php://stderr', "✅ Usando condición IVA desde configuracionAfip: " . $condicionIva . "\n");
    } elseif (isset($afipData['condicionIva']) && !empty($afipData['condicionIva'])) {
        $condicionIva = ucfirst(strtolower($afipData['condicionIva']));
        file_put_contents('php://stderr', "✅ Usando condición IVA desde afipData: " . $condicionIva . "\n");
    } else {
        file_put_contents('php://stderr', "⚠️ Usando condición IVA por defecto: " . $condicionIva . "\n");
    }
    
    $printer->text("Condición IVA: " . $condicionIva . "\n");
    
    $printer->text("-----------------------------\n");

    // Detalles de productos
    $printer->text("PRODUCTO      CANT    PRECIO    TOTAL\n");
    $printer->text("-----------------------------\n");

    $subtotalNeto = 0;
    $totalIva = 0;

    // Debug de items
    file_put_contents('php://stderr', "🔍 DEBUG ITEMS AFIP:\n");
    file_put_contents('php://stderr', json_encode($afipData['items'], JSON_PRETTY_PRINT) . "\n");

    foreach ($afipData['items'] as $item) {
        // Debug del item actual
        file_put_contents('php://stderr', "📦 Procesando item: " . json_encode($item) . "\n");
        
        $nombre = str_pad(substr($item['nombre'], 0, 12), 12);
        $cantidad = str_pad(number_format($item['cantidad'], 3), 8);
        
        // Calcular precio unitario desde el subtotal y cantidad si no está disponible
        $precioUnitario = isset($item['precio']) ? $item['precio'] : 
                         (isset($item['precioHistorico']) ? $item['precioHistorico'] : 
                         ($item['cantidad'] > 0 ? $item['subtotal'] / $item['cantidad'] : 0));
        
        file_put_contents('php://stderr', "💰 Precio calculado para {$item['nombre']}: $precioUnitario\n");
        
        $precio = str_pad('$' . number_format($precioUnitario, 2), 8);
        $subtotal = str_pad('$' . number_format($item['subtotal'], 2), 8);
        
        $printer->text("$nombre $cantidad $precio $subtotal\n");
        
        // Calcular subtotal neto (sin IVA) y IVA
        $subtotalNeto += $item['subtotal'] / 1.21; // Asumiendo IVA 21%
        $totalIva += $item['subtotal'] - ($item['subtotal'] / 1.21);
    }

    $printer->text("-----------------------------\n");
    
    // Discriminación de IVA para Factura B
    $printer->text("Subtotal: $" . number_format($subtotalNeto, 2) . "\n");
    $printer->text("IVA (21%): $" . number_format($totalIva, 2) . "\n");
    
    // Total
    $printer->setEmphasis(true);
    $printer->text(str_pad("TOTAL: $" . number_format($afipData['total'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    $printer->setEmphasis(false);

    // Información AFIP obligatoria
    $printer->text("-----------------------------\n");
    $printer->setJustification(Printer::JUSTIFY_CENTER);
    $printer->text("COMPROBANTE AUTORIZADO\n");
    
    // CAE - Código de Autorización Electrónico
    $cae = $afipData['cae'] ?? 'NO DISPONIBLE';
    $printer->setEmphasis(true);
    $printer->text("CAE: " . $cae . "\n");
    $printer->setEmphasis(false);
    
    // Fecha de vencimiento del CAE
    $fechaVtoCae = $afipData['fechaVtoCae'] ?? date('Ymd');
    if (strlen($fechaVtoCae) === 8) {
        // Formato YYYYMMDD de AFIP
        $fechaFormateada = date('d/m/Y', strtotime($fechaVtoCae));
    } else {
        $fechaFormateada = date('d/m/Y', strtotime($fechaVtoCae));
    }
    $printer->text("Fecha Vto CAE: $fechaFormateada\n");
    
    // Debug del CAE
    file_put_contents('php://stderr', "💾 CAE procesado: " . $cae . "\n");
    file_put_contents('php://stderr', "📅 Fecha Vto CAE: " . $fechaVtoCae . " -> " . $fechaFormateada . "\n");

    // Código QR (si está disponible)
    if (isset($afipData['qrData'])) {
        $printer->text("\n");
        $printer->text("Codigo QR:\n");
        // Aquí podrías generar un QR code si tienes la librería
        $printer->text($afipData['qrData'] . "\n");
    }

    // Pie de página
    $printer->text("\n¡Gracias por su compra!\n");
    $printer->text("Conserve este comprobante\n");
    
    $printer->feed(3);
    $printer->cut();
    $printer->pulse();
    $printer->close();
    


    // Debug final
    file_put_contents('php://stderr', "✅ Impresión AFIP completada exitosamente\n");
    file_put_contents('php://stderr', "📊 ESTADÍSTICAS FINALES AFIP:\n");
    file_put_contents('php://stderr', "- Items procesados: " . (isset($afipData['items']) ? count($afipData['items']) : 0) . "\n");
    file_put_contents('php://stderr', "- Total impreso: $" . number_format($afipData['total'], 2) . "\n");
    file_put_contents('php://stderr', "- CAE: " . ($afipData['cae'] ?? 'NO DISPONIBLE') . "\n");
    file_put_contents('php://stderr', "- Business mostrado: " . $businessName . "\n");
    file_put_contents('php://stderr', "- Razón Social: " . $razonSocial . "\n");
    file_put_contents('php://stderr', "- CUIT: " . ($afipData['cuit'] ?? 'NO DISPONIBLE') . "\n");
    file_put_contents('php://stderr', "- Vendedor: " . ($afipData['vendedor'] ?? $afipData['usuario'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Tipo Factura: " . ($afipData['tipoFactura'] ?? 'N/A') . "\n");
    file_put_contents('php://stderr', "- Número: " . ($afipData['puntoVenta'] ?? '0001') . "-" . str_pad($afipData['numeroFactura'] ?? '1', 8, '0', STR_PAD_LEFT) . "\n");
    
    // Debug de la nueva estructura de configuración AFIP
    if (isset($afipData['configuracionAfip'])) {
        file_put_contents('php://stderr', "- Configuración AFIP encontrada: " . json_encode($afipData['configuracionAfip']) . "\n");
    } else {
        file_put_contents('php://stderr', "- Configuración AFIP: NO DISPONIBLE\n");
    }
    
    file_put_contents('php://stderr', "====== FIN DEBUG IMPRESIÓN TICKET AFIP ======\n");

} catch (Exception $e) {
    file_put_contents('php://stderr', "Error AFIP: " . $e->getMessage() . "\n");
    exit(1);
}
?>