import { test } from "node:test";
import assert from "node:assert/strict";
import {
  camposDeOferta,
  consultaDeEsperado,
  datosDeCierre,
  descuentoEfectivoDisponible,
  diferenciaDeCaja,
  efectivoEsperado,
  evaluarCredito,
  leerMonto,
  montoACuentaCorriente,
  observacionConAutorizacion,
  ofertaVigente,
  precioDeVenta,
  rangoManual,
  textoArqueo,
  textoOferta,
} from "../src/utils/reglasVenta.ts";

const ahora = new Date("2026-10-14T15:00:00.000Z");

test("la oferta vigente baja el precio y redondea a centavos", () => {
  const p = { pricePerUnit: 1999, ofertaPorcentaje: 15, ofertaHasta: "2026-10-15T02:59:59.999Z" };
  assert.deepEqual(ofertaVigente(p, ahora), { porcentaje: 15, precio: 1699.15 });
  assert.equal(precioDeVenta({ ...p, pricePerUnit: 1000, ofertaPorcentaje: 20 }, ahora), 800);
  assert.equal(precioDeVenta({ ...p, pricePerUnit: 333.33, ofertaPorcentaje: 33 }, ahora), 223.33);
  assert.equal(precioDeVenta({ ...p, ofertaHasta: ahora.toISOString() }, ahora), 1699.15);
});

test("sin oferta, vencida o con datos raros queda el precio normal", () => {
  const base = { pricePerUnit: 1000 };
  assert.equal(precioDeVenta(base, ahora), 1000);
  assert.equal(precioDeVenta({ ...base, ofertaPorcentaje: 20, ofertaHasta: "2026-10-14T14:59:59.999Z" }, ahora), 1000);
  assert.equal(precioDeVenta({ ...base, ofertaPorcentaje: 20, ofertaHasta: null }, ahora), 1000);
  assert.equal(precioDeVenta({ ...base, ofertaPorcentaje: 0, ofertaHasta: "2026-12-01T00:00:00Z" }, ahora), 1000);
  assert.equal(precioDeVenta({ ...base, ofertaPorcentaje: null, ofertaHasta: "2026-12-01T00:00:00Z" }, ahora), 1000);
  assert.equal(precioDeVenta({ ...base, ofertaPorcentaje: 100, ofertaHasta: "2026-12-01T00:00:00Z" }, ahora), 1000);
  assert.equal(precioDeVenta({ ...base, ofertaPorcentaje: 20, ofertaHasta: "no es fecha" }, ahora), 1000);
});

test("los campos de oferta del carrito guardan el precio tachado y el porcentaje", () => {
  const p = { pricePerUnit: 1000, ofertaPorcentaje: 20, ofertaHasta: "2026-10-20T00:00:00Z" };
  assert.deepEqual(camposDeOferta(p, ahora), { pricePerUnit: 800, precioSinOferta: 1000, ofertaPorcentaje: 20 });
  assert.deepEqual(camposDeOferta({ pricePerUnit: 1000 }, ahora), { pricePerUnit: 1000, precioSinOferta: null, ofertaPorcentaje: null });
  assert.equal(textoOferta(20), "−20 %");
  assert.equal(textoOferta(12.5), "−12,5 %");
});

test("el límite de crédito avisa al 80 % y bloquea al superarlo", () => {
  assert.equal(evaluarCredito(5000, null, 1000).nivel, "sin-limite");
  assert.equal(evaluarCredito(5000, 0, 1000).nivel, "sin-limite");
  assert.equal(evaluarCredito(undefined, undefined, 1000).nivel, "sin-limite");
  assert.equal(evaluarCredito(5000, 10000, 2000).nivel, "normal");
  assert.equal(evaluarCredito(5000, 10000, 3000).nivel, "aviso");
  assert.equal(evaluarCredito(5000, 10000, 5000).nivel, "aviso");
  assert.deepEqual(evaluarCredito(5000, 10000, 5000.01), {
    nivel: "excedido",
    saldo: 5000,
    limite: 10000,
    nuevoSaldo: 10000.01,
    excedente: 0.01,
  });
  assert.equal(evaluarCredito(12000, 10000, 500).excedente, 2500);
});

test("el descuento en efectivo solo se ofrece desde el mínimo", () => {
  assert.ok(descuentoEfectivoDisponible(100, null));
  assert.ok(descuentoEfectivoDisponible(100, 0));
  assert.ok(descuentoEfectivoDisponible(100, undefined));
  assert.ok(descuentoEfectivoDisponible(5000, 5000));
  assert.ok(!descuentoEfectivoDisponible(4999.99, 5000));
});

test("el arqueo usa efectivo y efectivo d como el ticket de cierre", () => {
  const ventas = { efectivo: 10000.5, "Efectivo D": 2000, tarjeta: 9000, qr: 100 };
  assert.equal(efectivoEsperado(ventas), 12000.5);
  assert.equal(efectivoEsperado(null), 0);
  assert.equal(efectivoEsperado({ tarjeta: 10 }), 0);
  assert.deepEqual(diferenciaDeCaja(12000.5, 12000.5), { diferencia: 0, estado: "cuadra" });
  assert.deepEqual(diferenciaDeCaja(12000.9, 12000.5), { diferencia: 0, estado: "cuadra" });
  assert.deepEqual(diferenciaDeCaja(11500, 12000.5), { diferencia: -500.5, estado: "faltan" });
  assert.deepEqual(diferenciaDeCaja(12100, 12000), { diferencia: 100, estado: "sobran" });
});

test("el monto contado acepta vacío y rechaza negativos o texto", () => {
  assert.equal(leerMonto(""), null);
  assert.equal(leerMonto("  "), null);
  assert.equal(leerMonto("12000.5"), 12000.5);
  assert.equal(leerMonto("0"), 0);
  assert.equal(leerMonto("-1"), "invalido");
  assert.equal(leerMonto("abc"), "invalido");
});

test("el límite compara solo lo que queda en cuenta corriente", () => {
  assert.equal(montoACuentaCorriente(2880.02, 1000), 1880.02);
  assert.equal(montoACuentaCorriente(2880.02, undefined), 2880.02);
  assert.equal(montoACuentaCorriente(2880.02, 0), 2880.02);
  assert.equal(evaluarCredito(9000, 10000, montoACuentaCorriente(3000, 2500)).nivel, "aviso");
  assert.equal(evaluarCredito(9000, 10000, montoACuentaCorriente(3000, 0)).nivel, "excedido");
});

test("la autorización de un admin queda en las observaciones", () => {
  assert.equal(observacionConAutorizacion("", "a@b.com"), "Autorizó exceso de límite: a@b.com");
  assert.equal(observacionConAutorizacion(undefined, "a@b.com"), "Autorizó exceso de límite: a@b.com");
  assert.equal(
    observacionConAutorizacion("Entregar el lunes ", "a@b.com"),
    "Entregar el lunes\nAutorizó exceso de límite: a@b.com"
  );
});

test("el arqueo en vivo dice cuadra, faltan o sobran", () => {
  assert.deepEqual(textoArqueo(10500, 10500), { texto: "Cuadra", estado: "cuadra" });
  assert.deepEqual(textoArqueo(10200, 10500), { texto: "Faltan $300", estado: "faltan" });
  assert.deepEqual(textoArqueo(10750.5, 10500), { texto: "Sobran $250,5", estado: "sobran" });
});

const menosTres = (d: Date) => new Date(d.getTime() - 3 * 60 * 60 * 1000).toISOString();

test("el esperado se pide con los mismos datos que el cierre", () => {
  const ahora = new Date(2026, 9, 14, 18, 30);
  const todo = datosDeCierre(77, 5, "todo", ahora);
  assert.deepEqual(todo, {
    vendedorId: 77,
    sucursalId: 5,
    fechaInicio: menosTres(new Date(2026, 9, 14, 0, 0)),
    fechaCierre: menosTres(ahora),
    periodo: "todo",
  });
  assert.equal(datosDeCierre(77, 5, "tarde", ahora).fechaInicio, menosTres(new Date(2026, 9, 14, 12, 0)));
  const temprano = new Date(2026, 9, 14, 11, 0);
  assert.equal(datosDeCierre(77, 5, "tarde", temprano).fechaInicio, menosTres(new Date(2026, 9, 13, 12, 0)));
  const q = new URLSearchParams(consultaDeEsperado(todo));
  assert.equal(q.get("vendedorId"), "77");
  assert.equal(q.get("sucursalId"), "5");
  assert.equal(q.get("periodo"), "todo");
  assert.equal(q.get("fechaInicio"), todo.fechaInicio);
  assert.equal(q.get("fechaCierre"), todo.fechaCierre);
});

test("el cierre manual valida el rango y lo pasa a UTC como antes", () => {
  const dia = new Date(2026, 9, 14);
  assert.deepEqual(rangoManual(dia, dia, "08:00", "13:15"), {
    fechaInicio: menosTres(new Date(2026, 9, 14, 8, 0, 0, 0)),
    fechaCierre: menosTres(new Date(2026, 9, 14, 13, 15, 59, 999)),
  });
  assert.equal(rangoManual(undefined, dia, "00:00", "23:59"), "Por favor, completa todos los campos.");
  assert.equal(rangoManual(dia, new Date(2026, 9, 13), "00:00", "23:59"), "La fecha de fin no puede ser anterior a la fecha de inicio.");
  assert.equal(rangoManual(dia, dia, "13:00", "13:00"), "La hora de fin debe ser posterior a la hora de inicio.");
});
