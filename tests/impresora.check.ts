import { test } from "node:test";
import assert from "node:assert/strict";
import { esRecursoValido, recursoParaImpresora } from "../electron/impresora.ts";

test("el recurso compartido queda con un nombre que acepta escpos-php", () => {
  assert.equal(recursoParaImpresora("TP806L"), "TP806L");
  assert.equal(recursoParaImpresora("EPSON TM-T20II Receipt"), "EPSON TM-T20II Receipt");
  assert.equal(recursoParaImpresora("POS-80C (copia 1)"), "POS-80C copia 1");
  assert.equal(recursoParaImpresora("Impresora térmica"), "Impresora termica");
  assert.equal(recursoParaImpresora("((("), "POS");
  for (const nombre of ["POS-80C (copia 1)", "Impresora térmica", "  a  b  ", "x".repeat(200) + " y"]) {
    assert.ok(esRecursoValido(recursoParaImpresora(nombre)), nombre);
  }
});

test("detecta recursos que escpos-php rechaza", () => {
  assert.ok(esRecursoValido("TP806L"));
  assert.ok(!esRecursoValido("POS (1)"));
  assert.ok(!esRecursoValido("doble  espacio"));
});
