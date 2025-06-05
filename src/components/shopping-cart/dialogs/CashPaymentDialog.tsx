import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface BusinessInfo {
  sistemaPago?: string;
  descuentoEfectivo?: number;
  [key: string]: any;
}

interface CashPaymentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  originalAmount: number;
  roundedAmount: number;
  applyingDiscount: boolean;
  isProcessingPayment: boolean;
  businessInfo?: BusinessInfo;
  onConfirm: () => void;
  onApplyDiscount: () => void;
  onCancel?: () => void;
  isLoading?: boolean;
}

export function CashPaymentDialog({
  open,
  onOpenChange,
  originalAmount,
  roundedAmount,
  applyingDiscount,
  onConfirm,
  onApplyDiscount,
  isLoading = false,
}: CashPaymentDialogProps) {
  const hasDiscount = roundedAmount < originalAmount;

  // Debug logging
  console.log("💰 CashPaymentDialog - Valores recibidos:", {
    originalAmount,
    roundedAmount,
    hasDiscount,
    applyingDiscount,
    diferencia: originalAmount - roundedAmount,
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg max-w-[90vw] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Confirmar pago en efectivo</DialogTitle>
          <DialogDescription>
            {hasDiscount
              ? "Se ha aplicado un descuento por pago en efectivo"
              : "Confirma el monto a cobrar"}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {hasDiscount ? (
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Monto original:</span>
                <span className="line-through">
                  ${originalAmount.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between text-lg font-medium">
                <span>Nuevo monto con descuento:</span>
                <span className="text-emerald-600">
                  ${roundedAmount.toLocaleString()}
                </span>
              </div>
              <div className="text-sm text-muted-foreground mt-2">
                Descuento aplicado: $
                {(originalAmount - roundedAmount).toLocaleString()}
              </div>
            </div>
          ) : (
            <div className="space-y-2">
              {roundedAmount !== originalAmount ? (
                <>
                  <div className="flex justify-between">
                    <span>Monto original:</span>
                    <span className="line-through text-muted-foreground">
                      ${originalAmount.toLocaleString()}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-lg font-medium">
                      Monto a cobrar (redondeado):
                    </span>
                    <span className="text-3xl font-bold text-emerald-600">
                      ${roundedAmount.toLocaleString()}
                    </span>
                  </div>
                  <div className="text-sm text-muted-foreground mt-2">
                    Redondeo aplicado: $
                    {(originalAmount - roundedAmount).toLocaleString()}
                  </div>
                </>
              ) : (
                <div className="flex justify-between items-center">
                  <span className="text-lg font-medium">Monto a cobrar:</span>
                  <span className="text-3xl font-bold text-emerald-600">
                    ${roundedAmount.toLocaleString()}
                  </span>
                </div>
              )}
            </div>
          )}
        </div>

        <DialogFooter className="flex space-x-2 justify-end">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            tabIndex={3}
            className="text-base py-5 px-4"
            disabled={isLoading}
          >
            Cancelar
          </Button>

          {!applyingDiscount && (
            <Button
              variant="outline"
              className="bg-blue-50 border-blue-200 hover:bg-blue-100 hover:text-blue-700 text-base py-5 px-4"
              onClick={onApplyDiscount}
              tabIndex={2}
              disabled={isLoading}
            >
              Aplicar D (F5)
            </Button>
          )}

          <Button
            className="bg-emerald-gradient text-lg py-5 px-6"
            onClick={onConfirm}
            disabled={isLoading}
            autoFocus
            tabIndex={1}
          >
            {isLoading ? (
              <div className="flex items-center gap-2">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                <span>Procesando...</span>
              </div>
            ) : (
              "Confirmar pago"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
