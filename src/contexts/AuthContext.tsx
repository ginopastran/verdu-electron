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
    // Intentar recuperar el usuario del localStorage al cargar
    const storedUser = localStorage.getItem("user");
    if (storedUser) {
      setUser(JSON.parse(storedUser));
    }
    setLoading(false);
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
    localStorage.removeItem("user");
    localStorage.removeItem("offlineCredentials");
    setUser(null);
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
