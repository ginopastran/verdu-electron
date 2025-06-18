import { toast } from "sonner";

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

  const handleTicketPrinting = async (orderData: any) => {
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

      // Simular el ticket antes de imprimir
      console.log("\n====== SIMULACIÓN DEL TICKET ======");
      console.log("ISELIN II");
      console.log(`Vendedor: ${orderData.vendedor}`);
      console.log(
        `Fecha: ${formatFechaArgentina(orderData.createdAt || orderData.fecha)}`
      );
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

      // Intentar imprimir - probar múltiples métodos de acceso a la API
      try {
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
