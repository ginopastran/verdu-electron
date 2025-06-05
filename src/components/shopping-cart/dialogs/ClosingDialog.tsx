import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Sun, Moon, Calendar } from "lucide-react";

interface ClosingDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onPeriodSelect: (period: string) => void;
  isLoading?: boolean;
}

export function ClosingDialog({
  open,
  onOpenChange,
  onPeriodSelect,
  isLoading = false,
}: ClosingDialogProps) {
  const periods = [
    { id: "mañana", label: "Mañana", icon: Sun, key: "1" },
    { id: "tarde", label: "Tarde", icon: Moon, key: "2" },
    { id: "todo", label: "Todo el día", icon: Calendar, key: "3" },
  ];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Seleccionar período de cierre</DialogTitle>
          <DialogDescription>
            Presiona el número correspondiente al período o haz clic en el
            botón
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-3 gap-4">
          {periods.map((period) => (
            <Button
              key={period.id}
              onClick={() => onPeriodSelect(period.id)}
              className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
              variant="outline"
              disabled={isLoading}
            >
              {isLoading ? (
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
              ) : (
                <period.icon />
              )}
              <span className="text-base">
                {period.label} ({period.key})
              </span>
            </Button>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
