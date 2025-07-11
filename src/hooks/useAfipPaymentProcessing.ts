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
    console.log(
      "🧹 AFIP RESET: Limpiando estados del procesador de pagos AFIP"
    );
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);
    // Limpiar también estados de efectivo y mixto
    setRoundedAmountDialogOpen(false);
    setOriginalAmount(0);
    setRoundedAmount(0);
    setApplyingDiscount(false);
    setExactPaymentDialogOpen(false);
    setPaidAmount(0);
    setChangeAmount(0);
    // NUEVO: limpiar estados mixtos
    setCashAmount("");
    setSecondPaymentMethod("tarjeta");
    // Limpiar estados de QR
    setQrData(null);
    setPaymentStatus(null);
    setPollingInterval(null);
    setManualQrPasswordDialogOpen(false);
    setManualQrPassword("");
    setManualQrOrderDetails(null);
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
          fetchedBusinessInfo?.nombre ||
          fetchedBusinessInfo?.name ||
          fetchedBusinessInfo?.razonSocial ||
          afipResult.business?.name ||
          afipResult.business?.razonSocial ||
          user.business?.name ||
          user.business?.razonSocial ||
          user.nombre ||
          "Comercio",

        razonSocial:
          fetchedBusinessInfo?.razonSocial ||
          fetchedBusinessInfo?.nombre ||
          fetchedBusinessInfo?.name ||
          afipResult.business?.razonSocial ||
          afipResult.business?.name ||
          afipResult.razonSocial ||
          afipResult.empresa?.razonSocial ||
          afipResult.empresa?.nombre ||
          user.business?.razonSocial ||
          user.business?.name ||
          user.razonSocial ||
          user.empresa ||
          user.nombre ||
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
          fetchedBusinessInfo?.condicionIva ||
          afipResult.business?.condicionIva ||
          afipResult.condicionIva ||
          afipResult.empresa?.condicionIva ||
          user.business?.condicionIva ||
          user.condicionIva ||
          "Responsable Inscripto",

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
        const qrDataWithAmount = {
          ...result,
          monto: total,
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

  // 🆕 NUEVA FUNCIÓN: Crear factura AFIP después de confirmación de pago
  const createAfipInvoiceAfterPayment = async (
    orderId: string,
    items: Product[],
    paymentData?: any
  ) => {
    console.log(
      "🔥 CREAR FACTURA AFIP: Iniciando createAfipInvoiceAfterPayment"
    );
    console.log("🔥 CREAR FACTURA AFIP: orderId:", orderId);
    console.log("🔥 CREAR FACTURA AFIP: paymentData:", paymentData);
    console.log("🔥 CREAR FACTURA AFIP: items count:", items.length);
    console.log("🔥 CREAR FACTURA AFIP: Stack trace:", new Error().stack);

    if (!user) {
      console.error("❌ CREAR FACTURA AFIP: Usuario no encontrado");
      toast.error("Error: Usuario no encontrado para crear la factura.");
      return;
    }

    try {
      console.log("🔥 CREAR FACTURA AFIP: Preparando datos...");
      const businessName = await getBusinessName();
      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };
      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      const facturaData = {
        orderId,
        vendedor: user.nombre,
        businessName,
        items: items.map((item) => ({
          productoId: item.id,
          nombre: item.name,
          cantidad: item.quantity,
          subtotal: Number(item.subtotal.toFixed(2)),
          precioHistorico: item.pricePerUnit,
          costo: Number(item.costo),
        })),
        paymentData,
      };

      console.log("🔥 CREAR FACTURA AFIP: Enviando a /api/facturas/crear-afip");
      console.log("🔥 CREAR FACTURA AFIP: Datos:", facturaData);

      const response = await fetch(`${API_URL}/api/facturas/crear-afip`, {
        method: "POST",
        headers,
        body: JSON.stringify(facturaData),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || "Error al crear la factura AFIP");
      }

      const result = await response.json();
      console.log(
        "✅ CREAR FACTURA AFIP: Factura creada exitosamente:",
        result
      );

      // Limpiar estados solo después del éxito
      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }
      clearCart();
      resetPaymentState();

      toast.success("Factura AFIP creada y ticket impreso exitosamente");
    } catch (error: any) {
      console.error("❌ CREAR FACTURA AFIP: Error:", error);
      console.error("❌ CREAR FACTURA AFIP: Stack:", error.stack);
      toast.error(`Error al crear factura AFIP: ${error.message}`);
      // No limpiar aquí para que el usuario pueda reintentar
    }
  };

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
        console.log("🧾🔁✅ Estado QR mixto actualizado.");
        console.log("🔍 CRÍTICO: QR creado, NO debería crear factura aún");

        const qrDataWithAmount = {
          ...result,
          monto: qrAmount,
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
      console.error("❌ Error en generateAfipSplitQRPayment:", error);
      toast.error(error.message);
      resetPaymentState();
    }
  };

  // 🆕 AGREGAR: Funciones de polling similares a usePaymentProcessing
  const startPaymentStatusPolling = (
    orderId: string,
    isAfip: boolean = false
  ) => {
    console.log("🔄 POLLING: Iniciando polling para orden:", orderId);
    console.log("🔄 POLLING: isAfip:", isAfip);
    console.log(
      "🔍 CRÍTICO: Polling debería SOLO verificar estado, NO crear factura"
    );

    cleanupPolling();
    const interval = setInterval(async () => {
      try {
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
          const errorData = await response.json().catch(() => ({}));
          throw new Error(
            errorData.message || "Error al verificar el estado del pago"
          );
        }

        const statusData = await response.json();
        console.log(
          "🔄 POLLING: Estado actualizado para orden:",
          orderId,
          statusData
        );

        if (statusData.status === "completed") {
          console.log("🔄 POLLING: Orden completada, finalizando polling.");
          clearInterval(interval);
          setPollingInterval(null);

          // Si es un flujo AFIP, llamar a finalizeAfipPayment
          if (isAfip) {
            await finalizeAfipPayment(statusData);
          }
        } else if (statusData.status === "failed") {
          console.log("🔄 POLLING: Orden fallida, finalizando polling.");
          clearInterval(interval);
          setPollingInterval(null);
          toast.error("Error en el pago AFIP: Estado de la orden fallida.");
          resetPaymentState();
        } else if (statusData.status === "cancelled") {
          console.log("🔄 POLLING: Orden cancelada, finalizando polling.");
          clearInterval(interval);
          setPollingInterval(null);
          toast.error("Pago AFIP cancelado.");
          resetPaymentState();
        }
      } catch (error: any) {
        console.error("❌ Error en polling:", error);
        clearInterval(interval);
        setPollingInterval(null);
        toast.error(`Error en el estado del pago AFIP: ${error.message}`);
        resetPaymentState();
      }
    }, 2000); // Polling cada 2 segundos
    setPollingInterval(interval);
  };

  const cleanupPolling = () => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      setPollingInterval(null);
    }
  };

  // 🆕 FINALIZAR PAGO AFIP después de QR
  const finalizeAfipPayment = async (statusData: any) => {
    console.log("🔥 FINALIZAR PAGO AFIP: Iniciando finalizeAfipPayment");
    console.log("🔥 FINALIZAR PAGO AFIP: statusData:", statusData);

    if (!user) {
      console.error("❌ FINALIZAR PAGO AFIP: Usuario no encontrado");
      toast.error("Error: Usuario no encontrado para finalizar el pago.");
      return;
    }

    try {
      console.log(
        "🔥 FINALIZAR PAGO AFIP: Preparando datos para crear factura..."
      );
      const orderId = statusData.orderId;
      const paymentId = statusData.paymentId;
      const totalAmount = statusData.totalAmount;
      const cashAmount = statusData.cashAmount;
      const qrAmount = statusData.qrAmount;

      // ✅ CORREGIDO: Usar los items del carrito actual
      const items = getCurrentItems ? getCurrentItems() : [];

      console.log("🔥 FINALIZAR PAGO AFIP: Items para factura:", items);

      const businessName = await getBusinessName();
      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };
      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      const facturaData = {
        orderId,
        vendedor: user.nombre,
        businessName,
        items: items.map((item: any) => ({
          productoId: item.id,
          nombre: item.name,
          cantidad: item.quantity,
          subtotal: Number(item.subtotal.toFixed(2)),
          precioHistorico: item.pricePerUnit,
          costo: Number(item.costo),
        })),
        paymentId,
        totalAmount,
        cashAmount,
        qrAmount,
      };

      console.log(
        "🔥 FINALIZAR PAGO AFIP: Enviando a /api/facturas/crear-afip"
      );
      console.log("🔥 FINALIZAR PAGO AFIP: Datos:", facturaData);

      const response = await fetch(`${API_URL}/api/facturas/crear-afip`, {
        method: "POST",
        headers,
        body: JSON.stringify(facturaData),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || "Error al crear la factura AFIP");
      }

      const result = await response.json();
      console.log(
        "✅ FINALIZAR PAGO AFIP: Factura creada exitosamente:",
        result
      );

      // Limpiar estados solo después del éxito
      if (setQrDialogOpen) {
        setQrDialogOpen(false);
      }
      clearCart();
      resetPaymentState();

      toast.success("Factura AFIP creada y ticket impreso exitosamente");
    } catch (error: any) {
      console.error("❌ FINALIZAR PAGO AFIP: Error:", error);
      console.error("❌ FINALIZAR PAGO AFIP: Stack:", error.stack);
      toast.error(`Error al finalizar el pago AFIP: ${error.message}`);
      // No limpiar aquí para que el usuario pueda reintentar
    }
  };

  // Agregar funciones para polling, finalize, etc. similares a usePaymentProcessing pero adaptadas para AFIP (imprimir ticket AFIP al final)

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
    // NUEVA FUNCIÓN: Crear factura AFIP después de confirmación de pago
    createAfipInvoiceAfterPayment,
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
  };
}
