export interface Product {
  id: number;
  cartId: string;
  name: string;
  quantity: number;
  unit: string;
  pricePerUnit: number;
  subtotal: number;
  costo: number;
}

export interface AvailableProduct {
  id: number;
  name: string;
  pricePerUnit: number;
  unit: string;
  costo: number;
  codigoBarras: string | null;
}

export interface CartItem extends Product {
  cartId: string;
}

export interface BusinessInfo {
  sistemaPago: string;
  descuentoEfectivo?: number;
}

export interface Screen {
  id: number;
  items: CartItem[];
}

export interface PaymentDetails {
  orderId: number;
  isSplitPayment: boolean;
  cashAmount?: number;
}

export interface OrderData {
  id: number;
  total: number;
  items: Product[];
  paymentMethod: string;
  createdAt: string;
  status: string;
}
