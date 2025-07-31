"use client";

import React, { useState, useEffect } from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Plus,
  Trash2,
  Save,
  X,
  FileText,
  Calculator,
  User,
  ChevronDown,
  CalendarIcon,
} from "lucide-react";
import { toast } from "sonner";
import ClienteSelectorDialog from "./ClienteSelectorDialog";
import ProductoSelectorDialog from "./ProductoSelectorDialog";
import CantidadProductoDialog from "./CantidadProductoDialog";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { format } from "date-fns";
import { es } from "date-fns/locale";

interface Cliente {
  id: string;
  nombre: string;
  apellido?: string;
  razonSocial?: string;
  cuit?: string;
  condicionFiscal: string;
}

interface Producto {
  id: number;
  nombre: string;
  tipoMedida: string;
  precio: number;
  stock?: number;
}

interface DetalleFactura {
  id?: number;
  productoId: number;
  producto?: Producto;
  descripcion: string;
  cantidad: number;
  precioUnitario: number;
  subtotal: number;
}

interface FormData {
  id?: string;
  clienteId?: string;
  tipoFactura: string;
  observaciones?: string;
  detalles: DetalleFactura[];
  // 🆕 CAMPOS PARA CUENTA CORRIENTE
  pagoInicial?: number;
  // 🆕 CAMPO PARA IVA CONFIGURABLE
  porcentajeIva?: number;
}

interface FacturaFormProps {
  factura?: any;
  mode: "create" | "edit";
  onClose: () => void;
  onSuccess: () => void;
}

const FacturaForm: React.FC<FacturaFormProps> = ({
  factura,
  mode,
  onClose,
  onSuccess,
}) => {
  const { businessInfo, loading: businessLoading } = useBusinessInfo(
    import.meta.env.VITE_API_URL || "http://localhost:3000",
    import.meta.env.VITE_APP_ID || null
  );
  const [loading, setLoading] = useState(false);
  const [selectedCliente, setSelectedCliente] = useState<Cliente | null>(null);
  const [showClienteDialog, setShowClienteDialog] = useState(false);
  const [showProductoDialog, setShowProductoDialog] = useState(false);
  const [showCantidadDialog, setShowCantidadDialog] = useState(false);
  const [selectedProductoForCantidad, setSelectedProductoForCantidad] =
    useState<Producto | null>(null);

  const [formData, setFormData] = useState<FormData>({
    clienteId: factura?.clienteId,
    tipoFactura: "remito",
    observaciones: factura?.observaciones || "",
    detalles: factura?.detalles || [],
    // 🆕 CAMPOS PARA CUENTA CORRIENTE
    pagoInicial: factura?.pagoInicial || undefined,
    // 🆕 CAMPO PARA IVA CONFIGURABLE
    porcentajeIva: factura?.porcentajeIva || undefined,
  });

  useEffect(() => {
    if (formData.clienteId) {
      fetchClienteById(formData.clienteId);
    }
  }, []);

  const fetchClienteById = async (clienteId: string) => {
    try {
      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      const response = await fetch(`${API_URL}/api/clientes/${clienteId}`);
      if (response.ok) {
        const data = await response.json();
        setSelectedCliente(data);
      }
    } catch (error) {
      console.error("Error al obtener cliente:", error);
    }
  };

  const calculateTotals = () => {
    const subtotal = formData.detalles.reduce(
      (sum, detalle) => sum + detalle.subtotal,
      0
    );

    // Calcular IVA usando el porcentaje configurado
    const porcentajeIva = formData.porcentajeIva || 0;
    const impuestos = subtotal * (porcentajeIva / 100);

    const total = subtotal + impuestos;

    return { subtotal, impuestos, total };
  };

  const handleClienteSelect = (cliente: Cliente | null) => {
    setSelectedCliente(cliente);
    setFormData({
      ...formData,
      clienteId: cliente?.id,
    });
  };

  const handleProductoSelect = (producto: Producto) => {
    // En lugar de agregar directamente, abrir el diálogo de cantidad
    setSelectedProductoForCantidad(producto);
    setShowCantidadDialog(true);
    setShowProductoDialog(false);
  };

  const handleConfirmarCantidad = (producto: Producto, cantidad: number) => {
    const nuevoDetalle: DetalleFactura = {
      productoId: producto.id,
      producto,
      descripcion: producto.nombre,
      cantidad: cantidad,
      precioUnitario: producto.precio,
      subtotal: producto.precio * cantidad,
    };

    setFormData({
      ...formData,
      detalles: [...formData.detalles, nuevoDetalle],
    });

    // Limpiar estados
    setSelectedProductoForCantidad(null);
    setShowCantidadDialog(false);
  };

  const updateDetalle = (index: number, field: string, value: any) => {
    const newDetalles = [...formData.detalles];
    newDetalles[index] = { ...newDetalles[index], [field]: value };

    // Recalcular subtotal si cambia cantidad o precio
    if (field === "cantidad" || field === "precioUnitario") {
      newDetalles[index].subtotal =
        newDetalles[index].cantidad * newDetalles[index].precioUnitario;
    }

    setFormData({ ...formData, detalles: newDetalles });
  };

  const removeDetalle = (index: number) => {
    const newDetalles = formData.detalles.filter((_, i) => i !== index);
    setFormData({ ...formData, detalles: newDetalles });
  };

  // Función para establecer IVA por defecto según tipo de factura
  const setDefaultIva = (tipoFactura: string) => {
    let defaultIva = 0;

    if (tipoFactura === "A" || tipoFactura === "C") {
      defaultIva = 21; // 21% para facturas A y C
    } else if (tipoFactura === "remito") {
      defaultIva = 0; // 0% para remitos
    }

    setFormData({
      ...formData,
      tipoFactura,
      porcentajeIva: defaultIva,
    });
  };

  const handleSubmit = async () => {
    try {
      // Validaciones
      if (!selectedCliente) {
        toast.error("Debe seleccionar un cliente para crear la factura");
        return;
      }

      if (formData.detalles.length === 0) {
        toast.error("Debe agregar al menos un producto a la factura");
        return;
      }

      const { subtotal, impuestos, total } = calculateTotals();

      setLoading(true);

      // Determinar si usar AFIP o crear factura local
      const usarAfip =
        businessInfo?.afipHabilitado && formData.tipoFactura !== "remito";

      console.log(`🔧 [FacturaForm] Business Info:`, businessInfo);
      console.log(
        `🔧 [FacturaForm] AFIP Habilitado:`,
        businessInfo?.afipHabilitado
      );
      console.log(`🔧 [FacturaForm] Tipo Factura:`, formData.tipoFactura);
      console.log(`🔧 [FacturaForm] Usar AFIP:`, usarAfip);

      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      let dataToSend: any;
      let url: string;

      if (mode === "create" && usarAfip) {
        // Usar endpoint de AFIP para facturas A, B, C
        url = `${API_URL}/api/facturas/crear-afip`;
        dataToSend = {
          clienteId: formData.clienteId,
          tipoFactura: formData.tipoFactura,
          observaciones: formData.observaciones,
          porcentajeIva: formData.porcentajeIva || 0,
          pagoInicial: formData.pagoInicial || 0,
          esConsumidorFinal: false, // Siempre false porque cliente es obligatorio
          productos: formData.detalles.map((detalle) => ({
            productoId: detalle.productoId,
            cantidad: detalle.cantidad,
            precio: detalle.precioUnitario,
          })),
        };
      } else {
        // Usar endpoint estándar para remitos o cuando AFIP no está habilitado
        url =
          mode === "create"
            ? `${API_URL}/api/facturas`
            : `${API_URL}/api/facturas/${factura.id}`;
        dataToSend = {
          clienteId: formData.clienteId,
          tipoFactura: formData.tipoFactura,
          observaciones: formData.observaciones,
          porcentajeIva: formData.porcentajeIva || 0,
          pagoInicial: formData.pagoInicial || 0,
          subtotal,
          impuestos,
          total,
          detalles: formData.detalles.map((detalle) => ({
            productoId: detalle.productoId,
            descripcion: detalle.descripcion,
            cantidad: detalle.cantidad,
            precioUnitario: detalle.precioUnitario,
            subtotal: detalle.subtotal,
          })),
        };
      }

      const method = mode === "create" ? "POST" : "PUT";

      console.log(
        `📤 [FacturaForm] Enviando ${
          usarAfip ? "factura AFIP" : "factura local"
        } a: ${url}`
      );
      console.log(`📤 [FacturaForm] Datos:`, dataToSend);

      const response = await fetch(url, {
        method,
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(dataToSend),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Error al guardar factura");
      }

      const result = await response.json();
      console.log(`✅ [FacturaForm] Respuesta exitosa:`, result);

      if (usarAfip && result.afip) {
        toast.success(
          `Factura AFIP creada correctamente\nCAE: ${result.afip.cae}\nVencimiento: ${result.afip.vencimientoCae}`
        );
      } else {
        toast.success(
          mode === "create"
            ? "Factura creada correctamente"
            : "Factura actualizada correctamente"
        );
      }

      onSuccess();
    } catch (error) {
      console.error("Error:", error);
      toast.error(
        error instanceof Error ? error.message : "Error al guardar factura"
      );
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("es-AR", {
      style: "currency",
      currency: "ARS",
    }).format(amount);
  };

  const formatClienteName = (cliente: Cliente) => {
    return (
      cliente.razonSocial ||
      `${cliente.nombre} ${cliente.apellido || ""}`.trim()
    );
  };

  const { subtotal, impuestos, total } = calculateTotals();

  const excludeProductIds = formData.detalles.map(
    (detalle) => detalle.productoId
  );

  // Renderizar el select de tipo de factura según afipHabilitado
  const renderTipoFacturaSelect = () => {
    if (businessLoading) {
      return <div>Cargando...</div>;
    }

    if (!businessInfo?.afipHabilitado) {
      return (
        <div>
          <Label htmlFor="tipoFactura">Tipo de Factura</Label>
          <Select value="remito" disabled={true}>
            <SelectTrigger>
              <SelectValue placeholder="Seleccionar tipo" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="remito">
                Remito - Solo entrega (sin AFIP)
              </SelectItem>
            </SelectContent>
          </Select>
          <p className="text-sm text-muted-foreground mt-1">
            AFIP no está habilitado para este negocio
          </p>
        </div>
      );
    }

    return (
      <div>
        <Label htmlFor="tipoFactura">Tipo de Factura</Label>
        <Select
          value={formData.tipoFactura}
          onValueChange={(value) => setDefaultIva(value)}
        >
          <SelectTrigger>
            <SelectValue placeholder="Seleccionar tipo" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="A">Factura A - Responsable Inscripto</SelectItem>
            <SelectItem value="C">Factura C - Monotributista</SelectItem>
            <SelectItem value="remito">
              Remito - Solo entrega (sin AFIP)
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
    );
  };

  return (
    <>
      <Dialog open={true} onOpenChange={onClose}>
        <DialogContent className="max-w-6xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-3 text-emerald-gradient">
              <FileText className="h-5 w-5 text-emerald-900" />
              {mode === "create" ? "Nueva Factura" : "Editar Factura"}
            </DialogTitle>
            <DialogDescription>
              {mode === "create"
                ? "Complete los datos para crear una nueva factura"
                : "Modifique los datos de la factura existente"}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-6">
            {/* Datos básicos */}
            <div className="space-y-6">
              {/* Primera fila: Cliente y Tipo de Factura */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-4">
                  <div>
                    <Label>Cliente *</Label>
                    <Button
                      variant="outline"
                      className="w-full justify-between h-auto p-3 border-[#A7A7A7]"
                      onClick={() => setShowClienteDialog(true)}
                    >
                      <div className="text-left">
                        {selectedCliente ? (
                          <div>
                            <p className="font-medium">
                              {formatClienteName(selectedCliente)}
                            </p>
                            {selectedCliente.cuit && (
                              <p className="text-sm text-muted-foreground">
                                CUIT: {selectedCliente.cuit}
                              </p>
                            )}
                          </div>
                        ) : (
                          <div>
                            <p className="font-medium text-muted-foreground">
                              Seleccionar cliente
                            </p>
                            <p className="text-sm text-muted-foreground">
                              Debe elegir un cliente específico
                            </p>
                          </div>
                        )}
                      </div>
                      <div className="flex items-center gap-2">
                        <User className="h-4 w-4" />
                        <ChevronDown className="h-4 w-4" />
                      </div>
                    </Button>
                    {!selectedCliente && (
                      <p className="text-sm text-red-500 mt-1">
                        * Es obligatorio seleccionar un cliente
                      </p>
                    )}
                  </div>
                </div>

                <div className="space-y-4">{renderTipoFacturaSelect()}</div>
              </div>

              {/* Segunda fila: Pago inicial y Porcentaje de IVA */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-4">
                  <div>
                    <Label htmlFor="pagoInicial">Pago inicial (opcional)</Label>
                    <Input
                      id="pagoInicial"
                      type="number"
                      step="0.01"
                      min="0"
                      max={total}
                      value={
                        formData.pagoInicial
                          ? formData.pagoInicial.toString()
                          : ""
                      }
                      onChange={(e) =>
                        setFormData({
                          ...formData,
                          pagoInicial: e.target.value
                            ? parseFloat(e.target.value)
                            : undefined,
                        })
                      }
                      placeholder="0.00"
                      className="w-full"
                    />
                    {formData.pagoInicial && formData.pagoInicial > total && (
                      <p className="text-sm text-red-500">
                        El pago inicial no puede ser mayor al total
                      </p>
                    )}
                    {formData.pagoInicial && formData.pagoInicial > 0 && (
                      <p className="text-sm text-blue-600">
                        <strong>Saldo pendiente:</strong>{" "}
                        {formatCurrency(total - formData.pagoInicial)}
                      </p>
                    )}
                  </div>
                </div>

                <div className="space-y-4">
                  <div>
                    <Label htmlFor="porcentajeIva">Porcentaje de IVA (%)</Label>
                    <Input
                      id="porcentajeIva"
                      type="number"
                      step="0.01"
                      min="0"
                      max="100"
                      value={
                        formData.porcentajeIva !== undefined
                          ? formData.porcentajeIva.toString()
                          : ""
                      }
                      onChange={(e) =>
                        setFormData({
                          ...formData,
                          porcentajeIva: e.target.value
                            ? parseFloat(e.target.value)
                            : 0,
                        })
                      }
                      placeholder="0"
                      className="w-full"
                    />
                    <p className="text-sm text-muted-foreground">
                      {formData.tipoFactura === "A" ||
                      formData.tipoFactura === "C"
                        ? "Por defecto: 21% para facturas A y C"
                        : formData.tipoFactura === "remito"
                        ? "Por defecto: 0% para remitos"
                        : "Ingrese el porcentaje de IVA"}
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Productos */}
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold">Productos</h3>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => setShowProductoDialog(true)}
                  className="bg-emerald-gradient text-white hover:text-white"
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Agregar Producto
                </Button>
              </div>

              {/* Tabla de productos */}
              {formData.detalles.length > 0 && (
                <div className="border border-[#A7A7A7] rounded-xl">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Producto</TableHead>
                        <TableHead>Descripción</TableHead>
                        <TableHead className="w-[120px]">Cantidad</TableHead>
                        <TableHead className="w-[120px]">P. Unitario</TableHead>
                        <TableHead className="w-[120px]">Subtotal</TableHead>
                        <TableHead className="w-[50px]"></TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {formData.detalles.map((detalle, index) => (
                        <TableRow key={index}>
                          <TableCell>
                            <div>
                              <p className="font-medium">
                                {detalle.producto?.nombre ||
                                  `Producto ${detalle.productoId}`}
                              </p>
                              <p className="text-sm text-muted-foreground">
                                {detalle.producto?.tipoMedida}
                              </p>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Input
                              value={detalle.descripcion}
                              onChange={(e) =>
                                updateDetalle(
                                  index,
                                  "descripcion",
                                  e.target.value
                                )
                              }
                              className="w-full"
                            />
                          </TableCell>
                          <TableCell>
                            <Input
                              type="number"
                              value={detalle.cantidad}
                              onChange={(e) =>
                                updateDetalle(
                                  index,
                                  "cantidad",
                                  parseFloat(e.target.value) || 0
                                )
                              }
                              min="0"
                              step="0.01"
                              className="w-full"
                            />
                          </TableCell>
                          <TableCell>
                            <Input
                              type="number"
                              value={detalle.precioUnitario}
                              onChange={(e) =>
                                updateDetalle(
                                  index,
                                  "precioUnitario",
                                  parseFloat(e.target.value) || 0
                                )
                              }
                              min="0"
                              step="0.01"
                              className="w-full"
                            />
                          </TableCell>
                          <TableCell className="text-right font-medium">
                            {formatCurrency(detalle.subtotal)}
                          </TableCell>
                          <TableCell>
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              onClick={() => removeDetalle(index)}
                              className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}

              {/* Totales */}
              {formData.detalles.length > 0 && (
                <div className="flex justify-end">
                  <div className="w-80 space-y-2 p-4 border rounded-lg bg-muted/30">
                    <div className="flex items-center gap-2 mb-3">
                      <Calculator className="h-4 w-4" />
                      <span className="font-semibold">Resumen</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Subtotal:</span>
                      <span>{formatCurrency(subtotal)}</span>
                    </div>
                    {impuestos > 0 && (
                      <div className="flex justify-between">
                        <span>IVA ({formData.porcentajeIva || 0}%):</span>
                        <span>{formatCurrency(impuestos)}</span>
                      </div>
                    )}
                    <div className="border-t pt-2">
                      <div className="flex justify-between font-bold text-lg">
                        <span>Total:</span>
                        <span>{formatCurrency(total)}</span>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Observaciones */}
            <div className="space-y-2">
              <Label htmlFor="observaciones">Observaciones</Label>
              <Textarea
                id="observaciones"
                value={formData.observaciones}
                onChange={(e) =>
                  setFormData({ ...formData, observaciones: e.target.value })
                }
                placeholder="Observaciones adicionales..."
                rows={3}
              />
            </div>

            {/* Botones */}
            <div className="flex justify-end gap-3 pt-4 border-t">
              <Button type="button" variant="outline" onClick={onClose}>
                <X className="h-4 w-4 mr-2" />
                Cancelar
              </Button>
              <Button
                onClick={handleSubmit}
                disabled={loading}
                className="bg-emerald-gradient text-white hover:text-white"
              >
                <Save className="h-4 w-4 mr-2" />
                {loading
                  ? "Guardando..."
                  : mode === "create"
                  ? "Crear Factura"
                  : "Actualizar"}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Diálogos */}
      <ClienteSelectorDialog
        open={showClienteDialog}
        onClose={() => setShowClienteDialog(false)}
        onSelect={handleClienteSelect}
        selectedClienteId={selectedCliente?.id}
      />

      <ProductoSelectorDialog
        open={showProductoDialog}
        onClose={() => setShowProductoDialog(false)}
        onSelect={handleProductoSelect}
        excludeProductIds={excludeProductIds}
      />

      <CantidadProductoDialog
        open={showCantidadDialog}
        onClose={() => {
          setShowCantidadDialog(false);
          setSelectedProductoForCantidad(null);
        }}
        onConfirm={handleConfirmarCantidad}
        producto={selectedProductoForCantidad}
      />
    </>
  );
};

export default FacturaForm;
