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
            window.location.href =
              credentials.user.rol.nombre === "ADMIN" ? "/admin" : "/cart";
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
          ...(appId && { "X-App-ID": appId })
        },
        body: JSON.stringify(values),
        credentials: "include",
      });

      if (!response.ok) {
        throw new Error("Error en el login");
      }

      const sessionResponse = await fetch(`${API_URL}/api/auth/session`, {
        credentials: "include",
      });
      const sessionData = await sessionResponse.json();

      // Guardar credenciales para uso offline
      saveOfflineCredentials({
        email: values.email,
        user: sessionData.user,
      });

      // Actualizar el contexto con los datos completos
      updateUser(sessionData.user);

      window.location.href =
        sessionData.user.rol.nombre === "ADMIN" ? "/admin" : "/cart";
    } catch (error) {
      toast.error("Error al iniciar sesión", {
        description: "Verifica tus credenciales e intenta nuevamente",
      });
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
    </div>
  );
}
