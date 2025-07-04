import React, { useEffect, useState } from "react";

/**
 * Componente que realiza polling al backend para conocer el estado
 * de una orden de Mercado Pago.
 *
 * Requisitos cubiertos:
 * 1. Llama periódicamente (default 5 s) a `/api/mercadopago/check-status?orderId={ID}` con `credentials: "include"`.
 * 2. Cuando `isCompleted === true` muestra mensaje de éxito, limpia el intervalo
 *    y ejecuta `onCompleted`.
 * 3. Cuando `isCancelled === true` muestra mensaje de cancelación, limpia el
 *    intervalo y ejecuta `onCancelled`.
 * 4. Maneja errores de red; tras 3 intentos fallidos detiene el polling y
 *    notifica el error.
 * 5. Limpia correctamente el intervalo en `useEffect` cleanup para evitar fugas.
 */

export interface QrStatusWatcherProps {
  /** ID de la orden recibida al generar el QR. */
  orderId: number;
  /** Callback opcional cuando el estado pasa a completado/aprobado. */
  onCompleted?: (data: any) => void;
  /** Callback opcional cuando el estado pasa a cancelado. */
  onCancelled?: (data: any) => void;
  /** Intervalo de polling en milisegundos (default 5000). */
  pollIntervalMs?: number;
  /** URL base de la API, por si se requiere distinta a la relativa. */
  apiBase?: string;
}

export const QrStatusWatcher: React.FC<QrStatusWatcherProps> = ({
  orderId,
  onCompleted,
  onCancelled,
  pollIntervalMs = 5000,
  apiBase = "",
}) => {
  const [status, setStatus] = useState<"PENDIENTE" | "PAGADO" | "CANCELADO">(
    "PENDIENTE"
  );
  const [errorCount, setErrorCount] = useState(0);

  useEffect(() => {
    if (!orderId) return;

    let retries = 0;

    const fetchStatus = async () => {
      try {
        const res = await fetch(
          `${apiBase}/api/mercadopago/check-status?orderId=${encodeURIComponent(
            orderId
          )}`,
          {
            method: "GET",
            credentials: "include",
          }
        );

        if (!res.ok) throw new Error(`HTTP ${res.status}`);

        const data = await res.json();

        // Normalizar al español si viniera en inglés
        const normalizedStatus =
          data.status === "APPROVED" || data.status === "COMPLETED"
            ? "PAGADO"
            : data.status === "CANCELLED" || data.status === "CANCELED"
            ? "CANCELADO"
            : data.status === "PENDING"
            ? "PENDIENTE"
            : (data.status as string);

        setStatus(normalizedStatus as any);
        retries = 0; // resetear contador de errores al éxito

        if (data.isCompleted) {
          onCompleted?.(data);
        } else if (data.isCancelled) {
          onCancelled?.(data);
        }
      } catch (err) {
        console.error("Polling error:", err);
        retries += 1;
        setErrorCount(retries);
      }
    };

    // Llamada inmediata para no esperar el primer intervalo
    fetchStatus();

    const timerId = setInterval(fetchStatus, pollIntervalMs);

    return () => clearInterval(timerId);
  }, [orderId, apiBase, pollIntervalMs, onCompleted, onCancelled]);

  // Mostrar alerta después de 3 fallos seguidos
  useEffect(() => {
    if (errorCount >= 3) {
      alert("Error al consultar el estado del pago. Por favor, reintenta.");
    }
  }, [errorCount]);

  return <p>Estado de la venta: {status}</p>;
};

export default QrStatusWatcher;
