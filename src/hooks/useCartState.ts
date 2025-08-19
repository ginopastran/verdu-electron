import { useState } from "react";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

export interface Product {
  id: number;
  cartId: string;
  name: string;
  quantity: number;
  unit: string;
  pricePerUnit: number;
  subtotal: number;
  costo: number;
  ivaIncluido?: boolean;
  ivaPorcentaje?: number | null;
}

export interface CartScreen {
  id: number;
  items: Product[];
}

export function useCartState() {
  const [screens, setScreens] = useState<CartScreen[]>([{ id: 0, items: [] }]);
  const [activeScreen, setActiveScreen] = useState(0);

  // Obtener información del business para el cálculo de IVA
  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;
  const { businessInfo } = useBusinessInfo(API_URL, appId);

  const calculateTotal = () => {
    const currentScreen = screens[activeScreen];
    return currentScreen.items.reduce((total, item) => {
      return total + item.pricePerUnit * item.quantity;
    }, 0);
  };

  const calculateTotalWithIVA = () => {
    const currentScreen = screens[activeScreen];
    return currentScreen.items.reduce((total, item) => {
      const precioVisualPorUnidad = businessInfo
        ? calcularPrecioVisualConIVA(
            item.pricePerUnit,
            item.ivaIncluido || false,
            item.ivaPorcentaje ?? null,
            businessInfo.ivaIncluidoEnPrecios || false
          )
        : item.pricePerUnit;

      return total + precioVisualPorUnidad * item.quantity;
    }, 0);
  };

  // Función para calcular el total sin IVA (para enviar al backend)
  const calculateTotalWithoutIVA = () => {
    const currentScreen = screens[activeScreen];
    return currentScreen.items.reduce((total, item) => {
      return total + item.pricePerUnit * item.quantity;
    }, 0);
  };

  const addToCart = (newItem: Product) => {
    setScreens(
      screens.map((screen, index) => {
        if (index === activeScreen) {
          const existingItem = screen.items.find(
            (item) => item.id === newItem.id
          );
          if (existingItem) {
            // Si el item ya existe, actualizar la cantidad
            return {
              ...screen,
              items: screen.items.map((item) =>
                item.id === newItem.id
                  ? {
                      ...item,
                      quantity: item.quantity + newItem.quantity,
                      subtotal: item.subtotal + newItem.subtotal,
                    }
                  : item
              ),
            };
          } else {
            // Si es un nuevo item, agregarlo
            return {
              ...screen,
              items: [...screen.items, newItem],
            };
          }
        }
        return screen;
      })
    );
  };

  const removeFromCart = (productId: number) => {
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen
          ? {
              ...screen,
              items: screen.items.filter((item) => item.id !== productId),
            }
          : screen
      )
    );
  };

  const clearCart = () => {
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen ? { ...screen, items: [] } : screen
      )
    );
  };

  const addScreen = () => {
    if (screens.length >= 4) {
      return false;
    }
    const newId = screens.length;
    setScreens([...screens, { id: newId, items: [] }]);
    setActiveScreen(newId);
    return true;
  };

  const deleteScreen = (screenId: number) => {
    if (screens.length <= 1) {
      return false;
    }

    const newScreens = screens.filter((screen) => screen.id !== screenId);
    setScreens(newScreens);

    // Si la pantalla activa es la que se eliminó, cambiar a la primera pantalla
    if (activeScreen === screenId) {
      setActiveScreen(0);
    }

    return true;
  };

  const getCurrentItems = () => {
    return screens[activeScreen].items;
  };

  return {
    screens,
    activeScreen,
    setActiveScreen,
    addToCart,
    removeFromCart,
    clearCart,
    calculateTotal,
    calculateTotalWithIVA,
    calculateTotalWithoutIVA,
    addScreen,
    deleteScreen,
    getCurrentItems,
  };
}
