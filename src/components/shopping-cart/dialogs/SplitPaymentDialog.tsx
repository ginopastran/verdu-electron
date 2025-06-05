import { RefObject } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Wallet, QrCode, CreditCard } from "lucide-react";

interface SplitPaymentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  paymentProcessor: any;
  cartState: any;
  businessInfo: any;
  searchInputRef: RefObject<HTMLInputElement>;
}

export const SplitPaymentDialog = ({
  open,
  onOpenChange,
  paymentProcessor,
  cartState,
  businessInfo,
  searchInputRef,
}: SplitPaymentDialogProps) => {
  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        if (!open) {
          onOpenChange(false);
          paymentProcessor.resetPaymentState();
          setTimeout(() => searchInputRef.current?.focus(), 100);
        }
      }}
    >
      <DialogContent
        className="sm:max-w-[500px]"
        onKeyDown={(e) => {
          // Si se presiona Enter, simular clic en el botón "Completar pago"
          if (e.key === "Enter") {
            e.preventDefault();

            // Verificar si el botón debería estar habilitado
            const cashAmountValue = parseFloat(paymentProcessor.cashAmount);
            const isButtonEnabled = !(
              paymentProcessor.isProcessingPayment ||
              !paymentProcessor.cashAmount ||
              isNaN(cashAmountValue) ||
              cashAmountValue <= 0 ||
              cashAmountValue >= cartState.calculateTotal()
            );

            // Solo disparar la acción si el botón estaría habilitado
            if (isButtonEnabled) {
              console.log(
                "⌨️ TECLADO: Enter detectado en diálogo de pago mixto"
              );
              paymentProcessor.processSplitPayment(
                cartState.getCurrentItems(),
                cartState.calculateTotal(),
                businessInfo
              );
            }
          }
        }}
      >
        <DialogHeader>
          <DialogTitle className="text-xl">Pago mixto</DialogTitle>
          <DialogDescription>
            Ingresa el monto a pagar con efectivo y selecciona el método para el
            resto
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          <div className="text-center">
            <div className="text-lg text-muted-foreground">Total a pagar</div>
            <div className="text-3xl font-bold text-emerald-600 mt-1">
              ${cartState.calculateTotal().toLocaleString()}
            </div>
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <div className="flex justify-between">
                <label className="text-sm font-medium flex items-center">
                  <Wallet className="mr-2 h-4 w-4" />
                  Efectivo
                </label>
                <Input
                  type="number"
                  placeholder="Monto en efectivo"
                  value={paymentProcessor.cashAmount}
                  onChange={(e) =>
                    paymentProcessor.setCashAmount(e.target.value)
                  }
                  className="w-40 text-right"
                  step="0.01"
                  min="0"
                  max={cartState.calculateTotal().toString()}
                  disabled={paymentProcessor.isProcessingPayment}
                  autoFocus
                />
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Segundo método</label>
              <div className="grid grid-cols-2 gap-3">
                <Button
                  variant={
                    paymentProcessor.secondPaymentMethod === "tarjeta"
                      ? "default"
                      : "outline"
                  }
                  className={
                    paymentProcessor.secondPaymentMethod === "tarjeta"
                      ? "bg-emerald-600 hover:bg-emerald-700"
                      : ""
                  }
                  onClick={() =>
                    paymentProcessor.setSecondPaymentMethod("tarjeta")
                  }
                  disabled={paymentProcessor.isProcessingPayment}
                >
                  <CreditCard className="mr-2 h-4 w-4" />
                  Tarjeta
                </Button>
                <Button
                  variant={
                    paymentProcessor.secondPaymentMethod === "qr"
                      ? "default"
                      : "outline"
                  }
                  className={
                    paymentProcessor.secondPaymentMethod === "qr"
                      ? "bg-emerald-600 hover:bg-emerald-700"
                      : ""
                  }
                  onClick={() => paymentProcessor.setSecondPaymentMethod("qr")}
                  disabled={paymentProcessor.isProcessingPayment}
                >
                  <QrCode className="mr-2 h-4 w-4" />
                  Transferencia
                </Button>
              </div>
            </div>

            <div className="flex justify-between pt-2">
              <div className="flex items-center">
                <div className="text-sm font-medium">
                  {paymentProcessor.secondPaymentMethod === "tarjeta" ? (
                    <div className="flex items-center">
                      <CreditCard className="mr-2 h-4 w-4" />
                      Tarjeta
                    </div>
                  ) : (
                    <div className="flex items-center">
                      <QrCode className="mr-2 h-4 w-4" />
                      Transferencia
                    </div>
                  )}
                </div>
              </div>
              <div className="text-xl font-semibold text-emerald-600">
                $
                {!paymentProcessor.cashAmount ||
                isNaN(parseFloat(paymentProcessor.cashAmount))
                  ? cartState.calculateTotal().toLocaleString()
                  : (
                      cartState.calculateTotal() -
                      Math.min(
                        parseFloat(paymentProcessor.cashAmount),
                        cartState.calculateTotal()
                      )
                    ).toLocaleString()}
              </div>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => {
              onOpenChange(false);
              paymentProcessor.resetPaymentState();
            }}
            disabled={paymentProcessor.isProcessingPayment}
          >
            Cancelar
          </Button>
          <Button
            onClick={() =>
              paymentProcessor.processSplitPayment(
                cartState.getCurrentItems(),
                cartState.calculateTotal(),
                businessInfo
              )
            }
            disabled={
              paymentProcessor.isProcessingPayment ||
              !paymentProcessor.cashAmount ||
              isNaN(parseFloat(paymentProcessor.cashAmount)) ||
              parseFloat(paymentProcessor.cashAmount) <= 0 ||
              parseFloat(paymentProcessor.cashAmount) >=
                cartState.calculateTotal()
            }
            className="bg-emerald-gradient"
          >
            {paymentProcessor.isProcessingPayment ? (
              <div className="flex items-center gap-2">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                <span>Procesando...</span>
              </div>
            ) : (
              "Completar pago"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
