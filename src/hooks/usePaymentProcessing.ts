import { useState, useEffect } from "react";
import { toast } from "sonner";
import QRCode from "qrcode";
import { Product } from "./useCartState";

interface PaymentOptions {
  user: any;
  API_URL: string;
  appId: string | null;
  clearCart: () => void;
  calculateTotal: () => number;
}

// Declara la interface para las funciones y estados externos que se inyectarán
interface PaymentState {
  setQrDialogOpen?: (open: boolean) => void;
  setSplitPaymentDialogOpen?: (open: boolean) => void;
  qrDialogOpen?: boolean;
}

export function usePaymentProcessing({
  user,
  API_URL,
  appId,
  clearCart,
  calculateTotal,
}: PaymentOptions) {
  // Estado para QR
  const [qrData, setQrData] = useState<any>(null);
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(
    null
  );
  const [paymentStatus, setPaymentStatus] = useState<string | null>(null);

  // Estado para el procesamiento de pagos
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<
    string | null
  >(null);

  // Estado para pago en efectivo con redondeo
  const [roundedAmountDialogOpen, setRoundedAmountDialogOpen] = useState(false);
  const [originalAmount, setOriginalAmount] = useState<number>(0);
  const [roundedAmount, setRoundedAmount] = useState<number>(0);
  const [applyingDiscount, setApplyingDiscount] = useState(false);

  // Estado para pago mixto
  const [cashAmount, setCashAmount] = useState<string>("");
  const [secondPaymentMethod, setSecondPaymentMethod] =
    useState<string>("tarjeta");

  // Referencias para controles externos
  // (estas se establecerán desde el componente principal)
  let setQrDialogOpenRef: ((open: boolean) => void) | undefined = undefined;
  let setSplitPaymentDialogOpenRef: ((open: boolean) => void) | undefined =
    undefined;
  let qrDialogOpenRef: boolean = false;

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // Función para redondear a los 50 pesos más cercanos hacia abajo
  const roundToNearest50 = (amount: number): number => {
    // Primero asegurarse de que estamos trabajando con un número entero
    // Redondear a 2 decimales primero para evitar problemas de punto flotante
    const amountFixed = parseFloat(amount.toFixed(2));

    // Obtener la parte entera
    const integerPart = Math.floor(amountFixed);

    // Calcular el resto solo con la parte entera
    const remainder = integerPart % 50;

    // Obtener la parte decimal
    const decimalPart = amountFixed - integerPart;

    // Si el resto es 0 y no hay decimales, ya está redondeado a 50
    if (remainder === 0 && decimalPart === 0) {
      console.log("🧮 Ya está redondeado a 50:", amountFixed);
      return amountFixed;
    }

    // Redondear hacia abajo a múltiplo de 50
    const roundedDown = integerPart - remainder;

    console.log("🧮 DEBUG Redondeo:", {
      original: amount,
      redondeadoA2Decimales: amountFixed,
      parteEntera: integerPart,
      parteDecimal: decimalPart,
      resto: remainder,
      redondeadoA50: roundedDown,
    });

    return roundedDown;
  };

  // Función para procesar un pago estándar
  const processPayment = async (
    method: string,
    finalTotal: number,
    items: Product[]
  ) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      resetPaymentState();
      return;
    }

    setIsProcessingPayment(true);

    const orderItems = items.map((item) => ({
      productoId: item.id,
      cantidad: item.quantity,
      subtotal: Number(item.subtotal.toFixed(2)),
      precioHistorico: item.pricePerUnit,
      costo: Number(item.costo),
      nombre: item.name,
    }));

    const orderData = {
      metodoPago: method,
      total: finalTotal,
      items: orderItems,
      vendedorId: user.id,
      sucursalId: user.sucursalId,
      vendedor: user.nombre,
      createdAt: new Date().toISOString(),
    };

    try {
      // Crear la orden
      const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
        method: "POST",
        headers,
        body: JSON.stringify(orderData),
      });

      if (!orderResponse.ok) {
        throw new Error("Error al crear la orden");
      }

      // Imprimir ticket usando Electron IPC
      try {
        const { ipcRenderer } = window.require("electron");
        console.log("Enviando datos para impresión:", orderData);
        toast.info("Imprimiendo ticket...", {
          duration: 3000,
          description: "Enviando datos a la impresora",
        });

        const result = await ipcRenderer.invoke("print-ticket", orderData);
        console.log("Resultado de impresión:", result);

        if (result.success) {
          toast.success("Ticket impreso correctamente");
        } else {
          throw new Error(result.message || "Error desconocido al imprimir");
        }
      } catch (printError: any) {
        console.error("Error detallado al imprimir:", printError);
        toast.error(`Error al imprimir el ticket: ${printError.message}`);
      }

      // Limpiar carrito y estados
      clearCart();
      resetPaymentState();
      toast.success("Orden completada exitosamente");
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al procesar la orden");
      resetPaymentState();
    }
  };

  // Función para manejar un pago en efectivo con redondeo
  const handleCashPayment = (businessInfo: any, withDiscount = false) => {
    console.log(
      "🛒 EFECTIVO: Iniciando proceso de pago en efectivo",
      withDiscount ? "con descuento" : ""
    );

    // Establecer el estado de descuento
    setApplyingDiscount(withDiscount);

    // Calcular los importes para cualquier caso
    const originalTotal = Number(calculateTotal().toFixed(2));
    let finalTotal = originalTotal;

    // Aplicar descuento si es necesario
    if (withDiscount && businessInfo?.descuentoEfectivo) {
      const discountPercentage = Number(businessInfo.descuentoEfectivo);
      const discountAmount = (originalTotal * discountPercentage) / 100;
      finalTotal = originalTotal - discountAmount;

      console.log("💰 DESCUENTO: Cálculos:", {
        originalTotal,
        discountPercentage,
        discountAmount,
        finalTotal,
      });
    }

    // Si el sistema de pago es redondeo, calcular el monto redondeado
    let roundedTotal = finalTotal;
    if (businessInfo?.sistemaPago === "redondeo") {
      roundedTotal = roundToNearest50(finalTotal);
      console.log("🧮 EFECTIVO: Cálculos de redondeo:", {
        finalTotal,
        roundedTotal,
        diferencia: finalTotal - roundedTotal,
        sistemaRedondeo: businessInfo?.sistemaPago,
      });
    } else {
      console.log("💰 EFECTIVO: No hay redondeo, usando monto original:", {
        finalTotal,
        sistemaRedondeo: businessInfo?.sistemaPago,
      });
    }

    // Guardar los montos calculados en el estado
    setOriginalAmount(originalTotal);
    setRoundedAmount(roundedTotal);

    // Establecer efectivo como método seleccionado
    setSelectedPaymentMethod("efectivo");

    // Mostrar diálogo de redondeo o confirmación
    setRoundedAmountDialogOpen(true);
  };

  // Función para generar un pago QR
  const generateQRPayment = async (items: Product[]) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso");
      return;
    }

    setIsProcessingPayment(true);
    setSelectedPaymentMethod("qr");

    try {
      const orderItems = items.map((item) => ({
        productoId: item.id,
        nombre: item.name,
        cantidad: item.quantity,
        subtotal: Number(item.subtotal.toFixed(2)),
        precioHistorico: item.pricePerUnit,
        costo: Number(item.costo),
      }));

      const orderData = {
        monto: Number(calculateTotal().toFixed(2)),
        descripcion: `Compra de ${orderItems.length} productos`,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        externalPosId: import.meta.env.VITE_POS_ID,
        items: orderItems,
      };

      console.log("🔄 Enviando solicitud para generar QR:", orderData);

      // Mostrar cargando
      toast.loading("Generando código QR...", { id: "qr-loading" });

      // Realizar la solicitud para generar el QR
      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
        body: JSON.stringify(orderData),
      });

      if (!response.ok) {
        throw new Error("Error al generar el código QR");
      }

      const data = await response.json();
      console.log("✅ QR generado:", data);

      // Generar QR localmente
      let qrImageDataUrl;
      try {
        if (data.qrData) {
          qrImageDataUrl = await QRCode.toDataURL(data.qrData, {
            errorCorrectionLevel: "H",
            margin: 1,
            width: 256,
            color: {
              dark: "#000000",
              light: "#FFFFFF",
            },
          });
          console.log("✅ QR generado localmente con éxito");
        } else {
          throw new Error("Datos de QR no disponibles");
        }
      } catch (qrError: any) {
        console.error("❌ Error al generar QR local:", qrError);
        qrImageDataUrl = data.qrImageUrl || null;
        if (!qrImageDataUrl) {
          throw new Error("No se pudo generar ni obtener imagen QR");
        }
      }

      // Guardar datos del QR
      setQrData({
        ...data,
        qrImageUrl: qrImageDataUrl,
        monto: orderData.monto,
        items: orderItems,
      });

      // Cerrar el toast de carga
      toast.dismiss("qr-loading");

      // Abrir el diálogo QR
      console.log("🔄 Intentando abrir diálogo QR...");
      if (setQrDialogOpenRef) {
        setQrDialogOpenRef(true);
        console.log("✅ Función para abrir diálogo QR ejecutada");
      } else {
        console.error(
          "❌ No se pudo abrir el diálogo QR - función no disponible"
        );
      }

      // Iniciar el polling para verificar el estado del pago
      startPaymentStatusPolling(data.orderId);
    } catch (error: any) {
      console.error("❌ Error al generar QR:", error);
      toast.dismiss("qr-loading");
      toast.error(`Error al generar QR: ${error.message}`);
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    }
  };

  // Verificar el estado del pago
  const startPaymentStatusPolling = (orderId: number) => {
    console.log(
      "🔄 Iniciando polling para verificar estado del pago:",
      orderId
    );
    setPaymentStatus("PENDIENTE");

    // Limpiar intervalo existente
    cleanupPolling();

    // Crear nuevo intervalo
    const interval = setInterval(async () => {
      // Verificar si el diálogo está cerrado
      if (!qrDialogOpenRef) {
        console.log("⚠️ Diálogo QR cerrado, deteniendo polling");
        cleanupPolling();
        return;
      }

      try {
        console.log("🔄 Verificando estado del pago...");
        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status?orderId=${orderId}`,
          {
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
          }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        console.log("🔄 Estado actual del pago:", statusData);

        setPaymentStatus(statusData.status);

        // Si el pago se completó o canceló
        if (statusData.isCompleted || statusData.isCancelled) {
          console.log("🛑 Pago completado o cancelado, deteniendo polling");
          cleanupPolling();

          if (statusData.isCompleted) {
            console.log("✅ Pago completado exitosamente");
            toast.success("¡Pago completado! Cerrando en 5 segundos...");

            // Cerrar automáticamente después de 5 segundos
            setTimeout(() => {
              if (setQrDialogOpenRef) {
                setQrDialogOpenRef(false);
              }
              // Limpiar carrito
              clearCart();
              setIsProcessingPayment(false);
              setSelectedPaymentMethod(null);
            }, 5000);

            finalizeMPPayment(statusData);
          } else {
            console.log("❌ Pago cancelado o rechazado");
            toast.error("El pago ha sido cancelado o rechazado");
            if (setQrDialogOpenRef) {
              setQrDialogOpenRef(false);
            }
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
          }
        }
      } catch (error: any) {
        console.error("❌ Error al verificar estado:", error);
      }
    }, 3000);

    setPollingInterval(interval);

    // Timeout después de 5 minutos
    setTimeout(() => {
      if (pollingInterval) {
        console.log("⏱️ Tiempo de espera agotado");
        cleanupPolling();
        toast.error("Tiempo de espera agotado. Intente nuevamente.");
        if (setQrDialogOpenRef) {
          setQrDialogOpenRef(false);
        }
        setIsProcessingPayment(false);
        setSelectedPaymentMethod(null);
      }
    }, 5 * 60 * 1000);
  };

  // Cancelar el pago con QR
  const cancelQRPayment = () => {
    console.log("❌ Pago con QR cancelado por el usuario");
    cleanupPolling();

    // Cerrar diálogo y limpiar estados
    if (setQrDialogOpenRef) {
      setQrDialogOpenRef(false);
    }
    setQrData(null);
    setPaymentStatus(null);
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);
  };

  // Limpiar intervalo de polling
  const cleanupPolling = () => {
    console.log("🧹 Limpiando intervalo de polling");
    if (pollingInterval) {
      clearInterval(pollingInterval);
      setPollingInterval(null);
      console.log("✅ Intervalo de polling limpiado");
    }
  };

  // Función para finalizar el pago después de que MP confirme
  const finalizeMPPayment = async (paymentData: any) => {
    try {
      console.log("🔄 Finalizando pago con datos:", paymentData);
      console.log("🔄 Estado de qrData:", qrData);

      if (!user) {
        toast.error("Se perdió la sesión. Por favor inicia sesión nuevamente.");
        if (setQrDialogOpenRef) {
          setQrDialogOpenRef(false);
        }
        resetPaymentState();
        return;
      }

      // Imprimir ticket solo si el pago fue completado
      if (paymentData.isCompleted) {
        try {
          console.log("💰 Pago completado, preparando para crear orden en BD");

          // Primero intentar usar los datos capturados en statusData
          let ticketItems = paymentData.ticketItems;
          let ticketMonto = paymentData.ticketMonto;

          // Si no hay datos capturados, intentar usar qrData como respaldo
          if (!ticketItems && qrData && qrData.items) {
            console.log("⚠️ Usando datos de respaldo de qrData");
            ticketItems = qrData.items;
            ticketMonto = qrData.monto;
          }

          // Verificar que tenemos los datos necesarios
          if (!ticketItems || ticketItems.length === 0) {
            console.error(
              "❌ No se encontraron datos de items para el ticket",
              {
                ticketItems,
                ticketMonto,
                paymentData,
                qrData,
              }
            );

            // Como último recurso, crear un item genérico para imprimir al menos el total
            console.log("⚠️ Creando item genérico para el ticket");
            ticketItems = [
              {
                nombre: "Pago con QR",
                cantidad: 1,
                subtotal: paymentData.total || 0,
                precioHistorico: paymentData.total || 0,
                costo: 0,
              },
            ];
            ticketMonto = paymentData.total || 0;
          }

          // Crear los datos para la orden en BD
          const orderData = {
            metodoPago: "qr",
            total: paymentData.total || ticketMonto,
            items: ticketItems.map((item: any) => ({
              productoId: item.productoId || item.id || 0,
              cantidad: item.cantidad || item.quantity || 1,
              subtotal: Number((item.subtotal || 0).toFixed(2)),
              precioHistorico: item.precioHistorico || item.pricePerUnit || 0,
              costo: Number((item.costo || 0).toFixed(2)),
              nombre: item.nombre || item.name || "Producto",
            })),
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            createdAt: new Date().toISOString(),
            referencia: paymentData.orderId?.toString() || "unknown",
          };

          console.log("💾 Guardando orden en BD:", orderData);

          // Crear la orden en la BD
          const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
            method: "POST",
            headers,
            body: JSON.stringify(orderData),
          });

          if (!orderResponse.ok) {
            console.error(
              "❌ Error al crear la orden en BD:",
              await orderResponse.text()
            );
            throw new Error("Error al crear la orden en base de datos");
          }

          console.log("✅ Orden creada correctamente en BD");

          // Ahora imprimir el ticket
          console.log(
            "🖨️ Imprimiendo ticket con datos:",
            JSON.stringify(orderData)
          );

          const { ipcRenderer } = window.require("electron");
          toast.loading("Imprimiendo ticket...", { id: "print-ticket" });

          try {
            const result = await ipcRenderer.invoke("print-ticket", orderData);
            console.log("🖨️ Resultado de impresión:", result);

            toast.dismiss("print-ticket");
            if (result.success) {
              toast.success("Ticket impreso correctamente");
            } else {
              console.error("❌ Error al imprimir:", result.message);
              toast.error(`Error al imprimir: ${result.message}`);
            }
          } catch (innerError: any) {
            console.error("❌ Error en invoke print-ticket:", innerError);
            toast.dismiss("print-ticket");
            toast.error(`Error al invocar impresión: ${innerError.message}`);
          }

          // Esperar un poco antes de continuar para asegurar que la impresión se complete
          await new Promise((resolve) => setTimeout(resolve, 1000));
        } catch (printError: any) {
          console.error("❌ Error general al procesar/imprimir:", printError);
          toast.error(`Error: ${printError.message}`);
        }
      } else {
        console.log(
          "⚠️ El pago no está marcado como completado, no se creará orden ni imprimirá ticket"
        );
      }

      // Limpiar carrito y estados
      clearCart();
      resetPaymentState();
      toast.success("Pago completado exitosamente");
    } catch (error: any) {
      console.error("❌ Error al finalizar pago:", error);
      toast.error(`Error al finalizar el pago: ${error.message}`);
      // Limpiar estados
      if (setQrDialogOpenRef) {
        setQrDialogOpenRef(false);
      }
      resetPaymentState();
    }
  };

  // Preparar pago mixto
  const handleSplitPayment = () => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso");
      return;
    }

    // Establecer el método seleccionado
    setSelectedPaymentMethod("split");

    // Inicializar el efectivo con un valor vacío
    setCashAmount("");

    // Establecer tarjeta como método secundario por defecto
    setSecondPaymentMethod("tarjeta");

    // Abrir el diálogo de pago mixto
    if (setSplitPaymentDialogOpenRef) {
      setSplitPaymentDialogOpenRef(true);
    }
  };

  // Resetear el estado del procesador de pagos
  const resetPaymentState = () => {
    setSelectedPaymentMethod(null);
    setIsProcessingPayment(false);
    setQrData(null);
    setPaymentStatus(null);
    setCashAmount("");
    setSecondPaymentMethod("tarjeta");
    cleanupPolling();
  };

  // Función para procesar pagos mixtos
  const processSplitPayment = async (items: Product[], totalAmount: number) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Validar que el monto de efectivo sea válido
    const cashAmountValue = parseFloat(cashAmount);
    if (isNaN(cashAmountValue) || cashAmountValue <= 0) {
      toast.error("Ingresa un monto válido para el pago en efectivo");
      return;
    }

    // Validar que el monto de efectivo no sea mayor al total
    if (cashAmountValue > totalAmount) {
      toast.error("El monto en efectivo no puede ser mayor al total");
      return;
    }

    // Calcular el monto restante para el segundo método de pago
    const secondAmount = parseFloat((totalAmount - cashAmountValue).toFixed(2));

    // Validar que el monto restante sea positivo
    if (secondAmount <= 0) {
      toast.error("El monto para el segundo método debe ser mayor a cero");
      return;
    }

    // Si es tarjeta, continuar con el flujo normal
    setIsProcessingPayment(true);

    const orderItems = items.map((item) => ({
      productoId: item.id,
      cantidad: item.quantity,
      subtotal: Number(item.subtotal.toFixed(2)),
      precioHistorico: item.pricePerUnit,
      costo: Number(item.costo),
      nombre: item.name,
    }));

    // Crear la estructura de pagos múltiples siguiendo el formato API
    const orderData = {
      total: Number(totalAmount.toFixed(2)),
      items: orderItems,
      vendedorId: user.id,
      sucursalId: user.sucursalId,
      vendedor: user.nombre,
      createdAt: new Date().toISOString(),
      // Array de pagos con los dos métodos
      pagos: [
        {
          metodoPago: "efectivo",
          monto: cashAmountValue,
        },
        {
          metodoPago: secondPaymentMethod,
          monto: secondAmount,
        },
      ],
    };

    try {
      // Crear la orden
      const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
        body: JSON.stringify(orderData),
      });

      if (!orderResponse.ok) {
        throw new Error("Error al crear la orden");
      }

      // Imprimir ticket usando Electron IPC
      try {
        const { ipcRenderer } = window.require("electron");
        console.log("Enviando datos para impresión:", orderData);
        toast.info("Imprimiendo ticket...", {
          duration: 3000,
          description: "Enviando datos a la impresora",
        });

        const result = await ipcRenderer.invoke("print-ticket", orderData);
        console.log("Resultado de impresión:", result);

        if (result.success) {
          toast.success("Ticket impreso correctamente");
        } else {
          throw new Error(result.message || "Error desconocido al imprimir");
        }
      } catch (printError: any) {
        console.error("Error detallado al imprimir:", printError);
        toast.error(`Error al imprimir el ticket: ${printError.message}`);
      }

      // Limpiar todos los estados relacionados con el pago
      if (setSplitPaymentDialogOpenRef) {
        setSplitPaymentDialogOpenRef(false);
      }
      clearCart();
      resetPaymentState();

      toast.success("Orden completada exitosamente");
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al procesar la orden");
      resetPaymentState();
    }
  };

  return {
    // Estados
    isProcessingPayment,
    selectedPaymentMethod,
    qrData,
    paymentStatus,
    cashAmount,
    secondPaymentMethod,
    originalAmount,
    roundedAmount,
    roundedAmountDialogOpen,
    applyingDiscount,

    // Métodos
    processPayment,
    handleCashPayment,
    generateQRPayment,
    cancelQRPayment,
    handleSplitPayment,
    processSplitPayment,
    resetPaymentState,
    cleanupPolling,

    // Setters
    setCashAmount,
    setSecondPaymentMethod,
    setRoundedAmountDialogOpen,

    // Setters para funciones externas
    set setQrDialogOpen(fn: ((open: boolean) => void) | undefined) {
      console.log("✅ Registrando función setQrDialogOpen");
      setQrDialogOpenRef = fn;
    },
    set setSplitPaymentDialogOpen(fn: ((open: boolean) => void) | undefined) {
      console.log("✅ Registrando función setSplitPaymentDialogOpen");
      setSplitPaymentDialogOpenRef = fn;
    },
    set qrDialogOpen(value: boolean) {
      console.log("✅ Actualizando estado qrDialogOpen a:", value);
      qrDialogOpenRef = value;
    },
  };
}
