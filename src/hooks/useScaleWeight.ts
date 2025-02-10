import { useState, useEffect } from "react";

export function useScaleWeight() {
  const [weight, setWeight] = useState<number>(0);

  useEffect(() => {
    const fetchWeight = async () => {
      try {
        const { ipcRenderer } = window.require("electron");
        const weight = await ipcRenderer.invoke("read-weight");
        console.log("Peso actualizado:", weight);
        setWeight(weight);
      } catch (error) {
        console.error("Error al leer el peso:", error);
      }
    };

    const interval = setInterval(fetchWeight, 1000);
    return () => clearInterval(interval);
  }, []);

  return weight;
}
