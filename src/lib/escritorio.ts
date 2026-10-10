import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type Escucha = (...args: any[]) => void;

const fetchNativo = window.fetch.bind(window);
window.fetch = (recurso: RequestInfo | URL, opciones?: RequestInit) => {
  const url = recurso instanceof Request ? recurso.url : String(recurso);
  return url.startsWith("http://ipc.localhost/")
    ? Promise.reject(new TypeError("IPC por postMessage"))
    : fetchNativo(recurso, opciones);
};

const llamar = (canal: string, ...args: unknown[]) => invoke<any>("ipc", { canal, args });

const escuchas = new Map<string, Map<Escucha, Promise<UnlistenFn>>>();

const delCanal = (canal: string) => {
  if (!escuchas.has(canal)) escuchas.set(canal, new Map());
  return escuchas.get(canal)!;
};

Object.assign(window as any, {
  electron: {
    ipcRenderer: {
      invoke: llamar,
      on: (canal: string, escucha: Escucha) => {
        delCanal(canal).set(escucha, listen(canal, (evento) => escucha(evento.payload)));
      },
      removeListener: (canal: string, escucha: Escucha) => {
        delCanal(canal).get(escucha)?.then((dejar) => dejar());
        delCanal(canal).delete(escucha);
      },
    },
    process: { argv: (window as any).__ARGV__ ?? [] },
  },
  electronStore: {
    get: (clave: string) => llamar("store-get", clave),
    set: (clave: string, valor: unknown) => llamar("store-set", clave, valor),
    delete: (clave: string) => llamar("store-delete", clave),
    has: (clave: string) => llamar("store-has", clave),
  },
  autoUpdater: {
    checkForUpdates: () => llamar("check-for-updates"),
    downloadUpdate: () => llamar("download-update"),
    installUpdate: () => llamar("install-update"),
  },
  pesoReader: {
    readPeso: () => llamar("read-peso"),
  },
  printer: {
    printTicket: (datos: unknown) => llamar("print-ticket", datos),
    printAfipTicket: (datos: unknown) => llamar("print-afip-ticket", datos),
    printClosing: (datos: unknown) => llamar("print-closing", datos),
    getAvailablePrinters: () => llamar("get-available-printers"),
  },
});
