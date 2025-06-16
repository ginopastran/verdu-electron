import { useState, useEffect } from "react";
import { useBusiness } from "@/contexts/BusinessContext";

export const useBusinessInfo = (API_URL: string, appId: string | null) => {
  const [businessInfo, setBusinessInfo] = useState<any>(null);
  const { businessId } = useBusiness();

  useEffect(() => {
    const fetchBusinessInfo = async () => {
      if (!businessId) {
        console.log("⚠️ No hay businessId configurado");
        return;
      }

      try {
        console.log("🏢 Iniciando carga de información del negocio");
        const response = await fetch(`${API_URL}/api/business/${businessId}`, {
          headers: {
            "Content-Type": "application/json",
            ...(appId && { "X-App-ID": appId }),
          },
        });

        if (!response.ok) {
          console.error(
            "❌ Error en la respuesta al cargar información del negocio:",
            response.status
          );
          throw new Error("Error al cargar información del negocio");
        }

        const data = await response.json();
        console.log("✅ Información del negocio cargada:", data);

        // Verificar si tiene configuración de sistema de pago
        if (!data.sistemaPago) {
          console.warn(
            "⚠️ El negocio no tiene configurado sistemaPago, estableciendo por defecto"
          );
          data.sistemaPago = "redondeo"; // Establecer valor por defecto
        }

        setBusinessInfo(data);
      } catch (error) {
        console.error("❌ Error al cargar información del negocio:", error);
        // En caso de error, establecer un valor por defecto para evitar problemas
        setBusinessInfo({ sistemaPago: "redondeo" });
      }
    };

    // Ejecutar la carga de información
    fetchBusinessInfo();
  }, [API_URL, appId, businessId]);

  return businessInfo;
};
