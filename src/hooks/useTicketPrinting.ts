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
        }
        // Si hay error, solo lo logueamos pero no mostramos toast
        else {
          console.error(
            "❌ Error al imprimir (IPC invoke returned false):",
            result.message
          );
          toast.error(
            `Error al imprimir el ticket: ${result.message || "Desconocido"}`
          );
        }
        return result.success;
      } else {
        console.log("🌐 Modo desarrollo: ticket simulado en consola");
        toast.success("Ticket simulado (modo desarrollo)");
        return true;
      }
    } catch (error: any) {
      console.error("❌ Error al imprimir:", error);
      toast.error(
        `Error al imprimir el ticket: ${error.message || "Desconocido"}`
      );
      return false;
    }
  };

  return { handleTicketPrinting, formatFechaArgentina };
};
