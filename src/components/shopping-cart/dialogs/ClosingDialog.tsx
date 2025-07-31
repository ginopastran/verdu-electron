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

interface ClosingDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onHandleClosing: (period: string) => void;
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
    // Validaciones
    if (!startDate || !endDate || !selectedVendedorId || !selectedSucursalId) {
      toast.error("Por favor, completa todos los campos.");
      return;
    }

    // Validar que la fecha de fin no sea anterior a la de inicio
    if (endDate < startDate) {
      toast.error(
        "La fecha de fin no puede ser anterior a la fecha de inicio."
      );
      return;
    }

    const [startHour, startMinute] = startTime.split(":").map(Number);
    const [endHour, endMinute] = endTime.split(":").map(Number);

    // Si es el mismo día, validar que la hora de fin sea posterior a la de inicio
    if (startDate.getTime() === endDate.getTime()) {
      if (
        startHour > endHour ||
        (startHour === endHour && startMinute >= endMinute)
      ) {
        toast.error("La hora de fin debe ser posterior a la hora de inicio.");
        return;
      }
    }

    // Crear fechas en hora local Argentina
    const fechaInicio = new Date(startDate);
    fechaInicio.setHours(startHour, startMinute, 0, 0);

    const fechaCierre = new Date(endDate);
    fechaCierre.setHours(endHour, endMinute, 59, 999);

    // Convertir a UTC (Argentina UTC-3)
    const fechaInicioUTC = new Date(fechaInicio.getTime() - 3 * 60 * 60 * 1000);
    const fechaCierreUTC = new Date(fechaCierre.getTime() - 3 * 60 * 60 * 1000);

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
        fechaInicio: fechaInicioUTC.toISOString(),
        fechaCierre: fechaCierreUTC.toISOString(),
        periodo: "custom",
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

      toast.success("Cierre creado correctamente");
      setCreateOpen(false);
      // Resetear formulario
      setStartDate(new Date());
      setEndDate(new Date());
      setStartTime("00:00");
      setEndTime("23:59");
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
          <div className="grid grid-cols-2 gap-4">
            <Button
              onClick={() => onHandleClosing("mañana")}
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
            </Button>
            <Button
              onClick={() => onHandleClosing("tarde")}
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
            </Button>
            <Button
              onClick={() => onHandleClosing("todo")}
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
