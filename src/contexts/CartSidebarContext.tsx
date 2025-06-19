import React, { createContext, useContext, useState, ReactNode } from "react";
import { AvailableProduct } from "@/hooks/useProductSearch";

interface CartSidebarContextType {
  selectedProductFromSidebar: AvailableProduct | null;
  selectProductFromSidebar: (product: AvailableProduct) => void;
  clearSelectedProduct: () => void;
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

  const selectProductFromSidebar = (product: AvailableProduct) => {
    setSelectedProductFromSidebar(product);
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
      }}
    >
      {children}
    </CartSidebarContext.Provider>
  );
}
