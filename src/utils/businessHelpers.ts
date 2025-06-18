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
