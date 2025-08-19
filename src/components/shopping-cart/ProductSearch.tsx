import { useRef, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { useProductSearch, AvailableProduct } from "@/hooks/useProductSearch";
import { RefObject } from "react";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

interface ProductSearchProps {
  onProductSelect: (product: AvailableProduct) => void;
  inputRef: RefObject<HTMLInputElement | null>;
  onDirectAddToCart?: (product: AvailableProduct, quantity: number) => void;
}

export function ProductSearch({
  onProductSelect,
  inputRef,
  onDirectAddToCart,
}: ProductSearchProps) {
  const {
    searchQuery,
    searchResults,
    showResults,
    selectedIndex,
    selectedProduct,
    handleKeyPress,
    handleProductSelect,
    handleSearchInputChange,
    clearSelection,
  } = useProductSearch(onDirectAddToCart);

  // Obtener información del business para el cálculo de IVA
  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  // Efecto para detectar cuando el hook selecciona un producto
  useEffect(() => {
    if (selectedProduct) {
      onProductSelect(selectedProduct);
      clearSelection();
    }
  }, [selectedProduct, onProductSelect, clearSelection]);

  // Efecto para enfocar el input cuando se monta el componente
  useEffect(() => {
    const timer = setTimeout(() => {
      if (inputRef.current) {
        inputRef.current.focus();
        console.log("🎯 ProductSearch: Input enfocado al montar");
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [inputRef]);

  const selectProduct = (product: AvailableProduct) => {
    handleProductSelect(product);
  };

  // Modificar el manejo de teclado para permitir que las teclas de función sean manejadas por el controlador global
  const handleKeyDown = (e: React.KeyboardEvent) => {
    // Si es una tecla de función (F1-F12), no capturar el evento
    if (e.key.startsWith("F") && !isNaN(parseInt(e.key.slice(1)))) {
      return; // Permitir que el evento se propague
    }

    // Para otras teclas, seguir usando el manejador existente
    handleKeyPress(e);
  };

  return (
    <div className="relative w-full">
      <div className="flex items-center gap-2">
        <Input
          type="text"
          placeholder="Buscar productos (nombre o PLU)..."
          value={searchQuery}
          onChange={handleSearchInputChange}
          onKeyDown={handleKeyDown}
          className="w-full rounded-lg text-black" // Usar todo el ancho disponible en el sidebar
          ref={inputRef}
          autoFocus
        />
      </div>

      {showResults && searchResults.length > 0 && (
        <Card className="absolute w-full mt-1 bg-background border shadow-lg p-2 z-10 max-w-2xl rounded-xl">
          {searchResults.map((product, index) => (
            <div
              key={product.id}
              className={cn(
                "p-3 cursor-pointer rounded-lg transition-colors",
                index === selectedIndex
                  ? "bg-emerald-100 text-emerald-900 font-medium"
                  : "hover:bg-muted"
              )}
              onClick={() => selectProduct(product)}
            >
              <div className="text-base flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {product.plu && (
                    <span className="bg-blue-100 text-blue-800 text-xs font-medium px-2 py-1 rounded">
                      PLU: {product.plu}
                    </span>
                  )}
                  <span>{product.name}</span>
                </div>
                <span className="text-emerald-600">
                  $
                  {businessInfo
                    ? calcularPrecioVisualConIVA(
                        product.pricePerUnit,
                        product.ivaIncluido || false,
                        product.ivaPorcentaje ?? null,
                        businessInfo.ivaIncluidoEnPrecios || false
                      ).toLocaleString()
                    : product.pricePerUnit.toLocaleString()}
                  /{product.unit}
                </span>
              </div>
            </div>
          ))}
        </Card>
      )}
    </div>
  );
}
