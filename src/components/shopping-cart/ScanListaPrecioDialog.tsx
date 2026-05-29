import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { AvailableProduct } from "@/hooks/useProductSearch";
import { Product } from "@/hooks/useCartState";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

export type FilaListaScan = {
  listaPrecioId: number;
  precio: number;
  nombreLista: string;
};

interface ScanListaPrecioDialogProps {
  isOpen: boolean;
  product: AvailableProduct | null;
  rows: FilaListaScan[];
  quantity: number;
  onClose: () => void;
  onAddToCart: (product: Product) => void;
}

export function ScanListaPrecioDialog({
  isOpen,
  product,
  rows,
  quantity,
  onClose,
  onAddToCart,
}: ScanListaPrecioDialogProps) {
  const [selectedListaId, setSelectedListaId] = useState<number | null>(null);

  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  useEffect(() => {
    if (!isOpen) {
      setSelectedListaId(null);
      return;
    }
    const defId = businessInfo?.listaPrecioPorDefectoId;
    const match =
      defId != null ? rows.find((r) => r.listaPrecioId === defId) : null;
    setSelectedListaId(match?.listaPrecioId ?? rows[0]?.listaPrecioId ?? null);
  }, [isOpen, rows, businessInfo?.listaPrecioPorDefectoId]);

  const precioVisual = (unit: number) =>
    businessInfo && product
      ? calcularPrecioVisualConIVA(
          unit,
          product.ivaIncluido || false,
          product.ivaPorcentaje ?? null,
          businessInfo.ivaIncluidoEnPrecios || false
        )
      : unit;

  const selectedRow =
    rows.find((r) => r.listaPrecioId === selectedListaId) ?? null;
  const precioUnitario = selectedRow?.precio ?? product?.pricePerUnit ?? 0;

  const handleConfirm = () => {
    if (!product || !selectedRow) return;

    const uniqueId = `${product.id}-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 10)}`;

    const newItem: Product = {
      id: product.id,
      cartId: uniqueId,
      name: product.name,
      quantity,
      unit: product.unit || "Unidad",
      pricePerUnit: selectedRow.precio,
      subtotal: Number((selectedRow.precio * quantity).toFixed(2)),
      costo: product.costo,
      ivaIncluido: product.ivaIncluido,
      ivaPorcentaje: product.ivaPorcentaje,
      listaPrecioId: selectedRow.listaPrecioId,
      listaPrecioNombre: selectedRow.nombreLista,
    };

    onAddToCart(newItem);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleConfirm();
    }
  };

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent onKeyDown={handleKeyDown}>
        <DialogHeader>
          <DialogTitle className="text-emerald-gradient font-bold text-2xl">
            Elegí la lista de precios
          </DialogTitle>
          <DialogDescription>
            {product?.name} — ${precioVisual(precioUnitario).toLocaleString()} /{" "}
            {product?.unit}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-1">
          <label
            htmlFor="scan-lista-precio"
            className="text-sm font-medium text-gray-700"
          >
            Lista de precios
          </label>
          <select
            id="scan-lista-precio"
            className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            value={selectedListaId != null ? String(selectedListaId) : ""}
            onChange={(e) => setSelectedListaId(Number(e.target.value))}
            autoFocus
          >
            {rows.map((r) => (
              <option key={r.listaPrecioId} value={String(r.listaPrecioId)}>
                {r.nombreLista} — ${precioVisual(r.precio).toLocaleString()}
              </option>
            ))}
          </select>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            className="bg-cancel-gradient text-white hover:text-white text-base"
            onClick={onClose}
            type="button"
            tabIndex={2}
          >
            Cancelar
          </Button>
          <Button
            onClick={handleConfirm}
            className="bg-emerald-gradient text-white hover:text-white text-base"
            type="button"
            tabIndex={1}
          >
            Agregar al carrito
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
