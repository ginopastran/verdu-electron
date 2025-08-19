import React, { useState } from "react";
import { MainLayout } from "./MainLayout";
import ShoppingCartRefactored from "./ShoppingCartRefactored";
import {
  CartSidebarProvider,
  useCartSidebar,
} from "@/contexts/CartSidebarContext";
import { SearchInputProvider } from "@/contexts/SearchInputContext";

function CartWithSidebar() {
  const { selectProductFromSidebar, onBarcodeScanned } = useCartSidebar();

  console.log("🔧 DEBUG: 🚀 CartWrapper - callbacks disponibles:", {
    selectProductFromSidebar: !!selectProductFromSidebar,
    onBarcodeScanned: !!onBarcodeScanned,
    onBarcodeScannerType: typeof onBarcodeScanned,
  });

  return (
    <MainLayout
      onProductSelect={selectProductFromSidebar}
      onBarcodeScanned={onBarcodeScanned}
    >
      <ShoppingCartRefactored />
    </MainLayout>
  );
}

export function CartWrapper() {
  return (
    <SearchInputProvider>
      <CartSidebarProvider>
        <CartWithSidebar />
      </CartSidebarProvider>
    </SearchInputProvider>
  );
}
