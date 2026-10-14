import { useEffect, useRef, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { EstadoCredito } from "@/utils/reglasVenta";

const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

async function validarAdmin(
  email: string,
  password: string,
  businessId: number | null
) {
  const res = await fetch(`${API_URL}/api/login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-App-ID": import.meta.env.VITE_APP_ID,
    },
    body: JSON.stringify({ email, password }),
    credentials: "omit",
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || "No se pudo validar la contraseña");
  if (data.user?.rol?.nombre !== "ADMIN") {
    throw new Error("El usuario no es administrador");
  }
  if (businessId != null && data.user?.businessId !== businessId) {
    throw new Error("El administrador no pertenece a este negocio");
  }
}

const dinero = (n: number) =>
  new Intl.NumberFormat("es-AR", { style: "currency", currency: "ARS" }).format(n);

interface AutorizacionCreditoDialogProps {
  credito: EstadoCredito | null;
  emailInicial: string;
  businessId: number | null;
  onCancelar: () => void;
  onAutorizado: (emailAdmin: string) => void;
}

export function AutorizacionCreditoDialog({
  credito,
  emailInicial,
  businessId,
  onCancelar,
  onAutorizado,
}: AutorizacionCreditoDialogProps) {
  const [email, setEmail] = useState(emailInicial);
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [validando, setValidando] = useState(false);
  const passwordRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (credito) {
      setEmail(emailInicial);
      setPassword("");
      setError("");
    }
  }, [credito, emailInicial]);

  const autorizar = async () => {
    if (validando) return;
    if (!email.trim() || !password) {
      setError("Ingresá el email y la contraseña del administrador");
      return;
    }
    setValidando(true);
    setError("");
    try {
      await validarAdmin(email.trim(), password, businessId);
      onAutorizado(email.trim());
    } catch (e) {
      setError(e instanceof Error ? e.message : "No se pudo validar la contraseña");
    } finally {
      setValidando(false);
    }
  };

  return (
    <Dialog open={!!credito} onOpenChange={(open) => !open && onCancelar()}>
      <DialogContent
        className="sm:max-w-md"
        onOpenAutoFocus={(e) => {
          e.preventDefault();
          passwordRef.current?.focus();
        }}
      >
        <DialogHeader>
          <DialogTitle className="text-emerald-gradient font-bold text-2xl">
            Límite de crédito superado
          </DialogTitle>
          <DialogDescription>
            Para vender en cuenta corriente a este cliente hace falta la
            contraseña de un administrador del negocio.
          </DialogDescription>
        </DialogHeader>

        {credito && (
          <div className="space-y-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm">
            <div className="flex justify-between">
              <span>Saldo actual</span>
              <span>{dinero(credito.saldo)}</span>
            </div>
            <div className="flex justify-between">
              <span>Saldo con esta venta</span>
              <span>{dinero(credito.nuevoSaldo)}</span>
            </div>
            <div className="flex justify-between">
              <span>Límite</span>
              <span>{dinero(credito.limite ?? 0)}</span>
            </div>
            <div className="flex justify-between font-semibold text-red-700">
              <span>Excedente</span>
              <span>{dinero(credito.excedente)}</span>
            </div>
          </div>
        )}

        <div className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="admin-email">Email del administrador</Label>
            <Input
              id="admin-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={validando}
              autoComplete="off"
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="admin-password">Contraseña</Label>
            <Input
              id="admin-password"
              ref={passwordRef}
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  autorizar();
                }
              }}
              disabled={validando}
              autoComplete="off"
            />
          </div>
          {error && <p className="text-sm text-red-600">{error}</p>}
        </div>

        <DialogFooter className="sm:justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={onCancelar}
            disabled={validando}
          >
            Cancelar
          </Button>
          <Button
            type="button"
            onClick={autorizar}
            disabled={validando}
            className="bg-emerald-gradient text-white hover:text-white"
          >
            {validando ? "Validando..." : "Autorizar venta"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
