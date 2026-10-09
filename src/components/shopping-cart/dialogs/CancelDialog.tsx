import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface CancelDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  isLoading?: boolean;
}

export function CancelDialog({
  open,
  onOpenChange,
  onConfirm,
  isLoading = false,
}: CancelDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>¿Cancelar orden?</DialogTitle>
          <DialogDescription>
            ¿Estás seguro de que deseas cancelar la orden? Se eliminarán todos
            los productos del carrito.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="flex space-x-2 justify-end">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isLoading}
          >
            No, mantener productos
          </Button>
          <Button 
            className="bg-cancel-gradient" 
            onClick={onConfirm}
            disabled={isLoading}
          >
            {isLoading ? "Cancelando..." : "Sí, cancelar orden"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
