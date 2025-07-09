import { useState } from "react";
import { toast } from "sonner";

// Declaración de tipos para window
declare global {
  interface Window {
    printer?: {
      printTicket: (orderData: any) => Promise<any>;
      printClosing: (closingData: any) => Promise<any>;
    };
  }
}

export const useClosing = (
  user: any,
  API_URL: string,
  appId: string | null
) => {
  const [isClosing, setIsClosing] = useState(false);
  const [closingDialogOpen, setClosingDialogOpen] = useState(false);

  // Helper para acceder a la API de impresión
  const getPrinterAPI = () => {
    try {
      if (typeof window !== "undefined" && window.printer) {
        return window.printer;
      }
      return null;
    } catch (error) {
      console.error("❌ Error al acceder a la impresora:", error);
      return null;
    }
  };

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

  const handleClosing = async (period: string) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar el cierre");
      return;
    }

    if (!user.permisos?.cierreDeCajaEnabled) {
      toast.error("No tienes permiso para realizar cierres de caja");
      setClosingDialogOpen(false);
      return;
    }

    setIsClosing(true);

    try {
      // Función para crear fechas en hora local Argentina y convertir a UTC
      const crearFechaArgentina = (
        year: number,
        month: number,
        day: number,
        hour: number = 0,
        minute: number = 0
      ) => {
        // Crear fecha en hora local Argentina
        const fecha = new Date(year, month - 1, day, hour, minute, 0, 0);

        // Convertir a UTC restando 3 horas (Argentina es UTC-3)
        return new Date(fecha.getTime() - 3 * 60 * 60 * 1000);
      };

      // Obtener fecha y hora actuales
      const ahora = new Date();
      const year = ahora.getFullYear();
      const month = ahora.getMonth() + 1;
      const day = ahora.getDate();

      // ✅ FECHAS DINÁMICAS: Usar hora actual para fechaCierre
      let fechaInicioUTC;
      let fechaCierreUTC;

      // La fechaCierre siempre es la hora actual
      fechaCierreUTC = new Date(ahora.getTime() - 3 * 60 * 60 * 1000); // Hora actual en UTC

      if (period === "mañana") {
        // Para cierre de mañana: inicio a las 6:00 AM del mismo día
        fechaInicioUTC = crearFechaArgentina(year, month, day, 0, 0); // 0:00 AM
      } else if (period === "tarde") {
        // Para cierre de tarde: inicio depende de si ya hay cierre de mañana
        // Por simplicidad, usamos 12:00 PM (mediodía) como inicio de tarde
        fechaInicioUTC = crearFechaArgentina(year, month, day, 12, 0); // 12:00 PM
      } else {
        // Para cierre de todo el día: inicio a las 6:00 AM
        fechaInicioUTC = crearFechaArgentina(year, month, day, 0, 0); // 0:00 AM
      }

      // ✅ VALIDACIÓN: Asegurar que fechaInicio sea anterior a fechaCierre
      if (fechaInicioUTC.getTime() >= fechaCierreUTC.getTime()) {
        // Si fechaInicio es mayor o igual, ajustar fechaInicio al día anterior
        const fechaInicioAjustada = new Date(fechaInicioUTC);
        fechaInicioAjustada.setDate(fechaInicioAjustada.getDate() - 1);
        fechaInicioUTC = fechaInicioAjustada;

        console.log(
          "⚠️ Ajustando fechaInicio al día anterior para evitar conflicto"
        );
      }

      console.log(
        `🕒 Fecha inicio (${period}) UTC:`,
        fechaInicioUTC.toISOString()
      );
      console.log(
        `🕒 Fecha inicio (${period}) hora Argentina:`,
        formatFechaArgentina(fechaInicioUTC.toISOString())
      );
      console.log(
        `🕒 Fecha cierre (${period}) UTC:`,
        fechaCierreUTC.toISOString()
      );
      console.log(
        `🕒 Fecha cierre (${period}) hora Argentina:`,
        formatFechaArgentina(fechaCierreUTC.toISOString())
      );

      const closingData = {
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        fechaInicio: fechaInicioUTC.toISOString(),
        fechaCierre: fechaCierreUTC.toISOString(),
        periodo: period,
      };

      console.log("🔄 Enviando solicitud de cierre con datos:", closingData);

      const headers = {
        "Content-Type": "application/json",
        ...(appId && { "X-App-ID": appId }),
      };

      const cierreResponse = await fetch(`${API_URL}/api/cierres`, {
        method: "POST",
        headers,
        body: JSON.stringify(closingData),
      });

      const responseData = await cierreResponse.json();

      if (!cierreResponse.ok) {
        if (responseData.error === "ERROR_CIERRE_MAÑANA_REQUERIDO") {
          toast.error(
            "No puedes realizar un cierre de tarde sin haber realizado el cierre de mañana del día actual.",
            {
              duration: 5000,
              description: "Primero debes realizar el cierre de mañana",
            }
          );
        } else {
          throw new Error(
            responseData.message || "Error al registrar el cierre"
          );
        }
        return;
      }

      const cierreData = await responseData;
      console.log("✅ Datos de cierre recibidos:", cierreData);

      try {
        // ====== SIMULACIÓN DEL TICKET DE CIERRE ======
        console.log("\n====== SIMULACIÓN DEL TICKET DE CIERRE ======");
        console.log("ISELIN II");
        console.log(`CIERRE DE CAJA - ${period.toUpperCase()}`);
        console.log(`Vendedor: ${user.nombre}`);
        console.log(
          `Fecha inicio: ${formatFechaArgentina(cierreData.fechaInicio)}`
        );
        console.log(
          `Fecha cierre: ${formatFechaArgentina(cierreData.fechaCierre)}`
        );
        console.log("-------------------------------------");
        console.log("VENTAS POR MÉTODO DE PAGO:");

        // Mostrar ventas por método de pago
        if (cierreData.ventasPorMetodo) {
          Object.entries(cierreData.ventasPorMetodo).forEach(
            ([metodo, total]) => {
              console.log(
                `${metodo.toUpperCase()}: $${Number(total).toLocaleString()}`
              );
            }
          );
        }

        console.log("-------------------------------------");
        console.log("VENTAS POR VENDEDOR:");

        // Mostrar ventas por vendedor
        if (
          cierreData.ventasPorVendedor &&
          Array.isArray(cierreData.ventasPorVendedor)
        ) {
          cierreData.ventasPorVendedor.forEach((vendedor: any) => {
            console.log(
              `${vendedor.nombre}: $${Number(
                vendedor.totalVentas
              ).toLocaleString()} (${vendedor.cantidadVentas} ventas)`
            );

            // Mostrar métodos de pago por vendedor si existen
            if (vendedor.metodosPago) {
              Object.entries(vendedor.metodosPago).forEach(
                ([metodo, total]) => {
                  console.log(
                    `  ${metodo.toUpperCase()}: $${Number(
                      total
                    ).toLocaleString()}`
                  );
                }
              );
            }
          });
        }

        console.log("-------------------------------------");
        console.log(
          `TOTAL GENERAL: $${Number(
            cierreData.totalVentas
          ).toLocaleString()} (${cierreData.cantidadVentas} ventas)`
        );
        console.log("=====================================\n");

        // Intentar imprimir - usar window.printer API específica
        try {
          if (typeof window !== "undefined" && window.printer?.printClosing) {
            console.log(
              "📄 Enviando datos para impresión de cierre:",
              cierreData
            );
            const result = await window.printer.printClosing(cierreData);
            console.log("📄 Resultado de impresión de cierre:", result);

            if (result.success && !result.printerError) {
              toast.success("Ticket de cierre impreso correctamente");
            } else if (result.printerError) {
              // Error específico de la impresora TP806L - mostrar toast de error pero no fallar
              console.error(
                "❌ Error de impresora TP806L:",
                result.printerError
              );
              toast.error(`Error de impresión: ${result.printerError}`, {
                description:
                  "El cierre se completó correctamente pero no se pudo imprimir el ticket",
              });
            } else {
              // Error general - mostrar toast de error
              console.error(
                "❌ Error general al imprimir cierre:",
                result.message
              );
              toast.error(
                `Error al imprimir el ticket de cierre: ${
                  result.message || "Desconocido"
                }`
              );
            }
          } else {
            throw new Error("API de Electron no disponible");
          }
        } catch (electronError: any) {
          // Si no se puede acceder a Electron, mostrar error específico
          console.error(
            "❌ Error al acceder a Electron para cierre:",
            electronError
          );
          toast.error("Error de conexión con la impresora", {
            description:
              "No se pudo conectar con el sistema de impresión para el cierre",
          });
        }

        toast.success(`Cierre de ${period} realizado correctamente`);
        setClosingDialogOpen(false);
      } catch (printError: any) {
        console.error("Error al imprimir cierre:", printError);
        toast.success(`Cierre de ${period} realizado correctamente`);
        setClosingDialogOpen(false);
      }
    } catch (error: any) {
      toast.error(`Error: ${error.message || "Error al realizar el cierre"}`);
    } finally {
      setIsClosing(false);
    }
  };

  return {
    isClosing,
    closingDialogOpen,
    setClosingDialogOpen,
    handleClosing,
    formatFechaArgentina,
  };
};
