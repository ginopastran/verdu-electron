<?php
namespace Mike42\Escpos\PrintConnectors;

\define('GOLDEN_INICIO', \time());

require \getenv('ESCPOS_VENDOR') . '/autoload.php';

class WindowsPrintConnector implements PrintConnector
{
    private $datos = '';

    public function __construct($destino)
    {
    }

    public function write($data)
    {
        $this->datos .= $data;
    }

    public function read($len)
    {
        return false;
    }

    public function finalize()
    {
        $zona = new \DateTimeZone(\date_default_timezone_get());
        $fija = (new \DateTime('2026-01-02 03:04:05', $zona))->getTimestamp();
        foreach (['Y-m-d H:i:s', 'd/m/Y H:i:s', 'd/m/Y H:i', 'Y-m-d', 'd/m/Y', 'H:i:s', 'H:i'] as $formato) {
            for ($t = GOLDEN_INICIO - 2; $t <= \time() + 1; $t++) {
                $this->datos = \str_replace(\date($formato, $t), \date($formato, $fija), $this->datos);
            }
        }
        \file_put_contents(\getenv('GOLDEN_SALIDA'), $this->datos);
    }

    public function __destruct()
    {
    }
}
