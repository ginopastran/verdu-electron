import { useCallback } from "react";

const BASE_URL =
  import.meta.env.VITE_API_URL || "https://verdulerias-soft.vercel.app";

export async function fetchTransaccionesDelDia(
  vendedorId: number,
  includeFacturas: boolean = false
) {
  if (!vendedorId) throw new Error("vendedorId requerido");

  const endpoints = [
    fetch(`${BASE_URL}/api/ordenes/vendedor/${vendedorId}`, {
      credentials: "include",
    }).then((r) => r.json()),
  ];

  if (includeFacturas) {
    endpoints.push(
      fetch(`${BASE_URL}/api/facturas/vendedor/${vendedorId}`, {
        credentials: "include",
      }).then((r) => r.json())
    );
  }

  const [ordenes, facturas = []] = await Promise.all(endpoints);

  const ord = (ordenes || []).map((o: any) => ({ ...o, tipo: "orden" }));
  const fac = (facturas || []).map((f: any) => {
    const rawFecha = f.fecha || f.fechaEmision || f.fechaFactura || f.createdAt;

    return {
      ...f,
      tipo: "facturaB",
      // Mantener la fecha original sin aplicar desplazamientos manuales; la vista hará la conversión a ART
      fecha: rawFecha,
      metodoPago: f.metodoPago || f.formaPago || "N/A",
    };
  });

  return [...ord, ...fac].sort(
    (a, b) => new Date(b.fecha).getTime() - new Date(a.fecha).getTime()
  );
}

export function useFetchTransaccionesDelDia(
  API_URL: string,
  includeFacturas: boolean
) {
  return useCallback(
    (vendedorId: number) =>
      fetchTransaccionesDelDia(vendedorId, includeFacturas),
    [includeFacturas]
  );
}
