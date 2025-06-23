import { useState } from "react";
import { toast } from "sonner";
import { Product } from "./useCartState";
import { getBusinessName } from "@/utils/businessHelpers";

interface AfipPaymentOptions {
  user: any;
  API_URL: string;
  appId: string | null;
  clearCart: () => void;
  calculateTotal: () => number;
  setPaymentDialogOpen: (open: boolean) => void;
  searchInputRef?: React.RefObject<HTMLInputElement>;
}

export function useAfipPaymentProcessing({
  user,
  API_URL,
  appId,
  clearCart,
  calculateTotal,
  setPaymentDialogOpen,
  searchInputRef,
}: AfipPaymentOptions) {
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<
    string | null
  >(null);

  // Estados para efectivo con redondeo (igual que el hook normal)
  const [roundedAmountDialogOpen, setRoundedAmountDialogOpen] = useState(false);
  const [originalAmount, setOriginalAmount] = useState<number>(0);
  const [roundedAmount, setRoundedAmount] = useState<number>(0);
  const [applyingDiscount, setApplyingDiscount] = useState(false);

  // Estados para el sistema de pago exacto
  const [exactPaymentDialogOpen, setExactPaymentDialogOpen] = useState(false);
  const [paidAmount, setPaidAmount] = useState<number>(0);
  const [changeAmount, setChangeAmount] = useState<number>(0);

  // Función para redondear a los 50 pesos más cercanos hacia abajo (igual que hook normal)
  const roundToNearest50 = (amount: number): number => {
    const amountFixed = parseFloat(amount.toFixed(2));
    const integerPart = Math.floor(amountFixed);
    const remainder = integerPart % 50;
    const decimalPart = amountFixed - integerPart;

    if (remainder === 0 && decimalPart === 0) {
      console.log("🧮 AFIP: Ya está redondeado a 50:", amountFixed);
      return amountFixed;
    }

    const roundedDown = integerPart - remainder;

    console.log("🧮 AFIP: DEBUG Redondeo:", {
      original: amount,
      redondeadoA2Decimales: amountFixed,
      parteEntera: integerPart,
      parteDecimal: decimalPart,
      resto: remainder,
      redondeadoA50: roundedDown,
    });

    return roundedDown;
  };

  // Helper para llamar al script PHP de impresión AFIP
  const handleAfipTicketPrinting = async (afipData: any) => {
    try {
      console.log("🧾 Iniciando impresión de ticket AFIP...");
      console.log("📄 Datos AFIP para impresión:", afipData);

      // Verificar APIs disponibles
      console.log("🔍 Verificando APIs disponibles:");
      console.log("  - window.printer:", !!(window as any).printer);
      console.log(
        "  - window.printer.printAfipTicket:",
        !!(window as any).printer?.printAfipTicket
      );
      console.log("  - window.electron:", !!(window as any).electron);
      console.log(
        "  - window.electron.ipcRenderer:",
        !!(window as any).electron?.ipcRenderer
      );

      // Método 1: Usar window.printer (API específica para impresión)
      if (
        typeof window !== "undefined" &&
        (window as any).printer?.printAfipTicket
      ) {
        console.log("🖨️ Usando window.printer.printAfipTicket");
        return await (window as any).printer.printAfipTicket(afipData);
      }
      // Método 2: Usar window.electron.ipcRenderer (API general)
      else if (
        typeof window !== "undefined" &&
        (window as any).electron?.ipcRenderer
      ) {
        console.log(
          "🖨️ Usando window.electron.ipcRenderer.invoke('print-afip-ticket')"
        );
        const result = await (window as any).electron.ipcRenderer.invoke(
          "print-afip-ticket",
          afipData
        );
        console.log("📝 Resultado de print-afip-ticket:", result);
        return result;
      }
      // Si ninguna API está disponible
      else {
        console.error("❌ Ninguna API de impresión AFIP disponible");
        throw new Error("API de Electron no disponible para impresión AFIP");
      }
    } catch (error: any) {
      console.error("❌ Error al imprimir ticket AFIP:", error);
      console.error("❌ Stack trace:", error.stack);
      toast.error(`Error al imprimir el ticket AFIP: ${error.message}`);
      return {
        success: false,
        printerError: error.message,
        message: error.message,
      };
    }
  };

  // Función para manejar efectivo AFIP (igual que el hook normal)
  const handleAfipCashPayment = (businessInfo: any, withDiscount = false) => {
    console.log(
      "🛒 AFIP EFECTIVO: Iniciando proceso de pago en efectivo AFIP",
      withDiscount ? "con descuento" : ""
    );

    console.log("🏢 businessInfo recibido:", {
      businessInfo,
      sistemaPago: businessInfo?.sistemaPago,
      descuentoEfectivo: businessInfo?.descuentoEfectivo,
    });

    // Establecer efectivo como método seleccionado
    setSelectedPaymentMethod("efectivo");
    setApplyingDiscount(withDiscount);

    // Calcular los importes para cualquier caso
    const originalTotal = Number(calculateTotal().toFixed(2));
    let finalTotal = originalTotal;

    console.log("💰 AFIP: Total original calculado:", originalTotal);

    // Aplicar descuento si es necesario
    if (withDiscount && businessInfo?.descuentoEfectivo) {
      const discountPercentage = Number(businessInfo.descuentoEfectivo);
      const discountAmount = (originalTotal * discountPercentage) / 100;
      finalTotal = originalTotal - discountAmount;

      console.log("💰 AFIP DESCUENTO: Cálculos:", {
        originalTotal,
        discountPercentage,
        discountAmount,
        finalTotal,
      });
    }

    // Decidir qué flujo usar según el sistemaPago
    if (businessInfo?.sistemaPago === "pago-exacto") {
      console.log("💰 AFIP EFECTIVO: Usando sistema de pago exacto con vuelto");

      // Guardar los montos para el sistema de pago exacto
      setOriginalAmount(originalTotal);
      setRoundedAmount(finalTotal);

      console.log(
        "💾 AFIP: Abriendo diálogo de pago exacto para total:",
        finalTotal
      );

      // Cerrar diálogo de pago AFIP y abrir diálogo de pago exacto
      setPaymentDialogOpen(false);
      setExactPaymentDialogOpen(true);
    } else {
      // Sistema de redondeo (por defecto)
      console.log("🧮 AFIP EFECTIVO: Usando sistema de redondeo tradicional");

      let roundedTotal = roundToNearest50(finalTotal);
      console.log("🧮 AFIP EFECTIVO: Cálculos de redondeo:", {
        finalTotal,
        roundedTotal,
        diferencia: finalTotal - roundedTotal,
        sistemaRedondeo: businessInfo?.sistemaPago,
        redondeoAplicado: true,
      });

      // Guardar los montos calculados en el estado
      setOriginalAmount(originalTotal);
      setRoundedAmount(roundedTotal);

      console.log("💾 AFIP: Valores guardados en estado:", {
        originalAmount: originalTotal,
        roundedAmount: roundedTotal,
        diferencia: originalTotal - roundedTotal,
      });

      // Cerrar diálogo de pago AFIP y mostrar diálogo de redondeo
      setPaymentDialogOpen(false);
      setRoundedAmountDialogOpen(true);
    }
  };

  // Función para confirmar pago exacto AFIP
  const confirmAfipExactPayment = async (
    paidAmount: number,
    change: number,
    items: Product[]
  ) => {
    console.log("💰 AFIP EXACT PAYMENT: Confirmando pago exacto", {
      paidAmount,
      change,
      totalAmount: roundedAmount,
    });

    if (!user) {
      toast.error("Debes iniciar sesión para realizar una factura AFIP");
      return;
    }

    try {
      setIsProcessingPayment(true);

      // Procesar la factura AFIP con el monto exacto
      await processAfipPayment("efectivo", items, roundedAmount);

      // Los estados se limpian en processAfipPayment
      console.log("✅ AFIP EXACT PAYMENT: Pago procesado exitosamente");
    } catch (error: any) {
      console.error("❌ AFIP EXACT PAYMENT: Error al procesar:", error);
      toast.error(`Error en factura AFIP: ${error.message}`);
      resetPaymentState();
    }
  };

  const resetPaymentState = () => {
    console.log(
      "🧹 AFIP RESET: Limpiando estados del procesador de pagos AFIP"
    );
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);
    // Limpiar también estados de efectivo
    setRoundedAmountDialogOpen(false);
    setOriginalAmount(0);
    setRoundedAmount(0);
    setApplyingDiscount(false);
    setExactPaymentDialogOpen(false);
    setPaidAmount(0);
    setChangeAmount(0);
  };

  const processAfipPayment = async (
    method: string,
    items: Product[],
    totalAmount?: number
  ) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una factura");
      return;
    }

    const finalTotal = totalAmount || calculateTotal();

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago AFIP en proceso");
      return;
    }

    // Si es efectivo y no viene de los diálogos de efectivo, manejar redondeo
    if (method === "efectivo" && !totalAmount) {
      console.log(
        "💰 AFIP: Efectivo detectado sin totalAmount, necesita redondeo"
      );
      throw new Error("Use handleAfipCashPayment para pagos en efectivo");
    }

    setIsProcessingPayment(true);
    setSelectedPaymentMethod(method);

    const orderItems = items.map((item) => ({
      productoId: item.id,
      cantidad: item.quantity,
      precio: item.pricePerUnit,
      subtotal: Number(item.subtotal.toFixed(2)),
      nombre: item.name,
    }));

    try {
      console.log("🧾 Creando factura AFIP...");

      // Crear la factura AFIP usando el endpoint
      const afipResponse = await fetch(`${API_URL}/api/facturas/crear-afip`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
        body: JSON.stringify({
          tipoFactura: "B", // Siempre Factura B para consumidores finales
          esConsumidorFinal: true,
          clienteId: null, // null para consumidor final
          productos: orderItems,
          metodoPago: method,
          vendedorId: user.id,
          sucursalId: user.sucursalId,
          observaciones: "Factura generada desde Electron",
        }),
      });

      if (!afipResponse.ok) {
        const errorData = await afipResponse.json().catch(() => ({}));
        throw new Error(errorData.error || "Error al crear la factura AFIP");
      }

      const afipResult = await afipResponse.json();
      console.log("✅ Factura AFIP creada:", afipResult);

      // Verificar que tenemos los datos de AFIP
      if (!afipResult.afip?.cae) {
        throw new Error("No se recibió CAE de AFIP");
      }

      // Preparar datos para impresión
      const printData = {
        ...afipResult,
        metodoPago: method,
        items: orderItems,
        total: finalTotal,
        usuario: user.nombre || "Vendedor",
        fechaHora: new Date().toLocaleString("es-AR"),
        // Calcular descuento si aplica
        ...(method === "efectivo" &&
          originalAmount > finalTotal && {
            descuentoAplicado: originalAmount - finalTotal,
            totalOriginal: originalAmount,
          }),
      };

      // Imprimir ticket AFIP
      await handleAfipTicketPrinting(printData);

      // Mostrar el CAE al usuario
      toast.success(`Factura AFIP creada exitosamente`, {
        description: `CAE: ${afipResult.afip.cae}`,
        duration: 5000,
      });

      // Esperar un poco para que el usuario vea el éxito antes de cerrar
      await new Promise((resolve) => setTimeout(resolve, 1500));

      // Limpiar carrito y estados
      clearCart();
      resetPaymentState();
      setPaymentDialogOpen(false);

      // Devolver el foco al input de búsqueda (igual que las órdenes normales)
      setTimeout(() => {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
        }
      }, 100);
    } catch (error: any) {
      console.error("❌ Error al procesar factura AFIP:", error);
      toast.error(
        `Error: ${error.message || "Error al crear la factura AFIP"}`
      );
      resetPaymentState();
      setPaymentDialogOpen(false);

      // También devolver el foco en caso de error
      setTimeout(() => {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
        }
      }, 100);
    }
  };

  return {
    processAfipPayment,
    handleAfipCashPayment,
    confirmAfipExactPayment,
    isProcessingPayment,
    selectedPaymentMethod,
    resetPaymentState,
    // Estados de efectivo
    roundedAmountDialogOpen,
    setRoundedAmountDialogOpen,
    originalAmount,
    roundedAmount,
    applyingDiscount,
    // Estados de pago exacto
    exactPaymentDialogOpen,
    setExactPaymentDialogOpen,
    paidAmount,
    setPaidAmount,
    changeAmount,
    setChangeAmount,
  };
}
