<?php

require_once __DIR__ . '/ticket_formatter.php';

function fail_test(string $message): void
{
    fwrite(STDERR, $message . PHP_EOL);
    exit(1);
}

function assert_true(bool $condition, string $message): void
{
    if (!$condition) {
        fail_test($message);
    }
}

$shortLines = format_ticket_item_lines('Banana', 2, 1500.5, 3001);
assert_true(count($shortLines) === 1, 'Un nombre corto debe ocupar una sola linea');
assert_true(str_contains($shortLines[0], 'Banana'), 'El nombre corto debe mantenerse visible');
assert_true(str_contains($shortLines[0], '$1,500.50'), 'El precio corto debe figurar');

$longLines = format_ticket_item_lines('Maple de huevos blancos grandes', 240, 4666.67, 1120000.8);
assert_true(count($longLines) >= 2, 'Un nombre largo debe partirse en varias lineas');
assert_true(!str_contains($longLines[0], '$4,666.67'), 'Las lineas intermedias no deben incluir valores');
assert_true(str_contains($longLines[count($longLines) - 1], '$4,666.67'), 'La ultima linea debe incluir el precio');
assert_true(str_contains($longLines[count($longLines) - 1], '$1,120,000.80'), 'La ultima linea debe incluir el subtotal');
assert_true(str_contains(implode(' ', array_map('trim', $longLines)), 'Maple de huevos blancos grandes'), 'El nombre largo no debe truncarse');

$header = build_ticket_columns_header();
assert_true(str_contains($header, 'PRODUCTO'), 'El encabezado debe incluir PRODUCTO');
assert_true(str_contains($header, 'CANT'), 'El encabezado debe incluir CANT');
assert_true(str_contains($header, 'PRECIO'), 'El encabezado debe incluir PRECIO');
assert_true(str_contains($header, 'TOTAL'), 'El encabezado debe incluir TOTAL');

echo "ticket formatter ok" . PHP_EOL;
