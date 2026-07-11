import {
  useState,
  useEffect,
  useRef,
  forwardRef,
  useImperativeHandle,
  useCallback,
} from "react";
import { toast } from "sonner";

// Hooks
import { useAuth } from "@/contexts/AuthContext";
import { useCartSidebar } from "@/contexts/CartSidebarContext";
import { useSearchInput } from "@/contexts/SearchInputContext";
import { useCartState, Product } from "@/hooks/useCartState";
import { usePaymentProcessing } from "@/hooks/usePaymentProcessing";
import { useAfipPaymentProcessing } from "@/hooks/useAfipPaymentProcessing";
import { useScaleWeight } from "@/hooks/useScaleWeight";
import { useBusinessInfo } from "@/hooks/useBusinessInfo";
import { useProducts } from "@/hooks/useProducts";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useClosing } from "@/hooks/useClosing";
import { useTicketPrinting } from "@/hooks/useTicketPrinting";
import { useCancellationControl } from "@/hooks/useCancellationControl";
import { AvailableProduct } from "@/hooks/useProductSearch";
import { getBusinessName } from "@/utils/businessHelpers";

// Componentes
import {
  AddProductDialog,
  ScanListaPrecioDialog,
  CartItem,
  CartSummary,
  CartTabs,
  PaymentDialog,
  ManualQrDialog,
  HeaderActions,
  ClosingDialog,
  QRPaymentDialog,
  SplitPaymentDialog,
  AfipSplitPaymentDialog,
} from "./shopping-cart";

// Componentes de diálogo
import { CashPaymentDialog, CancelDialog } from "./shopping-cart/dialogs";
import { ExactPaymentDialog } from "./shopping-cart/dialogs/ExactPaymentDialog";
import { CancellationDialog } from "./shopping-cart/dialogs/CancellationDialog";
import { DiscountDialog } from "./shopping-cart/DiscountDialog";

// Importar el nuevo componente de diálogo de órdenes recientes
import { RecentOrdersDialog } from "./RecentOrdersDialog";
import type { FilaListaScan } from "./shopping-cart/ScanListaPrecioDialog";

// Importar el componente de factura
import FacturaForm from "./facturas/FacturaForm";

// Props para el componente
interface ShoppingCartRefactoredProps {
  // Removido onCuentaCorrienteClick - ahora se maneja con navegación
}

// Interfaz para las funciones expuestas via ref
export interface ShoppingCartRefactoredRef {
  autoAddScannedProduct: (product: any, quantity: number) => void;
}

// Tipos

const ShoppingCartRefactored = forwardRef<
  ShoppingCartRefactoredRef,
  ShoppingCartRefactoredProps
>(({}: ShoppingCartRefactoredProps = {}, ref) => {
  const { user, logout } = useAuth();
  const { selectedProductFromSidebar, clearSelectedProduct } = useCartSidebar();
  const { searchInputRef, focusSearchInput } = useSearchInput();
  const API_URL = import.meta.env.VITE_API_URL;

  // Bloqueo simple para evitar disparar múltiples pagos
  const paymentLockRef = useRef(false);
  const isInitialMount = useRef(true);

  // Enfocar la barra al montar el componente
  useEffect(() => {
    focusSearchInput("montaje inicial");
  }, [focusSearchInput]);

  // Estados para búsqueda y productos
  const [searchQuery, setSearchQuery] = useState("");
  const [showResults, setShowResults] = useState(false);
  const [searchResults, setSearchResults] = useState<AvailableProduct[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);

  // Estados para diálogos y UI
  const [dialogOpen, setDialogOpen] = useState(false);
  const [cancelDialogOpen, setCancelDialogOpen] = useState(false);
  const [paymentDialogOpen, setPaymentDialogOpen] = useState(false);
  const [afipPaymentDialogOpen, setAfipPaymentDialogOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] =
    useState<AvailableProduct | null>(null);

  // Estado para el popup de selección de lista de precios al escanear
  const [scanListaDialog, setScanListaDialog] = useState<{
    open: boolean;
    product: AvailableProduct;
    rows: FilaListaScan[];
    quantity: number;
  } | null>(null);

  // Estados para el diálogo de órdenes recientes
  const [ordersDialogOpen, setOrdersDialogOpen] = useState(false);

  // Estados para el diálogo de factura
  const [facturaDialogOpen, setFacturaDialogOpen] = useState(false);

  // Estados para el diálogo de descuento
  const [discountDialogOpen, setDiscountDialogOpen] = useState(false);
  const [pendingDiscountAction, setPendingDiscountAction] = useState<
    "payment" | "afip" | "auto" | null
  >(null);
  const [discountData, setDiscountData] = useState<
    | {
        type: "percentage" | "fixed";
        value: number;
        amount: number;
      }
    | undefined
  >(undefined);

  // Estado eliminado: paymentMethodDialogOpen - ahora usamos PaymentDialog directamente

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

  // Añadir estado para diálogo de pago mixto AFIP
  const [afipSplitPaymentDialogOpen, setAfipSplitPaymentDialogOpen] =
    useState(false);

  // Estados de procesamiento de pago (manejados en este componente)
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<
    string | null
  >(null);
  // ✅ NUEVO: Estado para distinguir entre flujo AFIP (F2) y flujo normal (F3)
  const [isCurrentlyAfipFlow, setIsCurrentlyAfipFlow] = useState(false);

  // Estado para rastrear el último código procesado y evitar dobles escaneos

  // ✅ CRÍTICO: useRef para bloqueo inmediato de duplicaciones
  const isProcessingRef = useRef(false);
  const lastProcessedCode = useRef<{ code: string; timestamp: number } | null>(
    null
  );

  // Utilizar los hooks personalizados
  const cartState = useCartState();
  const { businessInfo, loading: businessInfoLoading } = useBusinessInfo(
    API_URL,
    getAppId()
  );
  const { availableProducts } = useProducts(API_URL, getAppId());
  const closing = useClosing(user, API_URL, getAppId());
  const { handleTicketPrinting, formatFechaArgentina } = useTicketPrinting();
  const cancellationControl = useCancellationControl({
    user,
    API_URL,
    appId: getAppId(),
    businessInfo,
  });

  const paymentProcessor = usePaymentProcessing({
    user,
    API_URL,
    appId: getAppId(),
    clearCart: cartState.clearCart,
    calculateTotal: cartState.calculateTotalWithIVA,
    setPaymentDialogOpen: setPaymentDialogOpen,
    setQrDialogOpen: setQrDialogOpen,
    setSplitPaymentDialogOpen: setSplitPaymentDialogOpen,
  });

  const { roundedAmountDialogOpen: cashDialogOpen, exactPaymentDialogOpen } =
    paymentProcessor;

  const afipPaymentProcessor = useAfipPaymentProcessing({
    user,
    API_URL,
    appId: getAppId(),
    clearCart: cartState.clearCart,
    calculateTotal: cartState.calculateTotalWithIVA,
    setPaymentDialogOpen: setAfipPaymentDialogOpen,
    searchInputRef,
    // NUEVO: pasar referencia para abrir diálogo mixto
    setSplitPaymentDialogOpen: setAfipSplitPaymentDialogOpen,
    setQrDialogOpen: setQrDialogOpen,
    getCurrentItems: cartState.getCurrentItems, // ✅ AGREGADO
  });

  // 🔄 SYNC: Mantener los estados locales de pago en línea con el hook
  useEffect(() => {
    setIsProcessingPayment(paymentProcessor.isProcessingPayment);
  }, [paymentProcessor.isProcessingPayment]);

  useEffect(() => {
    setSelectedPaymentMethod(paymentProcessor.selectedPaymentMethod);
  }, [paymentProcessor.selectedPaymentMethod]);

  // 🆕 NUEVO: Effect para limpiar estados locales cuando el hook se resetea
  useEffect(() => {
    // Si el hook no tiene método seleccionado ni está procesando, limpiar estados locales
    if (
      !paymentProcessor.selectedPaymentMethod &&
      !paymentProcessor.isProcessingPayment
    ) {
      setSelectedPaymentMethod(null);
      setIsProcessingPayment(false);
    }
  }, [
    paymentProcessor.selectedPaymentMethod,
    paymentProcessor.isProcessingPayment,
  ]);

  // 🆕 NUEVO: Effect para resetear estados cuando se abre el diálogo de pago
  useEffect(() => {
    if (paymentDialogOpen) {
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      paymentProcessor.resetPaymentState();
    }
  }, [paymentDialogOpen]);

  // 🆕 NUEVO: Effect para resetear estados cuando se abre el diálogo AFIP
  useEffect(() => {
    if (afipPaymentDialogOpen) {
      afipPaymentProcessor.resetPaymentState();
    }
  }, [afipPaymentDialogOpen]);

  // 🆕 NUEVO: Effect para detectar cuando se cierra el diálogo QR y limpiar estados residuales
  useEffect(() => {
    if (
      !qrDialogOpen &&
      !paymentDialogOpen &&
      !splitPaymentDialogOpen &&
      !afipSplitPaymentDialogOpen &&
      !exactPaymentDialogOpen &&
      !cashDialogOpen &&
      !ordersDialogOpen &&
      !deleteScreenDialogOpen &&
      !isAddingToCart
    ) {
      // Si todos los diálogos están cerrados pero aún hay estados activos, limpiar
      if (isProcessingPayment || selectedPaymentMethod) {
        // Limpiar estados locales
        setIsProcessingPayment(false);
        setSelectedPaymentMethod(null);

        // Asegurar que el hook también esté limpio
        if (
          paymentProcessor.isProcessingPayment ||
          paymentProcessor.selectedPaymentMethod
        ) {
          paymentProcessor.resetPaymentState();
        }
      }
    }
  }, [
    qrDialogOpen,
    paymentDialogOpen,
    splitPaymentDialogOpen,
    afipSplitPaymentDialogOpen,
    exactPaymentDialogOpen,
    cashDialogOpen,
    ordersDialogOpen,
    deleteScreenDialogOpen,
    isAddingToCart,
    isProcessingPayment,
    selectedPaymentMethod,
  ]);

  // 🆕 NUEVO: Liberar paymentLock cuando el hook deja de procesar
  useEffect(() => {
    if (!paymentProcessor.isProcessingPayment) {
      paymentLockRef.current = false;
    }
  }, [paymentProcessor.isProcessingPayment]);

  // 🆕 NUEVO: Cerrar automáticamente el diálogo de pago cuando se abra el QRDialog
  useEffect(() => {
    if (qrDialogOpen && paymentDialogOpen) {
      setPaymentDialogOpen(false);
    }
  }, [qrDialogOpen, paymentDialogOpen]);

  // 🆕 NUEVO: Enfocar automáticamente la barra de búsqueda cuando todos los diálogos estén cerrados
  useEffect(() => {
    // No ejecutar en el montaje inicial
    if (isInitialMount.current) {
      isInitialMount.current = false;
      return;
    }

    if (
      !dialogOpen &&
      !cancelDialogOpen &&
      !paymentDialogOpen &&
      !afipPaymentDialogOpen &&
      !qrDialogOpen &&
      !splitPaymentDialogOpen &&
      !exactPaymentDialogOpen &&
      !cashDialogOpen &&
      !ordersDialogOpen &&
      !facturaDialogOpen &&
      !deleteScreenDialogOpen &&
      !isAddingToCart
    ) {
      focusSearchInput("cierre de diálogo o fin de acción");
    }
  }, [
    dialogOpen,
    cancelDialogOpen,
    paymentDialogOpen,
    afipPaymentDialogOpen,
    qrDialogOpen,
    splitPaymentDialogOpen,
    exactPaymentDialogOpen,
    cashDialogOpen,
    ordersDialogOpen,
    facturaDialogOpen,
    deleteScreenDialogOpen,
    isAddingToCart,
    focusSearchInput,
  ]);

  // Handler para cerrar sesión
  const handleLogout = () => {
    toast.success("Cerrando sesión...");
    setTimeout(() => {
      logout();
    }, 1000);
  };

  // Handler para seleccionar un producto
  const handleProductSelect = (product: AvailableProduct) => {
    if (!product) {
      return;
    }

    setSelectedProduct(product);
    setDialogOpen(true);
  };

  // Handler para agregar un producto al carrito
  const handleAddToCart = (product: any) => {
    cartState.addToCart(product);
    setDialogOpen(false);
    setSelectedProduct(null);

    // Enfocar el input de búsqueda
    focusSearchInput("agregado producto");
  };

  // Handlers del popup de lista de precios al escanear
  const handleScanListaAdd = (product: Product) => {
    cartState.addToCart(product);
    setScanListaDialog(null);
    if (searchInputRef.current) {
      searchInputRef.current.value = "";
      setSearchQuery("");
      searchInputRef.current.focus();
    }
  };

  const handleScanListaClose = () => {
    setScanListaDialog(null);
    if (searchInputRef.current) {
      searchInputRef.current.value = "";
      setSearchQuery("");
      searchInputRef.current.focus();
    }
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
  const handleCancelCart = async () => {
    // Si las cancelaciones están habilitadas, usar el flujo controlado
    if (cancellationControl.isCancellationEnabled) {
      // Calcular monto total del carrito
      const montoTotal = cartState.calculateTotal();

      // Obtener productos del carrito para la cancelación
      const productosCarrito = cartState.getCurrentItems();

      // Preparar productos para la API de cancelación
      const productosCancelacion = productosCarrito.map((item) => ({
        productoId: item.id,
        nombreProducto: item.name,
        cantidad: item.quantity,
        precioUnitario: item.pricePerUnit,
        subtotal: Number(item.subtotal.toFixed(2)),
      }));

      // Generar un ID temporal para la orden (en un caso real, esto sería el ID de la orden)
      const tempReferenciaId = `TEMP-${Date.now()}`;

      // Iniciar el proceso de cancelación controlada
      const usingControlledFlow = cancellationControl.initiateCancellation(
        "orden",
        tempReferenciaId,
        montoTotal,
        productosCancelacion
      );

      if (usingControlledFlow) {
        // El diálogo de cancelación se abrirá automáticamente
        return;
      }
    }

    // Flujo normal (cancelaciones deshabilitadas o fallback)
    cartState.clearCart();
    setCancelDialogOpen(false);

    // Enfocar el input de búsqueda
    focusSearchInput("carrito cancelado");

    toast.success("Carrito cancelado");
  };

  // Handler para confirmar cancelación controlada
  const handleConfirmCancellation = async (): Promise<boolean> => {
    const success = await cancellationControl.confirmCancellation();

    if (success) {
      // Limpiar el carrito después del registro exitoso
      cartState.clearCart();
      setCancelDialogOpen(false);

      // Enfocar el input de búsqueda
      focusSearchInput("carrito cancelado con registro");
    }

    return success || false;
  };

  // Handler para cancelar el proceso de cancelación controlada
  const handleCancelCancellationProcess = () => {
    cancellationControl.cancelCancellationProcess();
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

    // Resetear estados del flujo AFIP al abrir diálogo normal
    afipPaymentProcessor.resetPaymentState();

    // ✅ MARCAR: Flujo normal (F3 - Sin AFIP)
    setIsCurrentlyAfipFlow(false);

    // Resetear descuento al cambiar a diálogo de pago normal
    setDiscountData(undefined);

    setPaymentDialogOpen(true);
    setAfipPaymentDialogOpen(false); // Asegurar que el diálogo AFIP esté cerrado
  };

  // Handler para mostrar diálogo de pago AFIP
  const handleAfipPaymentClick = () => {
    const currentItems = cartState.getCurrentItems();
    if (currentItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de continuar",
      });
      return;
    }

    // Resetear estados del flujo normal al abrir AFIP
    paymentProcessor.resetPaymentState();
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);

    // ✅ MARCAR: Flujo AFIP (F2 - Con AFIP)
    setIsCurrentlyAfipFlow(true);

    // Resetear descuento al cambiar a diálogo de pago AFIP
    setDiscountData(undefined);

    setAfipPaymentDialogOpen(true);
    setPaymentDialogOpen(false); // Asegurar que el diálogo normal esté cerrado
  };

  // Handler para mostrar diálogo de factura (F6)
  const handleFacturaClick = () => {
    // Validar que la facturación esté habilitada
    if (!businessInfo?.facturacionHabilitada) {
      toast.error("Funcionalidad no disponible", {
        description: "La facturación no está habilitada para este negocio",
      });
      return;
    }

    const currentItems = cartState.getCurrentItems();
    if (currentItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de crear la factura",
      });
      return;
    }

    setFacturaDialogOpen(true);
  };

  // Handler para cerrar diálogo de factura
  const handleFacturaClose = () => {
    setFacturaDialogOpen(false);
    // Enfocar el input de búsqueda
    focusSearchInput("factura cerrada");
  };

  // Handler para éxito de factura
  const handleFacturaSuccess = () => {
    setFacturaDialogOpen(false);
    // Limpiar el carrito después de crear la factura exitosamente
    cartState.clearCart();
    // Enfocar el input de búsqueda
    focusSearchInput("factura creada exitosamente");
    // ✅ CORRECCIÓN: No mostrar toast aquí - ya se muestra en FacturaForm
  };

  // Handler para mostrar diálogo de pago con descuento (F7)
  const handleDiscountPaymentClick = () => {
    const currentItems = cartState.getCurrentItems();
    if (currentItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de continuar",
      });
      return;
    }


    // Configurar la acción pendiente y abrir el diálogo de descuento
    setPendingDiscountAction("payment");
    setDiscountDialogOpen(true);
  };

  // Handler para mostrar diálogo de pago AFIP con descuento (F8)
  const handleDiscountAfipPaymentClick = () => {
    const currentItems = cartState.getCurrentItems();
    if (currentItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de continuar",
      });
      return;
    }


    // Configurar la acción pendiente y abrir el diálogo de descuento
    setPendingDiscountAction("afip");
    setDiscountDialogOpen(true);
  };

  // Handler para confirmar descuento y proceder con el pago
  const handleDiscountConfirm = (discountDialogData: {
    tieneDescuento: boolean;
    tipoDescuento: "porcentual" | "cantidad";
    valorDescuento: number;
    subtotalSinDescuento: number;
    montoDescuento: number;
  }) => {
    // Mapear el tipo de DiscountDialog al tipo esperado por las funciones de pago
    const mappedDiscountData = {
      type:
        discountDialogData.tipoDescuento === "porcentual"
          ? ("percentage" as const)
          : ("fixed" as const),
      value: discountDialogData.valorDescuento,
      amount: discountDialogData.montoDescuento,
    };

    setDiscountData(mappedDiscountData);
    setDiscountDialogOpen(false);

    // Determinar si usar AFIP basado en la acción pendiente y configuración del negocio
    const shouldUseAfip =
      pendingDiscountAction === "afip" ||
      (pendingDiscountAction === "auto" && businessInfo?.afipEnabled);

    if (shouldUseAfip) {
      // ✅ MARCAR: Flujo AFIP (F7 con AFIP habilitado o F8)
      setIsCurrentlyAfipFlow(true);
      setAfipPaymentDialogOpen(true);
      setPaymentDialogOpen(false);
    } else {
      // ✅ MARCAR: Flujo normal (F7 sin AFIP o F8 sin AFIP)
      setIsCurrentlyAfipFlow(false);
      setPaymentDialogOpen(true);
      setAfipPaymentDialogOpen(false);
    }

    // Limpiar la acción pendiente
    setPendingDiscountAction(null);
  };

  // Handler para cancelar descuento
  const handleDiscountCancel = () => {
    setDiscountDialogOpen(false);
    setPendingDiscountAction(null);
    setDiscountData(undefined);
    // Enfocar el input de búsqueda
    focusSearchInput("descuento cancelado");
  };

  // Función eliminada: handlePaymentMethodSelect - ahora usamos PaymentDialog directamente

  // Handler para seleccionar método de pago AFIP
  const handleAfipPayment = async (method: string) => {
    if (paymentLockRef.current) return;
    paymentLockRef.current = true;

    try {
      // Si es efectivo, usar el handler especial como en el flujo normal
      if (method === "efectivo") {
        if (!businessInfo) {
          toast.error("Información de negocio no disponible");
          return;
        }
        afipPaymentProcessor.handleAfipCashPayment(businessInfo, false, discountData);
        setAfipPaymentDialogOpen(false);
        paymentLockRef.current = false;
        return;
      }

      // 🆕 NUEVO: QR con AFIP (generar QR primero, factura después)
      if (method === "qr") {

        // ✅ VERIFICAR SI MP ESTÁ HABILITADO IGUAL QUE EN FLUJO NORMAL
        if (businessInfo?.mpEnabled === false) {

          // Procesar como transferencia directa + factura AFIP
          await afipPaymentProcessor.processAfipPayment(
            "qr", // Método QR pero sin MercadoPago
            cartState.getCurrentItems(),
            undefined,
            discountData
          );

          setAfipPaymentDialogOpen(false);
          paymentLockRef.current = false;
          return;
        }

        // Si MP está habilitado, generar QR de MercadoPago + AFIP
        const qrData = await afipPaymentProcessor.handleAfipQrPayment(
          cartState.getCurrentItems(),
          discountData
        );

        if (qrData) {
          // ✅ CORRECCIÓN: Agregar el monto desde el carrito ya que el backend no lo incluye
          const qrDataWithAmount = {
            ...qrData,
            monto: Number(cartState.calculateTotal().toFixed(2)),
          };

          paymentProcessor.updateQrData(qrDataWithAmount);
          paymentProcessor.setPaymentStatus("PENDIENTE");

          // Abrir el diálogo QR con los datos
          setQrDialogOpen(true);
          setAfipPaymentDialogOpen(false);

          // ✅ MANTENER: isCurrentlyAfipFlow como true para el QR AFIP
          // (Ya está establecido en handleAfipPaymentClick)

          // El QRPaymentDialog manejará el polling y cuando se confirme el pago
          // llamará a afipPaymentProcessor.createAfipInvoiceAfterPayment
        }

        paymentLockRef.current = false;
        return;
      }

      // NUEVO: pago mixto
      if (method === "split") {
        afipPaymentProcessor.handleSplitPayment();
        // Cerrar diálogo principal después de un pequeño delay
        setTimeout(() => {
          setAfipPaymentDialogOpen(false);
        }, 100);
        paymentLockRef.current = false;
        return;
      }

      // Para otros métodos (tarjeta), procesar directamente
      await afipPaymentProcessor.processAfipPayment(
        method,
        cartState.getCurrentItems(),
        undefined,
        discountData
      );

      // Los estados se limpian en el hook afipPaymentProcessor
    } catch (error: any) {
      console.error("❌ Error en pago AFIP:", error);
      toast.error(`Error en factura AFIP: ${error.message}`);

      // ✅ CRÍTICO: NO limpiar el carrito en caso de error
      // Los productos deben permanecer para que el usuario pueda reintentar
      // NO limpiar estados locales aquí, solo los del hook AFIP
      afipPaymentProcessor.resetPaymentState();
    } finally {
      paymentLockRef.current = false;
    }
  };

  // Handler para seleccionar método de pago
  const handlePayment = async (method: string) => {
    if (paymentLockRef.current) return;
    paymentLockRef.current = true;
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      paymentLockRef.current = false;
      return;
    }


    // Prevenir procesamiento duplicado usando estado local antes de cualquier acción
    if (isProcessingPayment) {
      paymentLockRef.current = false;
      return;
    }

    // Si es efectivo, manejar con el sistema de redondeo. Pasa el control.
    if (method === "efectivo") {
      // NO establecer isProcessingPayment aquí, solo cuando se confirme el pago
      setSelectedPaymentMethod("efectivo");
      paymentProcessor.handleCashPayment(businessInfo, !!discountData, discountData);
      setPaymentDialogOpen(false);
      paymentLockRef.current = false;
      return;
    }

    // Si es pago dividido, preparar pago mixto. Pasa el control.
    if (method === "split") {
      // ✅ ASEGURAR: En F3, el pago mixto NO debe activar AFIP

      paymentProcessor.handleSplitPayment();

      // Cerrar el diálogo principal después de un pequeño delay
      setTimeout(() => {
        setPaymentDialogOpen(false);
      }, 100);
      paymentLockRef.current = false;
      return;
    }

    // Marcar como procesando y establecer método seleccionado localmente
    setIsProcessingPayment(true);
    setSelectedPaymentMethod(method);

    // Si es QR, generar el QR
    if (method === "qr") {

      // Si MP está deshabilitado, procesar como transferencia directamente
      if (businessInfo?.mpEnabled === false) {

        // Preparar datos de la orden
        const currentItems = cartState.getCurrentItems();
        const orderItems = currentItems.map((item) => ({
          productoId: item.id,
          cantidad: item.quantity,
          subtotal: Number(item.subtotal.toFixed(2)),
          precioHistorico: item.pricePerUnit,
          costo: Number(item.costo),
          nombre: item.name,
          listaPrecioId: item.listaPrecioId ?? null,
          listaPrecioNombre: item.listaPrecioNombre ?? undefined,
        }));

        // Calcular total con descuento si existe
        const subtotalSinDescuento = Number(cartState.calculateTotal().toFixed(2));
        const totalConDescuento = discountData 
          ? Number((subtotalSinDescuento - discountData.amount).toFixed(2))
          : subtotalSinDescuento;

        const orderData = {
          metodoPago: "qr",
          total: totalConDescuento,
          items: orderItems,
          vendedorId: user.id,
          sucursalId: user.sucursalId,
          vendedor: user.nombre,
          businessName: await getBusinessName(),
          estado: "COMPLETADA",
          createdAt: new Date().toISOString(),
          // ✅ CRÍTICO: Incluir datos de descuento cuando existe
          ...(discountData && {
            tieneDescuento: true,
            tipoDescuento: discountData.type === "percentage" ? "porcentual" : "cantidad",
            valorDescuento: discountData.value,
            montoDescuento: discountData.amount,
            subtotalSinDescuento: subtotalSinDescuento,
            subtotal: subtotalSinDescuento,
            discountData: {
              type: discountData.type,
              value: discountData.value,
              amount: discountData.amount,
            },
          }),
          ...(!discountData && {
            tieneDescuento: false,
          }),
        };

        try {
          // Mostrar toast de carga
          const processingToastId = toast.loading("Procesando orden...");

          // Preparar headers correctamente
          const headers: HeadersInit = {
            "Content-Type": "application/json",
          };

          const appId = getAppId();
          if (appId) {
            (headers as Record<string, string>)["X-App-ID"] = appId;
          }

          // Crear la orden
          const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
            method: "POST",
            headers,
            body: JSON.stringify(orderData),
          });

          if (!orderResponse.ok) {
            const errorData = await orderResponse.json().catch(() => ({}));
            console.error("❌ Error al crear la orden API:", errorData);
            throw new Error(errorData.message || "Error al crear la orden");
          }

          // Capturar la respuesta para obtener el ID real de la orden
          const orderResult = await orderResponse.json();

          // Añadir el idReal a los datos de la orden para impresión
          const enrichedOrderData = {
            ...orderData,
            idReal: orderResult.idReal || orderResult.id || null,
            id: orderResult.id || null,
          };

          // Ocultar toast de carga y mostrar toast de impresión
          toast.dismiss(processingToastId);

          // Mostrar toast de carga para la impresión
          const printingToastId = toast.loading("Imprimiendo ticket...");

          try {
            // ✅ IMPRESIÓN SIMPLIFICADA: El hook useTicketPrinting ya maneja la doble impresión internamente
            await handleTicketPrinting(enrichedOrderData, API_URL, getAppId());
          } catch (printError) {
            console.error("Error en impresión:", printError);
          } finally {
            // Cerrar el toast de carga de impresión SIEMPRE
            toast.dismiss(printingToastId);
          }

          // ✅ CRÍTICO: Limpiar carrito y estados locales SOLO después de que todo esté completo
          setTimeout(() => {
            cartState.clearCart();
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
            setPaymentDialogOpen(false);
          }, 100);

          toast.success("Orden completada exitosamente");
          paymentLockRef.current = false;
        } catch (error: any) {
          console.error("❌ Error en flujo QR/MP deshabilitado:", error);
          toast.error(`Error al procesar la orden: ${error.message}`);

          // ✅ CRÍTICO: NO limpiar el carrito en caso de error
          // Los productos deben permanecer para que el usuario pueda reintentar
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
          setPaymentDialogOpen(false);

          // Asegurar que cualquier toast de carga se cierre
          toast.dismiss("processing-order");
          toast.dismiss("printing-ticket");
          paymentLockRef.current = false;

        }

        return;
      }

      // Si MP está habilitado, generar el QR
      paymentProcessor.generateQRPayment(
        cartState.getCurrentItems(),
        discountData
      );
      // No cerramos inmediatamente el diálogo: permanecerá mostrando spinner
      // El diálogo se cerrará automáticamente cuando se abra el diálogo QR
      return;
    }

    // Para otros métodos (tarjeta, transferencia), procesar directamente
    try {
      // Calcular total con descuento aplicado si existe
      const subtotalSinDescuento = Number(cartState.calculateTotalWithIVA().toFixed(2));
      const totalConDescuento = discountData
        ? Number((subtotalSinDescuento - discountData.amount).toFixed(2))
        : subtotalSinDescuento;

      await paymentProcessor.processPayment(
        method,
        totalConDescuento,
        cartState.getCurrentItems(),
        discountData
      );
      // ✅ MEJORADO: Estados se limpian en processPayment, pero asegurar limpieza local
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    } catch (error: any) {
      console.error("❌ Error al procesar pago:", error);
      toast.error(`Error al procesar el pago: ${error.message}`);

      // ✅ CRÍTICO: NO limpiar el carrito en caso de error
      // Los productos deben permanecer para que el usuario pueda reintentar
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      setPaymentDialogOpen(false);

    } finally {
      // ✅ MEJORADO: Asegurar que el estado se resetee SIEMPRE
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      paymentLockRef.current = false;
    }
  };

  // Confirmar pago redondeado en efectivo
  const confirmRoundedPayment = async () => {

    if (isProcessingPayment) {
      return;
    }

    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    if (paymentProcessor.roundedAmount <= 0) {
      toast.error("Error: Monto inválido");
      return;
    }

    setIsProcessingPayment(true);

    try {
      const subtotalOriginal = cartState.calculateTotal();
      const discountAmount = Number(
        (subtotalOriginal - paymentProcessor.roundedAmount).toFixed(2)
      );

      let finalDiscountData = discountData;
      if (discountData) {
        finalDiscountData = {
          ...discountData,
          amount: discountAmount,
        };
      } else if (
        paymentProcessor.applyingDiscount &&
        businessInfo?.descuentoEfectivo
      ) {
        finalDiscountData = {
          type: "percentage" as const,
          value: Number(businessInfo.descuentoEfectivo),
          amount: discountAmount,
        };
      }

      await paymentProcessor.processPayment(
        "efectivo",
        paymentProcessor.roundedAmount,
        cartState.getCurrentItems(),
        finalDiscountData
      );

      // ✅ MEJORADO: Limpiar estados locales después de éxito
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);

      // NO cerrar el diálogo aquí - se cerrará en processPayment después del éxito
    } catch (error) {
      console.error("❌ Error al procesar pago:", error);
      toast.error("Error al procesar el pago");

      // ✅ MEJORADO: Limpiar estados en caso de error
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      paymentProcessor.setRoundedAmountDialogOpen(false);
    }
  };

  // Manejadores para pantallas múltiples
  const handleAddScreen = () => {
    const result = cartState.addScreen();
    if (!result) {
      toast.error("No se pueden crear más de 4 pantallas");
      return;
    }
    // ✅ NUEVO: Enfocar el input después de crear una nueva orden
    setTimeout(() => {
      focusSearchInput("nueva orden creada");
    }, 100);
  };

  const handleDeleteScreen = (screenId: number) => {
    if (cartState.screens.length <= 1) {
      toast.error("No se puede eliminar la última pantalla");
      return;
    }

    if (window.confirm("¿Seguro que deseas eliminar esta pantalla?")) {
      const result = cartState.deleteScreen(screenId);
      if (result) {
        toast.success("Pantalla eliminada correctamente");
        // ✅ NUEVO: Enfocar el input después de eliminar una orden
        setTimeout(() => {
          focusSearchInput("orden eliminada");
        }, 100);
      }
    }
  };

  // ✅ NUEVO: Manejar cambio de orden y enfocar input
  const handleChangeScreen = (screenId: number) => {
    cartState.setActiveScreen(screenId);
    // Enfocar el input cuando se cambia de orden
    setTimeout(() => {
      focusSearchInput(`cambio a orden ${screenId + 1}`);
    }, 100);
  };

  // ✅ CRÍTICO: Función centralizada para agregar productos escaneados
  const addScannedProduct = useCallback(
    async (product: any, quantity: number = 1) => {
      if (isProcessingRef.current) {
        return;
      }

      isProcessingRef.current = true;

      try {
        const now = Date.now();
        const productKey = `${product.id}-${product.name}`;

        if (
          lastProcessedCode.current &&
          (lastProcessedCode.current.code === productKey ||
            lastProcessedCode.current.code === product.codigoBarras ||
            lastProcessedCode.current.code === product.plu) &&
          now - lastProcessedCode.current.timestamp < 1000
        ) {
          return;
        }

        lastProcessedCode.current = { code: productKey, timestamp: now };

        let pricePerUnit = product.pricePerUnit;
        let listaPrecioId: number | null = null;
        let listaPrecioNombre: string | null = null;
        const etiquetaBase = (
          businessInfo?.etiquetaPrecioBase || "Precio catálogo"
        ).trim();

        // El popup de selección de lista de precios solo se muestra si el
        // negocio lo tiene habilitado desde el superadmin. Por defecto está
        // desactivado: se carga el producto con su precio base/catálogo.
        if (API_URL && businessInfo?.popupListaPreciosEnabled) {
          try {
            const headers: Record<string, string> = {
              "Content-Type": "application/json",
            };
            const aid = getAppId();
            if (aid) headers["X-App-ID"] = aid;
            const res = await fetch(
              `${API_URL}/api/productos/${product.id}/listas-precios`,
              { credentials: "include", headers }
            );
            if (res.ok) {
              const data = await res.json();
              const activeRows = (data.productos || []).filter(
                (x: any) => x.activa && x.listaPrecio?.activa
              );

              // Mostrar el popup siempre que el producto tenga al menos 1 lista
              // activa, para que el vendedor elija lista/medida (incluye precio base).
              if (activeRows.length >= 1) {
                const filas: FilaListaScan[] = activeRows.map((x: any) => ({
                  listaPrecioId: x.listaPrecioId,
                  precio: x.precio,
                  nombreLista:
                    x.listaPrecio?.nombre ?? `Lista #${x.listaPrecioId}`,
                  tipoMedida: x.tipoMedida ?? null,
                }));
                setScanListaDialog({ open: true, product, rows: filas, quantity });
                return;
              }
            }
          } catch {
            pricePerUnit = product.pricePerUnit;
          }
        }

        const uniqueId = `${product.id}-${Date.now()}-${Math.random()
          .toString(36)
          .substring(2, 10)}`;
        const newItem: Product = {
          id: product.id,
          cartId: uniqueId,
          name: product.name,
          quantity: quantity,
          unit: product.unit || "Unidad",
          pricePerUnit,
          subtotal: Number((pricePerUnit * quantity).toFixed(2)),
          costo: product.costo,
          ivaIncluido: product.ivaIncluido,
          ivaPorcentaje: product.ivaPorcentaje,
          listaPrecioId,
          listaPrecioNombre: listaPrecioNombre ?? (listaPrecioId ? null : etiquetaBase),
        };

        cartState.addToCart(newItem);

        if (searchInputRef.current) {
          searchInputRef.current.value = "";
          setSearchQuery("");
          searchInputRef.current.focus();
        }
      } catch (error) {
        console.error("Error al agregar producto:", error);
      } finally {
        isProcessingRef.current = false;
      }
    },
    [cartState, businessInfo, API_URL]
  );

  // ✅ CRÍTICO: Función para procesar códigos completos
  const processCompleteCode = useCallback(
    (completeCode: string) => {
      // ✅ BLOQUEO INMEDIATO: Si ya estamos procesando, salir
      if (isProcessingRef.current) {
        return;
      }

      // ✅ VALIDACIÓN: Productos cargados
      if (!availableProducts || availableProducts.length === 0) {
        return;
      }

      // ✅ VALIDACIÓN: Código no vacío
      if (!completeCode || completeCode.trim().length === 0) {
        return;
      }

      // ✅ PATRÓN 1: Código de barras estándar (13 dígitos)
      if (completeCode.length === 13 && /^\d{13}$/.test(completeCode)) {

        const productsWithBarcode = availableProducts.filter(
          (p) => p.codigoBarras
        );

        const product = productsWithBarcode.find(
          (p) => p.codigoBarras === completeCode
        );

        if (product) {

          // ✅ AGREGAR PRODUCTO usando la función centralizada
          addScannedProduct(product, 1);

          // ✅ DEBUG: Estado del carrito después de agregar código de barras
          setTimeout(() => {
            const itemsAfterAdd = cartState.getCurrentItems();

            // ✅ Verificar que el producto fue agregado correctamente
            if (itemsAfterAdd.length > 0) {
              const lastAdded = itemsAfterAdd[itemsAfterAdd.length - 1];
            }
          }, 150);
          return;
        } else {
          return;
        }
      }

      // ✅ PATRÓN 2: PLU + peso (12 o 13 dígitos)
      if (completeCode.length === 12 || completeCode.length === 13) {

        let plu: string;
        let weightStr: string;
        let formatUsed: string;

        if (completeCode.length === 12) {
          // Formato: XXX + 8 dígitos peso + 1
          plu = completeCode.substring(0, 3);
          weightStr = completeCode.substring(3, 11);
          formatUsed = "12 dígitos (XXX + 8 dígitos peso + 1)";
        } else {
          // Formato: 0 + XXX + 8 dígitos peso + 1
          plu = completeCode.substring(1, 4);
          weightStr = completeCode.substring(4, 12);
          formatUsed = "13 dígitos (0 + XXX + 8 dígitos peso + 1)";
        }

        // ✅ VALIDACIÓN: PLU debe ser numérico
        if (!/^\d{3}$/.test(plu)) {
          return;
        }

        // ✅ VALIDACIÓN: Peso debe ser numérico
        if (!/^\d{8}$/.test(weightStr)) {
          return;
        }

        // ✅ CONVERTIR peso a kilogramos
        const weightGrams = parseInt(weightStr, 10);
        const kgQuantity = weightGrams / 1000; // Convertir gramos a kilogramos

        // ✅ VALIDACIÓN: Rango de peso válido
        if (kgQuantity < 0.001 || kgQuantity > 999.999) {
          return;
        }

        const productByPlu: any = availableProducts.find((p: any) => {
          if (p.plu === null || p.plu === undefined) return false;
          return Number(p.plu) === Number(plu);
        });

        if (productByPlu) {
          // ✅ AGREGAR PRODUCTO usando la función centralizada
          addScannedProduct(productByPlu, kgQuantity);
        }
      }
    },
    [availableProducts, cartState, addScannedProduct]
  );

  // Usar el hook de atajos de teclado después de declarar todas las funciones
  useKeyboardShortcuts({
    paymentDialogOpen,
    afipPaymentDialogOpen,
    isProcessingPayment,
    selectedPaymentMethod,
    afipIsProcessingPayment: afipPaymentProcessor.isProcessingPayment,
    afipSelectedPaymentMethod: afipPaymentProcessor.selectedPaymentMethod,
    handlePayment,
    handleAfipPayment,
    handleLogout,
    handleCancelClick,
    handlePaymentClick,
    handleAfipPaymentClick,
    handleDiscountPaymentClick,
    handleDiscountAfipPaymentClick,
    handleFacturaClick,
    getCurrentItems: cartState.getCurrentItems,
    calculateTotal: cartState.calculateTotal,
    businessInfo,
  });

  // 🆕 NUEVO: Limpiar tracking de órdenes procesadas cuando se cierre el diálogo QR
  useEffect(() => {
    if (!qrDialogOpen) {
      // Limpiar tracking en ambos hooks
      paymentProcessor.clearProcessedOrdersTracking?.();
      afipPaymentProcessor.clearProcessedOrdersTracking?.();

      // ✅ NUEVO: NO limpiar el polling automáticamente al cerrar el diálogo
      // El polling debe continuar hasta que el pago se complete o falle
    }
  }, [qrDialogOpen, paymentProcessor, afipPaymentProcessor]);


  // ✅ CRÍTICO: Limpiar intervalos solo al desmontar el componente
  useEffect(() => {
    return () => {
      // Limpiar intervalos solo al desmontar el componente, no al cambiar estado
      paymentProcessor.cleanupPolling?.();
      afipPaymentProcessor.cleanupPolling?.();
    };
  }, []); // Solo se ejecuta al desmontar, no cuando cambian los processors

  // Escuchar productos seleccionados desde el sidebar
  useEffect(() => {
    if (selectedProductFromSidebar) {
      handleProductSelect(selectedProductFromSidebar);
      clearSelectedProduct();
    }
  }, [selectedProductFromSidebar, clearSelectedProduct]);

  // 🎯 ESCUCHA GLOBAL DEL INPUT PARA ESCÁNER DE CÓDIGOS
  useEffect(() => {
    const input = searchInputRef.current;
    if (!input) {
      return;
    }

    // ✅ VARIABLE DE CONTROL INMEDIATA - NO estado de React
    let isProcessingPaste = false;

    // Buffer para acumular caracteres del escáner
    let inputBuffer = "";
    let inputTimeout: NodeJS.Timeout | null = null;

    const processCompleteCode = (completeCode: string) => {
      if (!completeCode) {
        return;
      }

      // Verificar que tengamos productos cargados
      if (!availableProducts || availableProducts.length === 0) {
        return;
      }


      // 1. Código de barras estándar (8-15 dígitos) - EAN-8, UPC-A, EAN-13, Code 128, etc.
      if (/^\d{8,15}$/.test(completeCode)) {

        // Buscar coincidencia exacta
        let product = availableProducts.find(
          (p: any) => p.codigoBarras === completeCode
        );

        // Si no hay coincidencia exacta, buscar si algún código contiene el valor o viceversa
        if (!product) {
          product = availableProducts.find(
            (p: any) =>
              p.codigoBarras &&
              (p.codigoBarras.includes(completeCode) ||
                completeCode.includes(p.codigoBarras))
          );
        }

        if (product) {
          // ✅ USAR SOLO LA FUNCIÓN CENTRALIZADA - NO duplicar lógica
          addScannedProduct(product, 1);

          // Limpiar completamente el input y el buffer
          input.value = "";
          inputBuffer = "";
          if (inputTimeout) {
            clearTimeout(inputTimeout);
            inputTimeout = null;
          }

          return;
        }
      }

      // 2. PLU + peso: Múltiples formatos soportados
      // Formato 1: 3 dígitos PLU + 8 dígitos gramos + 1 dígito adicional (12 dígitos total) - "105000013552"
      // Formato 2: 0 + 3 dígitos PLU + 8 dígitos gramos + 1 dígito adicional (13 dígitos total) - "0105000013552"

      let plu = null;
      let grams = null;
      let kgQuantity = null;
      let formatUsed = null;

      // Intentar formato de 12 dígitos primero
      const pluWeightRegex12 = /^(\d{3})(\d{8})(\d{1})$/;
      const match12 = completeCode.match(pluWeightRegex12);

      if (match12) {
        plu = match12[1];
        grams = parseInt(match12[2], 10);
        kgQuantity = grams / 1000;
        formatUsed = "12 dígitos";
      } else {
        // Intentar formato de 13 dígitos (0 + PLU + peso + dígito final)
        const pluWeightRegex13 = /^0(\d{3})(\d{8})(\d{1})$/;
        const match13 = completeCode.match(pluWeightRegex13);

        if (match13) {
          plu = match13[1];
          grams = parseInt(match13[2], 10);
          kgQuantity = grams / 1000;
          formatUsed = "13 dígitos";
        }
      }

      if (plu && grams !== null && kgQuantity !== null) {
        // ✅ CORRECCIÓN: Validar que el peso sea razonable (entre 0.001 y 999.999 kg)
        if (kgQuantity < 0.001 || kgQuantity > 999.999) {
          return;
        }

        const productByPlu: any = availableProducts.find((p: any) => {
          if (p.plu === null || p.plu === undefined) return false;
          return Number(p.plu) === Number(plu);
        });

        if (productByPlu) {
          // ✅ USAR SOLO LA FUNCIÓN CENTRALIZADA - NO duplicar lógica
          addScannedProduct(productByPlu, kgQuantity);

          // Limpiar input y buffer
          input.value = "";
          inputBuffer = "";
          if (inputTimeout) {
            clearTimeout(inputTimeout);
            inputTimeout = null;
          }
        }
      }
    };

    const handleInputEvent = (e: Event) => {
      const value = (e.target as HTMLInputElement).value.trim();

      // ✅ CRÍTICO: Si estamos procesando un paste, ignorar este input event
      if (isProcessingPaste) {
        return;
      }

      // Si el valor está vacío, limpiar buffer
      if (!value) {
        inputBuffer = "";
        if (inputTimeout) {
          clearTimeout(inputTimeout);
          inputTimeout = null;
        }
        return;
      }

      // Actualizar buffer con el valor completo
      inputBuffer = value;

      // Limpiar timeout anterior si existe
      if (inputTimeout) {
        clearTimeout(inputTimeout);
      }

      // Establecer nuevo timeout para procesar después de que se complete la entrada
      inputTimeout = setTimeout(() => {
        processCompleteCode(inputBuffer);
        inputBuffer = "";
        inputTimeout = null;
      }, 100); // 100ms de delay para capturar el código completo
    };

    const handlePasteEvent = (e: ClipboardEvent) => {
      const pastedText = e.clipboardData?.getData("text") || "";

      // ✅ CRÍTICO: Marcar que estamos procesando un paste para evitar duplicación
      isProcessingPaste = true;

      // Limpiar buffer y timeout anteriores
      inputBuffer = "";
      if (inputTimeout) {
        clearTimeout(inputTimeout);
        inputTimeout = null;
      }

      // ✅ CAMBIO CRÍTICO: Solo procesar a través de processCompleteCode, no directamente
      setTimeout(() => {
        processCompleteCode(pastedText);
        // ✅ CRÍTICO: Después de procesar, permitir input events nuevamente
        setTimeout(() => {
          isProcessingPaste = false;
        }, 200); // Delay para asegurar que el input event no interfiera
      }, 50);
    };

    // Escuchar tanto 'input' como 'paste' para capturar pegado por Ctrl+V
    input.addEventListener("input", handleInputEvent);
    input.addEventListener("paste", handlePasteEvent);

    return () => {
      input.removeEventListener("input", handleInputEvent);
      input.removeEventListener("paste", handlePasteEvent);

      // Limpiar timeout si existe
      if (inputTimeout) {
        clearTimeout(inputTimeout);
      }
    };
  }, [availableProducts, searchInputRef]);

  // Exponer funciones vía ref
  useImperativeHandle(
    ref,
    () => ({
      autoAddScannedProduct: (product: any, quantity: number) => {
        addScannedProduct(product, quantity);
      },
    }),
    [addScannedProduct]
  );

  // Mostrar loading mientras se carga la información del negocio
  if (businessInfoLoading) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-600"></div>
          <p className="text-gray-600">Cargando configuración del negocio...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full p-6">
      {/* Header section */}
      <div className="flex w-full items-center mb-4 gap-2 justify-start">
        <div className="flex items-center gap-1">
          <CartTabs
            screens={cartState.screens}
            activeScreen={cartState.activeScreen}
            onChangeScreen={handleChangeScreen} // ✅ CAMBIAR: Usar handleChangeScreen en lugar de cartState.setActiveScreen
            onAddScreen={handleAddScreen}
            onDeleteScreen={handleDeleteScreen}
          />
        </div>

        <HeaderActions
          onOrdersClick={() => setOrdersDialogOpen(true)}
          onClosingClick={() => closing.setClosingDialogOpen(true)}
          user={user}
          userMenuUser={{
            nombre: user?.nombre || "",
            email: user?.email || "",
          }}
          businessInfo={businessInfo}
        />
      </div>

      {/* Items list */}
      <div className="flex-1 space-y-2 overflow-auto mb-4">
        {/* {cancellationControl.isCancellationEnabled &&
          cartState.getCurrentItems().length > 0 && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-3 mb-4">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                <p className="text-sm text-blue-700">
                  <strong>Modo de cancelación controlada:</strong> Solo puedes
                  cancelar toda la orden. Los items individuales no se pueden
                  eliminar.
                </p>
              </div>
            </div>
          )} */}

        {cartState
          .getCurrentItems()
          .slice()
          .reverse()
          .map((item) => (
            <CartItem
              key={item.cartId}
              item={item}
              onRemove={cartState.removeFromCart}
              isCancellationEnabled={cancellationControl.isCancellationEnabled}
            />
          ))}
      </div>

      {/* Cart summary */}
      <CartSummary
        total={Number(cartState.calculateTotalWithIVA().toFixed(2))}
        onCancel={handleCancelClick}
        onCheckout={
          businessInfo?.facturacionHabilitada && businessInfo?.afipHabilitado
            ? handleAfipPaymentClick
            : handlePaymentClick
        }
        facturacionHabilitada={businessInfo?.facturacionHabilitada || false}
      />

      {/* Dialogs */}
      <AddProductDialog
        isOpen={dialogOpen}
        product={selectedProduct}
        onClose={() => setDialogOpen(false)}
        onAddToCart={handleAddToCart}
      />

      <ScanListaPrecioDialog
        isOpen={scanListaDialog?.open ?? false}
        product={scanListaDialog?.product ?? null}
        rows={scanListaDialog?.rows ?? []}
        quantity={scanListaDialog?.quantity ?? 1}
        onClose={handleScanListaClose}
        onAddToCart={handleScanListaAdd}
      />

      <CancelDialog
        open={cancelDialogOpen}
        onOpenChange={setCancelDialogOpen}
        onConfirm={handleCancelCart}
        isLoading={isProcessingPayment}
      />

      <PaymentDialog
        isOpen={afipPaymentDialogOpen}
        onClose={() => {
          setAfipPaymentDialogOpen(false);
          afipPaymentProcessor.resetPaymentState();
          // NO tocar los estados locales del flujo normal

          focusSearchInput("cerrado AFIP payment dialog");
        }}
        onSelectPayment={handleAfipPayment}
        isProcessingPayment={afipPaymentProcessor.isProcessingPayment}
        selectedPaymentMethod={afipPaymentProcessor.selectedPaymentMethod}
        isAfipMode={true}
        discountData={discountData}
        subtotal={cartState.calculateTotal()}
      />

      {/* Diálogo de descuento */}
      <DiscountDialog
        open={discountDialogOpen}
        onOpenChange={setDiscountDialogOpen}
        subtotal={cartState.calculateTotal()}
        onConfirm={handleDiscountConfirm}
        onCancel={handleDiscountCancel}
      />

      {/* Diálogo de pago normal con descuento */}
      <PaymentDialog
        isOpen={paymentDialogOpen}
        onClose={() => {
          // ✅ RESETEAR ESTADOS INMEDIATAMENTE
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);

          setPaymentDialogOpen(false);
          paymentProcessor.resetPaymentState();

          focusSearchInput("cerrado payment dialog");
        }}
        onSelectPayment={handlePayment}
        isProcessingPayment={isProcessingPayment}
        selectedPaymentMethod={selectedPaymentMethod}
        isAfipMode={false}
        discountData={discountData}
        subtotal={cartState.calculateTotal()}
      />

      {/* Diálogo de efectivo para AFIP */}
      <CashPaymentDialog
        open={afipPaymentProcessor.roundedAmountDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            afipPaymentProcessor.setRoundedAmountDialogOpen(false);
            afipPaymentProcessor.resetPaymentState();
            focusSearchInput("cerrado AFIP cash payment dialog");
          }
        }}
        isProcessingPayment={afipPaymentProcessor.isProcessingPayment}
        isLoading={afipPaymentProcessor.isProcessingPayment}
        applyingDiscount={afipPaymentProcessor.applyingDiscount}
        businessInfo={businessInfo}
        originalAmount={afipPaymentProcessor.originalAmount}
        roundedAmount={afipPaymentProcessor.roundedAmount}
        onApplyDiscount={() => {
          if (!businessInfo) return;
          afipPaymentProcessor.setRoundedAmountDialogOpen(false);
          setTimeout(() => {
            afipPaymentProcessor.handleAfipCashPayment(businessInfo, true, discountData);
          }, 100);
        }}
        onConfirm={async () => {
          try {
            const subtotalOriginal = cartState.calculateTotal();
            const discountAmount = Number(
              (subtotalOriginal - afipPaymentProcessor.roundedAmount).toFixed(2)
            );

            let afipDiscountData = null;
            if (afipPaymentProcessor.applyingDiscount && discountData) {
              afipDiscountData = {
                type: discountData.type,
                value: discountData.value,
                amount: discountAmount,
              };
            } else if (
              afipPaymentProcessor.applyingDiscount &&
              businessInfo?.descuentoEfectivo
            ) {
              afipDiscountData = {
                type: "percentage" as const,
                value: Number(businessInfo.descuentoEfectivo),
                amount: discountAmount,
              };
            }

            await afipPaymentProcessor.processAfipPayment(
              "efectivo",
              cartState.getCurrentItems(),
              afipPaymentProcessor.roundedAmount,
              afipDiscountData
            );
          } catch (error) {
            console.error("❌ AFIP: Error al procesar pago redondeado:", error);
          }
        }}
        onCancel={() => {
          afipPaymentProcessor.setRoundedAmountDialogOpen(false);
          afipPaymentProcessor.resetPaymentState();
        }}
      />

      {/* Diálogo de pago exacto para AFIP */}
      <ExactPaymentDialog
        open={afipPaymentProcessor.exactPaymentDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            afipPaymentProcessor.setExactPaymentDialogOpen(false);
            afipPaymentProcessor.resetPaymentState();
            focusSearchInput("cerrado AFIP exact payment dialog");
          }
        }}
        totalAmount={afipPaymentProcessor.roundedAmount}
        isLoading={afipPaymentProcessor.isProcessingPayment}
        // Agregando props de descuento para AFIP ExactPaymentDialog
        discountData={discountData}
        subtotal={cartState.calculateTotal()}
        onConfirm={async (paidAmount: number, change: number) => {
          try {
            await afipPaymentProcessor.confirmAfipExactPayment(
              paidAmount,
              change,
              cartState.getCurrentItems(),
              discountData
            );

            focusSearchInput("AFIP exact payment confirmado");
          } catch (error) {
            console.error("❌ AFIP EXACT PAYMENT: Error al procesar:", error);
          }
        }}
        onCancel={() => {
          afipPaymentProcessor.setExactPaymentDialogOpen(false);
          afipPaymentProcessor.resetPaymentState();
        }}
      />

      <CashPaymentDialog
        open={paymentProcessor.roundedAmountDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            paymentProcessor.setRoundedAmountDialogOpen(false);
            paymentProcessor.resetPaymentState();
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
            focusSearchInput("cerrado cash payment dialog");
          }
        }}
        isProcessingPayment={isProcessingPayment}
        isLoading={isProcessingPayment}
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
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
        }}
      />

      <ExactPaymentDialog
        open={paymentProcessor.exactPaymentDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            paymentProcessor.setExactPaymentDialogOpen(false);
            paymentProcessor.resetPaymentState();
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
            focusSearchInput("cerrado exact payment dialog");
          }
        }}
        totalAmount={paymentProcessor.roundedAmount}
        isLoading={isProcessingPayment}
        // Agregando props de descuento para ExactPaymentDialog normal
        discountData={discountData}
        subtotal={cartState.calculateTotal()}
        onConfirm={async (paidAmount: number, change: number) => {
          setIsProcessingPayment(true);
          try {
            await paymentProcessor.confirmExactPayment(
              paidAmount,
              change,
              cartState.getCurrentItems(),
              discountData
            );

            // Limpiar estados locales después del procesamiento exitoso
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
            focusSearchInput("exact payment confirmado");
          } catch (error) {
            console.error("❌ EXACT PAYMENT: Error al procesar:", error);
            setIsProcessingPayment(false);
          }
        }}
        onCancel={() => {
          paymentProcessor.setExactPaymentDialogOpen(false);
          paymentProcessor.resetPaymentState();
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
        }}
      />

      <CancellationDialog
        open={cancellationControl.cancellationDialogOpen}
        onOpenChange={cancellationControl.setCancellationDialogOpen}
        onConfirm={handleConfirmCancellation}
        onCancel={handleCancelCancellationProcess}
        reason={cancellationControl.cancellationReason}
        onReasonChange={cancellationControl.setCancellationReason}
        isCancelling={cancellationControl.isCancelling}
        pendingCancellation={cancellationControl.pendingCancellation}
      />

      <ClosingDialog
        open={closing.closingDialogOpen}
        onOpenChange={closing.setClosingDialogOpen}
        onHandleClosing={closing.handleClosing}
        isClosing={closing.isClosing}
        searchInputRef={searchInputRef}
      />

      <QRPaymentDialog
        open={qrDialogOpen}
        onOpenChange={(open) => {
          setQrDialogOpen(open);
          // ✅ RESETEAR: Flujo AFIP cuando se cierra el diálogo QR
          if (!open) {
            setIsCurrentlyAfipFlow(false);
          }
        }}
        paymentProcessor={paymentProcessor}
        afipPaymentProcessor={afipPaymentProcessor}
        isAfipMode={isCurrentlyAfipFlow}
        cartItems={cartState.getCurrentItems()}
        businessInfo={businessInfo}
        // ✅ CRÍTICO: Pasar datos de descuento para mostrar en el QR
        discountData={discountData}
        subtotal={cartState.calculateTotal()}
        // ✅ DEBUGGING: Pasar estado adicional para logging
        debugInfo={{
          isCurrentlyAfipFlow,
          facturacionHabilitada: businessInfo?.facturacionHabilitada,
          activeScreen: cartState.activeScreen,
          componentName: "ShoppingCartRefactored",
        }}
      />

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
        onSubmit={() =>
          paymentProcessor.handleManualQrPasswordSubmit(isCurrentlyAfipFlow)
        }
        isLoading={paymentProcessor.isManualPasswordSubmitting}
      />

      {/* NUEVO: Diálogo de contraseña QR manual para AFIP */}
      <ManualQrDialog
        open={afipPaymentProcessor.manualQrPasswordDialogOpen}
        onOpenChange={(open: boolean) => {
          if (!open) {
            afipPaymentProcessor.setManualQrPasswordDialogOpen(false);
            afipPaymentProcessor.setManualQrPassword("");
          }
        }}
        password={afipPaymentProcessor.manualQrPassword}
        onPasswordChange={(value: string) =>
          afipPaymentProcessor.setManualQrPassword(value)
        }
        onSubmit={() => afipPaymentProcessor.handleManualQrPasswordSubmit()}
        isLoading={afipPaymentProcessor.isManualPasswordSubmitting}
      />

      <SplitPaymentDialog
        open={splitPaymentDialogOpen}
        onOpenChange={setSplitPaymentDialogOpen}
        paymentProcessor={paymentProcessor}
        cartState={cartState}
        businessInfo={businessInfo}
        searchInputRef={searchInputRef}
        discountData={discountData}
      />

      {/* NUEVO: Diálogo de pago mixto para AFIP */}
      <AfipSplitPaymentDialog
        open={afipSplitPaymentDialogOpen}
        onOpenChange={setAfipSplitPaymentDialogOpen}
        afipPaymentProcessor={afipPaymentProcessor}
        cartState={cartState}
        businessInfo={businessInfo}
        searchInputRef={searchInputRef}
        discountData={discountData}
      />

      {/* Diálogo de factura */}
      <FacturaForm
        isOpen={facturaDialogOpen}
        onClose={handleFacturaClose}
        onSuccess={handleFacturaSuccess}
        productosIniciales={cartState.getCurrentItems().map((item) => ({
          id: item.id,
          nombre: item.name,
          precio: item.pricePerUnit,
          cantidad: item.quantity,
          subtotal: item.subtotal,
        }))}
      />

      <RecentOrdersDialog
        isOpen={ordersDialogOpen}
        onClose={() => setOrdersDialogOpen(false)}
        API_URL={API_URL}
        appId={getAppId()}
        formatFechaArgentina={formatFechaArgentina}
        businessInfo={businessInfo}
      />
    </div>
  );
});

function getAppId() {
  return (
    (window as any).electron?.process?.argv
      ?.find((arg: string) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null
  );
}

export default ShoppingCartRefactored;
