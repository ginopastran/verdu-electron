import {
  createContext,
  useContext,
  ReactNode,
  useState,
  useEffect,
} from "react";
import { useAuth } from "@/contexts/AuthContext";
import { useScaleWeight } from "@/hooks/useScaleWeight";

interface WeightContextType {
  weight: number;
  useManualWeight: boolean;
  setUseManualWeight: (value: boolean) => void;
}

interface WeightProviderProps {
  children: ReactNode;
}

const WeightContext = createContext<WeightContextType | undefined>(undefined);

export function WeightProvider({ children }: WeightProviderProps) {
  const { user } = useAuth();
  const weight = useScaleWeight();
  const [useManualWeight, setUseManualWeight] = useState(false);

  useEffect(() => {
    if (user?.permisos?.pesoManualEnabled === true) {
      setUseManualWeight(true);
    } else {
      setUseManualWeight(false);
    }
  }, [user?.permisos?.pesoManualEnabled]);

  return (
    <WeightContext.Provider
      value={{ weight, useManualWeight, setUseManualWeight }}
    >
      {children}
    </WeightContext.Provider>
  );
}

export function useWeightContext() {
  const context = useContext(WeightContext);
  if (context === undefined) {
    throw new Error("useWeightContext must be used within a WeightProvider");
  }
  return context;
}
