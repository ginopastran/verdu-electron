import { createRequire } from "module";
import { crearFiltroEstable, crearLectorTramas, pesoDeTrama, type Trama, type UnidadEntera } from "./balanzaTrama.js";

const require = createRequire(import.meta.url);

export type FormatoSerie = "8N1" | "7E1" | "7O1" | "8E1";

const PARIDAD = { N: "none", E: "even", O: "odd" } as const;

export type ConfigBalanza = {
  modo: "archivo" | "serie";
  puerto?: string;
  baudRate: number;
  formato: FormatoSerie;
  consulta: "enq" | "continuo";
  unidadEntera: UnidadEntera;
};

export const CONFIG_BALANZA_DEFAULT: ConfigBalanza = {
  modo: "archivo",
  baudRate: 9600,
  formato: "8N1",
  consulta: "enq",
  unidadEntera: "gramos",
};

const ENQ = Buffer.from([0x05]);
const ACK = Buffer.from([0x06]);
const NACK = Buffer.from([0x15]);
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
  let leer = crearLectorTramas();
  const estable = crearFiltroEstable();

  const procesar = ({ texto, crc }: Trama) => {
    if (crc !== "sin" && config.consulta === "enq") port.write(crc === "ok" ? ACK : NACK);
    if (crc === "mal") return;
    const gramos = pesoDeTrama(texto, config.unidadEntera);
    if (gramos === null) return;
    const valor = estable(gramos);
    if (valor !== null) onPeso(valor);
  };

  const limpiar = () => {
    clearInterval(consulta);
    clearTimeout(silencio);
    leer = crearLectorTramas();
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
      dataBits: Number(config.formato[0]),
      parity: PARIDAD[config.formato[1] as keyof typeof PARIDAD],
      stopBits: Number(config.formato[2]),
      autoOpen: false,
    });

    port.on("data", (chunk: Buffer) => {
      leer(chunk).forEach(procesar);
      clearTimeout(silencio);
      silencio = setTimeout(() => leer(Buffer.alloc(0), true).forEach(procesar), SILENCIO_FIN_TRAMA_MS);
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
