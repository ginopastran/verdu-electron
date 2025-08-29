import { useState, RefObject, useRef } from "react";
import { toast } from "sonner";
import { Product } from "./useCartState";
import { getBusinessName } from "@/utils/businessHelpers";
import { useBusinessInfo } from "./useBusinessInfo";
import QRCode from "qrcode";

interface AfipPaymentOptions {
  user: any;
  API_URL: string;
  appId: string | null;
  clearCart: () => void;
  calculateTotal: () => number;
  setPaymentDialogOpen: (open: boolean) => void;
  searchInputRef?: RefObject<HTMLInputElement | null>;
  setSplitPaymentDialogOpen?: (open: boolean) => void;
  setQrDialogOpen?: (open: boolean) => void;
  getCurrentItems?: () => any[]; // ✅ AGREGADO
}

interface ManualQrDetails {
  orderId: number;
  paymentId: string;
  totalAmount: number;
  cashAmount: number;
  qrAmount: number;
}

export function useAfipPaymentProcessing({
  user,
  API_URL,
  appId,
  clearCart,
  calculateTotal,
  setPaymentDialogOpen,
  searchInputRef,
  setSplitPaymentDialogOpen,
  setQrDialogOpen,
  getCurrentItems, // ✅ AGREGADO
}: AfipPaymentOptions) {
  // 🆕 MEJORA: Usar businessInfo desde el hook
  const { businessInfo } = useBusinessInfo(API_URL, appId);

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

  // Estados para pago mixto
  const [cashAmount, setCashAmount] = useState<string>("");
  const [secondPaymentMethod, setSecondPaymentMethod] =
    useState<string>("tarjeta");

  // Estados para QR
  const [qrData, setQrData] = useState<any>(null);
  const qrDataRef = useRef<any>(null);
  const updateQrData = (data: any) => {
    qrDataRef.current = data;
    setQrData(data);
  };
  const [paymentStatus, setPaymentStatus] = useState<string | null>(null);
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(
    null
  );
  const [manualQrPasswordDialogOpen, setManualQrPasswordDialogOpen] =
    useState(false);
  const [manualQrPassword, setManualQrPassword] = useState("");
  const [manualQrOrderDetails, setManualQrOrderDetails] =
    useState<ManualQrDetails | null>(null);

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

      let result;

      // Método 1: Usar window.printer (API específica para impresión)
      if (
        typeof window !== "undefined" &&
        (window as any).printer?.printAfipTicket
      ) {
        console.log("🖨️ Usando window.printer.printAfipTicket");
        result = await (window as any).printer.printAfipTicket(afipData);
        console.log("📝 Resultado detallado de printAfipTicket:", result);
      }
      // Método 2: Usar window.electron.ipcRenderer (API general)
      else if (
        typeof window !== "undefined" &&
        (window as any).electron?.ipcRenderer
      ) {
        console.log(
          "🖨️ Usando window.electron.ipcRenderer.invoke('print-afip-ticket')"
        );
        result = await (window as any).electron.ipcRenderer.invoke(
          "print-afip-ticket",
          afipData
        );
        console.log("📝 Resultado de print-afip-ticket:", result);
      }
      // Si ninguna API está disponible
      else {
        console.error("❌ Ninguna API de impresión AFIP disponible");
        throw new Error("API de Electron no disponible para impresión AFIP");
      }

      // 🆕 VALIDACIÓN MEJORADA: Verificar que la impresión fue realmente exitosa
      console.log("🔍 VALIDANDO RESULTADO DE IMPRESIÓN:");
      console.log("- Tipo de resultado:", typeof result);
      console.log("- Resultado completo:", result);

      // Verificar diferentes formatos de respuesta
      if (result === undefined || result === null) {
        console.warn("⚠️ Resultado vacío - posible problema de impresión");
        return {
          success: false,
          printerError: "Resultado vacío de la función de impresión",
          message: "No se recibió confirmación de impresión",
        };
      }

      // Si es un objeto, verificar la propiedad success
      if (typeof result === "object") {
        if (result.success === false) {
          console.error(
            "❌ Error reportado por la función de impresión:",
            result
          );
          return result;
        }

        // Verificar si hay alguna propiedad que indique error
        if (result.error || result.printerError) {
          console.error("❌ Error en propiedades del resultado:", result);
          return {
            success: false,
            printerError: result.error || result.printerError,
            message: result.message || "Error en impresión",
          };
        }
      }

      console.log("✅ Impresión AFIP aparentemente exitosa");
      return result;
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
    console.log("🧹 Reseteando estados del hook de procesamiento AFIP");
    setQrData(null);
    setPaymentStatus(null);
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);
    setManualQrPassword("");
    setManualQrPasswordDialogOpen(false);
    setIsManualPasswordSubmitting(false);
    setRoundedAmountDialogOpen(false);
    setExactPaymentDialogOpen(false);
    setPaidAmount(0);
    setChangeAmount(0);
    setCashAmount("");
    setSecondPaymentMethod("tarjeta");

    // ✅ TOAST CONTROL: Limpiar tracking de órdenes procesadas
    clearProcessedOrdersTracking();

    // Limpiar polling
    cleanupPolling();

    console.log("✅ Estados del hook de procesamiento AFIP reseteados");
  };

  const processAfipPayment = async (
    method: string,
    items: Product[],
    totalAmount?: number
  ) => {
    console.log("🔥🔥🔥 PROCESS AFIP PAYMENT: INICIANDO");
    console.log("🔥🔥🔥 STACK TRACE:", new Error().stack);
    console.log("🔥🔥🔥 method:", method);
    console.log("🔥🔥🔥 items count:", items.length);
    console.log("🔥🔥🔥 totalAmount:", totalAmount);
    console.log("🔥🔥🔥 ⚠️⚠️⚠️ ESTA FUNCIÓN CREA FACTURA INMEDIATAMENTE");
    console.log("🔥🔥🔥 ⚠️⚠️⚠️ NO DEBERÍA LLAMARSE PARA QR SIN CONFIRMAR PAGO");

    if (!user) {
      console.error("❌ Usuario no encontrado");
      toast.error("Debes iniciar sesión para realizar una factura");
      return;
    }

    const finalTotal = totalAmount || calculateTotal();

    // 🚦 PREVENIR PROCESOS DUPLICADOS Y MOSTRAR CARGA INSTANTÁNEA
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago AFIP en proceso");
      return;
    }

    // Marcar inmediatamente como procesando para feedback instantáneo
    setIsProcessingPayment(true);
    setSelectedPaymentMethod(method);

    // Obtener la info del negocio para los datos del ticket
    let fetchedBusinessInfo: any = null;
    try {
      fetchedBusinessInfo = await (
        await import("@/utils/businessHelpers")
      ).getBusinessInfo(API_URL, appId);
      console.log(
        "🏢 AFIP: businessInfo obtenido dentro de processAfipPayment",
        fetchedBusinessInfo
      );
    } catch (err) {
      console.warn(
        "⚠️ AFIP: No se pudo obtener businessInfo, usando valores por defecto",
        err
      );
    }

    // Si es efectivo y no viene de los diálogos de efectivo, manejar redondeo
    if (method === "efectivo" && !totalAmount) {
      console.log(
        "💰 AFIP: Efectivo detectado sin totalAmount, necesita redondeo"
      );
      throw new Error("Use handleAfipCashPayment para pagos en efectivo");
    }

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
          tipoFactura: 
            // 🆕 OBTENER TIPO DE FACTURA DESDE CONFIGURACIÓN AFIP
            fetchedBusinessInfo?.configuracionAfip?.tipoFactura ||
            businessInfo?.configuracionAfip?.tipoFactura ||
            "C", // Factura C por defecto para consumidores finales
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

      // 🆕 PREPARAR DATOS MEJORADO: Usar businessInfo como fuente principal
      const printData = {
        ...afipResult,
        metodoPago: method,
        items: orderItems,
        total: finalTotal,
        usuario: user.nombre || "Vendedor",
        fechaHora: new Date().toLocaleString("es-AR"),

        // 🔧 INFORMACIÓN DEL NEGOCIO: Priorizar businessInfo del contexto
        businessName:
          // 🆕 PRIORIDAD 1: Nombre del business
          fetchedBusinessInfo?.nombre ||
          fetchedBusinessInfo?.name ||
          businessInfo?.nombre ||
          businessInfo?.name ||
          // PRIORIDAD 2: Razón social del business
          fetchedBusinessInfo?.razonSocial ||
          businessInfo?.razonSocial ||
          // PRIORIDAD 3: Datos del resultado AFIP
          afipResult.business?.name ||
          afipResult.business?.razonSocial ||
          // PRIORIDAD 4: Datos del usuario
          user.business?.name ||
          user.business?.razonSocial ||
          user.nombre ||
          // FALLBACK
          "Comercio",

        razonSocial:
          // 🆕 PRIORIDAD 1: Configuración AFIP del business
          fetchedBusinessInfo?.configuracionAfip?.razonSocial ||
          businessInfo?.configuracionAfip?.razonSocial ||
          // PRIORIDAD 2: Razón social directa del business
          fetchedBusinessInfo?.razonSocial ||
          businessInfo?.razonSocial ||
          // PRIORIDAD 3: Nombre del business
          fetchedBusinessInfo?.nombre ||
          fetchedBusinessInfo?.name ||
          businessInfo?.nombre ||
          businessInfo?.name ||
          // PRIORIDAD 4: Datos del resultado AFIP
          afipResult.business?.razonSocial ||
          afipResult.business?.name ||
          afipResult.razonSocial ||
          afipResult.empresa?.razonSocial ||
          afipResult.empresa?.nombre ||
          // PRIORIDAD 5: Datos del usuario
          user.business?.razonSocial ||
          user.business?.name ||
          user.razonSocial ||
          user.empresa ||
          user.nombre ||
          // FALLBACK
          "Comercio",

        cuit:
          fetchedBusinessInfo?.cuit ||
          fetchedBusinessInfo?.CUIT ||
          afipResult.business?.cuit ||
          afipResult.cuit ||
          afipResult.empresa?.cuit ||
          user.business?.cuit ||
          user.cuit ||
          user.empresa?.cuit ||
          "00-00000000-0",

        condicionIva:
          fetchedBusinessInfo?.configuracionAfip?.condicionIva ||
          businessInfo?.configuracionAfip?.condicionIva ||
          fetchedBusinessInfo?.condicionIva ||
          afipResult.business?.condicionIva ||
          afipResult.condicionIva ||
          afipResult.empresa?.condicionIva ||
          user.business?.condicionIva ||
          user.condicionIva ||
          "Responsable Inscripto",

        // 🆕 INCLUIR CONFIGURACIÓN AFIP COMPLETA
        configuracionAfip:
          fetchedBusinessInfo?.configuracionAfip ||
          businessInfo?.configuracionAfip ||
          null,

        // 🏢 DIRECCIÓN: Priorizar sucursal activa del businessInfo
        direccion:
          businessInfo?.sucursalActiva?.ubicacion ||
          businessInfo?.direccion ||
          fetchedBusinessInfo?.sucursalActiva?.ubicacion ||
          fetchedBusinessInfo?.direccion ||
          afipResult.business?.sucursalActiva?.ubicacion ||
          afipResult.business?.ubicacion ||
          afipResult.direccion ||
          afipResult.empresa?.ubicacion ||
          user.business?.sucursalActiva?.ubicacion ||
          user.business?.ubicacion ||
          user.direccion ||
          "Dirección no configurada",

        // 📞 TELÉFONO: Incluir desde sucursal si está disponible
        telefono:
          businessInfo?.sucursalActiva?.telefono ||
          businessInfo?.telefono ||
          fetchedBusinessInfo?.sucursalActiva?.telefono ||
          fetchedBusinessInfo?.telefono ||
          afipResult.business?.sucursalActiva?.telefono ||
          afipResult.business?.telefono ||
          afipResult.telefono ||
          "",

        // Datos AFIP
        cae: afipResult.afip?.cae || afipResult.cae,
        fechaVtoCae: afipResult.afip?.fechaVtoCae || afipResult.fechaVtoCae,
        puntoVenta:
          afipResult.afip?.puntoVenta ||
          afipResult.puntoVenta ||
          afipResult.punto_venta ||
          "0001",
        numeroFactura:
          afipResult.afip?.numeroFactura ||
          afipResult.numeroFactura ||
          afipResult.numero_factura ||
          afipResult.numero,
        tipoFactura:
          afipResult.afip?.tipoFactura ||
          afipResult.tipoFactura ||
          afipResult.tipo_factura ||
          "FACTURA B",
        vendedor: user.nombre || user.name || "Vendedor",
        // Identificador real de la factura/orden para mostrar en el ticket
        idReal:
          afipResult.factura?.idReal ||
          afipResult.idReal ||
          (afipResult.factura && afipResult.factura.idReal) ||
          null,
        // Calcular descuento si aplica
        ...(method === "efectivo" &&
          originalAmount > finalTotal && {
            descuentoAplicado: originalAmount - finalTotal,
            totalOriginal: originalAmount,
          }),
      };

      console.log("📋 Datos preparados para impresión AFIP:", printData);

      // 🔍 DEBUG MEJORADO: Mostrar fuentes de datos
      console.log("🔍 DEBUG FUENTES DE DATOS:");
      console.log("- businessInfo:", businessInfo);
      console.log(
        "- businessInfo.sucursalActiva:",
        businessInfo?.sucursalActiva
      );
      console.log(
        "- businessInfo.configuracionAfip:",
        businessInfo?.configuracionAfip
      );
      console.log("- fetchedBusinessInfo:", fetchedBusinessInfo);
      console.log(
        "- fetchedBusinessInfo.configuracionAfip:",
        fetchedBusinessInfo?.configuracionAfip
      );
      console.log("- afipResult.business:", afipResult.business);
      console.log("- user:", user);

      // DEBUG: Verificar qué valores están siendo seleccionados en printData
      console.log("🎯 VALORES FINALES SELECCIONADOS:");
      console.log("- businessName final:", printData.businessName);
      console.log("- razonSocial final:", printData.razonSocial);
      console.log("- cuit final:", printData.cuit);
      console.log("- condicionIva final:", printData.condicionIva);
      console.log("- direccion final:", printData.direccion);
      console.log("- telefono final:", printData.telefono);
      console.log("- cae final:", printData.cae);
      console.log("- fechaVtoCae final:", printData.fechaVtoCae);
      console.log("- vendedor final:", printData.vendedor);

      // Simular el ticket AFIP antes de imprimir
      console.log("\n🎭 ====== SIMULACIÓN DEL TICKET AFIP ======");

      // Encabezado reducido (sin nombre grande)
      console.log(`Razón Social: ${printData.razonSocial}`);
      console.log(`CUIT: ${printData.cuit || "00-00000000-0"}`);
      console.log(
        `Condición IVA: ${printData.condicionIva || "Responsable Inscripto"}`
      );
      console.log(
        `Dirección: ${printData.direccion || "Dirección no configurada"}`
      );
      console.log(`-----------------------------`);
      console.log(``);
      console.log(
        `           ${(printData.tipoFactura || "FACTURA B").toUpperCase()}`
      );
      console.log(``);
      console.log(
        `Nro: ${printData.puntoVenta || "0001"}-${String(
          printData.numeroFactura || "1"
        ).padStart(8, "0")}`
      );
      console.log(`Fecha: ${new Date().toLocaleString("es-AR")}`);
      console.log(
        `Vendedor: ${printData.vendedor || printData.usuario || "N/A"}`
      );
      console.log(`-----------------------------`);
      console.log(`Cliente: Consumidor Final`);
      console.log(`Condición IVA: Consumidor Final`);
      console.log(`-----------------------------`);
      console.log(`PRODUCTO      CANT    PRECIO    TOTAL`);
      console.log(`-----------------------------`);

      // Mostrar productos
      const items = printData.items || [];
      let subtotalNeto = 0;
      let totalIva = 0;

      if (items && items.length > 0) {
        items.forEach((item: any) => {
          const nombre = (item.nombre || "").substring(0, 12).padEnd(12);
          const cantidad = Number(item.cantidad || 0)
            .toFixed(3)
            .padStart(8);
          const precio = `$${Number(
            item.precioHistorico || item.precio || 0
          ).toFixed(2)}`.padStart(8);
          const subtotal = `$${Number(item.subtotal || 0).toFixed(2)}`.padStart(
            8
          );

          console.log(`${nombre} ${cantidad} ${precio} ${subtotal}`);

          // Calcular subtotal neto (sin IVA) y IVA
          const itemSubtotal = Number(item.subtotal || 0);
          subtotalNeto += itemSubtotal / 1.21; // Asumiendo IVA 21%
          totalIva += itemSubtotal - itemSubtotal / 1.21;
        });
      } else {
        console.log("❌ No hay items en la factura AFIP");
      }

      console.log(`-----------------------------`);
      console.log(`Subtotal: $${subtotalNeto.toFixed(2)}`);
      console.log(`IVA (21%): $${totalIva.toFixed(2)}`);
      console.log(
        `                 TOTAL: $${Number(printData.total).toFixed(2)}`
      );
      console.log(`-----------------------------`);
      console.log(`     COMPROBANTE AUTORIZADO`);
      console.log(`        CAE: ${printData.cae || "NO DISPONIBLE"}`);

      // Fecha de vencimiento del CAE
      let fechaFormateada = new Date().toLocaleDateString("es-AR");
      if (printData.fechaVtoCae) {
        const fechaVto = printData.fechaVtoCae;
        if (fechaVto.length === 8) {
          // Formato YYYYMMDD de AFIP
          const year = fechaVto.substring(0, 4);
          const month = fechaVto.substring(4, 6);
          const day = fechaVto.substring(6, 8);
          fechaFormateada = `${day}/${month}/${year}`;
        } else {
          fechaFormateada = new Date(fechaVto).toLocaleDateString("es-AR");
        }
      }
      console.log(`    Fecha Vto CAE: ${fechaFormateada}`);
      console.log(``);
      console.log(`     ¡Gracias por su compra!`);
      console.log(`    Conserve este comprobante`);
      console.log(`🎭 ======= FIN SIMULACIÓN TICKET AFIP =======\n`);

      // Imprimir ticket AFIP
      const firstPrintResult = await handleAfipTicketPrinting(printData);

      // 🆕 MANEJO MEJORADO DE ERRORES DE IMPRESIÓN
      let printingFailed = false;
      if (firstPrintResult && firstPrintResult.success === false) {
        console.error("❌ Primera impresión AFIP falló:", firstPrintResult);
        printingFailed = true;

        // 🎯 TOAST CORTO Y CLARO (no lanzar excepción)
        toast.error("Error al imprimir factura AFIP", {
          description:
            "La factura se creó exitosamente pero no se pudo imprimir",
          duration: 4000,
        });
      }

      // 🆕 DOBLE IMPRESIÓN AFIP: Solo si la primera fue exitosa
      if (
        !printingFailed &&
        (businessInfo?.dobleImpresionEnabled === true ||
          fetchedBusinessInfo?.dobleImpresionEnabled === true)
      ) {
        console.log("🖨️🖨️ AFIP DOBLE IMPRESIÓN: Imprimiendo segunda copia...");
        try {
          const secondPrintResult = await handleAfipTicketPrinting(printData);

          // Verificar resultado de segunda impresión
          if (secondPrintResult && secondPrintResult.success === false) {
            console.error(
              "❌ Segunda impresión AFIP falló:",
              secondPrintResult
            );
            toast.warning(
              "Primera impresión exitosa, pero la segunda copia falló"
            );
          } else {
            console.log(
              "✅ AFIP DOBLE IMPRESIÓN: Segunda copia impresa exitosamente"
            );
          }
        } catch (error) {
          console.error(
            "❌ AFIP DOBLE IMPRESIÓN: Error en segunda copia:",
            error
          );
          toast.warning(
            "Primera impresión exitosa, pero la segunda copia falló"
          );
        }
      }

      // 🎯 MOSTRAR ÉXITO SIEMPRE (factura creada correctamente)
      if (!printingFailed) {
        toast.success(`Factura AFIP creada exitosamente`, {
          description: `CAE: ${afipResult.afip.cae}`,
          duration: 5000,
        });
      } else {
        // Si falló la impresión, mostrar que la factura se creó pero con problema de impresión
        toast.success(`Factura AFIP creada exitosamente`, {
          description: `CAE: ${afipResult.afip.cae} (Error de impresión)`,
          duration: 5000,
        });
      }

      // ✅ LIMPIAR CARRITO SIEMPRE (factura creada exitosamente)
      // Esperar un poco para que el usuario vea el mensaje
      await new Promise((resolve) => setTimeout(resolve, 1500));

      // Limpiar carrito y estados SIEMPRE
      clearCart();
      resetPaymentState();
      setPaymentDialogOpen(false);

      // Devolver el foco al input de búsqueda
      setTimeout(() => {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
        }
      }, 100);
    } catch (error: any) {
      console.error("❌ Error al procesar factura AFIP:", error);

      // 🎯 TOAST DE ERROR MÁS CORTO
      if (
        error.message?.includes("Failed to copy file to printer") ||
        error.message?.includes("Error de impresión")
      ) {
        toast.error("Error al crear factura AFIP", {
          description: "Problema de conexión con la impresora",
          duration: 4000,
        });
      } else {
        toast.error("Error al crear factura AFIP", {
          description:
            error.message?.substring(0, 100) + "..." || "Error desconocido",
          duration: 4000,
        });
      }

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

  // Helper para abrir diálogo de pago mixto
  const handleSplitPayment = () => {
    console.log("🔄 AFIP handleSplitPayment llamado");

    if (!user) {
      toast.error("Debes iniciar sesión para realizar una factura");
      return;
    }

    // Evitar duplicados
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un proceso AFIP activo");
      return;
    }

    setSelectedPaymentMethod("split");
    setCashAmount("");
    setSecondPaymentMethod("tarjeta");

    if (setSplitPaymentDialogOpen) {
      setSplitPaymentDialogOpen(true);
    } else {
      console.error(
        "❌ setSplitPaymentDialogOpen no está definido en opciones AFIP"
      );
    }
  };

  // Procesar pago mixto AFIP
  const processSplitPayment = async (
    items: Product[],
    totalAmount: number,
    businessInfo?: any
  ) => {
    console.log("🧾 SPLIT PAYMENT: Iniciando processSplitPayment");
    console.log("🧾 SPLIT PAYMENT: cashAmount:", cashAmount);
    console.log("🧾 SPLIT PAYMENT: secondPaymentMethod:", secondPaymentMethod);

    const cashAmountValue = parseFloat(cashAmount);

    if (
      isNaN(cashAmountValue) ||
      cashAmountValue <= 0 ||
      cashAmountValue >= totalAmount
    ) {
      toast.error("El monto en efectivo no es válido.");
      return;
    }

    setIsProcessingPayment(true);
    setSelectedPaymentMethod("split");

    if (secondPaymentMethod === "qr") {
      console.log(
        "🧾 SPLIT PAYMENT: Proceso de pago mixto AFIP iniciado con QR."
      );
      console.log("🔍 CRÍTICO: Debería SOLO generar QR, NO crear factura");
      const qrAmount = totalAmount - cashAmountValue;
      await generateAfipSplitQRPayment(cashAmountValue, qrAmount, items);
    } else {
      // Flujo para tarjeta de crédito/débito
      console.log(
        "🧾 SPLIT PAYMENT: Procesando pago mixto AFIP con tarjeta..."
      );
      console.log("🔍 CRÍTICO: Tarjeta puede crear factura inmediatamente");
      await processAfipPayment("split", items);
      setIsProcessingPayment(false); // Limpiar solo si no es QR
    }
  };

  // 🆕 NUEVA FUNCIÓN: Manejar QR con AFIP (generar QR primero, factura después)
  const handleAfipQrPayment = async (items: Product[]) => {
    console.log("🔥🔥🔥 HANDLE AFIP QR PAYMENT: INICIANDO");
    console.log("🔥🔥🔥 STACK TRACE:", new Error().stack);
    console.log("🔥🔥🔥 items count:", items.length);
    console.log("🧾 Iniciando generación de QR para pago normal con AFIP...");
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

      const total = calculateTotal();

      const orderData = {
        monto: Number(total.toFixed(2)),
        descripcion: `Compra de ${orderItems.length} productos con Factura AFIP`,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        items: orderItems,
        requiresAfipInvoice: true,
        isSplitPayment: false,
        cashAmount: 0,
        // TODO: Implementar la captura de datos del cliente para facturas que no son a Consumidor Final
        afipData: null,
      };

      console.log("🔥🔥🔥 HANDLE AFIP QR - PAYLOAD COMPLETO:");
      console.log("🔥🔥🔥", JSON.stringify(orderData, null, 2));
      console.log("🔥🔥🔥 requiresAfipInvoice:", orderData.requiresAfipInvoice);
      console.log("🔥🔥🔥 isSplitPayment:", orderData.isSplitPayment);
      console.log("🔥🔥🔥 cashAmount:", orderData.cashAmount);
      console.log("🔥🔥🔥 ATENCIÓN: Enviando a /api/mercadopago/generate-qr");
      console.log("🔥🔥🔥 ESTE ENDPOINT SOLO DEBERÍA CREAR QR, NO FACTURA");
      console.log("📲 Generando QR (F2 - Con AFIP) con payload:", orderData);

      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };
      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      console.log("🔥🔥🔥 HANDLE AFIP QR - ENVIANDO REQUEST...");
      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers,
        body: JSON.stringify(orderData),
      });

      console.log(
        "🔥🔥🔥 HANDLE AFIP QR - RESPUESTA RECIBIDA, status:",
        response.status
      );
      console.log("🔥🔥🔥 HANDLE AFIP QR - RESPUESTA ok:", response.ok);

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        console.error("🔥🔥🔥 HANDLE AFIP QR - ERROR EN RESPUESTA:", errorData);
        throw new Error(
          errorData.message || "Error al generar el QR para pago AFIP"
        );
      }

      const result = await response.json();
      console.log("🔥🔥🔥 HANDLE AFIP QR - RESULTADO COMPLETO:");
      console.log("🔥🔥🔥", JSON.stringify(result, null, 2));
      console.log("🧾 Respuesta del backend (QR AFIP):", result);

      if (result.qrData && result.id) {
        // ✅ CORRECCIÓN: Incluir items originales para impresión AFIP
        const qrDataWithAmount = {
          ...result,
          monto: total,
          items: orderItems, // ✅ CRITICAL FIX: Incluir items originales para impresión AFIP
        };
        updateQrData(qrDataWithAmount);
        console.log("🧾✅ Estado QR actualizado.");
        setPaymentStatus("PENDIENTE");

        if (setQrDialogOpen) {
          console.log("🧾✅ Abriendo diálogo QR...");
          setQrDialogOpen(true);
        }
        if (setPaymentDialogOpen) {
          console.log("🧾✅ Cerrando diálogo de pago principal...");
          setPaymentDialogOpen(false);
        }

        console.log(`🧾✅ Iniciando polling para orderId: ${result.id}`);
        console.log("🔍 CRÍTICO: Solo debería hacer polling, NO crear factura");
        startPaymentStatusPolling(String(result.id), true);

        // ✅ CORREGIDO: Retornar los datos del QR
        return qrDataWithAmount;
      } else {
        console.error("🔥🔥🔥 HANDLE AFIP QR - DATOS FALTANTES:");
        console.error("🔥🔥🔥 result.qrData:", result.qrData);
        console.error("🔥🔥🔥 result.id:", result.id);
        throw new Error("La respuesta del backend no incluyó 'qrData' o 'id'.");
      }
    } catch (error: any) {
      console.error("🔥🔥🔥 HANDLE AFIP QR - ERROR:", error);
      console.error("🔥🔥🔥 HANDLE AFIP QR - ERROR STACK:", error.stack);
      console.error("❌ Error en handleAfipQrPayment:", error);
      toast.error(error.message);
      resetPaymentState();
      // ✅ CORREGIDO: Retornar null en caso de error
      return null;
    }
  };

  // ✅ FUNCIÓN ELIMINADA: createAfipInvoiceAfterPayment ya no es necesaria
  // El backend ahora crea automáticamente la factura AFIP cuando el pago se confirma
  // No necesitamos crear facturas manualmente desde el frontend

  // 🆕 IMPLEMENTACIÓN CORREGIDA: Generar QR para pago mixto AFIP
  const generateAfipSplitQRPayment = async (
    cashAmountValue: number,
    qrAmount: number,
    items: Product[]
  ) => {
    console.log("🔥🔥🔥 GENERATE AFIP SPLIT QR: INICIANDO");
    console.log("🔥🔥🔥 STACK TRACE:", new Error().stack);
    console.log("🔥🔥🔥 cashAmountValue:", cashAmountValue);
    console.log("🔥🔥🔥 qrAmount:", qrAmount);
    console.log("🔥🔥🔥 items count:", items.length);

    if (!user) {
      console.error("❌ Usuario no encontrado");
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    console.log("🧾🔁 Generando QR para pago mixto AFIP...", {
      cashAmountValue,
      qrAmount,
    });

    try {
      const orderItems = items.map((item) => ({
        productoId: item.id,
        nombre: item.name,
        cantidad: item.quantity,
        subtotal: Number(item.subtotal.toFixed(2)),
        precioHistorico: item.pricePerUnit,
        costo: Number(item.costo),
      }));

      const totalAmount = cashAmountValue + qrAmount;

      const orderData = {
        monto: Number(totalAmount.toFixed(2)),
        descripcion: `Pago mixto AFIP: QR $${qrAmount.toFixed(
          2
        )} + Efectivo $${cashAmountValue.toFixed(2)}`,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        items: orderItems,
        // ✅ CRÍTICO: Estos parámetros controlan el flujo
        requiresAfipInvoice: true,
        isSplitPayment: true,
        cashAmount: cashAmountValue,
        afipData: null, // Consumidor final
      };

      console.log("🔥🔥🔥 PAYLOAD COMPLETO A ENVIAR:");
      console.log("🔥🔥🔥", JSON.stringify(orderData, null, 2));
      console.log("🔥🔥🔥 requiresAfipInvoice:", orderData.requiresAfipInvoice);
      console.log("🔥🔥🔥 isSplitPayment:", orderData.isSplitPayment);
      console.log("🔥🔥🔥 cashAmount:", orderData.cashAmount);
      console.log("🔥🔥🔥 ATENCIÓN: Enviando a /api/mercadopago/generate-qr");
      console.log("🔥🔥🔥 ESTE ENDPOINT SOLO DEBERÍA CREAR QR, NO FACTURA");

      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };
      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      // ✅ CORRECCIÓN: Mostrar toast de loading como en el QR normal
      toast.loading("Generando código QR para pago mixto AFIP...", {
        id: "qr-loading",
      });

      console.log("🔥🔥🔥 ENVIANDO REQUEST...");
      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers,
        body: JSON.stringify(orderData),
      });

      console.log("🔥🔥🔥 RESPUESTA RECIBIDA, status:", response.status);
      console.log("🔥🔥🔥 RESPUESTA ok:", response.ok);

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        console.error("🔥🔥🔥 ERROR EN RESPUESTA:", errorData);
        throw new Error(
          errorData.message || "Error al generar el QR para pago mixto AFIP"
        );
      }

      const result = await response.json();
      console.log("🔥🔥🔥 RESULTADO COMPLETO:");
      console.log("🔥🔥🔥", JSON.stringify(result, null, 2));
      console.log("🧾🔁 Respuesta del backend (QR mixto AFIP):", result);

      if (result.qrData && result.id) {
        // ✅ CORRECCIÓN: Limpiar toast de loading ANTES de continuar
        toast.dismiss("qr-loading");

        console.log("🧾🔁✅ Estado QR mixto actualizado.");
        console.log("🔍 CRÍTICO: QR creado, NO debería crear factura aún");

        // ✅ CORRECCIÓN: Incluir items originales para impresión AFIP mixta
        const qrDataWithAmount = {
          ...result,
          monto: qrAmount,
          items: orderItems, // ✅ CRITICAL FIX: Incluir items originales para impresión AFIP mixta
        };
        updateQrData(qrDataWithAmount);
        setPaymentStatus("PENDIENTE");

        if (setQrDialogOpen) {
          console.log("🧾🔁✅ Abriendo diálogo de QR...");
          setQrDialogOpen(true);
        }
        if (setSplitPaymentDialogOpen) {
          console.log("🧾🔁✅ Cerrando diálogo de pago mixto...");
          setSplitPaymentDialogOpen(false);
        }

        console.log(`🧾🔁✅ Iniciando polling para orderId: ${result.id}`);
        console.log("🔍 CRÍTICO: Solo debería hacer polling, NO crear factura");
        startPaymentStatusPolling(String(result.id), true); // true for AFIP flow
      } else {
        console.error("🔥🔥🔥 DATOS FALTANTES EN RESPUESTA:");
        console.error("🔥🔥🔥 result.qrData:", result.qrData);
        console.error("🔥🔥🔥 result.id:", result.id);
        throw new Error("La respuesta del backend no incluyó 'qrData' o 'id'.");
      }
    } catch (error: any) {
      console.error("🔥🔥🔥 ERROR EN generateAfipSplitQRPayment:", error);
      console.error("🔥🔥🔥 ERROR STACK:", error.stack);
      // ✅ CORRECCIÓN: Limpiar toast de loading en caso de error
      toast.dismiss("qr-loading");
      toast.error(error.message);
      resetPaymentState();
    }
  };

  // 🆕 IMPLEMENTACIÓN CORREGIDA: Polling que verifica estado sin crear facturas prematuramente
  const startPaymentStatusPolling = (
    orderId: string,
    isAfip: boolean = false
  ) => {
    console.log("🔄 AFIP POLLING: Iniciando polling para orden:", orderId);
    console.log("🔄 AFIP POLLING: isAfip:", isAfip);
    console.log(
      "🔍 CRÍTICO: Polling SOLO verifica estado, factura se crea automáticamente en backend"
    );

    // ✅ CRÍTICO: Limpiar cualquier polling anterior ANTES de iniciar uno nuevo
    cleanupPolling();

    // ✅ CRÍTICO: Verificar si la orden ya fue procesada ANTES de iniciar polling
    if (isOrderAlreadyProcessed(orderId)) {
      console.log(
        `🛡️ AFIP POLLING CONTROL: Orden ${orderId} ya fue procesada, saltando polling`
      );
      return;
    }

    const POLLING_INTERVAL = 2000; // 2 segundos como especifica el backend
    const MAX_POLLING_TIME = 5 * 60 * 1000; // 5 minutos máximo

    let pollCount = 0;
    const maxPolls = Math.floor(MAX_POLLING_TIME / POLLING_INTERVAL);

    const interval = setInterval(async () => {
      pollCount++;

      // ✅ CRÍTICO: Verificar si la orden ya fue procesada en cada llamada
      if (isOrderAlreadyProcessed(orderId)) {
        console.log(
          `🛡️ AFIP POLLING CONTROL: Orden ${orderId} ya fue procesada, deteniendo polling`
        );
        clearInterval(interval);
        setPollingInterval(null);
        return;
      }

      if (pollCount > maxPolls) {
        console.log(
          "🔄 AFIP POLLING: Tiempo máximo excedido, finalizando polling."
        );
        clearInterval(interval);
        setPollingInterval(null);
        toast.error(
          "Tiempo de espera agotado. Verifique manualmente el estado del pago."
        );
        resetPaymentState();
        return;
      }

      try {
        // ✅ CORRECCIÓN: Usar query parameters como especifica el backend
        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status/${orderId}`,
          {
            method: "GET",
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
          }
        );

        if (!response.ok) {
          // ✅ CRÍTICO: Si es 404, la orden ya no existe (probablemente completada)
          if (response.status === 404) {
            console.log(
              `🛡️ AFIP POLLING CONTROL: Orden ${orderId} no encontrada (404), probablemente ya completada`
            );
            clearInterval(interval);
            setPollingInterval(null);
            return;
          }

          const errorData = await response.json().catch(() => ({}));
          console.error("🔄 AFIP POLLING: Error en respuesta:", errorData);

          // Si es 404, podría ser que el endpoint no existe, intentar con query params
          if (response.status === 404) {
            console.log("🔄 AFIP POLLING: Intentando con query parameters...");
            const altResponse = await fetch(
              `${API_URL}/api/mercadopago/check-status/${orderId}`,
              {
                method: "GET",
                headers: {
                  "Content-Type": "application/json",
                  ...(appId && { "X-App-ID": appId }),
                },
              }
            );

            if (altResponse.ok) {
              const altStatusData = await altResponse.json();
              console.log(
                "🔄 AFIP POLLING: Estado con query params:",
                altStatusData
              );
              await handlePollingResponse(
                altStatusData,
                orderId,
                interval,
                isAfip
              );
              return;
            }
          }

          throw new Error(`Error en polling: ${response.status}`);
        }

        const statusData = await response.json();
        console.log(
          "🔄 AFIP POLLING: Estado actualizado para orden:",
          orderId,
          statusData
        );

        await handlePollingResponse(statusData, orderId, interval, isAfip);
      } catch (error: any) {
        console.error("🔄 AFIP POLLING: Error en polling:", error);

        // ✅ CRÍTICO: Si es error 404, la orden ya no existe
        if (
          error.message?.includes("404") ||
          error.message?.includes("Not Found")
        ) {
          console.log(
            `🛡️ AFIP POLLING CONTROL: Orden ${orderId} no encontrada, deteniendo polling`
          );
          clearInterval(interval);
          setPollingInterval(null);
          return;
        }

        // Si hay muchos errores consecutivos, detener el polling
        if (pollCount > 10) {
          console.log(
            "🔄 AFIP POLLING: Demasiados errores consecutivos, deteniendo polling"
          );
          clearInterval(interval);
          setPollingInterval(null);
          toast.error(
            "Error al verificar el estado del pago. Verifique manualmente."
          );
          resetPaymentState();
        }
      }
    }, POLLING_INTERVAL);

    setPollingInterval(interval);
    console.log(
      "🔄 AFIP POLLING: Polling iniciado con intervalo:",
      POLLING_INTERVAL,
      "ms"
    );
  };

  // ✅ NUEVA FUNCIÓN: Manejar respuesta de polling de manera centralizada
  const handlePollingResponse = async (
    statusData: any,
    orderId: string,
    interval: NodeJS.Timeout,
    isAfip: boolean
  ) => {
    const status = statusData.status?.toLowerCase();

    if (status === "completed" || status === "approved") {
      console.log(
        "🔄 AFIP POLLING: ✅ Pago completado, verificando factura AFIP..."
      );
      clearInterval(interval);
      setPollingInterval(null);

      // ✅ TOAST CONTROL: Verificar si ya se procesó esta orden
      if (isOrderAlreadyProcessed(orderId)) {
        console.log(
          `🛡️ TOAST CONTROL AFIP: Orden ${orderId} ya fue procesada en polling, saltando`
        );
        return;
      }

      // ✅ TOAST CONTROL: Marcar como procesada ANTES de continuar
      markOrderAsProcessed(orderId);

      // ✅ CRÍTICO: Mostrar toast de éxito ANTES de procesar
      toast.success("¡Pago QR completado exitosamente!");

      // ✅ FLUJO CORRECTO: El backend ya debería haber creado la factura AFIP automáticamente
      if (statusData.afipInvoice?.created) {
        console.log(
          "🔄 AFIP POLLING: ✅ Factura AFIP creada automáticamente:",
          statusData.afipInvoice
        );

        // ✅ CRÍTICO: Mostrar toast específico de factura AFIP
        const facturaInfo = statusData.afipInvoice;
        if (facturaInfo.numero) {
          toast.success(
            `Factura AFIP N° ${facturaInfo.numero} generada automáticamente`
          );
        } else {
          toast.success("Factura AFIP generada automáticamente");
        }

        // ✅ CRÍTICO: Intentar imprimir la factura AFIP
        try {
          console.log("🖨️ AFIP POLLING: Iniciando impresión de factura AFIP");
          const printingToastId = toast.loading("Imprimiendo factura AFIP...");

          const printSuccess = await handleAfipTicketPrinting(
            statusData.afipInvoice
          );

          toast.dismiss(printingToastId);

          if (printSuccess) {
            toast.success("Factura AFIP impresa correctamente");
          } else {
            toast.error("Error al imprimir la factura AFIP");
          }
        } catch (printError) {
          console.error("❌ Error al imprimir factura AFIP:", printError);
          toast.error("Error al imprimir la factura AFIP");
        }

        if (setQrDialogOpen) {
          setQrDialogOpen(false);
        }

        // ✅ CRÍTICO: Limpiar carrito y estados SOLO después de que todo esté completo
        setTimeout(() => {
          clearCart();
          resetPaymentState();
          console.log("🛒 Carrito limpiado después de pago QR AFIP exitoso");

          // Enfocar input de búsqueda
          if (searchInputRef?.current) {
            searchInputRef.current.focus();
          }
        }, 2000);
      } else {
        console.log(
          "🔄 AFIP POLLING: ⚠️ Pago completado pero factura AFIP no encontrada"
        );

        // ✅ CRÍTICO: Mostrar toast de pago completado aunque no haya factura
        toast.success("¡Pago QR completado! Verificando factura AFIP...");

        // ✅ MEJORA: Solo intentar una vez más si no fue pago manual
        if (!isManualPasswordSubmitting) {
          setTimeout(() => {
            checkForAfipInvoice(orderId);
          }, 3000);
        }
      }
    } else if (
      status === "failed" ||
      status === "cancelled" ||
      status === "rejected"
    ) {
      console.log("🔄 AFIP POLLING: ❌ Pago cancelado o rechazado");
      clearInterval(interval);
      setPollingInterval(null);
      toast.error("El pago ha sido cancelado o rechazado");

      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }
      resetPaymentState();
    } else {
      console.log(
        "🔄 AFIP POLLING: ⏳ Pago aún pendiente, continuando polling..."
      );
    }
  };

  // ✅ NUEVA FUNCIÓN: Verificar factura AFIP después de confirmación de pago
  const checkForAfipInvoice = async (orderId: string) => {
    try {
      console.log(
        "🔍 AFIP INVOICE CHECK: Verificando factura para orden:",
        orderId
      );

      const response = await fetch(
        `${API_URL}/api/mercadopago/check-status/${orderId}`,
        {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
            ...(appId && { "X-App-ID": appId }),
          },
        }
      );

      if (response.ok) {
        const statusData = await response.json();

        if (statusData.afipInvoice?.created) {
          console.log(
            "🔍 AFIP INVOICE CHECK: ✅ Factura AFIP encontrada:",
            statusData.afipInvoice
          );
          // ✅ MEJORA: No mostrar toast aquí para evitar duplicados
          // El toast principal ya se mostró en handlePollingResponse o handleManualQrPasswordSubmit
          console.log(
            "🔍 AFIP INVOICE CHECK: Factura AFIP confirmada (toast ya mostrado)"
          );
        } else {
          console.log(
            "🔍 AFIP INVOICE CHECK: ⚠️ Factura AFIP aún no disponible"
          );
          // ✅ MEJORA: Solo mostrar warning si realmente necesario
          // toast.warning("Pago completado. Generando factura AFIP...");
        }
      } else {
        console.log(
          "🔍 AFIP INVOICE CHECK: Orden no encontrada (probablemente ya procesada)"
        );
      }

      // Limpiar estados independientemente del resultado
      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }
      clearCart();
      resetPaymentState();

      // Enfocar input de búsqueda
      setTimeout(() => {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
        }
      }, 100);
    } catch (error: any) {
      console.error("🔍 AFIP INVOICE CHECK: Error:", error);
      // ✅ MEJORA: No mostrar error al usuario ya que el pago fue exitoso
      // El error aquí es solo de verificación, no afecta el resultado del pago
      console.log(
        "🔍 AFIP INVOICE CHECK: Error en verificación pero pago exitoso"
      );

      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }
      clearCart();
      resetPaymentState();

      // Enfocar input de búsqueda
      setTimeout(() => {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
        }
      }, 100);
    }
  };

  const cleanupPolling = () => {
    console.log("🧹 AFIP: Limpiando polling y estados");

    if (pollingInterval) {
      console.log(
        "🧹 AFIP: Limpiando intervalo de polling con ID:",
        pollingInterval
      );
      clearInterval(pollingInterval);
      setPollingInterval(null);
    }

    // ✅ CRÍTICO: Limpiar también el tracking de órdenes procesadas
    clearProcessedOrdersTracking();

    console.log("🧹 AFIP: Polling y estados limpiados");
  };

  // ✅ NUEVO: Estado para proteger contra envíos múltiples de contraseña manual
  const [isManualPasswordSubmitting, setIsManualPasswordSubmitting] =
    useState(false);

  // ✅ NUEVO: Función para manejar completado manual con contraseña (igual que en hook normal)
  const handleManualQrPasswordSubmit = async () => {
    // ✅ PROTECCIÓN: Evitar envíos múltiples
    if (isManualPasswordSubmitting) {
      console.log("🛡️ AFIP: Ignorando envío múltiple de contraseña manual");
      return;
    }

    if (!qrData?.orderId) {
      toast.error("No hay una orden QR pendiente para completar.");
      return;
    }

    const orderId = qrData.orderId;
    console.log(
      `🔧 AFIP: Completando manualmente orden ${orderId} con requiresAfipInvoice: true`
    );

    // ✅ TOAST CONTROL: Verificar si ya se procesó esta orden
    if (isOrderAlreadyProcessed(orderId)) {
      console.log(
        `🛡️ TOAST CONTROL AFIP: Orden ${orderId} ya fue procesada manualmente, saltando`
      );
      return;
    }

    // ✅ CORRECCIÓN: Limpiar polling INMEDIATAMENTE para evitar 404s
    console.log("🧹 AFIP: Limpiando polling antes del pago manual");
    cleanupPolling();

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
        requiresAfipInvoice: true, // ✅ SIEMPRE true para AFIP
      };

      console.log("🔧 AFIP: Enviando a /manual-complete:", body);

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
      console.log("✅ AFIP: Pago manual completado:", result);

      // ✅ TOAST CONTROL: Marcar como procesada ANTES de mostrar toasts
      markOrderAsProcessed(orderId);

      // ✅ CORRECCIÓN: NO cerrar diálogos inmediatamente, esperar sincronización
      setManualQrPasswordDialogOpen(false);
      setManualQrPassword("");

      // ✅ PASO 1: Mostrar loading para procesamiento final
      const processingToastId = toast.loading("Finalizando pago AFIP...");

      try {
        // ✅ PASO 2: Simular breve procesamiento para mejor UX
        await new Promise((resolve) => setTimeout(resolve, 500));

        // ✅ PASO 3: Mostrar éxito específico para AFIP
        toast.dismiss(processingToastId);

        if (result.success && result.facturaId && result.cae) {
          toast.success(
            `¡Pago completado! Factura AFIP N° ${result.numero} generada exitosamente`
          );
        } else {
          toast.success("¡Pago completado y factura AFIP generada!");
        }
      } catch (processingError) {
        console.error("❌ Error en procesamiento final AFIP:", processingError);
        toast.dismiss(processingToastId);
        toast.success("¡Pago completado y factura AFIP generada!");
      }

      // ✅ PASO 4: AHORA SÍ limpiar todo después del procesamiento
      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }

      // Limpiar carrito y estados
      clearCart();
      resetPaymentState();

      // Enfocar input de búsqueda
      setTimeout(() => {
        if (searchInputRef?.current) {
          searchInputRef.current.focus();
        }
      }, 100);
    } catch (error: any) {
      console.error("❌ AFIP: Error en handleManualQrPasswordSubmit:", error);
      toast.error(`Error al completar el pago: ${error.message}`);
      // ✅ MANTENER diálogos abiertos en caso de error para que el usuario pueda reintentar
      // NO cerrar setManualQrPasswordDialogOpen ni setQrDialogOpen
    } finally {
      // ✅ PROTECCIÓN: Siempre limpiar el estado de loading
      setIsManualPasswordSubmitting(false);
    }
  };

  // ✅ FUNCIÓN ELIMINADA: finalizeAfipPayment ya no es necesaria
  // El backend ahora crea automáticamente la factura AFIP cuando el pago se confirma
  // Solo necesitamos verificar que la factura fue creada en handlePollingResponse

  // Agregar funciones para polling, finalize, etc. similares a usePaymentProcessing pero adaptadas para AFIP (imprimir ticket AFIP al final)

  // ✅ NUEVO: Control de toasts para evitar duplicados
  const [processedOrders] = useState<Set<string>>(new Set());

  const isOrderAlreadyProcessed = (orderId: string): boolean => {
    return processedOrders.has(orderId);
  };

  const markOrderAsProcessed = (orderId: string): void => {
    processedOrders.add(orderId);
    console.log(
      `✅ TOAST CONTROL AFIP: Orden ${orderId} marcada como procesada`
    );
  };

  const clearProcessedOrdersTracking = () => {
    processedOrders.clear();
    console.log(
      "🧹 TOAST CONTROL AFIP: Tracking de órdenes procesadas limpiado"
    );
  };

  // Estado para datos del QR

  return {
    processAfipPayment,
    handleAfipCashPayment,
    confirmAfipExactPayment,
    // NUEVO: pago mixto
    handleSplitPayment,
    processSplitPayment,
    // Estados
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
    // NUEVO: estados de pago mixto
    cashAmount,
    setCashAmount,
    secondPaymentMethod,
    setSecondPaymentMethod,
    // NUEVA FUNCIÓN: Manejar QR con AFIP (generar QR primero, factura después)
    handleAfipQrPayment,
    // Estados de QR
    qrData,
    updateQrData,
    paymentStatus,
    setPaymentStatus,
    pollingInterval,
    setPollingInterval,
    manualQrPasswordDialogOpen,
    setManualQrPasswordDialogOpen,
    manualQrPassword,
    setManualQrPassword,
    manualQrOrderDetails,
    setManualQrOrderDetails,
    // NUEVA FUNCIÓN: Completado manual con contraseña
    handleManualQrPasswordSubmit,
    // NUEVO ESTADO: Loading para completado manual con contraseña
    isManualPasswordSubmitting,
    // Función de limpieza
    cleanupPolling,
    // ✅ NUEVO: Control de toasts para evitar duplicados
    clearProcessedOrdersTracking,
  };
}
