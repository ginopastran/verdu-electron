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
  User,
  ChevronLeft,
  ChevronRight,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { toast } from "sonner";

interface Cliente {
  id: string;
  nombre: string;
  apellido?: string;
  razonSocial?: string;
  cuit?: string;
  condicionFiscal: string;
}

interface PaginationData {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

interface ClienteSelectorDialogProps {
  open: boolean;
  onClose: () => void;
  onSelect: (cliente: Cliente | null) => void;
  selectedClienteId?: string;
}

const ITEMS_PER_PAGE = 10;

const ClienteSelectorDialog: React.FC<ClienteSelectorDialogProps> = ({
  open,
  onClose,
  onSelect,
  selectedClienteId,
}) => {
  const [clientes, setClientes] = useState<Cliente[]>([]);
  const [searchTerm, setSearchTerm] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pagination, setPagination] = useState<PaginationData>({
    page: 1,
    limit: ITEMS_PER_PAGE,
    total: 0,
    totalPages: 0,
    hasNext: false,
    hasPrev: false,
  });

  // Resetear estado cuando se abre el diálogo
  useEffect(() => {
    if (open) {
      setSearchTerm("");
      setError(null);
      setPagination((prev) => ({ ...prev, page: 1 }));
    }
  }, [open]);

  // Fetch clients when page changes or search term changes
  useEffect(() => {
    if (open) {
      fetchClientes();
    }
  }, [open, pagination.page, searchTerm]);

  // Reset to first page when search changes
  useEffect(() => {
    if (searchTerm !== "") {
      setPagination((prev) => ({ ...prev, page: 1 }));
    }
  }, [searchTerm]);

  const fetchClientes = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      const params = new URLSearchParams({
        page: pagination.page.toString(),
        limit: ITEMS_PER_PAGE.toString(),
      });

      if (searchTerm) {
        params.append("search", searchTerm);
      }

      const response = await fetch(`${API_URL}/api/clientes?${params}`);

      if (!response.ok) {
        throw new Error(`Error ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      setClientes(data.clientes || []);
      setPagination(
        data.pagination || {
          page: 1,
          limit: ITEMS_PER_PAGE,
          total: 0,
          totalPages: 0,
          hasNext: false,
          hasPrev: false,
        }
      );
    } catch (error) {
      console.error("Error al cargar clientes:", error);

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
      toast.error("Error al cargar la lista de clientes");
    } finally {
      setLoading(false);
    }
  }, [pagination.page, searchTerm]);

  const formatClienteName = useCallback((cliente: Cliente) => {
    return (
      cliente.razonSocial ||
      `${cliente.nombre} ${cliente.apellido || ""}`.trim()
    );
  }, []);

  const handleSelectCliente = useCallback(
    (cliente: Cliente) => {
      onSelect(cliente);
      onClose();
    },
    [onSelect, onClose]
  );

  const goToPage = useCallback((page: number) => {
    setPagination((prev) => ({
      ...prev,
      page: Math.max(1, Math.min(page, prev.totalPages)),
    }));
  }, []);

  // Memoizar el texto de información de resultados
  const resultsInfo = useMemo(() => {
    if (loading) return "Cargando...";

    if (searchTerm) {
      return `${pagination.total} resultado${
        pagination.total !== 1 ? "s" : ""
      } encontrado${pagination.total !== 1 ? "s" : ""}`;
    }

    return `${pagination.total} cliente${
      pagination.total !== 1 ? "s" : ""
    } total${pagination.total !== 1 ? "es" : ""}`;
  }, [loading, searchTerm, pagination.total]);

  // Memoizar las páginas a mostrar
  const pageNumbers = useMemo(() => {
    const pages = [];
    const maxPages = Math.min(5, pagination.totalPages);

    if (pagination.totalPages <= 5) {
      for (let i = 1; i <= pagination.totalPages; i++) {
        pages.push(i);
      }
    } else if (pagination.page <= 3) {
      for (let i = 1; i <= maxPages; i++) {
        pages.push(i);
      }
    } else if (pagination.page >= pagination.totalPages - 2) {
      for (let i = pagination.totalPages - 4; i <= pagination.totalPages; i++) {
        pages.push(i);
      }
    } else {
      for (let i = pagination.page - 2; i <= pagination.page + 2; i++) {
        pages.push(i);
      }
    }

    return pages;
  }, [pagination.page, pagination.totalPages]);

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-3">
            <User className="h-5 w-5" />
            Seleccionar Cliente
          </DialogTitle>
          <DialogDescription>
            Busque y seleccione un cliente para la factura
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-hidden flex flex-col space-y-4">
          {/* Búsqueda */}
          <div className="relative">
            <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Buscar por nombre, razón social, CUIT..."
              className="pl-9"
              disabled={loading}
            />
          </div>

          {/* Información de resultados */}
          <div className="text-sm text-muted-foreground flex items-center justify-between">
            <span>{resultsInfo}</span>
            {pagination.totalPages > 1 && (
              <span>
                Página {pagination.page} de {pagination.totalPages}
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

          {/* Lista de clientes */}
          <div className="flex-1 overflow-y-auto space-y-2">
            {loading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                <span className="ml-2 text-muted-foreground">
                  Cargando clientes...
                </span>
              </div>
            ) : clientes.length === 0 ? (
              <div className="text-center py-8">
                <User className="h-12 w-12 text-muted-foreground mx-auto mb-2" />
                <p className="text-muted-foreground">
                  {searchTerm
                    ? "No se encontraron clientes con ese criterio de búsqueda"
                    : "No hay clientes disponibles"}
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
              clientes.map((cliente) => (
                <Button
                  key={cliente.id}
                  variant={
                    selectedClienteId === cliente.id ? "default" : "outline"
                  }
                  className="w-full justify-start h-auto p-4 border-[#A7A7A7] hover:bg-accent transition-colors"
                  onClick={() => handleSelectCliente(cliente)}
                  disabled={loading}
                >
                  <div className="text-left flex-1">
                    <div className="flex flex-col gap-1">
                      <p className="font-medium">
                        {formatClienteName(cliente)}
                      </p>
                      <div className="text-sm text-muted-foreground space-y-1">
                        {cliente.cuit && (
                          <p className="flex items-center gap-1">
                            <span className="font-medium">CUIT:</span>{" "}
                            {cliente.cuit}
                          </p>
                        )}
                        <p className="flex items-center gap-1">
                          <span className="font-medium">Condición Fiscal:</span>{" "}
                          {cliente.condicionFiscal}
                        </p>
                      </div>
                    </div>
                  </div>
                  {selectedClienteId === cliente.id && (
                    <div className="ml-2 text-primary">
                      <User className="h-4 w-4" />
                    </div>
                  )}
                </Button>
              ))
            )}
          </div>

          {/* Paginación */}
          {pagination.totalPages > 1 && !loading && (
            <div className="flex items-center justify-between pt-2 border-t">
              <Button
                variant="outline"
                size="sm"
                onClick={() => goToPage(pagination.page - 1)}
                disabled={!pagination.hasPrev}
              >
                <ChevronLeft className="h-4 w-4 mr-1" />
                Anterior
              </Button>

              <div className="flex items-center gap-2">
                {pageNumbers.map((pageNumber) => (
                  <Button
                    key={pageNumber}
                    variant={
                      pagination.page === pageNumber ? "default" : "outline"
                    }
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
                onClick={() => goToPage(pagination.page + 1)}
                disabled={!pagination.hasNext}
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

export default ClienteSelectorDialog;
