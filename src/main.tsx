import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";

// Debug: Verificar APIs de Electron al iniciar la aplicación
console.log("🚀 Aplicación iniciando - Verificando APIs de Electron:");
console.log("- window.electron:", typeof (window as any).electron);
console.log("- window.printer:", typeof (window as any).printer);
console.log("- window.electronStore:", typeof (window as any).electronStore);
console.log("- window.autoUpdater:", typeof (window as any).autoUpdater);
console.log("- window.pesoReader:", typeof (window as any).pesoReader);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
