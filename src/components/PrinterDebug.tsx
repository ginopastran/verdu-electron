import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Printer, RefreshCw } from "lucide-react";

// Declaración de tipos para window
declare global {
  interface Window {
    electron?: {
      ipcRenderer: {
        invoke: (channel: string, ...args: any[]) => Promise<any>;
      };
    };
  }
}

interface PrinterInfo {
  name: string;
  displayName: string;
  description: string;
  status: number;
  isDefault: boolean;
  statusText: string;
}

export const PrinterDebug = () => {
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const refreshPrinters = async () => {
    setLoading(true);
    try {
      if (typeof window !== "undefined" && window.electron?.ipcRenderer) {
        const printerList = await window.electron.ipcRenderer.invoke(
          "get-available-printers"
        );
        setPrinters(printerList);
        console.log("🖨️ Impresoras encontradas:", printerList);
      } else {
        console.error("❌ API de Electron no disponible");
      }
    } catch (error) {
      console.error("❌ Error al obtener impresoras:", error);
    } finally {
      setLoading(false);
    }
  };

  const getStatusBadge = (status: number, statusText: string) => {
    const variant =
      status === 0
        ? "default"
        : status === 1
        ? "secondary"
        : status === 2
        ? "destructive"
        : "outline";

    return <Badge variant={variant}>{statusText}</Badge>;
  };

  return (
    <Card className="w-full max-w-2xl mx-auto">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Printer className="w-5 h-5" />
          Debug de Impresoras
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={refreshPrinters} disabled={loading} className="w-full">
          {loading ? (
            <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
          ) : (
            <RefreshCw className="w-4 h-4 mr-2" />
          )}
          {loading ? "Actualizando..." : "Actualizar Lista de Impresoras"}
        </Button>

        {printers.length === 0 && !loading && (
          <div className="text-center text-muted-foreground py-8">
            Haz clic en "Actualizar" para ver las impresoras disponibles
          </div>
        )}

        {printers.length > 0 && (
          <div className="space-y-3">
            <h3 className="font-semibold">
              Impresoras encontradas ({printers.length})
            </h3>
            {printers.map((printer, index) => (
              <Card key={index} className="border">
                <CardContent className="p-4">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="font-medium">{printer.name}</h4>
                    {getStatusBadge(printer.status, printer.statusText)}
                  </div>

                  {printer.displayName &&
                    printer.displayName !== printer.name && (
                      <p className="text-sm text-muted-foreground mb-1">
                        <strong>Nombre para mostrar:</strong>{" "}
                        {printer.displayName}
                      </p>
                    )}

                  {printer.description && (
                    <p className="text-sm text-muted-foreground mb-1">
                      <strong>Descripción:</strong> {printer.description}
                    </p>
                  )}

                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-muted-foreground">
                      <strong>Estado:</strong> {printer.status}
                    </span>
                    {printer.isDefault && (
                      <Badge variant="outline">Predeterminada</Badge>
                    )}
                    {printer.name === "TP806L" && (
                      <Badge variant="default">Target</Badge>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
};
