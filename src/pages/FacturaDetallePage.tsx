import React, { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  ArrowLeft,
  Download,
  DollarSign,
  User,
  Package,
  Calendar,
  FileText,
} from "lucide-react";
import { Factura } from "@/types/factura";
import { facturasService } from "@/services/facturas";

export const FacturaDetallePage: React.FC = () => {
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [factura, setFactura] = useState<Factura | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  console.log("🔍 [FacturaDetalle] Params completos:", params);
  console.log("🔍 [FacturaDetalle] ID extraído:", params.id);
  console.log("🔍 [FacturaDetalle] Tipo del ID:", typeof params.id);

  const id = params.id;

  useEffect(() => {
    console.log("🔍 [FacturaDetalle] ID recibido:", id);
    if (id) {
      console.log(
        "🚀 [FacturaDetalle] Llamando a fetchFacturaDetalle con ID:",
        id
      );
      fetchFacturaDetalle(id);
    } else {
      console.error("❌ [FacturaDetalle] No se recibió ID de la URL");
      setError("No se encontró el ID de la factura");
    }
  }, [id]);

  const fetchFacturaDetalle = async (facturaId: string) => {
    setLoading(true);
    console.log(
      "🌐 [FacturaDetalle] Haciendo petición a:",
      `/facturas/${facturaId}`
    );
    try {
      const factura = await facturasService.getFactura(facturaId);
      setFactura(factura);
      console.log("📄 [FacturaDetalle] Factura cargada:", factura);
    } catch (error) {
      console.error("❌ [FacturaDetalle] Error:", error);
      console.error(
        "❌ [FacturaDetalle] URL que falló:",
        `/facturas/${facturaId}`
      );
      setError("Error al cargar la factura");
    } finally {
      setLoading(false);
    }
  };

  const handleBack = () => {
    navigate("/facturas");
  };

  const handleDownloadPDF = async () => {
    if (!factura) return;
    try {
      await facturasService.downloadPDF(factura.id);
      // Implementar descarga del PDF
    } catch (error) {
      console.error("Error al descargar PDF:", error);
    }
  };

  const handleAgregarPago = () => {
    // Aquí podrías abrir el diálogo de agregar pago
    console.log("Agregar pago para factura:", factura?.id);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="mb-4 h-8 w-8 animate-spin rounded-full border-4 border-emerald-400 border-t-transparent mx-auto"></div>
          <p>Cargando factura...</p>
        </div>
      </div>
    );
  }

  if (error || !factura) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-500 mb-4">Error: {error}</p>
          <Button onClick={handleBack}>Volver</Button>
        </div>
      </div>
    );
  }

  const totalPagado =
    factura.pagos?.reduce((sum, pago) => sum + pago.monto, 0) || 0;
  const saldoPendiente = factura.total - totalPagado;
  const isCompletamentePagada = saldoPendiente <= 0;

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Header */}
      <header className="p-6 border-b border-gray-200 bg-emerald-gradient">
        <div className="flex items-center gap-4 justify-between">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleBack}
              className="text-white hover:text-gray-900 hover:bg-gray-100"
            >
              <ArrowLeft className="w-4 h-4 mr-2" />
              Volver
            </Button>
            <div>
              <h1 className="text-2xl font-semibold text-white">
                Factura {factura.tipoFactura} {factura.numero}
              </h1>
              <p className="text-emerald-100">
                Punto de Venta: {factura.puntoVenta}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge
              variant={isCompletamentePagada ? "default" : "secondary"}
              className={
                isCompletamentePagada ? "bg-green-600" : "bg-yellow-600"
              }
            >
              {isCompletamentePagada ? "PAGADA" : "PENDIENTE"}
            </Badge>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownloadPDF}
              className="text-white bg-black"
            >
              <Download className="w-4 h-4 mr-2" />
              Descargar PDF
            </Button>
            {saldoPendiente > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleAgregarPago}
                className="text-white bg-black"
              >
                <DollarSign className="w-4 h-4 mr-2" />
                Agregar Pago
              </Button>
            )}
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 p-6 overflow-auto bg-gray-50">
        <div className="max-w-6xl mx-auto space-y-6">
          {/* Información de la factura y totales */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Información de la factura */}
            <Card className="border-l-4 border-l-blue-500 bg-white shadow-sm">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <FileText className="w-5 h-5" />
                  Información de la Factura
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-600">FECHA DE EMISIÓN:</span>
                  <span className="font-medium">
                    {new Date(factura.fecha).toLocaleDateString()}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">TIPO DE FACTURA:</span>
                  <span className="font-medium capitalize">
                    {factura.tipoFactura}
                  </span>
                </div>
                {factura.cae && (
                  <div className="flex justify-between">
                    <span className="text-gray-600">CAE:</span>
                    <span className="font-medium">{factura.cae}</span>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Totales */}
            <Card className="border-l-4 border-l-green-500 bg-white shadow-sm">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <DollarSign className="w-5 h-5" />
                  Totales
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-600">Subtotal:</span>
                  <span className="font-medium">
                    $ {factura.subtotal.toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Impuestos:</span>
                  <span className="font-medium">
                    $ {factura.impuestos.toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between text-lg font-semibold text-green-600 border-t pt-2">
                  <span>Total:</span>
                  <span>$ {factura.total.toLocaleString()}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Total pagado:</span>
                  <span className="font-medium text-yellow-600">
                    $ {totalPagado.toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Saldo pendiente:</span>
                  <span
                    className={`font-medium ${
                      saldoPendiente > 0 ? "text-red-600" : "text-green-600"
                    }`}
                  >
                    $ {saldoPendiente.toLocaleString()}
                  </span>
                </div>
              </CardContent>
            </Card>

            {/* Información del cliente */}
            <Card className="border-l-4 border-l-purple-500 bg-white shadow-sm">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <User className="w-5 h-5" />
                  Información del Cliente
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-600">Cliente:</span>
                  <span className="font-medium">
                    {factura.cliente?.nombre ||
                      factura.cliente?.razonSocial ||
                      "Sin cliente"}
                  </span>
                </div>
                {factura.cliente?.cuit && (
                  <div className="flex justify-between">
                    <span className="text-gray-600">CUIT:</span>
                    <span className="font-medium">{factura.cliente.cuit}</span>
                  </div>
                )}
                <div className="flex justify-between">
                  <span className="text-gray-600">Condición Fiscal:</span>
                  <span className="font-medium capitalize">
                    {factura.cliente?.condicionFiscal || "N/A"}
                  </span>
                </div>
                <Button variant="outline" size="sm" className="w-full mt-2">
                  <User className="w-4 h-4 mr-2" />
                  Ver Cliente
                </Button>
              </CardContent>
            </Card>
          </div>

          {/* Detalle de productos */}
          <Card className="border-l-4 border-l-blue-500 bg-white shadow-sm">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Package className="w-5 h-5" />
                Detalle de Productos
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-gray-200">
                      <th className="text-left py-3 px-4 font-medium text-gray-600">
                        Producto
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-gray-600">
                        Descripción
                      </th>
                      <th className="text-left py-3 px-4 font-medium text-gray-600">
                        Cantidad
                      </th>
                      <th className="text-right py-3 px-4 font-medium text-gray-600">
                        Precio Unit.
                      </th>
                      <th className="text-right py-3 px-4 font-medium text-gray-600">
                        Subtotal
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {factura.detalles.map((detalle) => (
                      <tr key={detalle.id} className="border-b border-gray-100">
                        <td className="py-3 px-4">
                          <div>
                            <div className="font-medium">
                              {detalle.producto.nombre}
                            </div>
                            <div className="text-sm text-gray-500">
                              {detalle.producto.tipoMedida}
                            </div>
                          </div>
                        </td>
                        <td className="py-3 px-4 text-gray-600">
                          {detalle.descripcion}
                        </td>
                        <td className="py-3 px-4">{detalle.cantidad}</td>
                        <td className="py-3 px-4 text-right">
                          $ {detalle.precioUnitario.toLocaleString()}
                        </td>
                        <td className="py-3 px-4 text-right font-medium">
                          $ {detalle.subtotal.toLocaleString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>

          {/* Historial de pagos */}
          {factura.pagos && factura.pagos.length > 0 && (
            <Card className="border-l-4 border-l-green-500 bg-white shadow-sm">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <DollarSign className="w-5 h-5" />
                  Historial de Pagos
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {factura.pagos.map((pago) => (
                    <div
                      key={pago.id}
                      className="flex items-center justify-between p-4 bg-gray-50 rounded-lg"
                    >
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 bg-green-100 rounded-full flex items-center justify-center">
                          <DollarSign className="w-5 h-5 text-green-600" />
                        </div>
                        <div>
                          <div className="font-semibold">
                            $ {pago.monto.toLocaleString()}
                          </div>
                          <div className="text-sm text-gray-600 capitalize">
                            {pago.metodoPago.replace("_", " ")}
                          </div>
                          {pago.referencia && (
                            <div className="text-xs text-gray-500">
                              Ref: {pago.referencia}
                            </div>
                          )}
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="text-sm text-gray-600">
                          {new Date(pago.fecha).toLocaleDateString()}
                        </div>
                        <Badge variant="default" className="bg-green-600">
                          {pago.estado}
                        </Badge>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Observaciones */}
          {factura.observaciones && (
            <Card className="bg-white shadow-sm">
              <CardHeader>
                <CardTitle>Observaciones</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-gray-600">{factura.observaciones}</p>
              </CardContent>
            </Card>
          )}
        </div>
      </main>
    </div>
  );
};
