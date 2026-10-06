import { createRequire } from "module";
import { crearFiltroEstable, pesoDeTrama, type UnidadEntera } from "./balanzaTrama.js";

const require = createRequire(import.meta.url);

export type ConfigBalanza = {
  modo: "archivo" | "serie";
  puerto?: string;
  baudRate: number;
  consulta: "enq" | "continuo";
  unidadEntera: UnidadEntera;
};

export const CONFIG_BALANZA_DEFAULT: ConfigBalanza = {
  modo: "archivo",
  baudRate: 9600,
  consulta: "enq",
  unidadEntera: "gramos",
};

const ENQ = Buffer.from([0x05]);
const INTERVALO_CONSULTA_MS = 300;
const SILENCIO_FIN_TRAMA_MS = 60;
const REINTENTO_MS = 5000;

export async function listarPuertos(): Promise<{ path: string; manufacturer?: string }[]> {
  const { SerialPort } = require("serialport");
  return SerialPort.list();
}

export function iniciarBalanzaSerie(
  config: ConfigBalanza,
  onPeso: (gramos: number) => void,
  onEstado: (estado: { conectada: boolean; error?: string }) => void
): () => void {
  const { SerialPort } = require("serialport");
  let detenida = false;
  let port: any;
  let consulta: NodeJS.Timeout | undefined;
  let silencio: NodeJS.Timeout | undefined;
  let reintento: NodeJS.Timeout | undefined;
  let buffer = "";
  const estable = crearFiltroEstable();

  const procesar = (trama: string) => {
    const gramos = pesoDeTrama(trama, config.unidadEntera);
    if (gramos === null) return;
    const valor = estable(gramos);
    if (valor !== null) onPeso(valor);
  };

  const limpiar = () => {
    clearInterval(consulta);
    clearTimeout(silencio);
    buffer = "";
  };

  const reintentar = (error?: string) => {
    limpiar();
    onEstado({ conectada: false, error });
    if (!detenida) reintento = setTimeout(abrir, REINTENTO_MS);
  };

  function abrir() {
    if (!config.puerto) return onEstado({ conectada: false, error: "Sin puerto configurado" });
    port = new SerialPort({
      path: config.puerto,
      baudRate: config.baudRate,
      dataBits: 8,
      parity: "none",
      stopBits: 1,
      autoOpen: false,
    });

    port.on("data", (chunk: Buffer) => {
      buffer += chunk.toString("latin1");
      const partes = buffer.split(/\r\n|\r|\n/);
      buffer = partes.pop() ?? "";
      partes.forEach(procesar);
      clearTimeout(silencio);
      silencio = setTimeout(() => {
        const resto = buffer;
        buffer = "";
        procesar(resto);
      }, SILENCIO_FIN_TRAMA_MS);
    });
    port.on("close", () => reintentar("Puerto cerrado"));
    port.on("error", (err: Error) => onEstado({ conectada: port.isOpen, error: err.message }));

    port.open((err: Error | null) => {
      if (err) return reintentar(err.message);
      onEstado({ conectada: true });
      if (config.consulta === "enq") {
        consulta = setInterval(() => port.write(ENQ), INTERVALO_CONSULTA_MS);
      }
    });
  }

  abrir();

  return () => {
    detenida = true;
    clearTimeout(reintento);
    limpiar();
    if (port?.isOpen) port.close();
  };
}
