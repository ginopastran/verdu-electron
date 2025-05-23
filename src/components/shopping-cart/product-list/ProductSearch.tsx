import { Input } from "@/components/ui/input";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { AvailableProduct } from "../types";

interface ProductSearchProps {
  searchQuery: string;
  onSearchChange: (value: string) => void;
  searchResults: AvailableProduct[];
  selectedIndex: number;
  onProductSelect: (product: AvailableProduct) => void;
  showResults: boolean;
}

export function ProductSearch({
  searchQuery,
  onSearchChange,
  searchResults,
  selectedIndex,
  onProductSelect,
  showResults,
}: ProductSearchProps) {
  return (
    <div className="relative w-full">
      <Input
        type="text"
        placeholder="Buscar productos..."
        value={searchQuery}
        onChange={(e) => onSearchChange(e.target.value)}
        className="w-[300px]"
      />

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
              onClick={() => onProductSelect(product)}
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
