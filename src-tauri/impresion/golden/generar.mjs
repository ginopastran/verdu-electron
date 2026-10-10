import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const recursos = process.argv[2];
const soloCarpeta = process.argv[3];
const golden = path.dirname(fileURLToPath(import.meta.url));
const scripts = {
  venta: "ticket_printer.php",
  factura: "factura_ticket_printer.php",
  afip: "afip_ticket_printer.php",
  cierre: "closing_printer.php",
};

for (const [carpeta, script] of Object.entries(scripts)) {
  const dir = path.join(golden, carpeta);
  if ((soloCarpeta && soloCarpeta !== carpeta) || !fs.existsSync(dir)) continue;
  for (const archivo of fs.readdirSync(dir).filter((a) => a.endsWith(".json"))) {
    const fixture = path.join(dir, archivo);
    const salida = fixture.replace(/\.json$/, ".bin");
    fs.rmSync(salida, { force: true });
    try {
      execFileSync(
        path.join(recursos, "php", "php.exe"),
        ["-c", path.join(recursos, "php", "php.ini"), "-d", `auto_prepend_file=${path.join(golden, "captura.php")}`, path.join(recursos, script), fixture],
        { cwd: path.dirname(golden), env: { ...process.env, ESCPOS_VENDOR: path.join(recursos, "vendor"), GOLDEN_SALIDA: salida }, stdio: ["ignore", "ignore", "pipe"] },
      );
    } catch (error) {
      const stderr = String(error.stderr ?? "");
      throw new Error(`${carpeta}/${archivo}: el PHP falló\n${stderr.split("\n").filter((l) => /Error|Fatal|Warning/.test(l)).join("\n")}`);
    }
    if (!fs.existsSync(salida)) throw new Error(`${carpeta}/${archivo}: el PHP no imprimió nada`);
    console.log(`${carpeta}/${archivo} -> ${fs.statSync(salida).size} bytes`);
  }
}
