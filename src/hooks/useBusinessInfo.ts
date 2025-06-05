import { useState, useEffect } from "react";

export const useBusinessInfo = (API_URL: string, appId: string | null) => {
  const [businessInfo, setBusinessInfo] = useState<any>(null);

  useEffect(() => {
    const fetchBusinessInfo = async () => {
      try {
        console.log("🏢 Iniciando carga de información del negocio");
        const response = await fetch(
          `${API_URL}/api/business/${import.meta.env.VITE_BUSINESS_ID}`,
          {
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
          }
        );

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
  }, [API_URL, appId]);

  return businessInfo;
};
