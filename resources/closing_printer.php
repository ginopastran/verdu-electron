<?php
require __DIR__ . '/vendor/autoload.php';

use Mike42\Escpos\Printer;
use Mike42\Escpos\PrintConnectors\WindowsPrintConnector;

try {
    file_put_contents('php://stderr', "Iniciando impresión de cierre...\n");
    
    $closingDataPath = $argv[1];
    if (!file_exists($closingDataPath)) {
        throw new Exception("Archivo de datos no encontrado: " . $closingDataPath);
    }
    
    $closingData = json_decode(file_get_contents($closingDataPath), true);
    $nombre_impresora = "TP806L";
    
    $connector = new WindowsPrintConnector($nombre_impresora);
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

    // Detalles por método de pago
    $printer->text("VENTAS POR MÉTODO DE PAGO:\n");
    $printer->text("-----------------------------\n");
    
    foreach ($closingData['ventasPorMetodo'] as $metodo => $monto) {
        $printer->text(str_pad(ucfirst($metodo), 15));
        $printer->text(str_pad('$' . number_format($monto, 2), 17, " ", STR_PAD_LEFT) . "\n");
    }

    // Total
    $printer->text("-----------------------------\n");
    $printer->setEmphasis(true);
    $printer->text(str_pad("TOTAL: $" . number_format($closingData['totalVentas'], 2), 32, " ", STR_PAD_LEFT) . "\n");
    $printer->text(str_pad("CANT. VENTAS: " . $closingData['cantidadVentas'], 32, " ", STR_PAD_LEFT) . "\n");
    $printer->setEmphasis(false);

    $printer->feed(3);
    $printer->cut();
    $printer->close();

} catch (Exception $e) {
    file_put_contents('php://stderr', "Error: " . $e->getMessage() . "\n");
    exit(1);
} 