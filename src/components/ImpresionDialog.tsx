import { useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const ipc = () => (window as any).electron?.ipcRenderer;

export function ImpresionDialog({
  open,
  onOpenChange,
  businessName,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  businessName: string;
}) {
  const [anchoPapel, setAnchoPapel] = useState("80");
  const [tieneLogo, setTieneLogo] = useState(false);
  const [imprimiendo, setImprimiendo] = useState(false);

  useEffect(() => {
    if (!open || !ipc()) return;
    ipc().invoke("impresion-config-get").then((c: { anchoPapel: number; tieneLogo: boolean }) => {
      setAnchoPapel(String(c.anchoPapel));
      setTieneLogo(c.tieneLogo);
    });
  }, [open]);

  const cambiarAncho = async (valor: string) => {
    setAnchoPapel(valor);
    await ipc()?.invoke("impresion-config-set", { anchoPapel: Number(valor) });
  };

  const imprimirPrueba = async () => {
    setImprimiendo(true);
    try {
      const resultado = await (window as any).printer?.printTicket({
        id: 0,
        idReal: 0,
        businessName,
        fecha: new Date().toISOString(),
        createdAt: new Date().toISOString(),
        vendedor: "Prueba",
        metodoPago: "efectivo",
        items: [{ nombre: "IMPRESION DE PRUEBA", cantidad: 1, precio: 0, subtotal: 0 }],
        subtotal: 0,
        total: 0,
      });
      if (resultado?.success === false) throw new Error(resultado.error ?? "No se pudo imprimir");
      toast.success("Impresión de prueba enviada");
    } catch (error) {
      toast.error((error as Error).message || "No se pudo imprimir");
    } finally {
      setImprimiendo(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Impresión</DialogTitle>
          <DialogDescription>Ancho del papel de la impresora térmica y prueba del encabezado.</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-1">
            <Label>Ancho del papel</Label>
            <Select value={anchoPapel} onValueChange={cambiarAncho}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="80">80 mm</SelectItem>
                <SelectItem value="58">58 mm</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <p className="text-sm text-muted-foreground">
            {tieneLogo
              ? "Se usa el logo del negocio cargado en Administración → Configuración."
              : "Sin logo del negocio: se imprime el logo por defecto. Cargalo en Administración → Configuración."}
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cerrar
          </Button>
          <Button onClick={imprimirPrueba} disabled={imprimiendo}>
            {imprimiendo && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Impresión de prueba
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
