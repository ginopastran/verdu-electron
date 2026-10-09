import { useEffect } from "react";

const ipc = () => (window as any).electron?.ipcRenderer;

async function aBase64(blob: Blob) {
  const buffer = new Uint8Array(await blob.arrayBuffer());
  let binario = "";
  for (let i = 0; i < buffer.length; i += 0x8000) {
    binario += String.fromCharCode(...buffer.subarray(i, i + 0x8000));
  }
  return btoa(binario);
}

export function useSincronizarLogo(apiUrl: string, activo: boolean) {
  useEffect(() => {
    if (!activo || !ipc()) return;
    let cancelado = false;
    (async () => {
      try {
        const res = await fetch(`${apiUrl}/api/business/logo-comprobante`, { credentials: "include" });
        if (cancelado) return;
        if (res.status === 404) {
          await ipc().invoke("logo-negocio-guardar", null);
        } else if (res.ok && res.headers.get("content-type")?.includes("image/png")) {
          await ipc().invoke("logo-negocio-guardar", await aBase64(await res.blob()));
        }
      } catch (error) {
        console.error("No se pudo sincronizar el logo del negocio:", error);
      }
    })();
    return () => {
      cancelado = true;
    };
  }, [apiUrl, activo]);
}
