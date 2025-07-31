import { Navigate } from "react-router-dom";
import { useBusiness } from "@/contexts/BusinessContext";
import { useAuth } from "@/contexts/AuthContext";

export default function AppRouter() {
  const { hasAdminConfigured, loading, adminData } = useBusiness();
  const { user } = useAuth();

  console.log(
    "🎯 AppRouter render - loading:",
    loading,
    "hasAdminConfigured:",
    hasAdminConfigured(),
    "adminData:",
    !!adminData
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
    console.log("🔍 Estado de configuración:", {
      adminData: !!adminData,
      hasAdminConfigured: hasAdminConfigured(),
    });
    return <Navigate to="/admin-login" replace />;
  }

  // Si hay admin configurado y usuario autenticado, redirigir al dashboard
  if (user) {
    console.log("🔄 Redirigiendo a /dashboard (usuario autenticado)");
    return <Navigate to="/dashboard" replace />;
  }

  // Si hay admin configurado pero no hay usuario, redirigir a UserLogin
  console.log("🔄 Redirigiendo a /user-login (admin configurado, sin usuario)");
  console.log("🔍 Configuración válida:", {
    adminData: !!adminData,
    businessId: adminData?.businessId,
    businessName: adminData?.businessName,
  });

  return <Navigate to="/user-login" replace />;
}
