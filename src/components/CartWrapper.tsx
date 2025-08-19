import React, { useState, useRef } from "react";
import { MainLayout } from "./MainLayout";
import ShoppingCartRefactored, {
  ShoppingCartRefactoredRef,
} from "./ShoppingCartRefactored";
import {
  CartSidebarProvider,
  useCartSidebar,
} from "@/contexts/CartSidebarContext";
import { SearchInputProvider } from "@/contexts/SearchInputContext";
import { AvailableProduct } from "@/hooks/useProductSearch";

export function CartWrapper() {
  const shoppingCartRef = useRef<ShoppingCartRefactoredRef>(null);

  const handleAutoAddToCart = (product: AvailableProduct, quantity: number) => {
    if (shoppingCartRef.current) {
      shoppingCartRef.current.autoAddScannedProduct(product, quantity);
    }
  };

  return (
    <SearchInputProvider>
      <CartSidebarProvider autoAddProductToCart={handleAutoAddToCart}>
        <CartWithSidebarWrapper shoppingCartRef={shoppingCartRef} />
      </CartSidebarProvider>
    </SearchInputProvider>
  );
}

function CartWithSidebarWrapper({
  shoppingCartRef,
}: {
  shoppingCartRef: React.RefObject<ShoppingCartRefactoredRef | null>;
}) {
  const { selectProductFromSidebar } = useCartSidebar();

  return (
    <MainLayout onProductSelect={selectProductFromSidebar}>
      <ShoppingCartRefactored ref={shoppingCartRef} />
    </MainLayout>
  );
}
