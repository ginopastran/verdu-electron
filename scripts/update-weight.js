const fs = require("fs");
const path = require("path");

const WEIGHT_FILE = "C:\\Peso\\peso.json";

// Obtener peso del argumento de línea de comandos
const newWeight = parseFloat(process.argv[2]);

if (isNaN(newWeight)) {
  console.log("❌ Error: Debes proporcionar un peso válido");
  console.log("Uso: node scripts/update-weight.js <peso>");
  console.log("Ejemplo: node scripts/update-weight.js 2.5");
  process.exit(1);
}

try {
  // Crear el directorio si no existe
  const dir = path.dirname(WEIGHT_FILE);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
    console.log(`📁 Directorio creado: ${dir}`);
  }

  // Escribir el nuevo peso
  const weightData = { peso: newWeight };
  fs.writeFileSync(WEIGHT_FILE, JSON.stringify(weightData, null, 2), "utf8");

  console.log(`⚖️ Peso actualizado: ${newWeight} kg`);
  console.log(`📄 Archivo: ${WEIGHT_FILE}`);
} catch (error) {
  console.error("❌ Error actualizando peso:", error.message);
  process.exit(1);
}
