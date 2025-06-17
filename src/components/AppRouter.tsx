import { Navigate } from "react-router-dom";
import { useBusiness } from "@/contexts/BusinessContext";

export default function AppRouter() {
  const { hasAdminConfigured, loading } = useBusiness();

  console.log(
    "🎯 AppRouter render - loading:",
    loading,
    "hasAdminConfigured:",
    hasAdminConfigured()
  );

  if (loading) {
    console.log("🔄 AppRouter: Mostrando loader porque loading =", loading);
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-center">
          <div className="mb-4 h-8 w-8 animate-spin rounded-full border-4 border-emerald-400 border-t-transparent mx-auto"></div>
          <p>Cargando configuración...</p>
        </div>
      </div>
    );
  }

  // Si no hay admin configurado, redirigir a AdminLogin
  if (!hasAdminConfigured()) {
    console.log("🔄 Redirigiendo a /admin-login (no hay admin configurado)");
    return <Navigate to="/admin-login" replace />;
  }

  // Si hay admin configurado, redirigir a UserLogin
  console.log("🔄 Redirigiendo a /user-login (admin configurado)");

  // DEBUG: Usar setTimeout para ver si la navegación está funcionando
  setTimeout(() => {
    console.log("🕐 Verificando que la navegación funcionó...");
    console.log("🌐 URL actual:", window.location.href);
    console.log("🌐 Pathname:", window.location.pathname);
  }, 100);

  return <Navigate to="/user-login" replace />;
}
