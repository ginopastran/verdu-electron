import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import { AvailableProduct } from "@/hooks/useProductSearch";

interface CartSidebarContextType {
  selectedProductFromSidebar: AvailableProduct | null;
  selectProductFromSidebar: (product: AvailableProduct) => void;
  clearSelectedProduct: () => void;
  onBarcodeScanned?: (product: AvailableProduct) => void;
  setOnBarcodeScanned: (callback: (product: AvailableProduct) => void) => void;
}

const CartSidebarContext = createContext<CartSidebarContextType | undefined>(
  undefined
);

export function useCartSidebar() {
  const context = useContext(CartSidebarContext);
  if (!context) {
    throw new Error("useCartSidebar must be used within a CartSidebarProvider");
  }
  return context;
}

interface CartSidebarProviderProps {
  children: ReactNode;
}

export function CartSidebarProvider({ children }: CartSidebarProviderProps) {
  const [selectedProductFromSidebar, setSelectedProductFromSidebar] =
    useState<AvailableProduct | null>(null);
  const [onBarcodeScanned, setOnBarcodeScanned] = useState<
    ((product: AvailableProduct) => void) | undefined
  >(undefined);

  // Debug: Agregar logging cuando onBarcodeScanned cambia
  useEffect(() => {
    console.log(
      "🔧 DEBUG: 🚀 CartSidebarContext - onBarcodeScanned actualizado:",
      {
        available: !!onBarcodeScanned,
        type: typeof onBarcodeScanned,
        isFunction: typeof onBarcodeScanned === "function",
      }
    );
  }, [onBarcodeScanned]);

  const selectProductFromSidebar = (product: AvailableProduct) => {
    setSelectedProductFromSidebar(product);
  };

  const setOnBarcodeScannerCallback = (
    callback: (product: AvailableProduct) => void
  ) => {
    console.log(
      "🔧 DEBUG: 🚀 CartSidebarContext - Registrando callback:",
      typeof callback
    );
    setOnBarcodeScanned(() => callback); // Envolver en función para evitar ejecución inmediata
  };

  const clearSelectedProduct = () => {
    setSelectedProductFromSidebar(null);
  };

  return (
    <CartSidebarContext.Provider
      value={{
        selectedProductFromSidebar,
        selectProductFromSidebar,
        clearSelectedProduct,
        onBarcodeScanned,
        setOnBarcodeScanned: setOnBarcodeScannerCallback,
      }}
    >
      {children}
    </CartSidebarContext.Provider>
  );
}
