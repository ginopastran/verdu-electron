import { createContext, useContext, ReactNode, useState } from "react";
import { CartItem, Screen } from "../types";

interface CartContextType {
  screens: Screen[];
  activeScreen: number;
  setActiveScreen: (index: number) => void;
  addScreen: () => void;
  deleteScreen: (screenId: number) => void;
  getCurrentItems: () => CartItem[];
  addToCart: (item: CartItem) => void;
  removeFromCart: (cartId: string) => void;
  clearCurrentCart: () => void;
  calculateTotal: () => number;
}

const CartContext = createContext<CartContextType | undefined>(undefined);

export function CartProvider({ children }: { children: ReactNode }) {
  const [screens, setScreens] = useState<Screen[]>([{ id: 0, items: [] }]);
  const [activeScreen, setActiveScreen] = useState(0);

  const addScreen = () => {
    if (screens.length >= 4) return;
    const newId = screens.length;
    setScreens([...screens, { id: newId, items: [] }]);
    setActiveScreen(newId);
  };

  const deleteScreen = (screenId: number) => {
    if (screens.length <= 1) return;
    const newScreens = screens.filter((screen) => screen.id !== screenId);
    setScreens(newScreens);
    if (activeScreen === screenId) {
      setActiveScreen(0);
    }
  };

  const getCurrentItems = () => {
    return screens[activeScreen].items;
  };

  const addToCart = (item: CartItem) => {
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen
          ? {
              ...screen,
              items: [...screen.items, item],
            }
          : screen
      )
    );
  };

  const removeFromCart = (cartId: string) => {
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen
          ? {
              ...screen,
              items: screen.items.filter((item) => item.cartId !== cartId),
            }
          : screen
      )
    );
  };

  const clearCurrentCart = () => {
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen ? { ...screen, items: [] } : screen
      )
    );
  };

  const calculateTotal = () => {
    return screens[activeScreen].items.reduce(
      (total, item) => total + item.subtotal,
      0
    );
  };

  return (
    <CartContext.Provider
      value={{
        screens,
        activeScreen,
        setActiveScreen,
        addScreen,
        deleteScreen,
        getCurrentItems,
        addToCart,
        removeFromCart,
        clearCurrentCart,
        calculateTotal,
      }}
    >
      {children}
    </CartContext.Provider>
  );
}

export function useCartContext() {
  const context = useContext(CartContext);
  if (context === undefined) {
    throw new Error("useCartContext must be used within a CartProvider");
  }
  return context;
}
