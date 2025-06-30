import { useState, useEffect, useRef } from "react";

export interface AvailableProduct {
  id: number;
  name: string;
  pricePerUnit: number;
  unit: string;
  costo: number;
  codigoBarras: string | null;
  plu: string | null;
}

export function useProductSearch() {
  const [availableProducts, setAvailableProducts] = useState<
    AvailableProduct[]
  >([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<AvailableProduct[]>([]);
  const [showResults, setShowResults] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [selectedProduct, setSelectedProduct] =
    useState<AvailableProduct | null>(null);
  const [loading, setLoading] = useState(false);
  const [barcodeBuffer, setBarcodeBuffer] = useState<string>("");
  const [lastInputTime, setLastInputTime] = useState<number>(0);

  const appId =
    window.electron?.process?.argv
      ?.find((arg: string) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null;

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // Cargar productos al inicializar
  useEffect(() => {
    /**
     * Descarga todos los productos utilizando el endpoint paginado
     * /api/productos?limit=150&page=n hasta que pagination.hasNext sea false.
     * En caso de error 401/403 informa al usuario que debe iniciar sesión.
     */
    const fetchAllProducts = async (limit = 150) => {
      try {
        setLoading(true);
        const API_URL = import.meta.env.VITE_API_URL;
        let page = 1;
        let hasNext = true;
        let allProducts: any[] = [];

        while (hasNext) {
          const res = await fetch(
            `${API_URL}/api/productos?limit=${limit}&page=${page}`,
            {
              headers,
              credentials: "include",
            }
          );

          if (res.status === 401 || res.status === 403) {
            throw new Error("No autorizado: inicia sesión nuevamente");
          }

          if (!res.ok) {
            throw new Error(`Error HTTP ${res.status}`);
          }

          const data = await res.json();
          const productsArray = data.productos || data.data || [];
          allProducts = allProducts.concat(productsArray);

          // Controlar el bucle con la respuesta del backend
          hasNext = data.pagination?.hasNext ?? false;
          page += 1;
        }

        // Transformar los productos al formato usado en el frontend
        const transformedProducts = allProducts.map((p: any) => ({
          id: p.id,
          name: p.nombre,
          pricePerUnit: p.precio,
          unit: p.tipoMedida,
          costo: p.costo,
          codigoBarras: p.codigoBarras,
          plu: p.plu,
        }));

        setAvailableProducts(transformedProducts);
      } catch (error: any) {
        console.error("Error al cargar productos:", error);
        // En producción podríamos mostrar un toast aquí indicando la acción a seguir
      } finally {
        setLoading(false);
      }
    };

    // Iniciar la descarga al montar el hook
    fetchAllProducts();
  }, []);

  // Filtrar productos según búsqueda
  useEffect(() => {
    if (searchQuery) {
      // Filtrar productos que coinciden con la búsqueda
      const filtered = availableProducts.filter((product) =>
        product.name.toLowerCase().includes(searchQuery.toLowerCase())
      );

      // Ordenar los resultados: primero por relevancia, luego alfabéticamente
      const sortedResults = filtered.sort((a, b) => {
        // 1. Priorizar coincidencias exactas al inicio
        const aStartsWithQuery = a.name
          .toLowerCase()
          .startsWith(searchQuery.toLowerCase());
        const bStartsWithQuery = b.name
          .toLowerCase()
          .startsWith(searchQuery.toLowerCase());

        if (aStartsWithQuery && !bStartsWithQuery) return -1;
        if (!aStartsWithQuery && bStartsWithQuery) return 1;

        // 2. Priorizar coincidencias de ID/código (números al inicio)
        const aStartsWithNumber =
          /^\d+/.test(a.name) &&
          a.name.toLowerCase().includes(searchQuery.toLowerCase());
        const bStartsWithNumber =
          /^\d+/.test(b.name) &&
          b.name.toLowerCase().includes(searchQuery.toLowerCase());

        if (aStartsWithNumber && !bStartsWithNumber) return -1;
        if (!aStartsWithNumber && bStartsWithNumber) return 1;

        // 3. Orden alfabético para misma relevancia
        return a.name.localeCompare(b.name);
      });

      setSearchResults(sortedResults);
      setShowResults(true);

      // Seleccionar automáticamente el primer resultado
      setSelectedIndex(sortedResults.length > 0 ? 0 : -1);
    } else {
      setSearchResults([]);
      setShowResults(false);
      setSelectedIndex(-1);
    }
  }, [searchQuery, availableProducts]);

  // Manejar la navegación en resultados
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (searchResults.length === 0) return;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < searchResults.length - 1 ? prev + 1 : prev
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
        break;
      case "Enter":
        e.preventDefault();
        if (selectedIndex >= 0) {
          const product = searchResults[selectedIndex];
          setSelectedProduct(product);
          setShowResults(false);
          setSearchQuery("");
        } else if (searchResults.length > 0) {
          const product = searchResults[0];
          setSelectedProduct(product);
          setShowResults(false);
          setSearchQuery("");
        }
        break;
      case "Escape":
        setShowResults(false);
        setSearchQuery("");
        setSelectedIndex(-1);
        break;
    }
  };

  // Seleccionar un producto
  const handleProductSelect = (product: AvailableProduct) => {
    setSelectedProduct(product);
    setShowResults(false);
    setSearchQuery("");
  };

  // Manejar cambios en el input de búsqueda, incluido el escaneo de código de barras
  const handleSearchInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const currentTime = Date.now();
    const value = e.target.value;
    setSearchQuery(value);

    // Si el tiempo entre teclas es menor a 50ms, probablemente sea un scanner
    if (currentTime - lastInputTime < 50) {
      setBarcodeBuffer((prev) => prev + value.slice(-1));
    } else {
      setBarcodeBuffer(value);
    }

    setLastInputTime(currentTime);

    // Si detectamos un patrón de código de barras (números y longitud específica)
    if (/^\d{8,13}$/.test(value)) {
      const product = availableProducts.find((p) => p.codigoBarras === value);
      if (product) {
        handleProductSelect(product);
        return;
      }
    }

    // Detectar PLU directo (3-6 dígitos numéricos)
    if (/^\d{3,6}$/.test(value)) {
      const productByPlu = availableProducts.find((p) => p.plu === value);
      if (productByPlu) {
        handleProductSelect(productByPlu);
      }
    }
  };

  // Limpiar la selección
  const clearSelection = () => {
    setSelectedProduct(null);
    setSearchQuery("");
    setShowResults(false);
  };

  return {
    availableProducts,
    searchQuery,
    setSearchQuery,
    searchResults,
    showResults,
    setShowResults,
    selectedIndex,
    setSelectedIndex,
    selectedProduct,
    setSelectedProduct,
    loading,
    barcodeBuffer,
    handleKeyPress,
    handleProductSelect,
    handleSearchInputChange,
    clearSelection,
  };
}
