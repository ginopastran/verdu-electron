import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import {
  clearBusinessInfoCache,
  fetchBusinessInfoShared,
  loadBusinessInfoLocal,
} from "@/lib/businessInfoStore";

declare global {
  interface Window {
    electronStore: {
      get: (key: string) => any;
      set: (key: string, value: any) => void;
      delete: (key: string) => void;
      has: (key: string) => boolean;
    };
  }
}

interface AdminData {
  id: number;
  nombre: string;
  email: string;
  businessId: number;
  businessName?: string;
}

interface BusinessContextType {
  adminData: AdminData | null;
  businessId: number | null;
  loading: boolean;
  businessInfo: any | null;
  businessInfoLoading: boolean;
  setAdminData: (data: AdminData) => Promise<void>;
  clearAdminData: () => Promise<void>;
  hasAdminConfigured: () => boolean;
  refreshBusinessInfo: () => Promise<void>;
}

const BusinessContext = createContext<BusinessContextType>({
  adminData: null,
  businessId: null,
  loading: true,
  businessInfo: null,
  businessInfoLoading: false,
  setAdminData: async () => {},
  clearAdminData: async () => {},
  hasAdminConfigured: () => false,
  refreshBusinessInfo: async () => {},
});

export function BusinessProvider({ children }: { children: ReactNode }) {
  const [adminData, setAdminDataState] = useState<AdminData | null>(null);
  const [businessId, setBusinessId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [businessInfo, setBusinessInfo] = useState<any | null>(null);
  const [businessInfoLoading, setBusinessInfoLoading] = useState(false);

  const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
  const appId = import.meta.env.VITE_APP_ID || null;

  useEffect(() => {
    let isMounted = true;

    const loadData = async () => {
      try {
        if (typeof window !== "undefined" && window.electronStore) {
          try {
            const storedAdmin = await window.electronStore.get("adminData");

            if (storedAdmin && isMounted) {
              if (storedAdmin.businessId && storedAdmin.businessId > 0) {
                setAdminDataState(storedAdmin);
                setBusinessId(storedAdmin.businessId);
              } else {
                await window.electronStore.delete("adminData");
              }
            }
          } catch (error) {
            console.error("❌ Error al acceder a electronStore:", error);
            const storedAdmin = localStorage.getItem("adminData");
            if (storedAdmin && isMounted) {
              const parsed = JSON.parse(storedAdmin);
              if (parsed.businessId && parsed.businessId > 0) {
                setAdminDataState(parsed);
                setBusinessId(parsed.businessId);
              } else {
                localStorage.removeItem("adminData");
              }
            }
          }
        } else {
          const storedAdmin = localStorage.getItem("adminData");
          if (storedAdmin && isMounted) {
            const parsed = JSON.parse(storedAdmin);
            if (parsed.businessId && parsed.businessId > 0) {
              setAdminDataState(parsed);
              setBusinessId(parsed.businessId);
            } else {
              localStorage.removeItem("adminData");
            }
          }
        }
      } catch (error) {
        console.error("❌ Error al cargar datos del admin:", error);
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    };

    loadData();

    return () => {
      isMounted = false;
    };
  }, []);

  useEffect(() => {
    if (!businessId) {
      setBusinessInfo(null);
      setBusinessInfoLoading(false);
      return;
    }

    let cancelled = false;

    const loadBusinessInfo = async () => {
      setBusinessInfoLoading(true);

      try {
        const cached = await loadBusinessInfoLocal(businessId);
        if (cached && !cancelled) {
          setBusinessInfo(cached);
          setBusinessInfoLoading(false);
        }

        const data = await fetchBusinessInfoShared(businessId, API_URL, appId);
        if (!cancelled) {
          setBusinessInfo(data);
        }
      } catch (error) {
        console.error("❌ Error al cargar información del negocio:", error);
        if (!cancelled) {
          const cached = await loadBusinessInfoLocal(businessId);
          if (cached) {
            setBusinessInfo(cached);
          } else {
            setBusinessInfo((prev: any) => prev || { sistemaPago: "redondeo" });
          }
        }
      } finally {
        if (!cancelled) {
          setBusinessInfoLoading(false);
        }
      }
    };

    loadBusinessInfo();

    return () => {
      cancelled = true;
    };
  }, [businessId, API_URL, appId]);

  const refreshBusinessInfo = async () => {
    if (!businessId) return;
    clearBusinessInfoCache(businessId);
    setBusinessInfoLoading(true);
    try {
      const data = await fetchBusinessInfoShared(businessId, API_URL, appId);
      setBusinessInfo(data);
    } catch (error) {
      console.error("❌ Error al refrescar información del negocio:", error);
    } finally {
      setBusinessInfoLoading(false);
    }
  };

  const setAdminData = async (data: AdminData) => {
    if (businessId && businessId !== data.businessId) {
      clearBusinessInfoCache(businessId);
      setBusinessInfo(null);
    }
    setAdminDataState(data);
    setBusinessId(data.businessId);

    try {
      if (window.electronStore) {
        await window.electronStore.set("adminData", data);
      } else {
        localStorage.setItem("adminData", JSON.stringify(data));
      }
    } catch (error) {
      console.error("❌ Error al guardar datos del admin:", error);
      try {
        localStorage.setItem("adminData", JSON.stringify(data));
      } catch (fallbackError) {
        console.error("❌ Error también en localStorage:", fallbackError);
      }
    }
  };

  const clearAdminData = async () => {
    clearBusinessInfoCache(businessId);
    setAdminDataState(null);
    setBusinessId(null);
    setBusinessInfo(null);

    try {
      if (window.electronStore) {
        await window.electronStore.delete("adminData");
      } else {
        localStorage.removeItem("adminData");
      }
    } catch (error) {
      console.error("❌ Error al limpiar datos del admin:", error);
      try {
        localStorage.removeItem("adminData");
      } catch (fallbackError) {
        console.error("❌ Error también en localStorage:", fallbackError);
      }
    }
  };

  const hasAdminConfigured = () => {
    return adminData !== null && businessId !== null && businessId > 0;
  };

  return (
    <BusinessContext.Provider
      value={{
        adminData,
        businessId,
        loading,
        businessInfo,
        businessInfoLoading,
        setAdminData,
        clearAdminData,
        hasAdminConfigured,
        refreshBusinessInfo,
      }}
    >
      {children}
    </BusinessContext.Provider>
  );
}

export const useBusiness = () => useContext(BusinessContext);
