import { Button } from "@/components/ui/button";
import { History, Store, Receipt } from "lucide-react";
import { UserMenu } from "@/components/user-menu";
import { useNavigate } from "react-router-dom";

interface HeaderActionsProps {
  onOrdersClick: () => void;
  onClosingClick: () => void;
  user: any;
  userMenuUser: { nombre: string; email: string };
  businessInfo?: any; // Agregar businessInfo como prop opcional
}

export const HeaderActions = ({
  onOrdersClick,
  onClosingClick,
  user,
  userMenuUser,
  businessInfo,
}: HeaderActionsProps) => {
  const navigate = useNavigate();

  const handleCuentaCorrienteClick = () => {
    navigate("/facturas");
  };

  return (
    <div className="w-full flex justify-end items-center gap-4">
      {/* Botón de Órdenes recientes */}
      <Button
        className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
        onClick={onOrdersClick}
      >
        <History />
        Órdenes
      </Button>

      {/* Botón de Cuenta Corriente - Solo mostrar si la facturación está habilitada */}
      {businessInfo?.facturacionHabilitada && (
        <Button
          className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
          onClick={handleCuentaCorrienteClick}
        >
          <Receipt />
          Cuenta Corriente
        </Button>
      )}

      {user?.permisos?.cierreDeCajaEnabled && (
        <Button
          className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
          onClick={onClosingClick}
        >
          <Store />
          Cierre de caja
        </Button>
      )}

      {/* <UserMenu user={userMenuUser} /> */}
    </div>
  );
};
