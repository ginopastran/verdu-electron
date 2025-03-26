import { useAuth } from "@/contexts/AuthContext";

interface OfflineCredentials {
  email: string;
  user: {
    id: number;
    email: string;
    nombre: string;
    sucursalId: number;
    rol: {
      id: number;
      nombre: string;
    };
    permisos?: {
      pesoManualEnabled?: boolean;
    };
  };
}

export function useOfflineAuth() {
  const saveOfflineCredentials = (credentials: OfflineCredentials) => {
    localStorage.setItem("offlineCredentials", JSON.stringify(credentials));
  };

  const getOfflineCredentials = (): OfflineCredentials | null => {
    const stored = localStorage.getItem("offlineCredentials");
    return stored ? JSON.parse(stored) : null;
  };

  return {
    saveOfflineCredentials,
    getOfflineCredentials,
  };
}
