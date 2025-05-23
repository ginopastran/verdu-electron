import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useCartContext } from "../contexts/CartContext";
import { useWeightContext } from "../contexts/WeightContext";
import { AvailableProduct } from "../types";

interface ProductQuantityInputProps {
  product: AvailableProduct;
  useManualWeight: boolean;
  onAdd: () => void;
}

export function ProductQuantityInput({
  product,
  useManualWeight: initialUseManualWeight,
  onAdd,
}: ProductQuantityInputProps) {
  const [quantity, setQuantity] = useState("");
  const { weight, useManualWeight, setUseManualWeight } = useWeightContext();
  const { addToCart } = useCartContext();

  const handleAdd = () => {
    if (!product) return;

    let finalQuantity: number;

    if (product.unit === "Kg") {
      if (useManualWeight) {
        if (!quantity) return;
        finalQuantity = parseFloat(quantity) / 1000; // Convertir gramos a kilos
      } else {
        finalQuantity = weight / 1000; // Convertir gramos a kilos
      }
    } else {
      if (!quantity) return;
      finalQuantity = parseFloat(quantity);
    }

    if (isNaN(finalQuantity) || finalQuantity <= 0) return;

    const cartId = `${product.id}-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 10)}-${Math.random().toString(36).substring(2, 10)}`;

    addToCart({
      id: product.id,
      cartId,
      name: product.name,
      quantity: finalQuantity,
      unit: product.unit,
      pricePerUnit: product.pricePerUnit,
      subtotal: product.pricePerUnit * finalQuantity,
      costo: product.costo,
    });

    setQuantity("");
    onAdd();
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">{product.name}</h3>
          <p className="text-sm text-muted-foreground">
            ${product.pricePerUnit}/{product.unit}
          </p>
        </div>

        {product.unit === "Kg" && (
          <div className="flex items-center space-x-2">
            <Switch
              id="weight-mode"
              checked={useManualWeight}
              onCheckedChange={setUseManualWeight}
            />
            <label
              htmlFor="weight-mode"
              className="text-sm font-medium leading-none"
            >
              Peso manual
            </label>
          </div>
        )}
      </div>

      <div className="flex gap-2">
        {product.unit === "Kg" && !useManualWeight ? (
          <div className="flex-1">
            <div className="text-2xl font-bold">{weight} g</div>
            <p className="text-sm text-muted-foreground">Peso de balanza</p>
          </div>
        ) : (
          <Input
            type="number"
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            placeholder={`Cantidad ${product.unit === "Kg" ? "en gramos" : ""}`}
            className="flex-1"
          />
        )}

        <Button onClick={handleAdd}>Agregar al carrito</Button>
      </div>
    </div>
  );
}
