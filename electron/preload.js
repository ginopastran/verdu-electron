const { contextBridge, ipcRenderer } = require("electron");

// Exponer funciones de electron-store al renderer
contextBridge.exposeInMainWorld("electronStore", {
  get: (key) => ipcRenderer.invoke("store-get", key),
  set: (key, value) => ipcRenderer.invoke("store-set", key, value),
  delete: (key) => ipcRenderer.invoke("store-delete", key),
  has: (key) => ipcRenderer.invoke("store-has", key),
});

// También exponer otras funciones existentes si las hay
contextBridge.exposeInMainWorld("electron", {
  ipcRenderer: {
    invoke: (channel, ...args) => ipcRenderer.invoke(channel, ...args),
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
