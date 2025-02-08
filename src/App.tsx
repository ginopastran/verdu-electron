import {
  HashRouter as Router,
  Routes,
  Route,
  Navigate,
} from "react-router-dom";
import { AuthProvider } from "./contexts/AuthContext";
import { OfflineModeProvider } from "./contexts/OfflineModeContext";
import { Toaster } from "sonner";
import { useAuth } from "./contexts/AuthContext";
import Login from "./pages/Login";
import ShoppingCart from "./components/ShoppingCart";
import AdminMessage from "./components/AdminMessage";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();

  if (loading) {
    return <div>Cargando...</div>;
  }

  if (!user) {
    return <Navigate to="/" replace />;
  }

  // Redirigir administradores al mensaje de acceso no permitido
  if (user.rol.nombre === "ADMIN") {
    return <Navigate to="/admin-message" replace />;
  }

  return <>{children}</>;
}

export default function App() {
  return (
    <Router>
      <AuthProvider>
        <OfflineModeProvider>
          <Toaster richColors position="top-center" theme="light" />
          <Routes>
            <Route path="/" element={<Login />} />
            <Route path="/admin-message" element={<AdminMessage />} />
            <Route
              path="/cart"
              element={
                <ProtectedRoute>
                  <ShoppingCart />
                </ProtectedRoute>
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </OfflineModeProvider>
      </AuthProvider>
    </Router>
  );
}
