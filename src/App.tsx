import {
  HashRouter as Router,
  Routes,
  Route,
  Navigate,
} from "react-router-dom";
import { AuthProvider } from "./contexts/AuthContext";
import { BusinessProvider } from "./contexts/BusinessContext";
import { OfflineModeProvider } from "./contexts/OfflineModeContext";
import { Toaster } from "sonner";
import { useAuth } from "./contexts/AuthContext";
import AdminLogin from "./pages/AdminLogin";
import UserLogin from "./pages/UserLogin";
// import ShoppingCart from "./components/ShoppingCart";
import ShoppingCartRefactored from "./components/ShoppingCartRefactored";
import AdminMessage from "./components/AdminMessage";
import AppRouter from "./components/AppRouter";
import UpdateNotification from "./components/UpdateNotification";
import { ElectronDebug } from "./components/ElectronDebug";
import { CartWrapper } from "./components/CartWrapper";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();

  if (loading) {
    return <div>Cargando...</div>;
  }

  if (!user) {
    return <Navigate to="/user-login" replace />;
  }

  // Redirigir administradores al mensaje de acceso no permitido
  if (user.role === "ADMIN") {
    return <Navigate to="/admin-message" replace />;
  }

  return <>{children}</>;
}

export default function App() {
  return (
    <BusinessProvider>
      <Router>
        <AuthProvider>
          <OfflineModeProvider>
            <Toaster richColors position="top-center" theme="light" />
            <UpdateNotification />
            <Routes>
              <Route path="/" element={<AppRouter />} />
              <Route path="/admin-login" element={<AdminLogin />} />
              <Route path="/user-login" element={<UserLogin />} />
              <Route path="/admin-message" element={<AdminMessage />} />
              <Route
                path="/cart"
                element={
                  <ProtectedRoute>
                    <CartWrapper />
                  </ProtectedRoute>
                }
              />
              <Route
                path="/cart-refactored"
                element={
                  <ProtectedRoute>
                    <CartWrapper />
                  </ProtectedRoute>
                }
              />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </OfflineModeProvider>
        </AuthProvider>
      </Router>
    </BusinessProvider>
  );
}
