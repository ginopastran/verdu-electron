import { useState } from "react";
import { LogOut, Scale, User } from "lucide-react";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
import { BalanzaDialog } from "@/components/BalanzaDialog";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface UserMenuProps {
  user: {
    nombre: string;
    email: string;
  };
}

export function UserMenu({ user }: UserMenuProps) {
  const { logout } = useAuth();
  const [balanzaAbierta, setBalanzaAbierta] = useState(false);

  const handleLogout = async () => {
    try {
      logout();
      toast.success("Sesión cerrada exitosamente");
    } catch (error) {
      toast.error("Error al cerrar sesión");
    }
  };

  return (
    <div className="flex items-center gap-2">
      <p className="font-medium">{user.nombre}</p>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            size="icon"
            className="h-10 w-10 rounded-lg [&_svg]:size-6 bg-emerald-gradient text-white hover:text-white"
          >
            <User />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel className="font-normal">
            <div className="flex flex-col space-y-1">
              <p className="text-sm font-medium leading-none">{user.nombre}</p>
              <p className="text-xs leading-none text-muted-foreground">
                {user.email}
              </p>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => setBalanzaAbierta(true)}
            className="cursor-pointer"
          >
            <Scale className="mr-2 h-4 w-4" />
            <span>Balanza</span>
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={handleLogout}
            className="cursor-pointer bg-cancel-gradient text-white hover:text-white focus:text-white"
          >
            <LogOut className="mr-2 h-4 w-4" />
            <span>Cerrar sesión</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <BalanzaDialog open={balanzaAbierta} onOpenChange={setBalanzaAbierta} />
    </div>
  );
}
