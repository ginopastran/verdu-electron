import { test } from "node:test";
import assert from "node:assert/strict";
import { pesoDeTrama, crearFiltroEstable } from "../electron/balanzaTrama.ts";

test("lee gramos enteros y kilos con decimales", () => {
  assert.equal(pesoDeTrama("001250\u0013"), 1250);
  assert.equal(pesoDeTrama("ST,GS,+00012.50kg"), 12500);
  assert.equal(pesoDeTrama(" 1,235 kg"), 1235);
  assert.equal(pesoDeTrama("000125", "kilos"), 125000);
  assert.equal(pesoDeTrama("\u0002\u0003"), null);
});

test("solo informa el peso cuando 3 lecturas seguidas difieren 2 g o menos", () => {
  const estable = crearFiltroEstable();
  assert.equal(estable(1000), null);
  assert.equal(estable(1010), null);
  assert.equal(estable(1012), null);
  assert.equal(estable(1011), 1011);
  assert.equal(estable(1500), null);
});
