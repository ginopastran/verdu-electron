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
    console.log("🛒 [useCartState] addToCart llamado con:", newItem);

    setScreens((prevScreens) => {
      const updatedScreens = prevScreens.map((screen, index) => {
        if (index === activeScreen) {
          // CAMBIO CRÍTICO: Buscar por cartId en lugar de solo id para permitir múltiples productos del mismo tipo
          const existingItem = screen.items.find(
            (item) => item.cartId === newItem.cartId
          );

          if (existingItem) {
            console.log(
              "🛒 [useCartState] Item existente encontrado por cartId, actualizando cantidad:",
              existingItem
            );
            // Si el item ya existe (mismo cartId), actualizar la cantidad
            return {
              ...screen,
              items: screen.items.map((item) =>
                item.cartId === newItem.cartId
                  ? {
                      ...item,
                      quantity: item.quantity + newItem.quantity,
                      subtotal: item.subtotal + newItem.subtotal,
                    }
                  : item
              ),
            };
          } else {
            console.log("🛒 [useCartState] Nuevo item agregado:", newItem);
            // Si es un nuevo item (cartId único), agregarlo
            return {
              ...screen,
              items: [...screen.items, newItem],
            };
          }
        }
        return screen;
      });

      console.log("🛒 [useCartState] Estado actualizado - screen activa:", {
        activeScreen,
        itemsCount: updatedScreens[activeScreen]?.items.length || 0,
        items:
          updatedScreens[activeScreen]?.items.map((item) => ({
            id: item.id,
            cartId: item.cartId,
            name: item.name,
            quantity: item.quantity,
          })) || [],
      });

      return updatedScreens;
    });
  };

  const removeFromCart = (cartId: string) => {
    console.log(
      "🛒 [useCartState] removeFromCart llamado para cartId:",
      cartId
    );

    setScreens((prevScreens) =>
      prevScreens.map((screen, index) =>
        index === activeScreen
          ? {
              ...screen,
              items: screen.items.filter((item) => item.cartId !== cartId),
            }
          : screen
      )
    );
  };

  const clearCart = () => {
    console.log(
      "🧹 [useCartState] clearCart llamado - limpiando screen activa:",
      activeScreen
    );
    const currentItems = screens[activeScreen]?.items || [];
    console.log(
      "🧹 [useCartState] Items que se van a eliminar:",
      currentItems.map((item) => ({
        id: item.id,
        cartId: item.cartId,
        name: item.name,
      }))
    );

    setScreens((prevScreens) =>
      prevScreens.map((screen, index) =>
        index === activeScreen ? { ...screen, items: [] } : screen
      )
    );

    console.log("🧹 [useCartState] clearCart completado");
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
