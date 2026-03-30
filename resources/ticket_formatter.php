<?php

const TICKET_TOTAL_WIDTH = 42;
const TICKET_QTY_WIDTH = 8;
const TICKET_PRICE_WIDTH = 9;
const TICKET_TOTAL_COL_WIDTH = 13;
const TICKET_SPACING_WIDTH = 3;
const TICKET_PRODUCT_WIDTH = TICKET_TOTAL_WIDTH - TICKET_QTY_WIDTH - TICKET_PRICE_WIDTH - TICKET_TOTAL_COL_WIDTH - TICKET_SPACING_WIDTH;

function ticket_strlen(string $value): int
{
    if (function_exists('mb_strlen')) {
        return mb_strlen($value, 'UTF-8');
    }

    return strlen($value);
}

function ticket_substr(string $value, int $start, ?int $length = null): string
{
    if (function_exists('mb_substr')) {
        return mb_substr($value, $start, $length, 'UTF-8');
    }

    return $length === null ? substr($value, $start) : substr($value, $start, $length);
}

function ticket_pad_right(string $value, int $width): string
{
    $padding = $width - ticket_strlen($value);
    return $padding > 0 ? $value . str_repeat(' ', $padding) : $value;
}

function ticket_pad_left(string $value, int $width): string
{
    $padding = $width - ticket_strlen($value);
    return $padding > 0 ? str_repeat(' ', $padding) . $value : $value;
}

function split_ticket_text(string $value, int $width): array
{
    $normalized = trim((string) preg_replace('/\s+/u', ' ', $value));

    if ($normalized === '') {
        return [''];
    }

    $words = preg_split('/\s+/u', $normalized) ?: [$normalized];
    $lines = [];
    $currentLine = '';

    foreach ($words as $word) {
        if ($currentLine === '') {
            $currentLine = $word;
            continue;
        }

        $candidate = $currentLine . ' ' . $word;
        if (ticket_strlen($candidate) <= $width) {
            $currentLine = $candidate;
            continue;
        }

        $lines[] = $currentLine;
        $currentLine = $word;
    }

    if ($currentLine !== '') {
        $lines[] = $currentLine;
    }

    $wrappedLines = [];
    foreach ($lines as $line) {
        if (ticket_strlen($line) <= $width) {
            $wrappedLines[] = $line;
            continue;
        }

        $offset = 0;
        while ($offset < ticket_strlen($line)) {
            $wrappedLines[] = ticket_substr($line, $offset, $width);
            $offset += $width;
        }
    }

    return $wrappedLines;
}

function format_ticket_number(float $value, int $decimals, bool $currency = false): string
{
    $formatted = number_format($value, $decimals);
    return $currency ? '$' . $formatted : $formatted;
}

function build_ticket_columns_header(): string
{
    return sprintf(
        "%s %s %s %s",
        ticket_pad_right('PRODUCTO', TICKET_PRODUCT_WIDTH),
        ticket_pad_left('CANT', TICKET_QTY_WIDTH),
        ticket_pad_left('PRECIO', TICKET_PRICE_WIDTH),
        ticket_pad_left('TOTAL', TICKET_TOTAL_COL_WIDTH)
    );
}

function build_ticket_separator(): string
{
    return str_repeat('-', TICKET_TOTAL_WIDTH);
}

function format_ticket_item_lines(string $productName, float $quantity, float $price, float $subtotal): array
{
    $nameLines = split_ticket_text($productName, TICKET_PRODUCT_WIDTH);
    $formattedLines = [];
    $lastIndex = count($nameLines) - 1;

    $quantityText = ticket_pad_left(format_ticket_number($quantity, 3), TICKET_QTY_WIDTH);
    $priceText = ticket_pad_left(format_ticket_number($price, 2, true), TICKET_PRICE_WIDTH);
    $subtotalText = ticket_pad_left(format_ticket_number($subtotal, 2, true), TICKET_TOTAL_COL_WIDTH);

    foreach ($nameLines as $index => $line) {
        if ($index === $lastIndex) {
            $formattedLines[] = sprintf(
                "%s %s %s %s",
                ticket_pad_right($line, TICKET_PRODUCT_WIDTH),
                $quantityText,
                $priceText,
                $subtotalText
            );
            continue;
        }

        $formattedLines[] = $line;
    }

    return $formattedLines;
}
