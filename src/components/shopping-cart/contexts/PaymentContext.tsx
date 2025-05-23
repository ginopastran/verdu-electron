import { createContext, useContext, ReactNode } from "react";
import { CartItem } from "../types";
import { toast } from "sonner";

interface PaymentContextType {
  handleQRPayment: (
    items: CartItem[],
    amount: number
  ) => Promise<{ orderId: number; qrData: string }>;
  checkPaymentStatus: (orderId: number) => Promise<{ status: string }>;
  completeOrder: (orderId: number) => Promise<void>;
}

interface PaymentProviderProps {
  children: ReactNode;
  API_URL: string;
  headers: Record<string, string>;
}

const PaymentContext = createContext<PaymentContextType | undefined>(undefined);

export function PaymentProvider({
  children,
  API_URL,
  headers,
}: PaymentProviderProps) {
  const handleQRPayment = async (items: CartItem[], amount: number) => {
    try {
      const orderData = {
        monto: amount,
        descripcion: `Compra de ${items.length} productos`,
        items: items.map((item) => ({
          productoId: item.id,
          nombre: item.name,
          cantidad: item.quantity,
          subtotal: Number(item.subtotal.toFixed(2)),
          precioHistorico: item.pricePerUnit,
          costo: Number(item.costo),
        })),
      };

      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers,
        body: JSON.stringify(orderData),
      });

      if (!response.ok) {
        throw new Error("Error al generar el código QR");
      }

      const data = await response.json();
      return { orderId: data.orderId, qrData: data.qrData };
    } catch (error) {
      console.error("Error al generar QR:", error);
      throw error;
    }
  };

  const checkPaymentStatus = async (orderId: number) => {
    try {
      const response = await fetch(
        `${API_URL}/api/mercadopago/check-status?orderId=${orderId}`,
        { headers }
      );

      if (!response.ok) {
        throw new Error("Error al verificar estado del pago");
      }

      const data = await response.json();
      return { status: data.status };
    } catch (error) {
      console.error("Error al verificar estado:", error);
      throw error;
    }
  };

  const completeOrder = async (orderId: number) => {
    try {
      const response = await fetch(
        `${API_URL}/api/mercadopago/complete-order`,
        {
          method: "POST",
          headers,
          body: JSON.stringify({ orderId }),
        }
      );

      if (!response.ok) {
        throw new Error("Error al completar la orden");
      }

      toast.success("Orden completada exitosamente");
    } catch (error) {
      console.error("Error al completar orden:", error);
      throw error;
    }
  };

  return (
    <PaymentContext.Provider
      value={{
        handleQRPayment,
        checkPaymentStatus,
        completeOrder,
      }}
    >
      {children}
    </PaymentContext.Provider>
  );
}

export function usePaymentContext() {
  const context = useContext(PaymentContext);
  if (context === undefined) {
    throw new Error("usePaymentContext must be used within a PaymentProvider");
  }
  return context;
}
