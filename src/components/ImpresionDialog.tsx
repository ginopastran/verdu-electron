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

const POR_DEFECTO = "__default__";

type ConfigImpresion = { anchoPapel: number; tieneLogo: boolean; impresora: string | null; recurso: string };

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
  const [impresoras, setImpresoras] = useState<string[]>([]);
  const [impresora, setImpresora] = useState(POR_DEFECTO);
  const [recurso, setRecurso] = useState("TP806L");
  const [guardandoImpresora, setGuardandoImpresora] = useState(false);

  useEffect(() => {
    if (!open || !ipc()) return;
    ipc().invoke("impresion-config-get").then((c: ConfigImpresion) => {
      setAnchoPapel(String(c.anchoPapel));
      setTieneLogo(c.tieneLogo);
      setImpresora(c.impresora ?? POR_DEFECTO);
      setRecurso(c.recurso);
    });
    ipc()
      .invoke("get-available-printers")
      .then((lista: { name: string }[]) => setImpresoras(lista.map((p) => p.name)))
      .catch(() => setImpresoras([]));
  }, [open]);

  const cambiarImpresora = async (valor: string) => {
    setGuardandoImpresora(true);
    try {
      const resultado = await ipc()?.invoke("impresora-elegir", valor === POR_DEFECTO ? null : valor);
      if (!resultado?.ok) throw new Error(resultado?.error ?? "No se pudo compartir la impresora");
      setImpresora(valor);
      setRecurso(resultado.recurso);
      toast.success("Impresora guardada");
    } catch (error) {
      toast.error((error as Error).message);
    } finally {
      setGuardandoImpresora(false);
    }
  };

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
          <DialogDescription>Impresora térmica, ancho del papel y prueba del encabezado.</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-1">
            <Label>Impresora</Label>
            <Select value={impresora} onValueChange={cambiarImpresora} disabled={guardandoImpresora}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={POR_DEFECTO}>Compartida como TP806L (por defecto)</SelectItem>
                {impresoras.map((nombre) => (
                  <SelectItem key={nombre} value={nombre}>
                    {nombre}
                  </SelectItem>
                ))}
                {impresora !== POR_DEFECTO && !impresoras.includes(impresora) && (
                  <SelectItem value={impresora}>{impresora} (no conectada)</SelectItem>
                )}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">Se imprime en el recurso compartido «{recurso}».</p>
          </div>
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
