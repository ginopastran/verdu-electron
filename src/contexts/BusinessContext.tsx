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
  setAdminData: (data: AdminData) => Promise<void>;
  clearAdminData: () => Promise<void>;
  hasAdminConfigured: () => boolean;
}

const BusinessContext = createContext<BusinessContextType>({
  adminData: null,
  businessId: null,
  loading: true,
  setAdminData: async () => {},
  clearAdminData: async () => {},
  hasAdminConfigured: () => false,
});

export function BusinessProvider({ children }: { children: ReactNode }) {
  const [adminData, setAdminDataState] = useState<AdminData | null>(null);
  const [businessId, setBusinessId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    console.log("🚀 BusinessContext useEffect ejecutándose");
    let isMounted = true;

    const loadData = async () => {
      try {
        console.log("🔄 BusinessContext: Iniciando carga de datos admin...");

        // Verificar si estamos en Electron
        if (typeof window !== "undefined" && window.electronStore) {
          console.log("📱 Usando electronStore...");
          try {
            const storedAdmin = await window.electronStore.get("adminData");
            console.log("📋 Resultado de electronStore.get:", storedAdmin);

            if (storedAdmin && isMounted) {
              console.log(
                "✅ Datos admin encontrados en electronStore:",
                storedAdmin
              );

              // Verificar que los datos son válidos
              if (storedAdmin.businessId && storedAdmin.businessId > 0) {
                setAdminDataState(storedAdmin);
                setBusinessId(storedAdmin.businessId);
                console.log("✅ Datos admin válidos cargados");
              } else {
                console.log("⚠️ Datos admin inválidos, limpiando...");
                await window.electronStore.delete("adminData");
              }
            } else if (isMounted) {
              console.log("ℹ️ No hay datos admin en electronStore");
            }
          } catch (error) {
            console.error("❌ Error al acceder a electronStore:", error);
            // Fallback a localStorage en caso de error
            console.log("🔄 Fallback a localStorage...");
            const storedAdmin = localStorage.getItem("adminData");
            if (storedAdmin && isMounted) {
              console.log("✅ Datos admin encontrados en localStorage");
              const parsed = JSON.parse(storedAdmin);
              console.log("📋 Datos parseados:", parsed);

              // Verificar que los datos son válidos
              if (parsed.businessId && parsed.businessId > 0) {
                setAdminDataState(parsed);
                setBusinessId(parsed.businessId);
                console.log(
                  "✅ Datos admin válidos cargados desde localStorage"
                );
              } else {
                console.log(
                  "⚠️ Datos admin inválidos, limpiando localStorage..."
                );
                localStorage.removeItem("adminData");
              }
            }
          }
        } else {
          console.log("🌐 Usando localStorage (desarrollo)...");
          // Fallback a localStorage para desarrollo
          const storedAdmin = localStorage.getItem("adminData");
          if (storedAdmin && isMounted) {
            console.log("✅ Datos admin encontrados en localStorage");
            const parsed = JSON.parse(storedAdmin);
            console.log("📋 Datos parseados:", parsed);

            // Verificar que los datos son válidos
            if (parsed.businessId && parsed.businessId > 0) {
              setAdminDataState(parsed);
              setBusinessId(parsed.businessId);
              console.log("✅ Datos admin válidos cargados");
            } else {
              console.log(
                "⚠️ Datos admin inválidos, limpiando localStorage..."
              );
              localStorage.removeItem("adminData");
            }
          } else if (isMounted) {
            console.log("ℹ️ No hay datos admin en localStorage");
          }
        }
      } catch (error) {
        console.error("❌ Error al cargar datos del admin:", error);
      } finally {
        if (isMounted) {
          console.log("✅ BusinessContext: Finalizando loading");
          setLoading(false);
        }
      }
    };

    loadData();

    return () => {
      console.log("🧹 BusinessContext cleanup");
      isMounted = false;
    };
  }, []);

  const setAdminData = async (data: AdminData) => {
    setAdminDataState(data);
    setBusinessId(data.businessId);

    try {
      if (window.electronStore) {
        await window.electronStore.set("adminData", data);
        console.log("✅ Datos admin guardados en electronStore");
      } else {
        localStorage.setItem("adminData", JSON.stringify(data));
        console.log("✅ Datos admin guardados en localStorage");
      }
    } catch (error) {
      console.error("❌ Error al guardar datos del admin:", error);
      // Fallback a localStorage en caso de error con electronStore
      try {
        localStorage.setItem("adminData", JSON.stringify(data));
        console.log("✅ Fallback: Datos admin guardados en localStorage");
      } catch (fallbackError) {
        console.error("❌ Error también en localStorage:", fallbackError);
      }
    }
  };

  const clearAdminData = async () => {
    setAdminDataState(null);
    setBusinessId(null);

    try {
      if (window.electronStore) {
        await window.electronStore.delete("adminData");
        console.log("✅ Datos admin eliminados de electronStore");
      } else {
        localStorage.removeItem("adminData");
        console.log("✅ Datos admin eliminados de localStorage");
      }
    } catch (error) {
      console.error("❌ Error al limpiar datos del admin:", error);
      // Fallback a localStorage en caso de error
      try {
        localStorage.removeItem("adminData");
        console.log("✅ Fallback: Datos admin eliminados de localStorage");
      } catch (fallbackError) {
        console.error("❌ Error también en localStorage:", fallbackError);
      }
    }
  };

  const hasAdminConfigured = () => {
    const isConfigured =
      adminData !== null && businessId !== null && businessId > 0;
    console.log("🔍 hasAdminConfigured:", {
      adminData: !!adminData,
      businessId,
      isConfigured,
    });
    return isConfigured;
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
