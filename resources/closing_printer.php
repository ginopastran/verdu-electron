<?php
require __DIR__ . '/vendor/autoload.php';

use Mike42\Escpos\Printer;
use Mike42\Escpos\EscposImage;
use Mike42\Escpos\PrintConnectors\WindowsPrintConnector;
use Mike42\Escpos\PrintConnectors\DummyPrintConnector;

try {
    file_put_contents('php://stderr', "==== INICIO DE CLOSING PRINTER ====\n");
    
    // Establecer zona horaria para Argentina
    date_default_timezone_set('America/Argentina/Buenos_Aires');
    
    // Recibir y validar datos
    $closingDataPath = $argv[1];
    if (!file_exists($closingDataPath)) {
        throw new Exception("Archivo de datos no encontrado: " . $closingDataPath);
    }
    
    // Guardar los datos raw recibidos como referencia
    file_put_contents('php://stderr', "Ruta de datos: " . $closingDataPath . "\n");
    $rawData = file_get_contents($closingDataPath);
    file_put_contents('php://stderr', "DATOS JSON RECIBIDOS: " . $rawData . "\n");
    
    // Cargar datos de cierre
    $closingData = json_decode($rawData, true);
    if (json_last_error() !== JSON_ERROR_NONE) {
        throw new Exception("Error decodificando JSON: " . json_last_error_msg());
    }
    
    file_put_contents('php://stderr', "ESTRUCTURA DE DATOS DECODIFICADA:\n");
    file_put_contents('php://stderr', "totalVentas: " . ($closingData['totalVentas'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "cantidadVentas: " . ($closingData['cantidadVentas'] ?? 'NO DEFINIDO') . "\n");
    file_put_contents('php://stderr', "ventasPorMetodo existe: " . (isset($closingData['ventasPorMetodo']) ? 'SÍ' : 'NO') . "\n");
    file_put_contents('php://stderr', "ventas.cuentaCorriente existe: " . (isset($closingData['ventas']['cuentaCorriente']) ? 'SÍ' : 'NO') . "\n");
    file_put_contents('php://stderr', "ventas.resumen existe: " . (isset($closingData['ventas']['resumen']) ? 'SÍ' : 'NO') . "\n");
    
    if (isset($closingData['ventasPorMetodo'])) {
        if (isset($closingData['ventasPorMetodo']['ventasPorMetodo'])) {
            file_put_contents('php://stderr', "MÉTODOS DISPONIBLES: " . json_encode($closingData['ventasPorMetodo']['ventasPorMetodo']) . "\n");
        } else {
            file_put_contents('php://stderr', "ventasPorMetodo['ventasPorMetodo'] NO EXISTE\n");
            file_put_contents('php://stderr', "Estructura completa de ventasPorMetodo: " . json_encode($closingData['ventasPorMetodo']) . "\n");
        }
    }
    
    $nombre_impresora = "TP806L";
    
    // Detectar modo simulación (pasar --simulate como segundo argumento)
    $simulate = in_array("--simulate", $argv, true);
    if ($simulate) {
        file_put_contents('php://stderr', "⚙️  Modo simulación activado - no se enviará a impresora física\n");
    }
    
    try {
        // Intentar conectar a la impresora - esto fallará si no existe
        if ($simulate) {
            $connector = new DummyPrintConnector();
        } else {
            $connector = new WindowsPrintConnector($nombre_impresora);
        }
        
        // Si llegamos aquí, la conexión fue exitosa
        $printer = new Printer($connector);

        // Logo (opcional)
        try {
            file_put_contents('php://stderr', "==== DEPURACIÓN AVANZADA LOGO - CIERRE ====\n");
            
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
                                    $tempPath = sys_get_temp_dir() . "/temp_logo_closing_" . uniqid() . ".png";
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

        // Encabezado simplificado - solo "Cierre de Caja"
        $printer->setJustification(Printer::JUSTIFY_CENTER);
        $printer->setEmphasis(true);
        $printer->setTextSize(2, 2);
        $printer->text("Cierre de Caja\n");
        $printer->setTextSize(1, 1);
        // Formatear el período para mostrar un texto más descriptivo
        $periodoTexto = $closingData['periodo'];
        if ($periodoTexto === 'custom') {
            $periodoTexto = 'MANUAL';
        } else {
            $periodoTexto = strtoupper($periodoTexto);
        }
        $printer->text("Periodo: " . $periodoTexto . "\n\n");
        
        // Detalles del periodo
        $printer->setJustification(Printer::JUSTIFY_LEFT);
        
        // Manejar las fechas con zona horaria
        $fechaInicio = new DateTime($closingData['fechaInicio'], new DateTimeZone('America/Argentina/Buenos_Aires'));
        $fechaCierre = new DateTime($closingData['fechaCierre'], new DateTimeZone('America/Argentina/Buenos_Aires'));
        
        // Para cierres custom, mostrar las fechas exactas sin modificar
        if ($closingData['periodo'] === 'custom') {
            $printer->text("Desde: " . $fechaInicio->format("d/m/Y H:i") . "\n");
            $printer->text("Hasta: " . $fechaCierre->format("d/m/Y H:i") . "\n");
        } else {
            // Para cierres normales, mantener el comportamiento original
            $fechaInicio->setTime(0, 0); // Establecer hora a 00:00 para cierres normales
            $printer->text("Fecha inicio: " . $fechaInicio->format("d/m/Y H:i") . "\n");
            $printer->text("Fecha cierre: " . $fechaCierre->format("d/m/Y H:i") . "\n");
        }
        $printer->text("-----------------------------\n");

        // Estructurar los datos de métodos de pago - Sección crítica
        file_put_contents('php://stderr', "==== DESERIALIZACIÓN DE MÉTODOS DE PAGO ====\n");
        
        // Imprimir para verificar estructura
        file_put_contents('php://stderr', "Estructura de closingData:\n" . print_r($closingData, true) . "\n");
        
        if (isset($closingData['ventasPorMetodo']) && isset($closingData['ventasPorMetodo']['ventasPorMetodo'])) {
            // Nueva estructura API
            $metodosPago = $closingData['ventasPorMetodo']['ventasPorMetodo'];
            file_put_contents('php://stderr', "✅ Estructura API nueva detectada\n");
            file_put_contents('php://stderr', "Métodos encontrados: " . json_encode($metodosPago) . "\n");
            
            // Verificar tipo y estructura
            if (is_array($metodosPago)) {
                file_put_contents('php://stderr', "✅ ventasPorMetodo es un array correcto\n");
            } else {
                file_put_contents('php://stderr', "❌ ventasPorMetodo NO es un array: " . gettype($metodosPago) . "\n");
                
                // Intentar corregir si es un objeto o string
                if (is_object($metodosPago)) {
                    file_put_contents('php://stderr', "Intentando convertir objeto a array...\n");
                    $metodosPago = (array)$metodosPago;
                } else if (is_string($metodosPago)) {
                    file_put_contents('php://stderr', "Intentando parsear string JSON...\n");
                    $metodosPago = json_decode($metodosPago, true);
                }
            }
        } else {
            // Estructura antigua o diferente
            if (isset($closingData['ventasPorMetodo'])) {
                if (is_array($closingData['ventasPorMetodo'])) {
                    $metodosPago = $closingData['ventasPorMetodo'];
                    file_put_contents('php://stderr', "⚠️ Usando estructura alternativa directa\n");
                } else {
                    file_put_contents('php://stderr', "❌ ventasPorMetodo existe pero NO es un array: " . gettype($closingData['ventasPorMetodo']) . "\n");
                    $metodosPago = [];
                }
            } else {
                file_put_contents('php://stderr', "❌ No se encontró ninguna estructura de ventasPorMetodo\n");
                $metodosPago = [];
            }
        }
        
        // Intentar reconstruir del _debug si está disponible (último recurso)
        if (empty($metodosPago) && isset($closingData['_debug']) && isset($closingData['_debug']['originalData'])) {
            file_put_contents('php://stderr', "Intentando recuperar métodos de pago desde debug data...\n");
            $debugData = $closingData['_debug']['originalData'];
            if (isset($debugData['ventasPorMetodo'])) {
                $metodosPago = $debugData['ventasPorMetodo'];
                file_put_contents('php://stderr', "Datos recuperados de debug: " . json_encode($metodosPago) . "\n");
            }
        }
        
        file_put_contents('php://stderr', "Métodos de pago finales: " . json_encode($metodosPago) . "\n");
        
        // Asegurar que siempre existan los tres métodos de pago principales
        $metodosCompletos = [
            'efectivo' => isset($metodosPago['efectivo']) ? $metodosPago['efectivo'] : 0,
            'tarjeta' => isset($metodosPago['tarjeta']) ? $metodosPago['tarjeta'] : 0,
            'qr' => isset($metodosPago['qr']) ? $metodosPago['qr'] : 0
        ];

        // Agregar otros métodos que puedan existir pero no son estándar
        foreach ($metodosPago as $metodo => $monto) {
            if (!array_key_exists($metodo, $metodosCompletos)) {
                $metodosCompletos[$metodo] = $monto;
            }
        }

        // Usar la lista completa para los cálculos y visualización
        $metodosPago = $metodosCompletos;

        // Verificar suma de totales
        $sumaPorMetodos = 0;
        foreach ($metodosPago as $metodo => $monto) {
            $sumaPorMetodos += $monto;
            file_put_contents('php://stderr', "Método: $metodo - Monto: $monto\n");
        }
        file_put_contents('php://stderr', "Suma por métodos: $sumaPorMetodos\n");
        file_put_contents('php://stderr', "Total general: {$closingData['totalVentas']}\n");

        // Comprobar si hay discrepancia (solo para log)
        $diferencia = $closingData['totalVentas'] - $sumaPorMetodos;
        if (abs($diferencia) > 0.01) {
            file_put_contents('php://stderr', "⚠️ ADVERTENCIA: Hay una diferencia de $diferencia entre la suma de métodos y el total general\n");
        }

        // Detalles por método de pago - Solo mostrar si hay métodos con valores > 0
        $tieneMetodosPago = false;
        foreach ($metodosPago as $metodo => $monto) {
            if ($monto > 0) {
                $tieneMetodosPago = true;
                break;
            }
        }
        
        if ($tieneMetodosPago) {
            $printer->text("VENTAS POR MÉTODO DE PAGO:\n");
            $printer->text("-----------------------------\n");
            
            // Para depuración - mostrar lo que realmente hay después de procesamiento
            file_put_contents('php://stderr', "MÉTODOS DE PAGO PARA IMPRESIÓN (después de procesamiento):\n");
            foreach ($metodosPago as $metodo => $monto) {
                file_put_contents('php://stderr', "  " . $metodo . ": " . $monto . "\n");
            }
            
            // Forzar el orden QR, Tarjeta, Efectivo (solo los que tengan valor > 0)
            $metodosOrdenados = array(
                'qr'       => isset($metodosPago['qr']) ? $metodosPago['qr'] : 0,
                'tarjeta'  => isset($metodosPago['tarjeta']) ? $metodosPago['tarjeta'] : 0,
                'efectivo' => isset($metodosPago['efectivo']) ? $metodosPago['efectivo'] : 0
            );
            
            // Imprimir métodos con valores positivos
            foreach ($metodosOrdenados as $metodo => $monto) {
                if ($monto > 0) {
                    $nombreFormateado = ucfirst($metodo); // Primera letra en mayúscula
                    $printer->text(str_pad($nombreFormateado, 15));
                    $printer->text(str_pad('$' . number_format($monto, 2), 17, " ", STR_PAD_LEFT) . "\n");
                    file_put_contents('php://stderr', "✓ Imprimiendo método: " . $metodo . " - $" . number_format($monto, 2) . "\n");
                }
            }
            
            // Imprimir otros métodos que no estén en la lista predefinida
            foreach ($metodosPago as $metodo => $monto) {
                if (!array_key_exists($metodo, $metodosOrdenados) && $monto > 0) {
                    $nombreFormateado = ucfirst($metodo);
                    $printer->text(str_pad($nombreFormateado, 15));
                    $printer->text(str_pad('$' . number_format($monto, 2), 17, " ", STR_PAD_LEFT) . "\n");
                    file_put_contents('php://stderr', "✓ Imprimiendo método adicional: " . $metodo . " - $" . number_format($monto, 2) . "\n");
                }
            }
        }
        
        // NO mostrar línea de diferencia, simplemente ir al total

        // Inicializar variables de cuenta corriente (para usar después)
        $totalCuentaCorriente = 0;
        $cantidadCuentaCorriente = 0;
        $metodosPagoCC = [];
        
        // Usar nueva estructura 'ventas.cuentaCorriente' de la API
        if (isset($closingData['ventas']['cuentaCorriente']) && is_array($closingData['ventas']['cuentaCorriente'])) {
            $cuentaCorriente = $closingData['ventas']['cuentaCorriente'];
            $totalCuentaCorriente = isset($cuentaCorriente['total']) ? floatval($cuentaCorriente['total']) : 0;
            $cantidadCuentaCorriente = isset($cuentaCorriente['cantidad']) ? intval($cuentaCorriente['cantidad']) : 0;
            
            // Obtener métodos de pago de cuenta corriente si están disponibles
            if (isset($cuentaCorriente['ventasPorMetodo']) && is_array($cuentaCorriente['ventasPorMetodo'])) {
                $metodosPagoCC = $cuentaCorriente['ventasPorMetodo'];
            }
        }
        // Compatibilidad con estructura antigua (nivel raíz)
        elseif (isset($closingData['cuentaCorriente']) && is_array($closingData['cuentaCorriente'])) {
            $cuentaCorriente = $closingData['cuentaCorriente'];
            $totalCuentaCorriente = isset($cuentaCorriente['total']) ? floatval($cuentaCorriente['total']) : 0;
            $cantidadCuentaCorriente = isset($cuentaCorriente['cantidad']) ? intval($cuentaCorriente['cantidad']) : 0;
            
            if (isset($cuentaCorriente['ventasPorMetodo']) && is_array($cuentaCorriente['ventasPorMetodo'])) {
                $metodosPagoCC = $cuentaCorriente['ventasPorMetodo'];
            }
        }
        // Compatibilidad con estructura antigua 'ventasCuentaCorriente'
        elseif (isset($closingData['ventasCuentaCorriente'])) {
            $ventasCC = $closingData['ventasCuentaCorriente'];
            $totalCuentaCorriente = isset($ventasCC['total']) ? floatval($ventasCC['total']) : 0;
            $cantidadCuentaCorriente = isset($ventasCC['cantidad']) ? intval($ventasCC['cantidad']) : 0;
        }

        // Total general (ventas normales)
        $printer->text("-----------------------------\n");
        $printer->setEmphasis(true);
        $printer->text(str_pad("TOTAL VENTAS NORMALES: $" . number_format($closingData['totalVentas'], 2), 32, " ", STR_PAD_LEFT) . "\n");
        $printer->text(str_pad("CANT. VENTAS NORMALES: " . $closingData['cantidadVentas'], 32, " ", STR_PAD_LEFT) . "\n");
        $printer->setEmphasis(false);
        
        // ✅ Sección de ventas por cuenta corriente (solo totales, sin lista detallada)
        if ($totalCuentaCorriente > 0 && $cantidadCuentaCorriente > 0) {
            $printer->text("-----------------------------\n");
            $printer->setJustification(Printer::JUSTIFY_CENTER);
            $printer->setEmphasis(true);
            $printer->text("VENTAS POR CUENTA CORRIENTE\n");
            $printer->setEmphasis(false);
            $printer->setJustification(Printer::JUSTIFY_LEFT);
            $printer->text("-----------------------------\n");
            
            // Validar si hay métodos de pago de cuenta corriente con valores > 0
            $tieneMetodosPagoCC = false;
            if (!empty($metodosPagoCC) && is_array($metodosPagoCC)) {
                foreach ($metodosPagoCC as $metodo => $monto) {
                    if ($monto > 0) {
                        $tieneMetodosPagoCC = true;
                        break;
                    }
                }
            }
            
            // Mostrar métodos de pago de cuenta corriente solo si hay métodos con valores > 0
            if ($tieneMetodosPagoCC) {
                $printer->text("MÉTODOS DE PAGO:\n");
                foreach ($metodosPagoCC as $metodo => $monto) {
                    if ($monto > 0) {
                        $nombreFormateado = ucfirst(str_replace('_', ' ', $metodo));
                        $printer->text(str_pad($nombreFormateado, 15));
                        $printer->text(str_pad('$' . number_format($monto, 2), 17, " ", STR_PAD_LEFT) . "\n");
                    }
                }
                $printer->text("-----------------------------\n");
            }
            
            // Obtener ventas por vendedor de cuenta corriente si están disponibles
            $ventasPorVendedorCC = [];
            if (isset($closingData['ventas']['cuentaCorriente']['ventasPorVendedor']) && is_array($closingData['ventas']['cuentaCorriente']['ventasPorVendedor'])) {
                $ventasPorVendedorCC = $closingData['ventas']['cuentaCorriente']['ventasPorVendedor'];
            } elseif (isset($closingData['cuentaCorriente']['ventasPorVendedor']) && is_array($closingData['cuentaCorriente']['ventasPorVendedor'])) {
                $ventasPorVendedorCC = $closingData['cuentaCorriente']['ventasPorVendedor'];
            }
            
            // Validar que haya vendedores con datos válidos antes de mostrar
            $vendedoresCCValidos = [];
            if (!empty($ventasPorVendedorCC) && is_array($ventasPorVendedorCC)) {
                foreach ($ventasPorVendedorCC as $vendedor) {
                    // Validar que el vendedor tenga nombre y total válido
                    if (isset($vendedor['nombre']) && !empty($vendedor['nombre']) && 
                        isset($vendedor['totalVentas']) && floatval($vendedor['totalVentas']) > 0) {
                        $vendedoresCCValidos[] = $vendedor;
                    }
                }
            }
            
            // Mostrar ventas por vendedor de cuenta corriente solo si hay vendedores válidos
            if (!empty($vendedoresCCValidos)) {
                $printer->text("VENTAS POR VENDEDOR:\n");
                foreach ($vendedoresCCValidos as $vendedor) {
                    $printer->setEmphasis(true);
                    $printer->text(strtoupper($vendedor['nombre']) . "\n");
                    $printer->setEmphasis(false);
                    
                    // Mostrar email solo si existe y no está vacío
                    if (isset($vendedor['email']) && !empty($vendedor['email'])) {
                        $printer->text("Email: " . $vendedor['email'] . "\n");
                    }
                    
                    // Mostrar métodos de pago solo si existen y tienen valores > 0
                    $tieneMetodosVendedor = false;
                    if (isset($vendedor['metodosPago']) && is_array($vendedor['metodosPago'])) {
                        $metodosPago = $vendedor['metodosPago'];
                        if (is_string($metodosPago) && substr($metodosPago, 0, 1) === '{') {
                            $metodosPago = json_decode($metodosPago, true);
                        }
                        
                        foreach ($metodosPago as $metodo => $monto) {
                            if ($monto > 0) {
                                $tieneMetodosVendedor = true;
                                break;
                            }
                        }
                        
                        if ($tieneMetodosVendedor) {
                            foreach ($metodosPago as $metodo => $monto) {
                                if ($monto > 0) {
                                    $nombreFormateado = ucfirst(str_replace('_', ' ', $metodo));
                                    $printer->text("$nombreFormateado: $" . number_format(floatval($monto), 2) . "\n");
                                }
                            }
                        }
                    }
                    
                    $printer->text("Total: $" . number_format($vendedor['totalVentas'], 2) . "\n");
                    if (isset($vendedor['cantidadVentas'])) {
                        $printer->text("Cantidad: " . $vendedor['cantidadVentas'] . "\n");
                    }
                    $printer->text("-----------------------------\n");
                }
            }
            
            $printer->text(str_pad("TOTAL CUENTA CORRIENTE: $" . number_format($totalCuentaCorriente, 2), 32, " ", STR_PAD_LEFT) . "\n");
            $printer->text(str_pad("CANT. VENTAS CC: " . $cantidadCuentaCorriente, 32, " ", STR_PAD_LEFT) . "\n");
            $printer->text("-----------------------------\n");
        }
        
        // ✅ Sección combinada (cuenta corriente + ventas normales) solo si hay ventas de cuenta corriente
        if ($totalCuentaCorriente > 0) {
            // Usar resumen.totalCombinado si está disponible (nueva estructura ventas.resumen)
            $totalCombinado = isset($closingData['ventas']['resumen']['totalCombinado']) 
                ? floatval($closingData['ventas']['resumen']['totalCombinado'])
                : (isset($closingData['resumen']['totalCombinado']) 
                    ? floatval($closingData['resumen']['totalCombinado'])
                    : $closingData['totalVentas'] + $totalCuentaCorriente);
            
            $cantidadCombinada = isset($closingData['ventas']['resumen']['cantidadTotalCombinada'])
                ? intval($closingData['ventas']['resumen']['cantidadTotalCombinada'])
                : (isset($closingData['resumen']['cantidadTotalCombinada'])
                    ? intval($closingData['resumen']['cantidadTotalCombinada'])
                    : $closingData['cantidadVentas'] + $cantidadCuentaCorriente);
            
            $printer->text("-----------------------------\n");
            $printer->setJustification(Printer::JUSTIFY_CENTER);
            $printer->setEmphasis(true);
            $printer->text("TOTAL GENERAL\n");
            $printer->setEmphasis(false);
            $printer->setJustification(Printer::JUSTIFY_LEFT);
            $printer->text("(Cuenta Corriente + Ventas Normales)\n");
            $printer->text("-----------------------------\n");
            $printer->setEmphasis(true);
            $printer->text(str_pad("TOTAL GENERAL: $" . number_format($totalCombinado, 2), 32, " ", STR_PAD_LEFT) . "\n");
            $printer->text(str_pad("CANT. TOTAL: " . $cantidadCombinada, 32, " ", STR_PAD_LEFT) . "\n");
            $printer->setEmphasis(false);
        }
        
        // Ventas por vendedor (nuevo) - Solo mostrar si hay vendedores válidos
        if (isset($closingData['ventasPorVendedor']) && is_array($closingData['ventasPorVendedor']) && !empty($closingData['ventasPorVendedor'])) {
            // Validar que haya al menos un vendedor con datos válidos
            $vendedoresValidos = [];
            foreach ($closingData['ventasPorVendedor'] as $vendedor) {
                // Validar que el vendedor tenga nombre y total válido
                if (isset($vendedor['nombre']) && !empty($vendedor['nombre']) && 
                    isset($vendedor['totalVentas']) && floatval($vendedor['totalVentas']) >= 0) {
                    $vendedoresValidos[] = $vendedor;
                }
            }
            
            // Solo mostrar la sección si hay vendedores válidos
            if (!empty($vendedoresValidos)) {
                file_put_contents('php://stderr', "=== DEPURACIÓN AVANZADA VENTAS POR VENDEDOR ===\n");
                file_put_contents('php://stderr', "Estructura completa de ventasPorVendedor: " . json_encode($closingData['ventasPorVendedor']) . "\n");
                
                $printer->text("\n\n");
                $printer->setJustification(Printer::JUSTIFY_CENTER);
                $printer->setEmphasis(true);
                $printer->text("VENTAS POR VENDEDOR\n");
                $printer->setEmphasis(false);
                $printer->text("=============================\n\n");
                $printer->setJustification(Printer::JUSTIFY_LEFT);
                
                foreach ($vendedoresValidos as $vendedor) {
                    // Añadir log para ver la estructura completa del vendedor
                    file_put_contents('php://stderr', "ESTRUCTURA COMPLETA DEL VENDEDOR: " . json_encode($vendedor) . "\n");
                    
                    $printer->setEmphasis(true);
                    $printer->text(strtoupper($vendedor['nombre']) . "\n");
                    $printer->setEmphasis(false);
                    
                    // Mostrar email solo si existe y no está vacío
                    if (isset($vendedor['email']) && !empty($vendedor['email'])) {
                        $printer->text("Email: " . $vendedor['email'] . "\n");
                    }
                    
                    // Validar si hay métodos de pago con valores > 0
                    $tieneMetodosVendedor = false;
                    $metodosPagoVendedor = [];
                    
                    // Intentar acceder a los datos con diferentes formatos posibles
                    if (isset($vendedor['metodosPago'])) {
                        // Si existe metodosPago como objeto
                        file_put_contents('php://stderr', "USANDO metodosPago: " . json_encode($vendedor['metodosPago']) . "\n");
                        
                        // Si es un objeto JSON en string, decodificarlo
                        $metodosPago = $vendedor['metodosPago'];
                        if (is_string($metodosPago) && substr($metodosPago, 0, 1) === '{') {
                            $metodosPago = json_decode($metodosPago, true);
                        }
                        
                        if (is_array($metodosPago)) {
                            $metodosPagoVendedor = $metodosPago;
                            foreach ($metodosPago as $metodo => $monto) {
                                if ($monto > 0) {
                                    $tieneMetodosVendedor = true;
                                    break;
                                }
                            }
                        }
                    } else {
                        // Imprimir directamente de las propiedades del vendedor
                        file_put_contents('php://stderr', "USANDO PROPIEDADES DIRECTAS\n");
                        
                        // Construir array de métodos desde propiedades directas
                        if (isset($vendedor['qr']) && $vendedor['qr'] > 0) {
                            $metodosPagoVendedor['qr'] = $vendedor['qr'];
                            $tieneMetodosVendedor = true;
                        }
                        if (isset($vendedor['tarjeta']) && $vendedor['tarjeta'] > 0) {
                            $metodosPagoVendedor['tarjeta'] = $vendedor['tarjeta'];
                            $tieneMetodosVendedor = true;
                        }
                        if (isset($vendedor['efectivo']) && $vendedor['efectivo'] > 0) {
                            $metodosPagoVendedor['efectivo'] = $vendedor['efectivo'];
                            $tieneMetodosVendedor = true;
                        }
                    }
                    
                    // Mostrar métodos de pago solo si hay métodos con valores > 0
                    if ($tieneMetodosVendedor && !empty($metodosPagoVendedor)) {
                        // Mostrar en orden: QR, Tarjeta, Efectivo
                        if (isset($metodosPagoVendedor['qr']) && $metodosPagoVendedor['qr'] > 0) {
                            $printer->text("QR: $" . number_format(floatval($metodosPagoVendedor['qr']), 2) . "\n");
                        }
                        if (isset($metodosPagoVendedor['tarjeta']) && $metodosPagoVendedor['tarjeta'] > 0) {
                            $printer->text("Tarjeta: $" . number_format(floatval($metodosPagoVendedor['tarjeta']), 2) . "\n");
                        }
                        if (isset($metodosPagoVendedor['efectivo']) && $metodosPagoVendedor['efectivo'] > 0) {
                            $printer->text("Efectivo: $" . number_format(floatval($metodosPagoVendedor['efectivo']), 2) . "\n");
                        }
                        
                        // Mostrar otros métodos que no estén en la lista predefinida
                        foreach ($metodosPagoVendedor as $metodo => $monto) {
                            if (!in_array($metodo, ['qr', 'tarjeta', 'efectivo']) && $monto > 0) {
                                $nombreFormateado = ucfirst(str_replace('_', ' ', $metodo));
                                $printer->text("$nombreFormateado: $" . number_format(floatval($monto), 2) . "\n");
                            }
                        }
                    }
                    
                    // Mostrar total solo si existe y es válido
                    if (isset($vendedor['totalVentas'])) {
                        $printer->text("Total: $" . number_format(floatval($vendedor['totalVentas']), 2) . "\n");
                    }
                    
                    // Usar totalVentasConCuentaCorriente del backend si existe, sino usar totalVentas como fallback
                    $totalConCuentaCorriente = isset($vendedor['totalVentasConCuentaCorriente']) 
                        ? floatval($vendedor['totalVentasConCuentaCorriente']) 
                        : (isset($vendedor['totalVentas']) ? floatval($vendedor['totalVentas']) : 0);
                    
                    $printer->text("Total con CC: $" . number_format($totalConCuentaCorriente, 2) . "\n");
                    
                    // Mostrar cantidad solo si existe
                    if (isset($vendedor['cantidadVentas'])) {
                        $printer->text("Cantidad: " . $vendedor['cantidadVentas'] . "\n");
                    }
                    
                    $printer->text("-----------------------------\n");
                }
            }
        }

        // Pie de página
        $printer->setJustification(Printer::JUSTIFY_CENTER);
        $printer->text("\n\n" . date("d/m/Y H:i:s") . "\n");
        $printer->text("Gracias por su trabajo\n");

        $printer->feed(3);
        $printer->cut();
        $printer->close();
        
        file_put_contents('php://stderr', "✓ Ticket de cierre impreso correctamente\n");
        echo "Cierre impreso correctamente";

        // Cerrar la impresora y, si es simulación, volcar datos al stdout
        $printer->close();

        if ($simulate && isset($connector) && $connector instanceof DummyPrintConnector) {
            // Mostrar el texto RAW que se enviaría
            echo "===== SIMULACIÓN DE IMPRESIÓN =====\n";
            echo $connector->getData();
            echo "===== FIN SIMULACIÓN =====\n";
        }

    } catch (Exception $printerError) {
        // Error específico de la impresora - reportarlo pero no interrumpir el proceso
        file_put_contents('php://stderr', "Error con la impresora: " . $printerError->getMessage() . "\n");
        
        // Proporcionar detalles del error en la salida
        if (strpos($printerError->getMessage(), "Failed to open printer") !== false) {
            file_put_contents('php://stderr', "La impresora '" . $nombre_impresora . "' no está disponible o no existe\n");
        } else if (strpos($printerError->getMessage(), "Access denied") !== false) {
            file_put_contents('php://stderr', "Acceso denegado a la impresora. Ejecute como administrador\n");
        }
        
        echo "Cierre registrado pero no se pudo imprimir el ticket";
    }

    // Cierre exitoso (aunque quizás sin impresión)
    exit(0);
    
} catch (Exception $e) {
    // Error general no relacionado con la impresora
    file_put_contents('php://stderr', "Error general: " . $e->getMessage() . "\n");
    echo "Error: " . $e->getMessage();
    exit(1);
} 
?> 