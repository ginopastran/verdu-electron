import { app, BrowserWindow, ipcMain, dialog } from "electron";
import * as path from "path";
import { fileURLToPath } from "url";
import { exec } from "child_process";
import * as fsPromises from "fs/promises";
import os from "os";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// APP_ID fijo para toda la aplicación
const APP_ID =
  "b2fa850b9a1782595da81d0699892e93a3f29f9d5b0fd74ef4ede03f05658942";

// Hacer disponible el APP_ID para el proceso de renderizado
process.env.VITE_APP_ID = APP_ID;

console.log("App ID:", APP_ID);

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
    title: "VerduSoft",
    icon: iconPath,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      devTools: true,
      webSecurity: false,
    },
  });

  // En desarrollo, carga la URL del servidor de Vite
  if (process.env.NODE_ENV === "development") {
    mainWindow.loadURL("http://localhost:5173");
    mainWindow.webContents.openDevTools();
  } else {
    // En producción, carga el archivo HTML construido
    mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));

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

app.whenReady().then(() => {
  createWindow();

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

    // Guardar datos de la orden en archivo temporal
    await fsPromises.writeFile(tempDataPath, JSON.stringify(orderData), "utf8");

    // Ejecutar script PHP
    const phpScriptPath = path.join(
      app.getAppPath(),
      "resources",
      "ticket_printer.php"
    );

    return new Promise((resolve, reject) => {
      exec(
        `php "${phpScriptPath}" "${tempDataPath}"`,
        async (error, stdout, stderr) => {
          try {
            // Limpiar archivo temporal
            await fsPromises.unlink(tempDataPath);

            if (error) {
              console.error("Error al imprimir:", error);
              reject(new Error(stderr));
              return;
            }

            resolve({ success: true, message: "Ticket impreso correctamente" });
          } catch (err) {
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

// Agregar un nuevo manejador IPC para DevTools
ipcMain.handle("toggle-devtools", () => {
  const win = BrowserWindow.getFocusedWindow();
  if (win) {
    win.webContents.toggleDevTools();
  }
});
