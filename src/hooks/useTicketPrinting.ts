import { toast } from "sonner";

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

      // Intentar imprimir
      if (typeof window !== "undefined" && window.require) {
        const { ipcRenderer } = window.require("electron");
        const result = await ipcRenderer.invoke("print-ticket", orderData);

        if (result.success) {
          toast.success("Ticket impreso correctamente");
        } else if (result.printerError) {
          // Error específico de la impresora TP806L - mostrar toast de error pero no fallar
          console.error("❌ Error de impresora TP806L:", result.printerError);
          toast.error(`Error de impresión: ${result.printerError}`, {
            description:
              "La venta se completó correctamente pero no se pudo imprimir el ticket",
          });
        } else {
          // Error general - mostrar toast de error
          console.error("❌ Error general al imprimir:", result.message);
          toast.error(
            `Error al imprimir el ticket: ${result.message || "Desconocido"}`
          );
        }

        // Siempre retornar true para no cortar el proceso de venta
        // Solo la impresión falló, la venta está completa
        return true;
      } else {
        console.log("🌐 Modo desarrollo: ticket simulado en consola");
        toast.success("Ticket simulado (modo desarrollo)");
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
