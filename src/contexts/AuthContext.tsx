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
    console.log("🔄 AuthContext: Iniciando sin restaurar usuario automáticamente...");
    
    // ✅ CAMBIO: No restaurar automáticamente el usuario al iniciar la app
    // Esto fuerza que siempre se vaya al selector de usuarios
    // El usuario se autenticará manualmente cada vez que abra la app
    
    try {
      // Limpiar cualquier sesión anterior al iniciar
      const storedUser = localStorage.getItem("user");
      if (storedUser) {
        console.log("🧹 Limpiando sesión anterior al iniciar la app");
        localStorage.removeItem("user");
      }
      
      // Mantener el usuario como null para forzar re-autenticación
      setUser(null);
      console.log("✅ Usuario configurado como null - se requiere nueva autenticación");
    } catch (error) {
      console.error("❌ Error al limpiar sesión anterior:", error);
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
    // console.log("🚪 AuthContext: Iniciando logout...");

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

    // console.log("✅ AuthContext: Logout completado y evento emitido");
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
