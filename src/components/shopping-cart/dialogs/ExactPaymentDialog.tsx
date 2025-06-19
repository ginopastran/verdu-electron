import { useState, useEffect } from "react";
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
}

export function ExactPaymentDialog({
  open,
  onOpenChange,
  totalAmount,
  onConfirm,
  onCancel,
  isLoading = false,
}: ExactPaymentDialogProps) {
  const [paidAmount, setPaidAmount] = useState<string>("");
  const [change, setChange] = useState<number>(0);
  const [error, setError] = useState<string>("");

  // Calcular vuelto cuando cambia el monto pagado
  useEffect(() => {
    const paid = parseFloat(paidAmount);
    if (!isNaN(paid) && paid > 0) {
      const calculatedChange = paid - totalAmount;
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
  }, [paidAmount, totalAmount]);

  // Limpiar campos al abrir/cerrar
  useEffect(() => {
    if (open) {
      setPaidAmount("");
      setChange(0);
      setError("");
    }
  }, [open]);

  const handleConfirm = () => {
    const paid = parseFloat(paidAmount);
    if (isNaN(paid) || paid <= 0) {
      setError("Ingresa un monto válido");
      return;
    }

    if (paid < totalAmount) {
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
          {/* Total a pagar */}
          <div className="bg-emerald-50 p-4 rounded-lg border border-emerald-200">
            <div className="flex justify-between items-center">
              <span className="text-lg font-medium text-emerald-700">
                Total a pagar:
              </span>
              <span className="text-2xl font-bold text-emerald-600">
                ${totalAmount.toLocaleString()}
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
