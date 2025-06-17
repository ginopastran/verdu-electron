import { useState, useEffect } from "react";
import { useBusiness } from "@/contexts/BusinessContext";

// Función helper para persistencia
const getStorageKey = (businessId: number) => `businessInfo_${businessId}`;

const saveBusinessInfo = (businessId: number, businessInfo: any) => {
  try {
    const key = getStorageKey(businessId);

    if (typeof window !== "undefined" && window.electronStore) {
      console.log("💾 Guardando businessInfo en electronStore:", key);
      window.electronStore.set(key, businessInfo);
    } else {
      console.log("💾 Guardando businessInfo en localStorage:", key);
      localStorage.setItem(key, JSON.stringify(businessInfo));
    }

    console.log("✅ BusinessInfo guardado:", businessInfo);
  } catch (error) {
    console.error("❌ Error al guardar businessInfo:", error);
  }
};

const loadBusinessInfo = (businessId: number): any | null => {
  try {
    const key = getStorageKey(businessId);

    if (typeof window !== "undefined" && window.electronStore) {
      console.log("📂 Cargando businessInfo desde electronStore:", key);
      const stored = window.electronStore.get(key);
      if (stored) {
        console.log("✅ BusinessInfo encontrado en electronStore:", stored);
        return stored;
      }
    } else {
      console.log("📂 Cargando businessInfo desde localStorage:", key);
      const stored = localStorage.getItem(key);
      if (stored) {
        const parsed = JSON.parse(stored);
        console.log("✅ BusinessInfo encontrado en localStorage:", parsed);
        return parsed;
      }
    }

    console.log("ℹ️ No hay businessInfo guardado para businessId:", businessId);
    return null;
  } catch (error) {
    console.error("❌ Error al cargar businessInfo:", error);
    return null;
  }
};

export const useBusinessInfo = (API_URL: string, appId: string | null) => {
  const [businessInfo, setBusinessInfo] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const { businessId } = useBusiness();

  useEffect(() => {
    const fetchBusinessInfo = async () => {
      if (!businessId) {
        console.log("⚠️ No hay businessId configurado");
        setLoading(false);
        return;
      }

      console.log("🏢 Iniciando carga de información del negocio...");

      // Primero intentar cargar desde storage local
      const cachedBusinessInfo = loadBusinessInfo(businessId);
      if (cachedBusinessInfo) {
        console.log("🚀 Usando businessInfo desde cache:", cachedBusinessInfo);
        setBusinessInfo(cachedBusinessInfo);
        setLoading(false);

        // Hacer fetch en background para actualizar el cache si es necesario
        console.log("🔄 Verificando actualizaciones en background...");
      }

      try {
        const response = await fetch(`${API_URL}/api/business/${businessId}`, {
          headers: {
            "Content-Type": "application/json",
            ...(appId && { "X-App-ID": appId }),
          },
        });

        if (!response.ok) {
          if (cachedBusinessInfo) {
            console.log("⚠️ Error en API pero tenemos cache, usando cache");
            return;
          }

          console.error(
            "❌ Error en la respuesta al cargar información del negocio:",
            response.status
          );
          throw new Error("Error al cargar información del negocio");
        }

        const data = await response.json();
        console.log("✅ Información del negocio cargada desde API:", data);

        // Verificar si tiene configuración de sistema de pago
        if (!data.sistemaPago) {
          console.warn(
            "⚠️ El negocio no tiene configurado sistemaPago, estableciendo por defecto"
          );
          data.sistemaPago = "redondeo"; // Establecer valor por defecto
        }

        // Guardar en storage local
        saveBusinessInfo(businessId, data);

        // Actualizar estado
        setBusinessInfo(data);
      } catch (error) {
        console.error("❌ Error al cargar información del negocio:", error);

        if (cachedBusinessInfo) {
          console.log("🔄 Error en API, manteniendo businessInfo desde cache");
          return;
        }

        // En caso de error y sin cache, establecer un valor por defecto para evitar problemas
        const defaultBusinessInfo = { sistemaPago: "redondeo" };
        setBusinessInfo(defaultBusinessInfo);
        console.log(
          "⚠️ Usando configuración por defecto:",
          defaultBusinessInfo
        );
      } finally {
        setLoading(false);
      }
    };

    setLoading(true);
    fetchBusinessInfo();
  }, [API_URL, appId, businessId]);

  return { businessInfo, loading };
};
