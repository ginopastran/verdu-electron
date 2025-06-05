import { useState, useEffect } from "react";
import { toast } from "sonner";
import { AvailableProduct } from "@/hooks/useProductSearch";

export const useProducts = (API_URL: string, appId: string | null) => {
  const [availableProducts, setAvailableProducts] = useState<
    AvailableProduct[]
  >([]);

  const fetchProducts = async () => {
    try {
      const response = await fetch(`${API_URL}/api/productos/all`, {
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
      });

      if (!response.ok) {
        throw new Error("Error al cargar productos");
      }

      const data = await response.json();
      console.log("✅ Productos cargados:", data);
      setAvailableProducts(data);
    } catch (error) {
      console.error("Error al cargar productos:", error);
      toast.error("Error al cargar los productos");
    }
  };

  useEffect(() => {
    fetchProducts();
  }, [API_URL, appId]);

  return { availableProducts, fetchProducts };
};
