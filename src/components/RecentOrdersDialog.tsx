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
import { Receipt, ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { getBusinessName } from "@/utils/businessHelpers";
import { useTicketPrinting } from "@/hooks/useTicketPrinting";
import { fetchTransaccionesDelDia } from "@/hooks/useTransactions";

interface RecentOrdersDialogProps {
  isOpen: boolean;
  onClose: () => void;
  API_URL: string;
  appId: string | null;
  formatFechaArgentina: (fecha: string | Date) => string;
  businessInfo: any;
}

export function RecentOrdersDialog({
  isOpen,
  onClose,
  API_URL,
  appId,
  formatFechaArgentina,
  businessInfo,
}: RecentOrdersDialogProps) {
  const { user } = useAuth();
  const [allOrders, setAllOrders] = useState<any[]>([]);
  const [isLoadingOrders, setIsLoadingOrders] = useState(false);
  const [isPrinting, setIsPrinting] = useState(false);

  // ✅ NUEVO: Usar el hook de impresión con soporte para doble impresión
  const { handleTicketPrinting } = useTicketPrinting();

  // Estados para paginación del lado del cliente
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(6); // ✅ CORREGIDO: Reducido de 10 a 6 para mostrar paginación

  // ✅ AGREGADO: Estado para tracking del usuario actual para detectar cambios
  const [currentUserId, setCurrentUserId] = useState<number | null>(null);

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // Calcular datos de paginación
  const totalOrders = allOrders.length;
  const totalPages = Math.ceil(totalOrders / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const currentOrders = allOrders.slice(startIndex, endIndex);

  // ✅ AGREGADO: Debug de paginación
  // console.log("🔍 Debug paginación:", {
  //   totalOrders,
  //   itemsPerPage,
  //   totalPages,
  //   currentPage,
  //   startIndex,
  //   endIndex,
  //   currentOrdersCount: currentOrders.length,
  // });

  // ✅ AGREGADO: Función para limpiar cache y resetear estados
  const clearCache = () => {
    // console.log("🧹 Limpiando cache de órdenes...");
    setAllOrders([]);
    setCurrentPage(1);
    setIsLoadingOrders(false);
    setIsPrinting(false);
  };

  // ✅ AGREGADO: useEffect para detectar cambios de usuario y limpiar cache
  useEffect(() => {
    if (user?.id && user.id !== currentUserId) {
      // console.log("👤 Cambio de usuario detectado:", {
      //   anteriorUserId: currentUserId,
      //   nuevoUserId: user.id,
      //   nombreUsuario: user.nombre,
      // });

      // Limpiar cache cuando cambia el usuario
      clearCache();
      setCurrentUserId(user.id);
    } else if (!user?.id && currentUserId) {
      // Usuario se deslogueó
      console.log("🚪 Usuario se deslogueó, limpiando cache");
      clearCache();
      setCurrentUserId(null);
    }
  }, [user?.id, currentUserId]);

  // ✅ AGREGADO: useEffect para escuchar eventos de logout y limpiar cache
  useEffect(() => {
    const handleUserLogout = (event: CustomEvent) => {
      console.log(
        "🚪 RecentOrdersDialog: Logout detectado via evento",
        event.detail
      );
      clearCache();
    };

    // Agregar listener para el evento personalizado de logout
    window.addEventListener("userLogout", handleUserLogout as EventListener);

    return () => {
      window.removeEventListener(
        "userLogout",
        handleUserLogout as EventListener
      );
    };
  }, []);

  // Función para cargar todas las órdenes del día
  const loadDayOrders = async () => {
    if (!user) {
      toast.error("Debes iniciar sesión para ver órdenes");
      return;
    }

    setIsLoadingOrders(true);

    try {
      console.log("📊 Cargando órdenes para usuario:", {
        userId: user.id,
        nombre: user.nombre,
      });

      const includeFacturas = businessInfo?.facturacionHabilitada === true;
      const transacciones = await fetchTransaccionesDelDia(
        user.id,
        includeFacturas
      );

      // Filtro local por fecha del día en zona horaria de Argentina (evitar UTC)
      const timeZone = "America/Argentina/Buenos_Aires";
      const formatYMDInTZ = (d: Date) => {
        const parts = new Intl.DateTimeFormat("en-CA", {
          timeZone,
          year: "numeric",
          month: "2-digit",
          day: "2-digit",
        }).formatToParts(d);
        const y = parts.find((p) => p.type === "year")?.value ?? "";
        const m = parts.find((p) => p.type === "month")?.value ?? "";
        const da = parts.find((p) => p.type === "day")?.value ?? "";
        return `${y}-${m}-${da}`; // YYYY-MM-DD
      };

      const todayArg = formatYMDInTZ(new Date());
      const filtered = transacciones.filter(
        (t: any) => formatYMDInTZ(new Date(t.fecha)) === todayArg
      );

      setAllOrders(filtered);

      // Resetear a la primera página cuando se cargan nuevos datos
      setCurrentPage(1);
    } catch (error) {
      console.error("❌ Error al cargar órdenes:", error);
      toast.error("Error al cargar las órdenes del día");

      // ✅ MEJORADO: Limpiar órdenes en caso de error para evitar mostrar datos incorrectos
      setAllOrders([]);
    } finally {
      setIsLoadingOrders(false);
    }
  };

  // Cargar órdenes al abrir el diálogo
  useEffect(() => {
    if (isOpen && user?.id) {
      console.log("🔄 Diálogo abierto, cargando órdenes para:", user.nombre);
      loadDayOrders();
    }
  }, [isOpen, user?.id]); // ✅ CORREGIDO: Agregar user?.id como dependencia

  // ✅ AGREGADO: useEffect para limpiar al cerrar el diálogo
  useEffect(() => {
    if (!isOpen) {
      // console.log(
      //   "🚪 Diálogo cerrado, manteniendo cache para próxima apertura"
      // );
      // No limpiar cache al cerrar, solo resetear estado de carga
      setIsLoadingOrders(false);
      setIsPrinting(false);
    }
  }, [isOpen]);

  // ✅ NUEVO: Función para reimprimir un ticket con soporte para doble impresión
  const handleReprintClick = async (order: any) => {
    if (isPrinting) return;
    setIsPrinting(true);
    try {
      // Debug de la estructura completa de la orden
      console.log("🖨️ Orden original para reimpresión:", order);
      console.log("🖨️ Claves disponibles en la orden:", Object.keys(order));
      console.log("🖨️ Items:", order.items);
      console.log("🖨️ Detalles:", order.detalles);
      console.log("🖨️ Vendedor:", order.vendedor);
      // Normalizar los datos del vendedor
      let vendedorNormalizado = "N/A";
      if (order.vendedor) {
        if (typeof order.vendedor === "string") {
          vendedorNormalizado = order.vendedor;
        } else if (
          typeof order.vendedor === "object" &&
          order.vendedor !== null
        ) {
          vendedorNormalizado =
            order.vendedor.nombre || order.vendedor.name || "N/A";
        }
      }

      // Normalizar los items de la orden
      let itemsNormalizados = [];
      if (order.items && Array.isArray(order.items)) {
        itemsNormalizados = order.items.map((item: any) => {
          // Debug del item para entender su estructura
          console.log("🔍 Item original:", item);

          const nombre =
            item.nombre ||
            item.producto?.nombre ||
            item.producto ||
            item.detalle?.nombre ||
            "Producto";
          const cantidad =
            item.cantidad || item.qty || item.cantidadProducto || 0;
          const precioHistorico =
            item.precioHistorico ||
            item.precio ||
            item.price ||
            item.precioUnitario ||
            0;
          const subtotal = item.subtotal || cantidad * precioHistorico;

          console.log("🔍 Item normalizado:", {
            nombre,
            cantidad,
            precioHistorico,
            subtotal,
          });

          return {
            nombre,
            cantidad,
            precioHistorico,
            subtotal,
          };
        });
      } else if (order.detalles && Array.isArray(order.detalles)) {
        // Si no hay items pero hay detalles (estructura alternativa)
        itemsNormalizados = order.detalles.map((detalle: any) => {
          console.log("🔍 Detalle original:", detalle);

          const nombre =
            detalle.nombre ||
            detalle.producto?.nombre ||
            detalle.producto ||
            "Producto";
          const cantidad =
            detalle.cantidad || detalle.qty || detalle.cantidadProducto || 0;
          const precioHistorico =
            detalle.precioHistorico ||
            detalle.precio ||
            detalle.price ||
            detalle.precioUnitario ||
            0;
          const subtotal = detalle.subtotal || cantidad * precioHistorico;

          console.log("🔍 Detalle normalizado:", {
            nombre,
            cantidad,
            precioHistorico,
            subtotal,
          });

          return {
            nombre,
            cantidad,
            precioHistorico,
            subtotal,
          };
        });
      }

      console.log("🖨️ Items normalizados finales:", itemsNormalizados);

      // Crear orden normalizada para impresión
      const enrichedOrder = {
        ...order,
        vendedor: vendedorNormalizado,
        items: itemsNormalizados,
        businessName: order.businessName || (await getBusinessName()),
      };

      console.log("🖨️ Datos normalizados para reimpresión:", enrichedOrder);

      // Usar la función de impresión con soporte para doble impresión
      await handleTicketPrinting(enrichedOrder, API_URL, appId);
    } catch (error) {
      console.error("Error al reimprimir desde diálogo:", error);
    } finally {
      setIsPrinting(false);
    }
  };

  // Funciones de navegación de páginas
  const goToPage = (page: number) => {
    if (page >= 1 && page <= totalPages) {
      setCurrentPage(page);
    }
  };

  const goToPreviousPage = () => {
    if (currentPage > 1) {
      setCurrentPage(currentPage - 1);
    }
  };

  const goToNextPage = () => {
    if (currentPage < totalPages) {
      setCurrentPage(currentPage + 1);
    }
  };

  // Generar números de página para mostrar
  const getPageNumbers = () => {
    const pageNumbers = [];
    const maxPagesToShow = 5;

    let startPage = Math.max(1, currentPage - Math.floor(maxPagesToShow / 2));
    let endPage = Math.min(totalPages, startPage + maxPagesToShow - 1);

    // Ajustar si estamos cerca del final
    if (endPage - startPage + 1 < maxPagesToShow) {
      startPage = Math.max(1, endPage - maxPagesToShow + 1);
    }

    for (let i = startPage; i <= endPage; i++) {
      pageNumbers.push(i);
    }

    return pageNumbers;
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-4xl max-h-[90vh] flex flex-col">
        <DialogHeader className="border-b border-emerald-100 pb-4 flex-shrink-0">
          <DialogTitle className="text-2xl font-bold text-emerald-gradient">
            Transacciones del día
          </DialogTitle>
          <DialogDescription className="text-lg">
            Todas las órdenes realizadas hoy por{" "}
            <span className="font-semibold text-emerald-700">
              {user?.nombre}
            </span>
            {totalOrders > 0 && (
              <span className="ml-2 text-sm text-gray-600">
                ({totalOrders} transacciones en total)
              </span>
            )}
          </DialogDescription>
        </DialogHeader>

        {/* ✅ CORREGIDO: Contenido con scroll independiente */}
        <div className="flex-1 overflow-y-auto py-4 min-h-0">
          {isLoadingOrders ? (
            <div className="flex justify-center items-center h-40">
              <div className="animate-spin rounded-full h-12 w-12 border-4 border-emerald-200 border-t-emerald-600"></div>
              <p className="ml-4 text-emerald-700 font-medium">
                Cargando órdenes del día...
              </p>
            </div>
          ) : totalOrders === 0 ? (
            <div className="text-center py-12">
              <div className="text-emerald-300 mb-4">
                <Receipt className="h-16 w-16 mx-auto" />
              </div>
              <p className="text-lg text-gray-500">
                No se encontraron órdenes ni facturas B para el día de hoy
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="rounded-lg border border-emerald-100 overflow-hidden">
                <Table>
                  <TableHeader className="bg-emerald-50">
                    <TableRow>
                      <TableHead className="font-semibold text-emerald-800">
                        Tipo
                      </TableHead>
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
                    {currentOrders.map((order, index) => (
                      <TableRow
                        key={order.id}
                        className={cn(
                          "hover:bg-emerald-25 transition-colors",
                          index % 2 === 0 ? "bg-white" : "bg-gray-50/50"
                        )}
                      >
                        <TableCell className="py-4 capitalize">
                          {order.tipo === "facturaB" ? "Factura B" : "Orden"}
                        </TableCell>
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
            </div>
          )}
        </div>

        {/* ✅ CORREGIDO: Paginación fija en la parte inferior */}
        {totalOrders > 0 && (
          <div className="border-t border-emerald-100 pt-4 flex-shrink-0">
            <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
              <div className="text-sm text-gray-600">
                Mostrando {startIndex + 1} a {Math.min(endIndex, totalOrders)}{" "}
                de {totalOrders} transacciones
              </div>

              {/* Solo mostrar controles de navegación si hay más de 1 página */}
              {totalPages > 1 && (
                <div className="flex items-center space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={goToPreviousPage}
                    disabled={currentPage === 1}
                    className="h-8 w-8 p-0"
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </Button>

                  {getPageNumbers().map((pageNum) => (
                    <Button
                      key={pageNum}
                      variant={currentPage === pageNum ? "default" : "outline"}
                      size="sm"
                      onClick={() => goToPage(pageNum)}
                      className={cn(
                        "h-8 w-8 p-0",
                        currentPage === pageNum
                          ? "bg-emerald-600 text-white hover:bg-emerald-700"
                          : "text-gray-600 hover:text-emerald-600"
                      )}
                    >
                      {pageNum}
                    </Button>
                  ))}

                  <Button
                    variant="outline"
                    size="sm"
                    onClick={goToNextPage}
                    disabled={currentPage === totalPages}
                    className="h-8 w-8 p-0"
                  >
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              )}

              <div className="text-sm text-gray-600">
                {totalPages > 1 ? (
                  <>
                    Página {currentPage} de {totalPages}
                  </>
                ) : (
                  <>Todas las órdenes</>
                )}
              </div>
            </div>
          </div>
        )}

        <DialogFooter className="border-t border-emerald-100 pt-4 flex-shrink-0">
          <Button
            variant="outline"
            onClick={onClose}
            className="border-emerald-200 text-emerald-700 hover:bg-emerald-50"
          >
            Cerrar
          </Button>
          <Button
            onClick={loadDayOrders}
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
