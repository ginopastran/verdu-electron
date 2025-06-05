import { useEffect } from "react";
import { toast } from "sonner";

interface UseKeyboardShortcutsProps {
  paymentDialogOpen: boolean;
  isProcessingPayment: boolean;
  selectedPaymentMethod: string | null;
  handlePayment: (method: string) => void;
  handleLogout: () => void;
  handleCancelClick: () => void;
  handlePaymentClick: () => void;
  getCurrentItems: () => any[];
  calculateTotal: () => number;
}

export const useKeyboardShortcuts = ({
  paymentDialogOpen,
  isProcessingPayment,
  selectedPaymentMethod,
  handlePayment,
  handleLogout,
  handleCancelClick,
  handlePaymentClick,
  getCurrentItems,
  calculateTotal,
}: UseKeyboardShortcutsProps) => {
  useEffect(() => {
    const handleGlobalKeyPress = (e: KeyboardEvent) => {
      // Handle F4 for logout
      if (e.key === "F4") {
        e.preventDefault();
        handleLogout();
        return;
      }

      // Para el diálogo de pago
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
            handlePaymentClick();
            break;
        }
      }
    };

    window.addEventListener("keydown", handleGlobalKeyPress);
    return () => window.removeEventListener("keydown", handleGlobalKeyPress);
  }, [
    paymentDialogOpen,
    isProcessingPayment,
    selectedPaymentMethod,
    handlePayment,
    handleLogout,
    handleCancelClick,
    handlePaymentClick,
    getCurrentItems,
    calculateTotal,
  ]);
};
