import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { AlertTriangle, FileText, Receipt } from "lucide-react";

interface CancellationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => Promise<boolean>;
  onCancel: () => void;
  reason: string;
  onReasonChange: (reason: string) => void;
  isCancelling: boolean;
  pendingCancellation: {
    tipo: "orden" | "factura";
    referenciaId: string;
    monto: number;
    productos: Array<{
      productoId: number;
      nombreProducto: string;
      cantidad: number;
      precioUnitario: number;
      subtotal: number;
    }>;
  } | null;
}

export function CancellationDialog({
  open,
  onOpenChange,
  onConfirm,
  onCancel,
  reason,
  onReasonChange,
  isCancelling,
  pendingCancellation,
}: CancellationDialogProps) {
  const [error, setError] = useState<string>("");
  const reasonTextareaRef = useRef<HTMLTextAreaElement>(null);

  // Limpiar error cuando se abre el diálogo
  useEffect(() => {
    if (open) {
      setError("");
    }
  }, [open]);

  // Enfocar el textarea cuando se abre el diálogo
  useEffect(() => {
    if (open) {
      setTimeout(() => {
        reasonTextareaRef.current?.focus();
      }, 100);
    }
  }, [open]);

  const handleConfirm = async () => {
    if (!reason.trim()) {
      setError("Debes ingresar un motivo para la cancelación");
      return;
    }

    if (reason.trim().length < 3) {
      setError("El motivo debe tener al menos 3 caracteres");
      return;
    }

    setError("");
    const success = await onConfirm();

    if (success) {
      onOpenChange(false);
    }
  };

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleConfirm();
    }
    if (e.key === "Escape") {
      handleCancel();
    }
  };

  const getDialogTitle = () => {
    if (!pendingCancellation) return "Cancelar";

    return pendingCancellation.tipo === "orden"
      ? "Cancelar Orden"
      : "Cancelar Factura";
  };

  const getDialogDescription = () => {
    if (!pendingCancellation) return "";

    return pendingCancellation.tipo === "orden"
      ? `¿Estás seguro de que deseas cancelar la orden #${pendingCancellation.referenciaId}? Esta acción quedará registrada en el sistema.`
      : `¿Estás seguro de que deseas cancelar la factura #${pendingCancellation.referenciaId}? Esta acción quedará registrada en el sistema.`;
  };

  const getIcon = () => {
    if (!pendingCancellation) return <AlertTriangle className="h-5 w-5" />;

    return pendingCancellation.tipo === "orden" ? (
      <FileText className="h-5 w-5" />
    ) : (
      <Receipt className="h-5 w-5" />
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md" onKeyDown={handleKeyDown}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-red-600">
            {getIcon()}
            {getDialogTitle()}
          </DialogTitle>
          <DialogDescription className="text-left">
            {getDialogDescription()}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <label
              htmlFor="cancellation-reason"
              className="text-sm font-medium"
            >
              Motivo de la cancelación *
            </label>
            <Textarea
              id="cancellation-reason"
              ref={reasonTextareaRef}
              placeholder="Ej: Cliente cambió de opinión, error en el pedido, producto no disponible..."
              value={reason}
              onChange={(e) => onReasonChange(e.target.value)}
              disabled={isCancelling}
              className="min-h-[100px] resize-none"
              maxLength={500}
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>Mínimo 3 caracteres</span>
              <span>{reason.length}/500</span>
            </div>
            {error && <p className="text-sm text-red-600">{error}</p>}
          </div>

          {/* Mostrar productos que se van a cancelar */}
          {pendingCancellation && pendingCancellation.productos.length > 0 && (
            <div>
              <label className="text-sm font-medium text-gray-700">
                Productos a cancelar ({pendingCancellation.productos.length})
              </label>
              <div className="mt-2 max-h-32 overflow-y-auto border rounded-md p-2 bg-gray-50">
                {pendingCancellation.productos.map((producto, index) => (
                  <div
                    key={index}
                    className="flex justify-between items-center py-1 text-sm"
                  >
                    <span className="font-medium">
                      {producto.nombreProducto}
                    </span>
                    <span className="text-gray-600">
                      {producto.cantidad} x $
                      {producto.precioUnitario.toFixed(2)} = $
                      {producto.subtotal.toFixed(2)}
                    </span>
                  </div>
                ))}
                <div className="border-t pt-2 mt-2">
                  <div className="flex justify-between items-center font-semibold">
                    <span>Total a cancelar:</span>
                    <span className="text-red-600">
                      ${pendingCancellation.monto.toFixed(2)}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="flex space-x-2 justify-end">
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={isCancelling}
          >
            No, mantener
          </Button>
          <Button
            variant="destructive"
            onClick={handleConfirm}
            disabled={
              isCancelling || !reason.trim() || reason.trim().length < 3
            }
            className="bg-red-600 hover:bg-red-700"
          >
            {isCancelling ? (
              <div className="flex items-center gap-2">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                <span>Registrando...</span>
              </div>
            ) : (
              "Sí, cancelar y registrar"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
