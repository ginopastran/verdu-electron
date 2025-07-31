"use client";

import React, { useState, useEffect, useCallback, useMemo } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Search,
  Plus,
  Package,
  ChevronLeft,
  ChevronRight,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { toast } from "sonner";

interface Producto {
  id: number;
  nombre: string;
  tipoMedida: string;
  precio: number;
  stock?: number;
}

interface ProductoSelectorDialogProps {
  open: boolean;
  onClose: () => void;
  onSelect: (producto: Producto) => void;
  excludeProductIds?: number[];
}

const ITEMS_PER_PAGE = 6;

const ProductoSelectorDialog: React.FC<ProductoSelectorDialogProps> = ({
  open,
  onClose,
  onSelect,
  excludeProductIds = [],
}) => {
  const [productos, setProductos] = useState<Producto[]>([]);
  const [searchTerm, setSearchTerm] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);

  // Resetear estado cuando se abre el diálogo
  useEffect(() => {
    if (open) {
      setSearchTerm("");
      setError(null);
      setCurrentPage(1);
      fetchProductos();
    }
  }, [open]);

  // Reset to first page when search changes
  useEffect(() => {
    setCurrentPage(1);
  }, [searchTerm]);

  const fetchProductos = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      const response = await fetch(`${API_URL}/api/productos?limit=1000`);

      if (!response.ok) {
        throw new Error(`Error ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      setProductos(data.productos || []);
    } catch (error) {
      console.error("Error al cargar productos:", error);

      // Mejorar el mensaje de error según el tipo
      let errorMessage = "Error desconocido";
      if (error instanceof Error) {
        if (error.message.includes("<!doctype")) {
          errorMessage =
            "El servidor no está respondiendo correctamente. Verifique que esté ejecutándose en http://localhost:3000";
        } else if (error.message.includes("Failed to fetch")) {
          errorMessage =
            "No se puede conectar al servidor. Verifique que esté ejecutándose";
        } else {
          errorMessage = error.message;
        }
      }

      setError(errorMessage);
      toast.error("Error al cargar la lista de productos");
    } finally {
      setLoading(false);
    }
  }, []);

  const formatCurrency = useCallback((amount: number) => {
    return new Intl.NumberFormat("es-AR", {
      style: "currency",
      currency: "ARS",
    }).format(amount);
  }, []);

  const filteredProductos = useMemo(() => {
    return productos.filter((producto) => {
      const matchesSearch = producto.nombre
        .toLowerCase()
        .includes(searchTerm.toLowerCase());
      const isNotExcluded = !excludeProductIds.includes(producto.id);
      return matchesSearch && isNotExcluded;
    });
  }, [productos, searchTerm, excludeProductIds]);

  const paginatedData = useMemo(() => {
    const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
    const endIndex = startIndex + ITEMS_PER_PAGE;
    return filteredProductos.slice(startIndex, endIndex);
  }, [filteredProductos, currentPage]);

  const totalPages = Math.ceil(filteredProductos.length / ITEMS_PER_PAGE);

  const handleSelectProducto = useCallback(
    (producto: Producto) => {
      onSelect(producto);
      setSearchTerm("");
      onClose();
    },
    [onSelect, onClose]
  );

  const goToPage = useCallback(
    (page: number) => {
      setCurrentPage(Math.max(1, Math.min(page, totalPages)));
    },
    [totalPages]
  );

  // Memoizar el texto de información de resultados
  const resultsInfo = useMemo(() => {
    if (loading) return "Cargando...";

    if (searchTerm) {
      return `${filteredProductos.length} producto${
        filteredProductos.length !== 1 ? "s" : ""
      } encontrado${filteredProductos.length !== 1 ? "s" : ""}`;
    }

    return `${filteredProductos.length} producto${
      filteredProductos.length !== 1 ? "s" : ""
    } disponible${filteredProductos.length !== 1 ? "s" : ""}`;
  }, [loading, searchTerm, filteredProductos.length]);

  // Memoizar las páginas a mostrar
  const pageNumbers = useMemo(() => {
    const pages = [];
    const maxPages = Math.min(5, totalPages);

    if (totalPages <= 5) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else if (currentPage <= 3) {
      for (let i = 1; i <= maxPages; i++) {
        pages.push(i);
      }
    } else if (currentPage >= totalPages - 2) {
      for (let i = totalPages - 4; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
      for (let i = currentPage - 2; i <= currentPage + 2; i++) {
        pages.push(i);
      }
    }

    return pages;
  }, [currentPage, totalPages]);

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-3">
            <Package className="h-5 w-5" />
            Seleccionar Producto
          </DialogTitle>
          <DialogDescription>
            Busque y seleccione productos para agregar a la factura
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-hidden flex flex-col space-y-4">
          {/* Búsqueda */}
          <div className="relative">
            <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Buscar producto..."
              className="pl-9"
              autoFocus
              disabled={loading}
            />
          </div>

          {/* Información de resultados */}
          <div className="text-sm text-muted-foreground flex items-center justify-between">
            <span>{resultsInfo}</span>
            {totalPages > 1 && (
              <span>
                Página {currentPage} de {totalPages}
              </span>
            )}
          </div>

          {/* Error message */}
          {error && (
            <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-md">
              <AlertCircle className="h-4 w-4 text-red-500" />
              <span className="text-sm text-red-700">{error}</span>
            </div>
          )}

          {/* Lista de productos */}
          <div className="flex-1 overflow-y-auto space-y-2">
            {loading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                <span className="ml-2 text-muted-foreground">
                  Cargando productos...
                </span>
              </div>
            ) : paginatedData.length === 0 ? (
              <div className="text-center py-8">
                <Package className="h-12 w-12 text-muted-foreground mx-auto mb-2" />
                <p className="text-muted-foreground">
                  {searchTerm
                    ? "No se encontraron productos con ese criterio de búsqueda"
                    : "No hay productos disponibles"}
                </p>
                {searchTerm && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSearchTerm("")}
                    className="mt-2"
                  >
                    Limpiar búsqueda
                  </Button>
                )}
              </div>
            ) : (
              paginatedData.map((producto) => (
                <Button
                  key={producto.id}
                  variant="outline"
                  className="w-full justify-between h-auto p-4 cursor-pointer hover:bg-accent transition-colors border-[#A7A7A7]"
                  onClick={() => handleSelectProducto(producto)}
                  disabled={loading}
                >
                  <div className="text-left flex-1">
                    <p className="font-medium">{producto.nombre}</p>
                    <div className="text-sm text-muted-foreground flex flex-wrap gap-2 mt-1">
                      <span className="bg-muted px-2 py-1 rounded text-xs">
                        {producto.tipoMedida}
                      </span>
                      <span className="font-medium text-primary">
                        {formatCurrency(producto.precio)}
                      </span>
                      {producto.stock !== undefined && (
                        <span
                          className={`px-2 py-1 rounded text-xs ${
                            producto.stock > 0
                              ? "bg-green-100 text-green-700"
                              : "bg-red-100 text-red-700"
                          }`}
                        >
                          Stock: {producto.stock}
                        </span>
                      )}
                    </div>
                  </div>
                  <Plus className="h-5 w-5 text-muted-foreground ml-4 flex-shrink-0" />
                </Button>
              ))
            )}
          </div>

          {/* Paginación */}
          {totalPages > 1 && !loading && (
            <div className="flex items-center justify-between pt-2 border-t">
              <Button
                variant="outline"
                size="sm"
                onClick={() => goToPage(currentPage - 1)}
                disabled={currentPage === 1}
              >
                <ChevronLeft className="h-4 w-4 mr-1" />
                Anterior
              </Button>

              <div className="flex items-center gap-2">
                {pageNumbers.map((pageNumber) => (
                  <Button
                    key={pageNumber}
                    variant={currentPage === pageNumber ? "default" : "outline"}
                    size="sm"
                    className="w-8 h-8 p-0"
                    onClick={() => goToPage(pageNumber)}
                  >
                    {pageNumber}
                  </Button>
                ))}
              </div>

              <Button
                variant="outline"
                size="sm"
                onClick={() => goToPage(currentPage + 1)}
                disabled={currentPage === totalPages}
              >
                Siguiente
                <ChevronRight className="h-4 w-4 ml-1" />
              </Button>
            </div>
          )}
        </div>

        {/* Botones */}
        <div className="flex justify-end gap-3 pt-4 border-t">
          <Button variant="outline" onClick={onClose} disabled={loading}>
            Cancelar
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default ProductoSelectorDialog;
