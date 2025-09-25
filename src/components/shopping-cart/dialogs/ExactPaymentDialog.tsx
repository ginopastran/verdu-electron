import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Calculator, DollarSign } from "lucide-react";

interface ExactPaymentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  totalAmount: number;
  onConfirm: (paidAmount: number, change: number) => void;
  onCancel?: () => void;
  isLoading?: boolean;
  // Props para mostrar información de descuento
  discountData?: {
    type: "percentage" | "fixed";
    value: number;
    amount: number;
  };
  subtotal?: number;
}

export function ExactPaymentDialog({
  open,
  onOpenChange,
  totalAmount,
  onConfirm,
  onCancel,
  isLoading = false,
  discountData,
  subtotal,
}: ExactPaymentDialogProps) {
  const [paidAmount, setPaidAmount] = useState<string>("");
  const [change, setChange] = useState<number>(0);
  const [error, setError] = useState<string>("");

  // Ref para el input y forzar foco al abrir
  const paidInputRef = useRef<HTMLInputElement>(null);

  // Calcular vuelto cuando cambia el monto pagado
  useEffect(() => {
    const paid = parseFloat(paidAmount);
    if (!isNaN(paid) && paid > 0) {
      // Usar el total con descuento aplicado para calcular el cambio
      const finalTotal =
        discountData && subtotal ? subtotal - discountData.amount : totalAmount;
      const calculatedChange = paid - finalTotal;
      setChange(calculatedChange);

      if (calculatedChange < 0) {
        setError("El monto ingresado es insuficiente");
      } else {
        setError("");
      }
    } else {
      setChange(0);
      setError("");
    }
  }, [paidAmount, totalAmount, discountData, subtotal]);

  // Limpiar campos al abrir/cerrar
  useEffect(() => {
    if (open) {
      setPaidAmount("");
      setChange(0);
      setError("");
    }
  }, [open]);

  // Forzar foco cuando se abra el diálogo (después de un tick para que el DOM esté listo)
  useEffect(() => {
    if (open) {
      setTimeout(() => {
        paidInputRef.current?.focus();
      }, 10);
      // Reintentar un poco más tarde por si otro componente roba el foco
      setTimeout(() => {
        paidInputRef.current?.focus();
      }, 400);
    }
  }, [open]);

  const handleConfirm = () => {
    const paid = parseFloat(paidAmount);
    if (isNaN(paid) || paid <= 0) {
      setError("Ingresa un monto válido");
      return;
    }

    // Usar el total con descuento aplicado para la validación
    const finalTotal =
      discountData && subtotal ? subtotal - discountData.amount : totalAmount;

    if (paid < finalTotal) {
      setError("El monto pagado debe ser mayor o igual al total");
      return;
    }

    onConfirm(paid, change);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !error && paidAmount && !isLoading) {
      e.preventDefault();
      handleConfirm();
    }
    if (e.key === "Escape") {
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-lg max-w-[90vw]"
        onKeyDown={handleKeyDown}
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Calculator className="h-5 w-5" />
            Pago exacto - Calcular vuelto
          </DialogTitle>
          <DialogDescription>
            Ingresa con cuánto dinero paga el cliente para calcular el vuelto
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Información de descuento si está disponible */}
          {discountData && subtotal && (
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Subtotal:</span>
                <span className="text-sm">${subtotal.toLocaleString()}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">
                  Descuento (
                  {discountData.type === "percentage"
                    ? `${discountData.value}%`
                    : `$${discountData.value}`}
                  ):
                </span>
                <span className="text-sm text-green-600">
                  -${discountData.amount.toLocaleString()}
                </span>
              </div>
              <hr className="border-gray-200" />
            </div>
          )}

          {/* Total a pagar */}
          <div className="bg-emerald-50 p-4 rounded-lg border border-emerald-200">
            <div className="flex justify-between items-center">
              <span className="text-lg font-medium text-emerald-700">
                Total a pagar:
              </span>
              <span className="text-2xl font-bold text-emerald-600">
                $
                {discountData && subtotal
                  ? (subtotal - discountData.amount).toLocaleString()
                  : totalAmount.toLocaleString()}
              </span>
            </div>
          </div>

          {/* Input para monto pagado */}
          <div className="space-y-2">
            <label className="flex items-center gap-2 text-sm font-medium">
              <DollarSign className="h-4 w-4" />
              Con cuánto paga el cliente:
            </label>
            <Input
              type="number"
              placeholder="Ej: 5000"
              value={paidAmount}
              onChange={(e) => setPaidAmount(e.target.value)}
              className={`text-lg text-center ${error ? "border-red-500" : ""}`}
              step="0.01"
              min={totalAmount.toString()}
              disabled={isLoading}
              ref={paidInputRef}
              autoFocus
            />
            {error && <p className="text-sm text-red-600 mt-1">{error}</p>}
          </div>

          {/* Vuelto calculado */}
          {paidAmount && !error && change >= 0 && (
            <div className="bg-blue-50 p-4 rounded-lg border border-blue-200">
              <div className="flex justify-between items-center">
                <span className="text-lg font-medium text-blue-700">
                  Vuelto a entregar:
                </span>
                <span className="text-2xl font-bold text-blue-600">
                  ${change.toFixed(2).toLocaleString()}
                </span>
              </div>
              {change === 0 && (
                <p className="text-sm text-blue-600 mt-2">
                  ✅ Pago exacto - No hay vuelto
                </p>
              )}
            </div>
          )}
        </div>

        <DialogFooter className="flex space-x-2 justify-end">
          <Button
            variant="outline"
            onClick={() => {
              if (onCancel) {
                onCancel();
              } else {
                onOpenChange(false);
              }
            }}
            tabIndex={3}
            className="text-base py-5 px-4"
            disabled={isLoading}
          >
            Cancelar
          </Button>

          <Button
            className="bg-emerald-gradient text-lg py-5 px-6"
            onClick={handleConfirm}
            disabled={isLoading || !!error || !paidAmount}
            autoFocus={false}
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
