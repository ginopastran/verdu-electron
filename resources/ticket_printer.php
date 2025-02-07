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
        $logo = EscposImage::load(__DIR__ . "/public/logo.png");
        $printer->bitImage($logo);
    } catch (Exception $e) {
        // Si no hay logo, continuamos sin él
    }

    // Encabezado
    $printer->setEmphasis(true);
    $printer->setTextSize(2, 2);
    $printer->text("\nVerdulería\n");
    $printer->setEmphasis(false);
    $printer->setTextSize(1, 1);
    $printer->text("Comprobante de Venta\n");
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