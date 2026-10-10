import { execFileSync, execSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const soloLocal = process.argv.includes("--local");
const repo = "ginopastran/verdu-electron";
const clave = process.env.ANDEX_CLAVE_ACTUALIZACIONES ?? "D:\\dev\\claves\\andex-market.key";
const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));

if (!fs.existsSync(clave)) {
  throw new Error(`No está la clave privada de actualizaciones en ${clave}`);
}

execSync("npx tauri build", {
  stdio: "inherit",
  env: {
    ...process.env,
    TAURI_SIGNING_PRIVATE_KEY: fs.readFileSync(clave, "utf8"),
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? "",
  },
});

const objetivo = process.env.CARGO_TARGET_DIR ?? path.join("src-tauri", "target");
const carpeta = path.join(objetivo, "release", "bundle", "nsis");
const instalador = `AndexMarket_${version}_x64-setup.exe`;
const rutaInstalador = path.join(carpeta, instalador);
const contenido = fs.readFileSync(rutaInstalador);
const fecha = new Date().toISOString();

fs.writeFileSync(
  path.join(carpeta, "latest.json"),
  JSON.stringify(
    {
      version,
      notes: `AndexMarket ${version}`,
      pub_date: fecha,
      platforms: {
        "windows-x86_64": {
          signature: fs.readFileSync(`${rutaInstalador}.sig`, "utf8"),
          url: `https://github.com/${repo}/releases/download/v${version}/${instalador}`,
        },
      },
    },
    null,
    2,
  ),
);

const sha512 = createHash("sha512").update(contenido).digest("base64");
fs.writeFileSync(
  path.join(carpeta, "latest.yml"),
  [
    `version: ${version}`,
    "files:",
    `  - url: ${instalador}`,
    `    sha512: ${sha512}`,
    `    size: ${contenido.length}`,
    `path: ${instalador}`,
    `sha512: ${sha512}`,
    `releaseDate: '${fecha}'`,
    "",
  ].join("\n"),
);

console.log(`Instalador y manifiestos listos en ${carpeta}`);

if (!soloLocal) {
  execFileSync(
    "gh",
    [
      "release",
      "create",
      `v${version}`,
      "--repo",
      repo,
      "--draft",
      "--title",
      `AndexMarket ${version}`,
      "--notes",
      `AndexMarket ${version}`,
      rutaInstalador,
      path.join(carpeta, "latest.json"),
      path.join(carpeta, "latest.yml"),
    ],
    { stdio: "inherit" },
  );
  console.log(`Release v${version} creado como borrador en ${repo}. Publicalo desde GitHub cuando esté probado.`);
}
