import { useState, useEffect, useCallback } from "react";
import { toast } from "sonner";

// Declaración para autoUpdater
declare global {
  interface Window {
    autoUpdater: {
      checkForUpdates: () => Promise<{
        available: boolean;
        info?: any;
        error?: string;
      }>;
      downloadUpdate: () => Promise<{ success: boolean; error?: string }>;
      installUpdate: () => void;
    };
  }
}

interface UpdateInfo {
  version: string;
  releaseDate: string;
  releaseNotes?: string;
}

export const useAutoUpdater = () => {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [checking, setChecking] = useState(false);

  // Debug: Log cuando cambia el estado checking
  useEffect(() => {
    console.log("🔍 Estado checking cambiado a:", checking);
  }, [checking]);

  const checkForUpdates = useCallback(async () => {
    if (!window.autoUpdater) {
      console.log("Auto-updater no disponible (probablemente en desarrollo)");
      return;
    }

    try {
      setChecking(true);
      console.log("📞 Llamando a checkForUpdates...");

      // Timeout de 30 segundos para evitar que se cuelgue
      const timeoutPromise = new Promise<any>((_, reject) =>
        setTimeout(
          () => reject(new Error("Timeout verificando actualizaciones")),
          30000
        )
      );

      const updatePromise = window.autoUpdater.checkForUpdates();
      const result = await Promise.race([updatePromise, timeoutPromise]);

      console.log("📋 Resultado de checkForUpdates:", result);

      if (result.error) {
        console.error("Error al verificar actualizaciones:", result.error);
        if (result.error !== "Ya se está verificando actualizaciones") {
          toast.error("Error al verificar actualizaciones");
        }
        return;
      }

      if (result.available && result.info) {
        setUpdateAvailable(true);
        setUpdateInfo({
          version: result.info.version,
          releaseDate: result.info.releaseDate,
          releaseNotes: result.info.releaseNotes,
        });

        toast.success("¡Nueva versión disponible!", {
          description: `Versión ${result.info.version} está lista para descargar`,
          action: {
            label: "Descargar",
            onClick: downloadUpdate,
          },
        });
      } else {
        console.log("No hay actualizaciones disponibles");
        // Solo mostrar toast si no hay mensaje específico
        if (
          !result.info?.message ||
          result.info.message.includes("actualizaciones disponibles")
        ) {
          // No mostrar toast automático para evitar spam
          console.log("✅ Aplicación actualizada");
        }
      }
    } catch (error) {
      console.error("Error al verificar actualizaciones:", error);
      if (error instanceof Error && error.message.includes("Timeout")) {
        toast.error("Tiempo de espera agotado verificando actualizaciones");
      } else {
        toast.error("Error al verificar actualizaciones");
      }
    } finally {
      setChecking(false);
      console.log("🏁 Verificación de actualizaciones completada");
    }
  }, []);

  const downloadUpdate = useCallback(async () => {
    if (!window.autoUpdater || !updateAvailable) return;

    try {
      setDownloading(true);
      const result = await window.autoUpdater.downloadUpdate();

      if (result.error) {
        console.error("Error al descargar actualización:", result.error);
        toast.error(`Error al descargar la actualización: ${result.error}`);
        setDownloading(false);
        return;
      }

      if (result.success) {
        setDownloaded(true);
        setDownloading(false);

        toast.success("Actualización descargada", {
          description:
            "La actualización se instalará al reiniciar la aplicación",
          action: {
            label: "Reiniciar ahora",
            onClick: installUpdate,
          },
        });
      }
    } catch (error) {
      console.error("Error al descargar actualización:", error);
      toast.error("Error al descargar la actualización");
      setDownloading(false);
    }
  }, [updateAvailable]);

  const installUpdate = useCallback(() => {
    if (!window.autoUpdater || !downloaded) return;

    toast.info("Reiniciando aplicación...", {
      description: "La aplicación se cerrará y se aplicará la actualización",
    });

    // Esperar un poco para mostrar el toast
    setTimeout(() => {
      window.autoUpdater.installUpdate();
    }, 1000);
  }, [downloaded]);

  const cancelUpdate = useCallback(() => {
    setUpdateAvailable(false);
    setUpdateInfo(null);
    setDownloading(false);
    setDownloaded(false);
    toast.info("Actualización cancelada");
  }, []);

  // Safety: Limpiar estado checking si se queda colgado
  useEffect(() => {
    if (checking) {
      const safetyTimer = setTimeout(() => {
        console.log(
          "⚠️ SAFETY: Limpiando estado checking después de 45 segundos"
        );
        setChecking(false);
      }, 45000); // 45 segundos de seguridad

      return () => clearTimeout(safetyTimer);
    }
  }, [checking]);

  // Verificar actualizaciones automáticamente al cargar
  useEffect(() => {
    // Verificar después de 5 segundos de cargar la app, solo en producción
    if (window.autoUpdater) {
      const timer = setTimeout(() => {
        console.log(
          "🔍 Iniciando verificación automática de actualizaciones..."
        );
        checkForUpdates();
      }, 5000);

      return () => clearTimeout(timer);
    } else {
      console.log("⚠️ Auto-updater no disponible en desarrollo");
    }
  }, [checkForUpdates]);

  return {
    updateAvailable,
    updateInfo,
    downloading,
    downloaded,
    checking,
    checkForUpdates,
    downloadUpdate,
    installUpdate,
    cancelUpdate,
  };
};
