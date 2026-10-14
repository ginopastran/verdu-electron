export type ConOferta = {
  pricePerUnit: number;
  ofertaPorcentaje?: number | null;
  ofertaHasta?: string | null;
};

const redondear = (n: number) => Number(n.toFixed(2));

export function ofertaVigente(p: ConOferta, ahora: Date = new Date()) {
  const porcentaje = Number(p.ofertaPorcentaje);
  if (!p.ofertaHasta || !(porcentaje > 0 && porcentaje < 100)) return null;
  if (!(new Date(p.ofertaHasta).getTime() >= ahora.getTime())) return null;
  return {
    porcentaje,
    precio: redondear(p.pricePerUnit * (1 - porcentaje / 100)),
  };
}

export function precioDeVenta(p: ConOferta, ahora: Date = new Date()) {
  return ofertaVigente(p, ahora)?.precio ?? p.pricePerUnit;
}

export function camposDeOferta(p: ConOferta, ahora: Date = new Date()) {
  const oferta = ofertaVigente(p, ahora);
  return {
    pricePerUnit: oferta?.precio ?? p.pricePerUnit,
    precioSinOferta: oferta ? p.pricePerUnit : null,
    ofertaPorcentaje: oferta?.porcentaje ?? null,
  };
}

export const textoOferta = (porcentaje: number) =>
  `−${porcentaje.toLocaleString("es-AR")} %`;

export type EstadoCredito = {
  nivel: "sin-limite" | "normal" | "aviso" | "excedido";
  saldo: number;
  limite: number | null;
  nuevoSaldo: number;
  excedente: number;
};

export function evaluarCredito(
  saldo: number | null | undefined,
  limite: number | null | undefined,
  monto: number
): EstadoCredito {
  const deuda = Number(saldo) || 0;
  const nuevoSaldo = redondear(deuda + monto);
  const tope = Number(limite);
  if (!(tope > 0)) {
    return { nivel: "sin-limite", saldo: deuda, limite: null, nuevoSaldo, excedente: 0 };
  }
  const excedente = redondear(nuevoSaldo - tope);
  const nivel =
    excedente > 0 ? "excedido" : nuevoSaldo >= tope * 0.8 ? "aviso" : "normal";
  return { nivel, saldo: deuda, limite: tope, nuevoSaldo, excedente: Math.max(excedente, 0) };
}

export const montoACuentaCorriente = (
  total: number,
  pagoInicial: number | null | undefined
) => redondear(total - (Number(pagoInicial) || 0));

export function observacionConAutorizacion(
  observaciones: string | null | undefined,
  emailAdmin: string
) {
  const nota = `Autorizó exceso de límite: ${emailAdmin}`;
  const previa = (observaciones ?? "").trim();
  return previa ? `${previa}\n${nota}` : nota;
}

export function descuentoEfectivoDisponible(
  total: number,
  minimo: number | null | undefined
) {
  const piso = Number(minimo);
  return !(piso > 0) || total >= piso;
}

const METODOS_EFECTIVO = ["efectivo", "efectivo d"];

export function efectivoEsperado(ventasPorMetodo: unknown) {
  if (!ventasPorMetodo || typeof ventasPorMetodo !== "object") return 0;
  const total = Object.entries(ventasPorMetodo as Record<string, unknown>).reduce(
    (suma, [metodo, monto]) =>
      METODOS_EFECTIVO.includes(metodo.toLowerCase().trim())
        ? suma + (Number(monto) || 0)
        : suma,
    0
  );
  return Math.round(total * 100) / 100;
}

export function diferenciaDeCaja(contado: number, esperado: number) {
  const diferencia = Math.round((contado - esperado) * 100) / 100;
  const redondeada = Math.round(diferencia);
  const estado =
    redondeada === 0 ? "cuadra" : redondeada < 0 ? "faltan" : "sobran";
  return { diferencia: estado === "cuadra" ? 0 : diferencia, estado };
}

export function leerMonto(texto: string): number | null | "invalido" {
  const limpio = texto.trim();
  if (!limpio) return null;
  const n = Number(limpio);
  return Number.isFinite(n) && n >= 0 && n < 1e12 ? n : "invalido";
}
