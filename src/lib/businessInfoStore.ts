const getStorageKey = (businessId: number) => `businessInfo_${businessId}`;

type CacheEntry = {
  data: any | null;
  promise: Promise<any> | null;
};

const memoryCache = new Map<number, CacheEntry>();

export const saveBusinessInfoLocal = async (
  businessId: number,
  businessInfo: any
) => {
  try {
    const key = getStorageKey(businessId);
    if (typeof window !== "undefined" && window.electronStore) {
      await window.electronStore.set(key, businessInfo);
    } else {
      localStorage.setItem(key, JSON.stringify(businessInfo));
    }
  } catch {
    try {
      localStorage.setItem(
        getStorageKey(businessId),
        JSON.stringify(businessInfo)
      );
    } catch {
      /* ignore */
    }
  }
};

export const loadBusinessInfoLocal = async (
  businessId: number
): Promise<any | null> => {
  try {
    const key = getStorageKey(businessId);
    if (typeof window !== "undefined" && window.electronStore) {
      const stored = await window.electronStore.get(key);
      if (stored) return stored;
    } else {
      const stored = localStorage.getItem(key);
      if (stored) return JSON.parse(stored);
    }
    return null;
  } catch {
    try {
      const stored = localStorage.getItem(getStorageKey(businessId));
      if (stored) return JSON.parse(stored);
    } catch {
      /* ignore */
    }
    return null;
  }
};

export const clearBusinessInfoCache = (businessId?: number | null) => {
  if (businessId != null) {
    memoryCache.delete(businessId);
  } else {
    memoryCache.clear();
  }
};

function enrichBusinessData(data: any) {
  if (data.sucursales && data.sucursales.length > 0) {
    const sucursalPrincipal =
      data.sucursales.find((s: any) => s.esPrincipal) || data.sucursales[0];
    data.direccion = sucursalPrincipal.direccion || data.direccion;
    data.telefono = sucursalPrincipal.telefono || data.telefono;
    data.sucursalActiva = sucursalPrincipal;
  }
  if (!data.sistemaPago) {
    data.sistemaPago = "redondeo";
  }
  return data;
}

export async function fetchBusinessInfoShared(
  businessId: number,
  API_URL: string,
  appId: string | null
): Promise<any> {
  const existing = memoryCache.get(businessId);
  if (existing?.data) return existing.data;
  if (existing?.promise) return existing.promise;

  const promise = (async () => {
    const response = await fetch(
      `${API_URL}/api/business/${businessId}?include=sucursales,configuracionAfip`,
      {
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
      }
    );

    if (!response.ok) {
      throw new Error("Error al cargar información del negocio");
    }

    const raw = await response.json();
    const data = enrichBusinessData(raw);
    memoryCache.set(businessId, { data, promise: null });
    await saveBusinessInfoLocal(businessId, data);
    return data;
  })().catch((error) => {
    const entry = memoryCache.get(businessId);
    if (entry && !entry.data) {
      memoryCache.delete(businessId);
    }
    throw error;
  });

  memoryCache.set(businessId, { data: null, promise });
  return promise;
}
