"use client";

import React, { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Edit, Save, X, Download, FileText } from "lucide-react";
import { toast } from "sonner";

interface Cliente {
  id: string;
  nombre: string;
  apellido?: string;
  razonSocial?: string;
  cuit?: string;
  condicionFiscal: string;
  email?: string;
  telefono?: string;
  direccion?: string;
}

interface Producto {
  id: number;
  nombre: string;
  tipoMedida: string;
}

interface DetalleFactura {
  id: number;
  descripcion: string;
  cantidad: number;
  precioUnitario: number;
  subtotal: number;
  producto: Producto;
}

interface Factura {
  id: string;
  numero: string;
  tipoFactura: string;
  puntoVenta: number;
  fecha: string;
  subtotal: number;
  impuestos: number;
  total: number;
  estado: string;
  cae?: string;
  vencimientoCae?: string;
  observaciones?: string;
  cliente?: Cliente;
  detalles: DetalleFactura[];
  business?: {
    nombre: string;
    tipo: string;
  };
}

interface FacturaModalProps {
  factura: Factura;
  mode: "view" | "edit";
  onClose: () => void;
  onUpdate: () => void;
}

const FacturaModal: React.FC<FacturaModalProps> = ({
  factura,
  mode,
  onClose,
  onUpdate,
}) => {
  const [editMode, setEditMode] = useState(mode === "edit");
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    estado: factura.estado,
    observaciones: factura.observaciones || "",
  });

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("es-AR", {
      style: "currency",
      currency: "ARS",
    }).format(amount);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("es-AR");
  };

  const formatClienteName = (cliente?: Cliente) => {
    if (!cliente) return "Cliente General";
    return (
      cliente.razonSocial ||
      `${cliente.nombre} ${cliente.apellido || ""}`.trim()
    );
  };

  const formatCondicionFiscal = (condicion: string) => {
    const condiciones: { [key: string]: string } = {
      consumidor_final: "Consumidor Final",
      responsable_inscripto: "Responsable Inscripto",
      exento: "Exento",
      monotributo: "Monotributo",
    };
    return condiciones[condicion] || condicion;
  };

  const getEstadoBadge = (estado: string) => {
    const config = {
      pendiente: {
        variant: "secondary" as const,
        color: "bg-yellow-100 text-yellow-800",
      },
      pagada: {
        variant: "default" as const,
        color: "bg-green-100 text-green-800",
      },
      anulada: {
        variant: "destructive" as const,
        color: "bg-red-100 text-red-800",
      },
    };

    const { variant, color } =
      config[estado as keyof typeof config] || config.pendiente;

    return (
      <Badge variant={variant} className={color}>
        {estado.toUpperCase()}
      </Badge>
    );
  };

  const getTipoFacturaBadge = (tipo: string) => {
    const colors = {
      A: "bg-blue-100 text-blue-800",
      B: "bg-purple-100 text-purple-800",
      C: "bg-orange-100 text-orange-800",
    };

    return (
      <Badge className={colors[tipo as keyof typeof colors] || colors.B}>
        Factura {tipo}
      </Badge>
    );
  };

  const handleSave = async () => {
    try {
      setLoading(true);

      const response = await fetch(`/api/facturas/${factura.id}`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          estado: formData.estado,
          observaciones: formData.observaciones,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Error al actualizar factura");
      }

      toast.success("Factura actualizada correctamente");
      onUpdate();
    } catch (error) {
      console.error("Error:", error);
      toast.error(
        error instanceof Error ? error.message : "Error al actualizar factura"
      );
    } finally {
      setLoading(false);
    }
  };

  const handleDownloadPDF = async () => {
    try {
      const response = await fetch(
        `/api/facturas/generar-pdf?id=${factura.id}`
      );

      if (!response.ok) {
        throw new Error("Error al generar PDF");
      }

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.style.display = "none";
      a.href = url;
      a.download = `factura-${factura.tipoFactura}-${factura.numero}.pdf`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

      toast.success("PDF descargado correctamente");
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al descargar PDF");
    }
  };

  return (
    <Dialog open={true} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <FileText className="h-5 w-5" />
              <span>
                Factura {factura.tipoFactura}-{factura.numero}
              </span>
              {getTipoFacturaBadge(factura.tipoFactura)}
              {getEstadoBadge(factura.estado)}
            </div>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={handleDownloadPDF}>
                <Download className="h-4 w-4 mr-2" />
                PDF
              </Button>
              {!editMode && factura.estado !== "anulada" && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setEditMode(true)}
                >
                  <Edit className="h-4 w-4 mr-2" />
                  Editar
                </Button>
              )}
              {editMode && (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setEditMode(false);
                      setFormData({
                        estado: factura.estado,
                        observaciones: factura.observaciones || "",
                      });
                    }}
                  >
                    <X className="h-4 w-4 mr-2" />
                    Cancelar
                  </Button>
                  <Button size="sm" onClick={handleSave} disabled={loading}>
                    <Save className="h-4 w-4 mr-2" />
                    {loading ? "Guardando..." : "Guardar"}
                  </Button>
                </>
              )}
            </div>
          </DialogTitle>
          <DialogDescription>
            Detalles completos de la factura {factura.tipoFactura}-
            {factura.numero}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Información de la factura */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Información General</h3>
              <div className="space-y-2">
                <div>
                  <Label className="text-sm font-medium">Número</Label>
                  <p className="font-mono">
                    {factura.tipoFactura}-{factura.numero}
                  </p>
                </div>
                <div>
                  <Label className="text-sm font-medium">Fecha</Label>
                  <p>{formatDate(factura.fecha)}</p>
                </div>
                <div>
                  <Label className="text-sm font-medium">Punto de Venta</Label>
                  <p>{factura.puntoVenta}</p>
                </div>
                {factura.cae && (
                  <>
                    <div>
                      <Label className="text-sm font-medium">CAE</Label>
                      <p className="font-mono">{factura.cae}</p>
                    </div>
                    {factura.vencimientoCae && (
                      <div>
                        <Label className="text-sm font-medium">
                          Vencimiento CAE
                        </Label>
                        <p>{formatDate(factura.vencimientoCae)}</p>
                      </div>
                    )}
                  </>
                )}
              </div>
            </div>

            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Estado y Control</h3>
              <div className="space-y-2">
                <div>
                  <Label className="text-sm font-medium">Estado</Label>
                  {editMode ? (
                    <Select
                      value={formData.estado}
                      onValueChange={(value) =>
                        setFormData({ ...formData, estado: value })
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="pendiente">Pendiente</SelectItem>
                        <SelectItem value="pagada">Pagada</SelectItem>
                        {!factura.cae && (
                          <SelectItem value="anulada">Anulada</SelectItem>
                        )}
                      </SelectContent>
                    </Select>
                  ) : (
                    <div>{getEstadoBadge(factura.estado)}</div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Información del cliente */}
          {factura.cliente && (
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Datos del Cliente</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label className="text-sm font-medium">
                    Nombre/Razón Social
                  </Label>
                  <p>{formatClienteName(factura.cliente)}</p>
                </div>
                {factura.cliente.cuit && (
                  <div>
                    <Label className="text-sm font-medium">CUIT</Label>
                    <p className="font-mono">{factura.cliente.cuit}</p>
                  </div>
                )}
                <div>
                  <Label className="text-sm font-medium">
                    Condición Fiscal
                  </Label>
                  <p>
                    {formatCondicionFiscal(factura.cliente.condicionFiscal)}
                  </p>
                </div>
                {factura.cliente.email && (
                  <div>
                    <Label className="text-sm font-medium">Email</Label>
                    <p>{factura.cliente.email}</p>
                  </div>
                )}
                {factura.cliente.telefono && (
                  <div>
                    <Label className="text-sm font-medium">Teléfono</Label>
                    <p>{factura.cliente.telefono}</p>
                  </div>
                )}
                {factura.cliente.direccion && (
                  <div className="md:col-span-2">
                    <Label className="text-sm font-medium">Dirección</Label>
                    <p>{factura.cliente.direccion}</p>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Detalles de la factura */}
          <div className="space-y-4">
            <h3 className="text-lg font-semibold">Detalles de la Factura</h3>
            <div className="border rounded-md">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Producto</TableHead>
                    <TableHead>Descripción</TableHead>
                    <TableHead className="text-right">Cantidad</TableHead>
                    <TableHead className="text-right">P. Unitario</TableHead>
                    <TableHead className="text-right">Subtotal</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {factura.detalles.map((detalle) => (
                    <TableRow key={detalle.id}>
                      <TableCell>
                        <div>
                          <p className="font-medium">
                            {detalle.producto.nombre}
                          </p>
                          <p className="text-sm text-muted-foreground">
                            {detalle.producto.tipoMedida}
                          </p>
                        </div>
                      </TableCell>
                      <TableCell>{detalle.descripcion}</TableCell>
                      <TableCell className="text-right">
                        {detalle.cantidad}
                      </TableCell>
                      <TableCell className="text-right">
                        {formatCurrency(detalle.precioUnitario)}
                      </TableCell>
                      <TableCell className="text-right font-medium">
                        {formatCurrency(detalle.subtotal)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            {/* Totales */}
            <div className="flex justify-end">
              <div className="w-80 space-y-2">
                <div className="flex justify-between">
                  <span>Subtotal:</span>
                  <span>{formatCurrency(factura.subtotal)}</span>
                </div>
                {factura.impuestos > 0 && (
                  <div className="flex justify-between">
                    <span>IVA (21%):</span>
                    <span>{formatCurrency(factura.impuestos)}</span>
                  </div>
                )}
                <div className="border-t pt-2">
                  <div className="flex justify-between font-bold text-lg">
                    <span>Total:</span>
                    <span>{formatCurrency(factura.total)}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Observaciones */}
          <div className="space-y-4">
            <h3 className="text-lg font-semibold">Observaciones</h3>
            {editMode ? (
              <Textarea
                value={formData.observaciones}
                onChange={(e) =>
                  setFormData({ ...formData, observaciones: e.target.value })
                }
                placeholder="Ingresa observaciones para la factura..."
                rows={3}
              />
            ) : (
              <div className="min-h-[60px] p-3 border rounded-md bg-muted/30">
                {factura.observaciones || "Sin observaciones"}
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default FacturaModal;
