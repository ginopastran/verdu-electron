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

export function textoArqueo(contado: number, esperado: number) {
  const { diferencia, estado } = diferenciaDeCaja(contado, esperado);
  const monto = `$${Math.abs(diferencia).toLocaleString("es-AR")}`;
  const texto =
    estado === "cuadra" ? "Cuadra" : `${estado === "faltan" ? "Faltan" : "Sobran"} ${monto}`;
  return { texto, estado };
}

const TRES_HORAS = 3 * 60 * 60 * 1000;

export function datosDeCierre(
  vendedorId: number,
  sucursalId: number,
  periodo: string,
  ahora: Date = new Date()
) {
  const inicio = new Date(
    ahora.getFullYear(),
    ahora.getMonth(),
    ahora.getDate(),
    periodo === "tarde" ? 12 : 0
  );
  let fechaInicio = new Date(inicio.getTime() - TRES_HORAS);
  const fechaCierre = new Date(ahora.getTime() - TRES_HORAS);
  if (fechaInicio.getTime() >= fechaCierre.getTime()) {
    fechaInicio = new Date(fechaInicio);
    fechaInicio.setDate(fechaInicio.getDate() - 1);
  }
  return {
    vendedorId,
    sucursalId,
    fechaInicio: fechaInicio.toISOString(),
    fechaCierre: fechaCierre.toISOString(),
    periodo,
  };
}

export function rangoManual(
  desde: Date | undefined,
  hasta: Date | undefined,
  horaDesde: string,
  horaHasta: string
): { fechaInicio: string; fechaCierre: string } | string {
  if (!desde || !hasta) return "Por favor, completa todos los campos.";
  if (hasta < desde) return "La fecha de fin no puede ser anterior a la fecha de inicio.";
  const [h1, m1] = horaDesde.split(":").map(Number);
  const [h2, m2] = horaHasta.split(":").map(Number);
  if (desde.getTime() === hasta.getTime() && (h1 > h2 || (h1 === h2 && m1 >= m2))) {
    return "La hora de fin debe ser posterior a la hora de inicio.";
  }
  const inicio = new Date(desde);
  inicio.setHours(h1, m1, 0, 0);
  const cierre = new Date(hasta);
  cierre.setHours(h2, m2, 59, 999);
  return {
    fechaInicio: new Date(inicio.getTime() - TRES_HORAS).toISOString(),
    fechaCierre: new Date(cierre.getTime() - TRES_HORAS).toISOString(),
  };
}

export function consultaDeEsperado(datos: {
  vendedorId: number;
  sucursalId: number;
  periodo: string;
  fechaInicio: string;
  fechaCierre: string;
}) {
  return new URLSearchParams({
    vendedorId: String(datos.vendedorId),
    sucursalId: String(datos.sucursalId),
    periodo: datos.periodo,
    fechaInicio: datos.fechaInicio,
    fechaCierre: datos.fechaCierre,
  }).toString();
}

export function leerMonto(texto: string): number | null | "invalido" {
  const limpio = texto.trim();
  if (!limpio) return null;
  const n = Number(limpio);
  return Number.isFinite(n) && n >= 0 && n < 1e12 ? n : "invalido";
}
