import { useState } from "react";
import { Product, AvailableProduct, Screen } from "../types";

export function useCart() {
  const [screens, setScreens] = useState<Screen[]>([{ id: 0, items: [] }]);
  const [activeScreen, setActiveScreen] = useState(0);
  const [quantity, setQuantity] = useState<string>("");

  const getCurrentScreen = () => screens[activeScreen];
  const getCurrentItems = () => getCurrentScreen().items;

  const calculateTotal = () => {
    return getCurrentItems().reduce((total, item) => total + item.subtotal, 0);
  };

  const addToCart = (product: AvailableProduct, quantity: number) => {
    const newItem: Product = {
      id: product.id,
      cartId: Math.random().toString(36).substr(2, 9),
      name: product.name,
      quantity,
      unit: product.unit,
      pricePerUnit: product.pricePerUnit,
      subtotal: product.pricePerUnit * quantity,
      costo: product.costo,
    };

    setScreens(
      screens.map((screen) => {
        if (screen.id === activeScreen) {
          return {
            ...screen,
            items: [...screen.items, newItem],
          };
        }
        return screen;
      })
    );
  };

  const removeFromCart = (cartId: string) => {
    setScreens(
      screens.map((screen) => {
        if (screen.id === activeScreen) {
          return {
            ...screen,
            items: screen.items.filter((item) => item.cartId !== cartId),
          };
        }
        return screen;
      })
    );
  };

  const clearCurrentCart = () => {
    setScreens(
      screens.map((screen) => {
        if (screen.id === activeScreen) {
          return { ...screen, items: [] };
        }
        return screen;
      })
    );
  };

  const addScreen = () => {
    const newScreen: Screen = {
      id: screens.length,
      items: [],
    };
    setScreens([...screens, newScreen]);
    setActiveScreen(newScreen.id);
  };

  const removeScreen = (screenId: number) => {
    if (screens.length <= 1) return;

    setScreens(screens.filter((screen) => screen.id !== screenId));
    if (activeScreen === screenId) {
      setActiveScreen(Math.max(0, screenId - 1));
    }
  };

  const switchScreen = (screenId: number) => {
    if (screenId >= 0 && screenId < screens.length) {
      setActiveScreen(screenId);
    }
  };

  return {
    screens,
    activeScreen,
    quantity,
    setQuantity,
    getCurrentItems,
    calculateTotal,
    addToCart,
    removeFromCart,
    clearCurrentCart,
    addScreen,
    removeScreen,
    switchScreen,
  };
}
