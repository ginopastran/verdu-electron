import { useAutoUpdater } from "@/hooks/useAutoUpdater";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Download, RefreshCw, CheckCircle, Loader2 } from "lucide-react";

export default function UpdateNotification() {
  const {
    updateAvailable,
    updateInfo,
    downloading,
    downloaded,
    checking,
    checkForUpdates,
    downloadUpdate,
    installUpdate,
    cancelUpdate,
  } = useAutoUpdater();

  // No mostrar nada si no hay actualizaciones disponibles
  if (!updateAvailable && !downloading && !downloaded) return null;

  return (
    <Card className="fixed bottom-4 right-4 w-80 z-50 shadow-lg border-emerald-200 bg-emerald-50/95 backdrop-blur-sm">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-emerald-800 flex items-center gap-2">
            {downloaded ? (
              <CheckCircle className="h-5 w-5" />
            ) : downloading ? (
              <Loader2 className="h-5 w-5 animate-spin" />
            ) : (
              <Download className="h-5 w-5" />
            )}
            {downloaded
              ? "Listo para instalar"
              : downloading
              ? "Descargando..."
              : "Actualización disponible"}
          </CardTitle>
          {updateInfo && (
            <Badge
              variant="secondary"
              className="bg-emerald-100 text-emerald-700"
            >
              v{updateInfo.version}
            </Badge>
          )}
        </div>
        <CardDescription className="text-emerald-600">
          {downloaded
            ? "La actualización se instalará al reiniciar"
            : downloading
            ? "Descargando la nueva versión..."
            : "Una nueva versión está disponible"}
        </CardDescription>
      </CardHeader>

      <CardContent>
        <div className="flex gap-2">
          {downloaded ? (
            <>
              <Button
                onClick={installUpdate}
                className="flex-1 bg-emerald-600 hover:bg-emerald-700"
              >
                <RefreshCw className="h-4 w-4 mr-2" />
                Reiniciar e instalar
              </Button>
            </>
          ) : downloading ? (
            <div className="flex-1 flex items-center justify-center py-2 text-emerald-600">
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
              Descargando...
            </div>
          ) : (
            <>
              <Button
                variant="outline"
                onClick={cancelUpdate}
                className="text-emerald-600 border-emerald-300 hover:bg-emerald-100"
              >
                Más tarde
              </Button>
              <Button
                onClick={downloadUpdate}
                className="flex-1 bg-emerald-600 hover:bg-emerald-700"
              >
                <Download className="h-4 w-4 mr-2" />
                Descargar
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
