import { RefObject, useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import {
  Sun,
  Moon,
  Calendar as CalendarIcon,
  User,
  Store,
  Clock,
} from "lucide-react";
import { format } from "date-fns";
import { es } from "date-fns/locale";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
import { Input } from "@/components/ui/input";
import { avisarArqueo } from "@/hooks/useClosing";
import {
  consultaDeEsperado,
  datosDeCierre,
  leerMonto,
  rangoManual,
  textoArqueo,
} from "@/utils/reglasVenta";

const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

function useEsperado(consulta: string | null) {
  const [esperado, setEsperado] = useState<number | null>(null);

  useEffect(() => {
    setEsperado(null);
    if (!consulta) return;
    let vigente = true;
    const appId = import.meta.env.VITE_APP_ID || null;
    fetch(`${API_URL}/api/cierres/esperado?${consulta}`, {
      headers: {
        "Content-Type": "application/json",
        ...(appId && { "X-App-ID": appId }),
      },
      credentials: "include",
    })
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => {
        if (vigente && typeof d?.esperado === "number") setEsperado(d.esperado);
      })
      .catch(() => {});
    return () => {
      vigente = false;
    };
  }, [consulta]);

  return esperado;
}

function ArqueoEnVivo({
  esperado,
  contado,
}: {
  esperado: number | null;
  contado: number | null | "invalido";
}) {
  if (esperado == null) return null;
  const arqueo = typeof contado === "number" ? textoArqueo(contado, esperado) : null;
  return (
    <span className="flex flex-col items-center text-xs font-normal leading-tight">
      <span className="text-muted-foreground">
        Esperado ${esperado.toLocaleString("es-AR")}
      </span>
      {arqueo && (
        <span
          className={
            arqueo.estado === "cuadra"
              ? "text-emerald-600 font-medium"
              : "text-red-600 font-medium"
          }
        >
          {arqueo.texto}
        </span>
      )}
    </span>
  );
}

interface ClosingDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onHandleClosing: (period: string, efectivoContado: number | null) => void;
  isClosing: boolean;
  searchInputRef: RefObject<HTMLInputElement | null>;
}

export const ClosingDialog = ({
  open,
  onOpenChange,
  onHandleClosing,
  isClosing,
  searchInputRef,
}: ClosingDialogProps) => {
  const { user } = useAuth();

  // Estados para el cierre manual
  const [createOpen, setCreateOpen] = useState(false);
  const [startDate, setStartDate] = useState<Date | undefined>(new Date());
  const [endDate, setEndDate] = useState<Date | undefined>(new Date());
  const [startTime, setStartTime] = useState("00:00");
  const [endTime, setEndTime] = useState("23:59");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [users, setUsers] = useState<any[]>([]);
  const [sucursales, setSucursales] = useState<any[]>([]);
  const [contadoTexto, setContadoTexto] = useState("");
  const contado = leerMonto(contadoTexto);

  const [abiertoEn, setAbiertoEn] = useState<Date | null>(null);

  useEffect(() => {
    if (!open) setContadoTexto("");
    setAbiertoEn(open ? new Date() : null);
  }, [open]);

  const consultaPeriodo = (periodo: string) =>
    abiertoEn && user
      ? consultaDeEsperado(
          datosDeCierre(user.id, user.sucursalId, periodo, abiertoEn)
        )
      : null;
  const esperadoManana = useEsperado(consultaPeriodo("mañana"));
  const esperadoTarde = useEsperado(consultaPeriodo("tarde"));
  const esperadoTodo = useEsperado(consultaPeriodo("todo"));
  const rangoEnVivo = createOpen
    ? rangoManual(startDate, endDate, startTime, endTime)
    : null;
  const esperadoManual = useEsperado(
    rangoEnVivo && typeof rangoEnVivo !== "string" && user
      ? consultaDeEsperado({
          vendedorId: user.id,
          sucursalId: user.sucursalId,
          periodo: "custom",
          ...rangoEnVivo,
        })
      : null
  );

  const cerrarPeriodo = (period: string) => {
    if (contado === "invalido") {
      toast.error("El efectivo contado tiene que ser un monto válido");
      return;
    }
    onHandleClosing(period, contado);
  };

  const campoContado = (id: string) => (
    <div className="space-y-2">
      <label htmlFor={id} className="text-sm font-medium">
        Efectivo contado en caja (opcional)
      </label>
      <Input
        id={id}
        type="number"
        min="0"
        step="0.01"
        inputMode="decimal"
        placeholder="$ 0"
        value={contadoTexto}
        onChange={(e) => setContadoTexto(e.target.value)}
      />
      {contado === "invalido" ? (
        <p className="text-sm text-red-600">Ingresá un monto válido</p>
      ) : (
        <p className="text-xs text-muted-foreground">
          Se compara con el efectivo esperado de cada período.
        </p>
      )}
    </div>
  );

  // Obtener vendedor y sucursal del usuario logueado automáticamente
  const selectedVendedorId = user?.id?.toString() || "";
  const selectedSucursalId = user?.sucursalId?.toString() || "";

  // Cargar datos cuando se abre el modal de creación
  useEffect(() => {
    const fetchData = async () => {
      try {
        const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
        const appId = import.meta.env.VITE_APP_ID || null;

        const headers: HeadersInit = {
          "Content-Type": "application/json",
        };

        if (appId) {
          (headers as Record<string, string>)["X-App-ID"] = appId;
        }

        const [usersRes, sucursalesRes] = await Promise.all([
          fetch(`${API_URL}/api/usuarios`, { headers }),
          fetch(`${API_URL}/api/sucursales`, { headers }),
        ]);

        if (usersRes.ok) {
          const usersData = await usersRes.json();
          const vendedores = usersData.filter(
            (user: any) => user.rol?.nombre === "VENDEDOR"
          );
          setUsers(vendedores);
        }

        if (sucursalesRes.ok) {
          const sucursalesData = await sucursalesRes.json();
          setSucursales(sucursalesData);
        }
      } catch (error) {
        toast.error("Error al cargar datos para el formulario.");
      }
    };

    if (createOpen) {
      fetchData();
    }
  }, [createOpen]);

  // Función para crear cierre manual
  const handleCreateCierre = async () => {
    if (contado === "invalido") {
      toast.error("El efectivo contado tiene que ser un monto válido");
      return;
    }

    const rango = rangoManual(startDate, endDate, startTime, endTime);
    if (typeof rango === "string" || !selectedVendedorId || !selectedSucursalId) {
      toast.error(
        typeof rango === "string" ? rango : "Por favor, completa todos los campos."
      );
      return;
    }

    setIsSubmitting(true);
    try {
      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      const appId = import.meta.env.VITE_APP_ID || null;

      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };

      if (appId) {
        (headers as Record<string, string>)["X-App-ID"] = appId;
      }

      const body = {
        vendedorId: Number(selectedVendedorId),
        sucursalId: Number(selectedSucursalId),
        ...rango,
        periodo: "custom",
        ...(contado != null && { efectivoContado: contado }),
      };

      const res = await fetch(`${API_URL}/api/cierres`, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
      });

      if (!res.ok) {
        const errorData = await res.json();
        throw new Error(errorData.message || "Error al crear el cierre");
      }

      const cierreData = await res.json();
      console.log("✅ Datos de cierre recibidos:", cierreData);
      avisarArqueo(cierreData.ventasPorMetodo, contado);

      // Intentar imprimir el ticket de cierre
      try {
        if (typeof window !== "undefined" && window.printer?.printClosing) {
          console.log(
            "📄 Enviando datos para impresión de cierre manual:",
            cierreData
          );
          const result = await window.printer.printClosing(cierreData);
          console.log("📄 Resultado de impresión de cierre manual:", result);

          if (result.success && !result.printerError) {
            toast.success("Ticket de cierre impreso correctamente");
          } else if (result.printerError) {
            // Error específico de la impresora TP806L - mostrar toast de error pero no fallar
            console.error(
              "❌ Error de impresora TP806L:",
              result.printerError
            );
            toast.error(`Error de impresión: ${result.printerError}`, {
              description:
                "El cierre se completó correctamente pero no se pudo imprimir el ticket",
            });
          } else {
            // Error general - mostrar toast de error
            console.error(
              "❌ Error general al imprimir cierre:",
              result.message
            );
            toast.error(
              `Error al imprimir el ticket de cierre: ${
                result.message || "Desconocido"
              }`
            );
          }
        } else {
          throw new Error("API de Electron no disponible");
        }
      } catch (electronError: any) {
        // Si no se puede acceder a Electron, mostrar error específico
        console.error(
          "❌ Error al acceder a Electron para cierre:",
          electronError
        );
        toast.error("Error de conexión con la impresora", {
          description:
            "No se pudo conectar con el sistema de impresión para el cierre",
        });
      }

      toast.success("Cierre creado correctamente");
      setCreateOpen(false);
      // Resetear formulario
      setStartDate(new Date());
      setEndDate(new Date());
      setStartTime("00:00");
      setEndTime("23:59");
      setContadoTexto("");
    } catch (err) {
      toast.error((err as Error).message);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Obtener información del vendedor y sucursal para mostrar
  const vendedorInfo = users.find(
    (u) => u.id.toString() === selectedVendedorId
  );
  const sucursalInfo = sucursales.find(
    (s) => s.id.toString() === selectedSucursalId
  );

  return (
    <>
      <Dialog
        open={open}
        onOpenChange={(open) => {
          if (!open) {
            onOpenChange(false);
            setTimeout(() => {
              searchInputRef.current?.focus();
            }, 100);
          } else {
            onOpenChange(open);
          }
        }}
      >
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Seleccionar período de cierre</DialogTitle>
            <DialogDescription>
              Presiona el número correspondiente al período o haz clic en el
              botón
            </DialogDescription>
          </DialogHeader>
          {campoContado("efectivo-contado")}
          <div className="grid grid-cols-2 gap-4">
            <Button
              onClick={() => cerrarPeriodo("mañana")}
              className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
              variant="outline"
              disabled={isClosing}
            >
              {isClosing ? (
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
              ) : (
                <Sun />
              )}
              <span className="text-base">Mañana (1)</span>
              <ArqueoEnVivo esperado={esperadoManana} contado={contado} />
            </Button>
            <Button
              onClick={() => cerrarPeriodo("tarde")}
              className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
              variant="outline"
              disabled={isClosing}
            >
              {isClosing ? (
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
              ) : (
                <Moon />
              )}
              <span className="text-base">Tarde (2)</span>
              <ArqueoEnVivo esperado={esperadoTarde} contado={contado} />
            </Button>
            <Button
              onClick={() => cerrarPeriodo("todo")}
              className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
              variant="outline"
              disabled={isClosing}
            >
              {isClosing ? (
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
              ) : (
                <CalendarIcon />
              )}
              <span className="text-base">Todo el día (3)</span>
              <ArqueoEnVivo esperado={esperadoTodo} contado={contado} />
            </Button>
            <Button
              onClick={() => setCreateOpen(true)}
              className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8 bg-emerald-gradient text-white hover:text-white"
              variant="outline"
              disabled={isClosing}
            >
              <Clock />
              <span className="text-base">Cierre Manual (4)</span>
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Modal de Cierre Manual */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-emerald-gradient text-xl">
              Crear Cierre Manual
            </DialogTitle>
            <DialogDescription>
              Selecciona las fechas y horarios para generar el cierre de caja
              manual. Puedes seleccionar fechas diferentes para cierres que
              abarquen múltiples días.
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {/* Vendedor y Sucursal - Solo lectura */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Vendedor</label>
                <div className="flex items-center gap-2 p-3 border border-gray-200 rounded-md bg-gray-50">
                  <User className="h-4 w-4 text-gray-500" />
                  <span className="text-sm text-gray-700">
                    {vendedorInfo?.nombre || user?.nombre || "Cargando..."}
                  </span>
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Sucursal</label>
                <div className="flex items-center gap-2 p-3 border border-gray-200 rounded-md bg-gray-50">
                  <Store className="h-4 w-4 text-gray-500" />
                  <span className="text-sm text-gray-700">
                    {sucursalInfo?.nombre || "Cargando..."}
                  </span>
                </div>
              </div>
            </div>

            {/* Fechas de inicio y fin */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Fecha de inicio</label>
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      className="w-full justify-start text-left font-normal"
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {startDate ? (
                        format(startDate, "PPP", { locale: es })
                      ) : (
                        <span>Elige fecha inicio</span>
                      )}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0" sideOffset={5}>
                    <Calendar
                      mode="single"
                      selected={startDate}
                      onSelect={setStartDate}
                      disabled={(date) => date > new Date()}
                      initialFocus
                      locale={es}
                    />
                  </PopoverContent>
                </Popover>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Fecha de fin</label>
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      className="w-full justify-start text-left font-normal"
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {endDate ? (
                        format(endDate, "PPP", { locale: es })
                      ) : (
                        <span>Elige fecha fin</span>
                      )}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0" sideOffset={5}>
                    <Calendar
                      mode="single"
                      selected={endDate}
                      onSelect={setEndDate}
                      disabled={(date) => date > new Date()}
                      initialFocus
                      locale={es}
                    />
                  </PopoverContent>
                </Popover>
              </div>
            </div>

            {/* Horas de inicio y fin */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Hora de inicio</label>
                <div className="flex items-center gap-2">
                  <Select
                    value={startTime.split(":")[0]}
                    onValueChange={(h) =>
                      setStartTime(`${h}:${startTime.split(":")[1]}`)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {Array.from({ length: 24 }, (_, i) => (
                        <SelectItem
                          key={i}
                          value={i.toString().padStart(2, "0")}
                        >
                          {i.toString().padStart(2, "0")}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <span className="text-muted-foreground">:</span>
                  <Select
                    value={startTime.split(":")[1]}
                    onValueChange={(m) =>
                      setStartTime(`${startTime.split(":")[0]}:${m}`)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {Array.from({ length: 60 }, (_, i) => (
                        <SelectItem
                          key={i}
                          value={i.toString().padStart(2, "0")}
                        >
                          {i.toString().padStart(2, "0")}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Hora de fin</label>
                <div className="flex items-center gap-2">
                  <Select
                    value={endTime.split(":")[0]}
                    onValueChange={(h) =>
                      setEndTime(`${h}:${endTime.split(":")[1]}`)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {Array.from({ length: 24 }, (_, i) => (
                        <SelectItem
                          key={i}
                          value={i.toString().padStart(2, "0")}
                        >
                          {i.toString().padStart(2, "0")}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <span className="text-muted-foreground">:</span>
                  <Select
                    value={endTime.split(":")[1]}
                    onValueChange={(m) =>
                      setEndTime(`${endTime.split(":")[0]}:${m}`)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {Array.from({ length: 60 }, (_, i) => (
                        <SelectItem
                          key={i}
                          value={i.toString().padStart(2, "0")}
                        >
                          {i.toString().padStart(2, "0")}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>

            {campoContado("efectivo-contado-manual")}
            <ArqueoEnVivo esperado={esperadoManual} contado={contado} />

            {/* Información del período seleccionado */}
            {startDate && endDate && (
              <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  <Clock className="h-4 w-4 text-blue-600" />
                  <span className="text-sm font-medium text-blue-800">
                    Período seleccionado:
                  </span>
                </div>
                <div className="text-sm text-blue-700 space-y-1">
                  <div>
                    <strong>Inicio:</strong>{" "}
                    {format(startDate, "PPP", { locale: es })} a las {startTime}
                  </div>
                  <div>
                    <strong>Fin:</strong>{" "}
                    {format(endDate, "PPP", { locale: es })} a las {endTime}
                  </div>
                  {startDate.getTime() !== endDate.getTime() && (
                    <div className="text-xs text-blue-600 mt-1">
                      ⚠️ Cierre de múltiples días
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>
              Cancelar
            </Button>
            <Button
              onClick={handleCreateCierre}
              disabled={isSubmitting}
              className="bg-emerald-gradient hover:text-white text-white"
            >
              {isSubmitting ? "Creando..." : "Crear Cierre"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};
