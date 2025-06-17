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
  setPaymentDialogOpen?: (open: boolean) => void;
  setQrDialogOpen?: (open: boolean) => void;
  setSplitPaymentDialogOpen?: (open: boolean) => void;
}

// Declara la interface para las funciones y estados externos que se inyectarán
interface PaymentState {
  setQrDialogOpen?: (open: boolean) => void;
  setSplitPaymentDialogOpen?: (open: boolean) => void;
  qrDialogOpen?: boolean;
}

// Nuevas interfaces para el sistema de QR manual
interface ManualQrDetails {
  orderId: number;
  isSplitPayment: boolean;
  cashAmount?: number;
}

export function usePaymentProcessing({
  user,
  API_URL,
  appId,
  clearCart,
  calculateTotal,
  setPaymentDialogOpen,
  setQrDialogOpen,
  setSplitPaymentDialogOpen,
}: PaymentOptions) {
  // Helper para llamadas a Electron IPC
  const getElectronAPI = () => {
    if (typeof window !== "undefined" && window.require) {
      return window.require("electron");
    }
    return null;
  };
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

  // Nuevos estados para mejoras en el sistema
  const [retryCount, setRetryCount] = useState(0);
  const [pollingStartTime, setPollingStartTime] = useState<number | null>(null);
  const [manualQrPasswordDialogOpen, setManualQrPasswordDialogOpen] =
    useState(false);
  const [manualQrPassword, setManualQrPassword] = useState("");
  const [manualQrOrderDetails, setManualQrOrderDetails] =
    useState<ManualQrDetails | null>(null);

  // Referencias para controles externos - Usar las funciones pasadas como parámetros
  const setQrDialogOpenRef = setQrDialogOpen;
  const setSplitPaymentDialogOpenRef = setSplitPaymentDialogOpen;
  let qrDialogOpenRef: boolean = false;

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // Función para redondear a los 50 pesos más cercanos hacia abajo
  const roundToNearest50 = (amount: number): number => {
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
    console.log("🎯 PAYMENT PROCESSOR: processPayment iniciado");
    console.log("🎯 Parámetros:", {
      method,
      finalTotal,
      itemsCount: items.length,
      user: user?.id,
      isProcessingPayment,
    });

    if (!user) {
      console.log("❌ PAYMENT PROCESSOR: Usuario no logueado");
      toast.error("Debes iniciar sesión para realizar una orden");
      resetPaymentState();
      return;
    }

    if (isProcessingPayment) {
      console.log("❌ PAYMENT PROCESSOR: Ya hay un pago en proceso");
      return;
    }

    console.log(
      "✅ PAYMENT PROCESSOR: Validaciones pasadas, iniciando procesamiento"
    );

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

      // Mostrar toast de carga para la impresión ANTES de imprimir
      const printingToastId = toast.loading("Imprimiendo ticket...");

      // Imprimir ticket usando Electron IPC
      const printSuccess = await handleTicketPrinting(orderData);

      // Cerrar el toast de carga de impresión
      toast.dismiss(printingToastId);

      // Mostrar toast de error si la impresión falló (handleTicketPrinting ya muestra éxito)
      if (!printSuccess) {
        toast.error("Error al imprimir el ticket.");
      }

      // Primero mostrar el toast de éxito
      toast.success("Orden completada exitosamente");

      // Esperar un poco para que el usuario vea el éxito antes de cerrar
      await new Promise((resolve) => setTimeout(resolve, 1500));

      // Limpiar carrito y estados
      clearCart();
      resetPaymentState();

      // Cerrar el diálogo del efectivo si está abierto
      setRoundedAmountDialogOpen(false);

      // Cerrar el diálogo de pago principal
      if (setPaymentDialogOpen) {
        setPaymentDialogOpen(false);
      }
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al procesar la orden");

      // En caso de error, también cerrar los diálogos y resetear estados
      setRoundedAmountDialogOpen(false);
      resetPaymentState();
      if (setPaymentDialogOpen) {
        setPaymentDialogOpen(false);
      }
    }
  };

  // Función para manejar un pago en efectivo con redondeo
  const handleCashPayment = (businessInfo: any, withDiscount = false) => {
    console.log(
      "🛒 EFECTIVO: Iniciando proceso de pago en efectivo",
      withDiscount ? "con descuento" : ""
    );

    console.log("🏢 businessInfo recibido:", {
      businessInfo,
      sistemaPago: businessInfo?.sistemaPago,
      descuentoEfectivo: businessInfo?.descuentoEfectivo,
    });

    // Establecer el estado de descuento
    setApplyingDiscount(withDiscount);

    // Calcular los importes para cualquier caso
    const originalTotal = Number(calculateTotal().toFixed(2));
    let finalTotal = originalTotal;

    console.log("💰 Total original calculado:", originalTotal);

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

    // Para pagos en efectivo, siempre aplicar redondeo a múltiplos de 50
    let roundedTotal = finalTotal;
    // Aplicar redondeo en efectivo independientemente de la configuración sistemaPago
    roundedTotal = roundToNearest50(finalTotal);
    console.log("🧮 EFECTIVO: Cálculos de redondeo:", {
      finalTotal,
      roundedTotal,
      diferencia: finalTotal - roundedTotal,
      sistemaRedondeo: businessInfo?.sistemaPago,
      redondeoAplicado: true,
    });

    // Guardar los montos calculados en el estado
    setOriginalAmount(originalTotal);
    setRoundedAmount(roundedTotal);

    console.log("💾 Valores guardados en estado:", {
      originalAmount: originalTotal,
      roundedAmount: roundedTotal,
      diferencia: originalTotal - roundedTotal,
    });

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
    const POLLING_INTERVAL = 3000;
    const MAX_POLLING_TIME = 10 * 60 * 1000;
    const MAX_RETRIES = 3;

    setPollingStartTime(Date.now());
    setRetryCount(0);

    const interval = setInterval(async () => {
      try {
        if (
          pollingStartTime &&
          Date.now() - pollingStartTime > MAX_POLLING_TIME
        ) {
          cleanupPolling();
          toast.error("Tiempo de espera agotado. El código QR ha expirado.");
          if (setQrDialogOpenRef) {
            setQrDialogOpenRef(false);
          }
          resetPaymentState();
          return;
        }

        console.log("🔄 Verificando estado del pago...");
        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status?orderId=${orderId}`,
          { headers }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        setRetryCount(0);
        setPaymentStatus(statusData.status);

        if (statusData.isCompleted || statusData.isCancelled) {
          cleanupPolling();

          if (statusData.isCompleted) {
            const cartData = {
              items: qrData.items,
              total: qrData.monto,
            };

            await finalizeMPPayment({
              ...statusData,
              cartData,
            });

            toast.success("¡Pago completado! Cerrando en 2 segundos...");
            setTimeout(() => {
              if (setQrDialogOpenRef) {
                setQrDialogOpenRef(false);
              }
              resetPaymentState();
              clearCart();
            }, 2000);
          } else {
            console.log("❌ Pago cancelado o rechazado");
            toast.error("El pago ha sido cancelado o rechazado");
            if (setQrDialogOpenRef) {
              setQrDialogOpenRef(false);
            }
            resetPaymentState();
          }
        }
      } catch (error: any) {
        console.error("❌ Error al verificar estado:", error);
        setRetryCount((prev) => prev + 1);

        if (retryCount >= MAX_RETRIES) {
          cleanupPolling();
          toast.error(
            "Error al verificar el estado del pago. Por favor, verifique manualmente."
          );
          if (setQrDialogOpenRef) {
            setQrDialogOpenRef(false);
          }
          resetPaymentState();
        }
      }
    }, POLLING_INTERVAL);

    setPollingInterval(interval);
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
  const finalizeMPPayment = async (
    paymentData: any,
    skipPrinting: boolean = false
  ) => {
    try {
      console.log("🔄 Finalizando pago con datos:", paymentData);

      if (!user) {
        toast.error("Se perdió la sesión. Por favor inicia sesión nuevamente.");
        if (setQrDialogOpenRef) {
          setQrDialogOpenRef(false);
        }
        resetPaymentState();
        return;
      }

      if (paymentData.isCompleted && paymentData.cartData) {
        try {
          console.log("💰 Pago completado, preparando para crear orden en BD");

          const orderItems = paymentData.cartData.items.map((item: any) => ({
            productoId: item.id,
            cantidad: item.quantity,
            subtotal: Number(item.subtotal.toFixed(2)),
            precioHistorico: item.pricePerUnit,
            costo: Number(item.costo.toFixed(2)),
            nombre: item.name,
          }));

          const orderData = {
            metodoPago: "qr",
            total: paymentData.cartData.total,
            items: orderItems,
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            createdAt: new Date().toISOString(),
            referencia: paymentData.orderId?.toString() || "unknown",
          };

          console.log("💾 Guardando orden en BD:", orderData);

          const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
            body: JSON.stringify(orderData),
          });

          if (!orderResponse.ok) {
            throw new Error("Error al crear la orden en base de datos");
          }

          // Solo imprimir el ticket si no se indica saltar la impresión
          if (!skipPrinting) {
            await handleTicketPrinting(orderData);
          } else {
            console.log("🖨️ Impresión de ticket omitida (skipPrinting=true)");
          }
        } catch (error: any) {
          console.error("❌ Error al procesar orden:", error);
          toast.error(`Error: ${error.message}`);
        }
      }
    } catch (error: any) {
      console.error("❌ Error al finalizar pago:", error);
      toast.error(`Error al finalizar el pago: ${error.message}`);
    }
  };

  // Preparar pago mixto
  const handleSplitPayment = () => {
    console.log("🔄 handleSplitPayment llamado");

    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso");
      return;
    }

    console.log("🔄 Configurando pago mixto...");

    // Establecer el método seleccionado
    setSelectedPaymentMethod("split");

    // Inicializar el efectivo con un valor vacío
    setCashAmount("");

    // Establecer tarjeta como método secundario por defecto
    setSecondPaymentMethod("tarjeta");

    // Abrir el diálogo de pago mixto
    console.log("🔄 Intentando abrir diálogo de pago mixto...");
    console.log(
      "🔄 setSplitPaymentDialogOpenRef:",
      setSplitPaymentDialogOpenRef
    );

    if (setSplitPaymentDialogOpenRef) {
      console.log("✅ Abriendo diálogo de pago mixto");
      setSplitPaymentDialogOpenRef(true);
    } else {
      console.error("❌ setSplitPaymentDialogOpenRef no está definido");
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
    setRetryCount(0);
    setPollingStartTime(null);
    cleanupPolling();
  };

  // Función para procesar pagos mixtos
  const processSplitPayment = async (
    items: Product[],
    totalAmount: number,
    businessInfo?: any
  ) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir múltiples procesamiento - verificar ANTES de cualquier acción
    if (isProcessingPayment) {
      console.log("⚠️ Procesamiento bloqueado - ya está procesando pago mixto");
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

    // Marcar como procesando ANTES de cualquier operación asíncrona
    setIsProcessingPayment(true);

    try {
      // Si el segundo método es QR, generar QR para el split payment
      if (secondPaymentMethod === "qr") {
        await generateSplitQRPayment(
          cashAmountValue,
          secondAmount,
          items,
          businessInfo
        );
        return;
      }

      // Si es tarjeta, continuar con el flujo normal
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
        const electronAPI = getElectronAPI();
        if (electronAPI) {
          const { ipcRenderer } = electronAPI;
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
        } else {
          console.log("🌐 Modo desarrollo: simulando impresión");
          toast.success("Ticket simulado (modo desarrollo)");
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
    } finally {
      // Asegurar que el estado se resetee SIEMPRE
      setIsProcessingPayment(false);
    }
  };

  // Función para generar QR en pago mixto
  const generateSplitQRPayment = async (
    cashAmountValue: number,
    qrAmount: number,
    items: Product[],
    businessInfo?: any
  ) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    console.log("🔄 Generando QR para pago mixto:", {
      cashAmount: cashAmountValue,
      qrAmount,
      businessInfo: businessInfo?.mpEnabled,
    });

    // Verificar si MP está habilitado
    if (businessInfo?.mpEnabled === false) {
      console.log(
        "🔄 MP deshabilitado - procesando pago mixto como transferencia directa"
      );

      // Preparar datos de la orden con pagos mixtos
      const orderItems = items.map((item) => ({
        productoId: item.id,
        cantidad: item.quantity,
        subtotal: Number(item.subtotal.toFixed(2)),
        precioHistorico: item.pricePerUnit,
        costo: Number(item.costo),
        nombre: item.name,
      }));

      const orderData = {
        total: Number((cashAmountValue + qrAmount).toFixed(2)),
        items: orderItems,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        vendedor: user.nombre,
        estado: "COMPLETADA",
        createdAt: new Date().toISOString(),
        // Array de pagos con los dos métodos
        pagos: [
          {
            metodoPago: "efectivo",
            monto: cashAmountValue,
          },
          {
            metodoPago: "qr",
            monto: qrAmount,
          },
        ],
      };

      try {
        // Mostrar toast de carga ANTES de la llamada a la API
        const processingToastId = toast.loading("Procesando orden mixta...");

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
          const errorData = await orderResponse.json().catch(() => ({}));
          console.error("❌ Error al crear la orden mixta:", errorData);
          throw new Error(errorData.message || "Error al crear la orden");
        }

        // Ocultar toast de carga y mostrar toast de impresión
        toast.dismiss(processingToastId);

        // Mostrar toast de carga para la impresión ANTES de imprimir
        const printingToastId = toast.loading("Imprimiendo ticket...");

        // Imprimir ticket
        await handleTicketPrinting(orderData);

        // Cerrar el toast de carga de impresión
        toast.dismiss(printingToastId);

        // Limpiar carrito y estados
        clearCart();
        resetPaymentState();

        // Cerrar el diálogo de pago mixto
        if (setSplitPaymentDialogOpenRef) {
          setSplitPaymentDialogOpenRef(false);
        }

        toast.success("Orden mixta completada exitosamente");
      } catch (error: any) {
        console.error("❌ Error en flujo pago mixto/MP deshabilitado:", error);
        toast.error(`Error al procesar la orden: ${error.message}`);
        resetPaymentState();
      }

      return;
    }

    // Si MP está habilitado, generar el QR para el pago mixto
    setIsProcessingPayment(true);
    setSelectedPaymentMethod("split");

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
        monto: qrAmount,
        descripcion: `Pago mixto: QR $${qrAmount} + Efectivo $${cashAmountValue}`,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        externalPosId: import.meta.env.VITE_POS_ID,
        items: orderItems,
        isSplitPayment: true,
        cashAmount: cashAmountValue,
      };

      console.log(
        "🔄 Enviando solicitud para generar QR de pago mixto:",
        orderData
      );

      // Mostrar cargando
      toast.loading("Generando código QR para pago mixto...", {
        id: "qr-loading",
      });

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
        throw new Error("Error al generar el código QR para pago mixto");
      }

      const data = await response.json();
      console.log("✅ QR de pago mixto generado:", data);

      // Generar QR localmente
      let qrImageDataUrl;
      try {
        qrImageDataUrl = await QRCode.toDataURL(data.qrData);
      } catch (qrError) {
        console.error("❌ Error al generar imagen QR:", qrError);
        throw new Error("Error al generar la imagen del código QR");
      }

      // Guardar datos del QR con información del pago mixto
      setQrData({
        ...data,
        qrImageUrl: qrImageDataUrl,
        isSplitPayment: true,
        cashAmount: cashAmountValue,
        items: orderItems,
      });

      // Ocultar toast de carga
      toast.dismiss("qr-loading");

      // Cerrar diálogo de pago mixto y abrir diálogo de QR
      if (setSplitPaymentDialogOpenRef) {
        setSplitPaymentDialogOpenRef(false);
      }

      if (setQrDialogOpenRef) {
        setQrDialogOpenRef(true);
      }

      // Iniciar polling para verificar el estado del pago
      startSplitPaymentStatusPolling(data.orderId, cashAmountValue);
    } catch (error: any) {
      console.error("❌ Error al generar QR de pago mixto:", error);
      toast.dismiss("qr-loading");
      toast.error(`Error: ${error.message}`);
      resetPaymentState();
    }
  };

  // Función para verificar el estado del pago mixto con QR
  const startSplitPaymentStatusPolling = (
    orderId: number,
    cashAmount: number
  ) => {
    console.log(
      "🔄 Iniciando polling para verificar estado del pago mixto:",
      orderId
    );

    setPaymentStatus("PENDIENTE");

    // Limpiar cualquier intervalo existente
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }

    const POLLING_INTERVAL = 3000; // 3 segundos
    const MAX_RETRIES = 20; // 1 minuto de intentos

    // Crear intervalo de polling
    const interval = setInterval(async () => {
      try {
        console.log("🔄 Verificando estado del pago mixto...");
        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status?orderId=${orderId}`,
          { headers }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        console.log("🔄 Estado actual del pago mixto:", statusData);

        setPaymentStatus(statusData.status);

        // Si el pago se completó o canceló, detener el polling
        if (statusData.isCompleted || statusData.isCancelled) {
          clearInterval(interval);
          setPollingInterval(null);

          if (statusData.isCompleted) {
            console.log("✅ Pago mixto QR completado exitosamente");
            await finalizeSplitMPPayment(statusData, cashAmount);

            toast.success("¡Pago mixto completado! Cerrando en 2 segundos...");
            setTimeout(() => {
              if (setQrDialogOpenRef) {
                setQrDialogOpenRef(false);
              }
              resetPaymentState();
              clearCart();
            }, 2000);
          } else {
            console.log("❌ Pago mixto cancelado o rechazado");
            toast.error("El pago mixto ha sido cancelado o rechazado");
            if (setQrDialogOpenRef) {
              setQrDialogOpenRef(false);
            }
            resetPaymentState();
          }
        }
      } catch (error: any) {
        console.error("❌ Error al verificar estado del pago mixto:", error);
        setRetryCount((prev) => prev + 1);

        if (retryCount >= MAX_RETRIES) {
          clearInterval(interval);
          setPollingInterval(null);
          toast.error(
            "Error al verificar el estado del pago mixto. Por favor, verifique manualmente."
          );
          if (setQrDialogOpenRef) {
            setQrDialogOpenRef(false);
          }
          resetPaymentState();
        }
      }
    }, POLLING_INTERVAL);

    setPollingInterval(interval);
  };

  // Función para completar orden manualmente
  const completarOrdenManualmente = async (
    orderId: number,
    isSplitPayment: boolean = false,
    cashAmount?: number
  ) => {
    setManualQrOrderDetails({ orderId, isSplitPayment, cashAmount });
    setManualQrPasswordDialogOpen(true);
  };

  // Función para manejar el envío de contraseña QR manual
  const handleManualQrPasswordSubmit = async () => {
    if (!manualQrOrderDetails) return;

    const { orderId, isSplitPayment, cashAmount } = manualQrOrderDetails;
    const enteredPassword = manualQrPassword;
    setManualQrPassword("");

    if (enteredPassword !== import.meta.env.VITE_MANUAL_QR_PASSWORD) {
      toast.error("Contraseña incorrecta");
      return;
    }

    setManualQrPasswordDialogOpen(false);

    try {
      console.log("🔄 Intentando completar orden manualmente con contraseña:", {
        orderId,
        isSplitPayment,
        cashAmount,
      });
      const processingToastId = toast.loading("Procesando orden manual...");

      const response = await fetch(
        `${API_URL}/api/mercadopago/manual-complete`,
        {
          method: "POST",
          headers,
          body: JSON.stringify({ orderId }),
        }
      );

      if (!response.ok) {
        const error = await response.json();
        toast.dismiss(processingToastId);
        throw new Error(error.message || "Error al completar la orden");
      }

      const result = await response.json();
      console.log("✅ Orden completada manualmente:", result);
      toast.dismiss(processingToastId);

      if (isSplitPayment && typeof cashAmount === "number") {
        await finalizeSplitMPPayment(
          {
            isCompleted: true,
            orderId: result.orderId,
          },
          cashAmount
        );
      } else {
        const cartData = {
          items: qrData.items,
          total: qrData.monto,
        };

        await finalizeMPPayment(
          {
            isCompleted: true,
            orderId: result.orderId,
            cartData,
          },
          true
        );
      }

      toast.success("Orden completada. Reiniciando carrito...");

      setTimeout(() => {
        if (pollingInterval) {
          clearInterval(pollingInterval);
          setPollingInterval(null);
        }

        if (setQrDialogOpenRef) {
          setQrDialogOpenRef(false);
        }
        resetPaymentState();
        clearCart();
      }, 2000);
    } catch (error: any) {
      console.error("❌ Error al completar manualmente:", error);
      toast.error(`Error al completar la orden manualmente: ${error.message}`);
    } finally {
      setManualQrOrderDetails(null);
    }
  };

  // Función para formatear fechas en zona horaria Argentina
  const formatFechaArgentina = (fecha: string | Date) => {
    const fechaObj = typeof fecha === "string" ? new Date(fecha) : fecha;
    const fechaArg = new Date(fechaObj.getTime() + 3 * 60 * 60 * 1000);
    return fechaArg.toLocaleString("es-AR", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  };

  // Función para manejar la impresión de tickets
  const handleTicketPrinting = async (orderData: any): Promise<boolean> => {
    try {
      console.log("====== SIMULACIÓN DEL TICKET ======");
      console.log("ISELIN II");
      console.log(`Vendedor: ${orderData.vendedor}`);
      console.log(
        `Fecha: ${formatFechaArgentina(orderData.createdAt || orderData.fecha)}`
      );
      console.log("-----------------------------");
      console.log("PRODUCTO      CANT    PRECIO    TOTAL");
      console.log("-----------------------------");

      const items = orderData.items || orderData.detalles || [];
      if (items && items.length > 0) {
        items.forEach((item: any) => {
          const nombre = (item.nombre || item.producto?.nombre || "").padEnd(
            12
          );
          const cantidad = (item.cantidad || 0).toString().padStart(8);
          const precio = `$${Number(
            item.precioHistorico || item.precio || 0
          ).toFixed(2)}`.padStart(8);
          const subtotal = `$${Number(item.subtotal || 0).toFixed(2)}`.padStart(
            8
          );
          console.log(`${nombre} ${cantidad} ${precio} ${subtotal}`);
        });
      } else {
        console.log("❌ No hay items en la orden");
      }

      console.log("-----------------------------");
      console.log(`TOTAL: $${Number(orderData.total).toFixed(2)}`);

      if (orderData.pagos && Array.isArray(orderData.pagos)) {
        console.log("MÉTODOS DE PAGO:");
        orderData.pagos.forEach((pago: any) => {
          console.log(
            `${pago.metodoPago.toUpperCase()}: $${Number(pago.monto).toFixed(
              2
            )}`
          );
        });
      } else {
        console.log(`Método de pago: ${orderData.metodoPago?.toUpperCase()}`);
      }

      console.log("¡Gracias por su compra!");
      console.log("==============================");

      // Intentar imprimir
      const electronAPI = getElectronAPI();
      if (electronAPI) {
        const { ipcRenderer } = electronAPI;
        console.log("Enviando datos para impresión:", orderData);
        // No mostrar toast de carga aquí, se hará antes de llamar a esta función

        const result = await ipcRenderer.invoke("print-ticket", orderData);
        console.log("Resultado de impresión:", result);

        if (result.success) {
          toast.success("Ticket impreso correctamente");
          return true;
        } else {
          // No mostrar toast de error aquí, se manejará en la función que llama
          console.error(
            "❌ Error al imprimir (IPC invoke returned false):",
            result.message
          );
          return false;
        }
      } else {
        console.log("🌐 Modo desarrollo: simulando impresión de ticket");
        toast.success("Ticket simulado (modo desarrollo)");
        return true;
      }
    } catch (error: any) {
      console.error("❌ Error al imprimir (catch):", error);
      // No mostrar toast de error aquí, se manejará en la función que llama
      return false;
    }
  };

  // Función para finalizar el pago mixto después de que QR sea confirmado
  const finalizeSplitMPPayment = async (
    paymentData: any,
    cashAmount: number
  ) => {
    try {
      console.log("🔄 Finalizando pago mixto con datos:", paymentData);

      if (!user) {
        toast.error(
          "Sesión no disponible. Por favor inicia sesión nuevamente."
        );
        return;
      }

      if (paymentData.isCompleted) {
        try {
          const orderItems = qrData.items.map((item: any) => ({
            productoId: item.id,
            cantidad: item.quantity,
            subtotal: Number(item.subtotal.toFixed(2)),
            precioHistorico: item.pricePerUnit,
            costo: Number(item.costo.toFixed(2)),
            nombre: item.name,
          }));

          const totalAmount = qrData.monto + cashAmount;

          const orderData = {
            total: totalAmount,
            items: orderItems,
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            createdAt: new Date().toISOString(),
            pagos: [
              {
                metodoPago: "efectivo",
                monto: cashAmount,
              },
              {
                metodoPago: "qr",
                monto: qrData.monto,
                referencia: paymentData.orderId?.toString() || "unknown",
              },
            ],
          };

          console.log("💾 Guardando orden mixta en BD:", orderData);

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

          await handleTicketPrinting(orderData);
        } catch (error: any) {
          console.error("❌ Error al procesar orden mixta:", error);
          toast.error(`Error: ${error.message}`);
        }
      }
    } catch (error: any) {
      console.error("❌ Error al finalizar pago mixto:", error);
      toast.error(`Error al procesar el pago mixto: ${error.message}`);
    }
  };

  useEffect(() => {
    return () => {
      cleanupPolling();
    };
  }, []);

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
    manualQrPasswordDialogOpen,
    manualQrPassword,

    // Métodos
    processPayment,
    handleCashPayment,
    generateQRPayment,
    generateSplitQRPayment,
    cancelQRPayment,
    handleSplitPayment,
    processSplitPayment,
    resetPaymentState,
    cleanupPolling,
    completarOrdenManualmente,
    handleManualQrPasswordSubmit,
    handleTicketPrinting,

    // Setters
    setCashAmount,
    setSecondPaymentMethod,
    setRoundedAmountDialogOpen,
    setManualQrPasswordDialogOpen,
    setManualQrPassword,
  };
}
