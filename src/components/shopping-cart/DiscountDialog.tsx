import { useState, useEffect } from "react";
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
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Percent, DollarSign } from "lucide-react";

interface DiscountData {
  tieneDescuento: boolean;
  tipoDescuento: "porcentual" | "cantidad";
  valorDescuento: number;
  subtotalSinDescuento: number;
  montoDescuento: number;
}

interface DiscountDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  subtotal: number;
  onConfirm: (discountData: DiscountData) => void;
  onCancel: () => void;
}

export const DiscountDialog = ({
  open,
  onOpenChange,
  subtotal,
  onConfirm,
  onCancel,
}: DiscountDialogProps) => {
  const [activeTab, setActiveTab] = useState<"porcentual" | "cantidad">(
    "porcentual"
  );
  const [percentageValue, setPercentageValue] = useState<string>("");
  const [amountValue, setAmountValue] = useState<string>("");
  const [calculatedDiscount, setCalculatedDiscount] = useState<number>(0);
  const [finalTotal, setFinalTotal] = useState<number>(subtotal);

  // Calcular descuento en tiempo real
  useEffect(() => {
    let discount = 0;

    if (activeTab === "porcentual" && percentageValue) {
      const percentage = parseFloat(percentageValue);
      if (!isNaN(percentage) && percentage > 0 && percentage <= 100) {
        discount = subtotal * (percentage / 100);
      }
    } else if (activeTab === "cantidad" && amountValue) {
      const amount = parseFloat(amountValue);
      if (!isNaN(amount) && amount > 0) {
        discount = Math.min(amount, subtotal); // No puede ser mayor al subtotal
      }
    }

    setCalculatedDiscount(discount);
    setFinalTotal(subtotal - discount);
  }, [activeTab, percentageValue, amountValue, subtotal]);

  // Resetear valores cuando se abre el diálogo
  useEffect(() => {
    if (open) {
      setActiveTab("porcentual");
      setPercentageValue("");
      setAmountValue("");
      setCalculatedDiscount(0);
      setFinalTotal(subtotal);
    }
  }, [open, subtotal]);

  const handleConfirm = () => {
    if (calculatedDiscount <= 0) {
      return;
    }

    const discountData: DiscountData = {
      tieneDescuento: true,
      tipoDescuento: activeTab,
      valorDescuento:
        activeTab === "porcentual"
          ? parseFloat(percentageValue)
          : parseFloat(amountValue),
      subtotalSinDescuento: subtotal,
      montoDescuento: calculatedDiscount,
    };

    onConfirm(discountData);
  };

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
  };

  const isValidDiscount = calculatedDiscount > 0;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Percent className="h-5 w-5" />
            Aplicar Descuento
          </DialogTitle>
          <DialogDescription>
            Selecciona el tipo de descuento y el valor a aplicar
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <Tabs
            value={activeTab}
            onValueChange={(value) =>
              setActiveTab(value as "porcentual" | "cantidad")
            }
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger
                value="porcentual"
                className="flex items-center gap-2"
              >
                <Percent className="h-4 w-4" />
                Porcentaje
              </TabsTrigger>
              <TabsTrigger value="cantidad" className="flex items-center gap-2">
                <DollarSign className="h-4 w-4" />
                Cantidad Fija
              </TabsTrigger>
            </TabsList>

            <TabsContent value="porcentual" className="space-y-3">
              <div className="space-y-2">
                <Label htmlFor="percentage">
                  Porcentaje de descuento (1-100%)
                </Label>
                <Input
                  id="percentage"
                  type="number"
                  min="1"
                  max="100"
                  step="0.01"
                  placeholder="Ej: 10"
                  value={percentageValue}
                  onChange={(e) => setPercentageValue(e.target.value)}
                  className="text-center text-lg"
                  autoFocus
                />
              </div>
            </TabsContent>

            <TabsContent value="cantidad" className="space-y-3">
              <div className="space-y-2">
                <Label htmlFor="amount">Cantidad fija de descuento</Label>
                <Input
                  id="amount"
                  type="number"
                  min="0.01"
                  max={subtotal}
                  step="0.01"
                  placeholder={`Máximo: $${subtotal.toFixed(2)}`}
                  value={amountValue}
                  onChange={(e) => setAmountValue(e.target.value)}
                  className="text-center text-lg"
                />
              </div>
            </TabsContent>
          </Tabs>

          {/* Preview del descuento */}
          <div className="bg-gray-50 p-6 rounded-lg space-y-3">
            <div className="flex justify-between text-lg">
              <span className="font-medium">Subtotal original:</span>
              <span className="font-bold text-xl">${subtotal.toFixed(2)}</span>
            </div>
            <div className="flex justify-between text-lg text-green-600">
              <span className="font-medium">Descuento aplicado:</span>
              <span className="font-bold text-xl">
                -${calculatedDiscount.toFixed(2)}
              </span>
            </div>
            <hr className="my-3 border-gray-300" />
            <div className="flex justify-between text-2xl font-bold">
              <span>Total final:</span>
              <span className="text-blue-600 text-3xl">
                ${finalTotal.toFixed(2)}
              </span>
            </div>
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button variant="outline" onClick={handleCancel} className="flex-1">
            Cancelar
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!isValidDiscount}
            className="flex-1 bg-green-600 hover:bg-green-700"
          >
            Aplicar Descuento
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
