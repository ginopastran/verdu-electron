<?php
require __DIR__ . '/vendor/autoload.php';

// Verificar la estructura de archivos
$paths = [
    'autoloader' => __DIR__ . '/vendor/autoload.php',
    'connector_class' => __DIR__ . '/vendor/mike42/escpos-php/src/Mike42/Escpos/PrintConnectors/WindowsPrintConnector.php',
];

echo "Verificando rutas:\n";
foreach ($paths as $name => $path) {
    echo "$name: " . (file_exists($path) ? "✓ Existe" : "✗ No existe") . " ($path)\n";
}

// Listar impresoras instaladas
echo "\nImpresoras instaladas:\n";
try {
    $wmi = new COM('WinMgmts:\\\\.');
    $printers = $wmi->InstancesOf('Win32_Printer');
    
    foreach ($printers as $printer) {
        echo "- " . $printer->Name . "\n";
    }
} catch (Exception $e) {
    echo "Error al listar impresoras: " . $e->getMessage() . "\n";
}

// Verificar que la clase existe
echo "\nVerificando clase WindowsPrintConnector:\n";
if (class_exists('Mike42\Escpos\PrintConnectors\WindowsPrintConnector', true)) {
    echo "✓ La clase existe\n";
} else {
    echo "✗ La clase no existe\n";
    
    // Mostrar clases cargadas
    echo "\nClases cargadas:\n";
    $classes = get_declared_classes();
    foreach ($classes as $class) {
        if (strpos($class, 'Mike42') !== false) {
            echo "- $class\n";
        }
    }
}

// Mostrar versión de PHP
echo "\nVersión de PHP: " . PHP_VERSION . "\n";

try {
    // Datos de prueba
    $orderData = [
        'items' => [
            [
                'nombre' => 'Manzana',
                'cantidad' => 2,
                'precioHistorico' => 100,
                'subtotal' => 200
            ]
        ],
        'total' => 200,
        'metodoPago' => 'efectivo'
    ];

    // Guardar datos en un archivo temporal
    $tempFile = __DIR__ . '/temp_order.json';
    file_put_contents($tempFile, json_encode($orderData));

    // Ejecutar el script de impresión
    $output = shell_exec(sprintf('php "%s/ticket_printer.php" "%s"', __DIR__, $tempFile));
    
    // Limpiar archivo temporal
    unlink($tempFile);

    echo "Resultado: " . $output;

} catch (Exception $e) {
    echo "Error: " . $e->getMessage();
} 