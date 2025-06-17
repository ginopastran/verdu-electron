import { useState } from "react";
import { toast } from "sonner";

export const useClosing = (
  user: any,
  API_URL: string,
  appId: string | null
) => {
  const [isClosing, setIsClosing] = useState(false);
  const [closingDialogOpen, setClosingDialogOpen] = useState(false);

  // Helper para llamadas a Electron IPC
  const getElectronAPI = () => {
    if (typeof window !== "undefined" && window.require) {
      return window.require("electron");
    }
    return null;
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
      const getUTCDate = (hoursArg: number) => {
        const date = new Date();
        date.setUTCHours(hoursArg + 3, 0, 0, 0);
        return date.toISOString();
      };

      let startDate;
      if (period === "mañana") {
        startDate = getUTCDate(6);
      } else if (period === "tarde") {
        startDate = getUTCDate(12);
      } else {
        startDate = getUTCDate(0);
      }

      console.log(`🕒 Fecha inicio (${period}) UTC:`, startDate);
      console.log(
        `🕒 Fecha inicio (${period}) hora Argentina:`,
        formatFechaArgentina(startDate)
      );

      const closingData = {
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        fechaInicio: startDate,
        fechaCierre: new Date().toISOString(),
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
        const electronAPI = getElectronAPI();

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

        if (electronAPI) {
          const { ipcRenderer } = electronAPI;
          const result = await ipcRenderer.invoke("print-closing", cierreData);

          if (result.success && !result.printerError) {
            toast.success("Ticket de cierre impreso correctamente");
          }
        } else {
          console.log("🌐 Modo desarrollo: simulando impresión de cierre");
          toast.success("Cierre simulado (modo desarrollo)");
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
