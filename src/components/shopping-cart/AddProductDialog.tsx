import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Skeleton } from "@/components/ui/skeleton";
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
import { useScaleWeight } from "@/hooks/useScaleWeight";
import { useAuth } from "@/contexts/AuthContext";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

interface AddProductDialogProps {
  isOpen: boolean;
  product: AvailableProduct | null;
  onClose: () => void;
  onAddToCart: (product: Product) => void;
}

type FilaLista = {
  listaPrecioId: number;
  precio: number;
  nombreLista: string;
  // tipoMedida específico de la lista (Unidad/Kg). null = hereda del producto
  tipoMedida?: string | null;
};

export function AddProductDialog({
  isOpen,
  product,
  onClose,
  onAddToCart,
}: AddProductDialogProps) {
  const [quantity, setQuantity] = useState<string>("");
  const [useManualWeight, setUseManualWeight] = useState(false);
  const [isAddingToCart, setIsAddingToCart] = useState(false);
  const [filasLista, setFilasLista] = useState<FilaLista[]>([]);
  const [listaPreciosLoading, setListaPreciosLoading] = useState(false);
  const [selectedBase, setSelectedBase] = useState(true);
  const [selectedListaId, setSelectedListaId] = useState<number | null>(null);
  const weight = useScaleWeight();
  const { user } = useAuth();

  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  const authHeaders: Record<string, string> = {
    "Content-Type": "application/json",
    ...(appId ? { "X-App-ID": appId } : {}),
  };

  useEffect(() => {
    if (user?.permisos?.pesoManualEnabled === true) {
      setUseManualWeight(true);
    } else {
      setUseManualWeight(false);
    }
  }, [user]);

  useEffect(() => {
    if (!isOpen) {
      setQuantity("");
      setIsAddingToCart(false);
      setFilasLista([]);
      setListaPreciosLoading(false);
      setSelectedBase(true);
      setSelectedListaId(null);
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen || !product) return;
    let cancelled = false;
    setListaPreciosLoading(true);
    setFilasLista([]);
    (async () => {
      try {
        const res = await fetch(
          `${API_URL}/api/productos/${product.id}/listas-precios`,
          { credentials: "include", headers: authHeaders }
        );
        if (!res.ok || cancelled) return;
        const data = await res.json();
        const productos = data.productos || [];
        const rows: FilaLista[] = productos
          .filter(
            (x: any) => x.activa && x.listaPrecio?.activa
          )
          .map((x: any) => ({
            listaPrecioId: x.listaPrecioId,
            precio: x.precio,
            nombreLista: x.listaPrecio.nombre,
            tipoMedida: x.tipoMedida ?? null,
          }));
        if (cancelled) return;
        setFilasLista(rows);
        const defId = businessInfo?.listaPrecioPorDefectoId;
        const match =
          defId != null ? rows.find((r) => r.listaPrecioId === defId) : null;
        if (match) {
          setSelectedBase(false);
          setSelectedListaId(match.listaPrecioId);
        } else {
          setSelectedBase(true);
          setSelectedListaId(null);
        }
      } catch {
        if (!cancelled) {
          setFilasLista([]);
          setSelectedBase(true);
          setSelectedListaId(null);
        }
      } finally {
        if (!cancelled) setListaPreciosLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [
    isOpen,
    product?.id,
    businessInfo?.listaPrecioPorDefectoId,
    API_URL,
  ]);

  const etiquetaBase =
    (businessInfo?.etiquetaPrecioBase || "Precio catálogo").trim();

  const precioUnitarioEfectivo = (() => {
    if (!product) return 0;
    if (selectedBase || selectedListaId == null) return product.pricePerUnit;
    const row = filasLista.find((r) => r.listaPrecioId === selectedListaId);
    return row?.precio ?? product.pricePerUnit;
  })();

  // Medida efectiva: la de la lista seleccionada, o la del producto (precio base)
  const tipoMedidaEfectivo = (() => {
    if (!product) return "Unidad";
    if (selectedBase || selectedListaId == null) return product.unit || "Unidad";
    const row = filasLista.find((r) => r.listaPrecioId === selectedListaId);
    return row?.tipoMedida || product.unit || "Unidad";
  })();
  const esPeso = tipoMedidaEfectivo === "Kg";

  const precioVisualLinea = (unit: number) =>
    businessInfo && product
      ? calcularPrecioVisualConIVA(
          unit,
          product.ivaIncluido || false,
          product.ivaPorcentaje ?? null,
          businessInfo.ivaIncluidoEnPrecios || false
        )
      : unit;

  const handleAddToCart = () => {
    if (isAddingToCart || listaPreciosLoading || !product) return;

    setIsAddingToCart(true);

    try {
      let finalQuantity: number;

      if (esPeso) {
        if (useManualWeight) {
          if (!quantity) {
            throw new Error(
              "Se requiere especificar una cantidad para este producto"
            );
          }
          finalQuantity = parseFloat(quantity) / 1000;
        } else {
          finalQuantity = weight / 1000;
        }
      } else {
        if (!quantity) {
          throw new Error(
            "Se requiere especificar una cantidad para este producto"
          );
        }
        finalQuantity = parseFloat(quantity);
      }

      if (isNaN(finalQuantity) || finalQuantity <= 0) {
        throw new Error("La cantidad debe ser un número mayor que cero");
      }

      const uniqueId = `${product.id}-${Date.now()}-${Math.random()
        .toString(36)
        .substring(2, 10)}-${Math.random().toString(36).substring(2, 10)}`;

      const rowSel =
        !selectedBase && selectedListaId != null
          ? filasLista.find((r) => r.listaPrecioId === selectedListaId)
          : null;

      const newItem: Product = {
        id: product.id,
        cartId: uniqueId,
        name: product.name,
        quantity: finalQuantity,
        unit: tipoMedidaEfectivo,
        pricePerUnit: precioUnitarioEfectivo,
        subtotal: Number((precioUnitarioEfectivo * finalQuantity).toFixed(2)),
        costo: product.costo,
        ivaIncluido: product.ivaIncluido,
        ivaPorcentaje: product.ivaPorcentaje,
        listaPrecioId: selectedBase ? null : selectedListaId,
        listaPrecioNombre: selectedBase
          ? etiquetaBase
          : rowSel?.nombreLista ?? null,
      };

      onAddToCart(newItem);

      onClose();
    } catch (error: any) {
      console.error("Error:", error.message);
      alert(error.message || "Error al añadir producto");
    } finally {
      setTimeout(() => {
        setIsAddingToCart(false);
      }, 500);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !isAddingToCart && !listaPreciosLoading) {
      e.preventDefault();

      if (esPeso && !useManualWeight) {
        handleAddToCart();
      } else if (quantity) {
        handleAddToCart();
      }
    }
  };

  const selectValue =
    selectedBase || selectedListaId == null
      ? "base"
      : String(selectedListaId);

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
            Agregar producto
          </DialogTitle>
          <DialogDescription>
            {product?.name}
            {selectedBase || selectedListaId == null
              ? ` — ${etiquetaBase}`
              : filasLista.find((r) => r.listaPrecioId === selectedListaId)
                ? ` — ${
                    filasLista.find((r) => r.listaPrecioId === selectedListaId)
                      ?.nombreLista
                  }`
                : ""}{" "}
            — $
            {precioVisualLinea(precioUnitarioEfectivo).toLocaleString()} /{" "}
            {tipoMedidaEfectivo}
          </DialogDescription>
        </DialogHeader>

        {listaPreciosLoading && (
          <div className="space-y-2" aria-busy="true" aria-label="Cargando listas de precios">
            <Skeleton className="h-4 w-36" />
            <Skeleton className="h-10 w-full" />
          </div>
        )}

        {!listaPreciosLoading && filasLista.length > 0 && (
          <div className="space-y-1">
            <label
              htmlFor="lista-precio-pos"
              className="text-sm font-medium text-gray-700"
            >
              Lista de precios
            </label>
            <select
              id="lista-precio-pos"
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              value={selectValue}
              onChange={(e) => {
                const v = e.target.value;
                if (v === "base") {
                  setSelectedBase(true);
                  setSelectedListaId(null);
                } else {
                  setSelectedBase(false);
                  setSelectedListaId(Number(v));
                }
              }}
            >
              <option value="base">{etiquetaBase}</option>
              {filasLista.map((r) => (
                <option key={r.listaPrecioId} value={String(r.listaPrecioId)}>
                  {r.nombreLista} — $
                  {precioVisualLinea(r.precio).toLocaleString()}
                </option>
              ))}
            </select>
          </div>
        )}

        {esPeso && (
          <div className="flex items-center space-x-4 py-2">
            {user?.permisos?.pesoManualEnabled === true && (
              <>
                <Switch
                  id="weight-mode"
                  checked={useManualWeight}
                  onCheckedChange={setUseManualWeight}
                  className="data-[state=checked]:bg-emerald-700"
                />
                <label
                  htmlFor="weight-mode"
                  className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                  Peso manual
                </label>
              </>
            )}
            {user?.permisos?.pesoManualEnabled !== true && (
              <p className="text-sm text-gray-500">Modo de peso automático</p>
            )}
          </div>
        )}

        {!esPeso || useManualWeight ? (
          <Input
            type="number"
            placeholder={esPeso ? "Ingrese el peso en gramos" : "Ingrese las unidades"}
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            step="1"
            min="0"
            autoFocus
          />
        ) : (
          <div className="text-start py-2">
            <p className="text-3xl font-bold">{weight} g</p>
            <p className="text-sm text-gray-500">Peso de la balanza</p>
          </div>
        )}

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
            onClick={handleAddToCart}
            className="bg-emerald-gradient text-white hover:text-white text-base"
            type="button"
            disabled={isAddingToCart || listaPreciosLoading}
            autoFocus={esPeso && !useManualWeight}
            tabIndex={1}
          >
            {isAddingToCart ? (
              <div className="flex items-center gap-2">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                <span>Agregando...</span>
              </div>
            ) : (
              "Agregar al carrito"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
