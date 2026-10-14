/// <reference path="../types/electron.d.ts" />

import { useState } from "react";
import { toast } from "sonner";
import { datosDeCierre, efectivoEsperado, textoArqueo } from "@/utils/reglasVenta";

export function avisarArqueo(ventasPorMetodo: unknown, contado: number | null) {
  if (contado == null) return;
  const esperado = efectivoEsperado(ventasPorMetodo);
  const { texto, estado } = textoArqueo(contado, esperado);
  const description = `Esperado en efectivo $${esperado.toLocaleString("es-AR")} · Contado $${contado.toLocaleString("es-AR")}`;
  if (estado === "cuadra") toast.success(texto, { description, duration: 20000 });
  else toast.warning(texto, { description, duration: 20000 });
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

  const handleClosing = async (
    period: string,
    efectivoContado: number | null = null
  ) => {
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
      const closingData = {
        ...datosDeCierre(user.id, user.sucursalId, period),
        ...(efectivoContado != null && { efectivoContado }),
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
      avisarArqueo(cierreData.ventasPorMetodo, efectivoContado);

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
        console.log(
          `TOTAL VENTAS NORMALES: $${Number(
            cierreData.totalVentas
          ).toLocaleString()} (${cierreData.cantidadVentas} ventas)`
        );

        // ✅ Mostrar ventas por cuenta corriente si existen (solo totales, sin lista detallada)
        const cuentaCorriente = cierreData.ventas?.cuentaCorriente || cierreData.cuentaCorriente;
        if (cuentaCorriente && cuentaCorriente.total > 0) {
          console.log("-------------------------------------");
          console.log("VENTAS POR CUENTA CORRIENTE:");
          
          // Mostrar métodos de pago de cuenta corriente si están disponibles
          if (cuentaCorriente.ventasPorMetodo) {
            console.log("MÉTODOS DE PAGO:");
            Object.entries(cuentaCorriente.ventasPorMetodo).forEach(
              ([metodo, total]: [string, any]) => {
                if (Number(total) > 0) {
                  console.log(
                    `  ${metodo.toUpperCase()}: $${Number(total).toLocaleString()}`
                  );
                }
              }
            );
            console.log("-------------------------------------");
          }
          
          // Mostrar ventas por vendedor de cuenta corriente si hay datos
          if (cuentaCorriente.ventasPorVendedor && Array.isArray(cuentaCorriente.ventasPorVendedor) && cuentaCorriente.ventasPorVendedor.length > 0) {
            console.log("VENTAS POR VENDEDOR:");
            cuentaCorriente.ventasPorVendedor.forEach((vendedor: any) => {
              console.log(`  ${vendedor.nombre.toUpperCase()}`);
              console.log(`  Email: ${vendedor.email}`);
              if (vendedor.metodosPago) {
                Object.entries(vendedor.metodosPago).forEach(
                  ([metodo, total]: [string, any]) => {
                    if (Number(total) > 0) {
                      console.log(`  ${metodo.toUpperCase()}: $${Number(total).toLocaleString()}`);
                    }
                  }
                );
              }
              console.log(`  Total: $${Number(vendedor.totalVentas).toLocaleString()}`);
              console.log(`  Cantidad: ${vendedor.cantidadVentas}`);
              console.log("-------------------------------------");
            });
          }
          
          console.log(
            `TOTAL CUENTA CORRIENTE: $${Number(
              cuentaCorriente.total
            ).toLocaleString()}`
          );
          console.log(
            `CANT. VENTAS CC: ${cuentaCorriente.cantidad}`
          );
          console.log("-------------------------------------");
        }

        // ✅ Mostrar total general combinado si existe resumen (nueva estructura ventas.resumen)
        const resumen = cierreData.ventas?.resumen || cierreData.resumen;
        if (resumen && resumen.totalCombinado) {
          console.log("-------------------------------------");
          console.log("TOTAL GENERAL");
          console.log("(Cuenta Corriente + Ventas Normales)");
          console.log("-------------------------------------");
          console.log(
            `TOTAL GENERAL: $${Number(
              resumen.totalCombinado
            ).toLocaleString()}`
          );
          console.log(
            `CANT. TOTAL: ${resumen.cantidadTotalCombinada}`
          );
        } else if (cuentaCorriente && cuentaCorriente.total > 0) {
          // Si no hay resumen pero hay cuenta corriente, calcular manualmente
          const totalCombinado = cierreData.totalVentas + cuentaCorriente.total;
          const cantidadCombinada = cierreData.cantidadVentas + cuentaCorriente.cantidad;
          console.log("-------------------------------------");
          console.log("TOTAL GENERAL");
          console.log("(Cuenta Corriente + Ventas Normales)");
          console.log("-------------------------------------");
          console.log(
            `TOTAL GENERAL: $${Number(
              totalCombinado
            ).toLocaleString()}`
          );
          console.log(
            `CANT. TOTAL: ${cantidadCombinada}`
          );
        }
        
        // Ventas por vendedor (normales)
        if (
          cierreData.ventasPorVendedor &&
          Array.isArray(cierreData.ventasPorVendedor)
        ) {
          console.log("\n\n");
          console.log("VENTAS POR VENDEDOR");
          console.log("=============================");
          console.log("");
          
          cierreData.ventasPorVendedor.forEach((vendedor: any) => {
            console.log(vendedor.nombre.toUpperCase());
            console.log(`Email: ${vendedor.email}`);

            // Mostrar métodos de pago por vendedor si existen
            if (vendedor.metodosPago) {
              Object.entries(vendedor.metodosPago).forEach(
                ([metodo, total]) => {
                  if (Number(total) > 0) {
                    const metodoFormateado = metodo.charAt(0).toUpperCase() + metodo.slice(1);
                    console.log(
                      `${metodoFormateado}: $${Number(total).toLocaleString()}`
                    );
                  }
                }
              );
            }
            
            console.log(
              `Total: $${Number(vendedor.totalVentas).toLocaleString()}`
            );
            
            // Mostrar Total con cuenta corriente usando el campo del backend
            const totalConCuentaCorriente = vendedor.totalVentasConCuentaCorriente || vendedor.totalVentas;
            console.log(
              `Total con CC: $${Number(totalConCuentaCorriente).toLocaleString()}`
            );
            
            console.log(`Cantidad: ${vendedor.cantidadVentas}`);
            console.log("-------------------------------------");
          });
        }
        
        console.log("=====================================\n");

        // Intentar imprimir - usar window.printer API específica
        try {
          if (typeof window !== "undefined" && window.printer?.printClosing) {
            const result = await window.printer.printClosing(cierreData);

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
