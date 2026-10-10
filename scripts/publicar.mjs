import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const soloLocal = process.argv.includes("--local");
const repo = "ginopastran/verdu-electron";
const clave = process.env.ANDEX_CLAVE_ACTUALIZACIONES ?? "D:\\dev\\claves\\andex-market.key";
const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));
const versionApp = fs.readFileSync(path.join("apps", "andexmarket", "Cargo.toml"), "utf8").match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (versionApp !== version) {
  throw new Error(`La versión de package.json (${version}) no coincide con la de apps/andexmarket/Cargo.toml (${versionApp})`);
}
if (!fs.existsSync(clave)) {
  throw new Error(`No está la clave privada de actualizaciones en ${clave}`);
}

const makensis =
  process.env.MAKENSIS ??
  [
    path.join(process.env.LOCALAPPDATA ?? "", "tauri", "NSIS", "makensis.exe"),
    "C:\\Program Files (x86)\\NSIS\\makensis.exe",
    "C:\\Program Files\\NSIS\\makensis.exe",
  ].find((ruta) => fs.existsSync(ruta)) ??
  "makensis";

const correr = (programa, argumentos) => execFileSync(programa, argumentos, { stdio: "inherit" });

correr("cargo", ["build", "--release", "-p", "andexmarket"]);

const objetivo = path.resolve(process.env.CARGO_TARGET_DIR ?? "target");
const ejecutable = path.join(objetivo, "release", "andexmarket.exe");
const carpeta = path.join(objetivo, "release", "instalador");
const instalador = `AndexMarket_${version}_x64-setup.exe`;
const rutaInstalador = path.join(carpeta, instalador);
fs.mkdirSync(carpeta, { recursive: true });

correr(makensis, [
  "/V2",
  `/DVERSION=${version}`,
  `/DEJECUTABLE=${ejecutable}`,
  `/DSALIDA=${rutaInstalador}`,
  path.resolve("empaquetado", "instalador.nsi"),
]);

correr("cargo", ["run", "-q", "-p", "actualizador", "--features", "firmar", "--bin", "firmar", "--", clave, rutaInstalador]);

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

console.log(`Instalador, firma y manifiestos listos en ${carpeta}`);

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
