import { useState, useEffect } from "react";
import { Navigate } from "react-router-dom";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
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
import { PlusCircle, Loader2 } from "lucide-react";

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

export default function LoginPage() {
  const { updateUser } = useAuth();
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
    fetchVendors();
  }, []);

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

      const API_URL = import.meta.env.VITE_API_URL;
      const appId = import.meta.env.VITE_APP_ID;

      const response = await fetch(
        `${API_URL}/api/usuarios/vendedores?businessId=1`,
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

  return (
    <div className="flex min-h-screen flex-col items-center bg-white">
      <div className="w-full max-w-5xl p-6">
        <h1 className="mb-12 mt-8 text-center text-5xl font-bold text-emerald-gradient">
          ¿Quién está vendiendo?
        </h1>

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

        {/* <div className="mt-12 text-center">
          <Button
            variant="ghost"
            className="text-gray-400 hover:text-white"
            onClick={fetchVendors}
            disabled={loadingVendors}
          >
            {loadingVendors ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Cargando...
              </>
            ) : (
              "Actualizar perfiles"
            )}
          </Button>
        </div> */}
      </div>

      {/* Password Dialog */}
      {selectedVendor && (
        <Dialog open={passwordDialogOpen} onOpenChange={setPasswordDialogOpen}>
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

      {redirectTo && <Navigate to={redirectTo} replace />}
    </div>
  );
}

// Vendor Card Component
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
