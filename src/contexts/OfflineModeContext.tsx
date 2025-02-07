import { createContext, useContext } from "react";
import { useOfflineMode as useOfflineModeHook } from "@/hooks/useOfflineMode";

const OfflineModeContext = createContext<ReturnType<typeof useOfflineModeHook>>(
  {
    isOnline: true,
    pendingOrders: [],
    savePendingOrder: async (order: any) => {},
    syncPendingOrders: async () => {},
    cacheProductos: async (productos: any[]) => {},
    getCachedProductos: () => [],
  }
);

export function OfflineModeProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const offlineMode = useOfflineModeHook();
  return (
    <OfflineModeContext.Provider value={offlineMode}>
      {children}
    </OfflineModeContext.Provider>
  );
}

export const useOfflineMode = () => useContext(OfflineModeContext);
