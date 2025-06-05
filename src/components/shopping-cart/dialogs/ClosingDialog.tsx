import { RefObject } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Sun, Moon, Calendar } from "lucide-react";

interface ClosingDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onHandleClosing: (period: string) => void;
  isClosing: boolean;
  searchInputRef: RefObject<HTMLInputElement>;
}

export const ClosingDialog = ({
  open,
  onOpenChange,
  onHandleClosing,
  isClosing,
  searchInputRef,
}: ClosingDialogProps) => {
  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        if (!open) {
          onOpenChange(false);
          setTimeout(() => {
            searchInputRef.current?.focus();
          }, 100);
        } else {
          onOpenChange(open);
        }
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Seleccionar período de cierre</DialogTitle>
          <DialogDescription>
            Presiona el número correspondiente al período o haz clic en el botón
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-3 gap-4">
          <Button
            onClick={() => onHandleClosing("mañana")}
            className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
            variant="outline"
            disabled={isClosing}
          >
            {isClosing ? (
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
            ) : (
              <Sun />
            )}
            <span className="text-base">Mañana (1)</span>
          </Button>
          <Button
            onClick={() => onHandleClosing("tarde")}
            className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
            variant="outline"
            disabled={isClosing}
          >
            {isClosing ? (
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
            ) : (
              <Moon />
            )}
            <span className="text-base">Tarde (2)</span>
          </Button>
          <Button
            onClick={() => onHandleClosing("todo")}
            className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
            variant="outline"
            disabled={isClosing}
          >
            {isClosing ? (
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
            ) : (
              <Calendar />
            )}
            <span className="text-base">Todo el día (3)</span>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};
