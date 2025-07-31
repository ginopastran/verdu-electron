import { useState, useEffect, useCallback } from "react";
import { toast } from "sonner";
import { facturasService } from "@/services/facturas";
import {
  Factura,
  FacturasResponse,
  FiltersState,
  PaginationInfo,
  CreateFacturaData,
  AddPagoData,
} from "@/types/factura";

export const useFacturas = () => {
  const [facturas, setFacturas] = useState<Factura[]>([]);
  const [loading, setLoading] = useState(false);
  const [pagination, setPagination] = useState<PaginationInfo>({
    total: 0,
    page: 1,
    limit: 12,
    totalPages: 0,
  });
  const [filters, setFilters] = useState<FiltersState>({
    search: "",
    estado: "",
    tipoFactura: "",
    sort: "fecha",
    order: "desc",
  });

  const fetchFacturas = useCallback(
    async (params = {}) => {
      setLoading(true);
      try {
        console.log("🌐 [useFacturas] Obteniendo facturas del backend...");
        const response = await facturasService.getFacturas({
          page: pagination.page,
          limit: pagination.limit,
          search: filters.search,
          sort: filters.sort,
          order: filters.order,
          estado: filters.estado,
          tipoFactura: filters.tipoFactura,
          excludeTipoFactura: "B", // Excluir facturas tipo "B" (boletas)
          ...params,
        });

        console.log("📄 [useFacturas] Facturas obtenidas:", response);

        // Filtrar facturas tipo "B" en el frontend también por seguridad
        const facturasFiltradas = response.facturas.filter(
          (factura) => factura.tipoFactura !== "B"
        );

        console.log(
          "🔍 [useFacturas] Facturas filtradas (sin tipo B):",
          facturasFiltradas.length
        );

        setFacturas(facturasFiltradas);
        setPagination(response.pagination);

        console.log(
          "✅ Facturas cargadas del backend:",
          response.facturas.length,
          "facturas"
        );
      } catch (error) {
        console.error("Error fetching facturas:", error);
        toast.error("Error al cargar las facturas");
      } finally {
        setLoading(false);
      }
    },
    [pagination.page, pagination.limit, filters]
  );

  const createFactura = async (data: CreateFacturaData): Promise<boolean> => {
    try {
      await facturasService.createFactura(data);
      toast.success("Factura creada exitosamente");
      fetchFacturas();
      return true;
    } catch (error) {
      console.error("Error creating factura:", error);
      toast.error("Error al crear la factura");
      return false;
    }
  };

  const updateFactura = async (
    id: string,
    data: Partial<CreateFacturaData>
  ): Promise<boolean> => {
    try {
      await facturasService.updateFactura(id, data);
      toast.success("Factura actualizada exitosamente");
      fetchFacturas();
      return true;
    } catch (error) {
      console.error("Error updating factura:", error);
      toast.error("Error al actualizar la factura");
      return false;
    }
  };

  const deleteFactura = async (id: string): Promise<boolean> => {
    try {
      await facturasService.deleteFactura(id);
      toast.success("Factura eliminada exitosamente");
      fetchFacturas();
      return true;
    } catch (error) {
      console.error("Error deleting factura:", error);
      toast.error("Error al eliminar la factura");
      return false;
    }
  };

  const addPago = async (data: AddPagoData): Promise<boolean> => {
    try {
      console.log("💰 Agregando pago:", data);
      await facturasService.addPago(data);
      toast.success("Pago agregado exitosamente");
      fetchFacturas(); // Recargar datos
      return true;
    } catch (error) {
      console.error("Error adding pago:", error);
      toast.error("Error al agregar el pago");
      return false;
    }
  };

  const downloadPDF = async (id: string): Promise<void> => {
    try {
      console.log("📄 Descargando PDF para factura:", id);
      await facturasService.downloadPDF(id);
      toast.success("PDF descargado exitosamente");
    } catch (error) {
      console.error("Error downloading PDF:", error);
      toast.error("Error al descargar el PDF");
    }
  };

  const printFactura = async (id: string): Promise<void> => {
    try {
      console.log("🖨️ Imprimiendo factura:", id);
      await facturasService.printFactura(id);
      toast.success("Factura enviada a impresión");
    } catch (error) {
      console.error("Error printing factura:", error);
      toast.error("Error al imprimir la factura");
    }
  };

  const updateFilters = (newFilters: Partial<FiltersState>) => {
    setFilters((prev) => ({ ...prev, ...newFilters }));
    setPagination((prev) => ({ ...prev, page: 1 })); // Reset to first page
  };

  const goToPage = (page: number) => {
    setPagination((prev) => ({ ...prev, page }));
  };

  useEffect(() => {
    fetchFacturas();
  }, [fetchFacturas]);

  return {
    facturas,
    loading,
    pagination,
    filters,
    fetchFacturas,
    createFactura,
    updateFactura,
    deleteFactura,
    addPago,
    downloadPDF,
    printFactura,
    updateFilters,
    goToPage,
  };
};
