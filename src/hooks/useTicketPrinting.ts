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
      // DEBUG: Verificar APIs disponibles al inicio
      console.log("🔍 Verificación inicial de APIs:");
      console.log("- window existe:", typeof window !== "undefined");
      console.log("- window.electron:", typeof window.electron);
      console.log("- window.printer:", typeof window.printer);
      console.log(
        "- window.electronStore:",
        typeof (window as any).electronStore
      );
      console.log("- window.autoUpdater:", typeof (window as any).autoUpdater);
      console.log("- window.pesoReader:", typeof (window as any).pesoReader);

      // ✅ NUEVO: Obtener información del business para verificar doble impresión
      let businessInfo = null;
      let dobleImpresionEnabled = false;

      console.log("🔍 DEBUG: ANTES DE LA DECISIÓN DE OBTENER BUSINESS INFO:", {
        API_URL_proporcionado: !!API_URL,
        API_URL_valor: API_URL,
        appId_proporcionado: appId !== undefined,
        appId_valor: appId,
        appId_tipo: typeof appId,
        condicionIf: API_URL && appId !== undefined,
      });

      if (API_URL && appId !== undefined) {
        console.log(
          "🏢 Obteniendo información del business para doble impresión..."
        );

        try {
          businessInfo = await getBusinessInfo(API_URL, appId);

          // console.log("🔍 DEBUG: RESULTADO DE getBusinessInfo:", {
          //   businessInfo_existe: !!businessInfo,
          //   businessInfo_completo: businessInfo,
          //   dobleImpresionEnabled_raw: businessInfo?.dobleImpresionEnabled,
          //   dobleImpresionEnabled_tipo:
          //     typeof businessInfo?.dobleImpresionEnabled,
          // });

          // Evaluar dobleImpresionEnabled con diferentes comparaciones
          const comparaciones = {
            estricta_true: businessInfo?.dobleImpresionEnabled === true,
            flexible_true: businessInfo?.dobleImpresionEnabled == true,
            string_true: businessInfo?.dobleImpresionEnabled === "true",
            truthy: !!businessInfo?.dobleImpresionEnabled,
            numero_1: businessInfo?.dobleImpresionEnabled === 1,
          };

          console.log(
            "🔍 DEBUG: COMPARACIONES dobleImpresionEnabled:",
            comparaciones
          );

          // Usar la comparación estricta por defecto
          dobleImpresionEnabled = businessInfo?.dobleImpresionEnabled === true;

          // console.log("📋 Business info obtenida:", {
          //   businessInfo: !!businessInfo,
          //   dobleImpresionEnabled_final: dobleImpresionEnabled,
          //   valor_original: businessInfo?.dobleImpresionEnabled,
          // });
        } catch (error) {
          console.error("❌ Error al obtener business info:", error);
          dobleImpresionEnabled = false;
        }
      } else {
        console.log(
          "⚠️ No se proporcionaron API_URL o appId, usando impresión simple"
        );
        console.log("🔍 DEBUG: Razones para usar impresión simple:", {
          API_URL_missing: !API_URL,
          appId_undefined: appId === undefined,
          API_URL_actual: API_URL,
          appId_actual: appId,
        });
      }

      // Simular el ticket antes de imprimir
      console.log("\n====== SIMULACIÓN DEL TICKET ======");

      // Determinar el nombre del business de manera dinámica
      let businessName = "Verdulería"; // Valor por defecto
      if (orderData.businessName && orderData.businessName.trim() !== "") {
        businessName = orderData.businessName;
        console.log(`✅ Usando nombre del business: ${businessName}`);
      } else if (orderData.sucursal && orderData.sucursal.trim() !== "") {
        businessName = orderData.sucursal;
        console.log(`✅ Usando nombre de sucursal: ${businessName}`);
      } else {
        console.log(`⚠️ Usando nombre por defecto: ${businessName}`);
      }

      console.log(businessName.toUpperCase());
      console.log(`Vendedor: ${orderData.vendedor}`);
      console.log(
        `Fecha: ${formatFechaArgentina(orderData.createdAt || orderData.fecha)}`
      );

      // Mostrar ID de la orden si está disponible
      if (orderData.idReal && orderData.idReal !== "") {
        console.log(`Orden #${orderData.idReal}`);
        console.log(`✅ ID Real encontrado: ${orderData.idReal}`);
      } else if (orderData.id && orderData.id !== "") {
        console.log(`Orden #${orderData.id}`);
        console.log(`⚠️ Usando ID regular: ${orderData.id}`);
      } else {
        console.log(`❌ No se encontró ID de orden`);
        console.log(
          `🔍 Claves disponibles: ${Object.keys(orderData).join(", ")}`
        );
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
        console.log("❌ No hay items en la orden");
      }

      console.log("-----------------------------");

      // 🆕 SIMULACIÓN DE DESCUENTOS - Replicar lógica del PHP
      let hasDiscount = false;
      let subtotalOriginal = 0;
      let descuentoMonto = 0;
      let tipoDescuento = '';
      let valorDescuento = 0;

      console.log("\n🔍 ANÁLISIS DE DESCUENTOS:");
      console.log("- orderData.discountData:", orderData.discountData);
      console.log("- orderData.tieneDescuento:", orderData.tieneDescuento);
      console.log("- orderData.subtotal:", orderData.subtotal);
      console.log("- orderData.subtotalSinDescuento:", orderData.subtotalSinDescuento);

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
          
          console.log("✅ DESCUENTO DETECTADO (discountData):");
          console.log(`- Subtotal original: $${subtotalOriginal.toFixed(2)}`);
          console.log(`- Descuento aplicado: -$${descuentoMonto.toFixed(2)}`);
          console.log(`- Total calculado: $${(subtotalOriginal - descuentoMonto).toFixed(2)}`);
          console.log(`- Total del backend: $${orderData.total.toFixed(2)}`);
          console.log(`- Tipo descuento: ${tipoDescuento}`);
          console.log(`- Valor descuento: ${valorDescuento}`);
          
          // ⚠️ CORRECCIÓN: Si el total del backend no refleja el descuento, usar el calculado
          const totalCalculado = subtotalOriginal - descuentoMonto;
          if (Math.abs(orderData.total - totalCalculado) > 0.01) {
            console.log(`⚠️ CORRECCIÓN: El total del backend ($${orderData.total.toFixed(2)}) no refleja el descuento`);
            console.log(`📝 Usando total calculado: $${totalCalculado.toFixed(2)}`);
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
        
        console.log("✅ DESCUENTO DETECTADO (campos backend):");
        console.log(`- Subtotal original: $${subtotalOriginal.toFixed(2)}`);
        console.log(`- Descuento calculado: -$${descuentoMonto.toFixed(2)}`);
        console.log(`- Total final: $${orderData.total.toFixed(2)}`);
        console.log(`- Tipo descuento: ${tipoDescuento}`);
        console.log(`- Valor descuento: ${valorDescuento}`);
      }
      // Fallback: calcular descuento basado en subtotal y total
      else if (orderData.subtotal && orderData.total && orderData.subtotal > orderData.total) {
        hasDiscount = true;
        subtotalOriginal = orderData.subtotal;
        descuentoMonto = subtotalOriginal - orderData.total;
        
        console.log("⚠️ DESCUENTO DETECTADO (fallback):");
        console.log(`- Subtotal original: $${subtotalOriginal.toFixed(2)}`);
        console.log(`- Descuento calculado: -$${descuentoMonto.toFixed(2)}`);
      } else {
        console.log("❌ NO SE DETECTÓ DESCUENTO");
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
        // 🆕 DEBUG DETALLADO DE DESCUENTOS ANTES DE ENVIAR AL PHP
        console.log("🔍 DEBUG DESCUENTOS TICKET NORMAL ANTES DE IMPRESIÓN:");
        console.log("- orderData completo:", orderData);
        console.log("- orderData.discountData:", orderData.discountData);
        console.log("- orderData.tieneDescuento:", orderData.tieneDescuento);
        console.log("- orderData.tipoDescuento:", orderData.tipoDescuento);
        console.log("- orderData.valorDescuento:", orderData.valorDescuento);
        console.log("- orderData.montoDescuento:", orderData.montoDescuento);
        console.log("- orderData.subtotalSinDescuento:", orderData.subtotalSinDescuento);
        console.log("- orderData.subtotal:", orderData.subtotal);
        console.log("- orderData.total:", orderData.total);
        console.log("- orderData.totalConDescuento:", orderData.totalConDescuento);
        console.log("- orderData.descuentoAplicado:", orderData.descuentoAplicado);
        console.log("- Diferencia subtotal-total:", (orderData.subtotal || 0) - (orderData.total || 0));
        console.log("🔍 FIN DEBUG DESCUENTOS TICKET NORMAL");

        // Método 1: Usar window.printer (API específica para impresión)
        if (typeof window !== "undefined" && window.printer?.printTicket) {
          console.log("🖨️ Llamando script PHP via window.printer.printTicket");
          return await window.printer.printTicket(orderData);
        }
        // Método 2: Usar window.electron.ipcRenderer (API general)
        else if (
          typeof window !== "undefined" &&
          window.electron?.ipcRenderer
        ) {
          console.log("🖨️ Llamando script PHP via window.electron.ipcRenderer");
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
        console.log(
          "- Todas las propiedades de window:",
          Object.keys(window).filter(
            (key) => key.includes("electron") || key.includes("printer")
          )
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
