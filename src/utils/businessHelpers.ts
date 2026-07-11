import {
  fetchBusinessInfoShared,
} from "@/lib/businessInfoStore";

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
      const result = await (window as any).electronStore.get("adminData");
      return result;
    }

    if (typeof window !== "undefined") {
      const adminData = localStorage.getItem("adminData");

      if (adminData) {
        const parsed = JSON.parse(adminData);
        return parsed;
      }
    }
    return null;
  } catch (error) {
    console.error("❌ Error obteniendo adminData:", error);
    console.error("❌ Stack trace completo:", error);
    return null;
  }
};

// Helper para obtener información completa del business desde la API
export const getBusinessInfo = async (
  API_URL: string,
  appId: string | null
): Promise<any> => {
  try {
    const adminData = await getAdminData();

    if (!adminData?.businessId) {
      console.warn("⚠️ No se encontró businessId en adminData");
      return null;
    }

    return await fetchBusinessInfoShared(
      adminData.businessId,
      API_URL,
      appId
    );
  } catch (error) {
    console.error("❌ Error obteniendo businessInfo:", error);
    return null;
  }
};
