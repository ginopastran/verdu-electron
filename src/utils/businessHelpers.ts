// Helper para obtener el nombre del business dinámicamente
export const getBusinessName = async (): Promise<string> => {
  try {
    // Intentar desde electronStore primero
    if (typeof window !== "undefined" && (window as any).electronStore) {
      const adminData = await (window as any).electronStore.get("adminData");
      if (adminData?.businessName) {
        return adminData.businessName;
      }
    }

    // Fallback a localStorage
    if (typeof window !== "undefined") {
      const adminData = localStorage.getItem("adminData");
      if (adminData) {
        const parsed = JSON.parse(adminData);
        if (parsed?.businessName) {
          return parsed.businessName;
        }
      }
    }

    // Valor por defecto
    return "Verdulería";
  } catch (error) {
    console.error("Error obteniendo businessName:", error);
    return "Verdulería";
  }
};

// Helper para obtener información completa del admin/business
export const getAdminData = async () => {
  try {
    if (typeof window !== "undefined" && (window as any).electronStore) {
      return await (window as any).electronStore.get("adminData");
    }

    if (typeof window !== "undefined") {
      const adminData = localStorage.getItem("adminData");
      if (adminData) {
        return JSON.parse(adminData);
      }
    }

    return null;
  } catch (error) {
    console.error("Error obteniendo adminData:", error);
    return null;
  }
};

// Helper para obtener información completa del business desde la API
export const getBusinessInfo = async (
  API_URL: string,
  appId: string | null
): Promise<any> => {
  try {
    // Primero obtener businessId
    const adminData = await getAdminData();
    if (!adminData?.businessId) {
      console.warn("⚠️ No se encontró businessId en adminData");
      return null;
    }

    const businessId = adminData.businessId;
    console.log("🏢 Obteniendo información del business ID:", businessId);

    // Hacer llamada a la API
    const response = await fetch(`${API_URL}/api/business/${businessId}`, {
      headers: {
        "Content-Type": "application/json",
        ...(appId && { "X-App-ID": appId }),
      },
    });

    if (!response.ok) {
      console.error(
        "❌ Error en la respuesta al obtener business info:",
        response.status
      );
      return null;
    }

    const businessInfo = await response.json();
    console.log("✅ Información del business obtenida:", businessInfo);

    return businessInfo;
  } catch (error) {
    console.error("❌ Error al obtener información del business:", error);
    return null;
  }
};
