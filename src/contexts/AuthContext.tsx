import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";

interface User {
  id: number;
  nombre: string;
  email: string;
  sucursalId: number;
  role?: string;
  permisos: {
    pesoManualEnabled?: boolean;
    cierreDeCajaEnabled?: boolean;
  };
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  updateUser: (user: User | null) => void;
  logout: () => void;
  refreshUserData: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType>({
  user: null,
  loading: true,
  updateUser: () => {},
  logout: () => {},
  refreshUserData: async () => {},
});

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    console.log("🔄 AuthContext: Iniciando carga de usuario...");
    try {
      // Intentar recuperar el usuario del localStorage al cargar
      const storedUser = localStorage.getItem("user");
      if (storedUser) {
        console.log("✅ Usuario encontrado en localStorage");
        setUser(JSON.parse(storedUser));
      } else {
        console.log("ℹ️ No hay usuario en localStorage");
      }
    } catch (error) {
      console.error("❌ Error al cargar usuario:", error);
    } finally {
      console.log("✅ AuthContext: Finalizando loading");
      setLoading(false);
    }

    // Safety timeout para AuthContext también
    const safetyTimeout = setTimeout(() => {
      console.log("⚠️ SAFETY: AuthContext timeout - forzando fin de loading");
      setLoading(false);
    }, 8000);

    return () => clearTimeout(safetyTimeout);
  }, []);

  const updateUser = (newUser: User | null) => {
    setUser(newUser);
    if (newUser) {
      localStorage.setItem("user", JSON.stringify(newUser));
    } else {
      localStorage.removeItem("user");
    }
  };

  const logout = () => {
    console.log("🚪 AuthContext: Iniciando logout...");

    // Limpiar datos del usuario
    localStorage.removeItem("user");
    localStorage.removeItem("offlineCredentials");
    setUser(null);

    // ✅ AGREGADO: Emitir evento personalizado para notificar logout a otros componentes
    window.dispatchEvent(
      new CustomEvent("userLogout", {
        detail: {
          timestamp: Date.now(),
          reason: "manual_logout",
        },
      })
    );

    console.log("✅ AuthContext: Logout completado y evento emitido");
  };

  const refreshUserData = async () => {
    // Ya no realizamos la petición HTTP a /api/usuarios/[id]
    // Simplemente usamos los datos que ya tenemos en memoria
    console.log("La actualización de datos de usuario ha sido desactivada");

    // Si en el futuro necesitas volver a implementar esta función,
    // aquí estaría el código para hacer una petición a la API
    return;
  };

  return (
    <AuthContext.Provider
      value={{ user, loading, updateUser, logout, refreshUserData }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => useContext(AuthContext);
