import { useState, useEffect } from "react";

// Declaración de tipos para window
declare global {
  interface Window {
    pesoReader?: {
      readPeso: () => Promise<{
        success?: boolean;
        error?: string;
        peso: number;
      }>;
    };
  }
}

export function useScaleWeight() {
  const [weight, setWeight] = useState<number>(0);

  useEffect(() => {
    const fetchWeight = async () => {
      try {
        // Intentar usar el nuevo IPC handler que funciona tanto en desarrollo como producción
        if (typeof window !== "undefined" && window.pesoReader) {
          console.log("📏 Leyendo peso usando pesoReader IPC...");
          const result = await window.pesoReader.readPeso();

          if (result.success) {
            console.log("✅ Peso leído correctamente:", result.peso);
            setWeight(result.peso);
          } else {
            console.log("⚠️ Error al leer peso:", result.error);
            setWeight(0);
          }
          return;
        }

        // Fallback: Verificar si estamos en Electron y window.require está disponible
        if (typeof window !== "undefined" && window.require) {
          console.log("🔧 Fallback: usando require electron");
          const { ipcRenderer } = window.require("electron");
          const weight = await ipcRenderer.invoke("read-peso");
          console.log("Peso actualizado:", weight);
          setWeight(weight.peso || 0);
          return;
        }

        // Fallback para desarrollo: intentar usar Vite endpoint
        if (import.meta.env.DEV) {
          console.log("🔧 Modo desarrollo: intentando endpoint Vite...");
          try {
            const response = await fetch("/api/peso", {
              method: "GET",
            }).catch(() => null);

            if (response && response.ok) {
              const data = await response.json();
              const peso = data.peso || 0;
              console.log("✅ Peso leído desde API local:", peso);
              setWeight(peso);
              return;
            }
          } catch (fetchError) {
            console.log("⚠️ Endpoint Vite no disponible:", fetchError);
          }
        }

        // Último fallback: valor por defecto
        console.log(
          "⚠️ No se puede acceder al archivo de peso, usando valor por defecto"
        );
        setWeight(0);
      } catch (error) {
        console.error("❌ Error al leer el peso:", error);
        setWeight(0);
      }
    };

    const interval = setInterval(fetchWeight, 1000);
    return () => clearInterval(interval);
  }, []);

  return weight;
}
