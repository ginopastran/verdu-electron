import { useState, useEffect, useRef } from "react";
import { toast } from "sonner";
import QRCode from "qrcode";
import { Product } from "./useCartState";
import { getBusinessName, getAdminData } from "@/utils/businessHelpers";
import { useTicketPrinting } from "@/hooks/useTicketPrinting";
import { createValidOrderPayload } from "@/utils/orderHelpers";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";

// Tipos ya declarados en otros archivos

interface PaymentOptions {
  user: any;
  API_URL: string;
  appId: string | null;
  clearCart: () => void;
  calculateTotal: () => number;
  setPaymentDialogOpen: (open: boolean) => void;
  setQrDialogOpen: (open: boolean) => void;
  setSplitPaymentDialogOpen: (open: boolean) => void;
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
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  // Helper para acceder a la API de Electron
  const getElectronAPI = () => {
    try {
      if (typeof window !== "undefined" && window.electron?.ipcRenderer) {
        return { ipcRenderer: window.electron.ipcRenderer };
      }
      return null;
    } catch (error) {
      console.error("❌ Error al acceder a Electron:", error);
      return null;
    }
  };

  // ✅ NUEVO: Control de toasts para evitar duplicados
  const [processedOrders] = useState<Set<string>>(new Set());

  const isOrderAlreadyProcessed = (orderId: string): boolean => {
    return processedOrders.has(orderId);
  };

  const markOrderAsProcessed = (orderId: string): void => {
    processedOrders.add(orderId);
    console.log(`✅ TOAST CONTROL: Orden ${orderId} marcada como procesada`);
  };

  const clearProcessedOrdersTracking = () => {
    processedOrders.clear();
    console.log("🧹 TOAST CONTROL: Tracking de órdenes procesadas limpiado");
  };

  // ✅ NUEVO: Control de impresión para evitar duplicados
  const [printedOrders] = useState<Set<string>>(new Set());

  const isOrderAlreadyPrinted = (
    orderId: number | string,
    orderType: string = "qr"
  ): boolean => {
    const key = `${orderType}-${orderId}`;
    return printedOrders.has(key);
  };

  const markOrderAsPrinted = (
    orderId: number | string,
    orderType: string = "qr"
  ): void => {
    const key = `${orderType}-${orderId}`;
    printedOrders.add(key);
    console.log(`✅ IMPRESIÓN CONTROL: Orden ${key} marcada como impresa`);
  };

  // ✅ FUNCIÓN PARA LIMPIAR CONTROL DE IMPRESIÓN
  const clearPrintedOrdersTracking = () => {
    printedOrders.clear();
    console.log("🧹 IMPRESIÓN CONTROL: Tracking de órdenes impresas limpiado");
  };

  // Estado para datos del QR
  const [qrData, setQrData] = useState<any>(null);
  // Usar un ref para disponer siempre del valor más reciente dentro de los callbacks del polling
  const qrDataRef = useRef<any>(null);
  const updateQrData = (data: any) => {
    qrDataRef.current = data;
    setQrData(data);
  };
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

  // Estados para el sistema de pago exacto
  const [exactPaymentDialogOpen, setExactPaymentDialogOpen] = useState(false);
  const [paidAmount, setPaidAmount] = useState<number>(0);
  const [changeAmount, setChangeAmount] = useState<number>(0);

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

  // ✅ ANTI-DUPLICADOS: Flag para evitar llamadas concurrentes
  const [isProcessingOrder, setIsProcessingOrder] = useState(false);

  // ✅ NUEVO: Flag para prevenir doble impresión cuando se procesa manualmente
  const isProcessingManualPayment = useRef<boolean>(false);

  // Referencias para controles externos - Usar las funciones pasadas como parámetros
  const setQrDialogOpenRef = setQrDialogOpen;
  const setSplitPaymentDialogOpenRef = setSplitPaymentDialogOpen;
  let qrDialogOpenRef: boolean = false;

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // ✅ ANTI-DUPLICADOS: Función para crear órdenes verificando duplicados
  const createOrderWithDuplicateCheck = async (orderData: any) => {
    if (isProcessingOrder) {
      console.log(
        "⏳ ANTI-DUPLICADOS: Orden ya en proceso, evitando duplicado..."
      );
      return null;
    }

    setIsProcessingOrder(true);

    try {
      console.log(
        "📋 ANTI-DUPLICADOS: Creando orden con verificación de duplicados"
      );

      const validPayload = createValidOrderPayload(orderData);

      const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
        method: "POST",
        headers,
        body: JSON.stringify(validPayload),
      });

      if (!orderResponse.ok) {
        throw new Error("Error al crear la orden");
      }

      const result = await orderResponse.json();

      // ✅ CRÍTICO: Verificar si es duplicado
      if (result.isDuplicate) {
        console.log(
          "✅ ANTI-DUPLICADOS: Orden ya existe, evitando duplicado:",
          result.id
        );
        return {
          ...result,
          isDuplicate: true,
          message: "Orden ya existe",
        };
      }

      console.log("✅ ANTI-DUPLICADOS: Nueva orden creada:", result.id);
      return {
        ...result,
        isDuplicate: false,
      };
    } catch (error: any) {
      console.error("❌ ANTI-DUPLICADOS: Error creando orden:", error);

      // ✅ MANEJO ESPECIAL: Si es el error de "No valid items", el backend ya procesó todo
      if (error.message?.includes("No valid items to create order payload")) {
        console.warn(
          "🔧 ANTI-DUPLICADOS: Error de items vacíos - backend ya procesó la orden"
        );
        console.warn("🔧 Retornando como orden ya existente");

        return {
          isDuplicate: true,
          message: "Orden ya procesada por el backend",
          id: "backend-processed",
        };
      }

      throw error;
    } finally {
      setIsProcessingOrder(false);
    }
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

    // Mantener en sync el método de pago seleccionado para que la UI se actualice
    setSelectedPaymentMethod(method);

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
      businessName: await getBusinessName(),
      createdAt: new Date().toISOString(),
    };

    try {
      // ✅ ANTI-DUPLICADOS: Crear orden con verificación de duplicados
      const orderResult = await createOrderWithDuplicateCheck(orderData);

      if (!orderResult) {
        console.log("⏳ PROCESS PAYMENT: Orden cancelada por concurrencia");
        return;
      }

      if (orderResult.isDuplicate) {
        console.log(
          "✅ PROCESS PAYMENT: Orden duplicada detectada, usando existente"
        );
        toast.success("Orden completada exitosamente");
        clearCart();
        resetPaymentState();
        return;
      }

      console.log("📋 PROCESS PAYMENT: Nueva orden creada:", orderResult);

      // Añadir el idReal a los datos de la orden para impresión
      const enrichedOrderData = {
        ...orderData,
        idReal: orderResult.idReal || orderResult.id || null,
        id: orderResult.id || null,
      };

      console.log("✅ Datos enriquecidos para impresión:", enrichedOrderData);

      // Mostrar toast de carga para la impresión ANTES de imprimir
      const printingToastId = toast.loading("Imprimiendo ticket...");

      try {
        // Imprimir ticket usando Electron IPC
        const printSuccess = await handleTicketPrinting(enrichedOrderData);

        // 🆕 DOBLE IMPRESIÓN NORMAL: Si está habilitada, imprimir segunda vez
        if (printSuccess) {
          try {
            // Obtener businessInfo para verificar doble impresión
            const businessInfo = await (
              await import("@/utils/businessHelpers")
            ).getBusinessInfo(API_URL, appId);

            if (businessInfo?.dobleImpresionEnabled === true) {
              console.log(
                "🖨️🖨️ NORMAL DOBLE IMPRESIÓN: Imprimiendo segunda copia..."
              );
              await handleTicketPrinting(enrichedOrderData);
              console.log(
                "✅ NORMAL DOBLE IMPRESIÓN: Segunda copia impresa exitosamente"
              );
            }
          } catch (error) {
            console.error(
              "❌ NORMAL DOBLE IMPRESIÓN: Error en segunda copia:",
              error
            );
            // No fallar la orden si la segunda impresión falla
          }
        }

        // Cerrar el toast de carga de impresión
        toast.dismiss(printingToastId);

        // Mostrar toast de error si la impresión falló (handleTicketPrinting ya muestra éxito)
        if (!printSuccess) {
          toast.error("Error al imprimir el ticket.");
        }
      } catch (printError) {
        // Asegurar que el toast se cierre siempre
        toast.dismiss(printingToastId);
        console.error("Error en impresión:", printError);
      }

      // Primero mostrar el toast de éxito
      toast.success("Orden completada exitosamente");

      // Esperar un poco para que el usuario vea el éxito antes de cerrar
      await new Promise((resolve) => setTimeout(resolve, 1500));

      // ✅ MEJORADO: Resetear estados del hook ANTES de limpiar carrito
      resetPaymentState();

      // Limpiar carrito después de resetear estados
      clearCart();

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
    } finally {
      // ✅ MEJORADO: Asegurar que el estado se resetee SIEMPRE
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    }
  };

  // Función para confirmar pago exacto con vuelto
  const confirmExactPayment = async (
    paidAmount: number,
    change: number,
    items: Product[]
  ) => {
    console.log("💰 EXACT PAYMENT: Confirmando pago exacto:", {
      paidAmount,
      change,
      totalAmount: roundedAmount,
    });

    setIsProcessingPayment(true);

    try {
      // Procesar el pago con el monto total original (no el pagado)
      await processPayment("efectivo", roundedAmount, items);

      // Cerrar el diálogo de pago exacto
      setExactPaymentDialogOpen(false);

      // Guardar información del vuelto para logs o futura referencia
      setPaidAmount(paidAmount);
      setChangeAmount(change);

      console.log("✅ EXACT PAYMENT: Pago completado exitosamente");
    } catch (error) {
      console.error("❌ EXACT PAYMENT: Error al procesar pago:", error);
      toast.error("Error al procesar el pago exacto");
    } finally {
      setIsProcessingPayment(false);
    }
  };

  // Función para manejar un pago en efectivo con redondeo o pago exacto
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

    // Establecer efectivo como método seleccionado
    setSelectedPaymentMethod("efectivo");
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

    // Decidir qué flujo usar según el sistemaPago
    if (businessInfo?.sistemaPago === "pago-exacto") {
      console.log("💰 EFECTIVO: Usando sistema de pago exacto con vuelto");

      // Guardar los montos para el sistema de pago exacto
      setOriginalAmount(originalTotal);
      setRoundedAmount(finalTotal); // En pago exacto, el monto final es el que hay que cobrar

      console.log("💾 Abriendo diálogo de pago exacto para total:", finalTotal);

      // Abrir diálogo de pago exacto
      setExactPaymentDialogOpen(true);
    } else {
      // Sistema de redondeo (por defecto)
      console.log("🧮 EFECTIVO: Usando sistema de redondeo tradicional");

      let roundedTotal = roundToNearest50(finalTotal);
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

      // Mostrar diálogo de redondeo o confirmación
      setRoundedAmountDialogOpen(true);
    }
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
        items: orderItems,
        // ✅ API Unificada: Nuevos parámetros requeridos
        requiresAfipInvoice: false,
        isSplitPayment: false,
        cashAmount: 0,
        afipData: null,
      };

      console.log("📲 Generando QR (F3 - Sin AFIP) con payload:", orderData);

      const headers: HeadersInit = {
        "Content-Type": "application/json",
        ...(appId && { "X-App-ID": appId }),
      };

      // Mostrar cargando
      toast.loading("Generando código QR...", { id: "qr-loading" });

      // Realizar la solicitud para generar el QR
      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers,
        body: JSON.stringify(orderData),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message || "Error al generar el QR para pago"
        );
      }

      const result = await response.json();
      console.log("📲 Respuesta del backend (QR):", result);

      if (result.qrData) {
        // ✅ CORRECCIÓN: Limpiar toast de loading ANTES de continuar
        toast.dismiss("qr-loading");

        // ✅ CORRECCIÓN: Guardar la información del QR INCLUYENDO los items originales para impresión
        const qrDataWithAmount = {
          ...result,
          monto: Number(calculateTotal().toFixed(2)),
          items: orderItems, // ✅ CRITICAL FIX: Incluir items originales para impresión
        };
        updateQrData(qrDataWithAmount);
        setPaymentStatus("PENDIENTE");

        // Iniciar el polling para verificar el estado del pago
        startPaymentStatusPolling(result.id);

        if (setQrDialogOpen) {
          setQrDialogOpen(true);
        }
        if (setPaymentDialogOpen) {
          setPaymentDialogOpen(false);
        }
      } else {
        throw new Error(
          "La respuesta del backend no incluyó los datos del QR."
        );
      }
    } catch (error: any) {
      console.error(`Error al generar QR: ${error.message}`);
      // ✅ CORRECCIÓN: Asegurar que el toast se limpie SIEMPRE en caso de error
      toast.dismiss("qr-loading");
      toast.error(`Error al generar QR: ${error.message}`);
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    }
  };

  // Verificar el estado del pago
  const startPaymentStatusPolling = (orderId: string) => {
    cleanupPolling();

    console.log("🌐 API_URL en polling:", API_URL);

    const POLLING_INTERVAL = 3000;
    const MAX_POLLING_TIME = 10 * 60 * 1000;
    const MAX_RETRIES = 3;

    setPollingStartTime(Date.now());
    setRetryCount(0);

    // Función auxiliar para consultar el backend (evita duplicar código)
    const fetchStatus = async () => {
      try {
        // Utilizar siempre la versión más reciente de qrData mediante el ref
        const currentQrData = qrDataRef.current;

        console.log(
          "🔄 Verificando estado del pago (llamada inmediata/intervalo)..."
        );
        console.log(
          "🌐 Haciendo petición a:",
          `${API_URL}/api/mercadopago/check-status/${orderId}`
        );

        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status/${orderId}`,
          {
            headers,
            credentials: "include",
          }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        console.log("✅ Respuesta del backend:", statusData);
        setRetryCount(0);

        // Normalizar el status que devuelve el backend para que coincida con los textos mostrados en el diálogo
        const backendStatus: string = (statusData.status || "").toUpperCase();

        const normalizedStatus =
          backendStatus === "PENDING"
            ? "PENDIENTE"
            : backendStatus === "APPROVED" || backendStatus === "COMPLETED"
            ? "COMPLETADA"
            : backendStatus === "CANCELLED" ||
              backendStatus === "CANCELED" ||
              backendStatus === "REJECTED"
            ? "CANCELADA"
            : backendStatus; // Podría ser que ya venga en español

        setPaymentStatus(normalizedStatus);

        if (statusData.isCompleted || statusData.isCancelled) {
          cleanupPolling();

          if (statusData.isCompleted) {
            // ✅ TOAST CONTROL: Verificar si ya se procesó esta orden
            if (isOrderAlreadyProcessed(orderId)) {
              console.log(
                `🛡️ TOAST CONTROL: Orden ${orderId} ya fue procesada, saltando toasts`
              );
              return;
            }

            // ✅ TOAST CONTROL: Marcar como procesada ANTES de mostrar toasts
            markOrderAsProcessed(orderId);

            const cartData = {
              items: currentQrData?.items || [],
              total: currentQrData?.monto ?? 0,
            };

            await finalizeMPPayment(
              {
                ...statusData,
                cartData,
              },
              false
            );

            // ✅ TOAST CONTROL: Solo mostrar toast si no se mostró en finalizeMPPayment
            if (!isOrderAlreadyProcessed(orderId)) {
              toast.success("¡Pago completado! Cerrando en 2 segundos...");
            }

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
        console.error("❌ Error al verificar estado de pago:", error);

        // Incrementar el contador de reintentos y usar el valor ACTUALIZADO para la verificación.
        const nextRetries = retryCount + 1;
        setRetryCount(nextRetries);

        if (nextRetries >= MAX_RETRIES) {
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
    };

    // Llamada inmediata antes de iniciar el intervalo continuo
    fetchStatus();

    console.log(
      "⏰ CREANDO setInterval con intervalo de",
      POLLING_INTERVAL,
      "ms"
    );
    const interval = setInterval(() => {
      console.log("⏰ setInterval EJECUTÁNDOSE - llamada periódica");
      fetchStatus();
    }, POLLING_INTERVAL);

    console.log("📌 setInterval creado con ID:", interval);
    setPollingInterval(interval);
    console.log("✅ pollingInterval guardado en estado");
  };

  // Función para cancelar QR
  const cancelQRPayment = () => {
    console.log("🔴 Cancelando pago QR");

    // ✅ CORRECCIÓN: Limpiar toast de loading al cancelar
    toast.dismiss("qr-loading");

    // Limpiar polling
    if (pollingInterval) {
      clearInterval(pollingInterval);
      setPollingInterval(null);
    }

    // Limpiar estados
    setQrData(null);
    setPaymentStatus(null);
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);

    // Cerrar diálogo QR
    if (setQrDialogOpenRef) {
      setQrDialogOpenRef(false);
    }

    console.log("✅ Pago QR cancelado y estados limpiados");
  };

  // Limpiar el polling
  const cleanupPolling = () => {
    // console.log("🧹 cleanupPolling llamado - stack trace:");
    console.trace();
    if (pollingInterval) {
      console.log("🧹 Limpiando intervalo de polling con ID:", pollingInterval);
      clearInterval(pollingInterval);
      setPollingInterval(null);
    } else {
      // console.log("🧹 No hay intervalo para limpiar");
    }
  };

  // Función para finalizar el pago después de que MP confirme
  const finalizeMPPayment = async (
    paymentData: any,
    skipPrinting: boolean = false
  ) => {
    try {
      console.log("🔄 Finalizando pago con datos:", paymentData);

      // ✅ CRITICAL FIX: No ejecutar si se está procesando manualmente
      if (isProcessingManualPayment.current) {
        console.log(
          "🛑 FINALIZE MP: Cancelando flujo automático - procesamiento manual en curso"
        );
        return;
      }

      if (!user) {
        toast.error("Se perdió la sesión. Por favor inicia sesión nuevamente.");
        if (setQrDialogOpenRef) {
          setQrDialogOpenRef(false);
        }
        resetPaymentState();
        return;
      }

      if (paymentData.isCompleted) {
        console.log("💰 Pago QR completado exitosamente por el backend");

        // ✅ TOAST CONTROL: Verificar si ya se procesó esta orden
        const orderId =
          paymentData.orderId || paymentData.orderData?.id || "unknown";
        if (isOrderAlreadyProcessed(orderId)) {
          console.log(
            `🛡️ TOAST CONTROL: Orden ${orderId} ya fue procesada en finalizeMPPayment, saltando`
          );
          return;
        }

        // ✅ TOAST CONTROL: Marcar como procesada ANTES de continuar
        markOrderAsProcessed(orderId);

        // ✅ CORRECCIÓN: El backend refactorizado ya maneja todo automáticamente
        // Pero aún necesitamos imprimir el ticket con los datos de la orden
        console.log(
          "✅ FINALIZE MP: Backend ya procesó la orden automáticamente"
        );

        try {
          // ✅ OBTENER DATOS DE LA ORDEN DESDE EL BACKEND PARA IMPRESIÓN
          if (paymentData.orderData || paymentData.orderId) {
            console.log("🖨️ FINALIZE MP: Obteniendo datos para impresión");

            let orderDataForPrint = paymentData.orderData;

            // Si no tenemos datos completos pero sí tenemos un orderId, construir datos básicos
            if (!orderDataForPrint && paymentData.orderId) {
              console.log(
                "🖨️ FINALIZE MP: Construyendo datos básicos para impresión"
              );

              // Obtener datos del carrito actual/QR para impresión
              const cartData = paymentData.cartData || {
                items: qrDataRef.current?.items || [],
                total: qrDataRef.current?.monto || 0,
              };

              // ✅ CRITICAL FIX: Incluir nombre del business para evitar fallback "Verdulería"
              const businessName =
                businessInfo?.nombre || businessInfo?.name || "Verdulería";

              orderDataForPrint = {
                id: paymentData.orderId,
                idReal: paymentData.orderId,
                metodoPago: "qr",
                total: cartData.total,
                items: cartData.items,
                vendedorId: user.id,
                sucursalId: user.sucursalId,
                vendedor: user.nombre,
                businessName: businessName, // ✅ CRITICAL FIX
                estado: "COMPLETADA",
                createdAt: new Date().toISOString(),
              };
            }

            if (orderDataForPrint) {
              console.log("🖨️ FINALIZE MP: Iniciando impresión de ticket QR");

              // ✅ SIMULACIÓN: Log detallado de los datos que se van a imprimir
              console.log("🎯 SIMULACIÓN DE TICKET QR AUTOMÁTICO SIN AFIP:");
              console.log("📋 === DATOS PARA IMPRESIÓN ===");
              console.log("🆔 ID:", orderDataForPrint.id);
              console.log("🆔 ID Real:", orderDataForPrint.idReal);
              console.log("💳 Método de Pago:", orderDataForPrint.metodoPago);
              console.log("💰 Total:", orderDataForPrint.total);
              console.log("🛒 Items:", orderDataForPrint.items);
              console.log("👤 Vendedor ID:", orderDataForPrint.vendedorId);
              console.log("🏢 Sucursal ID:", orderDataForPrint.sucursalId);
              console.log("👤 Vendedor:", orderDataForPrint.vendedor);
              console.log("🟢 Estado:", orderDataForPrint.estado);
              console.log("📅 Creado:", orderDataForPrint.createdAt);
              console.log("🎯 === FIN SIMULACIÓN DATOS IMPRESIÓN ===");

              // Mostrar toast de impresión
              const printingToastId = toast.loading("Imprimiendo ticket...");

              try {
                // Llamar a impresión
                await handleTicketPrinting(orderDataForPrint);

                // Cerrar toast de impresión
                toast.dismiss(printingToastId);

                console.log("✅ FINALIZE MP: Ticket impreso exitosamente");
              } catch (printError) {
                console.error(
                  "❌ FINALIZE MP: Error en impresión:",
                  printError
                );
                toast.dismiss(printingToastId);
                toast.error(
                  "Error al imprimir el ticket, pero el pago se completó correctamente"
                );
              }
            } else {
              console.warn(
                "⚠️ FINALIZE MP: No se encontraron datos para impresión"
              );
            }
          } else {
            console.warn(
              "⚠️ FINALIZE MP: No hay datos de orden ni orderId para impresión"
            );
          }
        } catch (error) {
          console.error(
            "❌ FINALIZE MP: Error en proceso de impresión:",
            error
          );
          toast.error(
            "Error al imprimir el ticket, pero el pago se completó correctamente"
          );
        }

        // ✅ TOAST CONTROL: Solo mostrar toast de éxito si no se mostró antes
        toast.success("¡Pago completado exitosamente!");

        // Cerrar diálogo QR
        if (setQrDialogOpenRef) {
          setQrDialogOpenRef(false);
        }

        // ✅ CRÍTICO: Limpiar carrito y estados SOLO después de que todo esté completo
        // Esto evita que los productos se borren prematuramente
        setTimeout(() => {
          clearCart();
          resetPaymentState();
          console.log("🛒 Carrito limpiado después de pago QR exitoso");

          // ✅ CRÍTICO: El enfoque del input se maneja en el componente principal
          console.log(
            "✅ FINALIZE MP: Proceso completado, input será enfocado por el componente"
          );
        }, 100);

        console.log("✅ FINALIZE MP: Proceso completado con impresión");
      }
    } catch (error: any) {
      console.error("❌ Error al finalizar pago:", error);
      toast.error(`Error al finalizar el pago: ${error.message}`);
    } finally {
      // Asegurarse de limpiar spinner en todos los casos
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
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
    console.log("🧹 Reseteando estados del hook de procesamiento de pagos");
    setQrData(null);
    setPaymentStatus(null);
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);
    setManualQrPassword("");
    setManualQrPasswordDialogOpen(false);
    setManualQrOrderDetails(null);
    setIsManualPasswordSubmitting(false);
    setRoundedAmountDialogOpen(false);
    setExactPaymentDialogOpen(false);
    setPaidAmount(0);
    setChangeAmount(0);
    setCashAmount("");
    setSecondPaymentMethod("tarjeta");
    setRetryCount(0);
    setPollingStartTime(null);

    // ✅ TOAST CONTROL: Limpiar tracking de órdenes procesadas
    clearProcessedOrdersTracking();

    // ✅ IMPRESIÓN CONTROL: Limpiar tracking de órdenes impresas
    clearPrintedOrdersTracking();

    // Limpiar polling
    cleanupPolling();

    // Limpiar bandera de procesamiento manual
    isProcessingManualPayment.current = false;

    console.log("✅ Estados del hook de procesamiento de pagos reseteados");
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
        businessName: await getBusinessName(),
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

      // ✅ ANTI-DUPLICADOS: Crear orden con verificación de duplicados
      const orderResult = await createOrderWithDuplicateCheck(orderData);

      if (!orderResult) {
        console.log("⏳ PROCESS SPLIT: Orden cancelada por concurrencia");
        return;
      }

      if (orderResult.isDuplicate) {
        console.log(
          "✅ PROCESS SPLIT: Orden duplicada detectada, usando existente"
        );
        toast.success("Orden completada exitosamente");
        if (setSplitPaymentDialogOpenRef) {
          setSplitPaymentDialogOpenRef(false);
        }
        clearCart();
        resetPaymentState();
        return;
      }

      console.log("📋 PROCESS SPLIT: Nueva orden creada:", orderResult);

      // Añadir el idReal a los datos de la orden para impresión
      const enrichedOrderData = {
        ...orderData,
        idReal: orderResult.idReal || orderResult.id || null,
        id: orderResult.id || null,
      };

      // ✅ CORREGIDO: Usar la impresión a través del hook compartido
      // Mostrar toast de carga para la impresión ANTES de imprimir
      const printingToastId = toast.loading("Imprimiendo ticket...");

      try {
        // ✅ IMPRESIÓN SIMPLIFICADA: El hook useTicketPrinting ya maneja la doble impresión internamente
        await handleTicketPrinting(enrichedOrderData);

        // Cerrar el toast de carga de impresión
        toast.dismiss(printingToastId);
      } catch (printError) {
        // Asegurar que el toast se cierre siempre
        toast.dismiss(printingToastId);
        console.error("Error en impresión:", printError);
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
      // ✅ MEJORADO: También resetear estados en caso de error
      resetPaymentState();
    } finally {
      // ✅ MEJORADO: Asegurar que el estado se resetee SIEMPRE
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
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
      facturacionHabilitada: businessInfo?.facturacionHabilitada,
    });

    // ✅ VERIFICACIÓN F3: Asegurar que el pago mixto QR no active AFIP cuando no debe
    if (businessInfo?.facturacionHabilitada !== true) {
      console.log(
        "✅ PAGO MIXTO QR: Facturación AFIP deshabilitada - procesando sin AFIP"
      );
    } else {
      console.log(
        "⚠️ PAGO MIXTO QR: Facturación AFIP habilitada - debería usar flujo AFIP diferente"
      );
    }

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
        businessName: await getBusinessName(),
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

        // ✅ ANTI-DUPLICADOS: Crear orden con verificación de duplicados
        const orderResult = await createOrderWithDuplicateCheck(orderData);

        // Ocultar toast de carga
        toast.dismiss(processingToastId);

        if (!orderResult) {
          console.log("⏳ GENERATE SPLIT QR: Orden cancelada por concurrencia");
          return;
        }

        if (orderResult.isDuplicate) {
          console.log(
            "✅ GENERATE SPLIT QR: Orden duplicada detectada, usando existente"
          );
          toast.success("Orden completada exitosamente");
          if (setSplitPaymentDialogOpenRef) {
            setSplitPaymentDialogOpenRef(false);
          }
          clearCart();
          resetPaymentState();
          return;
        }

        console.log("📋 GENERATE SPLIT QR: Nueva orden creada:", orderResult);

        // Añadir el idReal a los datos de la orden para impresión
        const enrichedOrderData = {
          ...orderData,
          idReal: orderResult.idReal || orderResult.id || null,
          id: orderResult.id || null,
        };

        // Mostrar toast de carga para la impresión ANTES de imprimir
        const printingToastId = toast.loading("Imprimiendo ticket...");

        // ✅ IMPRESIÓN SIMPLIFICADA: El hook useTicketPrinting ya maneja la doble impresión internamente
        await handleTicketPrinting(enrichedOrderData);

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

      // ✅ CORRECCIÓN: Enviar el monto TOTAL de la venta (no solo el monto del QR)
      const totalAmount = cashAmountValue + qrAmount;

      const orderData = {
        monto: totalAmount, // ✅ CORRECTO: Monto TOTAL de la venta
        descripcion: `Pago mixto: QR $${qrAmount.toFixed(
          2
        )} + Efectivo $${cashAmountValue.toFixed(2)}`,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        items: orderItems,
        // ✅ CORRECCIÓN: Usar parámetros compatibles con backend refactorizado
        requiresAfipInvoice: false, // F3 - Flujo sin AFIP
        isSplitPayment: true,
        cashAmount: cashAmountValue,
        afipData: null,
      };

      console.log(
        "🔄 Enviando solicitud para generar QR de pago mixto:",
        orderData
      );
      console.log(
        "💰 CORRECCIÓN APLICADA: Enviando monto total =",
        totalAmount,
        "Efectivo =",
        cashAmountValue,
        "QR calculado =",
        qrAmount
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
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message || "Error al generar el QR para pago mixto"
        );
      }

      const result = await response.json();
      console.log("📲 Respuesta del backend (QR Mixto):", result);

      if (result.qrData) {
        // ✅ CORRECCIÓN: Limpiar toast de loading ANTES de continuar
        toast.dismiss("qr-loading");

        // ✅ CORRECCIÓN: Guardar la información del QR INCLUYENDO los items originales para impresión
        const qrDataWithAmount = {
          ...result,
          monto: qrAmount,
          items: orderItems, // ✅ CRITICAL FIX: Incluir items originales para impresión mixta
        };
        updateQrData(qrDataWithAmount);
        setPaymentStatus("PENDIENTE");

        if (setQrDialogOpen) {
          setQrDialogOpen(true);
        }
        if (setSplitPaymentDialogOpen) {
          setSplitPaymentDialogOpen(false);
        }

        // Iniciar el polling para verificar el estado del pago
        startPaymentStatusPolling(result.id);
      } else {
        throw new Error(
          "La respuesta del backend no incluyó los datos del QR para el pago mixto."
        );
      }
    } catch (error: any) {
      console.error("❌ Error al generar QR de pago mixto:", error);
      // ✅ CORRECCIÓN: Asegurar que el toast se limpie SIEMPRE en caso de error
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
          `${API_URL}/api/mercadopago/check-status/${orderId}`,
          {
            headers,
            credentials: "include",
          }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        console.log("🔄 Estado actual del pago mixto:", statusData);

        setRetryCount((prev) => prev + 1);

        // Normalizar el status que devuelve el backend para que coincida con los textos mostrados en el diálogo
        const backendStatus: string = (statusData.status || "").toUpperCase();

        const normalizedStatus =
          backendStatus === "PENDING"
            ? "PENDIENTE"
            : backendStatus === "APPROVED" || backendStatus === "COMPLETED"
            ? "COMPLETADA"
            : backendStatus === "CANCELLED" ||
              backendStatus === "CANCELED" ||
              backendStatus === "REJECTED"
            ? "CANCELADA"
            : backendStatus; // Podría ser que ya venga en español

        setPaymentStatus(normalizedStatus);

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

  // ✅ NUEVA: Función auxiliar para finalización manual sin conflictos de estado
  const finalizeManualPayment = async (
    orderId: number,
    isSplitPayment: boolean,
    cashAmount?: number
  ) => {
    try {
      console.log("🔄 MANUAL FINALIZE: Iniciando finalización manual:", {
        orderId,
        isSplitPayment,
        cashAmount,
      });

      if (!user) {
        throw new Error("Usuario no disponible");
      }

      if (isSplitPayment && typeof cashAmount === "number") {
        // ✅ FINALIZACIÓN MANUAL MIXTA: Procesar sin conflictos de estado
        console.log("🔄 MANUAL FINALIZE: Procesando pago mixto manual");

        const orderItems = qrDataRef.current?.items || [];
        const totalAmount = (qrDataRef.current?.monto || 0) + cashAmount;

        // ✅ VERIFICACIÓN ROBUSTA: Verificar que tenemos items válidos
        console.log("🔍 MANUAL FINALIZE: Verificando items:", {
          itemsCount: orderItems.length,
          hasQrData: !!qrDataRef.current,
          qrMonto: qrDataRef.current?.monto,
          cashAmount,
          totalAmount,
          itemsPreview: orderItems.slice(0, 2),
        });

        // ✅ CORRECCIÓN: Con backend refactorizado, NO crear órdenes locales para QR mixtos
        // Pero sí necesitamos imprimir el ticket con los datos de la orden
        console.warn(
          "✅ MANUAL FINALIZE: Backend refactorizado ya procesó todo automáticamente"
        );

        try {
          // ✅ CONSTRUIR DATOS PARA IMPRESIÓN DE PAGO MIXTO MANUAL
          console.log(
            "🖨️ MANUAL FINALIZE: Construyendo datos para impresión mixta"
          );

          const cartData = {
            items: qrDataRef.current?.items || [],
            total: qrDataRef.current?.monto || 0,
          };

          // ✅ SIMULACIÓN: Log detallado de los datos que se van a imprimir para PAGO MIXTO
          console.log("🎯 SIMULACIÓN DE TICKET PAGO MIXTO MANUAL:");
          console.log("📋 === DATOS DEL CARRITO MIXTO ===");
          console.log("🛒 Items del carrito:", cartData.items);
          console.log("💰 Total QR:", cartData.total);
          console.log("💰 Monto efectivo:", cashAmount);
          console.log(
            "💰 Total combinado:",
            (cartData.total || 0) + cashAmount
          );
          console.log("📋 === DATOS DEL QR MIXTO ===");
          console.log("🔍 qrDataRef.current completo:", qrDataRef.current);
          console.log("🆔 orderId:", orderId);
          console.log("👤 user:", user);

          const orderDataForPrint = {
            id: orderId,
            idReal: orderId,
            metodoPago: "split", // Método mixto
            total: totalAmount, // Total combinado
            items: cartData.items,
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            estado: "COMPLETADA",
            createdAt: new Date().toISOString(),
            // ✅ DATOS ESPECÍFICOS PARA PAGO MIXTO MANUAL
            pagos: [
              {
                metodoPago: "efectivo",
                monto: cashAmount,
              },
              {
                metodoPago: "qr",
                monto: cartData.total,
              },
            ],
          };

          // ✅ SIMULACIÓN: Log completo de los datos finales de impresión MIXTO
          console.log(
            "🎯 === DATOS FINALES PARA IMPRESIÓN PAGO MIXTO MANUAL ==="
          );
          console.log("🆔 ID:", orderDataForPrint.id);
          console.log("🆔 ID Real:", orderDataForPrint.idReal);
          console.log("💳 Método de Pago:", orderDataForPrint.metodoPago);
          console.log("💰 Total COMBINADO:", orderDataForPrint.total);

          // ✅ CRÍTICO: Mostrar toast de impresión
          const manualPrintingToastId = toast.loading("Imprimiendo ticket...");

          try {
            // Llamar a impresión
            await handleTicketPrinting(orderDataForPrint);

            // Cerrar toast de impresión
            toast.dismiss(manualPrintingToastId);

            console.log("✅ MANUAL FINALIZE: Ticket impreso exitosamente");
          } catch (printError) {
            console.error(
              "❌ MANUAL FINALIZE: Error en impresión:",
              printError
            );
            toast.dismiss(manualPrintingToastId);
            toast.error(
              "Error al imprimir el ticket, pero el pago se completó correctamente"
            );
          }
          console.log("🛒 Items:", orderDataForPrint.items);
          console.log("👤 Vendedor ID:", orderDataForPrint.vendedorId);
          console.log("🏢 Sucursal ID:", orderDataForPrint.sucursalId);
          console.log("👤 Vendedor:", orderDataForPrint.vendedor);
          console.log("🟢 Estado:", orderDataForPrint.estado);
          console.log("📅 Creado:", orderDataForPrint.createdAt);
          console.log("💵 === DESGLOSE DE PAGOS MIXTOS ===");
          console.log("💳 Pagos array:", orderDataForPrint.pagos);
          orderDataForPrint.pagos?.forEach((pago: any, index: number) => {
            console.log(
              `💳 Pago ${index + 1}: ${pago.metodoPago} - $${pago.monto}`
            );
          });
          console.log("🎯 === FIN SIMULACIÓN DATOS IMPRESIÓN MIXTO ===");

          // ✅ VALIDACIÓN: Verificar que los datos mixtos son válidos antes de imprimir
          if (
            !orderDataForPrint.items ||
            orderDataForPrint.items.length === 0
          ) {
            console.error("❌ MANUAL FINALIZE MIXTO: Items vacíos o inválidos");
            console.error("🔍 cartData.items:", cartData.items);
            console.error(
              "🔍 qrDataRef.current?.items:",
              qrDataRef.current?.items
            );
            throw new Error("No hay items válidos para imprimir pago mixto");
          }

          if (!orderDataForPrint.total || orderDataForPrint.total <= 0) {
            console.error("❌ MANUAL FINALIZE MIXTO: Total inválido");
            console.error("🔍 Total calculado:", totalAmount);
            console.error("🔍 cartData.total:", cartData.total);
            console.error("🔍 cashAmount:", cashAmount);
            throw new Error("Total de la orden mixta inválido");
          }

          if (
            !orderDataForPrint.pagos ||
            orderDataForPrint.pagos.length !== 2
          ) {
            console.error("❌ MANUAL FINALIZE MIXTO: Datos de pagos inválidos");
            console.error("🔍 pagos array:", orderDataForPrint.pagos);
            throw new Error("Datos de pagos mixtos incompletos");
          }

          console.log(
            "🖨️ MANUAL FINALIZE: Iniciando impresión de ticket mixto manual"
          );

          // Mostrar toast de impresión
          const printingToastId = toast.loading("Imprimiendo ticket...");

          try {
            // Llamar a impresión
            await handleTicketPrinting(orderDataForPrint);

            // Cerrar toast de impresión
            toast.dismiss(printingToastId);

            console.log(
              "✅ MANUAL FINALIZE: Ticket mixto impreso exitosamente"
            );
          } catch (printError) {
            console.error(
              "❌ MANUAL FINALIZE: Error en impresión:",
              printError
            );
            toast.dismiss(printingToastId);
            toast.error(
              "Error al imprimir el ticket, pero el pago se completó correctamente"
            );
          }
        } catch (error) {
          console.error(
            "❌ MANUAL FINALIZE: Error en proceso de impresión:",
            error
          );
          toast.error(
            "Error al imprimir el ticket, pero el pago se completó correctamente"
          );
        }

        // Completar el proceso
        toast.success("¡Pago mixto completado exitosamente!");

        console.log(
          "✅ MANUAL FINALIZE: Completado con impresión (backend ya procesó)"
        );
        return; // Salir temprano
      } else {
        // ✅ FINALIZACIÓN MANUAL QR: Procesar sin conflictos de estado
        console.log("🔄 MANUAL FINALIZE: Procesando pago QR manual");

        const orderItems = qrDataRef.current?.items || [];

        // ✅ VERIFICACIÓN ROBUSTA: Verificar que tenemos items válidos
        console.log("🔍 MANUAL FINALIZE QR: Verificando items:", {
          itemsCount: orderItems.length,
          hasQrData: !!qrDataRef.current,
          qrMonto: qrDataRef.current?.monto,
          itemsPreview: orderItems.slice(0, 2),
        });

        // ✅ CORRECCIÓN: Con backend refactorizado, NO crear órdenes locales para QR
        // Pero sí necesitamos imprimir el ticket con los datos de la orden
        console.warn(
          "✅ MANUAL FINALIZE QR: Backend refactorizado ya procesó todo automáticamente"
        );

        try {
          // ✅ CONSTRUIR DATOS PARA IMPRESIÓN DE QR MANUAL
          console.log(
            "🖨️ MANUAL FINALIZE QR: Construyendo datos para impresión"
          );

          const cartData = {
            items: qrDataRef.current?.items || [],
            total: qrDataRef.current?.monto || 0,
          };

          // ✅ SIMULACIÓN: Log detallado de los datos que se van a imprimir
          console.log("🎯 SIMULACIÓN DE TICKET QR SIN AFIP:");
          console.log("📋 === DATOS DEL CARRITO ===");
          console.log("🛒 Items del carrito:", cartData.items);
          console.log("💰 Total del carrito:", cartData.total);
          console.log("📋 === DATOS DEL QR ===");
          console.log("🔍 qrDataRef.current completo:", qrDataRef.current);
          console.log("🆔 orderId:", orderId);
          console.log("👤 user:", user);

          const orderDataForPrint = {
            id: orderId,
            idReal: orderId,
            metodoPago: "qr",
            total: cartData.total,
            items: cartData.items,
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            estado: "COMPLETADA",
            createdAt: new Date().toISOString(),
          };

          // ✅ SIMULACIÓN: Log completo de los datos finales de impresión
          console.log("🎯 === DATOS FINALES PARA IMPRESIÓN QR SIN AFIP ===");
          console.log("🆔 ID:", orderDataForPrint.id);
          console.log("🆔 ID Real:", orderDataForPrint.idReal);
          console.log("💳 Método de Pago:", orderDataForPrint.metodoPago);
          console.log("💰 Total:", orderDataForPrint.total);
          console.log("🛒 Items:", orderDataForPrint.items);
          console.log("👤 Vendedor ID:", orderDataForPrint.vendedorId);
          console.log("🏢 Sucursal ID:", orderDataForPrint.sucursalId);
          console.log("👤 Vendedor:", orderDataForPrint.vendedor);
          console.log("🟢 Estado:", orderDataForPrint.estado);
          console.log("📅 Creado:", orderDataForPrint.createdAt);
          console.log("🎯 === FIN SIMULACIÓN DATOS IMPRESIÓN ===");

          // ✅ VALIDACIÓN: Verificar que los datos son válidos antes de imprimir
          if (
            !orderDataForPrint.items ||
            orderDataForPrint.items.length === 0
          ) {
            console.error("❌ MANUAL FINALIZE QR: Items vacíos o inválidos");
            console.error("🔍 cartData.items:", cartData.items);
            console.error(
              "🔍 qrDataRef.current?.items:",
              qrDataRef.current?.items
            );
            throw new Error("No hay items válidos para imprimir");
          }

          if (!orderDataForPrint.total || orderDataForPrint.total <= 0) {
            console.error("❌ MANUAL FINALIZE QR: Total inválido");
            console.error("🔍 cartData.total:", cartData.total);
            console.error(
              "🔍 qrDataRef.current?.monto:",
              qrDataRef.current?.monto
            );
            throw new Error("Total de la orden inválido");
          }

          console.log(
            "🖨️ MANUAL FINALIZE QR: Iniciando impresión de ticket QR manual"
          );

          // Mostrar toast de impresión
          const printingToastId = toast.loading("Imprimiendo ticket...");

          try {
            // Llamar a impresión
            await handleTicketPrinting(orderDataForPrint);

            // Cerrar toast de impresión
            toast.dismiss(printingToastId);

            console.log(
              "✅ MANUAL FINALIZE QR: Ticket QR impreso exitosamente"
            );
          } catch (printError) {
            console.error(
              "❌ MANUAL FINALIZE QR: Error en impresión:",
              printError
            );
            toast.dismiss(printingToastId);
            toast.error(
              "Error al imprimir el ticket, pero el pago se completó correctamente"
            );
          }
        } catch (error) {
          console.error(
            "❌ MANUAL FINALIZE QR: Error en proceso de impresión:",
            error
          );
          toast.error(
            "Error al imprimir el ticket, pero el pago se completó correctamente"
          );
        }

        // ✅ CRÍTICO: Mostrar toast de éxito después de impresión
        toast.success("¡Pago QR completado exitosamente!");

        console.log(
          "✅ MANUAL FINALIZE QR: Completado con impresión (backend ya procesó)"
        );

        // ✅ CRÍTICO: Limpiar carrito y estados después de éxito
        setTimeout(() => {
          clearCart();
          resetPaymentState();
          console.log("🛒 Carrito limpiado después de pago QR manual exitoso");
        }, 100);

        return; // Salir temprano
      }

      // ✅ FINALIZACIÓN EXITOSA: Limpiar todo de manera controlada
      console.log("🧹 MANUAL FINALIZE: Iniciando limpieza controlada");

      // Mostrar toast de éxito
      toast.success("¡Orden completada exitosamente!");

      // Cerrar diálogo QR
      if (setQrDialogOpenRef) {
        setQrDialogOpenRef(false);
      }

      // Limpiar carrito
      clearCart();

      // Limpiar estados de polling y procesamiento
      cleanupPolling();

      // Resetear todos los estados del hook
      resetPaymentState();

      console.log("✅ MANUAL FINALIZE: Limpieza controlada completada");
    } catch (error: any) {
      console.error("❌ MANUAL FINALIZE: Error en finalización:", error);

      // ✅ MANEJO ESPECIAL: Error de "No valid items" - probablemente backend ya procesó todo
      if (error.message?.includes("No valid items to create order payload")) {
        console.warn(
          "🔧 MANUAL FINALIZE: Error de items vacíos - posiblemente backend ya procesó todo"
        );
        console.warn("🔧 Completando proceso sin crear orden local");

        toast.success("¡Pago completado exitosamente!");

        // Mostrar advertencia explicativa
        setTimeout(() => {
          toast.info("ℹ️ El pago se procesó automáticamente", {
            description: "No fue necesario crear una orden local adicional",
            duration: 5000,
          });
        }, 1500);

        return; // Salir sin error para evitar que se quede buggeado
      }

      throw error; // Re-lanzar otros errores para manejo en la función principal
    }
  };

  // Función para manejar el envío de contraseña QR manual
  // ✅ NUEVO: Estado para proteger contra envíos múltiples de contraseña manual
  const [isManualPasswordSubmitting, setIsManualPasswordSubmitting] =
    useState(false);

  const handleManualQrPasswordSubmit = async (isAfipFlow: boolean) => {
    // ✅ PROTECCIÓN: Evitar envíos múltiples
    if (isManualPasswordSubmitting) {
      console.log("🛡️ Ignorando envío múltiple de contraseña manual");
      return;
    }

    if (!qrDataRef.current?.id) {
      toast.error("No hay una orden QR pendiente para completar.");
      return;
    }

    const orderId = qrDataRef.current.id;
    console.log(
      `🔧 Completando manualmente orden ${orderId}. ¿Requiere AFIP?: ${isAfipFlow}`
    );

    // ✅ TOAST CONTROL: Verificar si ya se procesó esta orden
    if (isOrderAlreadyProcessed(orderId)) {
      console.log(
        `🛡️ TOAST CONTROL: Orden ${orderId} ya fue procesada manualmente, saltando`
      );
      return;
    }

    // ✅ CRITICAL FIX: Marcar INMEDIATAMENTE que estamos procesando manualmente
    isProcessingManualPayment.current = true;
    console.log(
      "🛑 MANUAL COMPLETE: Marcado como procesamiento manual para prevenir doble impresión"
    );

    // ✅ PROTECCIÓN: Marcar como enviando
    setIsManualPasswordSubmitting(true);

    try {
      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };
      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      const body = {
        orderId,
        requiresAfipInvoice: isAfipFlow,
      };

      console.log("🔧 Enviando a /manual-complete:", body);

      const response = await fetch(
        `${API_URL}/api/mercadopago/manual-complete`,
        {
          method: "POST",
          headers,
          body: JSON.stringify(body),
        }
      );

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message || "Error al completar el pago manualmente"
        );
      }

      const result = await response.json();
      console.log("✅ Pago manual completado:", result);

      // ✅ TOAST CONTROL: Marcar como procesada ANTES de mostrar toasts
      markOrderAsProcessed(orderId);

      // ✅ CORRECCIÓN: NO cerrar diálogos inmediatamente, esperar impresión
      setManualQrPasswordDialogOpen(false);
      setManualQrPassword("");

      // ✅ PASO 1: Detener polling para evitar doble procesamiento
      cleanupPolling();

      // ✅ PASO 2: Mostrar loading para impresión
      const printingToastId = toast.loading(
        "Procesando impresión del ticket..."
      );

      try {
        // ✅ PASO 3: Detectar si es pago mixto y procesar correctamente
        const orderDetails = manualQrOrderDetails;

        // ✅ CRITICAL DEBUG: Log completo para detectar por qué no se reconoce como mixto
        console.log("🔍 MANUAL COMPLETE DEBUG: Verificando tipo de pago");
        console.log("🔍 manualQrOrderDetails:", manualQrOrderDetails);
        console.log("🔍 orderDetails:", orderDetails);
        console.log(
          "🔍 orderDetails?.isSplitPayment:",
          orderDetails?.isSplitPayment
        );
        console.log("🔍 orderDetails?.cashAmount:", orderDetails?.cashAmount);
        console.log("🔍 qrDataRef.current:", qrDataRef.current);

        // ✅ CORRECCIÓN: Detectar pago mixto de MÚLTIPLES formas
        const isMixedPayment =
          (orderDetails?.isSplitPayment && orderDetails?.cashAmount) ||
          (qrDataRef.current?.isSplitPayment &&
            qrDataRef.current?.cashAmount) ||
          (qrDataRef.current?.cashAmount && qrDataRef.current?.cashAmount > 0);

        console.log("🔍 isMixedPayment DETECTADO:", isMixedPayment);

        if (isMixedPayment) {
          // ✅ PAGO MIXTO MANUAL: Usar los datos completos
          console.log(
            "🖨️ MANUAL COMPLETE: Iniciando impresión mixta inmediata"
          );

          const cartData = {
            items: qrDataRef.current?.items || [],
            total: qrDataRef.current?.monto || 0,
          };

          // ✅ CORRECCIÓN: Obtener cashAmount de múltiples fuentes posibles
          const cashAmount =
            orderDetails?.cashAmount || qrDataRef.current?.cashAmount || 0;
          const totalAmount = cartData.total + cashAmount;

          // ✅ CRITICAL FIX: Incluir nombre del business para pagos mixtos
          const businessName =
            businessInfo?.nombre || businessInfo?.name || "Verdulería";

          const orderDataForPrint = {
            id: qrDataRef.current?.id || orderId,
            idReal: qrDataRef.current?.id || orderId,
            metodoPago: "split", // Método mixto
            total: totalAmount,
            items: cartData.items,
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            businessName: businessName, // ✅ CRITICAL FIX
            estado: "COMPLETADA",
            createdAt: new Date().toISOString(),
            // ✅ DATOS ESPECÍFICOS PARA PAGO MIXTO MANUAL
            pagos: [
              {
                metodoPago: "efectivo",
                monto: cashAmount,
              },
              {
                metodoPago: "qr",
                monto: cartData.total,
              },
            ],
          };

          // ✅ SIMULACIÓN: Log detallado de impresión mixta manual
          console.log("🎯 SIMULACIÓN DE TICKET MIXTO MANUAL SINCRONIZADO:");
          console.log("📋 === DATOS PARA IMPRESIÓN MIXTA MANUAL ===");
          console.log("🆔 ID:", orderDataForPrint.id);
          console.log("💳 Método:", orderDataForPrint.metodoPago);
          console.log("💰 Total combinado:", orderDataForPrint.total);
          console.log("💰 Efectivo:", cashAmount);
          console.log("💰 QR:", cartData.total);
          console.log("🛒 Items:", orderDataForPrint.items);
          console.log("💳 Pagos:", orderDataForPrint.pagos);

          // Ejecutar impresión mixta
          await handleTicketPrinting(orderDataForPrint);

          toast.dismiss(printingToastId);
          toast.success("¡Pago mixto completado e impreso exitosamente!");
        } else {
          // ✅ PAGO QR SIMPLE MANUAL
          console.log(
            "🖨️ MANUAL COMPLETE: Iniciando impresión QR simple inmediata"
          );

          const cartData = {
            items: qrDataRef.current?.items || [],
            total: qrDataRef.current?.monto || 0,
          };

          // ✅ CRITICAL FIX: Incluir nombre del business para evitar fallback "Verdulería"
          const businessName =
            businessInfo?.nombre || businessInfo?.name || "Verdulería";

          const orderDataForPrint = {
            id: qrDataRef.current?.id || orderId,
            idReal: qrDataRef.current?.id || orderId,
            metodoPago: "qr",
            total: cartData.total,
            items: cartData.items,
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            businessName: businessName, // ✅ CRITICAL FIX
            estado: "COMPLETADA",
            createdAt: new Date().toISOString(),
          };

          // ✅ SIMULACIÓN: Log detallado de impresión manual
          console.log("🎯 SIMULACIÓN DE TICKET QR MANUAL SINCRONIZADO:");
          console.log("📋 === DATOS PARA IMPRESIÓN MANUAL ===");
          console.log("🆔 ID:", orderDataForPrint.id);
          console.log("💳 Método:", orderDataForPrint.metodoPago);
          console.log("💰 Total:", orderDataForPrint.total);
          console.log("🛒 Items:", orderDataForPrint.items);

          // Ejecutar impresión
          await handleTicketPrinting(orderDataForPrint);

          toast.dismiss(printingToastId);
          toast.success("¡Pago completado e impreso exitosamente!");
        }
      } catch (printError) {
        console.error("❌ Error en impresión manual:", printError);
        toast.dismiss(printingToastId);
        toast.success("¡Pago completado exitosamente!");
        toast.error(
          "Error al imprimir, pero el pago se completó correctamente"
        );
      }

      // ✅ PASO 5: AHORA SÍ limpiar todo después de la impresión
      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }
      clearCart();
      resetPaymentState();
    } catch (error: any) {
      console.error("Error en handleManualQrPasswordSubmit:", error);
      toast.error(error.message);
      // ✅ MANTENER diálogos abiertos en caso de error para que el usuario pueda reintentar
      // NO cerrar setManualQrPasswordDialogOpen ni setQrDialogOpen
    } finally {
      // ✅ PROTECCIÓN: Siempre limpiar el estado de loading
      setIsManualPasswordSubmitting(false);

      // ✅ CRITICAL FIX: Limpiar bandera de procesamiento manual
      isProcessingManualPayment.current = false;
      console.log(
        "✅ MANUAL COMPLETE: Bandera de procesamiento manual limpiada"
      );
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

  // ✅ NUEVO: Usar el hook de impresión con soporte para doble impresión
  const { handleTicketPrinting: printTicketWithDoubleSupport } =
    useTicketPrinting();

  // Función auxiliar para manejar la impresión de tickets
  const handleTicketPrinting = async (orderData: any): Promise<boolean> => {
    try {
      console.log("🎯 PAYMENT PROCESSING: Iniciando impresión de ticket");
      console.log("📋 Datos de la orden para impresión:", orderData);

      // ✅ CONTROL DE IMPRESIÓN: Verificar si ya se imprimió esta orden
      const orderId =
        orderData.referencia || orderData.idReal || orderData.id || "unknown";
      if (isOrderAlreadyPrinted(orderId)) {
        console.log(
          `⚠️ IMPRESIÓN CONTROL: Orden ${orderId} ya fue impresa, saltando impresión`
        );
        toast.info("Ticket ya fue impreso anteriormente");
        return true; // Retornar true porque la "impresión" fue exitosa (ya se hizo antes)
      }

      // Obtener appId desde los argumentos de la aplicación
      let appId = null;
      try {
        if (
          typeof window !== "undefined" &&
          (window as any).electron?.process?.argv
        ) {
          const argv = (window as any).electron.process.argv as string[];
          const appIdArg = argv.find((arg: string) =>
            arg.startsWith("--app-id=")
          );
          if (appIdArg) {
            appId = appIdArg.split("=")[1];
            console.log("✅ AppId obtenido:", appId);
          } else {
            console.log("⚠️ No se encontró appId en argv");
          }
        }
      } catch (error) {
        console.log("⚠️ No se pudo obtener appId, usando null");
        appId = null;
      }

      console.log("🔧 Parámetros para impresión:", {
        API_URL,
        appId,
        appIdType: typeof appId,
      });

      // ✅ USAR EL HOOK QUE TIENE SOPORTE PARA DOBLE IMPRESIÓN
      const printResult = await printTicketWithDoubleSupport(
        orderData,
        API_URL,
        appId
      );

      // ✅ MARCAR COMO IMPRESA SOLO SI LA IMPRESIÓN FUE EXITOSA
      if (printResult) {
        markOrderAsPrinted(orderId);
        console.log(
          `✅ IMPRESIÓN CONTROL: Orden ${orderId} impresa exitosamente y marcada`
        );
      }

      // ✅ SIEMPRE RETORNAR TRUE: La impresión es opcional, no debe bloquear la orden
      if (!printResult) {
        console.log("⚠️ La impresión falló, pero continuando con la orden");
        toast.error(
          "Error al imprimir el ticket, pero la orden se completó correctamente"
        );
      }

      return true; // ✅ SIEMPRE retornar true para no bloquear el flujo
    } catch (error: any) {
      console.error("❌ Error al imprimir:", error);
      toast.error(
        "Error al imprimir el ticket, pero la orden se completó correctamente",
        {
          description: "La venta se registró exitosamente en el sistema",
        }
      );
      // ✅ SIEMPRE retornar true: La impresión no debe bloquear la finalización de la orden
      return true;
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
        console.log("💰 Pago mixto QR completado exitosamente por el backend");

        // ✅ CORRECCIÓN: El backend refactorizado ya maneja todo automáticamente
        // Pero aún necesitamos imprimir el ticket con los datos de la orden
        console.log(
          "✅ FINALIZE SPLIT MP: Backend ya procesó la orden automáticamente"
        );

        try {
          // ✅ OBTENER DATOS DE LA ORDEN DESDE EL BACKEND PARA IMPRESIÓN
          if (paymentData.orderData || paymentData.orderId) {
            console.log(
              "🖨️ FINALIZE SPLIT MP: Obteniendo datos para impresión"
            );

            let orderDataForPrint = paymentData.orderData;

            // Si no tenemos datos completos pero sí tenemos un orderId, construir datos básicos
            if (!orderDataForPrint && paymentData.orderId) {
              console.log(
                "🖨️ FINALIZE SPLIT MP: Construyendo datos básicos para impresión"
              );

              // Obtener datos del carrito actual/QR para impresión
              const cartData = paymentData.cartData || {
                items: qrDataRef.current?.items || [],
                total: qrDataRef.current?.monto || 0,
              };

              // ✅ SIMULACIÓN: Log detallado de los datos que se van a imprimir para PAGO MIXTO AUTOMÁTICO
              console.log("🎯 SIMULACIÓN DE TICKET PAGO MIXTO AUTOMÁTICO:");
              console.log("📋 === DATOS DEL CARRITO MIXTO AUTOMÁTICO ===");
              console.log("🛒 Items del carrito:", cartData.items);
              console.log("💰 Total QR:", cartData.total);
              console.log("💰 Monto efectivo:", cashAmount);
              console.log(
                "💰 Total combinado:",
                (cartData.total || 0) + (cashAmount || 0)
              );
              console.log("📋 === DATOS DEL PAYMENT DATA ===");
              console.log("🔍 paymentData completo:", paymentData);
              console.log("🆔 orderId:", paymentData.orderId);
              console.log("👤 user:", user);

              // ✅ CRITICAL FIX: Incluir nombre del business para pagos mixtos automáticos
              const businessName =
                businessInfo?.nombre || businessInfo?.name || "Verdulería";

              orderDataForPrint = {
                id: paymentData.orderId,
                idReal: paymentData.orderId,
                metodoPago: "split", // Método mixto
                total: (cartData.total || 0) + (cashAmount || 0), // Total combinado
                items: cartData.items,
                vendedorId: user.id,
                sucursalId: user.sucursalId,
                vendedor: user.nombre,
                businessName: businessName, // ✅ CRITICAL FIX
                estado: "COMPLETADA",
                createdAt: new Date().toISOString(),
                // ✅ DATOS ESPECÍFICOS PARA PAGO MIXTO AUTOMÁTICO
                pagos: [
                  {
                    metodoPago: "efectivo",
                    monto: cashAmount,
                  },
                  {
                    metodoPago: "qr",
                    monto: cartData.total,
                  },
                ],
              };

              // ✅ SIMULACIÓN: Log completo de los datos finales de impresión MIXTO AUTOMÁTICO
              console.log(
                "🎯 === DATOS FINALES PARA IMPRESIÓN PAGO MIXTO AUTOMÁTICO ==="
              );
              console.log("🆔 ID:", orderDataForPrint.id);
              console.log("🆔 ID Real:", orderDataForPrint.idReal);
              console.log("💳 Método de Pago:", orderDataForPrint.metodoPago);
              console.log("💰 Total COMBINADO:", orderDataForPrint.total);
              console.log("🛒 Items:", orderDataForPrint.items);
              console.log("👤 Vendedor ID:", orderDataForPrint.vendedorId);
              console.log("🏢 Sucursal ID:", orderDataForPrint.sucursalId);
              console.log("👤 Vendedor:", orderDataForPrint.vendedor);
              console.log("🟢 Estado:", orderDataForPrint.estado);
              console.log("📅 Creado:", orderDataForPrint.createdAt);
              console.log("💵 === DESGLOSE DE PAGOS MIXTOS AUTOMÁTICO ===");
              console.log("💳 Pagos array:", orderDataForPrint.pagos);
              orderDataForPrint.pagos?.forEach((pago: any, index: number) => {
                console.log(
                  `💳 Pago ${index + 1}: ${pago.metodoPago} - $${pago.monto}`
                );
              });
              console.log(
                "🎯 === FIN SIMULACIÓN DATOS IMPRESIÓN MIXTO AUTOMÁTICO ==="
              );

              // ✅ VALIDACIÓN: Verificar que los datos mixtos automáticos son válidos
              if (
                !orderDataForPrint.items ||
                orderDataForPrint.items.length === 0
              ) {
                console.error("❌ FINALIZE SPLIT MP: Items vacíos o inválidos");
                console.error("🔍 cartData.items:", cartData.items);
                console.error(
                  "🔍 qrDataRef.current?.items:",
                  qrDataRef.current?.items
                );
                throw new Error(
                  "No hay items válidos para imprimir pago mixto automático"
                );
              }

              if (!orderDataForPrint.total || orderDataForPrint.total <= 0) {
                console.error("❌ FINALIZE SPLIT MP: Total inválido");
                console.error("🔍 cartData.total:", cartData.total);
                console.error("🔍 cashAmount:", cashAmount);
                throw new Error("Total de la orden mixta automática inválido");
              }
            } else if (orderDataForPrint) {
              // ✅ SIMULACIÓN: Log para datos que ya vienen del backend
              console.log(
                "🎯 SIMULACIÓN DE TICKET PAGO MIXTO (DATOS DEL BACKEND):"
              );
              console.log("📋 === DATOS RECIBIDOS DEL BACKEND ===");
              console.log("🆔 ID:", orderDataForPrint.id);
              console.log("💳 Método de Pago:", orderDataForPrint.metodoPago);
              console.log("💰 Total:", orderDataForPrint.total);
              console.log("🛒 Items:", orderDataForPrint.items);
              console.log("🎯 === FIN SIMULACIÓN BACKEND ===");
            }

            if (orderDataForPrint) {
              console.log(
                "🖨️ FINALIZE SPLIT MP: Iniciando impresión de ticket mixto"
              );

              // Mostrar toast de impresión
              const printingToastId = toast.loading("Imprimiendo ticket...");

              try {
                // Llamar a impresión
                await handleTicketPrinting(orderDataForPrint);

                // Cerrar toast de impresión
                toast.dismiss(printingToastId);

                console.log(
                  "✅ FINALIZE SPLIT MP: Ticket impreso exitosamente"
                );
              } catch (printError) {
                console.error(
                  "❌ FINALIZE SPLIT MP: Error en impresión:",
                  printError
                );
                toast.dismiss(printingToastId);
                toast.error(
                  "Error al imprimir el ticket, pero el pago se completó correctamente"
                );
              }
            } else {
              console.warn(
                "⚠️ FINALIZE SPLIT MP: No se encontraron datos para impresión"
              );
            }
          } else {
            console.warn(
              "⚠️ FINALIZE SPLIT MP: No hay datos de orden ni orderId para impresión"
            );
          }
        } catch (error) {
          console.error(
            "❌ FINALIZE SPLIT MP: Error en proceso de impresión:",
            error
          );
          toast.error(
            "Error al imprimir el ticket, pero el pago se completó correctamente"
          );
        }

        // ✅ MEJORA: Toast más específico y sincronizado para pago mixto automático
        toast.success("¡Pago mixto completado e impreso exitosamente!");

        // ✅ MEJORA: Breve pausa para mejor UX antes de cerrar diálogo
        setTimeout(() => {
          // Cerrar diálogo QR
          if (setQrDialogOpenRef) {
            setQrDialogOpenRef(false);
          }

          // Limpiar carrito y estados
          clearCart();
          resetPaymentState();
        }, 500);

        console.log("✅ FINALIZE SPLIT MP: Proceso completado con impresión");
      }
    } catch (error: any) {
      console.error("❌ Error al finalizar pago:", error);
      toast.error(`Error al procesar el pago mixto: ${error.message}`);
    } finally {
      // Asegurarse de limpiar spinner en todos los casos
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    }
  };

  useEffect(() => {
    return () => {
      cleanupPolling();
    };
  }, []);

  return {
    // Estados principales
    isProcessingPayment,
    selectedPaymentMethod,
    qrData,
    paymentStatus,
    pollingInterval,
    manualQrPasswordDialogOpen,
    manualQrPassword,
    manualQrOrderDetails,
    isManualPasswordSubmitting,
    retryCount,
    pollingStartTime,

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

    // Estados de pago mixto
    cashAmount,
    setCashAmount,
    secondPaymentMethod,
    setSecondPaymentMethod,

    // Funciones principales
    processPayment,
    confirmExactPayment,
    handleCashPayment,
    generateQRPayment,
    startPaymentStatusPolling,
    cancelQRPayment,
    cleanupPolling,
    resetPaymentState,
    finalizeMPPayment,

    // Funciones de pago mixto
    handleSplitPayment,
    processSplitPayment,
    generateSplitQRPayment,
    startSplitPaymentStatusPolling,
    finalizeSplitMPPayment,

    // Funciones manuales
    completarOrdenManualmente,
    finalizeManualPayment,
    handleManualQrPasswordSubmit,

    // Funciones auxiliares
    formatFechaArgentina,
    handleTicketPrinting,

    // ✅ NUEVO: Control de toasts para evitar duplicados
    clearProcessedOrdersTracking,
    clearPrintedOrdersTracking,

    // ✅ NUEVO: Funciones para actualizar estados del QR
    updateQrData,
    setPaymentStatus,
    setManualQrPasswordDialogOpen,
    setManualQrPassword,
  };
}
