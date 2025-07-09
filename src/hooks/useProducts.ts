import { useState, useEffect } from "react";
import { toast } from "sonner";
import { AvailableProduct } from "@/hooks/useProductSearch";

// Tipo para los productos que vienen del backend
interface BackendProduct {
  id: number;
  nombre: string;
  precio: number;
  tipoMedida: string;
  costo: number;
  codigoBarras: string | null;
  plu: string | null;
}

export const useProducts = (API_URL: string, appId: string | null) => {
  const [availableProducts, setAvailableProducts] = useState<
    AvailableProduct[]
  >([]);

  const fetchAllProducts = async (limit = 150) => {
    try {
      let page = 1;
      let hasNext = true;
      let allProducts: BackendProduct[] = [];

      while (hasNext) {
        const response = await fetch(
          `${API_URL}/api/productos?limit=${limit}&page=${page}`,
          {
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
            credentials: "include",
          }
        );

        if (response.status === 401 || response.status === 403) {
          throw new Error("No autorizado: inicia sesión nuevamente");
        }

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        const productosPage = data.productos || data.data || [];
        allProducts = allProducts.concat(productosPage);
        hasNext = data.pagination?.hasNext ?? false;
        page += 1;
      }

      // Transformar los productos al formato usado en el frontend
      const transformedProducts = allProducts.map(
        (p: BackendProduct): AvailableProduct => ({
          id: p.id,
          name: p.nombre,
          pricePerUnit: p.precio,
          unit: p.tipoMedida,
          costo: p.costo,
          codigoBarras: p.codigoBarras,
          plu: p.plu,
        })
      );

      console.log("✅ Productos cargados:", transformedProducts.length);
      setAvailableProducts(transformedProducts);
    } catch (error) {
      console.error("Error al cargar productos:", error);
      toast.error(
        (error as Error).message || "Error inesperado al cargar los productos"
      );
    }
  };

  useEffect(() => {
    fetchAllProducts();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [API_URL, appId]);

  return { availableProducts, fetchAllProducts };
};
