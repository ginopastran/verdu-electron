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
import { useFacturaTicketPrinting } from "@/hooks/useFacturaTicketPrinting";
import { format } from "date-fns";
import { es } from "date-fns/locale";
import { calcularPrecioVisualConIVA } from "@/utils/ivaHelpers";

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
  ivaIncluido?: boolean;
  ivaPorcentaje?: number | null;
}

interface ListaPrecio {
  id: number;
  nombre: string;
  descripcion?: string;
  activa: boolean;
  _count: {
    productos: number;
  };
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
  fechaVencimiento?: string;
  // 🗑️ Eliminado: porcentajeIva - el backend calcula automáticamente
  // 🆕 CAMPO PARA LISTA DE PRECIOS
  listaPrecioId?: number;
}

interface ProductoInicial {
  id: number;
  nombre: string;
  precio: number;
  cantidad: number;
  subtotal: number;
}

interface FacturaFormProps {
  factura?: any;
  mode?: "create" | "edit";
  onClose: () => void;
  onSuccess: () => void;
  isOpen?: boolean;
  productosIniciales?: ProductoInicial[];
}

const FacturaForm: React.FC<FacturaFormProps> = ({
  factura,
  mode = "create",
  onClose,
  onSuccess,
  isOpen = true,
  productosIniciales = [],
}) => {
  const { businessInfo, loading: businessLoading } = useBusinessInfo(
    import.meta.env.VITE_API_URL || "http://localhost:3000",
    import.meta.env.VITE_APP_ID || null
  );
  const { handleFacturaTicketPrinting } = useFacturaTicketPrinting();
  const [loading, setLoading] = useState(false);
  const [selectedCliente, setSelectedCliente] = useState<Cliente | null>(null);
  const [showClienteDialog, setShowClienteDialog] = useState(false);
  const [showProductoDialog, setShowProductoDialog] = useState(false);
  const [showCantidadDialog, setShowCantidadDialog] = useState(false);
  const [selectedProductoForCantidad, setSelectedProductoForCantidad] =
    useState<Producto | null>(null);
  // 🆕 ESTADOS PARA LISTAS DE PRECIOS
  const [listasPrecios, setListasPrecios] = useState<ListaPrecio[]>([]);
  const [selectedListaPrecio, setSelectedListaPrecio] =
    useState<ListaPrecio | null>(null);
  const [preciosLista, setPreciosLista] = useState<Record<number, number>>({});

  // 🆕 Estado para persistir la lista de precios entre sesiones del diálogo
  const [persistedListaPrecioId, setPersistedListaPrecioId] = useState<
    number | undefined
  >(undefined);

  const [formData, setFormData] = useState<FormData>({
    clienteId: factura?.clienteId,
    tipoFactura: factura?.tipoFactura || "remito",
    observaciones: factura?.observaciones || "",
    detalles: factura?.detalles || [],
    // 🆕 CAMPOS PARA CUENTA CORRIENTE
    pagoInicial: factura?.pagoInicial || undefined,
    fechaVencimiento: factura?.fechaVencimiento || undefined,
    // 🆕 CAMPO PARA LISTA DE PRECIOS - usar persistedListaPrecioId si no hay factura
    listaPrecioId: factura?.listaPrecioId || persistedListaPrecioId,
  });

  useEffect(() => {
    if (formData.clienteId) {
      fetchClienteById(formData.clienteId);
    }
    // Cargar listas de precios
    fetchListasPrecios();
  }, []);

  // 🆕 useEffect para inicializar formData con lista persistida cuando se abre el diálogo
  useEffect(() => {
    if (isOpen && mode === "create" && !factura && persistedListaPrecioId) {
      setFormData((prev) => ({
        ...prev,
        listaPrecioId: persistedListaPrecioId,
      }));
    }
  }, [isOpen, mode, factura, persistedListaPrecioId]);

  useEffect(() => {
    // Si hay una lista seleccionada en formData, buscarla en las listas cargadas
    if (formData.listaPrecioId && listasPrecios.length > 0) {
      const lista = listasPrecios.find((l) => l.id === formData.listaPrecioId);
      setSelectedListaPrecio(lista || null);
    }
  }, [formData.listaPrecioId, listasPrecios]);

  useEffect(() => {
    // Cargar precios cuando cambie la lista seleccionada
    fetchPreciosLista();
  }, [selectedListaPrecio]);

  // 🆕 NUEVO: Resetear estado del formulario cuando se abra el diálogo
  useEffect(() => {
    if (isOpen && mode === "create") {
      // Resetear completamente el estado del formulario
      setFormData({
        clienteId: undefined,
        tipoFactura: "remito",
        observaciones: "",
        detalles: [],
        pagoInicial: undefined,
        fechaVencimiento: undefined,
        listaPrecioId: persistedListaPrecioId,
      });

      // Resetear otros estados
      setSelectedCliente(null);
      setSelectedListaPrecio(null);

      console.log("🧹 Estado del formulario reseteado para nueva factura");
    }
  }, [isOpen, mode]); // Removido persistedListaPrecioId de las dependencias

  // 🆕 NUEVO: Cargar productos iniciales del carrito (solo una vez al abrir)
  useEffect(() => {
    if (
      isOpen &&
      productosIniciales.length > 0 &&
      mode === "create" &&
      formData.detalles.length === 0
    ) {
      console.log(
        "📄 Cargando productos iniciales del carrito:",
        productosIniciales
      );

      const cargarProductosCompletos = async () => {
        const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
        const detallesIniciales: DetalleFactura[] = [];

        for (const productoInicial of productosIniciales) {
          try {
            const response = await fetch(
              `${API_URL}/api/productos/${productoInicial.id}`
            );
            if (response.ok) {
              const productoCompleto = await response.json();

              // 🔧 Usar precio del carrito inicialmente (se actualizará con updateExistingProductPrices si hay lista)
              const precioFinal = productoInicial.precio;

              detallesIniciales.push({
                productoId: productoInicial.id,
                producto: productoCompleto,
                descripcion: productoCompleto.nombre,
                cantidad: productoInicial.cantidad,
                precioUnitario: precioFinal,
                subtotal: precioFinal * productoInicial.cantidad,
              });
            } else {
              // Fallback si no se puede cargar el producto completo
              const precioFinal = productoInicial.precio;

              detallesIniciales.push({
                productoId: productoInicial.id,
                descripcion: productoInicial.nombre,
                cantidad: productoInicial.cantidad,
                precioUnitario: precioFinal,
                subtotal: precioFinal * productoInicial.cantidad,
              });
            }
          } catch (error) {
            console.log(
              `Error cargando producto ${productoInicial.id}:`,
              error
            );
            // Fallback si hay error
            const precioFinal = productoInicial.precio;

            detallesIniciales.push({
              productoId: productoInicial.id,
              descripcion: productoInicial.nombre,
              cantidad: productoInicial.cantidad,
              precioUnitario: precioFinal,
              subtotal: precioFinal * productoInicial.cantidad,
            });
          }
        }

        setFormData((prev) => ({
          ...prev,
          detalles: detallesIniciales,
        }));

        // 🔧 Si hay una lista de precios seleccionada, actualizar precios después de cargar
        if (selectedListaPrecio && Object.keys(preciosLista).length > 0) {
          setTimeout(() => {
            updateExistingProductPrices(selectedListaPrecio);
          }, 100);
        }
      };

      cargarProductosCompletos();
    }
  }, [isOpen, productosIniciales, mode, formData.detalles.length]);

  // 🆕 NUEVO: Actualizar precios cuando se carguen los precios de lista después de los productos iniciales
  useEffect(() => {
    if (
      formData.detalles.length > 0 &&
      selectedListaPrecio &&
      Object.keys(preciosLista).length > 0
    ) {
      updateExistingProductPrices(selectedListaPrecio);
    }
  }, [preciosLista, selectedListaPrecio]);

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

  const fetchListasPrecios = async () => {
    try {
      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      const response = await fetch(`${API_URL}/api/listas-precios?activa=true`);
      if (response.ok) {
        const data = await response.json();
        setListasPrecios(data.listasPrecios || []);
      }
    } catch (error) {
      console.error("Error al obtener listas de precios:", error);
    }
  };

  const fetchPreciosLista = async () => {
    if (!selectedListaPrecio) {
      setPreciosLista({});
      return;
    }

    try {
      const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
      const response = await fetch(
        `${API_URL}/api/listas-precios/${selectedListaPrecio.id}/productos`
      );
      if (response.ok) {
        const data = await response.json();
        const preciosMap: Record<number, number> = {};
        data.forEach((item: any) => {
          const key =
            typeof item.productoId === "number" ? item.productoId : item.id;
          if (typeof key === "number" && typeof item.precio === "number") {
            preciosMap[key] = item.precio;
          }
        });
        setPreciosLista(preciosMap);
      }
    } catch (error) {
      console.error("Error al cargar precios de lista:", error);
      setPreciosLista({});
    }
  };

  const isProductoEnLista = (productoId: number): boolean => {
    return !!(selectedListaPrecio && preciosLista[productoId] !== undefined);
  };

  const calculateTotals = () => {
    // Función para redondear a 2 decimales
    const round2 = (num: number) => Math.round(num * 100) / 100;

    let subtotalSinIva = 0;
    let totalIva = 0;

    // Si es un remito, no calcular impuestos
    const esRemito = formData.tipoFactura === "remito";

    formData.detalles.forEach((detalle) => {
      const producto = detalle.producto;
      const subtotalDetalle = detalle.subtotal;

      if (esRemito) {
        // Para remitos, siempre usar el subtotal tal como está
        // Los remitos no manejan IVA, independientemente de la configuración
        subtotalSinIva += subtotalDetalle;
        // No calcular IVA para remitos
      } else {
        // Para facturas normales, calcular IVA como antes
        const porcentajeIva = producto?.ivaPorcentaje ?? 21;
        const ivaIncluido =
          producto?.ivaIncluido || businessInfo?.ivaIncluidoEnPrecios || false;

        if (ivaIncluido) {
          // Si el precio incluye IVA, extraer la base imponible
          const factorIva = 1 + porcentajeIva / 100;
          const baseImponible = round2(subtotalDetalle / factorIva);
          const ivaDetalle = round2(subtotalDetalle - baseImponible);

          subtotalSinIva += baseImponible;
          totalIva += ivaDetalle;
        } else {
          // Si el precio NO incluye IVA, calcularlo
          const ivaDetalle = round2(subtotalDetalle * (porcentajeIva / 100));

          subtotalSinIva += subtotalDetalle;
          totalIva += ivaDetalle;
        }
      }
    });

    const subtotal = round2(subtotalSinIva);
    const impuestos = esRemito ? 0 : round2(totalIva);
    const total = round2(subtotal + impuestos);

    return { subtotal, impuestos, total };
  };

  const handleClienteSelect = (cliente: Cliente | null) => {
    setSelectedCliente(cliente);
    setFormData({
      ...formData,
      clienteId: cliente?.id,
    });
  };

  // 🆕 Función para actualizar precios de productos existentes cuando cambia la lista
  const updateExistingProductPrices = async (
    newListaPrecio: ListaPrecio | null
  ) => {
    if (formData.detalles.length === 0) return;

    const API_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";
    const updatedDetalles = await Promise.all(
      formData.detalles.map(async (detalle) => {
        // Obtener el precio base del producto desde la API para asegurar que sea correcto
        let precioBase = detalle.precioUnitario;
        try {
          const productoResponse = await fetch(
            `${API_URL}/api/productos/${detalle.productoId}`
          );
          if (productoResponse.ok) {
            const productoData = await productoResponse.json();
            precioBase = productoData.precio;
          }
        } catch (error) {
          console.log(
            `Error obteniendo precio base del producto ${detalle.productoId}:`,
            error
          );
        }

        let nuevoPrecio = precioBase;

        // Si hay una nueva lista seleccionada, intentar obtener el precio de esa lista
        if (newListaPrecio) {
          try {
            const response = await fetch(
              `${API_URL}/api/listas-precios/${newListaPrecio.id}/productos/${detalle.productoId}`
            );
            if (response.ok) {
              const data = await response.json();
              if (data.precio) {
                nuevoPrecio = data.precio;
                console.log(
                  `Producto ${detalle.productoId} encontrado en lista ${newListaPrecio.nombre}: ${nuevoPrecio}`
                );
              }
            }
          } catch (error) {
            console.log(
              `Producto ${detalle.productoId} no encontrado en lista ${newListaPrecio.nombre}, usando precio base: ${precioBase}`
            );
            nuevoPrecio = precioBase;
          }
        } else {
          // Si no hay lista seleccionada, usar siempre el precio base del producto
          nuevoPrecio = precioBase;
          console.log(
            `Sin lista seleccionada, usando precio base para producto ${detalle.productoId}: ${nuevoPrecio}`
          );
        }

        return {
          ...detalle,
          precioUnitario: nuevoPrecio,
          subtotal: detalle.cantidad * nuevoPrecio,
        };
      })
    );

    setFormData((prevFormData) => ({
      ...prevFormData,
      detalles: updatedDetalles,
      listaPrecioId: newListaPrecio?.id || undefined,
    }));
  };

  const handleListaPrecioSelect = async (value: string) => {
    const listaPrecioId = value === "none" ? null : parseInt(value);
    const lista = listaPrecioId
      ? listasPrecios.find((l) => l.id === listaPrecioId)
      : null;

    setSelectedListaPrecio(lista || null);
    // 🆕 Persistir la selección para futuras sesiones del diálogo
    setPersistedListaPrecioId(listaPrecioId || undefined);

    // 🆕 Actualizar precios de productos existentes
    await updateExistingProductPrices(lista || null);

    console.log(
      "🏷️ Lista de precios seleccionada:",
      lista?.nombre,
      "- Cliente mantenido:",
      selectedCliente?.nombre
    );
  };

  const handleProductoSelect = (producto: Producto) => {
    // En lugar de agregar directamente, abrir el diálogo de cantidad
    setSelectedProductoForCantidad(producto);
    setShowCantidadDialog(true);
    setShowProductoDialog(false);
  };

  const handleConfirmarCantidad = async (
    producto: Producto,
    cantidad: number
  ) => {
    let precioFinal = producto.precio;

    // 🔧 Usar precios ya cargados en lugar de hacer nueva llamada a la API
    if (selectedListaPrecio && preciosLista[producto.id] !== undefined) {
      precioFinal = preciosLista[producto.id];
      console.log(
        `🏷️ Usando precio de lista ${selectedListaPrecio.nombre} para ${producto.nombre}: ${precioFinal}`
      );
    } else {
      console.log(
        `💰 Usando precio base para ${producto.nombre}: ${precioFinal}`
      );
    }

    const nuevoDetalle: DetalleFactura = {
      productoId: producto.id,
      producto,
      descripcion: producto.nombre,
      cantidad: cantidad,
      precioUnitario: precioFinal,
      subtotal: precioFinal * cantidad,
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

  // Función para establecer tipo de factura
  const setTipoFactura = (tipoFactura: string) => {
    setFormData({
      ...formData,
      tipoFactura,
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
        
        // 🆕 Siempre es cuenta corriente cuando se crea desde FacturaForm
        // FacturaForm es específicamente para crear facturas/remitos de cuenta corriente
        const esCuentaCorriente = true;
        
        dataToSend = {
          clienteId: formData.clienteId,
          tipoFactura: formData.tipoFactura,
          observaciones: formData.observaciones,
          listaPrecioId: formData.listaPrecioId,
          pagoInicial: formData.pagoInicial || 0,
          esConsumidorFinal: false, // Siempre false porque cliente es obligatorio
          sucursalId: businessInfo?.sucursalId || 1, // Campo obligatorio para AFIP
          productos: formData.detalles.map((detalle) => ({
            productoId: detalle.productoId,
            cantidad: detalle.cantidad,
            precio: detalle.precioUnitario,
          })),
          // 🆕 Campos de cuenta corriente para facturas AFIP - siempre true
          esCuentaCorriente: true,
          ...(formData.fechaVencimiento && {
            fechaVencimiento: formData.fechaVencimiento,
          }),
        };
      } else {
        // Usar endpoint estándar para remitos o cuando AFIP no está habilitado
        url =
          mode === "create"
            ? `${API_URL}/api/facturas`
            : `${API_URL}/api/facturas/${factura.id}`;
        
        // 🆕 Siempre es cuenta corriente cuando se crea desde FacturaForm
        // FacturaForm es específicamente para crear facturas/remitos de cuenta corriente
        const esCuentaCorriente = true;
        
        dataToSend = {
          clienteId: formData.clienteId,
          tipoFactura: formData.tipoFactura,
          observaciones: formData.observaciones,
          listaPrecioId: formData.listaPrecioId,
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
          // 🆕 Campos de cuenta corriente para remitos - siempre true
          esCuentaCorriente: true,
          ...(formData.fechaVencimiento && {
            fechaVencimiento: formData.fechaVencimiento,
          }),
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

      // ✅ MOSTRAR TOAST DE ÉXITO DE CREACIÓN DE FACTURA
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

      // 🖨️ INTENTAR IMPRIMIR TICKET - SIEMPRE PARA FACTURAS REMITO TAMBIÉN
      if (mode === "create") {
        try {
          const facturaParaTicket = {
            id: result.factura?.id || result.id,
            numero: result.factura?.numero || result.numero,
            tipoFactura: formData.tipoFactura,
            fecha: new Date().toISOString(),
            cliente: selectedCliente,
            detalles: formData.detalles,
            subtotal,
            impuestos,
            total,
            observaciones: formData.observaciones,
            afip: result.afip || null,
          };

          console.log(
            `🖨️ [FacturaForm] Imprimiendo ticket para factura:`,
            facturaParaTicket
          );

          // ✅ MOSTRAR TOAST DE ESTADO DE IMPRESIÓN
          const printingToastId = toast.loading("Imprimiendo ticket...");

          const printSuccess = await handleFacturaTicketPrinting(
            facturaParaTicket,
            API_URL,
            import.meta.env.VITE_APP_ID || null
          );

          toast.dismiss(printingToastId);

          if (printSuccess) {
            toast.success("Ticket impreso correctamente");
          } else {
            toast.error("Error al imprimir el ticket");
          }
        } catch (printError) {
          console.error("Error al imprimir ticket:", printError);
          toast.error("Error de conexión con la impresora");
        }
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
          onValueChange={(value) => setTipoFactura(value)}
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
      <Dialog open={isOpen} onOpenChange={onClose}>
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
                  
                  {/* Campo de fecha de vencimiento para cuenta corriente (remitos y facturas AFIP) */}
                  {formData.pagoInicial && formData.pagoInicial > 0 && formData.pagoInicial < total && (
                    <div>
                      <Label htmlFor="fechaVencimiento">Fecha de vencimiento (opcional)</Label>
                      <Input
                        id="fechaVencimiento"
                        type="date"
                        value={
                          formData.fechaVencimiento
                            ? new Date(formData.fechaVencimiento).toISOString().split('T')[0]
                            : ""
                        }
                        onChange={(e) =>
                          setFormData({
                            ...formData,
                            fechaVencimiento: e.target.value
                              ? new Date(e.target.value).toISOString()
                              : undefined,
                          })
                        }
                        className="w-full"
                      />
                    </div>
                  )}
                </div>

                <div className="space-y-4">
                  <div>
                    <Label htmlFor="listaPrecio">
                      Lista de Precios (opcional)
                    </Label>
                    <Select
                      value={selectedListaPrecio?.id.toString() || "none"}
                      onValueChange={handleListaPrecioSelect}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Seleccionar lista de precios" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">
                          Sin lista específica (precios base)
                        </SelectItem>
                        {listasPrecios.map((lista) => (
                          <SelectItem
                            key={lista.id}
                            value={lista.id.toString()}
                          >
                            {lista.nombre} ({lista._count.productos} productos)
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <p className="text-sm text-muted-foreground mt-1">
                      {selectedListaPrecio
                        ? `Usando precios de "${selectedListaPrecio.nombre}". Si un producto no está en esta lista, se usará el precio base.`
                        : "Se usarán los precios base de los productos."}
                    </p>
                  </div>
                </div>
              </div>

              {/* Nueva fila: Información sobre cálculo automático de IVA */}
              <div className="grid grid-cols-1 gap-6">
                <div className="space-y-4">
                  <div className="p-4 bg-blue-50 border border-blue-200 rounded-2xl">
                    <h4 className="font-medium text-blue-800 mb-2">
                      📊 Cálculo Automático de IVA
                    </h4>
                    <div className="text-sm text-blue-700 space-y-1">
                      <p>
                        • <strong>IVA automático:</strong> Se calcula según la
                        configuración de cada producto
                      </p>
                      <p>
                        • <strong>Alícuotas soportadas:</strong> 0%, 10.5%, 21%
                        y 27%
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
                          <TableHead className="w-[120px]">
                            P. Unitario
                          </TableHead>
                          <TableHead className="w-[120px]">Subtotal</TableHead>
                          <TableHead className="w-[50px]"></TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {formData.detalles.map((detalle, index) => (
                          <TableRow key={index}>
                            <TableCell>
                              <div className="space-y-2">
                                <div>
                                  <p className="font-medium">
                                    {detalle.producto?.nombre ||
                                      `Producto ${detalle.productoId}`}
                                  </p>
                                  <p className="text-sm text-muted-foreground">
                                    {detalle.producto?.tipoMedida}
                                  </p>
                                </div>
                                {isProductoEnLista(detalle.productoId) && (
                                  <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-emerald-gradient text-white shadow-sm gap-1">
                                    <FileText className="size-4" />{" "}
                                    {selectedListaPrecio?.nombre}
                                  </span>
                                )}
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
                      {formData.tipoFactura !== "remito" && impuestos > 0 && (
                        <div className="flex justify-between">
                          <span>IVA:</span>
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
        selectedListaPrecio={selectedListaPrecio}
        preciosLista={preciosLista}
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
