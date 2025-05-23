import {
  createContext,
  useContext,
  ReactNode,
  useState,
  useEffect,
} from "react";

interface BusinessInfo {
  sistemaPago: string;
  descuentoEfectivo?: number;
}

interface BusinessContextType {
  businessInfo: BusinessInfo | null;
  isLoading: boolean;
  error: Error | null;
}

interface BusinessProviderProps {
  children: ReactNode;
  API_URL: string;
  headers: Record<string, string>;
}

const BusinessContext = createContext<BusinessContextType | undefined>(
  undefined
);

export function BusinessProvider({
  children,
  API_URL,
  headers,
}: BusinessProviderProps) {
  const [businessInfo, setBusinessInfo] = useState<BusinessInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchBusinessInfo = async () => {
      try {
        const response = await fetch(`${API_URL}/api/business/1`, { headers });

        if (!response.ok) {
          throw new Error("Error al cargar información del negocio");
        }

        const data = await response.json();

        // Establecer valores por defecto si es necesario
        if (!data.sistemaPago) {
          data.sistemaPago = "redondeo";
        }

        setBusinessInfo(data);
      } catch (error) {
        setError(error as Error);
        // Establecer valores por defecto en caso de error
        setBusinessInfo({ sistemaPago: "redondeo" });
      } finally {
        setIsLoading(false);
      }
    };

    fetchBusinessInfo();
  }, [API_URL]);

  return (
    <BusinessContext.Provider value={{ businessInfo, isLoading, error }}>
      {children}
    </BusinessContext.Provider>
  );
}

export function useBusinessContext() {
  const context = useContext(BusinessContext);
  if (context === undefined) {
    throw new Error(
      "useBusinessContext must be used within a BusinessProvider"
    );
  }
  return context;
}
