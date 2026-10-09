import { useEffect, useMemo, useState } from "react";
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
import { AvailableProduct } from "@/hooks/useProductSearch";
import { Product } from "@/hooks/useCartState";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

export type FilaListaScan = {
  listaPrecioId: number;
  precio: number;
  nombreLista: string;
  // tipoMedida específico de esta lista (Unidad/Kg). null = hereda del producto
  tipoMedida?: string | null;
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
  // "base" o el id de la lista como string
  const [selectedValue, setSelectedValue] = useState<string>("base");
  const [qtyInput, setQtyInput] = useState<string>("");

  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  const etiquetaBase = (
    businessInfo?.etiquetaPrecioBase || "Precio catálogo"
  ).trim();

  // Medida base del producto (fallback cuando la lista no define una)
  const medidaProducto = product?.unit || "Unidad";

  // Opción seleccionada (base o lista)
  const selectedRow =
    selectedValue !== "base"
      ? rows.find((r) => String(r.listaPrecioId) === selectedValue) ?? null
      : null;

  const precioUnitario =
    selectedRow?.precio ?? product?.pricePerUnit ?? 0;

  // tipoMedida efectivo: el de la lista, o el del producto
  const tipoMedida = selectedRow?.tipoMedida || medidaProducto;
  const esPeso = tipoMedida === "Kg";

  // Al abrir / cambiar de opción, prefijar la cantidad según la medida.
  // Para Kg el input se expresa en gramos; para Unidad en unidades.
  useEffect(() => {
    if (!isOpen) {
      setSelectedValue("base");
      setQtyInput("");
      return;
    }
    const defId = businessInfo?.listaPrecioPorDefectoId;
    const match =
      defId != null ? rows.find((r) => r.listaPrecioId === defId) : null;
    setSelectedValue(
      match ? String(match.listaPrecioId) : rows[0] ? String(rows[0].listaPrecioId) : "base"
    );
  }, [isOpen, rows, businessInfo?.listaPrecioPorDefectoId]);

  // Prefijar el valor del input cuando cambia la medida efectiva o se abre
  useEffect(() => {
    if (!isOpen) return;
    const base = quantity > 0 ? quantity : 1;
    setQtyInput(esPeso ? String(Math.round(base * 1000)) : String(base));
    // Solo depende de la medida efectiva y la apertura
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen, esPeso]);

  const precioVisual = (unit: number) =>
    businessInfo && product
      ? calcularPrecioVisualConIVA(
        unit,
        product.ivaIncluido || false,
        product.ivaPorcentaje ?? null,
        businessInfo.ivaIncluidoEnPrecios || false
      )
      : unit;

  const finalQuantity = useMemo(() => {
    const n = parseFloat(qtyInput);
    if (isNaN(n) || n <= 0) return 0;
    return esPeso ? n / 1000 : n;
  }, [qtyInput, esPeso]);

  const handleConfirm = () => {
    if (!product) return;
    if (finalQuantity <= 0) return;

    const uniqueId = `${product.id}-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 10)}`;

    const newItem: Product = {
      id: product.id,
      cartId: uniqueId,
      name: product.name,
      quantity: finalQuantity,
      unit: tipoMedida,
      pricePerUnit: precioUnitario,
      subtotal: Number((precioUnitario * finalQuantity).toFixed(2)),
      costo: product.costo,
      ivaIncluido: product.ivaIncluido,
      ivaPorcentaje: product.ivaPorcentaje,
      listaPrecioId: selectedRow ? selectedRow.listaPrecioId : null,
      listaPrecioNombre: selectedRow ? selectedRow.nombreLista : etiquetaBase,
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
            {tipoMedida}
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
            value={selectedValue}
            onChange={(e) => setSelectedValue(e.target.value)}
            autoFocus
          >
            <option value="base">
              {etiquetaBase} — ${precioVisual(product?.pricePerUnit ?? 0).toLocaleString()} / {medidaProducto}
            </option>
            {rows.map((r) => (
              <option key={r.listaPrecioId} value={String(r.listaPrecioId)}>
                {r.nombreLista} — ${precioVisual(r.precio).toLocaleString()} /{" "}
                {r.tipoMedida || medidaProducto}
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-1">
          <label
            htmlFor="scan-cantidad"
            className="text-sm font-medium text-gray-700"
          >
            {esPeso ? "Ingrese el peso en gramos" : "Ingrese las unidades"}
          </label>
          <Input
            id="scan-cantidad"
            type="number"
            min="0"
            step={esPeso ? "1" : "1"}
            value={qtyInput}
            onChange={(e) => setQtyInput(e.target.value)}
            placeholder={esPeso ? "Gramos" : "Unidades"}
          />
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
            disabled={finalQuantity <= 0}
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
