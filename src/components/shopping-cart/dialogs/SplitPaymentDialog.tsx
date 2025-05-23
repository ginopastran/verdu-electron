import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem } from "@/components/ui/select";
import { useCartContext } from "../contexts/CartContext";
import { usePaymentContext } from "../contexts/PaymentContext";

interface SplitPaymentDialogProps {
  open: boolean;
  onClose: () => void;
  cashAmount: string;
  onCashAmountChange: (value: string) => void;
  secondPaymentMethod: string;
  onSecondPaymentMethodChange: (value: string) => void;
}

export function SplitPaymentDialog({
  open,
  onClose,
  cashAmount,
  onCashAmountChange,
  secondPaymentMethod,
  onSecondPaymentMethodChange,
}: SplitPaymentDialogProps) {
  const { getCurrentItems, calculateTotal, clearCurrentCart } =
    useCartContext();
  const { isProcessingPayment, handleSplitPayment } = usePaymentContext();

  const handleConfirm = async () => {
    if (!cashAmount) return;

    const items = getCurrentItems();
    const total = calculateTotal();
    const cashAmountNum = Number(cashAmount);
    const qrAmount = total - cashAmountNum;

    try {
      await handleSplitPayment(items, cashAmountNum, qrAmount);
      clearCurrentCart();
      onClose();
    } catch (error) {
      console.error("Error en el pago dividido:", error);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Pago dividido</DialogTitle>
        </DialogHeader>
        <div className="grid gap-4">
          <div className="grid gap-2">
            <label>Monto en efectivo</label>
            <Input
              type="number"
              value={cashAmount}
              onChange={(e) => onCashAmountChange(e.target.value)}
              placeholder="Ingrese monto en efectivo"
              disabled={isProcessingPayment}
            />
          </div>
          <div className="grid gap-2">
            <label>Segundo método de pago</label>
            <Select
              value={secondPaymentMethod}
              onValueChange={onSecondPaymentMethodChange}
              disabled={isProcessingPayment}
            >
              <SelectContent>
                <SelectItem value="tarjeta">Tarjeta</SelectItem>
                <SelectItem value="qr">QR</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isProcessingPayment}
          >
            Cancelar
          </Button>
          <Button onClick={handleConfirm} disabled={isProcessingPayment}>
            Confirmar
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
