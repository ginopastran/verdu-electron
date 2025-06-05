import { useState, useEffect, useRef } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Sun,
  Moon,
  Calendar,
  History,
  Store,
  Wallet,
  QrCode,
  Receipt,
  CreditCard,
} from "lucide-react";

// Hooks
import { useAuth } from "@/contexts/AuthContext";
import { useCartState } from "@/hooks/useCartState";
import { usePaymentProcessing } from "@/hooks/usePaymentProcessing";
import { useScaleWeight } from "@/hooks/useScaleWeight";
import { AvailableProduct } from "@/hooks/useProductSearch";

// Componentes
import { UserMenu } from "@/components/user-menu";
import {
  AddProductDialog,
  CartItem,
  CartSummary,
  CartTabs,
  ProductSearch,
  PaymentDialog,
  ManualQrDialog,
} from "./shopping-cart";

// Componentes de diálogo
import { CashPaymentDialog, CancelDialog } from "./shopping-cart/dialogs";

// Importar las imágenes como recursos desde assets
import iselinLogo from "../assets/iselin-logo.png";
import andextechLogo from "../assets/andextech-black.png";

// Importar el nuevo componente de diálogo de órdenes recientes
import { RecentOrdersDialog } from "./RecentOrdersDialog";

// Tipos
// Eliminada la importación duplicada de History

interface Product {
  id: number;
  cartId: string;
  name: string;
  quantity: number;
  unit: string;
  pricePerUnit: number;
  subtotal: number;
  costo: number;
}

export default function ShoppingCartRefactored() {
  const { user, logout } = useAuth();
  const API_URL = import.meta.env.VITE_API_URL;
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Estados para búsqueda y productos
  const [searchQuery, setSearchQuery] = useState("");
  const [availableProducts, setAvailableProducts] = useState<
    AvailableProduct[]
  >([]);
  const [showResults, setShowResults] = useState(false);
  const [searchResults, setSearchResults] = useState<AvailableProduct[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);

  // Estados para diálogos y UI
  const [dialogOpen, setDialogOpen] = useState(false);
  const [cancelDialogOpen, setCancelDialogOpen] = useState(false);
  const [paymentDialogOpen, setPaymentDialogOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] =
    useState<AvailableProduct | null>(null);

  // Estados para el diálogo de órdenes recientes
  const [ordersDialogOpen, setOrdersDialogOpen] = useState(false);

  // Estado para información del negocio
  const [businessInfo, setBusinessInfo] = useState<any>(null);

  // Estados para cierre de caja
  const [closingDialogOpen, setClosingDialogOpen] = useState(false);
  const [closeResultDialogOpen, setCloseResultDialogOpen] = useState(false);
  const [closeResultData, setCloseResultData] = useState<any>(null);
  const [isClosing, setIsClosing] = useState(false);

  // Estados para código de barras
  const [lastInputTime, setLastInputTime] = useState<number>(0);
  const [barcodeBuffer, setBarcodeBuffer] = useState<string>("");

  // Estados para peso manual/automático
  const [useManualWeight, setUseManualWeight] = useState(false);
  const weight = useScaleWeight();

  // Estados para pantallas múltiples
  const [activeScreen, setActiveScreen] = useState(0);
  const [screens, setScreens] = useState<{ id: number; items: Product[] }[]>([
    { id: 0, items: [] },
  ]);
  const [deleteScreenDialogOpen, setDeleteScreenDialogOpen] = useState(false);
  const [screenToDelete, setScreenToDelete] = useState<number | null>(null);

  // Protección contra adiciones duplicadas
  const [isAddingToCart, setIsAddingToCart] = useState(false);
  const lastAddRequestRef = useRef<string>("");

  // Agregar estados para QR
  const [qrDialogOpen, setQrDialogOpen] = useState(false);
  const [splitPaymentDialogOpen, setSplitPaymentDialogOpen] = useState(false);

  // Estados de procesamiento de pago (manejados en este componente)
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<
    string | null
  >(null);

  // Utilizar los hooks
  const cartState = useCartState();

  const appId =
    window.electron?.process?.argv
      ?.find((arg) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null;

  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  const paymentProcessor = usePaymentProcessing({
    user,
    API_URL,
    appId,
    clearCart: cartState.clearCart,
    calculateTotal: cartState.calculateTotal,
    setPaymentDialogOpen: setPaymentDialogOpen,
    setQrDialogOpen: setQrDialogOpen,
    setSplitPaymentDialogOpen: setSplitPaymentDialogOpen,
  });

  // Cargar información del negocio
  useEffect(() => {
    const fetchBusinessInfo = async () => {
      try {
        console.log("🏢 Iniciando carga de información del negocio");
        const response = await fetch(
          `${API_URL}/api/business/${import.meta.env.VITE_BUSINESS_ID}`,
          {
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
          }
        );

        if (!response.ok) {
          console.error(
            "❌ Error en la respuesta al cargar información del negocio:",
            response.status
          );
          throw new Error("Error al cargar información del negocio");
        }

        const data = await response.json();
        console.log("✅ Información del negocio cargada:", data);

        // Verificar si tiene configuración de sistema de pago
        if (!data.sistemaPago) {
          console.warn(
            "⚠️ El negocio no tiene configurado sistemaPago, estableciendo por defecto"
          );
          data.sistemaPago = "redondeo"; // Establecer valor por defecto
        }

        setBusinessInfo(data);
      } catch (error) {
        console.error("❌ Error al cargar información del negocio:", error);
        // En caso de error, establecer un valor por defecto para evitar problemas
        setBusinessInfo({ sistemaPago: "redondeo" });
      }
    };

    // Ejecutar la carga de información
    fetchBusinessInfo();
  }, [API_URL, appId]);

  // Cargar productos al montar el componente
  useEffect(() => {
    fetchProducts();
  }, [API_URL, appId]);

  // Comportamiento para las teclas globales
  useEffect(() => {
    const handleGlobalKeyPress = (e: KeyboardEvent) => {
      // Handle F4 for logout
      if (e.key === "F4") {
        e.preventDefault();
        handleLogout();
        return;
      }

      // Para el diálogo de pago
      if (paymentDialogOpen) {
        // Si ya se está procesando o hay un método seleccionado, ignorar teclas
        if (isProcessingPayment || selectedPaymentMethod) {
          return;
        }
        switch (e.key) {
          case "1":
            e.preventDefault();
            handlePayment("qr");
            break;
          case "2":
            e.preventDefault();
            handlePayment("tarjeta");
            break;
          case "3":
            e.preventDefault();
            handlePayment("efectivo");
            break;
          case "4":
            e.preventDefault();
            handlePayment("split");
            break;
        }
      } else {
        // Teclas para el carrito, siempre deben funcionar independientemente del foco
        switch (e.key) {
          case "F1":
            e.preventDefault();
            if (cartState.getCurrentItems().length === 0) {
              toast.error("No hay productos en el carrito", {
                description: "El carrito ya está vacío",
              });
              return;
            }
            handleCancelClick();
            break;
          case "F2":
            e.preventDefault();
            if (cartState.getCurrentItems().length === 0) {
              toast.error("No hay productos en el carrito", {
                description: "Agrega al menos un producto antes de continuar",
              });
              return;
            }
            handlePaymentClick();
            break;
        }
      }
    };

    window.addEventListener("keydown", handleGlobalKeyPress);
    return () => window.removeEventListener("keydown", handleGlobalKeyPress);
  }, [
    paymentDialogOpen,
    cartState.getCurrentItems,
    cartState.calculateTotal,
    paymentProcessor.isProcessingPayment,
    paymentProcessor.selectedPaymentMethod,
  ]);

  // Limpiar intervalos al desmontar o cuando cambia el estado del diálogo QR
  useEffect(() => {
    if (!qrDialogOpen) {
      paymentProcessor.cleanupPolling();
    }
    return () => {
      paymentProcessor.cleanupPolling();
    };
  }, [qrDialogOpen]);

  // Fetch products from API
  const fetchProducts = async () => {
    try {
      const response = await fetch(`${API_URL}/api/productos/all`, {
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
      });

      if (!response.ok) {
        throw new Error("Error al cargar productos");
      }

      const data = await response.json();
      console.log("✅ Productos cargados:", data);
      setAvailableProducts(data);
    } catch (error) {
      console.error("Error al cargar productos:", error);
      toast.error("Error al cargar los productos");
    }
  };

  // Handler para seleccionar un producto
  const handleProductSelect = (product: AvailableProduct) => {
    if (!product) return; // No proceder si no hay producto seleccionado

    console.log("Producto seleccionado:", product.name);
    setSelectedProduct(product);
    setDialogOpen(true);
  };

  // Handler para agregar un producto al carrito
  const handleAddToCart = (product: any) => {
    cartState.addToCart(product);
    setDialogOpen(false);
    setSelectedProduct(null);

    // Enfocar el input de búsqueda
    setTimeout(() => {
      if (searchInputRef.current) {
        searchInputRef.current.focus();
      }
    }, 100);
  };

  // Handler para mostrar diálogo de cancelación
  const handleCancelClick = () => {
    const currentItems = cartState.getCurrentItems();
    if (currentItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "El carrito ya está vacío",
      });
      return;
    }
    setCancelDialogOpen(true);
  };

  // Handler para cancelar el carrito
  const handleCancelCart = () => {
    cartState.clearCart();
    setCancelDialogOpen(false);

    // Enfocar el input de búsqueda
    setTimeout(() => {
      searchInputRef.current?.focus();
    }, 0);

    toast.success("Carrito cancelado");
  };

  // Handler para mostrar diálogo de pago
  const handlePaymentClick = () => {
    const currentItems = cartState.getCurrentItems();
    if (currentItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de continuar",
      });
      return;
    }
    setPaymentDialogOpen(true);
  };

  // Handler para seleccionar método de pago
  const handlePayment = async (method: string) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    console.log("🔄 handlePayment llamado con método:", method);

    // Prevenir procesamiento duplicado usando estado local antes de cualquier acción
    if (isProcessingPayment) {
      console.log("⚠️ Procesamiento bloqueado - ya está procesando");
      return;
    }

    // Si es efectivo, manejar con el sistema de redondeo. Pasa el control.
    if (method === "efectivo") {
      console.log("💰 Seleccionando efectivo - businessInfo:", businessInfo);
      setIsProcessingPayment(true); // Marcar como procesando antes de pasar el control
      setSelectedPaymentMethod("efectivo");
      paymentProcessor.handleCashPayment(businessInfo); // Este hook abrirá su propio diálogo
      setPaymentDialogOpen(false); // Cerrar este diálogo inmediatamente
      return;
    }

    // Si es pago dividido, preparar pago mixto. Pasa el control.
    if (method === "split") {
      console.log("🔄 Iniciando pago mixto desde ShoppingCartRefactored");
      paymentProcessor.handleSplitPayment(); // Este hook abrirá el diálogo de Pago Mixto y manejará su estado

      // Cerrar el diálogo principal después de un pequeño delay para asegurar que el split dialog se abre
      setTimeout(() => {
        setPaymentDialogOpen(false);
      }, 100);
      return;
    }

    // Marcar como procesando y establecer método seleccionado localmente SOLO para los flujos manejados aquí
    setIsProcessingPayment(true);
    setSelectedPaymentMethod(method);

    // Si es QR, generar el QR
    if (method === "qr") {
      console.log("🔄 Verificando configuración de Mercado Pago...");
      console.log("🔄 businessInfo:", businessInfo);
      console.log("🔄 mpEnabled:", businessInfo?.mpEnabled);

      // Si MP está deshabilitado, procesar como transferencia directamente
      if (businessInfo?.mpEnabled === false) {
        console.log(
          "🔄 MP deshabilitado - procesando como transferencia directa"
        );

        // Preparar datos de la orden
        const currentItems = cartState.getCurrentItems();
        const orderItems = currentItems.map((item) => ({
          productoId: item.id,
          cantidad: item.quantity,
          subtotal: Number(item.subtotal.toFixed(2)),
          precioHistorico: item.pricePerUnit,
          costo: Number(item.costo),
          nombre: item.name,
        }));

        const orderData = {
          metodoPago: "qr",
          total: Number(cartState.calculateTotal().toFixed(2)),
          items: orderItems,
          vendedorId: user.id,
          sucursalId: user.sucursalId,
          vendedor: user.nombre,
          estado: "COMPLETADA", // Añadir estado COMPLETADA
          createdAt: new Date().toISOString(),
        };

        try {
          // Mostrar toast de carga ANTES de la llamada a la API
          const processingToastId = toast.loading("Procesando orden...");

          // Crear la orden
          const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
            body: JSON.stringify(orderData),
          });

          if (!orderResponse.ok) {
            // Intentar leer el error del cuerpo de la respuesta
            const errorData = await orderResponse.json().catch(() => ({}));
            console.error("❌ Error al crear la orden API:", errorData);
            throw new Error(errorData.message || "Error al crear la orden");
          }

          // Ocultar toast de carga y mostrar toast de impresión
          toast.dismiss(processingToastId);

          // Mostrar toast de carga para la impresión ANTES de imprimir
          const printingToastId = toast.loading("Imprimiendo ticket...");

          // Imprimir ticket usando la función reutilizable LOCAL
          await handleTicketPrinting(orderData);

          // Cerrar el toast de carga de impresión
          toast.dismiss(printingToastId);

          // handleTicketPrinting ya muestra toast de éxito o error si falla. No necesitamos uno adicional aquí.

          // Limpiar carrito y estados locales
          cartState.clearCart();
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);

          // Cerrar el diálogo de pago principal LOCAL
          setPaymentDialogOpen(false);

          toast.success("Orden completada exitosamente");
        } catch (error: any) {
          console.error("❌ Error en flujo QR/MP deshabilitado:", error);
          toast.error(`Error al procesar la orden: ${error.message}`);
          // Limpiar estados locales también en caso de error
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
          setPaymentDialogOpen(false); // Cerrar el diálogo de pago principal también en caso de error
        }

        return;
      }

      // Si MP está habilitado, generar el QR. Pasa el control.
      paymentProcessor.generateQRPayment(cartState.getCurrentItems()); // Este hook abrirá el diálogo QR y manejará su estado
      setPaymentDialogOpen(false); // Cerrar este diálogo inmediatamente
      return;
    }

    // Para otros métodos (tarjeta), procesar directamente
    // Llamar al procesador de pagos para el flujo general
    await paymentProcessor.processPayment(
      method,
      Number(cartState.calculateTotal().toFixed(2)),
      cartState.getCurrentItems()
    );

    // El procesador de pagos limpia sus propios estados y cierra el diálogo principal (si se le pasó la setter)
    // en caso de éxito o error en su flujo interno. No necesitamos limpiar los estados LOCALES aquí
    // ni cerrar el diálogo, ya que eso lo hace el hook si se le pasó setPaymentDialogOpen.
  };

  // Confirmar pago redondeado en efectivo
  const confirmRoundedPayment = () => {
    // Usar estado local para prevenir duplicación
    if (isProcessingPayment) {
      return;
    }
    // Marcar como procesando localmente antes de llamar al hook
    setIsProcessingPayment(true);

    // Llamar al procesador de pagos. El hook gestionará su propio estado interno
    // y al finalizar, llamará a setPaymentDialogOpen(false) y reseteará sus estados.
    paymentProcessor.processPayment(
      "efectivo",
      paymentProcessor.roundedAmount,
      cartState.getCurrentItems()
    );
    paymentProcessor.setRoundedAmountDialogOpen(false);
  };

  // Cerrar sesión
  const handleLogout = () => {
    toast.success("Cerrando sesión...");
    setTimeout(() => {
      logout();
    }, 1000);
  };

  // Manejadores para pantallas múltiples
  const handleAddScreen = () => {
    const result = cartState.addScreen();
    if (!result) {
      toast.error("No se pueden crear más de 4 pantallas");
    }
  };

  const handleDeleteScreen = (screenId: number) => {
    if (cartState.screens.length <= 1) {
      toast.error("No se puede eliminar la última pantalla");
      return;
    }

    // Mostrar un confirm
    if (window.confirm("¿Seguro que deseas eliminar esta pantalla?")) {
      const result = cartState.deleteScreen(screenId);
      if (result) {
        toast.success("Pantalla eliminada correctamente");
      }
    }
  };

  // Función para formatear fechas en zona horaria Argentina
  const formatFechaArgentina = (fecha: string | Date) => {
    const fechaObj = typeof fecha === "string" ? new Date(fecha) : fecha;
    const fechaArg = new Date(fechaObj.getTime() + 3 * 60 * 60 * 1000);
    return fechaArg.toLocaleString("es-AR", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  };

  // Función para manejar cierre de caja
  const handleClosing = async (period: string) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar el cierre");
      return;
    }

    if (!user.permisos?.cierreDeCajaEnabled) {
      toast.error("No tienes permiso para realizar cierres de caja");
      setClosingDialogOpen(false);
      return;
    }

    setIsClosing(true);

    try {
      const getUTCDate = (hoursArg: number) => {
        const date = new Date();
        date.setUTCHours(hoursArg + 3, 0, 0, 0);
        return date.toISOString();
      };

      let startDate;
      if (period === "mañana") {
        startDate = getUTCDate(6);
      } else if (period === "tarde") {
        startDate = getUTCDate(12);
      } else {
        startDate = getUTCDate(0);
      }

      console.log(`🕒 Fecha inicio (${period}) UTC:`, startDate);
      console.log(
        `🕒 Fecha inicio (${period}) hora Argentina:`,
        formatFechaArgentina(startDate)
      );

      const closingData = {
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        fechaInicio: startDate,
        fechaCierre: new Date().toISOString(),
        periodo: period,
      };

      console.log("🔄 Enviando solicitud de cierre con datos:", closingData);

      const cierreResponse = await fetch(`${API_URL}/api/cierres`, {
        method: "POST",
        headers,
        body: JSON.stringify(closingData),
      });

      const responseData = await cierreResponse.json();

      if (!cierreResponse.ok) {
        if (responseData.error === "ERROR_CIERRE_MAÑANA_REQUERIDO") {
          toast.error(
            "No puedes realizar un cierre de tarde sin haber realizado el cierre de mañana del día actual.",
            {
              duration: 5000,
              description: "Primero debes realizar el cierre de mañana",
            }
          );
        } else {
          throw new Error(
            responseData.message || "Error al registrar el cierre"
          );
        }
        return;
      }

      const cierreData = await responseData;
      console.log("✅ Datos de cierre recibidos:", cierreData);

      try {
        const { ipcRenderer } = window.require("electron");

        // ====== SIMULACIÓN DEL TICKET DE CIERRE ======
        console.log("\n====== SIMULACIÓN DEL TICKET DE CIERRE ======");
        console.log("ISELIN II");
        console.log(`CIERRE DE CAJA - ${period.toUpperCase()}`);
        console.log(`Vendedor: ${user.nombre}`);
        console.log(
          `Fecha inicio: ${formatFechaArgentina(cierreData.fechaInicio)}`
        );
        console.log(
          `Fecha cierre: ${formatFechaArgentina(cierreData.fechaCierre)}`
        );
        console.log("-------------------------------------");
        console.log("VENTAS POR MÉTODO DE PAGO:");

        // Mostrar ventas por método de pago
        if (cierreData.ventasPorMetodo) {
          Object.entries(cierreData.ventasPorMetodo).forEach(
            ([metodo, total]) => {
              console.log(
                `${metodo.toUpperCase()}: $${Number(total).toLocaleString()}`
              );
            }
          );
        }

        console.log("-------------------------------------");
        console.log("VENTAS POR VENDEDOR:");

        // Mostrar ventas por vendedor
        if (
          cierreData.ventasPorVendedor &&
          Array.isArray(cierreData.ventasPorVendedor)
        ) {
          cierreData.ventasPorVendedor.forEach((vendedor: any) => {
            console.log(
              `${vendedor.nombre}: $${Number(
                vendedor.totalVentas
              ).toLocaleString()} (${vendedor.cantidadVentas} ventas)`
            );

            // Mostrar métodos de pago por vendedor si existen
            if (vendedor.metodosPago) {
              Object.entries(vendedor.metodosPago).forEach(
                ([metodo, total]) => {
                  console.log(
                    `  ${metodo.toUpperCase()}: $${Number(
                      total
                    ).toLocaleString()}`
                  );
                }
              );
            }
          });
        }

        console.log("-------------------------------------");
        console.log(
          `TOTAL GENERAL: $${Number(
            cierreData.totalVentas
          ).toLocaleString()} (${cierreData.cantidadVentas} ventas)`
        );
        console.log("=====================================\n");

        const result = await ipcRenderer.invoke("print-closing", cierreData);

        if (result.success && !result.printerError) {
          toast.success("Ticket de cierre impreso correctamente");
        }

        toast.success(`Cierre de ${period} realizado correctamente`);
        setClosingDialogOpen(false);
      } catch (printError: any) {
        console.error("Error al imprimir cierre:", printError);
        toast.success(`Cierre de ${period} realizado correctamente`);
        setClosingDialogOpen(false);
      }
    } catch (error: any) {
      toast.error(`Error: ${error.message || "Error al realizar el cierre"}`);
    } finally {
      setIsClosing(false);
    }
  };

  // Función para manejar búsqueda con código de barras
  const handleSearchInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const currentTime = Date.now();
    const value = e.target.value;
    setSearchQuery(value);

    if (currentTime - lastInputTime < 50) {
      setBarcodeBuffer((prev) => prev + value.slice(-1));
    } else {
      setBarcodeBuffer(value);
    }

    setLastInputTime(currentTime);

    if (/^\d{8,13}$/.test(value)) {
      const product = availableProducts.find((p) => p.codigoBarras === value);
      if (product) {
        handleProductSelect(product);
      }
    }
  };

  // Función para confirmar eliminación de pantalla
  const confirmDeleteScreen = () => {
    if (screenToDelete === null) return;

    const newScreens = screens.filter((screen) => screen.id !== screenToDelete);
    setScreens(newScreens);

    if (activeScreen === screenToDelete) {
      setActiveScreen(0);
    }

    setDeleteScreenDialogOpen(false);
    setScreenToDelete(null);
    toast.success("Pantalla eliminada correctamente");
  };

  // Función para manejar la impresión de tickets (necesaria para RecentOrdersDialog)
  const handleTicketPrinting = async (orderData: any) => {
    try {
      // Simular el ticket antes de imprimir
      console.log("\n====== SIMULACIÓN DEL TICKET ======");
      console.log("ISELIN II");
      console.log(`Vendedor: ${orderData.vendedor}`);
      console.log(
        `Fecha: ${formatFechaArgentina(orderData.createdAt || orderData.fecha)}`
      );
      console.log("-----------------------------");
      console.log("PRODUCTO      CANT    PRECIO    TOTAL");
      console.log("-----------------------------");

      // Mostrar productos
      const items = orderData.items || orderData.detalles || [];
      if (items && items.length > 0) {
        items.forEach((item: any) => {
          const nombre = (item.nombre || item.producto?.nombre || "").padEnd(
            12
          );
          const cantidad = (item.cantidad || 0).toString().padStart(8);
          const precio = `$${Number(
            item.precioHistorico || item.precio || 0
          ).toFixed(2)}`.padStart(8);
          const subtotal = `$${Number(item.subtotal || 0).toFixed(2)}`.padStart(
            8
          );
          console.log(`${nombre} ${cantidad} ${precio} ${subtotal}`);
        });
      } else {
        console.log("❌ No hay items en la orden");
      }

      console.log("-----------------------------");
      console.log(`TOTAL: $${Number(orderData.total).toFixed(2)}`);

      // Mostrar método(s) de pago
      if (orderData.pagos && Array.isArray(orderData.pagos)) {
        console.log("\nMÉTODOS DE PAGO:");
        orderData.pagos.forEach((pago: any) => {
          console.log(
            `${pago.metodoPago.toUpperCase()}: $${Number(pago.monto).toFixed(
              2
            )}`
          );
        });
      } else {
        console.log(`\nMétodo de pago: ${orderData.metodoPago?.toUpperCase()}`);
      }

      console.log("\n¡Gracias por su compra!");
      console.log("==============================\n");

      // Intentar imprimir
      const { ipcRenderer } = window.require("electron");
      const result = await ipcRenderer.invoke("print-ticket", orderData);

      if (result.success) {
        toast.success("Ticket impreso correctamente");
      }
      // Si hay error, solo lo logueamos pero no mostramos toast
      else {
        console.error(
          "❌ Error al imprimir (IPC invoke returned false):",
          result.message
        );
        toast.error(
          `Error al imprimir el ticket: ${result.message || "Desconocido"}`
        );
      }
      return result.success;
    } catch (error: any) {
      console.error("❌ Error al imprimir:", error);
      toast.error(
        `Error al imprimir el ticket: ${error.message || "Desconocido"}`
      );
      return false;
    }
  };

  return (
    <div className="min-h-screen bg-white-cream h-screen relative overflow-hidden">
      {/* Background logos */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-0">
        <div className="relative w-full h-full flex flex-col items-center justify-center gap-10">
          <img
            src={iselinLogo}
            alt="Iselin Logo"
            className="w-[540px] opacity-[.07] select-none"
          />
          <img
            src={andextechLogo}
            alt="Andextech Logo"
            className="w-[440px] opacity-[.07] select-none"
          />
        </div>
      </div>

      <div className="w-full mx-auto flex h-full flex-col px-8 py-6 relative z-10">
        {/* Header section */}
        <div className="flex w-full items-center mb-4 gap-2 justify-start">
          <div className="flex items-center gap-1">
            <ProductSearch
              onProductSelect={handleProductSelect}
              inputRef={searchInputRef}
            />
            <CartTabs
              screens={cartState.screens}
              activeScreen={cartState.activeScreen}
              onChangeScreen={cartState.setActiveScreen}
              onAddScreen={handleAddScreen}
              onDeleteScreen={handleDeleteScreen}
            />
          </div>

          <div className="w-full flex justify-end items-center gap-4">
            {/* Botón de Órdenes recientes */}
            <Button
              className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
              onClick={() => setOrdersDialogOpen(true)}
            >
              <History />
              Órdenes
            </Button>

            {user?.permisos?.cierreDeCajaEnabled && (
              <Button
                className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
                onClick={() => setClosingDialogOpen(true)}
              >
                <Store />
                Cierre de caja
              </Button>
            )}

            <UserMenu
              user={{ nombre: user?.nombre || "", email: user?.email || "" }}
            />
          </div>
        </div>

        {/* Items list */}
        <div className="flex-1 space-y-2 overflow-auto mb-4">
          {cartState.getCurrentItems().map((item) => (
            <CartItem
              key={item.cartId}
              item={item}
              onRemove={cartState.removeFromCart}
            />
          ))}
        </div>

        {/* Cart summary */}
        <CartSummary
          total={cartState.calculateTotal()}
          onCancel={handleCancelClick}
          onCheckout={handlePaymentClick}
        />
      </div>

      {/* Dialogs */}
      <AddProductDialog
        isOpen={dialogOpen}
        product={selectedProduct}
        onClose={() => setDialogOpen(false)}
        onAddToCart={handleAddToCart}
      />

      <CancelDialog
        open={cancelDialogOpen}
        onOpenChange={setCancelDialogOpen}
        onConfirm={handleCancelCart}
        isLoading={isProcessingPayment}
      />

      <PaymentDialog
        isOpen={paymentDialogOpen}
        onClose={() => {
          setPaymentDialogOpen(false);
          paymentProcessor.resetPaymentState();
          // También resetear los estados locales del componente
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
          setTimeout(() => searchInputRef.current?.focus(), 100);
        }}
        onSelectPayment={handlePayment}
        isProcessingPayment={isProcessingPayment}
        selectedPaymentMethod={selectedPaymentMethod}
      />

      <CashPaymentDialog
        open={paymentProcessor.roundedAmountDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            paymentProcessor.setRoundedAmountDialogOpen(false);
            paymentProcessor.resetPaymentState();
            // También resetear los estados locales del componente
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
            setTimeout(() => searchInputRef.current?.focus(), 100);
          }
        }}
        isProcessingPayment={isProcessingPayment}
        applyingDiscount={paymentProcessor.applyingDiscount}
        businessInfo={businessInfo}
        originalAmount={paymentProcessor.originalAmount}
        roundedAmount={paymentProcessor.roundedAmount}
        onApplyDiscount={() => {
          if (!businessInfo) return;
          paymentProcessor.setRoundedAmountDialogOpen(false);
          setTimeout(() => {
            paymentProcessor.handleCashPayment(businessInfo, true);
          }, 100);
        }}
        onConfirm={confirmRoundedPayment}
        onCancel={() => {
          paymentProcessor.setRoundedAmountDialogOpen(false);
          paymentProcessor.resetPaymentState();
          // También resetear los estados locales del componente
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
        }}
      />

      {/* Diálogo de cierre de caja */}
      <Dialog
        open={closingDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            setClosingDialogOpen(false);
            setTimeout(() => {
              searchInputRef.current?.focus();
            }, 100);
          } else {
            setClosingDialogOpen(open);
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
          <div className="grid grid-cols-3 gap-4">
            <Button
              onClick={() => handleClosing("mañana")}
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
              onClick={() => handleClosing("tarde")}
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
              onClick={() => handleClosing("todo")}
              className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
              variant="outline"
              disabled={isClosing}
            >
              {isClosing ? (
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900" />
              ) : (
                <Calendar />
              )}
              <span className="text-base">Todo el día (3)</span>
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Diálogo de resultado de cierre */}
      <Dialog
        open={closeResultDialogOpen}
        onOpenChange={setCloseResultDialogOpen}
      >
        <DialogContent className="sm:max-w-[700px]">
          <DialogHeader>
            <DialogTitle>Resultado del Cierre de Caja</DialogTitle>
            <DialogDescription>
              Resumen de ventas del período seleccionado
            </DialogDescription>
          </DialogHeader>

          {closeResultData && (
            <div className="space-y-4 max-h-[70vh] overflow-y-auto">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <h3 className="text-sm font-medium">Período</h3>
                  <p className="text-xl font-bold">
                    {closeResultData.periodo.toUpperCase()}
                  </p>
                </div>
                <div>
                  <h3 className="text-sm font-medium">Fecha</h3>
                  <p>
                    {formatFechaArgentina(closeResultData.fechaInicio)} -{" "}
                    {formatFechaArgentina(closeResultData.fechaCierre)}
                  </p>
                </div>
              </div>

              <div className="rounded-lg border p-4">
                <h3 className="text-lg font-semibold mb-2">Resumen General</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-muted-foreground">
                      Total Ventas
                    </p>
                    <p className="text-2xl font-bold text-emerald-600">
                      ${Number(closeResultData.totalVentas).toLocaleString()}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-muted-foreground">
                      Cantidad de Ventas
                    </p>
                    <p className="text-2xl font-bold">
                      {closeResultData.cantidadVentas}
                    </p>
                  </div>
                </div>
              </div>

              {closeResultData.ventasPorMetodo && (
                <div className="rounded-lg border p-4">
                  <h3 className="text-lg font-semibold mb-2">
                    Ventas por Método de Pago
                  </h3>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Método</TableHead>
                        <TableHead className="text-right">Monto</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {Object.entries(closeResultData.ventasPorMetodo).map(
                        ([metodo, monto]: [string, any]) => (
                          <TableRow key={metodo}>
                            <TableCell className="font-medium capitalize">
                              {metodo}
                            </TableCell>
                            <TableCell className="text-right">
                              ${Number(monto).toLocaleString()}
                            </TableCell>
                          </TableRow>
                        )
                      )}
                    </TableBody>
                  </Table>
                </div>
              )}

              {closeResultData.ventasPorVendedor && (
                <div className="rounded-lg border p-4">
                  <h3 className="text-lg font-semibold mb-2">
                    Desglose por Vendedor
                  </h3>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Vendedor</TableHead>
                        <TableHead>Email</TableHead>
                        <TableHead className="text-right">Total</TableHead>
                        <TableHead className="text-right">Ventas</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {closeResultData.ventasPorVendedor.map(
                        (vendedor: any) => (
                          <TableRow key={vendedor.id}>
                            <TableCell className="font-medium">
                              {vendedor.nombre}
                            </TableCell>
                            <TableCell>{vendedor.email}</TableCell>
                            <TableCell className="text-right">
                              ${Number(vendedor.totalVentas).toLocaleString()}
                            </TableCell>
                            <TableCell className="text-right">
                              {vendedor.cantidadVentas}
                            </TableCell>
                          </TableRow>
                        )
                      )}
                    </TableBody>
                  </Table>
                </div>
              )}

              <DialogFooter>
                <Button onClick={() => setCloseResultDialogOpen(false)}>
                  Cerrar
                </Button>
              </DialogFooter>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Diálogo de QR */}
      <Dialog
        open={qrDialogOpen}
        onOpenChange={(open) => {
          console.log("🔄 QR Dialog onOpenChange:", open);
          if (!open) {
            console.log("🔄 Cerrando diálogo QR, cancelando pago");
            setQrDialogOpen(false);
            paymentProcessor.cancelQRPayment();
          }
        }}
      >
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-xl">
              Escanea el código QR para pagar
            </DialogTitle>
            <DialogDescription>
              Usa la app de Mercado Pago para escanear
            </DialogDescription>
          </DialogHeader>

          <div className="flex flex-col items-center justify-center py-4">
            {paymentProcessor.qrData ? (
              <div className="space-y-4 text-center">
                <img
                  src={paymentProcessor.qrData.qrImageUrl}
                  alt="Código QR de Mercado Pago"
                  className="mx-auto w-64 h-64 border border-gray-200 p-2"
                  onError={(e) => {
                    console.error("Error loading QR image");
                    e.currentTarget.style.display = "none";
                    toast.error("Error al cargar la imagen QR", {
                      description: "Intenta nuevamente o contacta a soporte",
                    });
                  }}
                />

                <div className="font-medium text-lg">
                  {paymentProcessor.qrData.isSplitPayment ? (
                    <div className="space-y-1">
                      <div className="text-sm text-muted-foreground">
                        Pago mixto:
                      </div>
                      <div className="flex justify-between items-center text-base">
                        <span className="flex items-center">
                          <Wallet className="h-4 w-4 mr-1" />
                          Efectivo:
                        </span>
                        <span>
                          $
                          {Number(
                            paymentProcessor.qrData.cashAmount || 0
                          ).toLocaleString()}
                        </span>
                      </div>
                      <div className="flex justify-between items-center font-semibold">
                        <span className="flex items-center">
                          <QrCode className="h-4 w-4 mr-1" />
                          QR:
                        </span>
                        <span>
                          $
                          {Number(
                            paymentProcessor.qrData.monto || 0
                          ).toLocaleString()}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <span>
                      Monto a pagar: $
                      {Number(
                        paymentProcessor.qrData.monto || 0
                      ).toLocaleString()}
                    </span>
                  )}
                </div>

                <div className="space-y-2">
                  <div
                    className={`text-center py-2 px-4 rounded-full font-medium ${
                      paymentProcessor.paymentStatus === "PENDIENTE"
                        ? "bg-yellow-100 text-yellow-800"
                        : paymentProcessor.paymentStatus === "COMPLETADA"
                        ? "bg-green-100 text-green-800"
                        : paymentProcessor.paymentStatus === "CANCELADA"
                        ? "bg-red-100 text-red-800"
                        : "bg-gray-100 text-gray-800"
                    }`}
                  >
                    Estado:{" "}
                    {paymentProcessor.paymentStatus === "PENDIENTE"
                      ? "Esperando pago..."
                      : paymentProcessor.paymentStatus === "COMPLETADA"
                      ? "¡Pago completado!"
                      : paymentProcessor.paymentStatus === "CANCELADA"
                      ? "Pago cancelado"
                      : "Desconocido"}
                    {paymentProcessor.paymentStatus === "PENDIENTE" && (
                      <span className="inline-block ml-2">
                        <div className="animate-pulse w-2 h-2 bg-yellow-500 rounded-full inline-block mx-0.5"></div>
                        <div className="animate-pulse delay-150 w-2 h-2 bg-yellow-500 rounded-full inline-block mx-0.5"></div>
                        <div className="animate-pulse delay-300 w-2 h-2 bg-yellow-500 rounded-full inline-block mx-0.5"></div>
                      </span>
                    )}
                  </div>

                  <p className="text-sm text-muted-foreground">
                    No cierres esta ventana hasta que el pago sea completado
                  </p>

                  {/* Botón para completar manualmente */}
                  {paymentProcessor.paymentStatus === "PENDIENTE" && (
                    <Button
                      variant="outline"
                      className="w-full mt-4 bg-emerald-50 border-emerald-200 hover:bg-emerald-100 hover:text-emerald-700"
                      onClick={() =>
                        paymentProcessor.completarOrdenManualmente(
                          paymentProcessor.qrData.orderId,
                          paymentProcessor.qrData.isSplitPayment,
                          paymentProcessor.qrData.cashAmount
                        )
                      }
                    >
                      <Receipt className="h-4 w-4 mr-2" />
                      Completar manualmente
                    </Button>
                  )}
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center h-64">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900"></div>
                <p className="mt-4">Generando código QR...</p>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={paymentProcessor.cancelQRPayment}
              className="w-full"
            >
              Cancelar
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Diálogo de contraseña para QR manual */}
      <ManualQrDialog
        open={paymentProcessor.manualQrPasswordDialogOpen}
        onOpenChange={(open: boolean) => {
          if (!open) {
            paymentProcessor.setManualQrPasswordDialogOpen(false);
            paymentProcessor.setManualQrPassword("");
          }
        }}
        password={paymentProcessor.manualQrPassword}
        onPasswordChange={(value: string) =>
          paymentProcessor.setManualQrPassword(value)
        }
        onSubmit={paymentProcessor.handleManualQrPasswordSubmit}
      />

      {/* Diálogo de pago mixto */}
      <Dialog
        open={splitPaymentDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            setSplitPaymentDialogOpen(false);
            paymentProcessor.resetPaymentState();
            setTimeout(() => searchInputRef.current?.focus(), 100);
          }
        }}
      >
        <DialogContent
          className="sm:max-w-[500px]"
          onKeyDown={(e) => {
            // Si se presiona Enter, simular clic en el botón "Completar pago"
            if (e.key === "Enter") {
              e.preventDefault();

              // Verificar si el botón debería estar habilitado
              const cashAmountValue = parseFloat(paymentProcessor.cashAmount);
              const isButtonEnabled = !(
                paymentProcessor.isProcessingPayment ||
                !paymentProcessor.cashAmount ||
                isNaN(cashAmountValue) ||
                cashAmountValue <= 0 ||
                cashAmountValue >= cartState.calculateTotal()
              );

              // Solo disparar la acción si el botón estaría habilitado
              if (isButtonEnabled) {
                console.log(
                  "⌨️ TECLADO: Enter detectado en diálogo de pago mixto"
                );
                paymentProcessor.processSplitPayment(
                  cartState.getCurrentItems(),
                  cartState.calculateTotal(),
                  businessInfo
                );
              }
            }
          }}
        >
          <DialogHeader>
            <DialogTitle className="text-xl">Pago mixto</DialogTitle>
            <DialogDescription>
              Ingresa el monto a pagar con efectivo y selecciona el método para
              el resto
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-6 py-4">
            <div className="text-center">
              <div className="text-lg text-muted-foreground">Total a pagar</div>
              <div className="text-3xl font-bold text-emerald-600 mt-1">
                ${cartState.calculateTotal().toLocaleString()}
              </div>
            </div>

            <div className="space-y-4">
              <div className="space-y-2">
                <div className="flex justify-between">
                  <label className="text-sm font-medium flex items-center">
                    <Wallet className="mr-2 h-4 w-4" />
                    Efectivo
                  </label>
                  <Input
                    type="number"
                    placeholder="Monto en efectivo"
                    value={paymentProcessor.cashAmount}
                    onChange={(e) =>
                      paymentProcessor.setCashAmount(e.target.value)
                    }
                    className="w-40 text-right"
                    step="0.01"
                    min="0"
                    max={cartState.calculateTotal().toString()}
                    disabled={paymentProcessor.isProcessingPayment}
                    autoFocus
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-sm font-medium">Segundo método</label>
                <div className="grid grid-cols-2 gap-3">
                  <Button
                    variant={
                      paymentProcessor.secondPaymentMethod === "tarjeta"
                        ? "default"
                        : "outline"
                    }
                    className={
                      paymentProcessor.secondPaymentMethod === "tarjeta"
                        ? "bg-emerald-600 hover:bg-emerald-700"
                        : ""
                    }
                    onClick={() =>
                      paymentProcessor.setSecondPaymentMethod("tarjeta")
                    }
                    disabled={paymentProcessor.isProcessingPayment}
                  >
                    <CreditCard className="mr-2 h-4 w-4" />
                    Tarjeta
                  </Button>
                  <Button
                    variant={
                      paymentProcessor.secondPaymentMethod === "qr"
                        ? "default"
                        : "outline"
                    }
                    className={
                      paymentProcessor.secondPaymentMethod === "qr"
                        ? "bg-emerald-600 hover:bg-emerald-700"
                        : ""
                    }
                    onClick={() =>
                      paymentProcessor.setSecondPaymentMethod("qr")
                    }
                    disabled={paymentProcessor.isProcessingPayment}
                  >
                    <QrCode className="mr-2 h-4 w-4" />
                    Transferencia
                  </Button>
                </div>
              </div>

              <div className="flex justify-between pt-2">
                <div className="flex items-center">
                  <div className="text-sm font-medium">
                    {paymentProcessor.secondPaymentMethod === "tarjeta" ? (
                      <div className="flex items-center">
                        <CreditCard className="mr-2 h-4 w-4" />
                        Tarjeta
                      </div>
                    ) : (
                      <div className="flex items-center">
                        <QrCode className="mr-2 h-4 w-4" />
                        Transferencia
                      </div>
                    )}
                  </div>
                </div>
                <div className="text-xl font-semibold text-emerald-600">
                  $
                  {!paymentProcessor.cashAmount ||
                  isNaN(parseFloat(paymentProcessor.cashAmount))
                    ? cartState.calculateTotal().toLocaleString()
                    : (
                        cartState.calculateTotal() -
                        Math.min(
                          parseFloat(paymentProcessor.cashAmount),
                          cartState.calculateTotal()
                        )
                      ).toLocaleString()}
                </div>
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setSplitPaymentDialogOpen(false);
                paymentProcessor.resetPaymentState();
              }}
              disabled={paymentProcessor.isProcessingPayment}
            >
              Cancelar
            </Button>
            <Button
              onClick={() =>
                paymentProcessor.processSplitPayment(
                  cartState.getCurrentItems(),
                  cartState.calculateTotal(),
                  businessInfo
                )
              }
              disabled={
                paymentProcessor.isProcessingPayment ||
                !paymentProcessor.cashAmount ||
                isNaN(parseFloat(paymentProcessor.cashAmount)) ||
                parseFloat(paymentProcessor.cashAmount) <= 0 ||
                parseFloat(paymentProcessor.cashAmount) >=
                  cartState.calculateTotal()
              }
              className="bg-emerald-gradient"
            >
              {paymentProcessor.isProcessingPayment ? (
                <div className="flex items-center gap-2">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  <span>Procesando...</span>
                </div>
              ) : (
                "Completar pago"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Nuevo Diálogo de Órdenes Recientes */}
      <RecentOrdersDialog
        isOpen={ordersDialogOpen}
        onClose={() => setOrdersDialogOpen(false)}
        API_URL={API_URL}
        appId={appId}
        formatFechaArgentina={formatFechaArgentina}
        handleTicketPrinting={handleTicketPrinting}
      />
    </div>
  );
}
