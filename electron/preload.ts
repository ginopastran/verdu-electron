const { contextBridge, ipcRenderer } = require("electron");

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
