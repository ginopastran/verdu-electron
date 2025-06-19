import { toast } from "sonner";
import { getBusinessInfo } from "@/utils/businessHelpers";

// Declaración de tipos para window
declare global {
  interface Window {
    electron?: {
      ipcRenderer: {
        invoke: (channel: string, ...args: any[]) => Promise<any>;
      };
    };
    printer?: {
      printTicket: (orderData: any) => Promise<any>;
      printClosing: (closingData: any) => Promise<any>;
    };
  }
}

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

          console.log("🔍 DEBUG: RESULTADO DE getBusinessInfo:", {
            businessInfo_existe: !!businessInfo,
            businessInfo_completo: businessInfo,
            dobleImpresionEnabled_raw: businessInfo?.dobleImpresionEnabled,
            dobleImpresionEnabled_tipo:
              typeof businessInfo?.dobleImpresionEnabled,
          });

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

          console.log("📋 Business info obtenida:", {
            businessInfo: !!businessInfo,
            dobleImpresionEnabled_final: dobleImpresionEnabled,
            valor_original: businessInfo?.dobleImpresionEnabled,
          });
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

      // ✅ NUEVO: Función auxiliar para realizar una impresión
      const performSinglePrint = async (): Promise<any> => {
        let result;
        let apiUsed = "";

        // Método 1: Usar window.printer (API específica para impresión)
        if (typeof window !== "undefined" && window.printer?.printTicket) {
          console.log("🖨️ Usando window.printer.printTicket");
          apiUsed = "window.printer";
          result = await window.printer.printTicket(orderData);
        }
        // Método 2: Usar window.electron.ipcRenderer (API general)
        else if (
          typeof window !== "undefined" &&
          window.electron?.ipcRenderer
        ) {
          console.log("📡 Usando window.electron.ipcRenderer");
          apiUsed = "window.electron";
          result = await window.electron.ipcRenderer.invoke(
            "print-ticket",
            orderData
          );
        }
        // Si ninguna API está disponible
        else {
          // Verificar qué APIs están disponibles para debug
          console.log("🔍 Debug - APIs disponibles:", {
            electron: typeof window.electron,
            printer: typeof window.printer,
            electronStore: typeof (window as any).electronStore,
            autoUpdater: typeof (window as any).autoUpdater,
          });
          throw new Error(
            "API de Electron no disponible - ninguna API de impresión encontrada"
          );
        }

        console.log(`📄 Resultado de impresión (${apiUsed}):`, result);
        return { result, apiUsed };
      };

      // ✅ NUEVO: Debug antes de la decisión de impresión
      console.log("🔍 DEBUG: ANTES DE LA DECISIÓN DE IMPRESIÓN:", {
        dobleImpresionEnabled,
        tipoValor: typeof dobleImpresionEnabled,
        condicionIf: dobleImpresionEnabled === true,
        businessInfoExists: !!businessInfo,
        valorOriginalBusiness: businessInfo?.dobleImpresionEnabled,
      });

      // ✅ NUEVO: Implementar lógica de doble impresión
      try {
        if (dobleImpresionEnabled) {
          console.log(
            "🖨️🖨️ DOBLE IMPRESIÓN HABILITADA - Imprimiendo 2 tickets"
          );
          console.log("🎯 ENTRANDO EN FLUJO DE DOBLE IMPRESIÓN");

          // Primera impresión
          console.log("📄 Realizando primera impresión...");
          const { result: result1, apiUsed } = await performSinglePrint();

          console.log("🔍 DEBUG: Resultado primera impresión:", {
            result1,
            success: result1?.success,
            printerError: result1?.printerError,
            apiUsed,
          });

          if (result1 && result1.success && !result1.printerError) {
            console.log("✅ Primera impresión exitosa");

            // Esperar un momento entre impresiones para evitar conflictos
            console.log(
              "⏱️ Esperando 1 segundo antes de la segunda impresión..."
            );
            await new Promise((resolve) => setTimeout(resolve, 1000));

            // Segunda impresión
            console.log("📄 Realizando segunda impresión...");
            const { result: result2 } = await performSinglePrint();

            console.log("🔍 DEBUG: Resultado segunda impresión:", {
              result2,
              success: result2?.success,
              printerError: result2?.printerError,
            });

            if (result2 && result2.success && !result2.printerError) {
              console.log("✅ Segunda impresión exitosa");
              toast.success("Doble ticket impreso correctamente", {
                description: "Se imprimieron 2 tickets como está configurado",
              });
            } else {
              console.log(
                "⚠️ Segunda impresión falló, pero la primera fue exitosa"
              );
              toast.warning("Primer ticket impreso, segundo falló", {
                description:
                  "Se imprimió solo un ticket debido a un error en la segunda impresión",
              });
            }
          } else {
            console.log("❌ Primera impresión falló");
            if (result1 && result1.printerError) {
              console.error(
                "❌ Error de impresora TP806L:",
                result1.printerError
              );
              toast.error(`Error de impresión: ${result1.printerError}`, {
                description:
                  "La venta se completó correctamente pero no se pudo imprimir el ticket",
              });
            } else {
              console.error("❌ Error general al imprimir:", result1?.message);
              toast.error(
                `Error al imprimir el ticket: ${
                  result1?.message || "Desconocido"
                }`
              );
            }
          }
        } else {
          // Impresión simple (comportamiento original)
          console.log("🖨️ IMPRESIÓN SIMPLE - Imprimiendo 1 ticket");
          console.log("🎯 ENTRANDO EN FLUJO DE IMPRESIÓN SIMPLE");
          console.log("🔍 DEBUG: Razón para impresión simple:", {
            dobleImpresionEnabled,
            businessInfoExists: !!businessInfo,
            API_URL_proporcionado: !!API_URL,
            appId_proporcionado: appId !== undefined,
            valorOriginalBusiness: businessInfo?.dobleImpresionEnabled,
          });
          const { result, apiUsed } = await performSinglePrint();

          if (result && result.success && !result.printerError) {
            toast.success("Ticket impreso correctamente");
          } else if (result && result.printerError) {
            // Error específico de la impresora TP806L - mostrar toast de error pero no fallar
            console.error("❌ Error de impresora TP806L:", result.printerError);
            toast.error(`Error de impresión: ${result.printerError}`, {
              description:
                "La venta se completó correctamente pero no se pudo imprimir el ticket",
            });
          } else {
            // Error general - mostrar toast de error
            console.error("❌ Error general al imprimir:", result?.message);
            toast.error(
              `Error al imprimir el ticket: ${result?.message || "Desconocido"}`
            );
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
