import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { CreditCard, QrCode, Wallet } from "lucide-react";
import { usePaymentContext } from "../contexts/PaymentContext";
import { useCartContext } from "../contexts/CartContext";

interface PaymentDialogProps {
  open: boolean;
  onClose: () => void;
}

export function PaymentDialog({ open, onClose }: PaymentDialogProps) {
  const { getCurrentItems } = useCartContext();
  const { handleQRPayment } = usePaymentContext();

  const handlePayment = async (method: string) => {
    const items = getCurrentItems();
    if (items.length === 0) return;

    if (method === "qr") {
      try {
        const total = items.reduce((sum, item) => sum + item.subtotal, 0);
        await handleQRPayment(items, total);
      } catch (error) {
        console.error("Error al procesar pago QR:", error);
      }
    }

    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Seleccionar método de pago</DialogTitle>
          <DialogDescription>
            Presiona el número correspondiente al método de pago o haz clic en
            el botón
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-4">
          <Button
            onClick={() => handlePayment("qr")}
            className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
            variant="outline"
          >
            <QrCode className="h-12 w-12" />
            <span>Transferencia (1)</span>
          </Button>
          <Button
            onClick={() => handlePayment("tarjeta")}
            className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
            variant="outline"
          >
            <CreditCard className="h-12 w-12" />
            <span>Tarjeta (2)</span>
          </Button>
          <Button
            onClick={() => handlePayment("efectivo")}
            className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
            variant="outline"
          >
            <Wallet className="h-12 w-12" />
            <span>Efectivo (3)</span>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
