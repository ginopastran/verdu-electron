import { AuthProvider } from "./contexts/AuthContext";
import { OfflineModeProvider } from "./contexts/OfflineModeContext";
import { Toaster } from "sonner";
import { useAuth } from "./contexts/AuthContext";
import Login from "./pages/Login";
import ShoppingCart from "./components/ShoppingCart";

function AppContent() {
  const { user, loading } = useAuth();

  if (loading) {
    return <div>Cargando...</div>;
  }

  if (!user) {
    return <Login />;
  }

  return <ShoppingCart />;
}

export default function App() {
  return (
    <AuthProvider>
      <OfflineModeProvider>
        <Toaster richColors position="top-center" theme="light" />
        <AppContent />
      </OfflineModeProvider>
    </AuthProvider>
  );
}
