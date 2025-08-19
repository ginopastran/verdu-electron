import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Search, Package, DollarSign, Hash } from "lucide-react";
import { Producto } from "@/types/factura";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { formatearPrecioConIVA } from "@/utils/ivaHelpers";

interface ProductoSelectorProps {
  productos: Producto[];
  onSelect: (producto: Producto) => void;
  onClose: () => void;
}

export const ProductoSelector: React.FC<ProductoSelectorProps> = ({
  productos,
  onSelect,
  onClose,
}) => {
  const [searchTerm, setSearchTerm] = useState("");

  // Obtener información del business para el cálculo de IVA
  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  const filteredProductos = productos.filter((producto) =>
    producto.nombre.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const getStockStatus = (stock?: number) => {
    if (stock === undefined) return null;
    if (stock <= 0)
      return { label: "Sin stock", variant: "destructive" as const };
    if (stock < 10)
      return { label: "Stock bajo", variant: "secondary" as const };
    return { label: "En stock", variant: "default" as const };
  };

  return (
    <Dialog open={true} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Package className="w-5 h-5" />
            Seleccionar Producto
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="search-producto">Buscar producto</Label>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
              <Input
                id="search-producto"
                placeholder="Buscar por nombre..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>

          <div className="max-h-64 overflow-y-auto space-y-2">
            {filteredProductos.map((producto) => {
              const stockStatus = getStockStatus(producto.stock);

              return (
                <Card
                  key={producto.id}
                  className="cursor-pointer hover:bg-muted/50 transition-colors"
                  onClick={() => onSelect(producto)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <Package className="w-4 h-4 text-muted-foreground" />
                          <span className="font-medium">{producto.nombre}</span>
                        </div>
                        <div className="flex items-center gap-4 text-sm text-muted-foreground mb-2">
                          <div className="flex items-center gap-1">
                            <DollarSign className="w-3 h-3" />
                            <span>
                              $
                              {businessInfo
                                ? formatearPrecioConIVA(
                                    producto.precio,
                                    producto.ivaIncluido || false,
                                    producto.ivaPorcentaje ?? null,
                                    businessInfo.ivaIncluidoEnPrecios || false
                                  )
                                : producto.precio.toFixed(2)}
                            </span>
                          </div>
                          <div className="flex items-center gap-1">
                            <Hash className="w-3 h-3" />
                            <span>{producto.tipoMedida}</span>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          {stockStatus && (
                            <Badge
                              variant={stockStatus.variant}
                              className="text-xs"
                            >
                              {stockStatus.label}
                            </Badge>
                          )}
                          {producto.stock !== undefined && (
                            <span className="text-xs text-muted-foreground">
                              Stock: {producto.stock}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>

          {filteredProductos.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">
              <Package className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p>No se encontraron productos</p>
              <p className="text-sm">
                {searchTerm
                  ? "Intenta con otros términos de búsqueda"
                  : "No hay productos registrados"}
              </p>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};
