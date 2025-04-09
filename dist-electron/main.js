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
const APP_ID = "b2fa850b9a1782595da81d0699892e93a3f29f9d5b0fd74ef4ede03f05658942";
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
    const iconPath = path.join(__dirname, process.env.NODE_ENV === "development"
        ? "../../public/icon.png"
        : "../icon.png");
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
    }
    else {
        // En producción, oculta el menú nativo (File, Edit, View, Window, Help)
        mainWindow.setMenuBarVisibility(false);
        // En producción, carga el archivo HTML construido
        mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));
        // Configurar protocolo para recursos estáticos
        mainWindow.webContents.session.protocol.registerFileProtocol("app", (request, callback) => {
            const url = request.url.substr(6);
            callback({ path: path.normalize(`${__dirname}/../dist/${url}`) });
        });
        // Manejar navegación para SPA
        mainWindow.webContents.on("did-fail-load", (event, errorCode, errorDescription, validatedURL) => {
            if (errorCode === -6) {
                mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));
            }
        });
    }
}
app.whenReady().then(() => {
    createWindow();
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
            }
            else {
                console.warn(`Logo no encontrado en: ${logoSourcePath}`);
            }
        }
        catch (err) {
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
        }
        catch (error) {
            console.error("Error al leer el peso:", error);
            return 0;
        }
    });
    app.on("activate", () => {
        if (BrowserWindow.getAllWindows().length === 0)
            createWindow();
    });
});
app.on("window-all-closed", () => {
    if (process.platform !== "darwin")
        app.quit();
});
ipcMain.handle("print-ticket", async (_, orderData) => {
    try {
        const tempDir = os.tmpdir();
        const tempDataPath = path.join(tempDir, `order-data-${Date.now()}.json`);
        // Guardar datos de la orden en archivo temporal
        await fsPromises.writeFile(tempDataPath, JSON.stringify(orderData), "utf8");
        // Rutas del PHP y logo
        const isProduction = process.env.NODE_ENV !== "development";
        let phpScriptPath;
        if (isProduction) {
            phpScriptPath = path.join(process.resourcesPath, "resources", "ticket_printer.php");
            // En producción - Lógica simple: copiar el logo directamente junto al PHP
            console.log("------ CONFIGURACIÓN LOGO PRODUCCIÓN ------");
            // Posibles ubicaciones de origen del logo
            const possibleSources = [
                path.join(process.resourcesPath, "logo.png"),
                path.join(process.resourcesPath, "resources", "logo.png"),
                path.join(process.resourcesPath, "public", "logo.png"),
                path.join(app.getAppPath(), "resources", "logo.png"),
                path.join(app.getAppPath(), "public", "logo.png"),
                path.join(__dirname, "../resources", "logo.png"),
                path.join(app.getAppPath(), "public", "logos", "logo.png"),
                path.join(__dirname, "../public", "logo.png"),
            ];
            // Destino - siempre al lado del PHP script
            const logoDestPath = path.join(path.dirname(phpScriptPath), "logo.png");
            console.log(`Destino del logo: ${logoDestPath}`);
            // Verificar si el destino es escribible
            try {
                // Intentar acceder al directorio destino para verificar permisos
                const destDir = path.dirname(logoDestPath);
                const testFile = path.join(destDir, `test_write_${Date.now()}.tmp`);
                fs.writeFileSync(testFile, "test");
                fs.unlinkSync(testFile);
                console.log(`✅ Directorio destino ${destDir} tiene permisos de escritura`);
            }
            catch (err) {
                console.error(`❌ ALERTA: No se puede escribir en el directorio destino:`, err);
                console.log(`Intentando continuar de todas formas...`);
            }
            // Buscar y copiar el logo
            let found = false;
            for (const src of possibleSources) {
                console.log(`Buscando logo en: ${src}`);
                if (fs.existsSync(src)) {
                    console.log(`Logo encontrado en: ${src}`);
                    try {
                        const logoStats = fs.statSync(src);
                        console.log(`Tamaño original: ${logoStats.size} bytes`);
                        // Verificar que sea un archivo válido
                        if (logoStats.size === 0) {
                            console.error(`Logo encontrado pero tiene tamaño cero: ${src}`);
                            continue;
                        }
                        console.log(`Copiando a: ${logoDestPath}`);
                        fs.copyFileSync(src, logoDestPath);
                        // Verificar que se copió correctamente
                        if (fs.existsSync(logoDestPath)) {
                            const destStats = fs.statSync(logoDestPath);
                            console.log(`Logo copiado exitosamente (${destStats.size} bytes)`);
                            found = true;
                            break;
                        }
                        else {
                            console.error(`No se pudo verificar la copia del logo en: ${logoDestPath}`);
                        }
                    }
                    catch (err) {
                        console.error(`Error copiando logo desde ${src}:`, err);
                    }
                }
            }
            // Como último recurso, generar un logo mínimo
            if (!found) {
                try {
                    console.log("Intentando generar logo mínimo con PHP...");
                    const logoGenPath = path.join(tempDir, "gen_logo.php");
                    const logoContent = `<?php
          $img = imagecreatetruecolor(400, 100);
          $white = imagecolorallocate($img, 255, 255, 255);
          $black = imagecolorallocate($img, 0, 0, 0);
          imagefill($img, 0, 0, $white);
          imagestring($img, 5, 150, 40, 'ISELIN II', $black);
          imagepng($img, '${logoDestPath.replace(/\\/g, "\\\\")}');
          echo "Logo creado";
          ?>`;
                    fs.writeFileSync(logoGenPath, logoContent);
                    // Ejecutar PHP para generar la imagen
                    const { error, stdout, stderr } = await new Promise((resolve) => {
                        exec(`php "${logoGenPath}"`, (error, stdout, stderr) => {
                            resolve({ error, stdout, stderr });
                        });
                    });
                    if (error) {
                        console.error("Error generando logo:", stderr || error.message);
                    }
                    else {
                        console.log("Logo generado:", stdout);
                        if (fs.existsSync(logoDestPath)) {
                            console.log(`Logo generado verificado: ${fs.statSync(logoDestPath).size} bytes`);
                        }
                    }
                    // Limpiar archivo temporal
                    fs.unlinkSync(logoGenPath);
                }
                catch (genErr) {
                    console.error("Error generando logo:", genErr);
                }
            }
        }
        else {
            // En desarrollo - Ruta normal
            phpScriptPath = path.join(app.getAppPath(), "resources", "ticket_printer.php");
        }
        console.log("------ INFO DE IMPRESIÓN ------");
        console.log(`NODE_ENV: ${process.env.NODE_ENV}`);
        console.log(`PHP Script: ${phpScriptPath}`);
        console.log(`Datos: ${tempDataPath}`);
        return new Promise((resolve, reject) => {
            exec(`set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`, async (error, stdout, stderr) => {
                try {
                    // Limpiar archivo temporal
                    await fsPromises.unlink(tempDataPath);
                    if (error) {
                        console.error("Error al ejecutar PHP:", error);
                        console.error("Salida de error:", stderr);
                        reject(new Error(`Error al imprimir: ${stderr || error.message}`));
                        return;
                    }
                    console.log("Salida del script PHP:", stdout);
                    resolve({
                        success: true,
                        message: "Ticket impreso correctamente",
                    });
                }
                catch (err) {
                    console.error("Error en el callback:", err);
                    reject(err);
                }
            });
        });
    }
    catch (error) {
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
        // Crear una copia para depuración
        const debugDataPath = path.join(app.getPath("desktop"), `debug-closing-data-${Date.now()}.json`);
        // Guardar una copia de los datos recibidos para análisis
        try {
            await fsPromises.writeFile(debugDataPath, JSON.stringify(closingData, null, 2), "utf8");
            console.log(`Copia de depuración guardada en: ${debugDataPath}`);
        }
        catch (err) {
            console.error("Error guardando archivo de depuración:", err);
        }
        // Logs completos y detallados
        console.log("==========================================");
        console.log("DATOS COMPLETOS RECIBIDOS PARA CIERRE:");
        console.log("==========================================");
        console.log(JSON.stringify(closingData, null, 2));
        // Análisis específico de los datos críticos
        console.log("==========================================");
        console.log("ANÁLISIS DE DATOS CRÍTICOS:");
        console.log("==========================================");
        console.log("Total ventas:", closingData.totalVentas);
        console.log("Cantidad ventas:", closingData.cantidadVentas);
        // Análisis profundo de la estructura ventasPorMetodo
        console.log("ESTRUCTURA VENTAS POR MÉTODO:");
        if (closingData.ventasPorMetodo) {
            console.log("Tipo de ventasPorMetodo:", typeof closingData.ventasPorMetodo);
            console.log("Es array:", Array.isArray(closingData.ventasPorMetodo));
            console.log("Claves disponibles:", Object.keys(closingData.ventasPorMetodo));
            if (typeof closingData.ventasPorMetodo === "object") {
                for (const key in closingData.ventasPorMetodo) {
                    console.log(`Propiedad '${key}':`, JSON.stringify(closingData.ventasPorMetodo[key]));
                }
            }
            if (closingData.ventasPorMetodo.ventasPorMetodo) {
                // Nueva estructura API
                console.log("DESGLOSE POR MÉTODO DE PAGO (NUEVA ESTRUCTURA):");
                const metodos = closingData.ventasPorMetodo.ventasPorMetodo;
                for (const metodo in metodos) {
                    console.log(`- ${metodo}: ${metodos[metodo]}`);
                }
                // Calcular suma para verificación
                let suma = 0;
                for (const metodo in metodos) {
                    suma += metodos[metodo];
                }
                console.log("Suma calculada de ventas por método:", suma);
                if (Math.abs(suma - closingData.totalVentas) > 0.01) {
                    console.log("⚠️ ADVERTENCIA: Discrepancia entre la suma de métodos y el total de ventas");
                    console.log("Diferencia:", closingData.totalVentas - suma);
                }
            }
            else {
                // Estructura antigua
                console.log("DESGLOSE POR MÉTODO DE PAGO (ESTRUCTURA ANTIGUA):");
                console.log(JSON.stringify(closingData.ventasPorMetodo, null, 2));
            }
        }
        else {
            console.log("❌ No hay datos de ventas por método");
        }
        // Análisis de la estructura de vendedores
        console.log("VENDEDORES:");
        if (closingData.ventasPorMetodo &&
            closingData.ventasPorMetodo.ventasPorVendedor) {
            const vendedores = closingData.ventasPorMetodo.ventasPorVendedor;
            for (const vendedor of vendedores) {
                console.log(`- ${vendedor.nombre} (${vendedor.email}): ${vendedor.totalVentas} (${vendedor.cantidadVentas} ventas)`);
            }
            // Sumar totales de vendedores para verificación
            let sumaVendedores = 0;
            for (const vendedor of vendedores) {
                sumaVendedores += vendedor.totalVentas;
            }
            console.log("Suma calculada de ventas por vendedor:", sumaVendedores);
            if (Math.abs(sumaVendedores - closingData.totalVentas) > 0.01) {
                console.log("⚠️ ADVERTENCIA: Discrepancia entre la suma de vendedores y el total de ventas");
                console.log("Diferencia:", closingData.totalVentas - sumaVendedores);
            }
        }
        else {
            console.log("❌ No hay datos de ventas por vendedor");
        }
        console.log("==========================================");
        await fsPromises.writeFile(tempDataPath, JSON.stringify(closingData), "utf8");
        // Rutas del PHP y logo
        const isProduction = process.env.NODE_ENV !== "development";
        let phpScriptPath;
        if (isProduction) {
            phpScriptPath = path.join(process.resourcesPath, "resources", "closing_printer.php");
            // En producción - Lógica simple: copiar el logo directamente junto al PHP
            console.log("------ CONFIGURACIÓN LOGO CIERRE PRODUCCIÓN ------");
            // Posibles ubicaciones de origen del logo
            const possibleSources = [
                path.join(process.resourcesPath, "logo.png"),
                path.join(process.resourcesPath, "resources", "logo.png"),
                path.join(process.resourcesPath, "public", "logo.png"),
                path.join(app.getAppPath(), "resources", "logo.png"),
                path.join(app.getAppPath(), "public", "logo.png"),
                path.join(__dirname, "../resources", "logo.png"),
                path.join(app.getAppPath(), "public", "logos", "logo.png"),
                path.join(__dirname, "../public", "logo.png"),
            ];
            // Destino - siempre al lado del PHP script
            const logoDestPath = path.join(path.dirname(phpScriptPath), "logo.png");
            console.log(`Destino del logo: ${logoDestPath}`);
            // Verificar si el destino es escribible
            try {
                // Intentar acceder al directorio destino para verificar permisos
                const destDir = path.dirname(logoDestPath);
                const testFile = path.join(destDir, `test_write_${Date.now()}.tmp`);
                fs.writeFileSync(testFile, "test");
                fs.unlinkSync(testFile);
                console.log(`✅ Directorio destino ${destDir} tiene permisos de escritura`);
            }
            catch (err) {
                console.error(`❌ ALERTA: No se puede escribir en el directorio destino:`, err);
                console.log(`Intentando continuar de todas formas...`);
            }
            // Buscar y copiar el logo
            let found = false;
            for (const src of possibleSources) {
                console.log(`Buscando logo en: ${src}`);
                if (fs.existsSync(src)) {
                    console.log(`Logo encontrado en: ${src}`);
                    try {
                        const logoStats = fs.statSync(src);
                        console.log(`Tamaño original: ${logoStats.size} bytes`);
                        // Verificar que sea un archivo válido
                        if (logoStats.size === 0) {
                            console.error(`Logo encontrado pero tiene tamaño cero: ${src}`);
                            continue;
                        }
                        console.log(`Copiando a: ${logoDestPath}`);
                        fs.copyFileSync(src, logoDestPath);
                        // Verificar que se copió correctamente
                        if (fs.existsSync(logoDestPath)) {
                            const destStats = fs.statSync(logoDestPath);
                            console.log(`Logo copiado exitosamente (${destStats.size} bytes)`);
                            found = true;
                            break;
                        }
                        else {
                            console.error(`No se pudo verificar la copia del logo en: ${logoDestPath}`);
                        }
                    }
                    catch (err) {
                        console.error(`Error copiando logo desde ${src}:`, err);
                    }
                }
            }
            // Como último recurso, generar un logo mínimo
            if (!found) {
                try {
                    console.log("Intentando generar logo mínimo con PHP...");
                    const logoGenPath = path.join(tempDir, "gen_logo_closing.php");
                    const logoContent = `<?php
          $img = imagecreatetruecolor(400, 100);
          $white = imagecolorallocate($img, 255, 255, 255);
          $black = imagecolorallocate($img, 0, 0, 0);
          imagefill($img, 0, 0, $white);
          imagestring($img, 5, 150, 40, 'ISELIN II', $black);
          imagepng($img, '${logoDestPath.replace(/\\/g, "\\\\")}');
          echo "Logo creado";
          ?>`;
                    fs.writeFileSync(logoGenPath, logoContent);
                    // Ejecutar PHP para generar la imagen
                    const { error, stdout, stderr } = await new Promise((resolve) => {
                        exec(`php "${logoGenPath}"`, (error, stdout, stderr) => {
                            resolve({ error, stdout, stderr });
                        });
                    });
                    if (error) {
                        console.error("Error generando logo para cierre:", stderr || error.message);
                    }
                    else {
                        console.log("Logo generado para cierre:", stdout);
                        if (fs.existsSync(logoDestPath)) {
                            console.log(`Logo generado verificado: ${fs.statSync(logoDestPath).size} bytes`);
                        }
                    }
                    // Limpiar archivo temporal
                    fs.unlinkSync(logoGenPath);
                }
                catch (genErr) {
                    console.error("Error generando logo para cierre:", genErr);
                }
            }
        }
        else {
            // En desarrollo - Ruta normal
            phpScriptPath = path.join(app.getAppPath(), "resources", "closing_printer.php");
        }
        console.log("------ INFO DE IMPRESIÓN CIERRE ------");
        console.log(`NODE_ENV: ${process.env.NODE_ENV}`);
        console.log(`PHP Script: ${phpScriptPath}`);
        console.log(`Datos: ${tempDataPath}`);
        return new Promise((resolve, reject) => {
            exec(`set NODE_ENV=${process.env.NODE_ENV}&& php "${phpScriptPath}" "${tempDataPath}"`, async (error, stdout, stderr) => {
                try {
                    // Limpiar archivo temporal
                    await fsPromises.unlink(tempDataPath);
                    let printerError = null;
                    // Verificar si hay errores en la salida del script PHP relacionados con la impresora
                    if (stderr && stderr.includes("Error con la impresora:")) {
                        // Extraer el mensaje de error específico
                        const errorMatch = stderr.match(/Error con la impresora: (.*?)(\n|$)/);
                        if (errorMatch && errorMatch[1]) {
                            printerError = errorMatch[1];
                        }
                        else {
                            printerError = "Error desconocido con la impresora";
                        }
                    }
                    // Verificar también errores generales del comando
                    if (error) {
                        if (!printerError) {
                            printerError =
                                error.message || "Error al ejecutar el comando de impresión";
                        }
                        // En caso de error grave (no solo de impresora), rechazar la promesa
                        if (!stderr.includes("Error con la impresora:")) {
                            reject(new Error(`Error al imprimir: ${stderr || error.message}`));
                            return;
                        }
                    }
                    // Siempre resolver con información sobre si hubo un error de impresora
                    resolve({
                        success: !error || stderr.includes("Error con la impresora:"), // Consideramos éxito parcial si solo falló la impresora
                        printerError,
                        message: printerError
                            ? "Cierre registrado pero no se pudo imprimir"
                            : "Cierre impreso correctamente",
                    });
                }
                catch (err) {
                    reject(err);
                }
            });
        });
    }
    catch (error) {
        console.error("Error en impresión del cierre:", error);
        throw error;
    }
});
