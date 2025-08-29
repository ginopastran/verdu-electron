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
    console.log("🔍 DEBUG getAdminData: Iniciando...");
    console.log("🔍 DEBUG: Estado del window:", {
      hasWindow: typeof window !== "undefined",
      hasElectronStore:
        typeof window !== "undefined" && !!(window as any).electronStore,
      windowKeys:
        typeof window !== "undefined"
          ? Object.keys(window)
              .filter((k) => k.includes("electron"))
              .slice(0, 5)
          : [],
    });

    if (typeof window !== "undefined" && (window as any).electronStore) {
      console.log("🔍 DEBUG: Obteniendo adminData desde electronStore...");
      const result = await (window as any).electronStore.get("adminData");
      console.log("🔍 DEBUG: AdminData desde electronStore:", {
        result,
        hasBusinessId: !!result?.businessId,
        businessId: result?.businessId,
        allKeys: result ? Object.keys(result) : [],
      });
      return result;
    }

    if (typeof window !== "undefined") {
      console.log("🔍 DEBUG: Obteniendo adminData desde localStorage...");
      const adminData = localStorage.getItem("adminData");
      console.log("🔍 DEBUG: AdminData raw desde localStorage:", adminData);

      if (adminData) {
        const parsed = JSON.parse(adminData);
        console.log("🔍 DEBUG: AdminData parseado desde localStorage:", {
          parsed,
          hasBusinessId: !!parsed?.businessId,
          businessId: parsed?.businessId,
          allKeys: Object.keys(parsed),
        });
        return parsed;
      }
    }

    console.log("🔍 DEBUG: No se encontró adminData en ningún lado");
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
    console.log("🔍 DEBUG getBusinessInfo: Iniciando...");
    console.log("🔍 DEBUG: Parámetros de entrada:", {
      API_URL,
      appId,
      appIdType: typeof appId,
    });

    // Primero obtener businessId
    const adminData = await getAdminData();
    console.log("🔍 DEBUG: AdminData obtenida:", {
      adminData,
      hasBusinessId: !!adminData?.businessId,
      businessId: adminData?.businessId,
    });

    if (!adminData?.businessId) {
      console.warn("⚠️ No se encontró businessId en adminData");
      return null;
    }

    const businessId = adminData.businessId;
    console.log("🏢 Obteniendo información del business ID:", businessId);

    // Construir URL y headers
    const url = `${API_URL}/api/business/${businessId}?include=configuracionAfip`;
    const headers = {
      "Content-Type": "application/json",
      ...(appId && { "X-App-ID": appId }),
    };

    console.log("🔍 DEBUG: Llamada HTTP:", {
      url,
      headers,
      method: "GET",
    });

    // Hacer llamada a la API
    const response = await fetch(url, { headers });

    console.log("🔍 DEBUG: Respuesta HTTP:", {
      status: response.status,
      statusText: response.statusText,
      ok: response.ok,
      headers: Object.fromEntries(response.headers.entries()),
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

    // ✅ NUEVO: Debug específico para doble impresión
    // console.log("🖨️ DEBUG DOBLE IMPRESIÓN:");
    // console.log(
    //   "- dobleImpresionEnabled existe:",
    //   "dobleImpresionEnabled" in businessInfo
    // );
    // console.log(
    //   "- dobleImpresionEnabled valor:",
    //   businessInfo.dobleImpresionEnabled
    // );
    // console.log(
    //   "- dobleImpresionEnabled tipo:",
    //   typeof businessInfo.dobleImpresionEnabled
    // );
    // console.log(
    //   "- Comparación === true:",
    //   businessInfo.dobleImpresionEnabled === true
    // );
    // console.log(
    //   "- Comparación == true:",
    //   businessInfo.dobleImpresionEnabled == true
    // );
    // console.log(
    //   "- Comparación === 'true':",
    //   businessInfo.dobleImpresionEnabled === "true"
    // );
    // console.log("- Valor truthy:", !!businessInfo.dobleImpresionEnabled);

    // // ✅ NUEVO: Mostrar todos los campos del business para debug
    // console.log("🔍 DEBUG: Todos los campos del business:", {
    //   id: businessInfo.id,
    //   nombre: businessInfo.nombre,
    //   dobleImpresionEnabled: businessInfo.dobleImpresionEnabled,
    //   descuentoEfectivo: businessInfo.descuentoEfectivo,
    //   sistemaPago: businessInfo.sistemaPago,
    //   todosLosCampos: Object.keys(businessInfo).sort(),
    // });

    return businessInfo;
  } catch (error) {
    console.error("❌ Error al obtener información del business:", error);
    console.error("❌ Stack trace:", error);
    return null;
  }
};
