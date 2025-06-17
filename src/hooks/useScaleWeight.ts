import { useState, useEffect } from "react";

export function useScaleWeight() {
  const [weight, setWeight] = useState<number>(0);

  useEffect(() => {
    const fetchWeight = async () => {
      try {
        // Verificar si estamos en Electron y window.require está disponible
        if (typeof window !== "undefined" && window.require) {
          const { ipcRenderer } = window.require("electron");
          const weight = await ipcRenderer.invoke("read-weight");
          console.log("Peso actualizado:", weight);
          setWeight(weight);
        } else {
          // En desarrollo, usar Node.js directamente para leer el archivo
          console.log(
            "🔧 Modo desarrollo: leyendo peso directamente desde C:\\Peso\\peso.json"
          );

          try {
            // Usar fetch para hacer una petición a un endpoint local que lea el archivo
            // o implementar lectura directa si es posible
            const response = await fetch("/api/peso", {
              method: "GET",
            }).catch(() => null);

            if (response && response.ok) {
              const data = await response.json();
              const peso = data.peso || 0;
              console.log("Peso leído desde API local:", peso);
              setWeight(peso);
            } else {
              // Como fallback, intentar leer usando require si está disponible
              if (
                typeof window !== "undefined" &&
                (window as any).electronAPI
              ) {
                // Si hay API de Electron expuesta de otra manera
                const peso = await (window as any).electronAPI.readWeight();
                setWeight(peso);
              } else {
                // Último fallback: simular peso estático para desarrollo puro web
                console.log(
                  "⚠️ No se puede acceder al archivo de peso, usando valor por defecto"
                );
                setWeight(0);
              }
            }
          } catch (fileError) {
            console.error("Error leyendo peso en desarrollo:", fileError);
            setWeight(0);
          }
        }
      } catch (error) {
        console.error("Error al leer el peso:", error);
        setWeight(0);
      }
    };

    const interval = setInterval(fetchWeight, 1000);
    return () => clearInterval(interval);
  }, []);

  return weight;
}
