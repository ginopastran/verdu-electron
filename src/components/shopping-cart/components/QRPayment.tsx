import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Receipt } from "lucide-react";
import { useCartContext } from "../contexts/CartContext";
import { usePaymentContext } from "../contexts/PaymentContext";
import QRCode from "qrcode";

interface QRPaymentProps {
  open: boolean;
  onClose: () => void;
  isSplitPayment?: boolean;
  cashAmount?: number;
}

export function QRPayment({
  open,
  onClose,
  isSplitPayment = false,
  cashAmount = 0,
}: QRPaymentProps) {
  const [qrImageUrl, setQrImageUrl] = useState<string | null>(null);
  const [paymentStatus, setPaymentStatus] = useState<string>("PENDIENTE");
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(
    null
  );
  const [pollingStartTime, setPollingStartTime] = useState<number | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  const { getCurrentItems, calculateTotal, clearCurrentCart } =
    useCartContext();
  const { handleQRPayment, checkPaymentStatus, completeOrder } =
    usePaymentContext();

  useEffect(() => {
    if (open) {
      generateQR();
    } else {
      cleanupQR();
    }
  }, [open]);

  const generateQR = async () => {
    try {
      const items = getCurrentItems();
      const total = calculateTotal();
      const qrAmount = isSplitPayment ? total - cashAmount : total;

      const { orderId, qrData } = await handleQRPayment(items, qrAmount);

      // Generar QR localmente
      const qrImageDataUrl = await QRCode.toDataURL(qrData, {
        errorCorrectionLevel: "H",
        margin: 1,
        width: 256,
      });

      setQrImageUrl(qrImageDataUrl);
      startPolling(orderId);
    } catch (error) {
      console.error("Error al generar QR:", error);
    }
  };

  const startPolling = (orderId: number) => {
    const POLLING_INTERVAL = 3000; // 3 segundos
    const MAX_POLLING_TIME = 10 * 60 * 1000; // 10 minutos
    const MAX_RETRIES = 3;

    setPollingStartTime(Date.now());
    setRetryCount(0);

    const interval = setInterval(async () => {
      try {
        // Verificar timeout
        if (
          pollingStartTime &&
          Date.now() - pollingStartTime > MAX_POLLING_TIME
        ) {
          cleanupQR();
          return;
        }

        const statusData = await checkPaymentStatus(orderId);
        setRetryCount(0);
        setPaymentStatus(statusData.status);

        if (statusData.status === "COMPLETADO") {
          await completeOrder(orderId);
          clearCurrentCart();
          onClose();
        } else if (statusData.status === "CANCELADO") {
          cleanupQR();
        }
      } catch (error) {
        setRetryCount((prev) => prev + 1);
        if (retryCount >= MAX_RETRIES) {
          cleanupQR();
        }
      }
    }, POLLING_INTERVAL);

    setPollingInterval(interval);
  };

  const cleanupQR = () => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      setPollingInterval(null);
    }
    setQrImageUrl(null);
    setPaymentStatus("PENDIENTE");
    setPollingStartTime(null);
    setRetryCount(0);
  };

  const handleManualComplete = async () => {
    // Aquí iría la lógica para completar manualmente
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Escanea el código QR para pagar</DialogTitle>
        </DialogHeader>

        <div className="flex flex-col items-center justify-center py-4">
          {qrImageUrl ? (
            <div className="space-y-4 text-center">
              <img
                src={qrImageUrl}
                alt="Código QR de Mercado Pago"
                className="mx-auto w-64 h-64 border border-gray-200 p-2"
              />

              <div className="font-medium text-lg">
                {isSplitPayment ? (
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">
                      Pago mixto:
                    </div>
                    <div className="flex justify-between items-center text-base">
                      <span>Efectivo:</span>
                      <span>${cashAmount.toLocaleString()}</span>
                    </div>
                    <div className="flex justify-between items-center font-semibold">
                      <span>QR:</span>
                      <span>
                        ${(calculateTotal() - cashAmount).toLocaleString()}
                      </span>
                    </div>
                  </div>
                ) : (
                  <span>
                    Monto a pagar: ${calculateTotal().toLocaleString()}
                  </span>
                )}
              </div>

              <div className="space-y-2">
                <div
                  className={`text-center py-2 px-4 rounded-full font-medium ${
                    paymentStatus === "PENDIENTE"
                      ? "bg-yellow-100 text-yellow-800"
                      : paymentStatus === "COMPLETADO"
                      ? "bg-green-100 text-green-800"
                      : paymentStatus === "CANCELADO"
                      ? "bg-red-100 text-red-800"
                      : "bg-gray-100 text-gray-800"
                  }`}
                >
                  Estado: {paymentStatus}
                </div>

                <Button
                  variant="outline"
                  className="w-full mt-4 bg-emerald-50 border-emerald-200 hover:bg-emerald-100 hover:text-emerald-700"
                  onClick={handleManualComplete}
                >
                  <Receipt className="h-4 w-4 mr-2" />
                  Completar manualmente
                </Button>
              </div>
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-64">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900"></div>
              <p className="mt-4">Generando código QR...</p>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancelar
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
