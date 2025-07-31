import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { FacturaCard } from "./FacturaCard";
import { Factura } from "@/types/factura";
import { Search, Plus, ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import { useDebounce } from "@/hooks/useDebounce";

interface FacturaListProps {
  facturas: Factura[];
  loading: boolean;
  pagination: {
    total: number;
    page: number;
    limit: number;
    totalPages: number;
  };
  filters: {
    search: string;
    cuentaCorriente: string;
    sort: string;
    order: "asc" | "desc";
  };
  onVer: (factura: Factura) => void;
  onEditar: (factura: Factura) => void;
  onDescargar: (id: string) => void;
  onEliminar: (id: string) => void;
  onVerPagos: (factura: Factura) => void;
  onAgregarPago: (factura: Factura) => void;
  onCreateNew: () => void;
  onUpdateFilters: (filters: any) => void;
  onGoToPage: (page: number) => void;
}

export const FacturaList: React.FC<FacturaListProps> = ({
  facturas,
  loading,
  pagination,
  filters,
  onVer,
  onEditar,
  onDescargar,
  onEliminar,
  onVerPagos,
  onAgregarPago,
  onCreateNew,
  onUpdateFilters,
  onGoToPage,
}) => {
  const [searchValue, setSearchValue] = useState(filters.search);
  const debouncedSearch = useDebounce(searchValue, 300);

  useEffect(() => {
    if (debouncedSearch !== filters.search) {
      onUpdateFilters({ search: debouncedSearch });
    }
  }, [debouncedSearch, filters.search, onUpdateFilters]);

  const handleSortChange = (value: string) => {
    const [sort, order] = value.split("-");
    onUpdateFilters({ sort, order: order as "asc" | "desc" });
  };

  const handleCuentaCorrienteChange = (value: string) => {
    onUpdateFilters({ cuentaCorriente: value === "all" ? "" : value });
  };

  const formatSortValue = () => {
    return `${filters.sort}-${filters.order}`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex items-center gap-2">
          <Loader2 className="w-6 h-6 animate-spin text-emerald-600" />
          <span className="text-gray-600">Cargando facturas...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header con filtros */}
      <div className="flex flex-col lg:flex-row gap-4 items-start lg:items-center justify-between">
        <div className="flex flex-col sm:flex-row gap-3 flex-1">
          {/* Búsqueda */}
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
            <Input
              placeholder="Buscar facturas de cuenta corriente..."
              value={searchValue}
              onChange={(e) => setSearchValue(e.target.value)}
              className="pl-10"
            />
          </div>

          {/* Filtros */}
          <div className="flex gap-2">
            <Select
              value={filters.cuentaCorriente || "all"}
              onValueChange={handleCuentaCorrienteChange}
            >
              <SelectTrigger className="w-48">
                <SelectValue placeholder="Todas las facturas" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">Todas las facturas CC</SelectItem>
                <SelectItem value="cuenta_corriente">
                  Solo Cuenta Corriente
                </SelectItem>
                <SelectItem value="pagadas">CC Pagadas</SelectItem>
                <SelectItem value="pendientes">CC Pendientes</SelectItem>
              </SelectContent>
            </Select>

            <Select value={formatSortValue()} onValueChange={handleSortChange}>
              <SelectTrigger className="w-48">
                <SelectValue placeholder="Ordenar por" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="fecha-desc">Fecha (más reciente)</SelectItem>
                <SelectItem value="fecha-asc">Fecha (más antigua)</SelectItem>
                <SelectItem value="numero-desc">
                  Número (descendente)
                </SelectItem>
                <SelectItem value="numero-asc">Número (ascendente)</SelectItem>
                <SelectItem value="total-desc">
                  Total (mayor a menor)
                </SelectItem>
                <SelectItem value="total-asc">Total (menor a mayor)</SelectItem>
                <SelectItem value="cliente-asc">Cliente (A-Z)</SelectItem>
                <SelectItem value="cliente-desc">Cliente (Z-A)</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Botón Nueva Factura */}
        <Button
          onClick={onCreateNew}
          className="bg-emerald-gradient text-white hover:text-white "
        >
          <Plus className="w-4 h-4 mr-2" />
          Nueva Factura
        </Button>
      </div>

      {/* Lista de facturas */}
      {facturas.length === 0 ? (
        <div className="text-center py-12">
          <div className="text-gray-600 mb-4">
            {filters.search || filters.cuentaCorriente
              ? "No se encontraron facturas con los filtros aplicados"
              : "No hay facturas de cuenta corriente registradas"}
          </div>
          <div className="text-sm text-gray-500 mb-4">
            Esta página muestra solo facturas de cuenta corriente (remitos,
            facturas A, C). Las boletas (tipo B) se pueden ver en el historial.
          </div>
          {filters.search || filters.cuentaCorriente ? (
            <Button
              variant="outline"
              onClick={() =>
                onUpdateFilters({ search: "", cuentaCorriente: "" })
              }
            >
              Limpiar filtros
            </Button>
          ) : (
            <Button
              onClick={onCreateNew}
              className="bg-emerald-600 hover:bg-emerald-700"
            >
              <Plus className="w-4 h-4 mr-2" />
              Crear primera factura
            </Button>
          )}
        </div>
      ) : (
        <>
          {/* Grid de facturas */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {facturas.map((factura) => (
              <FacturaCard
                key={factura.id}
                factura={factura}
                onVer={onVer}
                onEditar={onEditar}
                onDescargar={onDescargar}
                onEliminar={onEliminar}
                onVerPagos={onVerPagos}
                onAgregarPago={onAgregarPago}
              />
            ))}
          </div>

          {/* Paginación */}
          {pagination.totalPages > 1 && (
            <div className="flex items-center justify-between">
              <div className="text-sm text-gray-500">
                Mostrando {(pagination.page - 1) * pagination.limit + 1} a{" "}
                {Math.min(pagination.page * pagination.limit, pagination.total)}{" "}
                de {pagination.total} facturas
              </div>

              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onGoToPage(pagination.page - 1)}
                  disabled={pagination.page <= 1}
                >
                  <ChevronLeft className="w-4 h-4" />
                  Anterior
                </Button>

                <div className="flex items-center gap-1">
                  {Array.from(
                    { length: Math.min(5, pagination.totalPages) },
                    (_, i) => {
                      const page = i + 1;
                      return (
                        <Button
                          key={page}
                          variant={
                            page === pagination.page ? "default" : "outline"
                          }
                          size="sm"
                          onClick={() => onGoToPage(page)}
                          className="w-8 h-8 p-0"
                        >
                          {page}
                        </Button>
                      );
                    }
                  )}
                </div>

                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onGoToPage(pagination.page + 1)}
                  disabled={pagination.page >= pagination.totalPages}
                >
                  Siguiente
                  <ChevronRight className="w-4 h-4" />
                </Button>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};
