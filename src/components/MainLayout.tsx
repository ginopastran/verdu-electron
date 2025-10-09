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
import { useSearchInput } from "@/contexts/SearchInputContext";
import { useCartSidebar } from "@/contexts/CartSidebarContext";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";

interface MainLayoutProps {
  children: React.ReactNode;
  onProductSelect?: (product: AvailableProduct) => void;
}

export function MainLayout({ children, onProductSelect }: MainLayoutProps) {
  const { user } = useAuth();
  const { searchInputRef } = useSearchInput();
  const { autoAddProductToCart } = useCartSidebar();

  // Obtener información del negocio para verificar configuración AFIP
  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  // Verificar si AFIP está habilitado
  const isAfipEnabled = businessInfo?.afipHabilitado || false;

  // Verificar si facturación está habilitada
  const isFacturacionEnabled = businessInfo?.facturacionHabilitada || false;

  // Definir las teclas según la configuración AFIP
  const keyboardShortcuts = [
    {
      key: "F1",
      description: "para cancelar",
      showAlways: true,
    },
    {
      key: "F2",
      description: isAfipEnabled ? "para pagar con AFIP" : "para pagar",
      showAlways: true,
    },
    {
      key: "F7",
      description: isAfipEnabled
        ? "para pagar con descuento AFIP"
        : "para pagar con descuento",
      showAlways: true,
    },
    {
      key: "F3",
      description: "para pagar orden",
      showAlways: isAfipEnabled, // Solo mostrar si AFIP está habilitado
    },
    {
      key: "F8",
      description: "para pagar con descuento orden",
      showAlways: isAfipEnabled, // Solo mostrar si AFIP está habilitado
    },
    {
      key: "F6",
      description: "para cuenta corriente",
      showAlways: isFacturacionEnabled, // Solo mostrar si facturación está habilitada
    },
    {
      key: "F4",
      description: "para cerrar sesión",
      showAlways: true,
    },
  ];

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <div className="w-96 flex flex-col bg-emerald-gradient text-white h-full">
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
              onDirectAddToCart={autoAddProductToCart}
            />
          )}
        </div>

        <div className="w-full h-full flex flex-col justify-end p-2 text-muted-foreground font-light">
          <div className="w-full bg-gray-50 min-h-32 rounded-lg p-1 flex flex-col gap-1.5">
            {keyboardShortcuts
              .filter((shortcut) => shortcut.showAlways)
              .map((shortcut, index) => (
                <div key={shortcut.key} className="flex items-center gap-2">
                  <img
                    src={`keys/${shortcut.key}.png`}
                    alt={shortcut.key}
                    className="size-10"
                  />
                  <span>{shortcut.description}</span>
                </div>
              ))}
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
