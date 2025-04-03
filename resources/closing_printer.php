<?php
require __DIR__ . '/vendor/autoload.php';

use Mike42\Escpos\Printer;
use Mike42\Escpos\PrintConnectors\WindowsPrintConnector;

try {
    file_put_contents('php://stderr', "Iniciando impresión de cierre...\n");
    
    // Recibir y validar datos
    $closingDataPath = $argv[1];
    if (!file_exists($closingDataPath)) {
        throw new Exception("Archivo de datos no encontrado: " . $closingDataPath);
    }
    
    // Cargar datos de cierre
    $closingData = json_decode(file_get_contents($closingDataPath), true);
    $nombre_impresora = "TP806L";
    
    try {
        // Intentar conectar a la impresora - esto fallará si no existe
        $connector = new WindowsPrintConnector($nombre_impresora);
        
        // Si llegamos aquí, la conexión fue exitosa
        $printer = new Printer($connector);

        // Encabezado
        $printer->setJustification(Printer::JUSTIFY_CENTER);
        $printer->setEmphasis(true);
        $printer->setTextSize(2, 2);
        $printer->text("\nCierre de Caja\n");
        $printer->setTextSize(1, 1);
        $printer->text("Periodo: " . strtoupper($closingData['periodo']) . "\n\n");
        
        // Detalles del periodo
        $printer->setJustification(Printer::JUSTIFY_LEFT);
        $printer->text("Fecha inicio: " . date("d/m/Y H:i", strtotime($closingData['fechaInicio'])) . "\n");
        $printer->text("Fecha cierre: " . date("d/m/Y H:i", strtotime($closingData['fechaCierre'])) . "\n");
        $printer->text("-----------------------------\n");

        // Estructurar los datos de métodos de pago
        if (isset($closingData['ventasPorMetodo']) && isset($closingData['ventasPorMetodo']['ventasPorMetodo'])) {
            // Nueva estructura API
            $metodosPago = $closingData['ventasPorMetodo']['ventasPorMetodo'];
        } else {
            // Estructura antigua
            $metodosPago = $closingData['ventasPorMetodo'] ?? [];
        }

        // Detalles por método de pago
        $printer->text("VENTAS POR MÉTODO DE PAGO:\n");
        $printer->text("-----------------------------\n");
        
        foreach ($metodosPago as $metodo => $monto) {
            $printer->text(str_pad(ucfirst($metodo), 15));
            $printer->text(str_pad('$' . number_format($monto, 2), 17, " ", STR_PAD_LEFT) . "\n");
        }

        // Total general
        $printer->text("-----------------------------\n");
        $printer->setEmphasis(true);
        $printer->text(str_pad("TOTAL: $" . number_format($closingData['totalVentas'], 2), 32, " ", STR_PAD_LEFT) . "\n");
        $printer->text(str_pad("CANT. VENTAS: " . $closingData['cantidadVentas'], 32, " ", STR_PAD_LEFT) . "\n");
        $printer->setEmphasis(false);
        
        // Ventas por vendedor (nuevo)
        if (isset($closingData['ventasPorMetodo']) && isset($closingData['ventasPorMetodo']['ventasPorVendedor'])) {
            $printer->text("\n\n");
            $printer->setJustification(Printer::JUSTIFY_CENTER);
            $printer->setEmphasis(true);
            $printer->text("VENTAS POR VENDEDOR\n");
            $printer->setEmphasis(false);
            $printer->text("=============================\n\n");
            $printer->setJustification(Printer::JUSTIFY_LEFT);
            
            foreach ($closingData['ventasPorMetodo']['ventasPorVendedor'] as $vendedor) {
                $printer->setEmphasis(true);
                $printer->text(strtoupper($vendedor['nombre']) . "\n");
                $printer->setEmphasis(false);
                $printer->text("Email: " . $vendedor['email'] . "\n");
                $printer->text("Total: $" . number_format($vendedor['totalVentas'], 2) . "\n");
                $printer->text("Cantidad: " . $vendedor['cantidadVentas'] . "\n");
                $printer->text("-----------------------------\n");
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