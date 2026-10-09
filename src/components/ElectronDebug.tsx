import { useEffect, useState } from "react";

export const ElectronDebug = () => {
  const [apis, setApis] = useState<any>({});

  useEffect(() => {
    const checkAPIs = () => {
      const apiStatus = {
        window: typeof window !== "undefined",
        electron: typeof window.electron,
        printer: typeof window.printer,
        electronStore: typeof (window as any).electronStore,
        autoUpdater: typeof (window as any).autoUpdater,
        pesoReader: typeof (window as any).pesoReader,
        // Verificar si las funciones específicas existen
        electronIpcRenderer: !!(window as any).electron?.ipcRenderer,
        printerPrintTicket: !!(window as any).printer?.printTicket,
        storeGet: !!(window as any).electronStore?.get,
      };

      console.log("🔧 API Status Check:", apiStatus);
      setApis(apiStatus);
    };

    // Verificar inmediatamente
    checkAPIs();

    // Verificar cada 2 segundos por si se cargan después
    const interval = setInterval(checkAPIs, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="fixed bottom-4 right-4 bg-gray-900 text-white p-4 rounded-lg max-w-sm z-50">
      <h3 className="font-bold mb-2">🔧 Debug Electron APIs</h3>
      <div className="text-xs space-y-1">
        <div>Window: {apis.window ? "✅" : "❌"}</div>
        <div>Electron: {apis.electron !== "undefined" ? "✅" : "❌"}</div>
        <div>Printer: {apis.printer !== "undefined" ? "✅" : "❌"}</div>
        <div>Store: {apis.electronStore !== "undefined" ? "✅" : "❌"}</div>
        <div>IPC: {apis.electronIpcRenderer ? "✅" : "❌"}</div>
        <div>Print Fn: {apis.printerPrintTicket ? "✅" : "❌"}</div>
      </div>
    </div>
  );
};
