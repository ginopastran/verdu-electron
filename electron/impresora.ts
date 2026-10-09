import { execFile } from "child_process";

export const IMPRESORA_DEFAULT = "TP806L";

const RECURSO_VALIDO = /^[A-Za-z0-9_-]+( [A-Za-z0-9_-]+)*$/;

export const esRecursoValido = (nombre: string) => RECURSO_VALIDO.test(nombre);

export function recursoParaImpresora(nombre: string) {
  const limpio = nombre
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/[^A-Za-z0-9_ -]/g, " ")
    .trim()
    .replace(/\s+/g, " ")
    .slice(0, 80)
    .trim();
  return limpio || "POS";
}

const textoPs = (valor: string) => `'${valor.replace(/'/g, "''")}'`;

function powershell(script: string) {
  return new Promise<string>((resolve, reject) => {
    execFile(
      "powershell.exe",
      ["-NoProfile", "-NonInteractive", "-Command", script],
      { windowsHide: true },
      (error, stdout, stderr) => (error ? reject(new Error(stderr.trim() || error.message)) : resolve(stdout.trim()))
    );
  });
}

export async function compartirImpresora(nombre: string) {
  const datos = JSON.parse(
    await powershell(`Get-Printer -Name ${textoPs(nombre)} | Select-Object Shared,ShareName | ConvertTo-Json -Compress`)
  ) as { Shared: boolean; ShareName: string | null };
  if (datos.Shared && datos.ShareName && esRecursoValido(datos.ShareName)) return datos.ShareName;
  const recurso = recursoParaImpresora(nombre);
  await powershell(`Set-Printer -Name ${textoPs(nombre)} -Shared $true -ShareName ${textoPs(recurso)}`);
  return recurso;
}
