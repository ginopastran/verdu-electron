const { contextBridge, ipcRenderer } = require("electron");

console.log("🔧 Preload script iniciado correctamente");
console.log("🔧 Proceso de preload - Entorno:", process.versions);
console.log(
  "🔧 Proceso de preload - Node integración disponible:",
  process.versions.node
);

// Exponer funciones de electron-store al renderer
contextBridge.exposeInMainWorld("electronStore", {
  get: (key: string) => ipcRenderer.invoke("store-get", key),
  set: (key: string, value: any) => ipcRenderer.invoke("store-set", key, value),
  delete: (key: string) => ipcRenderer.invoke("store-delete", key),
  has: (key: string) => ipcRenderer.invoke("store-has", key),
});

// También exponer otras funciones existentes si las hay
contextBridge.exposeInMainWorld("electron", {
  ipcRenderer: {
    invoke: (channel: string, ...args: any[]) =>
      ipcRenderer.invoke(channel, ...args),
  },
  process: {
    argv: process.argv,
  },
});

// Exponer funciones del auto-updater
contextBridge.exposeInMainWorld("autoUpdater", {
  checkForUpdates: () => ipcRenderer.invoke("check-for-updates"),
  downloadUpdate: () => ipcRenderer.invoke("download-update"),
  installUpdate: () => ipcRenderer.invoke("install-update"),
});

// Exponer función para leer peso
contextBridge.exposeInMainWorld("pesoReader", {
  readPeso: () => ipcRenderer.invoke("read-peso"),
});

// Exponer funciones específicas para impresión
contextBridge.exposeInMainWorld("printer", {
  printTicket: (orderData: any) =>
    ipcRenderer.invoke("print-ticket", orderData),
  printAfipTicket: (afipData: any) =>
    ipcRenderer.invoke("print-afip-ticket", afipData),
  printClosing: (closingData: any) =>
    ipcRenderer.invoke("print-closing", closingData),
  getAvailablePrinters: () => ipcRenderer.invoke("get-available-printers"),
});

console.log("🔧 Preload script - APIs expuestas exitosamente:", {
  electronStore: "disponible",
  electron: "disponible",
  autoUpdater: "disponible",
  pesoReader: "disponible",
  printer:
    "disponible (printTicket, printAfipTicket, printClosing, getAvailablePrinters)",
});

// Verificar que printAfipTicket esté disponible
console.log(
  "🔧 Verificando printAfipTicket:",
  typeof window !== "undefined" ? "disponible en window" : "no disponible"
);
console.log(
  "🔧 printAfipTicket function:",
  typeof (window as any).printer?.printAfipTicket
);
