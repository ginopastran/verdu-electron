import { toast } from "sonner";
import { getBusinessInfo } from "@/utils/businessHelpers";

// Los tipos de Window están definidos en src/types/electron.d.ts

export const useTicketPrinting = () => {
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

  const handleTicketPrinting = async (
    orderData: any,
    API_URL?: string,
    appId?: string | null
  ) => {
    try {
      // ✅ NUEVO: Obtener información del business para verificar doble impresión
      let businessInfo = null;
      let dobleImpresionEnabled = false;

      if (API_URL && appId !== undefined) {
        try {
          businessInfo = await getBusinessInfo(API_URL, appId);
          dobleImpresionEnabled = businessInfo?.dobleImpresionEnabled === true;
        } catch (error) {
          console.error("❌ Error al obtener business info:", error);
          dobleImpresionEnabled = false;
        }
      }

      // Simular el ticket antes de imprimir
      console.log("\n====== SIMULACIÓN DEL TICKET ======");

      // Determinar el nombre del business de manera dinámica
      let businessName = "Verdulería"; // Valor por defecto
      if (orderData.businessName && orderData.businessName.trim() !== "") {
        businessName = orderData.businessName;
        } else if (orderData.sucursal && orderData.sucursal.trim() !== "") {
        businessName = orderData.sucursal;
      }

      console.log(businessName.toUpperCase());
      console.log(`Vendedor: ${orderData.vendedor}`);
      console.log(
        `Fecha: ${formatFechaArgentina(orderData.createdAt || orderData.fecha)}`
      );

      // Mostrar ID de la orden si está disponible
      if (orderData.idReal && orderData.idReal !== "") {
        console.log(`Orden #${orderData.idReal}`);
      } else if (orderData.id && orderData.id !== "") {
        console.log(`Orden #${orderData.id}`);
      }

      console.log("-----------------------------");
      console.log("PRODUCTO      CANT    PRECIO    TOTAL");
      console.log("-----------------------------");

      // Mostrar productos
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
        // No hay items en la orden
      }

      console.log("-----------------------------");

      // 🆕 SIMULACIÓN DE DESCUENTOS - Replicar lógica del PHP
      let hasDiscount = false;
      let subtotalOriginal = 0;
      let descuentoMonto = 0;
      let tipoDescuento = '';
      let valorDescuento = 0;


      // Verificar si hay información de descuentos (método 1: discountData)
      if (orderData.discountData && typeof orderData.discountData === 'object') {
        const discountData = orderData.discountData;
        
        // Si discountData.amount es 0 pero hay type y value, calcular el descuento
        if (discountData.type && discountData.value > 0) {
          hasDiscount = true;
          tipoDescuento = discountData.type;
          valorDescuento = discountData.value;
          
          // Usar el subtotal que viene del frontend como base
          if (orderData.subtotal && orderData.subtotal > 0) {
            subtotalOriginal = orderData.subtotal;
          } else if (orderData.subtotalSinDescuento && orderData.subtotalSinDescuento > 0) {
            subtotalOriginal = orderData.subtotalSinDescuento;
          } else {
            // Fallback: usar el total como subtotal original si no hay descuento real
            subtotalOriginal = orderData.total;
          }
          
          // Calcular el descuento basado en el tipo
          if (discountData.type === 'percentage' || discountData.type === 'porcentual') {
            descuentoMonto = (subtotalOriginal * valorDescuento) / 100;
          } else {
            descuentoMonto = valorDescuento;
          }
          
          // Si discountData.amount existe y es mayor que 0, usarlo
          if (discountData.amount && discountData.amount > 0) {
            descuentoMonto = discountData.amount;
          }
          
          // ⚠️ CORRECCIÓN: Si el total del backend no refleja el descuento, usar el calculado
          const totalCalculado = subtotalOriginal - descuentoMonto;
          if (Math.abs(orderData.total - totalCalculado) > 0.01) {
            // Actualizar el total para la simulación
            orderData.total = totalCalculado;
          }
        }
      } 
      // Verificar método 2: campos directos del backend
      else if (orderData.tieneDescuento && orderData.subtotalSinDescuento) {
        hasDiscount = true;
        subtotalOriginal = orderData.subtotalSinDescuento;
        descuentoMonto = subtotalOriginal - orderData.total;
        tipoDescuento = orderData.tipoDescuento || 'unknown';
        valorDescuento = orderData.valorDescuento || 0;
        
      }
      // Fallback: calcular descuento basado en subtotal y total
      else if (orderData.subtotal && orderData.total && orderData.subtotal > orderData.total) {
        hasDiscount = true;
        subtotalOriginal = orderData.subtotal;
        descuentoMonto = subtotalOriginal - orderData.total;
        
      }

      // Mostrar desglose si hay descuento
      if (hasDiscount) {
        console.log(`Subtotal: $${subtotalOriginal.toFixed(2)}`);
        
        // Mostrar información del descuento
        if (tipoDescuento && valorDescuento > 0) {
          if (tipoDescuento === 'percentage' || tipoDescuento === 'porcentual') {
            console.log(`Descuento (${valorDescuento}%): -$${descuentoMonto.toFixed(2)}`);
          } else {
            console.log(`Descuento: -$${descuentoMonto.toFixed(2)}`);
          }
        } else {
          console.log(`Descuento aplicado: -$${descuentoMonto.toFixed(2)}`);
        }
        console.log("-----------------------------");
      }

      console.log(`TOTAL: $${Number(orderData.total).toFixed(2)}`);

      // Mostrar método(s) de pago
      if (orderData.pagos && Array.isArray(orderData.pagos)) {
        console.log("\nMÉTODOS DE PAGO:");
        orderData.pagos.forEach((pago: any) => {
          console.log(
            `${pago.metodoPago.toUpperCase()}: $${Number(pago.monto).toFixed(
              2
            )}`
          );
        });
      } else {
        console.log(`\nMétodo de pago: ${orderData.metodoPago?.toUpperCase()}`);
      }

      console.log("\n¡Gracias por su compra!");
      console.log("==============================\n");

      // ✅ FUNCIÓN SIMPLE para llamar al script PHP de impresión
      const callPrintScript = async () => {
        // Método 1: Usar window.printer (API específica para impresión)
        if (typeof window !== "undefined" && window.printer?.printTicket) {
          return await window.printer.printTicket(orderData);
        }
        // Método 2: Usar window.electron.ipcRenderer (API general)
        else if (
          typeof window !== "undefined" &&
          window.electron?.ipcRenderer
        ) {
          return await window.electron.ipcRenderer.invoke(
            "print-ticket",
            orderData
          );
        }
        // Si ninguna API está disponible
        else {
          throw new Error("API de Electron no disponible");
        }
      };

      // ✅ LÓGICA SÚPER SIMPLE: Si dobleImpresionEnabled = true, hacer 2 llamadas
      // console.log("🔍 DEBUG DOBLE IMPRESIÓN:", {
      //   dobleImpresionEnabled,
      //   businessInfoExists: !!businessInfo,
      //   valorOriginal: businessInfo?.dobleImpresionEnabled,
      // });

      try {
        if (dobleImpresionEnabled === true) {
          console.log(
            "🖨️🖨️ DOBLE IMPRESIÓN ACTIVADA - Mandando 2 impresiones al script PHP"
          );

          // ✅ PRIMERA IMPRESIÓN
          console.log("📄 1️⃣ Primera llamada al script PHP...");
          const result1 = await callPrintScript();
          console.log("📄 1️⃣ Resultado primera impresión:", result1);

          // ✅ SEGUNDA IMPRESIÓN (después de un pequeño delay)
          await new Promise((resolve) => setTimeout(resolve, 500));
          console.log("📄 2️⃣ Segunda llamada al script PHP...");
          const result2 = await callPrintScript();
          console.log("📄 2️⃣ Resultado segunda impresión:", result2);

          // Mostrar resultado
          if (result1?.success && result2?.success) {
            toast.success("Doble ticket impreso correctamente", {
              description: "Se mandaron 2 impresiones al script PHP",
            });
          } else {
            toast.warning("Problemas con la doble impresión", {
              description: `Primera: ${
                result1?.success ? "OK" : "Error"
              }, Segunda: ${result2?.success ? "OK" : "Error"}`,
            });
          }
        } else {
          console.log(
            "🖨️ IMPRESIÓN SIMPLE - Mandando 1 impresión al script PHP"
          );
          console.log("📄 Única llamada al script PHP...");
          const result = await callPrintScript();
          console.log("📄 Resultado impresión simple:", result);

          if (result?.success) {
            toast.success("Ticket impreso correctamente");
          } else if (result?.printerError) {
            toast.error(`Error de impresión: ${result.printerError}`);
          } else {
            toast.error("Error al imprimir el ticket");
          }
        }

        // Siempre retornar true para no cortar el proceso de venta
        // Solo la impresión falló, la venta está completa
        return true;
      } catch (electronError: any) {
        // Si no se puede acceder a Electron, mostrar error específico
        console.error("❌ Error al acceder a Electron:", electronError);

        // Mostrar información de debug para ayudar a diagnosticar
        console.log("🔍 Debug detallado - Estado del preload:");
        console.log("- window existe:", typeof window !== "undefined");
        console.log("- window.electron:", typeof window.electron);
        console.log("- window.printer:", typeof window.printer);
        console.log(
          "- window.electronStore:",
          typeof (window as any).electronStore
        );

        toast.error("Error de conexión con la impresora", {
          description:
            "El sistema de impresión no está disponible. Verifica que la aplicación se esté ejecutando correctamente.",
        });
        return true;
      }
    } catch (error: any) {
      console.error("❌ Error al imprimir:", error);
      toast.error(
        `Error al imprimir el ticket: ${error.message || "Desconocido"}`,
        {
          description:
            "La venta se completó correctamente pero no se pudo imprimir el ticket",
        }
      );
      // Retornar true para no cortar el proceso de venta
      return true;
    }
  };

  return { handleTicketPrinting, formatFechaArgentina };
};
