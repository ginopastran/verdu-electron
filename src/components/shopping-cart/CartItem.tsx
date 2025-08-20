import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Product } from "@/hooks/useCartState";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

interface CartItemProps {
  item: Product;
  onRemove: (cartId: string) => void;
  isCancellationEnabled?: boolean;
}

export function CartItem({
  item,
  onRemove,
  isCancellationEnabled = false,
}: CartItemProps) {
  // Obtener información del business para el cálculo de IVA
  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  // Calcular precios visuales con IVA
  const precioVisualPorUnidad = businessInfo
    ? calcularPrecioVisualConIVA(
        item.pricePerUnit,
        item.ivaIncluido || false,
        item.ivaPorcentaje ?? null,
        businessInfo.ivaIncluidoEnPrecios || false
      )
    : item.pricePerUnit;

  const subtotalVisual = precioVisualPorUnidad * item.quantity;
  return (
    <Card className="bg-background border p-4 flex items-center justify-between shadow-sm rounded-xl border-[#A7A7A7]">
      <div className="flex justify-between w-full items-center">
        <div className="flex justify-between items-end gap-10">
          <span className="text-2xl font-medium">{item.name}</span>
          <div className="text-muted-foreground">
            Cantidad: {item.quantity} {item.unit}
          </div>
          <div className="text-muted-foreground">
            {item.unit === "Kg" ? "$/Kg" : "$/U"}: $
            {precioVisualPorUnidad.toLocaleString()}
          </div>
          <div className="text-muted-foreground">
            SUBTOTAL: ${subtotalVisual.toLocaleString()}
          </div>
        </div>

        {/* Solo mostrar botón de eliminar si las cancelaciones NO están habilitadas */}
        {!isCancellationEnabled && (
          <Button
            variant="destructive"
            size="icon"
            className="h-8 w-8 rounded-lg bg-cancel-gradient"
            onClick={() => onRemove(item.cartId)}
            title="Eliminar producto"
          >
            <Trash2 className="h-6 w-6" />
          </Button>
        )}
      </div>
    </Card>
  );
}
