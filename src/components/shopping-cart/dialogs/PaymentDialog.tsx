import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { CreditCard, Wallet, QrCode } from "lucide-react";

interface PaymentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectPayment: (method: string) => void;
  isProcessingPayment: boolean;
  selectedPaymentMethod: string | null;
  isAfipMode?: boolean;
  discountData?: {
    type: "percentage" | "fixed";
    value: number;
    amount: number;
  } | null;
  subtotal?: number;
}

export function PaymentDialog({
  isOpen,
  onClose,
  onSelectPayment,
  isProcessingPayment,
  selectedPaymentMethod,
  isAfipMode = false,
  discountData = null,
  subtotal = 0,
}: PaymentDialogProps) {
  // console.log("🎯 PaymentDialog render:", {
  //   isOpen,
  //   isProcessingPayment,
  //   selectedPaymentMethod,
  //   isAfipMode,
  // });

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isAfipMode
              ? "Seleccionar método de pago - Factura AFIP"
              : "Seleccionar método de pago"}
          </DialogTitle>
          <DialogDescription>
            {isAfipMode
              ? "Presiona el número correspondiente al método de pago para generar factura AFIP"
              : "Presiona el número correspondiente al método de pago o haz clic en el botón"}
          </DialogDescription>
        </DialogHeader>

        {/* Mostrar información del descuento si existe */}
        {discountData && (
          <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-4">
            <div className="flex justify-between items-center">
              <div>
                <h4 className="font-semibold text-green-800">
                  Descuento Aplicado
                </h4>
                <p className="text-sm text-green-600">
                  {discountData.type === "percentage"
                    ? `${discountData.value}% de descuento`
                    : `$${discountData.value.toFixed(2)} de descuento`}
                </p>
              </div>
              <div className="text-right">
                <p className="text-sm text-gray-600">
                  Subtotal: ${subtotal.toFixed(2)}
                </p>
                <p className="text-sm text-green-600 font-semibold">
                  Descuento: -${discountData.amount.toFixed(2)}
                </p>
                <p className="font-bold text-green-800">
                  Total: ${(subtotal - discountData.amount).toFixed(2)}
                </p>
              </div>
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 gap-4">
          <Button
            onClick={() => onSelectPayment("qr")}
            className={`h-32 flex flex-col border-[#A7A7A7] items-center justify-center space-y-2 [&_svg]:size-8 ${
              selectedPaymentMethod === "qr"
                ? "bg-emerald-100 border-emerald-600 border-2"
                : ""
            }`}
            variant="outline"
            disabled={
              isProcessingPayment ||
              (selectedPaymentMethod !== null && selectedPaymentMethod !== "qr")
            }
          >
            <div className="h-12 flex items-center justify-center">
              {isProcessingPayment && selectedPaymentMethod === "qr" ? (
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
              ) : (
                <QrCode className="h-12 w-12" />
              )}
            </div>
            <span>Transferencia (1)</span>
          </Button>
          <Button
            onClick={() => onSelectPayment("tarjeta")}
            className={`h-32 flex flex-col items-center border-[#A7A7A7] justify-center space-y-2 [&_svg]:size-8 ${
              selectedPaymentMethod === "tarjeta"
                ? "bg-emerald-100 border-emerald-600 border-2"
                : ""
            }`}
            variant="outline"
            disabled={
              isProcessingPayment ||
              (selectedPaymentMethod !== null &&
                selectedPaymentMethod !== "tarjeta")
            }
          >
            <div className="h-12 flex items-center justify-center">
              {isProcessingPayment && selectedPaymentMethod === "tarjeta" ? (
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
              ) : (
                <CreditCard className="h-12 w-12" />
              )}
            </div>
            <span>Tarjeta (2)</span>
          </Button>
          <Button
            onClick={() => onSelectPayment("efectivo")}
            className={`h-32 flex flex-col border-[#A7A7A7] items-center justify-center space-y-2 [&_svg]:size-8 ${
              selectedPaymentMethod === "efectivo"
                ? "bg-emerald-100 border-emerald-600 border-2"
                : ""
            }`}
            variant="outline"
            disabled={
              isProcessingPayment ||
              (selectedPaymentMethod !== null &&
                selectedPaymentMethod !== "efectivo")
            }
          >
            <div className="h-12 flex items-center justify-center">
              {isProcessingPayment && selectedPaymentMethod === "efectivo" ? (
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
              ) : (
                <Wallet className="h-12 w-12" />
              )}
            </div>
            <span>Efectivo (3)</span>
          </Button>
          <Button
            onClick={() => onSelectPayment("split")}
            className={`h-32 flex flex-col items-center border-[#A7A7A7] justify-center space-y-2 [&_svg]:size-8 ${
              selectedPaymentMethod === "split"
                ? "bg-emerald-100 border-emerald-600 border-2"
                : ""
            }`}
            variant="outline"
            disabled={
              isProcessingPayment ||
              (selectedPaymentMethod !== null &&
                selectedPaymentMethod !== "split")
            }
          >
            <div className="h-12 flex items-center justify-center">
              {isProcessingPayment && selectedPaymentMethod === "split" ? (
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
              ) : (
                <div className="flex items-center gap-1">
                  <Wallet className="h-10 w-10" />
                  <span className="text-xl">+</span>
                  <CreditCard className="h-10 w-10" />
                </div>
              )}
            </div>
            <span>Pago Mixto (4)</span>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
