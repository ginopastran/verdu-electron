import React from "react";
import { MainLayout } from "./MainLayout";
import ShoppingCartRefactored from "./ShoppingCartRefactored";
import {
  CartSidebarProvider,
  useCartSidebar,
} from "@/contexts/CartSidebarContext";

function CartWithSidebar() {
  const { selectProductFromSidebar } = useCartSidebar();

  return (
    <MainLayout onProductSelect={selectProductFromSidebar}>
      <ShoppingCartRefactored />
    </MainLayout>
  );
}

export function CartWrapper() {
  return (
    <CartSidebarProvider>
      <CartWithSidebar />
    </CartSidebarProvider>
  );
}
