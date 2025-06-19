import React, { useRef } from "react";
import {
  Store,
  BarChart3,
  Package,
  Users,
  Settings,
  History,
  Home,
  ShoppingCart,
} from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { ProductSearch } from "./shopping-cart/ProductSearch";
import { AvailableProduct } from "@/hooks/useProductSearch";

interface MainLayoutProps {
  children: React.ReactNode;
  onProductSelect?: (product: AvailableProduct) => void;
}

export function MainLayout({ children, onProductSelect }: MainLayoutProps) {
  const { user } = useAuth();
  const searchInputRef = useRef<HTMLInputElement>(null);

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <div className="w-96 flex flex-col bg-emerald-gradient text-white">
        {/* Logo/Header */}
        <div className="p-6 ">
          <div className="flex items-center gap-3">
            <img
              src="andexmarket-logo.png"
              alt=""
              className="w-[12rem] h-auto"
            />
          </div>
        </div>

        {/* Búsqueda de Productos */}
        <div className="p-4 ">
          {onProductSelect && (
            <ProductSearch
              onProductSelect={onProductSelect}
              inputRef={searchInputRef}
            />
          )}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
