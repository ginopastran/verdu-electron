export type UnidadEntera = "gramos" | "kilos";

export function pesoDeTrama(trama: string, unidadEntera: UnidadEntera = "gramos"): number | null {
  const numero = trama.match(/[-+]?\d+(?:[.,]\d+)?/);
  if (!numero) return null;
  const texto = numero[0].replace(",", ".");
  const valor = Number(texto);
  if (!Number.isFinite(valor)) return null;
  const enKilos = texto.includes(".") || unidadEntera === "kilos";
  return Math.round(enKilos ? valor * 1000 : valor);
}

export function crearFiltroEstable(lecturas = 3, toleranciaGramos = 2) {
  const ultimas: number[] = [];
  return (gramos: number): number | null => {
    ultimas.push(gramos);
    if (ultimas.length > lecturas) ultimas.shift();
    if (ultimas.length < lecturas) return null;
    return Math.max(...ultimas) - Math.min(...ultimas) <= toleranciaGramos ? gramos : null;
  };
}

const STX = 0x02;
const ETX = 0x03;
const CR = 0x0d;
const LF = 0x0a;
const CRC_PARA_CONFIRMAR = 3;

export type Trama = { texto: string; crc: "ok" | "mal" | "sin" };

const xor = (bytes: Uint8Array) => bytes.reduce((a, b) => a ^ b, 0);
const sinControl = (bytes: Buffer) => bytes.toString("latin1").replace(/[\x00-\x1f]/g, "");

export function crearLectorTramas() {
  let buffer = Buffer.alloc(0);
  let crcSeguidos = 0;
  return (chunk: Buffer, cortar = false): Trama[] => {
    buffer = Buffer.concat([buffer, chunk]);
    const tramas: Trama[] = [];
    for (;;) {
      const fin = buffer.findIndex((b) => b === ETX || b === CR || b === LF);
      if (fin < 0) break;
      if (buffer[fin] !== ETX) {
        tramas.push({ texto: sinControl(buffer.subarray(0, fin)), crc: "sin" });
        buffer = buffer.subarray(fin + 1);
        continue;
      }
      if (fin + 1 >= buffer.length && !cortar) break;
      const inicio = buffer.lastIndexOf(STX, fin);
      const texto = sinControl(buffer.subarray(inicio + 1, fin));
      if (buffer[fin + 1] === xor(buffer.subarray(Math.max(inicio, 0), fin + 1))) {
        crcSeguidos++;
        tramas.push({ texto, crc: "ok" });
        buffer = buffer.subarray(fin + 2);
      } else if (crcSeguidos >= CRC_PARA_CONFIRMAR) {
        tramas.push({ texto, crc: "mal" });
        buffer = buffer.subarray(fin + 2);
      } else {
        crcSeguidos = 0;
        tramas.push({ texto, crc: "sin" });
        buffer = buffer.subarray(fin + 1);
      }
    }
    if (cortar && buffer.length) {
      tramas.push({ texto: sinControl(buffer), crc: "sin" });
      buffer = Buffer.alloc(0);
    }
    return tramas;
  };
}
