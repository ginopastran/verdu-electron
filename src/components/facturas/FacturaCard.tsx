import React from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Eye,
  Edit,
  Download,
  Trash2,
  DollarSign,
  Calendar,
  User,
  Package,
  CheckCircle,
  Clock,
  XCircle,
} from "lucide-react";
import { Factura } from "@/types/factura";

interface FacturaCardProps {
  factura: Factura;
  onVer: (factura: Factura) => void;
  onEditar: (factura: Factura) => void;
  onDescargar: (id: string) => void;
  onEliminar: (id: string) => void;
  onVerPagos: (factura: Factura) => void;
  onAgregarPago: (factura: Factura) => void;
}

const getEstadoBadge = (factura: Factura) => {
  const totalPagado =
    factura.pagos?.reduce((sum, pago) => sum + pago.monto, 0) || 0;
  const saldoPendiente = factura.total - totalPagado;
  const isCompletamentePagada = saldoPendiente <= 0;

  // Si no hay pagos registrados, usar el estado del backend
  if (!factura.pagos || factura.pagos.length === 0) {
    return (
      <Badge className="bg-gray-100 text-gray-800 border-gray-200">
        {factura.estado === "pagada" ? (
          <>
            <CheckCircle className="w-3 h-3 mr-1" />
            Pagada
          </>
        ) : (
          <>
            <Clock className="w-3 h-3 mr-1" />
            {factura.estado}
          </>
        )}
      </Badge>
    );
  }

  // Si hay pagos, calcular el estado basado en el saldo pendiente
  if (isCompletamentePagada) {
    return (
      <Badge className="bg-green-100 text-green-800 border-green-200">
        <CheckCircle className="w-3 h-3 mr-1" />
        Pagada
      </Badge>
    );
  } else if (totalPagado > 0) {
    return (
      <Badge className="bg-yellow-100 text-yellow-800 border-yellow-200">
        <Clock className="w-3 h-3 mr-1" />
        Pendiente
      </Badge>
    );
  } else {
    return (
      <Badge className="bg-red-100 text-red-800 border-red-200">
        <XCircle className="w-3 h-3 mr-1" />
        Pendiente
      </Badge>
    );
  }
};

const getTipoFacturaBadge = (tipo: string) => {
  switch (tipo.toLowerCase()) {
    case "remito":
      return (
        <Badge className="bg-orange-100 text-orange-800 border-orange-200">
          Remito
        </Badge>
      );
    case "a":
      return (
        <Badge className="bg-blue-100 text-blue-800 border-blue-200">
          Factura A
        </Badge>
      );
    case "c":
      return (
        <Badge className="bg-purple-100 text-purple-800 border-purple-200">
          Factura C
        </Badge>
      );
    default:
      return (
        <Badge className="bg-gray-100 text-gray-800 border-gray-200">
          {tipo}
        </Badge>
      );
  }
};

export const FacturaCard: React.FC<FacturaCardProps> = ({
  factura,
  onVer,
  onEditar,
  onDescargar,
  onEliminar,
  onVerPagos,
  onAgregarPago,
}) => {
  const navigate = useNavigate();
  const totalPagado =
    factura.pagos?.reduce((sum, pago) => sum + pago.monto, 0) || 0;
  const saldoPendiente = factura.total - totalPagado;
  const cantidadProductos = factura.detalles?.length || 0;

  // Debug: Log de datos de la factura
  console.log("🔍 [FacturaCard] Datos de factura:", {
    id: factura.id,
    numero: factura.numero,
    total: factura.total,
    estado: factura.estado,
    esCuentaCorriente: factura.esCuentaCorriente,
    pagos: factura.pagos,
    totalPagado,
    saldoPendiente,
  });

  const handleVer = () => {
    console.log("🔍 [FacturaCard] Navegando a factura con ID:", factura.id);
    console.log("🔍 [FacturaCard] URL completa:", `/facturas/${factura.id}`);
    navigate(`/facturas/${factura.id}`);
  };

  return (
    <div className="bg-white border border-[#A7A7A7] p-6 shadow-sm hover:shadow-md transition-shadow border-l-4 border-l-emerald-800 rounded-2xl">
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-emerald-gradient rounded-full flex items-center justify-center text-white font-semibold">
            {factura.numero.slice(-2)}
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900">
              {factura.numero}
            </h3>
            <p className="text-sm text-gray-600">
              {factura.cliente?.nombre ||
                factura.cliente?.razonSocial ||
                "Cliente no especificado"}
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          {getTipoFacturaBadge(factura.tipoFactura)}
          {getEstadoBadge(factura)}
        </div>
      </div>

      {/* Detalles */}
      <div className="space-y-3 mb-4">
        <div className="flex items-center gap-2 text-sm text-gray-600">
          <Calendar className="w-4 h-4" />
          <span>{new Date(factura.fecha).toLocaleDateString("es-AR")}</span>
        </div>

        <div className="flex items-center gap-2 text-lg font-semibold text-gray-900">
          <DollarSign className="w-5 h-5" />
          <span>
            $
            {factura.total.toLocaleString("es-AR", {
              minimumFractionDigits: 2,
            })}
          </span>
        </div>

        {/* Mostrar información de pagos si hay pagos registrados o es cuenta corriente */}
        {factura.pagos && factura.pagos.length > 0 && (
          <div className="space-y-1">
            <div className="flex justify-between text-sm">
              <span className="text-gray-600">Total pagado:</span>
              <span className="text-green-600 font-medium">
                $
                {totalPagado.toLocaleString("es-AR", {
                  minimumFractionDigits: 2,
                })}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-600">Saldo pendiente:</span>
              <span
                className={`font-medium ${
                  saldoPendiente > 0 ? "text-red-600" : "text-green-600"
                }`}
              >
                $
                {saldoPendiente.toLocaleString("es-AR", {
                  minimumFractionDigits: 2,
                })}
              </span>
            </div>
          </div>
        )}

        <div className="flex items-center gap-4 text-sm text-gray-600">
          <div className="flex items-center gap-1">
            <User className="w-4 h-4" />
            <span>
              {factura.cliente?.nombre || factura.cliente?.razonSocial || "N/A"}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <Package className="w-4 h-4" />
            <span>
              {cantidadProductos} producto{cantidadProductos !== 1 ? "s" : ""}
            </span>
          </div>
        </div>
      </div>

      {/* Botones de acción */}
      <div className="flex gap-1 pt-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleVer}
          className="flex-1 text-xs border-[#A7A7A7] rounded-xl"
        >
          <Eye className="w-3 h-3 mr-1" />
          Ver
        </Button>

        <Button
          variant="outline"
          size="sm"
          onClick={() => onEditar(factura)}
          className="flex-1 text-xs border-[#A7A7A7] rounded-xl"
        >
          <Edit className="w-3 h-3 mr-1" />
          Editar
        </Button>

        <Button
          variant="outline"
          size="sm"
          onClick={() => onDescargar(factura.id)}
          className="text-blue-600 hover:text-blue-700 hover:bg-blue-50 px-2 border-[#A7A7A7] rounded-xl"
        >
          <Download className="w-3 h-3" />
        </Button>

        <Button
          variant="outline"
          size="sm"
          onClick={() => onEliminar(factura.id)}
          className="text-red-600 hover:text-red-700 hover:bg-red-50 px-2 border-[#A7A7A7] rounded-xl"
        >
          <Trash2 className="w-3 h-3" />
        </Button>
      </div>

      {/* Botones de pago */}
      <div className="flex gap-1 pt-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => onVerPagos(factura)}
          className="text-xs border-green-200 text-green-700 hover:bg-green-50 w-full rounded-xl"
        >
          <DollarSign className="w-3 h-3 mr-1" />
          Ver Pagos
        </Button>

        {saldoPendiente > 0 && (
          <Button
            size="sm"
            onClick={() => onAgregarPago(factura)}
            className="text-xs bg-emerald-gradient w-full rounded-xl"
          >
            <DollarSign className="w-3 h-3 mr-1" />
            Agregar Pago
          </Button>
        )}
      </div>
    </div>
  );
};
