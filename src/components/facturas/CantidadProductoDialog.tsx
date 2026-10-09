"use client";

import React, { useState, useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Package, Plus, Minus } from "lucide-react";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

interface Producto {
  id: number;
  nombre: string;
  tipoMedida: string;
  precio: number;
  stock?: number;
  ivaIncluido?: boolean;
  ivaPorcentaje?: number | null;
  plu?: string | null;
}

interface CantidadProductoDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (producto: Producto, cantidad: number) => void;
  producto: Producto | null;
}

const CantidadProductoDialog: React.FC<CantidadProductoDialogProps> = ({
  open,
  onClose,
  onConfirm,
  producto,
}) => {
  const [cantidad, setCantidad] = useState(1);
  const [error, setError] = useState<string | null>(null);

  // Obtener información del business para el cálculo de IVA
  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  // Resetear cantidad cuando se abre el diálogo
  useEffect(() => {
    if (open && producto) {
      setCantidad(1);
      setError(null);
    }
  }, [open, producto]);

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("es-AR", {
      style: "currency",
      currency: "ARS",
    }).format(amount);
  };

  const handleCantidadChange = (value: string) => {
    const numValue = parseFloat(value);
    if (isNaN(numValue) || numValue <= 0) {
      setCantidad(0);
      setError("La cantidad debe ser mayor a 0");
      return;
    }

    // Permitir agregar productos sin validar stock
    setError(null);
    setCantidad(numValue);
  };

  const handleIncrement = () => {
    const newCantidad = cantidad + 1;
    setCantidad(newCantidad);
    setError(null);
  };

  const handleDecrement = () => {
    if (cantidad > 1) {
      setCantidad(cantidad - 1);
      setError(null);
    }
  };

  const handleConfirm = () => {
    if (!producto) return;

    if (cantidad <= 0) {
      setError("La cantidad debe ser mayor a 0");
      return;
    }

    // Permitir confirmar productos sin validar stock
    onConfirm(producto, cantidad);
    onClose();
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleConfirm();
    }
  };

  if (!producto) return null;

  // Calcular precio visual con IVA para el subtotal
  const precioVisual = businessInfo
    ? calcularPrecioVisualConIVA(
        producto.precio,
        producto.ivaIncluido || false,
        producto.ivaPorcentaje ?? null,
        businessInfo.ivaIncluidoEnPrecios || false
      )
    : producto.precio;

  const subtotal = cantidad * precioVisual;

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-3">
            <Package className="h-5 w-5" />
            Agregar Producto
          </DialogTitle>
          <DialogDescription>
            Especifique la cantidad para agregar a la factura
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Información del producto */}
          <div className="p-4 bg-muted/30 rounded-lg">
            <div className="flex items-center gap-2 flex-wrap mb-2">
              {producto.plu && (
                <span className="bg-blue-100 text-blue-800 text-xs font-medium px-2 py-1 rounded">
                  PLU: {producto.plu}
                </span>
              )}
              <h3 className="font-semibold text-lg">{producto.nombre}</h3>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Precio unitario:</span>
                <span className="font-medium">
                  {businessInfo
                    ? formatCurrency(
                        calcularPrecioVisualConIVA(
                          producto.precio,
                          producto.ivaIncluido || false,
                          producto.ivaPorcentaje ?? null,
                          businessInfo.ivaIncluidoEnPrecios || false
                        )
                      )
                    : formatCurrency(producto.precio)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Unidad:</span>
                <span className="font-medium">{producto.tipoMedida}</span>
              </div>
              {producto.stock !== undefined && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">
                    Stock disponible:
                  </span>
                  <span
                    className={`font-medium ${
                      producto.stock > 0 ? "text-green-600" : "text-red-600"
                    }`}
                  >
                    {producto.stock}
                  </span>
                </div>
              )}
            </div>
          </div>

          {/* Selector de cantidad */}
          <div className="space-y-4">
            <Label htmlFor="cantidad">Cantidad</Label>
            <div className="flex items-center gap-3">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleDecrement}
                disabled={cantidad <= 1}
                className="h-10 w-10 p-0"
              >
                <Minus className="h-4 w-4" />
              </Button>

              <Input
                id="cantidad"
                type="number"
                value={cantidad}
                onChange={(e) => handleCantidadChange(e.target.value)}
                onKeyPress={handleKeyPress}
                min="0.01"
                step="0.01"
                className="text-center text-lg font-medium"
                autoFocus
              />

              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleIncrement}
                className="h-10 w-10 p-0"
              >
                <Plus className="h-4 w-4" />
              </Button>
            </div>

            {/* Error message */}
            {error && <p className="text-sm text-red-600">{error}</p>}

            {/* Subtotal */}
            <div className="p-3 bg-primary/5 rounded-lg">
              <div className="flex justify-between items-center">
                <span className="font-medium">Subtotal:</span>
                <span className="text-lg font-bold text-primary">
                  {formatCurrency(subtotal)}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Botones */}
        <div className="flex justify-end gap-3 pt-4 border-t">
          <Button variant="outline" onClick={onClose}>
            Cancelar
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={cantidad <= 0 || !!error}
            className="bg-emerald-gradient text-white hover:text-white"
          >
            <Plus className="h-4 w-4 mr-2" />
            Agregar a Factura
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default CantidadProductoDialog;
