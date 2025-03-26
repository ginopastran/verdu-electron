import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";

interface User {
  id: string;
  email: string;
  nombre: string;
  sucursalId: string;
  rol: {
    id: number;
    nombre: string;
  };
  permisos?: {
    pesoManualEnabled?: boolean;
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
    if (!user) return;

    try {
      const API_URL = import.meta.env.VITE_API_URL;
      const appId = import.meta.env.VITE_APP_ID;

      const response = await fetch(`${API_URL}/api/usuarios/${user.id}`, {
        headers: {
          "Content-Type": "application/json",
          "X-App-ID": appId,
          "Cache-Control": "no-cache",
        },
      });

      if (!response.ok) {
        throw new Error("Error al actualizar los datos del usuario");
      }

      const userData = await response.json();

      // Ensure we have the permisos object with proper properties
      if (!userData.permisos) {
        userData.permisos = {};
      }

      // Check if pesoManualEnabled permission exists in the response
      if (userData.pesoManualEnabled !== undefined) {
        // If it comes as a direct property, move it to the permisos object
        userData.permisos.pesoManualEnabled = userData.pesoManualEnabled;
        // Remove the direct property to avoid duplication
        delete userData.pesoManualEnabled;
      }

      console.log("Usuario actualizado:", userData);

      // Update user in state and localStorage
      updateUser(userData);

      // Update offline credentials too
      const storedCredentials = localStorage.getItem("offlineCredentials");
      if (storedCredentials) {
        const credentials = JSON.parse(storedCredentials);
        credentials.user = userData;
        localStorage.setItem("offlineCredentials", JSON.stringify(credentials));
      }
    } catch (error) {
      console.error("Error al actualizar datos del usuario:", error);
    }
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
