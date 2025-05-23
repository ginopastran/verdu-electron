import { useState, useEffect } from "react";
import { AvailableProduct } from "../types";
import { toast } from "sonner";

interface UseProductSearchProps {
  API_URL: string;
  headers: Record<string, string>;
}

export function useProductSearch({ API_URL, headers }: UseProductSearchProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [showResults, setShowResults] = useState(false);
  const [searchResults, setSearchResults] = useState<AvailableProduct[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [selectedProduct, setSelectedProduct] =
    useState<AvailableProduct | null>(null);
  const [availableProducts, setAvailableProducts] = useState<
    AvailableProduct[]
  >([]);

  useEffect(() => {
    const fetchProducts = async () => {
      try {
        const response = await fetch(`${API_URL}/api/productos`, { headers });
        if (!response.ok) {
          throw new Error("Error al cargar productos");
        }
        const data = await response.json();

        const transformedProducts = data.map((p: any) => ({
          id: p.id,
          name: p.nombre,
          pricePerUnit: p.precio,
          unit: p.tipoMedida,
          costo: p.costo,
          codigoBarras: p.codigoBarras,
        }));

        setAvailableProducts(transformedProducts);
      } catch (error) {
        console.error("Error al cargar productos:", error);
        toast.error("Error al cargar los productos");
      }
    };

    fetchProducts();
  }, [API_URL]);

  useEffect(() => {
    if (searchQuery) {
      const filtered = availableProducts.filter((product) =>
        product.name.toLowerCase().includes(searchQuery.toLowerCase())
      );

      const sortedResults = filtered.sort((a, b) => {
        const aStartsWithQuery = a.name
          .toLowerCase()
          .startsWith(searchQuery.toLowerCase());
        const bStartsWithQuery = b.name
          .toLowerCase()
          .startsWith(searchQuery.toLowerCase());

        if (aStartsWithQuery && !bStartsWithQuery) return -1;
        if (!aStartsWithQuery && bStartsWithQuery) return 1;

        const aStartsWithNumber =
          /^\d+/.test(a.name) &&
          a.name.toLowerCase().includes(searchQuery.toLowerCase());
        const bStartsWithNumber =
          /^\d+/.test(b.name) &&
          b.name.toLowerCase().includes(searchQuery.toLowerCase());

        if (aStartsWithNumber && !bStartsWithNumber) return -1;
        if (!aStartsWithNumber && bStartsWithNumber) return 1;

        return a.name.localeCompare(b.name);
      });

      setSearchResults(sortedResults);
      setShowResults(true);
      setSelectedIndex(sortedResults.length > 0 ? 0 : -1);
    } else {
      setSearchResults([]);
      setShowResults(false);
      setSelectedIndex(-1);
    }
  }, [searchQuery, availableProducts]);

  const handleSearchChange = (value: string) => {
    setSearchQuery(value);
  };

  const handleProductSelect = (product: AvailableProduct) => {
    setSelectedProduct(product);
    setShowResults(false);
    setSearchQuery("");
  };

  return {
    searchQuery,
    showResults,
    searchResults,
    selectedIndex,
    selectedProduct,
    setSelectedProduct,
    handleSearchChange,
    handleProductSelect,
  };
}
