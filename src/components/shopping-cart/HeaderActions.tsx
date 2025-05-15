import { Button } from "@/components/ui/button";
import { UserMenu } from "@/components/user-menu";
import { History, Store } from "lucide-react";

interface HeaderActionsProps {
  userName: string;
  userEmail: string;
  canCloseCashDesk: boolean;
  onOpenOrders: () => void;
  onOpenClosing: () => void;
}

export function HeaderActions({
  userName,
  userEmail,
  canCloseCashDesk,
  onOpenOrders,
  onOpenClosing,
}: HeaderActionsProps) {
  return (
    <div className="flex items-center gap-2">
      {/* Botón de Órdenes recientes */}
      <Button
        className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
        onClick={onOpenOrders}
      >
        <History />
        Órdenes
      </Button>

      {canCloseCashDesk && (
        <Button
          className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
          onClick={onOpenClosing}
        >
          <Store />
          Cierre de caja
        </Button>
      )}

      <UserMenu user={{ nombre: userName, email: userEmail }} />
    </div>
  );
}
