import { useState } from "react";
import { cn } from "@/lib/utils";
import { textoOferta } from "@/utils/reglasVenta";

const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

export function FotoProducto({
  id,
  fotoActualizadaEn,
  className,
}: {
  id: number;
  fotoActualizadaEn?: string | null;
  className?: string;
}) {
  const [fallo, setFallo] = useState<string | null>(null);
  if (!fotoActualizadaEn || fallo === fotoActualizadaEn) return null;
  return (
    <img
      src={`${API_URL}/api/productos/${id}/foto?v=${encodeURIComponent(fotoActualizadaEn)}`}
      alt=""
      loading="lazy"
      draggable={false}
      onError={() => setFallo(fotoActualizadaEn)}
      className={cn("shrink-0 rounded-md object-cover bg-white", className)}
    />
  );
}

export function EtiquetaOferta({ porcentaje }: { porcentaje: number }) {
  return (
    <span className="bg-amber-100 text-amber-800 text-xs font-medium px-2 py-1 rounded shrink-0">
      {textoOferta(porcentaje)}
    </span>
  );
}

export function PrecioTachado({ precio }: { precio: number }) {
  return (
    <span className="line-through text-muted-foreground font-normal">
      ${precio.toLocaleString()}
    </span>
  );
}
