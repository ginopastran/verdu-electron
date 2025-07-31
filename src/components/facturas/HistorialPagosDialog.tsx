import React from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Factura } from "@/types/factura";
import {
  DollarSign,
  Calendar,
  CheckCircle,
  Wallet,
  CreditCard,
  ArrowRight,
  X,
} from "lucide-react";

interface HistorialPagosDialogProps {
  factura: Factura;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const HistorialPagosDialog: React.FC<HistorialPagosDialogProps> = ({
  factura,
  open,
  onOpenChange,
}) => {
  const totalPagado =
    factura.pagos?.reduce((sum, pago) => sum + pago.monto, 0) || 0;
  const saldoPendiente = factura.total - totalPagado;
  const porcentajePagado =
    factura.total > 0 ? (totalPagado / factura.total) * 100 : 0;

  const getMetodoPagoIcon = (metodo: string) => {
    switch (metodo.toLowerCase()) {
      case "efectivo":
        return <Wallet className="w-4 h-4" />;
      case "tarjeta":
        return <CreditCard className="w-4 h-4" />;
      case "transferencia":
        return <ArrowRight className="w-4 h-4" />;
      default:
        return <DollarSign className="w-4 h-4" />;
    }
  };

  const getMetodoPagoLabel = (metodo: string) => {
    switch (metodo.toLowerCase()) {
      case "efectivo":
        return "Efectivo";
      case "tarjeta":
        return "Tarjeta";
      case "transferencia":
        return "Transferencia";
      default:
        return metodo;
    }
  };

  const getEstadoFactura = () => {
    if (saldoPendiente <= 0) {
      return {
        texto: "Pagada completamente",
        badge: <Badge className="bg-black text-white">PAGADA</Badge>,
      };
    } else if (totalPagado > 0) {
      return {
        texto: "Pago parcial",
        badge: (
          <Badge className="bg-yellow-100 text-yellow-800 border-yellow-200">
            PARCIAL
          </Badge>
        ),
      };
    } else {
      return {
        texto: "Pendiente de pago",
        badge: (
          <Badge className="bg-red-100 text-red-800 border-red-200">
            PENDIENTE
          </Badge>
        ),
      };
    }
  };

  const estadoFactura = getEstadoFactura();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <DollarSign className="w-5 h-5" />
            Historial de Pagos
          </DialogTitle>
          <DialogDescription>
            Detalle de todos los pagos realizados para esta factura
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Resumen de totales */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 p-4 bg-gray-50 rounded-lg">
            <div>
              <div className="text-sm text-gray-600">Total Factura</div>
              <div className="text-lg font-semibold text-black">
                $
                {factura.total.toLocaleString("es-AR", {
                  minimumFractionDigits: 2,
                })}
              </div>
            </div>
            <div>
              <div className="text-sm text-gray-600">Total Pagado</div>
              <div className="text-lg font-semibold text-green-600">
                $
                {totalPagado.toLocaleString("es-AR", {
                  minimumFractionDigits: 2,
                })}
              </div>
            </div>
            <div>
              <div className="text-sm text-gray-600">Saldo Pendiente</div>
              <div className="text-lg font-semibold text-red-600">
                $
                {saldoPendiente.toLocaleString("es-AR", {
                  minimumFractionDigits: 2,
                })}
              </div>
            </div>
            <div>
              <div className="text-sm text-gray-600">% Pagado</div>
              <div className="text-lg font-semibold text-blue-600">
                {porcentajePagado.toFixed(1)}%
              </div>
            </div>
          </div>

          {/* Barra de progreso */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Progreso de pago</span>
              <span>{porcentajePagado.toFixed(1)}%</span>
            </div>
            <Progress value={porcentajePagado} className="h-2" />
          </div>

          {/* Lista de pagos */}
          <div className="space-y-4">
            <h3 className="text-lg font-semibold">Pagos Realizados</h3>

            {factura.pagos && factura.pagos.length > 0 ? (
              <div className="space-y-3">
                {factura.pagos.map((pago) => (
                  <div
                    key={pago.id}
                    className="flex items-center justify-between p-4 border border-gray-200 rounded-lg"
                  >
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 bg-emerald-600 rounded-full flex items-center justify-center text-white">
                        {getMetodoPagoIcon(pago.metodoPago)}
                      </div>
                      <div>
                        <div className="font-semibold">
                          $
                          {pago.monto.toLocaleString("es-AR", {
                            minimumFractionDigits: 2,
                          })}
                        </div>
                        <div className="text-sm text-gray-600">
                          {getMetodoPagoLabel(pago.metodoPago)}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-4">
                      <div className="text-right">
                        <div className="text-sm text-gray-600 flex items-center gap-1">
                          <Calendar className="w-3 h-3" />
                          {new Date(pago.fecha).toLocaleDateString("es-AR")}
                        </div>
                        {pago.referencia && (
                          <div className="text-xs text-gray-500">
                            Ref: {pago.referencia}
                          </div>
                        )}
                      </div>
                      <Badge className="bg-green-100 text-green-800 border-green-200">
                        <CheckCircle className="w-3 h-3 mr-1" />
                        PAGADO
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-gray-500">
                No hay pagos registrados para esta factura
              </div>
            )}
          </div>

          {/* Estado de la factura */}
          <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
            <span className="font-medium">{estadoFactura.texto}</span>
            {estadoFactura.badge}
          </div>

          {/* Botón cerrar */}
          <div className="flex justify-end">
            <Button onClick={() => onOpenChange(false)}>Cerrar</Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
