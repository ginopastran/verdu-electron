import { useEffect, useState } from "react";
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
import { useScaleWeight } from "@/hooks/useScaleWeight";

type ConfigBalanza = {
  modo: "archivo" | "serie";
  puerto?: string;
  baudRate: number;
  consulta: "enq" | "continuo";
  unidadEntera: "gramos" | "kilos";
};

type Estado = { conectada: boolean; error?: string };

const ipc = () => (window as any).electron?.ipcRenderer;

export function BalanzaDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [config, setConfig] = useState<ConfigBalanza | null>(null);
  const [estado, setEstado] = useState<Estado>({ conectada: false });
  const [puertos, setPuertos] = useState<{ path: string; manufacturer?: string }[]>([]);
  const peso = useScaleWeight();

  useEffect(() => {
    if (!open || !ipc()) return;
    ipc().invoke("balanza-config-get").then((r: { config: ConfigBalanza; estado: Estado }) => {
      setConfig(r.config);
      setEstado(r.estado);
    });
    ipc().invoke("balanza-puertos").then(setPuertos);
    const alCambiar = (nuevo: Estado) => setEstado(nuevo);
    ipc().on("balanza-estado", alCambiar);
    return () => ipc().removeListener("balanza-estado", alCambiar);
  }, [open]);

  const guardar = async () => {
    if (!config) return;
    await ipc().invoke("balanza-config-set", config);
    toast.success("Configuración de balanza guardada");
  };

  const cambiar = (cambios: Partial<ConfigBalanza>) =>
    setConfig((actual) => (actual ? { ...actual, ...cambios } : actual));

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Balanza</DialogTitle>
          <DialogDescription>
            Origen del peso para los productos por kilo.
          </DialogDescription>
        </DialogHeader>

        {!ipc() && <p className="text-sm">Solo disponible en la app de escritorio.</p>}

        {config && (
          <div className="space-y-4">
            <div className="space-y-1">
              <Label>Origen</Label>
              <Select value={config.modo} onValueChange={(modo) => cambiar({ modo: modo as ConfigBalanza["modo"] })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="archivo">Archivo C:\Peso\peso.json</SelectItem>
                  <SelectItem value="serie">Puerto serie (Croma / RS-232)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {config.modo === "serie" && (
              <>
                <div className="space-y-1">
                  <Label>Puerto</Label>
                  <Select value={config.puerto ?? ""} onValueChange={(puerto) => cambiar({ puerto })}>
                    <SelectTrigger>
                      <SelectValue placeholder={puertos.length ? "Elegí un puerto" : "No se encontraron puertos"} />
                    </SelectTrigger>
                    <SelectContent>
                      {puertos.map((p) => (
                        <SelectItem key={p.path} value={p.path}>
                          {p.path}
                          {p.manufacturer ? ` · ${p.manufacturer}` : ""}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid grid-cols-3 gap-2">
                  <div className="space-y-1">
                    <Label>Velocidad</Label>
                    <Select
                      value={String(config.baudRate)}
                      onValueChange={(v) => cambiar({ baudRate: Number(v) })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {[2400, 4800, 9600, 19200].map((b) => (
                          <SelectItem key={b} value={String(b)}>
                            {b}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1">
                    <Label>Lectura</Label>
                    <Select
                      value={config.consulta}
                      onValueChange={(v) => cambiar({ consulta: v as ConfigBalanza["consulta"] })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="enq">Por consulta</SelectItem>
                        <SelectItem value="continuo">Continua</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-1">
                    <Label>Sin decimales</Label>
                    <Select
                      value={config.unidadEntera}
                      onValueChange={(v) => cambiar({ unidadEntera: v as ConfigBalanza["unidadEntera"] })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="gramos">Gramos</SelectItem>
                        <SelectItem value="kilos">Kilos</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                <p className="text-sm">
                  Estado:{" "}
                  <span className={estado.conectada ? "text-emerald-600" : "text-red-600"}>
                    {estado.conectada ? "conectada" : "desconectada"}
                  </span>
                  {estado.error ? ` (${estado.error})` : ""}
                </p>
              </>
            )}

            <div className="rounded-lg border p-4 text-center">
              <p className="text-xs text-muted-foreground">Peso actual</p>
              <p className="text-3xl font-bold">{(peso / 1000).toFixed(3)} kg</p>
            </div>
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cerrar
          </Button>
          <Button onClick={guardar} disabled={!config}>
            Guardar
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
