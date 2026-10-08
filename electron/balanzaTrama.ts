export type UnidadEntera = "gramos" | "kilos";

export function pesoDeTrama(trama: string, unidadEntera: UnidadEntera = "gramos"): number | null {
  const numero = trama.match(/[-+]?\d+(?:[.,]\d+)?/);
  if (!numero) return null;
  const texto = numero[0].replace(",", ".");
  const valor = Number(texto);
  if (!Number.isFinite(valor)) return null;
  const enKilos = texto.includes(".") || unidadEntera === "kilos";
  return Math.round(enKilos ? valor * 1000 : valor);
}

export function crearFiltroEstable(lecturas = 3, toleranciaGramos = 2) {
  const ultimas: number[] = [];
  return (gramos: number): number | null => {
    ultimas.push(gramos);
    if (ultimas.length > lecturas) ultimas.shift();
    if (ultimas.length < lecturas) return null;
    return Math.max(...ultimas) - Math.min(...ultimas) <= toleranciaGramos ? gramos : null;
  };
}
