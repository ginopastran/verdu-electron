export interface Pago {
  id: number;
  monto: number;
  metodoPago: string;
  fecha: string;
  referencia?: string;
  estado: string;
}

export interface Cliente {
  id: string;
  nombre: string;
  apellido?: string;
  razonSocial?: string;
  cuit?: string;
  condicionFiscal: string;
}

export interface Producto {
  id: number;
  nombre: string;
  precio: number;
  tipoMedida: string;
  stock?: number;
  ivaIncluido?: boolean;
  ivaPorcentaje?: number | null;
}

export interface DetalleFactura {
  id: number;
  descripcion: string;
  cantidad: number;
  precioUnitario: number;
  subtotal: number;
  producto: Producto;
}

export interface Factura {
  id: string;
  numero: string;
  tipoFactura: string;
  puntoVenta: number;
  fecha: string;
  subtotal: number;
  impuestos: number;
  total: number;
  estado: string;
  cae?: string;
  vencimientoCae?: string;
  observaciones?: string;
  cliente?: Cliente;
  detalles: DetalleFactura[];
  pagos?: Pago[];
  esCuentaCorriente?: boolean;
  pagoInicial?: number;
  saldoPendiente?: number;
}

export interface PaginationInfo {
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export interface FacturasResponse {
  facturas: Factura[];
  pagination: PaginationInfo;
}

export interface FiltersState {
  search: string;
  estado: string;
  tipoFactura: string;
  sort: string;
  order: "asc" | "desc";
}

export interface CreateFacturaData {
  clienteId: string;
  tipoFactura: "remito" | "A" | "C";
  observaciones?: string;
  subtotal: number;
  impuestos: number;
  total: number;
  esCuentaCorriente: boolean;
  pagoInicial?: number;
  detalles: Omit<DetalleFactura, "id" | "producto">[];
}

export interface AddPagoData {
  facturaId: string;
  monto: number;
  metodoPago: "efectivo" | "tarjeta" | "transferencia";
  fecha: string;
  referencia?: string;
}
