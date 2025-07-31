const BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

export const api = {
  async request(endpoint: string, options: RequestInit = {}) {
    const url = `${BASE_URL}${endpoint}`;

    const config: RequestInit = {
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
      ...options,
    };

    try {
      const response = await fetch(url, config);

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new ApiError(
          response.status,
          errorData.error || `HTTP error! status: ${response.status}`
        );
      }

      return response.json();
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }
      throw new ApiError(
        0,
        error instanceof Error ? error.message : "Network error"
      );
    }
  },

  async get(endpoint: string, params?: Record<string, any>) {
    // Asegurar que endpoint comience con /api/
    const apiEndpoint = endpoint.startsWith("/api/")
      ? endpoint
      : `/api${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;
    const url = new URL(`${BASE_URL}${apiEndpoint}`);

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== "") {
          url.searchParams.append(key, String(value));
        }
      });
    }

    return this.request(
      apiEndpoint + (params ? `?${url.searchParams.toString()}` : "")
    );
  },

  async post(endpoint: string, data?: any) {
    // Asegurar que endpoint comience con /api/
    const apiEndpoint = endpoint.startsWith("/api/")
      ? endpoint
      : `/api${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

    return this.request(apiEndpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  },

  async put(endpoint: string, data?: any) {
    // Asegurar que endpoint comience con /api/
    const apiEndpoint = endpoint.startsWith("/api/")
      ? endpoint
      : `/api${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

    return this.request(apiEndpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  },

  async delete(endpoint: string) {
    // Asegurar que endpoint comience con /api/
    const apiEndpoint = endpoint.startsWith("/api/")
      ? endpoint
      : `/api${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

    return this.request(apiEndpoint, {
      method: "DELETE",
    });
  },
};
