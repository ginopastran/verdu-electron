import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

console.log("=== VERIFICACIÓN DE LOGO ANTES DE BUILD ===");

// Obtener el directorio actual en ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Rutas importantes
const rootDir = path.resolve(__dirname, "..");
const publicDir = path.join(rootDir, "public");
const resourcesDir = path.join(rootDir, "resources");
const logoPath = path.join(publicDir, "logo.png");
const resourcesLogoPath = path.join(resourcesDir, "logo.png");

// Verificar si existe el logo en public
console.log(`Verificando logo en: ${logoPath}`);
if (fs.existsSync(logoPath)) {
  const stats = fs.statSync(logoPath);
  console.log(`✅ Logo encontrado en public con tamaño: ${stats.size} bytes`);

  // Verificar si el directorio resources existe
  if (!fs.existsSync(resourcesDir)) {
    console.log(`Creando directorio resources: ${resourcesDir}`);
    fs.mkdirSync(resourcesDir, { recursive: true });
  }

  // Copiar logo a resources
  try {
    fs.copyFileSync(logoPath, resourcesLogoPath);
    console.log(`✅ Logo copiado a: ${resourcesLogoPath}`);

    if (fs.existsSync(resourcesLogoPath)) {
      const resourcesStats = fs.statSync(resourcesLogoPath);
      console.log(
        `✅ Logo en resources verificado con tamaño: ${resourcesStats.size} bytes`
      );
    }
  } catch (err) {
    console.error(`❌ Error al copiar logo a resources: ${err.message}`);
  }
} else {
  console.error("❌ ERROR: Logo no encontrado en public/logo.png");
  console.error(
    "Por favor, asegúrate de que exista el archivo public/logo.png"
  );
  process.exit(1);
}

console.log("=== VERIFICACIÓN DE LOGO COMPLETADA ===");
