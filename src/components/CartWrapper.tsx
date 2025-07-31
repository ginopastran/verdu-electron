import React, { useState } from "react";
import { MainLayout } from "./MainLayout";
import ShoppingCartRefactored from "./ShoppingCartRefactored";
import {
  CartSidebarProvider,
  useCartSidebar,
} from "@/contexts/CartSidebarContext";
import { SearchInputProvider } from "@/contexts/SearchInputContext";

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
    <SearchInputProvider>
      <CartSidebarProvider>
        <CartWithSidebar />
      </CartSidebarProvider>
    </SearchInputProvider>
  );
}
