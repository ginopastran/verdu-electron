import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Loader2, UserCog, Building2 } from "lucide-react";
import { toast } from "sonner";
import { useBusiness } from "@/contexts/BusinessContext";

export default function AdminLogin() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const navigate = useNavigate();
  const { setAdminData } = useBusiness();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password) {
      setError("Por favor, ingresa email y contraseña");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const API_URL = import.meta.env.VITE_API_URL;
      const appId = import.meta.env.VITE_APP_ID;

      // Primer paso: Login del admin
      const loginResponse = await fetch(`${API_URL}/api/login`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-App-ID": appId,
        },
        body: JSON.stringify({
          email,
          password,
        }),
        credentials: "include",
      });

      if (!loginResponse.ok) {
        const errorData = await loginResponse.json();
        throw new Error(errorData.message || "Error en el login");
      }

      const loginData = await loginResponse.json();

      if (!loginData.token) {
        throw new Error("No se recibió el token de autenticación");
      }

      // Segundo paso: Obtener datos de sesión
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

      // Verificar que sea un admin
      if (!sessionData.user || sessionData.user.rol?.nombre !== "ADMIN") {
        throw new Error(
          "Solo los administradores pueden configurar la aplicación"
        );
      }

      // Verificar que tenga businessId (debería venir en los datos del usuario)
      const businessId =
        sessionData.user.businessId || sessionData.user.sucursal?.businessId;

      if (!businessId) {
        throw new Error(
          "No se pudo obtener el ID del negocio del administrador"
        );
      }

      // Obtener información del business
      const businessResponse = await fetch(
        `${API_URL}/api/business/${businessId}`,
        {
          headers: {
            "Content-Type": "application/json",
            "X-App-ID": appId,
            Authorization: `Bearer ${loginData.token}`,
          },
        }
      );

      let businessName = "Negocio";
      if (businessResponse.ok) {
        const businessData = await businessResponse.json();
        businessName = businessData.nombre || businessData.name || "Negocio";
      }

      // Guardar datos del admin
      const adminData = {
        id: sessionData.user.id,
        nombre: sessionData.user.nombre,
        email: sessionData.user.email,
        businessId: businessId,
        businessName: businessName,
      };

      setAdminData(adminData);

      toast.success("Configuración guardada", {
        description: `Negocio: ${businessName}`,
      });

      // Redirigir al login de usuarios
      navigate("/user-login");
    } catch (error) {
      console.error("Error en login de admin:", error);
      const message =
        error instanceof Error ? error.message : "Error desconocido";
      setError(message);
      toast.error("Error al configurar", {
        description: message,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-emerald-50 to-blue-50 p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-emerald-100">
            <UserCog className="h-6 w-6 text-emerald-600" />
          </div>
          <CardTitle className="text-2xl font-bold text-gray-900">
            Configuración Inicial
          </CardTitle>
          <CardDescription className="text-gray-600">
            Ingresa con una cuenta de administrador para configurar el negocio
          </CardDescription>
        </CardHeader>

        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="email">Email del Administrador</Label>
              <Input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="admin@empresa.com"
                disabled={loading}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="password">Contraseña</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                disabled={loading}
                required
              />
            </div>

            <Button type="submit" className="w-full" disabled={loading}>
              {loading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Configurando...
                </>
              ) : (
                <>
                  <Building2 className="mr-2 h-4 w-4" />
                  Configurar Negocio
                </>
              )}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
