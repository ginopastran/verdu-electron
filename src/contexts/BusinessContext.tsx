import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";

// Declaración para electron-store
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
  setAdminData: (data: AdminData) => void;
  clearAdminData: () => void;
  hasAdminConfigured: () => boolean;
}

const BusinessContext = createContext<BusinessContextType>({
  adminData: null,
  businessId: null,
  loading: true,
  setAdminData: () => {},
  clearAdminData: () => {},
  hasAdminConfigured: () => false,
});

export function BusinessProvider({ children }: { children: ReactNode }) {
  const [adminData, setAdminDataState] = useState<AdminData | null>(null);
  const [businessId, setBusinessId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAdminData();
  }, []);

  const loadAdminData = () => {
    try {
      // Verificar si estamos en Electron
      if (window.electronStore) {
        const storedAdmin = window.electronStore.get("adminData");
        if (storedAdmin) {
          setAdminDataState(storedAdmin);
          setBusinessId(storedAdmin.businessId);
        }
      } else {
        // Fallback a localStorage para desarrollo
        const storedAdmin = localStorage.getItem("adminData");
        if (storedAdmin) {
          const parsed = JSON.parse(storedAdmin);
          setAdminDataState(parsed);
          setBusinessId(parsed.businessId);
        }
      }
    } catch (error) {
      console.error("Error al cargar datos del admin:", error);
    } finally {
      setLoading(false);
    }
  };

  const setAdminData = (data: AdminData) => {
    setAdminDataState(data);
    setBusinessId(data.businessId);

    try {
      if (window.electronStore) {
        window.electronStore.set("adminData", data);
      } else {
        localStorage.setItem("adminData", JSON.stringify(data));
      }
    } catch (error) {
      console.error("Error al guardar datos del admin:", error);
    }
  };

  const clearAdminData = () => {
    setAdminDataState(null);
    setBusinessId(null);

    try {
      if (window.electronStore) {
        window.electronStore.delete("adminData");
      } else {
        localStorage.removeItem("adminData");
      }
    } catch (error) {
      console.error("Error al limpiar datos del admin:", error);
    }
  };

  const hasAdminConfigured = () => {
    return adminData !== null && businessId !== null;
  };

  return (
    <BusinessContext.Provider
      value={{
        adminData,
        businessId,
        loading,
        setAdminData,
        clearAdminData,
        hasAdminConfigured,
      }}
    >
      {children}
    </BusinessContext.Provider>
  );
}

export const useBusiness = () => useContext(BusinessContext);
