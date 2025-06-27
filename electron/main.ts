import { app, BrowserWindow, ipcMain, dialog } from "electron";
import * as path from "path";
import { fileURLToPath } from "url";
import { exec, spawn } from "child_process";
import * as fsPromises from "fs/promises";
import os from "os";
import * as fs from "fs";
// ES Module dynamic imports para compatibilidad
import { createRequire } from "module";
const require = createRequire(import.meta.url);

// electron-store es ES Module, usamos import dinámico
let Store: any;
const { autoUpdater } = require("electron-updater");

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// APP_ID fijo para toda la aplicación
const APP_ID =
  "b2fa850b9a1782595da81d0699892e93a3f29f9d5b0fd74ef4ede03f05658942";

// Hacer disponible el APP_ID para el proceso de renderizado
process.env.VITE_APP_ID = APP_ID;

console.log("App ID:", APP_ID);

// Inicializar electron-store (será inicializado dinámicamente)
let store: any;

// Configurar IPC handlers para electron-store
ipcMain.handle("store-get", (_, key) => {
  return store.get(key);
});

ipcMain.handle("store-set", (_, key, value) => {
  store.set(key, value);
  return true;
});

ipcMain.handle("store-delete", (_, key) => {
  store.delete(key);
  return true;
});

ipcMain.handle("store-has", (_, key) => {
  return store.has(key);
});

// IPC handler para leer el peso desde el archivo
ipcMain.handle("read-peso", async () => {
  const pesoPath = "C:\\Peso\\peso.json";

  try {
    console.log("📏 Leyendo peso desde:", pesoPath);

    // Verificar si el archivo existe
    if (!fs.existsSync(pesoPath)) {
      console.log("⚠️ Archivo de peso no encontrado:", pesoPath);
      return { error: "Archivo no encontrado", peso: 0 };
    }

    // Leer el archivo
    const data = await fsPromises.readFile(pesoPath, "utf8");

    // Limpiar BOM y espacios extra antes del parsing
    const cleanData = data.replace(/^\uFEFF/, "").trim();
    console.log("📋 Datos leídos del archivo:", cleanData);

    if (!cleanData) {
      console.log("⚠️ Archivo de peso está vacío");
      return { error: "Archivo vacío", peso: 0 };
    }

    // Parsear el JSON
    const weightData = JSON.parse(cleanData);
    console.log("✅ Peso parseado exitosamente:", weightData);

    return { success: true, peso: weightData.peso || 0 };
  } catch (error) {
    console.error("❌ Error al leer archivo de peso:", error);
    return { error: (error as Error).message, peso: 0 };
  }
});

// Configuración del auto-updater (log será configurado dinámicamente)
// autoUpdater.logger.transports.file.level = "info"; // Comentado porque console no tiene transports
autoUpdater.autoDownload = false; // No descargar automáticamente
autoUpdater.autoInstallOnAppQuit = false; // No instalar automáticamente al cerrar

// Eventos del auto-updater
autoUpdater.on("checking-for-update", () => {
  console.log("Verificando actualizaciones...");
});

autoUpdater.on("update-available", (info: any) => {
  console.log("Actualización disponible.");
  console.log("Versión:", info.version);

  // Mostrar diálogo de actualización
  dialog
    .showMessageBox({
      type: "info",
      title: "Actualización disponible",
      message: `Nueva versión ${info.version} disponible`,
      detail: "¿Deseas descargar e instalar la actualización ahora?",
      buttons: ["Descargar", "Más tarde"],
      defaultId: 0,
      cancelId: 1,
    })
    .then((response) => {
      if (response.response === 0) {
        // Descargar actualización
        autoUpdater.downloadUpdate();
      }
    });
});

autoUpdater.on("update-not-available", (info: any) => {
  console.log("Actualización no disponible.");
});

autoUpdater.on("error", (err: any) => {
  console.log("Error en auto-updater. " + err);
});

autoUpdater.on("download-progress", (progressObj: any) => {
  let log_message = "Velocidad de descarga: " + progressObj.bytesPerSecond;
  log_message = log_message + " - Descargado " + progressObj.percent + "%";
  log_message =
    log_message +
    " (" +
    progressObj.transferred +
    "/" +
    progressObj.total +
    ")";
  console.log(log_message);
});

autoUpdater.on("update-downloaded", (info: any) => {
  console.log("Actualización descargada");
  autoUpdater.quitAndInstall();
});

// Estado global para el auto-updater
let updateCheckResult: any = null;
let currentlyChecking = false;

// IPC handlers para el auto-updater
ipcMain.handle("check-for-updates", async () => {
  if (currentlyChecking) {
    return { error: "Ya se está verificando actualizaciones" };
  }

  try {
    currentlyChecking = true;
    console.log("🔍 Iniciando verificación de actualizaciones...");
    console.log("📦 Versión actual:", app.getVersion());

    const result = await autoUpdater.checkForUpdates();
    updateCheckResult = result;

    if (result && result.updateInfo) {
      const currentVersion = app.getVersion();
      const availableVersion = result.updateInfo.version;

      console.log("📋 Versión disponible:", availableVersion);
      console.log("📋 Versión actual:", currentVersion);

      // Función para comparar versiones semánticamente
      const compareVersions = (available: string, current: string): number => {
        const parseVersion = (v: string) =>
          v.split(".").map((n) => parseInt(n) || 0);
        const availableParts = parseVersion(available);
        const currentParts = parseVersion(current);

        for (
          let i = 0;
          i < Math.max(availableParts.length, currentParts.length);
          i++
        ) {
          const a = availableParts[i] || 0;
          const c = currentParts[i] || 0;
          if (a > c) return 1; // available es mayor
          if (a < c) return -1; // current es mayor
        }
        return 0; // son iguales
      };

      const versionComparison = compareVersions(
        availableVersion,
        currentVersion
      );

      if (versionComparison > 0) {
        console.log("✅ Nueva versión encontrada (más reciente)");
        return { available: true, info: result.updateInfo };
      } else if (versionComparison < 0) {
        console.log("ℹ️ Tienes una versión más nueva que la disponible");
        return {
          available: false,
          info: { message: "Tienes una versión más nueva que la disponible" },
        };
      } else {
        console.log("ℹ️ Ya tienes la última versión");
        return {
          available: false,
          info: { message: "Ya tienes la última versión" },
        };
      }
    } else {
      console.log("ℹ️ No hay actualizaciones disponibles");
      return {
        available: false,
        info: { message: "No hay actualizaciones disponibles" },
      };
    }
  } catch (error) {
    console.error("❌ Error al verificar actualizaciones:", error);
    updateCheckResult = null;
    return { error: (error as Error).message };
  } finally {
    currentlyChecking = false;
  }
});

ipcMain.handle("download-update", async () => {
  if (!updateCheckResult) {
    return { error: "Please check update first" };
  }

  try {
    console.log("📥 Iniciando descarga de actualización...");
    await autoUpdater.downloadUpdate();
    console.log("✅ Descarga completada");
    return { success: true };
  } catch (error) {
    console.error("❌ Error al descargar actualización:", error);
    return { error: (error as Error).message };
  }
});

ipcMain.handle("install-update", () => {
  autoUpdater.quitAndInstall();
});

// Agregar un manejador IPC para mostrar el APP_ID
ipcMain.handle("show-app-id", () => {
  dialog.showMessageBox({
    type: "info",
    title: "APP ID",
    message: `APP_ID actual: ${APP_ID}`,
    buttons: ["OK"],
  });
  return APP_ID;
});

// Función para inicializar electron-store dinámicamente
async function initializeStore() {
  const { default: ElectronStore } = await import("electron-store");

  Store = ElectronStore;
  store = new Store();

  // Configurar el logger del auto-updater con console simple
  autoUpdater.logger = console;
}

function createWindow() {
  const iconPath = path.join(
    __dirname,
    process.env.NODE_ENV === "development"
      ? "../../public/icon.png"
      : "../icon.png"
  );

  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    title: "AndexMarket",
    icon: iconPath,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: true,
      devTools: true,
      webSecurity: false,
      preload: path.join(__dirname, "preload.cjs"),
    },
  });

  if (process.env.NODE_ENV === "development") {
    mainWindow.loadURL("http://localhost:5173");
    mainWindow.webContents.openDevTools();
  } else {
    // En producción, oculta el menú nativo (File, Edit, View, Window, Help)
    mainWindow.setMenuBarVisibility(false);

    // En producción, carga el archivo HTML construido
    mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));

    // Configurar protocolo para recursos estáticos
    mainWindow.webContents.session.protocol.registerFileProtocol(
      "app",
      (request, callback) => {
        const url = request.url.substr(6);
        callback({ path: path.normalize(`${__dirname}/../dist/${url}`) });
      }
    );

    // Manejar navegación para SPA
    mainWindow.webContents.on(
      "did-fail-load",
      (event, errorCode, errorDescription, validatedURL) => {
        if (errorCode === -6) {
          mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));
        }
      }
    );
  }
}

app.whenReady().then(async () => {
  // Inicializar electron-store antes de crear la ventana
  await initializeStore();

  createWindow();

  // Verificar actualizaciones después de 3 segundos en producción
  if (process.env.NODE_ENV !== "development") {
    // Habilitar logging detallado
    autoUpdater.logger = console;
    autoUpdater.logger.transports.file.level = "info";

    console.log("=== AUTO-UPDATER DEBUG ===");
    console.log("NODE_ENV:", process.env.NODE_ENV);
    console.log("App Version:", app.getVersion());
    console.log("Platform:", process.platform);
    console.log(
      "Repository URL will be:",
      `https://api.github.com/repos/ginopastran/verdu-electron/releases`
    );

    setTimeout(() => {
      console.log("🔄 Iniciando verificación automática de actualizaciones...");
      autoUpdater
        .checkForUpdates()
        .then((result: any) => {
          console.log("✅ Verificación automática completada:", result);
        })
        .catch((error: any) => {
          console.error("❌ Error en verificación automática:", error);
        });
    }, 3000);

    // También chequear cada 10 minutos
    setInterval(() => {
      console.log("🔄 Verificación periódica de actualizaciones...");
      autoUpdater.checkForUpdates();
    }, 10 * 60 * 1000); // 10 minutos
  }

  // Copiar logo al iniciar la aplicación (en producción)
  if (process.env.NODE_ENV !== "development") {
    const logoSourcePath = path.join(app.getAppPath(), "public", "logo.png");
    const logoDestPath = path.join(process.resourcesPath, "logo.png");

    console.log("------ COPIANDO LOGO AL INICIAR ------");
    console.log(`Origen: ${logoSourcePath}`);
    console.log(`Destino: ${logoDestPath}`);

    try {
      if (fs.existsSync(logoSourcePath)) {
        fs.copyFileSync(logoSourcePath, logoDestPath);
        console.log("Logo copiado exitosamente al iniciar la aplicación");
      } else {
        console.warn(`Logo no encontrado en: ${logoSourcePath}`);
      }
    } catch (err) {
      console.error("Error copiando logo al iniciar:", err);
    }
  }

  // Registrar todos los handlers IPC aquí
  ipcMain.handle("read-weight", async () => {
    try {
      const weightPath = "C:\\Peso\\peso.json";
      const data = await fsPromises.readFile(weightPath, "utf8");
      const weightData = JSON.parse(data);
      console.log("Peso leído:", weightData.peso);
      return weightData.peso;
    } catch (error) {
      console.error("Error al leer el peso:", error);
      return 0;
    }
  });

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

ipcMain.handle("print-ticket", async (_, orderData) => {
  try {
    const tempDir = os.tmpdir();
    const tempDataPath = path.join(tempDir, `order-data-${Date.now()}.json`);
    await fsPromises.writeFile(tempDataPath, JSON.stringify(orderData), "utf8");

    const isProduction = process.env.NODE_ENV !== "development";
    let phpScriptPath;
    if (isProduction) {
      phpScriptPath = path.join(
        process.resourcesPath,
        "resources",
        "ticket_printer.php"
      );
    } else {
      phpScriptPath = path.join(
        app.getAppPath(),
        "resources",
        "ticket_printer.php"
      );
    }

    // LOG: Verificar existencia del script PHP
    if (!fs.existsSync(phpScriptPath)) {
      console.error("❌ No se encontró el script PHP:", phpScriptPath);
      return {
        success: false,
        printerError: `No se encontró el script PHP: ${phpScriptPath}`,
        message: `No se encontró el script PHP: ${phpScriptPath}`,
      };
    }

    console.log("------ INFO DE IMPRESIÓN ------");
    console.log(`NODE_ENV: ${process.env.NODE_ENV}`);
    console.log(`PHP Script: ${phpScriptPath}`);
    console.log(`Datos: ${tempDataPath}`);

    return new Promise((resolve, reject) => {
      exec(
        `set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`,
        async (error, stdout, stderr) => {
          try {
            await fsPromises.unlink(tempDataPath);
            console.log("[PRINT] error:", error);
            console.log("[PRINT] stdout:", stdout);
            console.log("[PRINT] stderr:", stderr);
            if (error) {
              console.error("[PRINT] Código de salida:", error.code);
            }
            let printerError = null;
            if (stderr && stderr.includes("Error: ")) {
              const errorMatch = stderr.match(/Error: (.*?)(\n|$)/);
              if (errorMatch && errorMatch[1]) {
                printerError = errorMatch[1];
              }
            }
            if (error) {
              resolve({
                success: false,
                printerError: printerError || error.message,
                message: printerError
                  ? `Error de impresión: ${printerError}`
                  : error.message
                  ? `Error de impresión: ${error.message}`
                  : "Error desconocido",
              });
              return;
            }
            if (printerError) {
              resolve({
                success: false,
                printerError,
                message: `Error de impresión: ${printerError}`,
              });
              return;
            }
            console.log("Salida del script PHP:", stdout);
            resolve({
              success: true,
              printerError: null,
              message: "Ticket impreso correctamente",
            });
          } catch (err) {
            console.error("Error en el callback:", err);
            reject(err);
          }
        }
      );
    });
  } catch (error) {
    console.error("Error en impresión:", error);
    throw error;
  }
});

// Handler para impresión AFIP
console.log("🔧 Registrando handler para print-afip-ticket...");
ipcMain.handle("print-afip-ticket", async (_, afipData) => {
  console.log("🖨️ Handler print-afip-ticket ejecutándose con datos:", afipData);
  try {
    const tempDir = os.tmpdir();
    const tempDataPath = path.join(tempDir, `afip-data-${Date.now()}.json`);

    console.log("📁 AFIP: Escribiendo datos temporales en:", tempDataPath);
    await fsPromises.writeFile(tempDataPath, JSON.stringify(afipData), "utf8");
    console.log("✅ AFIP: Archivo temporal creado exitosamente");

    const isProduction = process.env.NODE_ENV !== "development";
    let phpScriptPath;
    if (isProduction) {
      phpScriptPath = path.join(
        process.resourcesPath,
        "resources",
        "afip_ticket_printer.php"
      );
    } else {
      phpScriptPath = path.join(
        app.getAppPath(),
        "resources",
        "afip_ticket_printer.php"
      );
    }

    console.log("🔍 AFIP: Ruta del script PHP:", phpScriptPath);
    console.log("🔍 AFIP: ¿Existe el script?", fs.existsSync(phpScriptPath));

    // LOG: Verificar existencia del script PHP AFIP
    if (!fs.existsSync(phpScriptPath)) {
      console.error("❌ No se encontró el script PHP AFIP:", phpScriptPath);
      return {
        success: false,
        printerError: `No se encontró el script PHP AFIP: ${phpScriptPath}`,
        message: `No se encontró el script PHP AFIP: ${phpScriptPath}`,
      };
    }

    console.log("------ INFO DE IMPRESIÓN AFIP ------");
    console.log(`NODE_ENV: ${process.env.NODE_ENV}`);
    console.log(`PHP Script AFIP: ${phpScriptPath}`);
    console.log(`Datos AFIP: ${tempDataPath}`);

    const command = `set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`;
    console.log("💻 AFIP: Comando a ejecutar:", command);

    return new Promise((resolve, reject) => {
      exec(command, async (error, stdout, stderr) => {
        try {
          console.log("🔄 AFIP: Eliminando archivo temporal...");
          await fsPromises.unlink(tempDataPath);

          console.log("📊 ===== RESULTADOS AFIP =====");
          console.log("[PRINT-AFIP] error:", error);
          console.log("[PRINT-AFIP] stdout:", stdout);
          console.log("[PRINT-AFIP] stderr:", stderr);
          console.log("📊 ========================");

          if (error) {
            console.error("[PRINT-AFIP] Código de salida:", error.code);
          }

          let printerError = null;
          if (stderr && stderr.includes("Error: ")) {
            const errorMatch = stderr.match(/Error: (.*?)(\n|$)/);
            if (errorMatch && errorMatch[1]) {
              printerError = errorMatch[1];
            }
          }

          if (error) {
            console.error("❌ AFIP: Error en exec:", error);
            resolve({
              success: false,
              printerError: printerError || error.message,
              message: printerError
                ? `Error de impresión AFIP: ${printerError}`
                : error.message
                ? `Error de impresión AFIP: ${error.message}`
                : "Error desconocido en impresión AFIP",
            });
            return;
          }

          if (printerError) {
            console.error("❌ AFIP: Error en stderr:", printerError);
            resolve({
              success: false,
              printerError,
              message: `Error de impresión AFIP: ${printerError}`,
            });
            return;
          }

          console.log("✅ AFIP: Ticket impreso correctamente");
          resolve({
            success: true,
            printerError: null,
            message: "Ticket AFIP impreso correctamente",
          });
        } catch (err) {
          console.error("❌ AFIP: Error en el callback:", err);
          reject(err);
        }
      });
    });
  } catch (error: any) {
    console.error("❌ AFIP: Error general en print-afip-ticket:", error);
    return {
      success: false,
      printerError: error.message,
      message: error.message,
    };
  }
});

// Agregar un nuevo manejador IPC para DevTools
ipcMain.handle("toggle-devtools", () => {
  const win = BrowserWindow.getFocusedWindow();
  if (win) {
    win.webContents.toggleDevTools();
  }
});

// Agregar un nuevo manejador IPC para obtener lista de impresoras
ipcMain.handle("get-available-printers", async () => {
  try {
    let mainWindow = BrowserWindow.getAllWindows()[0];
    if (!mainWindow) {
      throw new Error("No hay ventana principal disponible");
    }

    console.log("🔍 Obteniendo lista completa de impresoras...");
    const availablePrinters = await mainWindow.webContents.getPrintersAsync();

    const printerInfo = availablePrinters.map((printer) => ({
      name: printer.name,
      displayName: printer.displayName,
      description: printer.description,
      status: printer.status,
      isDefault: printer.isDefault,
      statusText:
        printer.status === 0
          ? "Disponible"
          : printer.status === 1
          ? "Imprimiendo"
          : printer.status === 2
          ? "Error"
          : printer.status === 3
          ? "No disponible"
          : "Estado desconocido",
    }));

    console.log("📋 Información detallada de impresoras:", printerInfo);
    return printerInfo;
  } catch (error) {
    console.error("❌ Error al obtener impresoras:", error);
    throw error;
  }
});

ipcMain.handle("print-closing", async (_, closingData) => {
  try {
    const tempDir = os.tmpdir();
    const tempDataPath = path.join(tempDir, `closing-data-${Date.now()}.json`);
    await fsPromises.writeFile(
      tempDataPath,
      JSON.stringify(closingData),
      "utf8"
    );

    const isProduction = process.env.NODE_ENV !== "development";
    let phpScriptPath;
    if (isProduction) {
      phpScriptPath = path.join(
        process.resourcesPath,
        "resources",
        "closing_printer.php"
      );
    } else {
      phpScriptPath = path.join(
        app.getAppPath(),
        "resources",
        "closing_printer.php"
      );
    }

    // LOG: Verificar existencia del script PHP
    if (!fs.existsSync(phpScriptPath)) {
      console.error("❌ No se encontró el script PHP:", phpScriptPath);
      return {
        success: false,
        printerError: `No se encontró el script PHP: ${phpScriptPath}`,
        message: `No se encontró el script PHP: ${phpScriptPath}`,
      };
    }

    console.log("------ INFO DE IMPRESIÓN CIERRE ------");
    console.log(`NODE_ENV: ${process.env.NODE_ENV}`);
    console.log(`PHP Script: ${phpScriptPath}`);
    console.log(`Datos: ${tempDataPath}`);

    return new Promise((resolve, reject) => {
      exec(
        `set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`,
        async (error, stdout, stderr) => {
          try {
            await fsPromises.unlink(tempDataPath);
            console.log("[PRINT-CLOSING] error:", error);
            console.log("[PRINT-CLOSING] stdout:", stdout);
            console.log("[PRINT-CLOSING] stderr:", stderr);
            if (error) {
              console.error("[PRINT-CLOSING] Código de salida:", error.code);
            }
            let printerError = null;
            if (stderr && stderr.includes("Error: ")) {
              const errorMatch = stderr.match(/Error: (.*?)(\n|$)/);
              if (errorMatch && errorMatch[1]) {
                printerError = errorMatch[1];
              }
            }
            if (error) {
              resolve({
                success: false,
                printerError: printerError || error.message,
                message: printerError
                  ? `Error de impresión: ${printerError}`
                  : error.message
                  ? `Error de impresión: ${error.message}`
                  : "Error desconocido",
              });
              return;
            }
            if (printerError) {
              resolve({
                success: false,
                printerError,
                message: `Error de impresión: ${printerError}`,
              });
              return;
            }
            console.log("Salida del script PHP cierre:", stdout);
            resolve({
              success: true,
              printerError: null,
              message: "Ticket de cierre impreso correctamente",
            });
          } catch (err) {
            console.error("Error en el callback:", err);
            reject(err);
          }
        }
      );
    });
  } catch (error) {
    console.error("Error en impresión de cierre:", error);
    throw error;
  }
});

// Log de confirmación de handlers registrados
console.log("✅ Todos los handlers IPC registrados:");
console.log("   - print-ticket");
console.log("   - print-afip-ticket");
console.log("   - print-closing");
console.log("   - toggle-devtools");
console.log("   - get-available-printers");
console.log("   - store-get, store-set, store-delete, store-has");
console.log("   - check-for-updates, download-update, install-update");
console.log("   - read-peso");
