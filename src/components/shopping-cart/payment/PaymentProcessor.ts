import { Product, OrderData } from "../types";

interface PaymentProcessorConfig {
  API_URL: string;
  headers: Record<string, string>;
}

export class PaymentProcessor {
  private config: PaymentProcessorConfig;

  constructor(config: PaymentProcessorConfig) {
    this.config = config;
  }

  private async makeRequest(endpoint: string, options: RequestInit) {
    const response = await fetch(`${this.config.API_URL}${endpoint}`, {
      ...options,
      headers: {
        ...this.config.headers,
        ...options.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`Error en la solicitud: ${response.status}`);
    }

    return response.json();
  }

  async processCashPayment(
    items: Product[],
    total: number
  ): Promise<OrderData> {
    return this.makeRequest("/api/orders", {
      method: "POST",
      body: JSON.stringify({
        items,
        total,
        paymentMethod: "efectivo",
      }),
    });
  }

  async processCardPayment(
    items: Product[],
    total: number
  ): Promise<OrderData> {
    return this.makeRequest("/api/orders", {
      method: "POST",
      body: JSON.stringify({
        items,
        total,
        paymentMethod: "tarjeta",
      }),
    });
  }

  async generateQRPayment(
    items: Product[],
    total: number
  ): Promise<{ orderId: number; qrData: string }> {
    return this.makeRequest("/api/qr-payment", {
      method: "POST",
      body: JSON.stringify({
        items,
        total,
      }),
    });
  }

  async processSplitPayment(
    items: Product[],
    cashAmount: number,
    qrAmount: number
  ): Promise<OrderData> {
    return this.makeRequest("/api/split-payment", {
      method: "POST",
      body: JSON.stringify({
        items,
        cashAmount,
        qrAmount,
      }),
    });
  }

  async checkPaymentStatus(orderId: number): Promise<OrderData> {
    return this.makeRequest(`/api/orders/${orderId}/status`, {
      method: "GET",
    });
  }

  async completeOrder(orderId: number): Promise<OrderData> {
    return this.makeRequest(`/api/orders/${orderId}/complete`, {
      method: "POST",
    });
  }
}
