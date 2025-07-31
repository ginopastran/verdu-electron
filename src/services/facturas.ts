import { api } from "./api";
import {
  Factura,
  FacturasResponse,
  CreateFacturaData,
  AddPagoData,
  Cliente,
  Producto,
} from "@/types/factura";

export const facturasService = {
  // Obtener lista de facturas
  async getFacturas(
    params: {
      page?: number;
      limit?: number;
      search?: string;
      sort?: string;
      order?: "asc" | "desc";
      excludeTipoFactura?: string;
      estado?: string;
      tipoFactura?: string;
    } = {}
  ): Promise<FacturasResponse> {
    // Para la página de cuenta corriente, excluir facturas tipo "B" por defecto
    const defaultParams = {
      excludeTipoFactura: "B",
      ...params,
    };
    return api.get("/facturas", defaultParams);
  },

  // Obtener factura específica
  async getFactura(id: string): Promise<Factura> {
    return api.get(`/facturas/${id}`);
  },

  // Crear nueva factura
  async createFactura(data: CreateFacturaData): Promise<Factura> {
    return api.post("/facturas", data);
  },

  // Actualizar factura
  async updateFactura(
    id: string,
    data: Partial<CreateFacturaData>
  ): Promise<Factura> {
    return api.put(`/facturas/${id}`, data);
  },

  // Eliminar factura
  async deleteFactura(id: string): Promise<void> {
    return api.delete(`/facturas/${id}`);
  },

  // Agregar pago a factura
  async addPago(data: AddPagoData): Promise<{ success: boolean; pago: any }> {
    return api.post("/facturas/agregar-pago", data);
  },

  // Obtener clientes
  async getClientes(limit: number = 1000): Promise<Cliente[]> {
    return api.get("/clientes", { limit });
  },

  // Obtener productos
  async getProductos(limit: number = 1000): Promise<Producto[]> {
    return api.get("/productos", { limit });
  },

  // Descargar PDF de factura
  async downloadPDF(id: string): Promise<Blob> {
    const response = await fetch(
      `${
        import.meta.env.VITE_API_URL || "http://localhost:3000"
      }/api/facturas/${id}/pdf`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    if (!response.ok) {
      throw new Error(`Error al descargar PDF: ${response.status}`);
    }

    return response.blob();
  },

  // Imprimir factura
  async printFactura(id: string): Promise<{ success: boolean }> {
    return api.post(`/facturas/${id}/imprimir`);
  },
};
