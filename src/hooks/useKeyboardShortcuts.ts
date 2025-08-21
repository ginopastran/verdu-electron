import { useEffect } from "react";
import { toast } from "sonner";

interface UseKeyboardShortcutsProps {
  paymentDialogOpen: boolean;
  afipPaymentDialogOpen: boolean;
  isProcessingPayment: boolean;
  selectedPaymentMethod: string | null;
  afipIsProcessingPayment: boolean;
  afipSelectedPaymentMethod: string | null;
  handlePayment: (method: string) => void;
  handleAfipPayment: (method: string) => void;
  handleLogout: () => void;
  handleCancelClick: () => void;
  handlePaymentClick: () => void;
  handleAfipPaymentClick: () => void;
  handleFacturaClick: () => void;
  getCurrentItems: () => any[];
  calculateTotal: () => number;
  businessInfo: any;
}

export const useKeyboardShortcuts = ({
  paymentDialogOpen,
  afipPaymentDialogOpen,
  isProcessingPayment,
  selectedPaymentMethod,
  afipIsProcessingPayment,
  afipSelectedPaymentMethod,
  handlePayment,
  handleAfipPayment,
  handleLogout,
  handleCancelClick,
  handlePaymentClick,
  handleAfipPaymentClick,
  handleFacturaClick,
  getCurrentItems,
  calculateTotal,
  businessInfo,
}: UseKeyboardShortcutsProps) => {
  useEffect(() => {
    const handleGlobalKeyPress = (e: KeyboardEvent) => {
      // Handle F4 for logout
      if (e.key === "F4") {
        e.preventDefault();
        handleLogout();
        return;
      }

      // Para el diálogo de pago normal
      if (paymentDialogOpen) {
        // Si ya se está procesando o hay un método seleccionado, ignorar teclas
        if (isProcessingPayment || selectedPaymentMethod) {
          return;
        }
        switch (e.key) {
          case "1":
            e.preventDefault();
            handlePayment("qr");
            break;
          case "2":
            e.preventDefault();
            handlePayment("tarjeta");
            break;
          case "3":
            e.preventDefault();
            handlePayment("efectivo");
            break;
          case "4":
            e.preventDefault();
            handlePayment("split");
            break;
        }
      }
      // Para el diálogo de pago AFIP
      else if (afipPaymentDialogOpen) {
        // Si ya se está procesando o hay un método seleccionado, ignorar teclas
        if (afipIsProcessingPayment || afipSelectedPaymentMethod) {
          return;
        }
        switch (e.key) {
          case "1":
            e.preventDefault();
            handleAfipPayment("qr");
            break;
          case "2":
            e.preventDefault();
            handleAfipPayment("tarjeta");
            break;
          case "3":
            e.preventDefault();
            handleAfipPayment("efectivo");
            break;
          case "4":
            e.preventDefault();
            handleAfipPayment("split");
            break;
        }
      } else {
        // Teclas para el carrito, siempre deben funcionar independientemente del foco
        switch (e.key) {
          case "F1":
            e.preventDefault();
            if (getCurrentItems().length === 0) {
              toast.error("No hay productos en el carrito", {
                description: "El carrito ya está vacío",
              });
              return;
            }
            handleCancelClick();
            break;
          case "F2":
            e.preventDefault();
            if (getCurrentItems().length === 0) {
              toast.error("No hay productos en el carrito", {
                description: "Agrega al menos un producto antes de continuar",
              });
              return;
            }
            // Verificar si la facturación AFIP está habilitada
            if (businessInfo?.facturacionHabilitada) {
              handleAfipPaymentClick();
            } else {
              handlePaymentClick();
            }
            break;
          case "F3":
            e.preventDefault();
            // Verificar si la facturación AFIP está habilitada
            if (!businessInfo?.facturacionHabilitada) {
              toast.error("Facturación AFIP no habilitada", {
                description:
                  "La facturación AFIP no está habilitada para este negocio",
              });
              return;
            }
            if (getCurrentItems().length === 0) {
              toast.error("No hay productos en el carrito", {
                description: "Agrega al menos un producto antes de continuar",
              });
              return;
            }
            // ✅ Con facturación habilitada, F3 crea una orden normal (sin AFIP)
            handlePaymentClick();
            break;
          case "F6":
            e.preventDefault();
            if (!businessInfo?.facturacionHabilitada) {
              toast.error("Funcionalidad no disponible", {
                description: "La facturación no está habilitada para este negocio",
              });
              return;
            }
            if (getCurrentItems().length === 0) {
              toast.error("No hay productos en el carrito", {
                description: "Agrega al menos un producto antes de crear la factura",
              });
              return;
            }
            handleFacturaClick();
            break;
        }
      }
    };

    window.addEventListener("keydown", handleGlobalKeyPress);
    return () => window.removeEventListener("keydown", handleGlobalKeyPress);
  }, [
    paymentDialogOpen,
    afipPaymentDialogOpen,
    isProcessingPayment,
    selectedPaymentMethod,
    afipIsProcessingPayment,
    afipSelectedPaymentMethod,
    handlePayment,
    handleAfipPayment,
    handleLogout,
    handleCancelClick,
    handlePaymentClick,
    handleAfipPaymentClick,
    handleFacturaClick,
    getCurrentItems,
    calculateTotal,
    businessInfo,
  ]);
};
