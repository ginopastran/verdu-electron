import { Button } from "@/components/ui/button";
import { History, Store } from "lucide-react";
import { UserMenu } from "@/components/user-menu";

interface HeaderActionsProps {
  onOrdersClick: () => void;
  onClosingClick: () => void;
  user: any;
  userMenuUser: { nombre: string; email: string };
}

export const HeaderActions = ({
  onOrdersClick,
  onClosingClick,
  user,
  userMenuUser,
}: HeaderActionsProps) => {
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

      {user?.permisos?.cierreDeCajaEnabled && (
        <Button
          className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
          onClick={onClosingClick}
        >
          <Store />
          Cierre de caja
        </Button>
      )}

      <UserMenu user={userMenuUser} />
    </div>
  );
};
