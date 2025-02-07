import { useState, useEffect } from "react";

export function useOfflineMode() {
  const [isOnline, setIsOnline] = useState(true);
  const [pendingOrders, setPendingOrders] = useState<any[]>([]);

  useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);
    setIsOnline(navigator.onLine);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  }, []);

  const cacheProductos = async (productos: any[]) => {
    localStorage.setItem("cachedProducts", JSON.stringify(productos));
  };

  const getCachedProductos = () => {
    return JSON.parse(localStorage.getItem("cachedProducts") || "[]");
  };

  const savePendingOrder = async (order: any) => {
    const orders = JSON.parse(localStorage.getItem("pendingOrders") || "[]");
    orders.push(order);
    localStorage.setItem("pendingOrders", JSON.stringify(orders));
    setPendingOrders(orders);
  };

  const syncPendingOrders = async () => {
    if (!isOnline) return;

    const orders = JSON.parse(localStorage.getItem("pendingOrders") || "[]");
    for (const order of orders) {
      try {
        // Here you should use your Electron IPC to communicate with the main process
        // For now, we'll just clear the orders
        console.log("Syncing order:", order);
      } catch (error) {
        console.error("Error syncing order:", error);
      }
    }
    localStorage.removeItem("pendingOrders");
    setPendingOrders([]);
  };

  return {
    isOnline,
    pendingOrders,
    savePendingOrder,
    syncPendingOrders,
    cacheProductos,
    getCachedProductos,
  };
}
