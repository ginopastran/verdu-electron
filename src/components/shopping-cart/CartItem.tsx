import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Product } from "@/hooks/useCartState";

interface CartItemProps {
  item: Product;
  onRemove: (id: number) => void;
}

export function CartItem({ item, onRemove }: CartItemProps) {
  return (
    <Card className="bg-background border p-4 flex items-center justify-between shadow-sm rounded-xl">
      <div className="flex justify-between w-full items-center">
        <div className="flex justify-between items-end gap-10">
          <span className="text-2xl font-medium">{item.name}</span>
          <div className="text-muted-foreground">
            Cantidad: {item.quantity} {item.unit}
          </div>
          <div className="text-muted-foreground">
            {item.unit === "Kg" ? "$/Kg" : "$/U"}: $
            {item.pricePerUnit.toLocaleString()}
          </div>
          <div className="text-muted-foreground">
            SUBTOTAL: ${item.subtotal.toLocaleString()}
          </div>
        </div>

        <Button
          variant="destructive"
          size="icon"
          className="h-8 w-8 rounded-lg bg-cancel-gradient"
          onClick={() => onRemove(item.id)}
        >
          <Trash2 className="h-6 w-6" />
        </Button>
      </div>
    </Card>
  );
}
