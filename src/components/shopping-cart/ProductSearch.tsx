import { useRef, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { useProductSearch, AvailableProduct } from "@/hooks/useProductSearch";

interface ProductSearchProps {
  onProductSelect: (product: AvailableProduct) => void;
  inputRef: React.Ref<HTMLInputElement>;
}

export function ProductSearch({
  onProductSelect,
  inputRef,
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
  } = useProductSearch();

  // Efecto para detectar cuando el hook selecciona un producto
  useEffect(() => {
    if (selectedProduct) {
      onProductSelect(selectedProduct);
      clearSelection();
    }
  }, [selectedProduct, onProductSelect, clearSelection]);

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
          placeholder="Buscar productos..."
          value={searchQuery}
          onChange={handleSearchInputChange}
          onKeyDown={handleKeyDown}
          className="w-[380px]" // Aumentar el ancho del input
          ref={inputRef}
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
                <span>{product.name}</span>
                <span className="text-emerald-600">
                  ${product.pricePerUnit.toLocaleString()}/{product.unit}
                </span>
              </div>
            </div>
          ))}
        </Card>
      )}
    </div>
  );
}
