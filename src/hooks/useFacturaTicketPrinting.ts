/// <reference path="../types/electron.d.ts" />

import { toast } from "sonner";
import { getBusinessInfo } from "@/utils/businessHelpers";

// Los tipos de Window están definidos en src/types/electron.d.ts

export const useFacturaTicketPrinting = () => {
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

  const handleFacturaTicketPrinting = async (
    facturaData: any,
    API_URL?: string,
    appId?: string | null
  ) => {
    try {
      // Obtener información del business para verificar doble impresión
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

      // Simular el ticket de factura antes de imprimir
      console.log("\n====== SIMULACIÓN DEL TICKET DE FACTURA ======");

      // Determinar el nombre del business con lógica de prioridades robusta
      let businessName = 
        // 🆕 PRIORIDAD 1: Información del business obtenida del contexto
        businessInfo?.nombre ||
        businessInfo?.name ||
        businessInfo?.razonSocial ||
        // PRIORIDAD 2: Datos de la factura
        facturaData.businessName ||
        facturaData.sucursal ||
        // PRIORIDAD 3: Datos del negocio en la factura
        facturaData.business?.nombre ||
        facturaData.business?.name ||
        facturaData.business?.razonSocial ||
        // FALLBACK - usar un nombre más apropiado
        "Mi Negocio";


      console.log(businessName.toUpperCase());
      console.log(`Tipo: ${facturaData.tipoFactura || "FACTURA"}`);

      // Mostrar número de factura si está disponible
      if (facturaData.numero && facturaData.numero !== "") {
        console.log(`Nº ${facturaData.numero}`);
      } else if (facturaData.id && facturaData.id !== "") {
        console.log(`ID: ${facturaData.id}`);
      }

      console.log(
        `Fecha: ${formatFechaArgentina(
          facturaData.fechaEmision || facturaData.createdAt || new Date()
        )}`
      );

      // Datos del cliente
      console.log("-----------------------------");
      console.log("DATOS DEL CLIENTE");
      if (facturaData.cliente) {
        const cliente = facturaData.cliente;

        if (cliente.razonSocial) {
          console.log(`Razón Social: ${cliente.razonSocial}`);
        } else {
          const nombreCompleto = `${cliente.nombre || ""} ${
            cliente.apellido || ""
          }`.trim();
          if (nombreCompleto) {
            console.log(`Cliente: ${nombreCompleto}`);
          }
        }

        if (cliente.cuit) {
          console.log(`CUIT: ${cliente.cuit}`);
        }
      }

      console.log("-----------------------------");
      console.log("PRODUCTO      CANT    PRECIO    TOTAL");
      console.log("-----------------------------");

      // Mostrar productos
      const detalles = facturaData.detalles || [];
      if (detalles && detalles.length > 0) {
        detalles.forEach((detalle: any) => {
          // Manejar diferentes estructuras de nombre del producto
          let nombreProducto = "";
          if (detalle.descripcion) {
            nombreProducto = detalle.descripcion;
          } else if (
            detalle.producto &&
            typeof detalle.producto === "object" &&
            detalle.producto.nombre
          ) {
            nombreProducto = detalle.producto.nombre;
          } else if (detalle.producto && typeof detalle.producto === "string") {
            nombreProducto = detalle.producto;
          } else {
            nombreProducto = "Producto";
          }

          const nombre = nombreProducto.substring(0, 12).padEnd(12);
          const cantidad = (detalle.cantidad || 0).toString().padStart(8);
          const precio = `$${Number(
            detalle.precioUnitario || detalle.precio || 0
          ).toFixed(2)}`.padStart(8);
          const subtotal = `$${Number(detalle.subtotal || 0).toFixed(
            2
          )}`.padStart(8);

          console.log(`${nombre} ${cantidad} ${precio} ${subtotal}`);
        });
      } else {
        console.log("❌ No hay detalles en la factura");
      }

      console.log("-----------------------------");

      // Mostrar totales
      if (facturaData.subtotal) {
        console.log(`SUBTOTAL: $${Number(facturaData.subtotal).toFixed(2)}`);
      }

      if (facturaData.impuestos && facturaData.impuestos > 0) {
        console.log(`IVA: $${Number(facturaData.impuestos).toFixed(2)}`);
      }

      console.log(`TOTAL: $${Number(facturaData.total).toFixed(2)}`);

      // Pago inicial si es cuenta corriente
      if (facturaData.pagoInicial && facturaData.pagoInicial > 0) {
        console.log(
          `PAGO INICIAL: $${Number(facturaData.pagoInicial).toFixed(2)}`
        );
        const saldo = facturaData.total - facturaData.pagoInicial;
        console.log(`SALDO: $${Number(saldo).toFixed(2)}`);
      }

      // Observaciones
      if (facturaData.observaciones) {
        console.log("-----------------------------");
        console.log("OBSERVACIONES:");
        console.log(facturaData.observaciones);
      }

      console.log("\n¡Gracias por su compra!");
      console.log("==============================\n");

      // FUNCIÓN para llamar al script PHP de impresión de facturas
      const callFacturaPrintScript = async () => {
        // 🆕 Construir datos mejorados para enviar al printer con businessName correcto
        const printData = {
          ...facturaData,
          // Asegurar que el businessName correcto esté incluido
          businessName: businessName,
          // Incluir información adicional del business si está disponible
          nombre: businessInfo?.nombre || businessName,
          razonSocial: businessInfo?.razonSocial || businessName,
          cuit: businessInfo?.cuit || facturaData.cuit,
          telefono: businessInfo?.telefono || facturaData.telefono
        };

        // Método 1: Usar window.printer (API específica para impresión de facturas)
        if (
          typeof window !== "undefined" &&
          (window as any).printer?.printFacturaTicket
        ) {
          return await (window as any).printer.printFacturaTicket(printData);
        }
        // Método 2: Usar window.electron.ipcRenderer (API general)
        else if (
          typeof window !== "undefined" &&
          (window as any).electron?.ipcRenderer
        ) {
          return await (window as any).electron.ipcRenderer.invoke(
            "print-factura-ticket",
            printData
          );
        }
        // Si ninguna API está disponible
        else {
          throw new Error(
            "API de Electron no disponible para impresión de facturas"
          );
        }
      };

      // LÓGICA: Si dobleImpresionEnabled = true, hacer 2 llamadas
      try {
        if (dobleImpresionEnabled === true) {
          // PRIMERA IMPRESIÓN
          const result1 = await callFacturaPrintScript();

          // SEGUNDA IMPRESIÓN (después de un pequeño delay)
          await new Promise((resolve) => setTimeout(resolve, 500));
          const result2 = await callFacturaPrintScript();

          // Mostrar resultado
          if (result1?.success && result2?.success) {
            toast.success("Doble ticket de factura impreso correctamente", {
              description: "Se mandaron 2 impresiones del ticket de factura",
            });
          } else {
            toast.warning("Problemas con la doble impresión de factura", {
              description: `Primera: ${
                result1?.success ? "OK" : "Error"
              }, Segunda: ${result2?.success ? "OK" : "Error"}`,
            });
          }
        } else {
          const result = await callFacturaPrintScript();
          // ✅ RETORNAR ESTADO REAL DE IMPRESIÓN (sin toasts - se manejan en FacturaForm)
          return result?.success === true;
        }

        // ✅ RETORNAR ESTADO REAL DE IMPRESIÓN
        return true;
      } catch (electronError: any) {
        // Si no se puede acceder a Electron, mostrar error específico
        console.error(
          "❌ Error al acceder a Electron para factura:",
          electronError
        );

        // ✅ RETORNAR FALSE EN CASO DE ERROR DE CONEXIÓN (sin toast - se maneja en FacturaForm)
        return false;
      }
    } catch (error: any) {
      console.error("❌ Error al imprimir factura:", error);
      // ✅ RETORNAR FALSE EN CASO DE ERROR GENERAL (sin toast - se maneja en FacturaForm)
      return false;
    }
  };

  return { handleFacturaTicketPrinting, formatFechaArgentina };
};
