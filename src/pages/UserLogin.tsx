import { useState, useEffect } from "react";
import { Navigate } from "react-router-dom";
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
  const { updateUser } = useAuth();
  const { businessId, adminData, clearAdminData } = useBusiness();
  const { isOnline } = useOfflineMode();
  const { saveOfflineCredentials } = useOfflineAuth();
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

  // Fetch vendors on component mount
  useEffect(() => {
    if (businessId) {
      fetchVendors();
    }
  }, [businessId]);

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

  // Fetch vendors from API
  const fetchVendors = async () => {
    try {
      setLoadingVendors(true);

      if (!isOnline) {
        toast.error("No hay conexión a internet");
        setLoadingVendors(false);
        return;
      }

      if (!businessId) {
        toast.error("No se ha configurado el negocio");
        setLoadingVendors(false);
        return;
      }

      const API_URL = import.meta.env.VITE_API_URL;
      const appId = import.meta.env.VITE_APP_ID;

      const response = await fetch(
        `${API_URL}/api/usuarios/vendedores?businessId=${businessId}`,
        {
          headers: {
            "Content-Type": "application/json",
            "X-App-ID": appId,
          },
        }
      );

      if (!response.ok) {
        throw new Error("Error al obtener vendedores");
      }

      const data = await response.json();
      setVendors(data);
    } catch (error) {
      console.error("Error fetching vendors:", error);
      toast.error("Error al cargar vendedores", {
        description: "Por favor, intenta nuevamente más tarde.",
      });
    } finally {
      setLoadingVendors(false);
    }
  };

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

  // Handle reconfigure business
  const handleReconfigure = () => {
    clearAdminData();
    // Esto hará que la app redirija al AdminLogin automáticamente
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
            className="flex items-center gap-2"
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
        <Dialog open={passwordDialogOpen} onOpenChange={setPasswordDialogOpen}>
          <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
              <DialogTitle>
                Iniciar sesión como {selectedVendor?.nombre}
              </DialogTitle>
            </DialogHeader>
            <Form {...form}>
              <form
                onSubmit={form.handleSubmit(onSubmitPassword)}
                className="space-y-4"
              >
                <FormField
                  control={form.control}
                  name="password"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Contraseña</FormLabel>
                      <FormControl>
                        <Input
                          type="password"
                          placeholder="Ingresa tu contraseña"
                          {...field}
                          disabled={isLoading}
                          autoFocus
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <DialogFooter>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => setPasswordDialogOpen(false)}
                    disabled={isLoading}
                  >
                    Cancelar
                  </Button>
                  <Button type="submit" disabled={isLoading}>
                    {isLoading ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Iniciando...
                      </>
                    ) : (
                      "Iniciar sesión"
                    )}
                  </Button>
                </DialogFooter>
              </form>
            </Form>
          </DialogContent>
        </Dialog>
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
  const displayName = vendor.nombre || "Sin nombre";

  return (
    <Card className="group relative overflow-hidden border-2 border-gray-200 bg-white transition-all duration-200 hover:border-emerald-400 hover:shadow-lg cursor-pointer">
      <div
        className="flex h-32 flex-col items-center justify-center p-4 text-center"
        onClick={onClick}
      >
        {/* Avatar */}
        <div className="mb-3 flex h-16 w-16 items-center justify-center rounded-full bg-emerald-100 text-xl font-bold text-emerald-600 transition-colors group-hover:bg-emerald-200">
          {vendor.avatar ? (
            <img
              src={vendor.avatar}
              alt={displayName}
              className="h-full w-full rounded-full object-cover"
            />
          ) : (
            displayName.charAt(0).toUpperCase()
          )}
        </div>

        {/* Name */}
        <p className="text-sm font-medium text-gray-900 group-hover:text-emerald-600">
          {displayName}
        </p>

        {/* Index Badge */}
        <div className="absolute right-2 top-2 flex h-6 w-6 items-center justify-center rounded-full bg-gray-100 text-xs font-bold text-gray-600 transition-colors group-hover:bg-emerald-100 group-hover:text-emerald-600">
          {index}
        </div>
      </div>
    </Card>
  );
}
