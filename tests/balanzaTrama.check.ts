import { test } from "node:test";
import assert from "node:assert/strict";
import { pesoDeTrama, crearFiltroEstable, crearLectorTramas } from "../electron/balanzaTrama.ts";

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

const tramaSystel = (peso: string) => {
  const cuerpo = Buffer.from([0x02, ...Buffer.from(peso, "latin1"), 0x03]);
  return Buffer.from([...cuerpo, cuerpo.reduce((a, b) => a ^ b, 0)]);
};

test("lee tramas STX/ETX con CRC de Systel y detecta las corruptas una vez aprendido el CRC", () => {
  const leer = crearLectorTramas();
  for (let i = 0; i < 3; i++) assert.deepEqual(leer(tramaSystel("01.250")), [{ texto: "01.250", crc: "ok" }]);
  const corrupta = tramaSystel("01.250");
  corrupta[corrupta.length - 1] ^= 0xff;
  assert.deepEqual(leer(corrupta), [{ texto: "01.250", crc: "mal" }]);
  const partida = tramaSystel("00.800");
  assert.deepEqual(leer(partida.subarray(0, 3)), []);
  assert.deepEqual(leer(partida.subarray(3)), [{ texto: "00.800", crc: "ok" }]);
});

test("lee balanzas sin CRC: STX/ETX sueltos, lineas y tramas cortadas por silencio", () => {
  const leer = crearLectorTramas();
  assert.deepEqual(leer(Buffer.from("\u000200.500\u0003\u000200.600\u0003", "latin1")), [{ texto: "00.500", crc: "sin" }]);
  assert.deepEqual(leer(Buffer.alloc(0), true), [{ texto: "00.600", crc: "sin" }]);
  assert.deepEqual(leer(Buffer.from("ST,GS,+00012.50kg\r\n", "latin1")).map((t) => t.texto), ["ST,GS,+00012.50kg", ""]);
  assert.deepEqual(leer(Buffer.from("001250", "latin1")), []);
  assert.deepEqual(leer(Buffer.alloc(0), true), [{ texto: "001250", crc: "sin" }]);
});
