import { app, BrowserWindow, ipcMain, dialog } from "electron";
import * as path from "path";
import { fileURLToPath } from "url";
import { exec } from "child_process";
import * as fsPromises from "fs/promises";
import os from "os";
import * as fs from "fs";

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

  if (process.env.NODE_ENV === "development") {
    mainWindow.loadURL("http://localhost:5173");
    mainWindow.webContents.openDevTools();
  } else {
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

app.whenReady().then(() => {
  createWindow();

  // Copiar el logo a la carpeta resources en producción
  if (process.env.NODE_ENV !== "development") {
    try {
      const sourceLogoPath = path.join(__dirname, "../public/logo.png");
      const targetLogoPath = path.join(
        process.resourcesPath,
        "resources/logo.png"
      );

      // Asegurarse de que el directorio de destino existe
      const targetDir = path.dirname(targetLogoPath);
      if (!fs.existsSync(targetDir)) {
        fs.mkdirSync(targetDir, { recursive: true });
      }

      // Copiar el archivo solo si existe
      if (fs.existsSync(sourceLogoPath)) {
        fs.copyFileSync(sourceLogoPath, targetLogoPath);
        console.log(`Logo copiado de ${sourceLogoPath} a ${targetLogoPath}`);
      } else {
        console.error(`Logo no encontrado en ${sourceLogoPath}`);
      }
    } catch (error) {
      console.error("Error al copiar el logo:", error);
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

    // Guardar datos de la orden en archivo temporal
    await fsPromises.writeFile(tempDataPath, JSON.stringify(orderData), "utf8");

    // Obtener la ruta correcta del script PHP
    const phpScriptPath =
      process.env.NODE_ENV === "development"
        ? path.join(app.getAppPath(), "resources", "ticket_printer.php")
        : path.join(process.resourcesPath, "resources", "ticket_printer.php");

    console.log("Ruta del script PHP:", phpScriptPath);
    console.log("Ruta del archivo temporal:", tempDataPath);

    return new Promise((resolve, reject) => {
      exec(
        `set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`,
        async (error, stdout, stderr) => {
          try {
            // Limpiar archivo temporal
            await fsPromises.unlink(tempDataPath);

            if (error) {
              console.error("Error al ejecutar PHP:", error);
              console.error("Salida de error:", stderr);
              reject(
                new Error(`Error al imprimir: ${stderr || error.message}`)
              );
              return;
            }

            console.log("Salida del script PHP:", stdout);
            resolve({ success: true, message: "Ticket impreso correctamente" });
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

// Agregar un nuevo manejador IPC para DevTools
ipcMain.handle("toggle-devtools", () => {
  const win = BrowserWindow.getFocusedWindow();
  if (win) {
    win.webContents.toggleDevTools();
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

    const phpScriptPath =
      process.env.NODE_ENV === "development"
        ? path.join(app.getAppPath(), "resources", "closing_printer.php")
        : path.join(process.resourcesPath, "resources", "closing_printer.php");

    return new Promise((resolve, reject) => {
      exec(
        `set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`,
        async (error, stdout, stderr) => {
          try {
            await fsPromises.unlink(tempDataPath);
            if (error) {
              reject(
                new Error(`Error al imprimir: ${stderr || error.message}`)
              );
              return;
            }
            resolve({ success: true, message: "Cierre impreso correctamente" });
          } catch (err) {
            reject(err);
          }
        }
      );
    });
  } catch (error) {
    console.error("Error en impresión del cierre:", error);
    throw error;
  }
});
