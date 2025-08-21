import { Button } from "@/components/ui/button";

interface CartSummaryProps {
  total: number;
  onCancel: () => void;
  onCheckout: () => void;
  facturacionHabilitada?: boolean;
}

export function CartSummary({ total, onCancel, onCheckout, facturacionHabilitada = false }: CartSummaryProps) {
  return (
    <div className="sticky bottom-0">
      <div className="flex items-center justify-between">
        <div className="space-x-2">
          <Button
            variant="outline"
            className="text-white px-14 py-8 text-xl rounded-xl bg-cancel-gradient hover:text-white"
            onClick={onCancel}
          >
            CANCELAR
          </Button>
          <Button
            className="bg-emerald-gradient text-primary-foreground px-14 py-8 text-xl rounded-xl"
            onClick={onCheckout}
          >
            PAGAR
          </Button>
        </div>
        <div className="text-4xl font-semibold bg-white border px-8 py-6 shadow-sm rounded-2xl border-[#A7A7A7]">
          TOTAL: ${total.toLocaleString()}
        </div>
      </div>
      <div className="text-sm text-muted-foreground mt-2">
        F2 para pagar{facturacionHabilitada ? ", F6 para cuenta corriente" : ""}, F1 para cancelar, F4 para
        cerrar sesión
      </div>
    </div>
  );
}
