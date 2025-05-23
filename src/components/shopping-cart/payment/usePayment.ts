import { useState } from "react";
import { PaymentProcessor } from "./PaymentProcessor";
import { Product, OrderData } from "../types";
import { toast } from "sonner";

interface UsePaymentProps {
  API_URL: string;
  headers: Record<string, string>;
}

export function usePayment({ API_URL, headers }: UsePaymentProps) {
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [currentOrderId, setCurrentOrderId] = useState<number | null>(null);
  const paymentProcessor = new PaymentProcessor({ API_URL, headers });

  const handleCashPayment = async (items: Product[], total: number) => {
    try {
      setIsProcessingPayment(true);
      const orderData = await paymentProcessor.processCashPayment(items, total);
      toast.success("Pago en efectivo procesado correctamente");
      return orderData;
    } catch (error) {
      toast.error("Error al procesar el pago en efectivo");
      throw error;
    } finally {
      setIsProcessingPayment(false);
    }
  };

  const handleCardPayment = async (items: Product[], total: number) => {
    try {
      setIsProcessingPayment(true);
      const orderData = await paymentProcessor.processCardPayment(items, total);
      toast.success("Pago con tarjeta procesado correctamente");
      return orderData;
    } catch (error) {
      toast.error("Error al procesar el pago con tarjeta");
      throw error;
    } finally {
      setIsProcessingPayment(false);
    }
  };

  const handleQRPayment = async (items: Product[], total: number) => {
    try {
      setIsProcessingPayment(true);
      const { orderId, qrData } = await paymentProcessor.generateQRPayment(
        items,
        total
      );
      setCurrentOrderId(orderId);
      return { orderId, qrData };
    } catch (error) {
      toast.error("Error al generar el pago QR");
      throw error;
    }
  };

  const handleSplitPayment = async (
    items: Product[],
    cashAmount: number,
    qrAmount: number
  ) => {
    try {
      setIsProcessingPayment(true);
      const orderData = await paymentProcessor.processSplitPayment(
        items,
        cashAmount,
        qrAmount
      );
      toast.success("Pago dividido procesado correctamente");
      return orderData;
    } catch (error) {
      toast.error("Error al procesar el pago dividido");
      throw error;
    } finally {
      setIsProcessingPayment(false);
    }
  };

  const checkPaymentStatus = async (orderId: number): Promise<OrderData> => {
    return paymentProcessor.checkPaymentStatus(orderId);
  };

  const completeOrder = async (orderId: number) => {
    try {
      const orderData = await paymentProcessor.completeOrder(orderId);
      setCurrentOrderId(null);
      setIsProcessingPayment(false);
      return orderData;
    } catch (error) {
      toast.error("Error al completar la orden");
      throw error;
    }
  };

  return {
    isProcessingPayment,
    currentOrderId,
    handleCashPayment,
    handleCardPayment,
    handleQRPayment,
    handleSplitPayment,
    checkPaymentStatus,
    completeOrder,
  };
}
