import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
import { useOfflineAuth } from "@/hooks/useOfflineAuth";
import { useOfflineMode } from "@/hooks/useOfflineMode";
import { useState } from "react";
import { Navigate } from "react-router-dom";

const formSchema = z.object({
  email: z.string().email({
    message: "Email inválido.",
  }),
  password: z.string().min(6, {
    message: "La contraseña debe tener al menos 6 caracteres.",
  }),
});

export default function LoginPage() {
  const { updateUser } = useAuth();
  const { isOnline } = useOfflineMode();
  const { saveOfflineCredentials } = useOfflineAuth();
  const [redirectTo, setRedirectTo] = useState<string | null>(null);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  const onSubmit = async (values: z.infer<typeof formSchema>) => {
    try {
      if (!isOnline) {
        const stored = localStorage.getItem("offlineCredentials");
        if (stored) {
          const credentials = JSON.parse(stored);
          if (credentials.email === values.email) {
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

      console.log("Enviando APP_ID:", appId);

      const response = await fetch(`${API_URL}/api/login`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-App-ID": appId,
        },
        body: JSON.stringify(values),
        credentials: "include",
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || "Error en el login");
      }

      const loginData = await response.json();
      console.log("Respuesta de login:", loginData);

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
      console.log("Datos de sesión:", sessionData);

      // Verificar si tenemos datos de usuario válidos
      if (!sessionData.user) {
        throw new Error("No se pudo obtener la información del usuario");
      }

      // Verificar si el usuario tiene un rol asignado
      if (!sessionData.user.rol) {
        throw new Error("El usuario no tiene un rol asignado");
      }

      // Guardar credenciales para uso offline
      saveOfflineCredentials({
        email: values.email,
        user: sessionData.user,
      });

      // Actualizar el contexto con los datos completos
      updateUser(sessionData.user);

      setRedirectTo(
        sessionData.user.rol.nombre === "ADMIN" ? "/admin-message" : "/cart"
      );
    } catch (error) {
      console.error("Error completo:", error); // Debug
      toast.error("Error al iniciar sesión", {
        description:
          error instanceof Error
            ? error.message
            : "Verifica tus credenciales e intenta nuevamente",
      });
    }
  };

  const checkAppId = async () => {
    try {
      const { ipcRenderer } = window.require("electron");
      await ipcRenderer.invoke("show-app-id");
    } catch (error) {
      console.error("Error al mostrar APP_ID:", error);
    }
  };

  const toggleDevTools = async () => {
    try {
      const { ipcRenderer } = window.require("electron");
      await ipcRenderer.invoke("toggle-devtools");
    } catch (error) {
      console.error("Error al abrir DevTools:", error);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="w-full max-w-md space-y-8 px-4">
        <div className="space-y-2 text-center">
          <h1 className="text-3xl font-bold text-emerald-gradient">
            Iniciar Sesión
          </h1>
          <p className="text-gray-500">
            Ingresa tus credenciales para acceder al sistema
          </p>
        </div>

        <div className="flex gap-2 justify-center">
          <button
            onClick={checkAppId}
            className="text-sm text-gray-500 hover:text-gray-700"
          >
            Verificar APP_ID
          </button>
          <button
            onClick={toggleDevTools}
            className="text-sm text-gray-500 hover:text-gray-700"
          >
            Abrir DevTools
          </button>
        </div>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input placeholder="tu@email.com" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Contraseña</FormLabel>
                  <FormControl>
                    <Input type="password" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" className="w-full bg-emerald-gradient">
              Iniciar Sesión
            </Button>
          </form>
        </Form>
      </div>
      {redirectTo && <Navigate to={redirectTo} replace />}
    </div>
  );
}
