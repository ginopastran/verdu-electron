import { useState, useEffect, useCallback } from "react";
import { Navigate, useNavigate } from "react-router-dom";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
import { useBusiness } from "@/contexts/BusinessContext";
import { useOfflineAuth } from "@/hooks/useOfflineAuth";
import { useOfflineMode } from "@/hooks/useOfflineMode";

// UI Components
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Card } from "@/components/ui/card";
import { PlusCircle, Loader2, Settings, Building2 } from "lucide-react";

// Password form schema
const passwordFormSchema = z.object({
  password: z.string().min(4, {
    message: "La contraseña debe tener al menos 4 caracteres.",
  }),
});

// Interface for vendor data
interface Vendor {
  id: number;
  nombre: string;
  email: string;
  avatar?: string;
}

export default function UserLoginPage() {
  console.log("🚀 UserLogin: Componente montándose...");

  const { updateUser } = useAuth();
  const { businessId, adminData, clearAdminData } = useBusiness();
  const { isOnline } = useOfflineMode();
  const { saveOfflineCredentials } = useOfflineAuth();
  const navigate = useNavigate();

  console.log("📊 UserLogin state:", {
    businessId,
    adminData: !!adminData,
    isOnline,
  });

  // SAFETY: Si businessId está undefined pero tenemos adminData,
  // extraer businessId del adminData
  const effectiveBusinessId = businessId || adminData?.businessId;
  console.log("🔧 Usando businessId:", effectiveBusinessId);

  const [redirectTo, setRedirectTo] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [loadingVendors, setLoadingVendors] = useState(true);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [selectedVendor, setSelectedVendor] = useState<Vendor | null>(null);
  const [passwordDialogOpen, setPasswordDialogOpen] = useState(false);

  // Initialize form for password input
  const form = useForm<z.infer<typeof passwordFormSchema>>({
    resolver: zodResolver(passwordFormSchema),
    defaultValues: {
      password: "",
    },
  });

  // Fetch vendors from API
  const fetchVendors = useCallback(async () => {
    console.log("🔄 UserLogin: Iniciando carga de vendors...");
    console.log(
      "📊 businessId:",
      businessId,
      "effectiveBusinessId:",
      effectiveBusinessId,
      "isOnline:",
      isOnline
    );

    try {
      setLoadingVendors(true);

      if (!isOnline) {
        console.log("❌ Sin conexión a internet");
        toast.error("No hay conexión a internet");
        setLoadingVendors(false);
        return;
      }

      if (!effectiveBusinessId) {
        console.log("❌ No hay businessId configurado");
        toast.error("No se ha configurado el negocio");
        setLoadingVendors(false);
        return;
      }

      const API_URL = import.meta.env.VITE_API_URL;
      const appId = import.meta.env.VITE_APP_ID;

      console.log(
        "🌐 Haciendo petición a API...",
        `${API_URL}/api/usuarios/vendedores?businessId=${effectiveBusinessId}`
      );

      const response = await fetch(
        `${API_URL}/api/usuarios/vendedores?businessId=${effectiveBusinessId}`,
        {
          headers: {
            "Content-Type": "application/json",
            "X-App-ID": appId,
          },
        }
      );

      console.log(
        "📡 Respuesta recibida:",
        response.status,
        response.statusText
      );

      if (!response.ok) {
        throw new Error("Error al obtener vendedores");
      }

      const data = await response.json();
      console.log("✅ Vendors cargados:", data.length, "vendors");
      setVendors(data);
    } catch (error) {
      console.error("❌ Error fetching vendors:", error);
      toast.error("Error al cargar vendedores", {
        description: "Por favor, intenta nuevamente más tarde.",
      });
    } finally {
      console.log("✅ fetchVendors terminado, setLoadingVendors(false)");
      setLoadingVendors(false);
    }
  }, [businessId, effectiveBusinessId, isOnline]);

  // Fetch vendors on component mount
  useEffect(() => {
    console.log(
      "🔄 UserLogin useEffect triggered - effectiveBusinessId:",
      effectiveBusinessId
    );
    if (effectiveBusinessId) {
      fetchVendors();
    } else {
      console.log("⏳ Esperando businessId...");
    }
  }, [effectiveBusinessId, isOnline, fetchVendors]);

  // Add keyboard event listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore key events if they occur within input fields
      if (e.target instanceof HTMLInputElement) return;

      // Only handle number keys 1-9
      if (!/^[1-9]$/.test(e.key)) return;

      // Prevent default to avoid the key from being entered in any input
      e.preventDefault();

      const vendorIndex = parseInt(e.key) - 1;
      if (vendors[vendorIndex]) {
        handleVendorSelect(vendors[vendorIndex]);
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [vendors]);

  // Select a vendor and open password dialog
  const handleVendorSelect = (vendor: Vendor) => {
    setSelectedVendor(vendor);
    setPasswordDialogOpen(true);
    form.reset();
  };

  // Handle password submission
  const onSubmitPassword = async (
    values: z.infer<typeof passwordFormSchema>
  ) => {
    if (!selectedVendor) return;

    try {
      setIsLoading(true);

      if (!isOnline) {
        const stored = localStorage.getItem("offlineCredentials");
        if (stored) {
          const credentials = JSON.parse(stored);
          if (credentials.email === selectedVendor.email) {
            updateUser(credentials.user);
            setRedirectTo(
              credentials.user.rol.nombre === "ADMIN"
                ? "/admin-message"
                : "/cart"
            );
            return;
          }
        }
        toast.error("No hay credenciales guardadas para modo offline");
        return;
      }

      const API_URL = import.meta.env.VITE_API_URL;
      const appId = import.meta.env.VITE_APP_ID;

      const response = await fetch(`${API_URL}/api/login`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-App-ID": appId,
        },
        body: JSON.stringify({
          email: selectedVendor.email,
          password: values.password,
        }),
        credentials: "include",
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || "Error en el login");
      }

      const loginData = await response.json();

      if (!loginData.token) {
        throw new Error("No se recibió el token de autenticación");
      }

      const sessionResponse = await fetch(`${API_URL}/api/auth/session`, {
        headers: {
          "Content-Type": "application/json",
          "X-App-ID": appId,
          Authorization: `Bearer ${loginData.token}`,
          "Cache-Control": "no-cache",
        },
        credentials: "include",
      });

      if (!sessionResponse.ok) {
        const errorData = await sessionResponse.json();
        throw new Error(errorData.message || "Error al obtener la sesión");
      }

      const sessionData = await sessionResponse.json();

      // Verificar si tenemos datos de usuario válidos
      if (!sessionData.user) {
        throw new Error("No se pudo obtener la información del usuario");
      }

      // Verificar si el usuario tiene un rol asignado
      if (!sessionData.user.rol) {
        throw new Error("El usuario no tiene un rol asignado");
      }

      // Ensure we have the permisos object with proper properties
      if (!sessionData.user.permisos) {
        sessionData.user.permisos = {};
      }

      // Check permissions
      if (sessionData.user.pesoManualEnabled !== undefined) {
        sessionData.user.permisos.pesoManualEnabled =
          sessionData.user.pesoManualEnabled;
        delete sessionData.user.pesoManualEnabled;
      }

      if (sessionData.user.cierreDeCajaEnabled !== undefined) {
        sessionData.user.permisos.cierreDeCajaEnabled =
          sessionData.user.cierreDeCajaEnabled;
        delete sessionData.user.cierreDeCajaEnabled;
      }

      // Guardar credenciales para uso offline
      saveOfflineCredentials({
        email: selectedVendor.email,
        user: sessionData.user,
      });

      // Actualizar el contexto con los datos completos
      updateUser(sessionData.user);

      setPasswordDialogOpen(false);
      setRedirectTo(
        sessionData.user.rol.nombre === "ADMIN" ? "/admin-message" : "/cart"
      );
    } catch (error) {
      console.error("Error completo:", error);
      toast.error("Error al iniciar sesión", {
        description:
          error instanceof Error
            ? error.message
            : "Verifica tu contraseña e intenta nuevamente",
      });
    } finally {
      setIsLoading(false);
    }
  };

  // ✅ MEJORADO: Handle reconfigure business con navegación robusta
  const handleReconfigure = async () => {
    try {
      console.log("🔄 Iniciando reconfiguración del negocio...");

      // Mostrar feedback visual inmediato
      toast.info("Reconfigurar negocio", {
        description: "Limpiando configuración actual...",
        duration: 2000,
      });

      // Limpiar datos del admin
      await clearAdminData();
      console.log("✅ Datos del admin limpiados");

      // Limpiar cualquier estado de autenticación actual
      updateUser(null);
      console.log("✅ Usuario deslogueado");

      // Limpiar cualquier cache/estado local adicional
      if (typeof window !== "undefined") {
        // Limpiar localStorage relacionado con autenticación
        localStorage.removeItem("offlineCredentials");
        console.log("✅ Cache offline limpiado");
      }

      // Usar navegación programática directa con fallback
      console.log("🚀 Navegando a /admin-login");
      navigate("/admin-login", { replace: true });

      // Verificar navegación después de un tiempo
      setTimeout(() => {
        console.log("🔍 Verificando navegación...");
        console.log("🌐 URL actual:", window.location.pathname);

        if (window.location.pathname !== "/admin-login") {
          console.log("⚠️ Navegación falló, intentando forzar...");
          // Fallback usando window.location
          window.location.href = "#/admin-login";

          // Si aún falla, recargar completamente
          setTimeout(() => {
            if (window.location.pathname !== "/admin-login") {
              console.log("🔄 Forzando recarga para limpiar estado...");
              window.location.reload();
            }
          }, 1000);
        } else {
          toast.success("Configuración limpiada", {
            description: "Redirigido al login de administrador",
            duration: 3000,
          });
        }
      }, 500);
    } catch (error) {
      console.error("❌ Error durante la reconfiguración:", error);
      toast.error("Error al reconfigurar", {
        description:
          "No se pudo limpiar la configuración. Recargando aplicación...",
        duration: 3000,
      });

      // Como último recurso, recargar la página para limpiar todo
      setTimeout(() => {
        window.location.reload();
      }, 2000);
    }
  };

  // If user should be redirected, do it
  if (redirectTo) {
    return <Navigate to={redirectTo} replace />;
  }

  return (
    <div className="flex min-h-screen flex-col items-center bg-white">
      <div className="w-full max-w-5xl p-6">
        <div className="flex items-center justify-between mb-8 mt-8">
          <h1 className="text-center text-5xl font-bold text-emerald-gradient flex-1">
            ¿Quién está vendiendo?
          </h1>
          <Button
            variant="outline"
            size="sm"
            onClick={handleReconfigure}
            className="flex items-center gap-2 hover:bg-orange-50 hover:border-orange-300 hover:text-orange-700 transition-colors"
            title="Limpiar configuración actual y volver al login de administrador"
          >
            <Settings className="h-4 w-4" />
            Reconfigurar
          </Button>
        </div>

        {/* Business info */}
        {adminData && (
          <div className="mb-6 p-4 bg-emerald-50 rounded-lg border border-emerald-200">
            <div className="flex items-center gap-2 text-emerald-700">
              <Building2 className="h-4 w-4" />
              <span className="font-medium">
                {adminData.businessName || "Negocio"} - Configurado por{" "}
                {adminData.nombre}
              </span>
            </div>
          </div>
        )}

        {loadingVendors ? (
          <div className="flex min-h-[300px] items-center justify-center">
            <Loader2 className="h-12 w-12 animate-spin text-emerald-400" />
          </div>
        ) : (
          <>
            <div className="grid grid-cols-2 gap-8 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
              {vendors.map((vendor, index) => (
                <VendorCard
                  key={vendor.id}
                  vendor={vendor}
                  index={index + 1}
                  onClick={() => handleVendorSelect(vendor)}
                />
              ))}
            </div>
            <p className="text-center text-gray-500 mt-8 font-light">
              También puedes presionar las teclas numéricas 1-9 para seleccionar
              un vendedor
            </p>
          </>
        )}

        {/* Password Dialog */}
        {selectedVendor && (
          <Dialog
            open={passwordDialogOpen}
            onOpenChange={setPasswordDialogOpen}
          >
            <DialogContent className="sm:max-w-md">
              <DialogHeader>
                <DialogTitle className="text-emerald-gradient text-2xl font-bold">
                  Contraseña para {selectedVendor.nombre}
                </DialogTitle>
              </DialogHeader>

              <Form {...form}>
                <form onSubmit={form.handleSubmit(onSubmitPassword)}>
                  <FormField
                    control={form.control}
                    name="password"
                    render={({ field }) => (
                      <FormItem className="mt-4">
                        <FormLabel>Contraseña</FormLabel>
                        <FormControl>
                          <Input
                            type="password"
                            placeholder="Ingresa tu contraseña"
                            autoFocus
                            {...field}
                            disabled={isLoading}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <DialogFooter className="mt-6">
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => setPasswordDialogOpen(false)}
                      disabled={isLoading}
                    >
                      Cancelar
                    </Button>
                    <Button
                      type="submit"
                      className="bg-emerald-gradient"
                      disabled={isLoading}
                    >
                      {isLoading ? (
                        <div className="flex items-center gap-2">
                          <Loader2 className="h-4 w-4 animate-spin" />
                          <span>Iniciando sesión...</span>
                        </div>
                      ) : (
                        "Iniciar Sesión"
                      )}
                    </Button>
                  </DialogFooter>
                </form>
              </Form>
            </DialogContent>
          </Dialog>
        )}
      </div>
    </div>
  );
}

function VendorCard({
  vendor,
  index,
  onClick,
}: {
  vendor: Vendor;
  index: number;
  onClick: () => void;
}) {
  // Get first letter of name for the avatar
  const firstLetter = vendor.nombre.charAt(0).toUpperCase();

  return (
    <div className="flex flex-col items-center">
      <button
        onClick={onClick}
        className="transition-transform duration-200 hover:scale-105 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 relative"
      >
        <div className="flex h-36 w-36 items-center justify-center rounded-lg shadow-lg md:h-40 md:w-40 bg-emerald-gradient">
          {vendor.avatar ? (
            <img
              src={vendor.avatar}
              alt={vendor.nombre}
              className="h-full w-full rounded-lg object-cover"
            />
          ) : (
            <span className="text-5xl font-bold text-white">{firstLetter}</span>
          )}
        </div>
        {index <= 9 && (
          <div className="absolute top-4 left-4 bg-black/40 text-white w-6 h-6 flex items-center justify-center rounded-full text-base font-bold  transform -translate-x-2 -translate-y-2 z-10">
            {index}
          </div>
        )}
      </button>
      <p className="mt-2 text-center text-lg font-bold text-emerald-gradient">
        {vendor.nombre}
      </p>
    </div>
  );
}
