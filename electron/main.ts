import { app, BrowserWindow, ipcMain } from "electron";
import * as path from "path";
import { fileURLToPath } from "url";
import { exec } from "child_process";
import fs from 'fs';
import * as fsPromises from 'fs/promises';
import os from "os";
import crypto from 'node:crypto';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Generar ID único para la aplicación
const generateAppId = () => {
  const storedId = app.getPath('userData') + '/app-id.txt';
  try {
    if (fs.existsSync(storedId)) {
      return fs.readFileSync(storedId, 'utf8');
    }
    const newId = crypto.randomBytes(32).toString('hex');
    fs.writeFileSync(storedId, newId);
    return newId;
  } catch (error) {
    console.error('Error al generar/leer ID:', error);
    return crypto.randomBytes(32).toString('hex');
  }
};

const APP_ID = generateAppId();

console.log('App ID generado:', APP_ID);

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
      devTools: process.env.NODE_ENV === "development",
    },
  });

  // En desarrollo, carga la URL del servidor de Vite
  if (process.env.NODE_ENV === "development") {
    mainWindow.loadURL("http://localhost:5173");
    mainWindow.webContents.openDevTools();
  } else {
    // En producción, carga el archivo HTML construido
    mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));
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
    await fsPromises.writeFile(tempDataPath, JSON.stringify(orderData), 'utf8');

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
