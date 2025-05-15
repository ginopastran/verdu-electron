import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { AvailableProduct } from "@/hooks/useProductSearch";
import { Product } from "@/hooks/useCartState";
import { useScaleWeight } from "@/hooks/useScaleWeight";
import { useAuth } from "@/contexts/AuthContext";

interface AddProductDialogProps {
  isOpen: boolean;
  product: AvailableProduct | null;
  onClose: () => void;
  onAddToCart: (product: Product) => void;
}

export function AddProductDialog({
  isOpen,
  product,
  onClose,
  onAddToCart,
}: AddProductDialogProps) {
  const [quantity, setQuantity] = useState<string>("");
  const [useManualWeight, setUseManualWeight] = useState(false);
  const [isAddingToCart, setIsAddingToCart] = useState(false);
  const weight = useScaleWeight();
  const { user } = useAuth();

  // Initialize and update useManualWeight based on user permissions
  useEffect(() => {
    // Set weight mode based on user permissions
    if (user?.permisos?.pesoManualEnabled === true) {
      setUseManualWeight(true);
    } else {
      setUseManualWeight(false);
    }
  }, [user]);

  // Cleanup when dialog closes
  useEffect(() => {
    if (!isOpen) {
      setQuantity("");
      setIsAddingToCart(false);
    }
  }, [isOpen]);

  const handleAddToCart = () => {
    if (isAddingToCart || !product) return;

    setIsAddingToCart(true);

    try {
      let finalQuantity: number;

      // Validación específica para productos de tipo Kg
      if (product.unit === "Kg") {
        if (useManualWeight) {
          if (!quantity) {
            throw new Error(
              "Se requiere especificar una cantidad para este producto"
            );
          }
          finalQuantity = parseFloat(quantity) / 1000; // Convertir gramos a kilos
        } else {
          finalQuantity = weight / 1000; // Convertir gramos a kilos
        }
      } else {
        if (!quantity) {
          throw new Error(
            "Se requiere especificar una cantidad para este producto"
          );
        }
        finalQuantity = parseFloat(quantity);
      }

      // Validación adicional de la cantidad
      if (isNaN(finalQuantity) || finalQuantity <= 0) {
        throw new Error("La cantidad debe ser un número mayor que cero");
      }

      // Crear un ID único para el item del carrito
      const uniqueId = `${product.id}-${Date.now()}-${Math.random()
        .toString(36)
        .substring(2, 10)}-${Math.random().toString(36).substring(2, 10)}`;

      // Crear el objeto a añadir
      const newItem: Product = {
        id: product.id,
        cartId: uniqueId,
        name: product.name,
        quantity: finalQuantity,
        unit: product.unit,
        pricePerUnit: product.pricePerUnit,
        subtotal: product.pricePerUnit * finalQuantity,
        costo: product.costo,
      };

      // Añadir al carrito
      onAddToCart(newItem);

      // Cerrar el diálogo y limpiar
      onClose();
    } catch (error: any) {
      console.error("Error:", error.message);
      alert(error.message || "Error al añadir producto");
    } finally {
      // Aseguramos que el estado isAddingToCart se resetea
      setTimeout(() => {
        setIsAddingToCart(false);
      }, 500);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !isAddingToCart) {
      e.preventDefault();

      if (product?.unit === "Kg" && !useManualWeight) {
        handleAddToCart();
      } else if (quantity) {
        handleAddToCart();
      }
    }
  };

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent onKeyDown={handleKeyDown}>
        <DialogHeader>
          <DialogTitle className="text-emerald-gradient font-bold text-2xl">
            Agregar producto
          </DialogTitle>
          <DialogDescription>
            {product?.name} - ${product?.pricePerUnit}/ {product?.unit}
          </DialogDescription>
        </DialogHeader>

        {product?.unit === "Kg" && (
          <div className="flex items-center space-x-4 py-2">
            {user?.permisos?.pesoManualEnabled === true && (
              <>
                <Switch
                  id="weight-mode"
                  checked={useManualWeight}
                  onCheckedChange={setUseManualWeight}
                  className="data-[state=checked]:bg-emerald-700"
                />
                <label
                  htmlFor="weight-mode"
                  className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                  Peso manual
                </label>
              </>
            )}
            {user?.permisos?.pesoManualEnabled !== true && (
              <p className="text-sm text-gray-500">Modo de peso automático</p>
            )}
          </div>
        )}

        {product?.unit !== "Kg" || useManualWeight ? (
          <Input
            type="number"
            placeholder={`Cantidad ${
              product?.unit === "Kg" ? "en gramos" : ""
            }`}
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            step={product?.unit === "Kg" ? "1" : "1"}
            min="0"
            autoFocus
          />
        ) : (
          <div className="text-start py-2">
            <p className="text-3xl font-bold">{weight} g</p>
            <p className="text-sm text-gray-500">Peso de la balanza</p>
          </div>
        )}

        <DialogFooter>
          <Button
            variant="outline"
            className="bg-cancel-gradient text-white hover:text-white text-base"
            onClick={onClose}
            type="button"
            tabIndex={2}
          >
            Cancelar
          </Button>
          <Button
            onClick={handleAddToCart}
            className="bg-emerald-gradient text-white hover:text-white text-base"
            type="button"
            disabled={isAddingToCart}
            autoFocus={!(product?.unit !== "Kg" || useManualWeight)}
            tabIndex={1}
          >
            {isAddingToCart ? (
              <div className="flex items-center gap-2">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                <span>Agregando...</span>
              </div>
            ) : (
              "Agregar al carrito"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
