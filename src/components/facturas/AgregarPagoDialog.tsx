import React, { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Factura, AddPagoData } from "@/types/factura";
import {
  DollarSign,
  Calendar,
  CreditCard,
  Wallet,
  ArrowRight,
} from "lucide-react";

interface AgregarPagoDialogProps {
  factura: Factura;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
  onSubmit: (data: AddPagoData) => Promise<boolean>;
}

export const AgregarPagoDialog: React.FC<AgregarPagoDialogProps> = ({
  factura,
  open,
  onOpenChange,
  onSuccess,
  onSubmit,
}) => {
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    monto: "",
    metodoPago: "efectivo" as "efectivo" | "tarjeta" | "transferencia",
    fecha: new Date().toISOString().split("T")[0],
    referencia: "",
  });

  const saldoPendiente = factura.saldoPendiente || factura.total;
  const maxMonto = saldoPendiente;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const monto = parseFloat(formData.monto);
    if (isNaN(monto) || monto <= 0) {
      alert("El monto debe ser un número válido mayor a 0");
      return;
    }

    if (monto > maxMonto) {
      alert(
        `El monto no puede ser mayor al saldo pendiente ($${maxMonto.toLocaleString(
          "es-AR",
          { minimumFractionDigits: 2 }
        )})`
      );
      return;
    }

    setLoading(true);
    try {
      const success = await onSubmit({
        facturaId: factura.id,
        monto,
        metodoPago: formData.metodoPago,
        fecha: formData.fecha,
        referencia: formData.referencia || undefined,
      });

      if (success) {
        onSuccess();
        onOpenChange(false);
        // Reset form
        setFormData({
          monto: "",
          metodoPago: "efectivo",
          fecha: new Date().toISOString().split("T")[0],
          referencia: "",
        });
      }
    } catch (error) {
      console.error("Error al agregar pago:", error);
    } finally {
      setLoading(false);
    }
  };

  const getMetodoPagoIcon = (metodo: string) => {
    switch (metodo) {
      case "efectivo":
        return <Wallet className="w-4 h-4" />;
      case "tarjeta":
        return <CreditCard className="w-4 h-4" />;
      case "transferencia":
        return <ArrowRight className="w-4 h-4" />;
      default:
        return <DollarSign className="w-4 h-4" />;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <DollarSign className="w-5 h-5" />
            Agregar Pago
          </DialogTitle>
          <DialogDescription>
            Agregar un pago a la factura. Saldo pendiente: $
            {maxMonto.toLocaleString("es-AR", { minimumFractionDigits: 2 })}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Monto */}
          <div className="space-y-2">
            <Label htmlFor="monto">Monto *</Label>
            <div className="relative">
              <DollarSign className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
              <Input
                id="monto"
                type="number"
                step="0.01"
                min="0"
                max={maxMonto}
                value={formData.monto}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, monto: e.target.value }))
                }
                placeholder="0.00"
                className="pl-10"
                required
              />
            </div>
          </div>

          {/* Método de Pago */}
          <div className="space-y-2">
            <Label htmlFor="metodoPago">Método de Pago *</Label>
            <Select
              value={formData.metodoPago}
              onValueChange={(
                value: "efectivo" | "tarjeta" | "transferencia"
              ) => setFormData((prev) => ({ ...prev, metodoPago: value }))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="efectivo">
                  <div className="flex items-center gap-2">
                    <Wallet className="w-4 h-4" />
                    EFECTIVO
                  </div>
                </SelectItem>
                <SelectItem value="tarjeta">
                  <div className="flex items-center gap-2">
                    <CreditCard className="w-4 h-4" />
                    TARJETA
                  </div>
                </SelectItem>
                <SelectItem value="transferencia">
                  <div className="flex items-center gap-2">
                    <ArrowRight className="w-4 h-4" />
                    TRANSFERENCIA
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Fecha del Pago */}
          <div className="space-y-2">
            <Label htmlFor="fecha">Fecha del Pago *</Label>
            <div className="relative">
              <Calendar className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
              <Input
                id="fecha"
                type="date"
                value={formData.fecha}
                onChange={(e) =>
                  setFormData((prev) => ({ ...prev, fecha: e.target.value }))
                }
                max={new Date().toISOString().split("T")[0]}
                className="pl-10"
                required
              />
            </div>
          </div>

          {/* Referencia */}
          <div className="space-y-2">
            <Label htmlFor="referencia">Referencia (opcional)</Label>
            <Input
              id="referencia"
              value={formData.referencia}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, referencia: e.target.value }))
              }
              placeholder="Número de comprobante, transferencia, etc."
            />
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancelar
            </Button>
            <Button
              type="submit"
              disabled={
                loading || !formData.monto || parseFloat(formData.monto) <= 0
              }
            >
              {loading ? "Agregando..." : "Agregar Pago"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
};
