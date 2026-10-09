import { useState } from "react";
import { toast } from "sonner";

interface CancellationData {
  businessId: number;
  tipo: "orden" | "factura";
  referenciaId: string;
  motivo: string;
  monto: number;
  vendedorId?: number;
  sucursalId?: number;
  productos: Array<{
    productoId: number;
    nombreProducto: string;
    cantidad: number;
    precioUnitario: number;
    subtotal: number;
  }>;
}

interface UseCancellationControlProps {
  user: any;
  API_URL: string;
  appId: string | null;
  businessInfo: any;
}

export function useCancellationControl({
  user,
  API_URL,
  appId,
  businessInfo,
}: UseCancellationControlProps) {
  const [isCancelling, setIsCancelling] = useState(false);
  const [cancellationDialogOpen, setCancellationDialogOpen] = useState(false);
  const [cancellationReason, setCancellationReason] = useState("");
  const [pendingCancellation, setPendingCancellation] = useState<{
    tipo: "orden" | "factura";
    referenciaId: string;
    monto: number;
    productos: Array<{
      productoId: number;
      nombreProducto: string;
      cantidad: number;
      precioUnitario: number;
      subtotal: number;
    }>;
  } | null>(null);

  // Verificar si las cancelaciones están habilitadas
  const isCancellationEnabled = businessInfo?.cancelacionHabilitada === true;

  // Función para registrar cancelación usando la API unificada
  const registerCancellation = async (cancellationData: CancellationData) => {
    try {
      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };

      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      console.log("📋 Enviando cancelación a API unificada:", cancellationData);

      const response = await fetch(`${API_URL}/api/cancelaciones`, {
        method: "POST",
        headers,
        body: JSON.stringify(cancellationData),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || "Error al registrar cancelación");
      }

      const result = await response.json();
      console.log("✅ Cancelación registrada exitosamente:", result);
      return result;
    } catch (error: any) {
      console.error("❌ Error al registrar cancelación:", error);
      throw error;
    }
  };

  // Función para iniciar el proceso de cancelación
  const initiateCancellation = (
    tipo: "orden" | "factura",
    referenciaId: string,
    monto: number,
    productos: Array<{
      productoId: number;
      nombreProducto: string;
      cantidad: number;
      precioUnitario: number;
      subtotal: number;
    }>
  ) => {
    if (!isCancellationEnabled) {
      console.log("ℹ️ Cancelaciones no habilitadas, usando flujo normal");
      return false; // Indicar que debe usar el flujo normal
    }

    if (!user) {
      toast.error("Debes iniciar sesión para cancelar");
      return false;
    }

    console.log(
      `🔄 Iniciando cancelación de ${tipo} ID: ${referenciaId}, monto: $${monto}, productos: ${productos.length}`
    );
    setPendingCancellation({ tipo, referenciaId, monto, productos });
    setCancellationDialogOpen(true);
    return true; // Indicar que se está usando el flujo de cancelación controlada
  };

  // Función para confirmar la cancelación
  const confirmCancellation = async () => {
    if (!pendingCancellation || !cancellationReason.trim()) {
      toast.error("Debes ingresar un motivo para la cancelación");
      return;
    }

    if (cancellationReason.trim().length < 3) {
      toast.error("El motivo debe tener al menos 3 caracteres");
      return;
    }

    if (!user) {
      toast.error("Debes iniciar sesión para cancelar");
      return;
    }

    setIsCancelling(true);

    try {
      const cancellationData: CancellationData = {
        businessId: businessInfo?.id || user.businessId || 1,
        tipo: pendingCancellation.tipo,
        referenciaId: pendingCancellation.referenciaId,
        motivo: cancellationReason.trim(),
        monto: pendingCancellation.monto,
        vendedorId: user.id,
        sucursalId: user.sucursalId || 1,
        productos: pendingCancellation.productos,
      };

      console.log(
        "📋 Datos de cancelación para API unificada:",
        cancellationData
      );

      // Registrar la cancelación usando la API unificada
      const result = await registerCancellation(cancellationData);

      // Mostrar confirmación con ID real si está disponible
      const idReal = result?.cancelacion?.idReal || result?.id || "N/A";
      toast.success(`Cancelación #${idReal} registrada exitosamente`);

      // Limpiar estados
      setCancellationDialogOpen(false);
      setCancellationReason("");
      setPendingCancellation(null);

      return true; // Indicar éxito
    } catch (error: any) {
      console.error("❌ Error en confirmación de cancelación:", error);
      toast.error(`Error al registrar cancelación: ${error.message}`);
      return false; // Indicar error
    } finally {
      setIsCancelling(false);
    }
  };

  // Función para cancelar el proceso de cancelación
  const cancelCancellationProcess = () => {
    setCancellationDialogOpen(false);
    setCancellationReason("");
    setPendingCancellation(null);
    console.log("❌ Proceso de cancelación cancelado por el usuario");
  };

  return {
    // Estados
    isCancellationEnabled,
    isCancelling,
    cancellationDialogOpen,
    cancellationReason,
    pendingCancellation,

    // Funciones
    initiateCancellation,
    confirmCancellation,
    cancelCancellationProcess,
    setCancellationDialogOpen,
    setCancellationReason,
  };
}
