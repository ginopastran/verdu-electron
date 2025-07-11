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
import { useEffect } from "react";

interface QRPaymentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  paymentProcessor: any;
  // 🆕 NUEVO: Props para AFIP
  afipPaymentProcessor?: any;
  isAfipMode?: boolean;
  cartItems?: any[];
  // ✅ NUEVO: Agregar businessInfo para verificar facturación
  businessInfo?: any;
  // ✅ DEBUGGING: Info adicional para logging
  debugInfo?: any;
}

export const QRPaymentDialog = ({
  open,
  onOpenChange,
  paymentProcessor,
  afipPaymentProcessor,
  isAfipMode = false,
  cartItems = [],
  businessInfo,
  debugInfo,
}: QRPaymentDialogProps) => {
  // ✅ DEBUG: Verificar estado del flujo AFIP
  // console.log("🔍 QRPaymentDialog - isAfipMode:", isAfipMode);
  // console.log(
  //   "🔍 QRPaymentDialog - facturacionHabilitada:",
  //   businessInfo?.facturacionHabilitada
  // );

  // ✅ NUEVA VERIFICACIÓN: Solo activar AFIP si está realmente habilitado
  const shouldUseAfipMode =
    isAfipMode && businessInfo?.facturacionHabilitada === true;
  // console.log("🔍 QRPaymentDialog - shouldUseAfipMode:", shouldUseAfipMode);

  const qrData = isAfipMode
    ? afipPaymentProcessor?.qrData
    : paymentProcessor.qrData;
  const paymentStatus = isAfipMode
    ? afipPaymentProcessor?.paymentStatus
    : paymentProcessor.paymentStatus;

  // ✅ NUEVO: Iniciar polling automáticamente cuando se abre el diálogo con datos de QR
  useEffect(() => {
    if (open && paymentProcessor.qrData?.orderId) {
      console.log(
        "🔄 QRPaymentDialog: Iniciando polling automático para orden:",
        paymentProcessor.qrData.orderId
      );

      if (paymentProcessor.qrData.isSplitPayment) {
        // Para pago mixto, usar el polling específico
        paymentProcessor.startSplitPaymentStatusPolling(
          paymentProcessor.qrData.orderId,
          paymentProcessor.qrData.cashAmount || 0
        );
      } else {
        // Para pago normal, usar el polling regular
        paymentProcessor.startPaymentStatusPolling(
          paymentProcessor.qrData.orderId
        );
      }
    }
  }, [open, paymentProcessor.qrData?.orderId]);

  // 🆕 NUEVO: Manejar finalización de pago con AFIP
  const handlePaymentSuccess = async (paymentData: any) => {
    console.log("🔔 PAYMENT SUCCESS TRIGGERED - Estado completo:", {
      isAfipMode,
      facturacionHabilitada: businessInfo?.facturacionHabilitada,
      shouldUseAfipMode,
      afipPaymentProcessorExists: !!afipPaymentProcessor,
      cartItemsLength: cartItems.length,
      paymentData: {
        orderId: paymentData.orderId,
        isCompleted: paymentData.isCompleted,
      },
    });

    // ✅ VERIFICACIÓN ROBUSTA: Solo crear factura AFIP si está realmente habilitado
    if (shouldUseAfipMode && afipPaymentProcessor && cartItems.length > 0) {
      console.log(
        "🧾✅ QR Dialog: Pago exitoso en modo AFIP, creando factura..."
      );
      try {
        await afipPaymentProcessor.createAfipInvoiceAfterPayment(
          paymentData.orderId,
          cartItems,
          paymentData
        );
        console.log("✅ QR Dialog: Factura AFIP creada exitosamente");
      } catch (error) {
        console.error("❌ QR Dialog: Error al crear factura AFIP:", error);
        // El error ya se maneja en createAfipInvoiceAfterPayment
      }
    }
  };

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
            {isAfipMode
              ? "Escanea el código QR para pagar y generar factura AFIP"
              : "Escanea el código QR para pagar"}
          </DialogTitle>
          <DialogDescription>
            {isAfipMode
              ? "Usa la app de Mercado Pago para escanear. La factura AFIP se generará automáticamente después del pago."
              : "Usa la app de Mercado Pago para escanear"}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col items-center justify-center py-4">
          {qrData ? (
            <div className="space-y-4 text-center">
              <img
                src={qrData.qrImageUrl}
                alt="Código QR de Mercado Pago"
                className="mx-auto w-64 h-64 border border-gray-200 p-2"
                onError={(e) => {
                  console.error("Error loading QR image");
                  e.currentTarget.style.display = "none";
                }}
              />

              <div className="font-medium text-lg">
                {qrData.isSplitPayment ? (
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
                        ${Number(qrData.cashAmount || 0).toLocaleString()}
                      </span>
                    </div>
                    <div className="flex justify-between items-center font-semibold">
                      <span className="flex items-center">
                        <QrCode className="h-4 w-4 mr-1" />
                        QR:
                      </span>
                      <span>${Number(qrData.monto || 0).toLocaleString()}</span>
                    </div>
                  </div>
                ) : (
                  <span>
                    Monto a pagar: $
                    {Number(qrData?.monto || 0).toLocaleString()}
                  </span>
                )}
              </div>

              <div className="space-y-2">
                <div
                  className={`text-center py-2 px-4 rounded-full font-medium ${
                    paymentStatus === "PENDIENTE"
                      ? "bg-yellow-100 text-yellow-800"
                      : paymentStatus === "COMPLETADA"
                      ? "bg-green-100 text-green-800"
                      : paymentStatus === "CANCELADA"
                      ? "bg-red-100 text-red-800"
                      : "bg-gray-100 text-gray-800"
                  }`}
                >
                  Estado:{" "}
                  {paymentStatus === "PENDIENTE"
                    ? "Esperando pago..."
                    : paymentStatus === "COMPLETADA"
                    ? "¡Pago completado!"
                    : paymentStatus === "CANCELADA"
                    ? "Pago cancelado"
                    : "Desconocido"}
                  {paymentStatus === "PENDIENTE" && (
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
                {paymentStatus === "PENDIENTE" && (
                  <Button
                    variant="outline"
                    className="w-full mt-4 bg-emerald-50 border-emerald-200 hover:bg-emerald-100 hover:text-emerald-700"
                    onClick={async () => {
                      // Completar orden manualmente
                      await paymentProcessor.completarOrdenManualmente(
                        qrData.orderId,
                        qrData.isSplitPayment,
                        qrData.cashAmount
                      );

                      // 🆕 NUEVO: Si es modo AFIP Y facturación está habilitada, crear factura después
                      if (
                        shouldUseAfipMode &&
                        afipPaymentProcessor &&
                        cartItems.length > 0
                      ) {
                        console.log(
                          "🧾📱 QR Dialog: Completado manualmente en modo AFIP, creando factura..."
                        );
                        try {
                          await afipPaymentProcessor.createAfipInvoiceAfterPayment(
                            qrData.orderId,
                            cartItems,
                            {
                              payment_id: `manual_${qrData.orderId}`,
                              orderId: qrData.orderId,
                              isManual: true,
                            }
                          );
                        } catch (error) {
                          console.error(
                            "❌ QR Dialog: Error al crear factura AFIP manual:",
                            error
                          );
                        }
                      }
                    }}
                  >
                    <Receipt className="h-4 w-4 mr-2" />
                    {isAfipMode
                      ? "Completar manualmente y generar factura"
                      : "Completar manualmente"}
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
