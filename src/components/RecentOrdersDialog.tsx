import { useState, useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
} from "@/components/ui/table";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
import { Receipt } from "lucide-react";
import { cn } from "@/lib/utils"; // Importar cn
import { getBusinessName } from "@/utils/businessHelpers";

interface RecentOrdersDialogProps {
  isOpen: boolean;
  onClose: () => void;
  API_URL: string;
  appId: string | null;
  formatFechaArgentina: (fecha: string | Date) => string;
  handleTicketPrinting: (orderData: any) => Promise<boolean>;
}

export function RecentOrdersDialog({
  isOpen,
  onClose,
  API_URL,
  appId,
  formatFechaArgentina,
  handleTicketPrinting,
}: RecentOrdersDialogProps) {
  const { user } = useAuth();
  const [recentOrders, setRecentOrders] = useState<any[]>([]);
  const [isLoadingOrders, setIsLoadingOrders] = useState(false);
  const [isPrinting, setIsPrinting] = useState(false);

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // Función para cargar órdenes recientes
  const loadRecentOrders = async () => {
    if (!user) {
      toast.error("Debes iniciar sesión para ver órdenes");
      return;
    }

    setIsLoadingOrders(true);

    try {
      const response = await fetch(
        `${API_URL}/api/ordenes/vendedor/${user.id}?limit=15`,
        { headers }
      );

      if (!response.ok) {
        throw new Error("Error al cargar órdenes recientes");
      }

      const data = await response.json();
      console.log("Órdenes recientes cargadas (raw):", data);

      const ordersWithVendor = data.map((order: any) => ({
        ...order,
        vendedor:
          typeof order.vendedor === "object"
            ? order.vendedor.nombre || user.nombre
            : order.vendedor || user.nombre,
        vendedorId: order.vendedorId || user.id,
        sucursalId: order.sucursalId || user.sucursalId,
        items: Array.isArray(order.detalles)
          ? order.detalles.map((detalle: any) => ({
              ...detalle,
              nombre:
                detalle.producto?.nombre ||
                detalle.nombre ||
                "Producto sin nombre",
              cantidad: Number(detalle.cantidad || 0),
              precioHistorico: Number(
                detalle.precioHistorico || detalle.precio || 0
              ),
              subtotal: Number(detalle.subtotal || 0),
            }))
          : [],
        total: Number(order.total || 0),
        metodoPago: order.metodoPago || "N/A",
        fecha: order.fecha || order.createdAt,
      }));

      console.log("Órdenes procesadas:", ordersWithVendor);
      setRecentOrders(ordersWithVendor);
    } catch (error) {
      console.error("Error al cargar órdenes:", error);
      toast.error("Error al cargar las órdenes recientes");
    } finally {
      setIsLoadingOrders(false);
    }
  };

  // Cargar órdenes al abrir el diálogo
  useEffect(() => {
    if (isOpen) {
      loadRecentOrders();
    }
  }, [isOpen]);

  // Función para reimprimir un ticket (llama a la función pasada por props)
  const handleReprintClick = async (order: any) => {
    if (isPrinting) return;
    setIsPrinting(true);
    try {
      // Añadir businessName si no lo tiene
      const enrichedOrder = {
        ...order,
        businessName: order.businessName || (await getBusinessName()),
      };
      await handleTicketPrinting(enrichedOrder);
    } catch (error) {
      console.error("Error al reimprimir desde diálogo:", error);
    } finally {
      setIsPrinting(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-4xl">
        <DialogHeader className="border-b border-emerald-100 pb-4">
          <DialogTitle className="text-2xl font-bold text-emerald-gradient">
            Órdenes recientes
          </DialogTitle>
          <DialogDescription className="text-lg">
            Últimas 15 órdenes realizadas por{" "}
            <span className="font-semibold text-emerald-700">
              {user?.nombre}
            </span>
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 max-h-[65vh] overflow-y-auto py-4 scrollbar-thin scrollbar-thumb-emerald-200 scrollbar-track-gray-100">
          {isLoadingOrders ? (
            <div className="flex justify-center items-center h-40">
              <div className="animate-spin rounded-full h-12 w-12 border-4 border-emerald-200 border-t-emerald-600"></div>
              <p className="ml-4 text-emerald-700 font-medium">
                Cargando órdenes...
              </p>
            </div>
          ) : recentOrders.length === 0 ? (
            <div className="text-center py-12">
              <div className="text-emerald-300 mb-4">
                <Receipt className="h-16 w-16 mx-auto" />
              </div>
              <p className="text-lg text-gray-500">
                No se encontraron órdenes recientes
              </p>
            </div>
          ) : (
            <div className="rounded-lg border border-emerald-100 overflow-hidden">
              <Table>
                <TableHeader className="bg-emerald-50">
                  <TableRow>
                    <TableHead className="font-semibold text-emerald-800">
                      Fecha
                    </TableHead>
                    <TableHead className="font-semibold text-emerald-800">
                      Método de pago
                    </TableHead>
                    <TableHead className="text-right font-semibold text-emerald-800">
                      Total
                    </TableHead>
                    <TableHead className="text-center font-semibold text-emerald-800">
                      Acciones
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {recentOrders.map((order, index) => (
                    <TableRow
                      key={order.id}
                      className={cn(
                        "hover:bg-emerald-25 transition-colors",
                        index % 2 === 0 ? "bg-white" : "bg-gray-50/50"
                      )}
                    >
                      <TableCell className="py-4">
                        <div className="font-medium text-gray-900">
                          {formatFechaArgentina(order.fecha)}
                        </div>
                      </TableCell>
                      <TableCell className="py-4">
                        <span
                          className={cn(
                            "capitalize px-3 py-1 rounded-full text-sm font-medium",
                            order.metodoPago === "efectivo"
                              ? "bg-green-100 text-green-800"
                              : order.metodoPago === "tarjeta"
                              ? "bg-blue-100 text-blue-800"
                              : order.metodoPago === "qr"
                              ? "bg-purple-100 text-purple-800"
                              : "bg-gray-100 text-gray-800"
                          )}
                        >
                          {order.metodoPago}
                        </span>
                      </TableCell>
                      <TableCell className="text-right font-semibold text-lg py-4">
                        <span className="text-emerald-700">
                          ${Number(order.total).toLocaleString()}
                        </span>
                      </TableCell>
                      <TableCell className="text-center py-4">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleReprintClick(order)}
                          disabled={isPrinting}
                          className="bg-emerald-50 hover:bg-emerald-100 text-emerald-700 hover:text-emerald-800 border border-emerald-200 transition-all duration-200"
                        >
                          {isPrinting ? (
                            <div className="animate-spin rounded-full h-4 w-4 border-2 border-emerald-200 border-t-emerald-600"></div>
                          ) : (
                            <Receipt className="h-4 w-4 mr-2" />
                          )}
                          {isPrinting ? "Reimprimiendo..." : "Reimprimir"}
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </div>

        <DialogFooter className="border-t border-emerald-100 pt-4">
          <Button
            variant="outline"
            onClick={onClose}
            className="border-emerald-200 text-emerald-700 hover:bg-emerald-50"
          >
            Cerrar
          </Button>
          <Button
            onClick={loadRecentOrders}
            disabled={isLoadingOrders}
            className="bg-emerald-gradient text-white hover:opacity-90 transition-opacity"
          >
            {isLoadingOrders ? (
              <div className="flex items-center gap-2">
                <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
                <span>Cargando...</span>
              </div>
            ) : (
              <>
                <Receipt className="h-4 w-4 mr-2" />
                Actualizar
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
