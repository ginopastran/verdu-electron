import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import fs from "fs";

// Plugin personalizado para manejar la lectura del peso
const weightReaderPlugin = () => {
  return {
    name: "weight-reader",
    configureServer(server: any) {
      server.middlewares.use("/api/peso", (req: any, res: any, next: any) => {
        if (req.method === "GET") {
          try {
            const weightPath = "C:\\Peso\\peso.json";
            console.log("🔧 Leyendo peso desde:", weightPath);

            if (fs.existsSync(weightPath)) {
              const data = fs.readFileSync(weightPath, "utf8");
              // Limpiar BOM y espacios extra antes del parsing
              const cleanData = data.replace(/^\uFEFF/, "").trim();
              console.log("Datos leídos:", JSON.stringify(cleanData));
              const weightData = JSON.parse(cleanData);
              console.log("Peso leído en desarrollo:", weightData.peso);

              res.setHeader("Content-Type", "application/json");
              res.setHeader("Access-Control-Allow-Origin", "*");
              res.end(JSON.stringify({ peso: weightData.peso }));
            } else {
              console.log("⚠️ Archivo de peso no encontrado, devolviendo 0");
              res.setHeader("Content-Type", "application/json");
              res.setHeader("Access-Control-Allow-Origin", "*");
              res.end(JSON.stringify({ peso: 0 }));
            }
          } catch (error) {
            console.error("Error leyendo peso:", error);
            res.setHeader("Content-Type", "application/json");
            res.setHeader("Access-Control-Allow-Origin", "*");
            res.end(JSON.stringify({ peso: 0 }));
          }
        } else {
          next();
        }
      });
    },
  };
};

export default defineConfig({
  plugins: [react(), weightReaderPlugin()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  base: "./",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
      },
    },
  },
  publicDir: "public",
  server: {
    host: true,
  },
});
