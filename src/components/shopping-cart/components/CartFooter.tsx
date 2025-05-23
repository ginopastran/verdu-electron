import { Button } from "@/components/ui/button";
import { useCartContext } from "../contexts/CartContext";

interface CartFooterProps {
  onPaymentClick: () => void;
  onSplitPaymentClick: () => void;
}

export function CartFooter({
  onPaymentClick,
  onSplitPaymentClick,
}: CartFooterProps) {
  const { calculateTotal } = useCartContext();

  return (
    <div className="sticky bottom-0">
      <div className="flex items-center justify-between">
        <div className="space-x-2">
          <Button
            variant="outline"
            className="text-white px-14 py-8 text-xl rounded-xl bg-cancel-gradient hover:text-white"
            onClick={onSplitPaymentClick}
          >
            PAGO MIXTO
          </Button>
          <Button
            className="bg-emerald-gradient text-primary-foreground px-14 py-8 text-xl rounded-xl"
            onClick={onPaymentClick}
          >
            PAGAR
          </Button>
        </div>
        <div className="text-4xl font-semibold bg-white border px-8 py-6 shadow-sm rounded-xl">
          TOTAL: ${calculateTotal().toLocaleString()}
        </div>
      </div>
    </div>
  );
}
