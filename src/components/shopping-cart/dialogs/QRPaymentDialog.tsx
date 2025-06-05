import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Wallet, QrCode, Receipt } from "lucide-react";

interface QRPaymentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  paymentProcessor: any;
}

export const QRPaymentDialog = ({
  open,
  onOpenChange,
  paymentProcessor,
}: QRPaymentDialogProps) => {
  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        console.log("🔄 QR Dialog onOpenChange:", open);
        if (!open) {
          console.log("🔄 Cerrando diálogo QR, cancelando pago");
          onOpenChange(false);
          paymentProcessor.cancelQRPayment();
        }
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="text-xl">
            Escanea el código QR para pagar
          </DialogTitle>
          <DialogDescription>
            Usa la app de Mercado Pago para escanear
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col items-center justify-center py-4">
          {paymentProcessor.qrData ? (
            <div className="space-y-4 text-center">
              <img
                src={paymentProcessor.qrData.qrImageUrl}
                alt="Código QR de Mercado Pago"
                className="mx-auto w-64 h-64 border border-gray-200 p-2"
                onError={(e) => {
                  console.error("Error loading QR image");
                  e.currentTarget.style.display = "none";
                }}
              />

              <div className="font-medium text-lg">
                {paymentProcessor.qrData.isSplitPayment ? (
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">
                      Pago mixto:
                    </div>
                    <div className="flex justify-between items-center text-base">
                      <span className="flex items-center">
                        <Wallet className="h-4 w-4 mr-1" />
                        Efectivo:
                      </span>
                      <span>
                        $
                        {Number(
                          paymentProcessor.qrData.cashAmount || 0
                        ).toLocaleString()}
                      </span>
                    </div>
                    <div className="flex justify-between items-center font-semibold">
                      <span className="flex items-center">
                        <QrCode className="h-4 w-4 mr-1" />
                        QR:
                      </span>
                      <span>
                        $
                        {Number(
                          paymentProcessor.qrData.monto || 0
                        ).toLocaleString()}
                      </span>
                    </div>
                  </div>
                ) : (
                  <span>
                    Monto a pagar: $
                    {Number(
                      paymentProcessor.qrData.monto || 0
                    ).toLocaleString()}
                  </span>
                )}
              </div>

              <div className="space-y-2">
                <div
                  className={`text-center py-2 px-4 rounded-full font-medium ${
                    paymentProcessor.paymentStatus === "PENDIENTE"
                      ? "bg-yellow-100 text-yellow-800"
                      : paymentProcessor.paymentStatus === "COMPLETADA"
                      ? "bg-green-100 text-green-800"
                      : paymentProcessor.paymentStatus === "CANCELADA"
                      ? "bg-red-100 text-red-800"
                      : "bg-gray-100 text-gray-800"
                  }`}
                >
                  Estado:{" "}
                  {paymentProcessor.paymentStatus === "PENDIENTE"
                    ? "Esperando pago..."
                    : paymentProcessor.paymentStatus === "COMPLETADA"
                    ? "¡Pago completado!"
                    : paymentProcessor.paymentStatus === "CANCELADA"
                    ? "Pago cancelado"
                    : "Desconocido"}
                  {paymentProcessor.paymentStatus === "PENDIENTE" && (
                    <span className="inline-block ml-2">
                      <div className="animate-pulse w-2 h-2 bg-yellow-500 rounded-full inline-block mx-0.5"></div>
                      <div className="animate-pulse delay-150 w-2 h-2 bg-yellow-500 rounded-full inline-block mx-0.5"></div>
                      <div className="animate-pulse delay-300 w-2 h-2 bg-yellow-500 rounded-full inline-block mx-0.5"></div>
                    </span>
                  )}
                </div>

                <p className="text-sm text-muted-foreground">
                  No cierres esta ventana hasta que el pago sea completado
                </p>

                {/* Botón para completar manualmente */}
                {paymentProcessor.paymentStatus === "PENDIENTE" && (
                  <Button
                    variant="outline"
                    className="w-full mt-4 bg-emerald-50 border-emerald-200 hover:bg-emerald-100 hover:text-emerald-700"
                    onClick={() =>
                      paymentProcessor.completarOrdenManualmente(
                        paymentProcessor.qrData.orderId,
                        paymentProcessor.qrData.isSplitPayment,
                        paymentProcessor.qrData.cashAmount
                      )
                    }
                  >
                    <Receipt className="h-4 w-4 mr-2" />
                    Completar manualmente
                  </Button>
                )}
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
          <Button
            variant="outline"
            onClick={paymentProcessor.cancelQRPayment}
            className="w-full"
          >
            Cancelar
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
