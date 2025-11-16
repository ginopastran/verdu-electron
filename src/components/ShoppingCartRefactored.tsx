import {
  useState,
  useEffect,
  useRef,
  forwardRef,
  useImperativeHandle,
  useCallback,
} from "react";
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
    // console.log(
    //   "🔄 SYNC: Actualizando isProcessingPayment local:",
    //   paymentProcessor.isProcessingPayment
    // );
    setIsProcessingPayment(paymentProcessor.isProcessingPayment);
  }, [paymentProcessor.isProcessingPayment]);

  useEffect(() => {
    // console.log(
    //   "🔄 SYNC: Actualizando selectedPaymentMethod local:",
    //   paymentProcessor.selectedPaymentMethod
    // );
    setSelectedPaymentMethod(paymentProcessor.selectedPaymentMethod);
  }, [paymentProcessor.selectedPaymentMethod]);

  // 🆕 NUEVO: Effect para limpiar estados locales cuando el hook se resetea
  useEffect(() => {
    // Si el hook no tiene método seleccionado ni está procesando, limpiar estados locales
    if (
      !paymentProcessor.selectedPaymentMethod &&
      !paymentProcessor.isProcessingPayment
    ) {
      // console.log("🧹 SYNC: Hook reseteado, limpiando estados locales");
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
      console.log("🚪 ABRIR: Diálogo de pago abierto, reseteando estados");
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      paymentProcessor.resetPaymentState();
    }
  }, [paymentDialogOpen]);

  // 🆕 NUEVO: Effect para resetear estados cuando se abre el diálogo AFIP
  useEffect(() => {
    if (afipPaymentDialogOpen) {
      console.log("🚪 ABRIR: Diálogo AFIP abierto, reseteando estados");
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
        console.log(
          "🧹 SYNC CLEANUP: Diálogos cerrados pero estados activos, limpiando:",
          {
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
          }
        );

        // Limpiar estados locales
        setIsProcessingPayment(false);
        setSelectedPaymentMethod(null);

        // Asegurar que el hook también esté limpio
        if (
          paymentProcessor.isProcessingPayment ||
          paymentProcessor.selectedPaymentMethod
        ) {
          console.log("🧹 SYNC CLEANUP: También limpiando estados del hook");
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
      console.log("🔄 Cerrando PaymentDialog porque se abrió QRDialog");
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
    console.log("🔧 DEBUG: 🎯 INICIO handleProductSelect");
    console.log("🔧 DEBUG: Producto recibido:", product);

    if (!product) {
      console.log("🔧 DEBUG: ❌ Producto es null/undefined, retornando");
      return;
    }

    console.log("🔧 DEBUG: Producto seleccionado:", product.name);
    console.log("🔧 DEBUG: Estableciendo selectedProduct y abriendo diálogo");
    setSelectedProduct(product);
    setDialogOpen(true);
    console.log("🔧 DEBUG: ✅ FIN handleProductSelect - Diálogo abierto");
  };

  // Handler para agregar un producto al carrito
  const handleAddToCart = (product: any) => {
    cartState.addToCart(product);
    setDialogOpen(false);
    setSelectedProduct(null);

    // Enfocar el input de búsqueda
    focusSearchInput("agregado producto");
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
      console.log("🔄 Cancelaciones habilitadas - usando flujo controlado");

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

      console.log(
        "📋 Productos preparados para cancelación:",
        productosCancelacion
      );

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
    console.log("🔄 Usando flujo normal de cancelación");
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

      console.log("✅ Cancelación registrada y carrito limpiado");
    }

    return success || false;
  };

  // Handler para cancelar el proceso de cancelación controlada
  const handleCancelCancellationProcess = () => {
    cancellationControl.cancelCancellationProcess();
    console.log("❌ Proceso de cancelación cancelado por el usuario");
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

    console.log(
      "💰 handlePaymentClick - Artículos en carrito (F3 - Sin AFIP):",
      currentItems
    );

    // Resetear estados del flujo AFIP al abrir diálogo normal
    console.log("🧹 Reseteando estados del flujo AFIP antes de pago normal");
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

    console.log("🧾 Abriendo diálogo de pago AFIP (F2 - Con AFIP)");

    // Resetear estados del flujo normal al abrir AFIP
    console.log("🧹 Reseteando estados del flujo normal antes de AFIP");
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

    console.log(
      "📄 Abriendo diálogo de factura (F6) con productos del carrito:",
      currentItems
    );
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

    console.log(
      "💰🏷️ handleDiscountPaymentClick - Artículos en carrito (F7 - Sin AFIP con descuento):",
      currentItems
    );

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

    console.log(
      "🧾🏷️ Abriendo diálogo de pago AFIP con descuento (F8 - Con AFIP con descuento)"
    );

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
    console.log("🧾 Procesando pago AFIP con método:", method);

    try {
      // Si es efectivo, usar el handler especial como en el flujo normal
      if (method === "efectivo") {
        console.log("🧾 AFIP: Efectivo detectado - usando handler especial");
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
        console.log("🧾📱 AFIP: QR detectado - usando flujo híbrido");
        console.log(
          "🧾📱 AFIP: Verificando configuración MP:",
          businessInfo?.mpEnabled
        );

        // ✅ VERIFICAR SI MP ESTÁ HABILITADO IGUAL QUE EN FLUJO NORMAL
        if (businessInfo?.mpEnabled === false) {
          console.log(
            "🧾📱 AFIP: MP deshabilitado - procesando como transferencia directa con factura"
          );

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
        console.log("🧾📱 AFIP: MP habilitado - generando QR de MercadoPago");
        const qrData = await afipPaymentProcessor.handleAfipQrPayment(
          cartState.getCurrentItems(),
          discountData
        );

        if (qrData) {
          // ✅ CRÍTICO: Actualizar estado del QR en el hook principal
          console.log(
            "🧾📱 AFIP: Actualizando estado del QR con datos:",
            qrData
          );

          // ✅ CORRECCIÓN: Agregar el monto desde el carrito ya que el backend no lo incluye
          const qrDataWithAmount = {
            ...qrData,
            monto: Number(cartState.calculateTotal().toFixed(2)),
          };

          console.log("🧾📱 AFIP: QR con monto agregado:", qrDataWithAmount);
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
        console.log("🧾 AFIP: Pago mixto seleccionado");
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

      console.log("🛒 Error en pago AFIP - carrito mantenido para reintento");
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

    console.log("🔄 handlePayment llamado con método:", method);
    console.log("🔄 Estados ANTES de procesar:", {
      isProcessingPayment,
      selectedPaymentMethod,
      hookIsProcessing: paymentProcessor.isProcessingPayment,
      hookSelectedMethod: paymentProcessor.selectedPaymentMethod,
    });

    // Prevenir procesamiento duplicado usando estado local antes de cualquier acción
    if (isProcessingPayment) {
      console.log("⚠️ Procesamiento bloqueado - ya está procesando");
      paymentLockRef.current = false;
      return;
    }

    // Si es efectivo, manejar con el sistema de redondeo. Pasa el control.
    if (method === "efectivo") {
      console.log("💰 Seleccionando efectivo - businessInfo:", businessInfo);
      console.log("💰 Descuento disponible:", discountData);
      // NO establecer isProcessingPayment aquí, solo cuando se confirme el pago
      setSelectedPaymentMethod("efectivo");
      paymentProcessor.handleCashPayment(businessInfo, !!discountData, discountData);
      setPaymentDialogOpen(false);
      paymentLockRef.current = false;
      return;
    }

    // Si es pago dividido, preparar pago mixto. Pasa el control.
    if (method === "split") {
      console.log(
        "🔄 Iniciando pago mixto desde ShoppingCartRefactored (F3 - Sin AFIP)"
      );
      console.log(
        "🔄 Estado isCurrentlyAfipFlow ANTES de mixto:",
        isCurrentlyAfipFlow
      );

      // ✅ ASEGURAR: En F3, el pago mixto NO debe activar AFIP
      if (!isCurrentlyAfipFlow) {
        console.log(
          "✅ Confirmando que pago mixto permanece en modo F3 (sin AFIP)"
        );
      }

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
          businessName: await getBusinessName(),
          estado: "COMPLETADA",
          createdAt: new Date().toISOString(),
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
          console.log("📋 Respuesta del API (Shopping Cart):", orderResult);

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
            console.log(
              "🛒 Carrito limpiado después de pago QR directo exitoso"
            );
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

          console.log("🛒 Error en pago QR - carrito mantenido para reintento");
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

    // Para otros métodos (tarjeta), procesar directamente
    try {
      await paymentProcessor.processPayment(
        method,
        Number(cartState.calculateTotalWithIVA().toFixed(2)),
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

      console.log("🛒 Error en pago normal - carrito mantenido para reintento");
    } finally {
      // ✅ MEJORADO: Asegurar que el estado se resetee SIEMPRE
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      paymentLockRef.current = false;
    }
  };

  // Confirmar pago redondeado en efectivo
  const confirmRoundedPayment = async () => {
    console.log("🔄 =========================");
    console.log("🔄 REFACTORED: Iniciando confirmRoundedPayment");
    console.log("🔄 Estado actual detallado:", {
      isProcessingPayment,
      selectedPaymentMethod,
      roundedAmount: paymentProcessor.roundedAmount,
      originalAmount: paymentProcessor.originalAmount,
      user: user?.id,
      roundedAmountDialogOpen: paymentProcessor.roundedAmountDialogOpen,
    });

    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso, ignorando solicitud");
      return;
    }

    if (!user) {
      console.log("⚠️ Usuario no está logueado");
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    if (paymentProcessor.roundedAmount <= 0) {
      console.log(
        "⚠️ Monto redondeado es 0 o negativo:",
        paymentProcessor.roundedAmount
      );
      toast.error("Error: Monto inválido");
      return;
    }

    console.log(
      "✅ Validaciones pasadas, procesando pago con monto:",
      paymentProcessor.roundedAmount
    );

    setIsProcessingPayment(true);

    try {
      console.log("🔄 Llamando a paymentProcessor.processPayment");
      
      // Calcular el discountAmount si hay discountData
      let finalDiscountData = discountData;
      if (discountData) {
        const subtotalOriginal = cartState.calculateTotal();
        const discountAmount = subtotalOriginal - paymentProcessor.roundedAmount;
        finalDiscountData = {
          ...discountData,
          amount: discountAmount
        };
        console.log("💰 ROUNDED PAYMENT: Calculando discount amount:", {
          subtotalOriginal,
          roundedAmount: paymentProcessor.roundedAmount,
          discountAmount,
          finalDiscountData
        });
      }
      
      await paymentProcessor.processPayment(
        "efectivo",
        paymentProcessor.roundedAmount,
        cartState.getCurrentItems(),
        finalDiscountData
      );
      console.log("🔄 processPayment llamado exitosamente");

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
    console.log("🔄 =========================");
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
    (product: any, quantity: number = 1) => {
      // ✅ BLOQUEO INMEDIATO: Si ya estamos procesando, salir
      if (isProcessingRef.current) {
        console.log("🔒 DEBUG: 🚫 BLOQUEADO - Procesamiento en curso");
        return;
      }

      // ✅ BLOQUEO INMEDIATO: Marcar como procesando
      isProcessingRef.current = true;
      console.log("🔒 DEBUG: 🔒 INICIO - Marcando como procesando");

      try {
        // ✅ VALIDACIÓN DE DUPLICADOS: Verificar código reciente
        const now = Date.now();
        const productKey = `${product.id}-${product.name}`;

        if (
          lastProcessedCode.current &&
          (lastProcessedCode.current.code === productKey ||
            lastProcessedCode.current.code === product.codigoBarras ||
            lastProcessedCode.current.code === product.plu) &&
          now - lastProcessedCode.current.timestamp < 1000 // 1 segundo de debounce
        ) {
          console.log(
            "🔒 DEBUG: ⚠️ PRODUCTO DUPLICADO - Ignorando:",
            productKey
          );
          return;
        }

        // ✅ MARCAR COMO PROCESADO
        lastProcessedCode.current = { code: productKey, timestamp: now };
        console.log(
          "🔒 DEBUG: ✅ Producto marcado como procesado:",
          productKey
        );

        // ✅ CREAR ITEM ÚNICO
        const uniqueId = `${product.id}-${Date.now()}-${Math.random()
          .toString(36)
          .substring(2, 10)}`;
        const newItem: Product = {
          id: product.id,
          cartId: uniqueId,
          name: product.name,
          quantity: quantity,
          unit: product.unit || "Unidad",
          pricePerUnit: product.pricePerUnit,
          subtotal: Number((product.pricePerUnit * quantity).toFixed(2)),
          costo: product.costo,
          ivaIncluido: product.ivaIncluido,
          ivaPorcentaje: product.ivaPorcentaje,
        };

        console.log("🔒 DEBUG: 🛒 Agregando producto al carrito:", newItem);
        cartState.addToCart(newItem);
        console.log("🔒 DEBUG: ✅ Producto agregado exitosamente");

        // ✅ LIMPIAR INPUT
        if (searchInputRef.current) {
          searchInputRef.current.value = "";
          setSearchQuery("");
          searchInputRef.current.focus();
        }
      } catch (error) {
        console.error("🔒 DEBUG: ❌ Error al agregar producto:", error);
      } finally {
        // ✅ DESBLOQUEO: Siempre liberar el bloqueo
        isProcessingRef.current = false;
        console.log("🔒 DEBUG: 🔓 FIN - Bloqueo liberado");
      }
    },
    [cartState]
  );

  // ✅ CRÍTICO: Función para procesar códigos completos
  const processCompleteCode = useCallback(
    (completeCode: string) => {
      // ✅ BLOQUEO INMEDIATO: Si ya estamos procesando, salir
      if (isProcessingRef.current) {
        console.log("🔒 DEBUG: 🚫 BLOQUEADO - Procesamiento en curso");
        return;
      }

      console.log("🔧 DEBUG: 🎯 INICIO processCompleteCode");
      console.log("🔧 DEBUG: Código a procesar:", completeCode);
      console.log("🔧 DEBUG: Longitud del código:", completeCode.length);

      // ✅ VALIDACIÓN: Productos cargados
      if (!availableProducts || availableProducts.length === 0) {
        console.log("🔧 DEBUG: ❌ Productos aún no cargados, esperando...");
        return;
      }

      // ✅ VALIDACIÓN: Código no vacío
      if (!completeCode || completeCode.trim().length === 0) {
        console.log("🔧 DEBUG: ❌ Código vacío, ignorando");
        return;
      }

      // ✅ DEBUG: Estado del carrito antes de procesar
      const cartBefore = cartState.getCurrentItems();
      console.log("🔧 DEBUG: Estado del carrito antes de procesar:", {
        count: cartBefore.length,
        items: cartBefore.map((item) => ({
          id: item.id,
          name: item.name,
          cartId: item.cartId,
        })),
      });

      // ✅ PATRÓN 1: Código de barras estándar (13 dígitos)
      if (completeCode.length === 13 && /^\d{13}$/.test(completeCode)) {
        console.log(
          "🔧 DEBUG: 🔍 Verificando si es código de barras estándar..."
        );

        const productsWithBarcode = availableProducts.filter(
          (p) => p.codigoBarras
        );
        console.log(
          "🔧 DEBUG: Productos con código de barras:",
          productsWithBarcode.length
        );

        const product = productsWithBarcode.find(
          (p) => p.codigoBarras === completeCode
        );

        if (product) {
          console.log(
            "🔧 DEBUG: ✅ Producto encontrado por código de barras:",
            {
              id: product.id,
              name: product.name,
              codigoBarras: product.codigoBarras,
              precio: product.pricePerUnit,
              cantidad: 1,
            }
          );

          // ✅ DEBUG: Estado del carrito ANTES de agregar código de barras
          const itemsBeforeAdd = cartState.getCurrentItems();
          console.log(
            "🔧 DEBUG: Items en carrito ANTES de agregar código de barras:",
            {
              count: itemsBeforeAdd.length,
              items: itemsBeforeAdd.map((item) => ({
                id: item.id,
                name: item.name,
                cartId: item.cartId,
              })),
            }
          );

          // ✅ AGREGAR PRODUCTO usando la función centralizada
          addScannedProduct(product, 1);

          // ✅ DEBUG: Estado del carrito después de agregar código de barras
          setTimeout(() => {
            const itemsAfterAdd = cartState.getCurrentItems();
            console.log(
              "🔧 DEBUG: Items en carrito DESPUÉS de agregar código de barras:",
              {
                count: itemsAfterAdd.length,
                items: itemsAfterAdd.map((item) => ({
                  id: item.id,
                  name: item.name,
                  cartId: item.cartId,
                })),
              }
            );

            // ✅ Verificar que el producto fue agregado correctamente
            if (itemsAfterAdd.length > 0) {
              const lastAdded = itemsAfterAdd[itemsAfterAdd.length - 1];
              console.log("🔧 DEBUG: ✅ Último producto agregado:", {
                name: lastAdded.name,
                cartId: lastAdded.cartId,
                totalItems: itemsAfterAdd.length,
              });
            }
          }, 150);
          return;
        } else {
          console.log(
            "🔧 DEBUG: ❌ No se encontró producto con código de barras:",
            completeCode
          );
          return;
        }
      }

      // ✅ PATRÓN 2: PLU + peso (12 o 13 dígitos)
      if (completeCode.length === 12 || completeCode.length === 13) {
        console.log("🔧 DEBUG: 🔍 Verificando si es PLU + peso...");

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

        console.log(`🔧 DEBUG: PLU extraído: ${plu}`);
        console.log(`🔧 DEBUG: Peso extraído: ${weightStr}`);
        console.log(`🔧 DEBUG: Formato usado: ${formatUsed}`);

        // ✅ VALIDACIÓN: PLU debe ser numérico
        if (!/^\d{3}$/.test(plu)) {
          console.log("🔧 DEBUG: ❌ PLU inválido:", plu);
          console.log(`   - Debe ser exactamente 3 dígitos numéricos`);
          console.log(`   - Formato usado: ${formatUsed}`);
          return;
        }

        // ✅ VALIDACIÓN: Peso debe ser numérico
        if (!/^\d{8}$/.test(weightStr)) {
          console.log("🔧 DEBUG: ❌ Peso inválido:", weightStr);
          console.log(`   - Debe ser exactamente 8 dígitos numéricos`);
          console.log(`   - Formato usado: ${formatUsed}`);
          return;
        }

        // ✅ CONVERTIR peso a kilogramos
        const weightGrams = parseInt(weightStr, 10);
        const kgQuantity = weightGrams / 1000; // Convertir gramos a kilogramos

        console.log(`🔧 DEBUG: Peso en gramos: ${weightGrams}`);
        console.log(`🔧 DEBUG: Peso en kilogramos: ${kgQuantity}`);

        // ✅ VALIDACIÓN: Rango de peso válido
        if (kgQuantity < 0.001 || kgQuantity > 999.999) {
          console.log("🔧 DEBUG: ❌ Peso fuera de rango válido:", kgQuantity);
          console.log(`   - Rango válido: 0.001 - 999.999 kg`);
          console.log(`   - Formato usado: ${formatUsed}`);
          return;
        }

        console.log(`🔧 DEBUG: 🔍 Buscando producto con PLU: ${plu}`);
        console.log(
          `🔧 DEBUG: Productos disponibles con PLU:`,
          availableProducts
            .filter((p) => p.plu)
            .map((p) => ({ name: p.name, plu: p.plu, id: p.id }))
        );

        const productByPlu: any = availableProducts.find((p: any) => {
          if (p.plu === null || p.plu === undefined) return false;
          return Number(p.plu) === Number(plu);
        });

        if (productByPlu) {
          console.log("🔧 DEBUG: ✅ Producto encontrado por PLU:", {
            id: productByPlu.id,
            name: productByPlu.name,
            plu: productByPlu.plu,
            precio: productByPlu.pricePerUnit,
            cantidad: kgQuantity,
            formato: formatUsed,
            codigoOriginal: completeCode,
          });

          // ✅ DEBUG: Estado del carrito ANTES de agregar PLU+peso
          const itemsBeforeAdd = cartState.getCurrentItems();
          console.log("🔧 DEBUG: Items en carrito ANTES de agregar PLU+peso:", {
            count: itemsBeforeAdd.length,
            items: itemsBeforeAdd.map((item) => ({
              id: item.id,
              name: item.name,
              cartId: item.cartId,
            })),
          });

          // ✅ AGREGAR PRODUCTO usando la función centralizada
          addScannedProduct(productByPlu, kgQuantity);

          // ✅ DEBUG: Estado del carrito después de agregar PLU+peso
          setTimeout(() => {
            const itemsAfterAdd = cartState.getCurrentItems();
            console.log(
              "🔧 DEBUG: Items en carrito DESPUÉS de agregar PLU+peso:",
              {
                count: itemsAfterAdd.length,
                items: itemsAfterAdd.map((item) => ({
                  id: item.id,
                  name: item.name,
                  cartId: item.cartId,
                  cantidad: item.quantity,
                  subtotal: item.subtotal,
                })),
              }
            );
          }, 100);
        } else {
          console.log("🔧 DEBUG: ❌ No se encontró producto con PLU:", plu);
          console.log(`   - Formato usado: ${formatUsed}`);
          console.log(`   - Código original: ${completeCode}`);
          console.log(
            "🔧 DEBUG: Productos disponibles con PLU:",
            availableProducts
              .filter((p) => p.plu)
              .map((p) => ({ name: p.name, plu: p.plu, id: p.id }))
          );
        }
      } else {
        // ✅ NUEVO: Solo procesar códigos que tengan el formato correcto
        // NO procesar códigos parciales que coincidan con PLUs existentes
        console.log("🔧 DEBUG: ❌ No coincide con ningún patrón PLU + peso");
        console.log(`   - Código ingresado: ${completeCode}`);
        console.log(`   - Longitud: ${completeCode.length} dígitos`);
        console.log(
          `   - Formatos soportados: 12 dígitos (XXX + 8 dígitos peso + 1) o 13 dígitos (0 + XXX + 8 dígitos peso + 1)`
        );
        console.log(
          `   - ⚠️ Código ignorado: Solo se procesan códigos completos con formato PLU + peso`
        );
      }

      console.log("🔧 DEBUG: 🏁 FIN processCompleteCode");
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
      console.log(
        "🧹 QR Dialog cerrado - limpiando tracking de órdenes procesadas"
      );
      // Limpiar tracking en ambos hooks
      paymentProcessor.clearProcessedOrdersTracking?.();
      afipPaymentProcessor.clearProcessedOrdersTracking?.();

      // ✅ NUEVO: NO limpiar el polling automáticamente al cerrar el diálogo
      // El polling debe continuar hasta que el pago se complete o falle
      console.log(
        "🔄 QR Dialog cerrado - manteniendo polling activo para detectar pago"
      );

      // ✅ CRÍTICO: NO limpiar el carrito automáticamente al cerrar el diálogo QR
      // El carrito se debe limpiar solo cuando el pago se complete exitosamente
      console.log("🛒 QR Dialog cerrado - manteniendo carrito intacto");
    }
  }, [qrDialogOpen, paymentProcessor, afipPaymentProcessor]);

  // 🆕 NUEVO: Monitorear estado de diálogos de contraseña manual
  useEffect(() => {
    console.log("🔧 Estado de diálogos de contraseña manual:", {
      normalDialogOpen: paymentProcessor.manualQrPasswordDialogOpen,
      afipDialogOpen: afipPaymentProcessor.manualQrPasswordDialogOpen,
      isCurrentlyAfipFlow,
    });
  }, [
    paymentProcessor.manualQrPasswordDialogOpen,
    afipPaymentProcessor.manualQrPasswordDialogOpen,
    isCurrentlyAfipFlow,
  ]);

  // ✅ CRÍTICO: Limpiar intervalos solo al desmontar el componente
  useEffect(() => {
    return () => {
      // Limpiar intervalos solo al desmontar el componente, no al cambiar estado
      console.log("🧹 Componente desmontándose - limpiando polling");
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
    console.log(
      "🔧 DEBUG: 🚀 Iniciando configuración de listeners para input de búsqueda"
    );

    const input = searchInputRef.current;
    if (!input) {
      console.log("🔧 DEBUG: ❌ searchInputRef.current es null");
      return;
    }

    console.log("🔧 DEBUG: ✅ searchInputRef.current encontrado:", input);
    console.log("🔧 DEBUG: Productos disponibles:", availableProducts.length);

    // ✅ VARIABLE DE CONTROL INMEDIATA - NO estado de React
    let isProcessingPaste = false;

    // Buffer para acumular caracteres del escáner
    let inputBuffer = "";
    let inputTimeout: NodeJS.Timeout | null = null;

    const processCompleteCode = (completeCode: string) => {
      console.log("🔧 DEBUG: 🎯 INICIO processCompleteCode");
      console.log(`🔧 DEBUG: Código a procesar: "${completeCode}"`);
      console.log(`🔧 DEBUG: Longitud del código: ${completeCode.length}`);

      if (!completeCode) {
        console.log("🔧 DEBUG: ❌ Código vacío, no procesando");
        return;
      }

      // Verificar que tengamos productos cargados
      if (!availableProducts || availableProducts.length === 0) {
        console.log("🔧 DEBUG: ❌ Productos aún no cargados, esperando...");
        return;
      }

      // 🛒 DEBUG: Estado actual del carrito antes de procesar
      const currentItems = cartState.getCurrentItems();
      console.log("🔧 DEBUG: Estado del carrito antes de procesar:", {
        itemsCount: currentItems.length,
        items: currentItems.map((item) => ({
          id: item.id,
          name: item.name,
          cartId: item.cartId,
        })),
      });

      // 1. Código de barras estándar (8-15 dígitos) - EAN-8, UPC-A, EAN-13, Code 128, etc.
      console.log(
        "🔧 DEBUG: 🔍 Verificando si es código de barras estándar..."
      );
      if (/^\d{8,15}$/.test(completeCode)) {
        console.log("🔧 DEBUG: ✅ Patrón de código de barras detectado");
        console.log(
          "🔧 DEBUG: Productos con código de barras:",
          availableProducts
            .filter((p) => p.codigoBarras)
            .map((p) => ({ name: p.name, codigoBarras: p.codigoBarras }))
        );

        // Buscar coincidencia exacta
        let product = availableProducts.find(
          (p: any) => p.codigoBarras === completeCode
        );

        // Si no hay coincidencia exacta, buscar si algún código contiene el valor o viceversa
        if (!product) {
          console.log(
            "🔧 DEBUG: No encontrado exacto, buscando coincidencias parciales..."
          );
          product = availableProducts.find(
            (p: any) =>
              p.codigoBarras &&
              (p.codigoBarras.includes(completeCode) ||
                completeCode.includes(p.codigoBarras))
          );
        }

        if (product) {
          console.log(
            "🔧 DEBUG: ✅ Producto encontrado por código de barras:",
            {
              id: product.id,
              name: product.name,
              codigoBarras: product.codigoBarras,
              precio: product.pricePerUnit,
              cantidad: 1,
            }
          );

          // 🛒 DEBUG: Estado del carrito ANTES de agregar código de barras
          const itemsBeforeAdd = cartState.getCurrentItems();
          console.log(
            "🔧 DEBUG: Items en carrito ANTES de agregar código de barras:",
            {
              count: itemsBeforeAdd.length,
              items: itemsBeforeAdd.map((item) => ({
                id: item.id,
                name: item.name,
                cartId: item.cartId,
              })),
            }
          );

          // ✅ USAR SOLO LA FUNCIÓN CENTRALIZADA - NO duplicar lógica
          console.log(
            "🔧 DEBUG: 🛒 Usando addScannedProduct (función centralizada)"
          );
          addScannedProduct(product, 1);
          console.log(
            "🔧 DEBUG: ✅ Producto procesado a través de addScannedProduct"
          );

          // Limpiar completamente el input y el buffer
          console.log(
            "🔧 DEBUG: 🧹 Limpiando input y buffer después de código de barras"
          );
          input.value = "";
          inputBuffer = "";
          if (inputTimeout) {
            clearTimeout(inputTimeout);
            inputTimeout = null;
          }

          // 🛒 DEBUG: Estado del carrito después de agregar código de barras
          setTimeout(() => {
            const itemsAfterAdd = cartState.getCurrentItems();
            console.log(
              "🔧 DEBUG: Items en carrito DESPUÉS de agregar código de barras:",
              {
                count: itemsAfterAdd.length,
                items: itemsAfterAdd.map((item) => ({
                  id: item.id,
                  name: item.name,
                  cartId: item.cartId,
                })),
              }
            );

            // ✅ Verificar que el producto fue agregado correctamente
            if (itemsAfterAdd.length > 0) {
              const lastAdded = itemsAfterAdd[itemsAfterAdd.length - 1];
              console.log("🔧 DEBUG: ✅ Último producto agregado:", {
                name: lastAdded.name,
                cartId: lastAdded.cartId,
                totalItems: itemsAfterAdd.length,
              });
            }
          }, 150); // ✅ AUMENTADO: 150ms para dar más tiempo al estado
          return;
        } else {
          console.log(
            "🔧 DEBUG: ❌ No se encontró producto con código de barras:",
            completeCode
          );
        }
      } else {
        console.log("🔧 DEBUG: ❌ No es código de barras estándar");
      }

      // 2. PLU + peso: Múltiples formatos soportados
      console.log("🔧 DEBUG: 🔍 Verificando si es PLU + peso...");
      // Formato 1: 3 dígitos PLU + 8 dígitos gramos + 1 dígito adicional (12 dígitos total) - "105000013552"
      // Formato 2: 0 + 3 dígitos PLU + 8 dígitos gramos + 1 dígito adicional (13 dígitos total) - "0105000013552"

      let plu = null;
      let grams = null;
      let kgQuantity = null;
      let formatUsed = null;

      // Intentar formato de 12 dígitos primero
      console.log("🔧 DEBUG: 🔍 Probando formato de 12 dígitos...");
      const pluWeightRegex12 = /^(\d{3})(\d{8})(\d{1})$/;
      const match12 = completeCode.match(pluWeightRegex12);

      if (match12) {
        console.log("🔧 DEBUG: ✅ Formato de 12 dígitos detectado");
        plu = match12[1];
        grams = parseInt(match12[2], 10);
        kgQuantity = grams / 1000;
        formatUsed = "12 dígitos";

        console.log(`🔧 DEBUG: PLU + peso detectado (formato ${formatUsed}):`);
        console.log(`   - Código completo: ${completeCode}`);
        console.log(`   - PLU extraído: ${plu}`);
        console.log(`   - Gramos extraídos: ${grams}`);
        console.log(`   - Kilogramos calculados: ${kgQuantity}`);
      } else {
        console.log(
          "🔧 DEBUG: ❌ No es formato de 12 dígitos, probando 13 dígitos..."
        );
        // Intentar formato de 13 dígitos (0 + PLU + peso + dígito final)
        const pluWeightRegex13 = /^0(\d{3})(\d{8})(\d{1})$/;
        const match13 = completeCode.match(pluWeightRegex13);

        if (match13) {
          console.log("🔧 DEBUG: ✅ Formato de 13 dígitos detectado");
          plu = match13[1];
          grams = parseInt(match13[2], 10);
          kgQuantity = grams / 1000;
          formatUsed = "13 dígitos";

          console.log(
            `🔧 DEBUG: PLU + peso detectado (formato ${formatUsed}):`
          );
          console.log(`   - Código completo: ${completeCode}`);
          console.log(`   - PLU extraído: ${plu}`);
          console.log(`   - Gramos extraídos: ${grams}`);
          console.log(`   - Kilogramos calculados: ${kgQuantity}`);
          console.log(`   - Análisis detallado:`);
          console.log(`     * Dígito inicial: 0`);
          console.log(`     * PLU (3 dígitos): ${plu}`);
          console.log(
            `     * Peso (8 dígitos): ${match13[2]} = ${grams}g = ${kgQuantity}kg`
          );
          console.log(`     * Dígito final: ${match13[3]}`);
        } else {
          console.log("🔧 DEBUG: ❌ No es formato de 13 dígitos");
        }
      }

      if (plu && grams !== null && kgQuantity !== null) {
        console.log("🔧 DEBUG: ✅ PLU y peso extraídos correctamente");
        // ✅ CORRECCIÓN: Validar que el peso sea razonable (entre 0.001 y 999.999 kg)
        if (kgQuantity < 0.001 || kgQuantity > 999.999) {
          console.log(
            `🔧 DEBUG: ❌ Peso fuera de rango válido: ${kgQuantity} kg`
          );
          console.log(`   - Rango válido: 0.001 - 999.999 kg`);
          console.log(`   - Formato usado: ${formatUsed}`);
          return;
        }

        console.log(`🔧 DEBUG: 🔍 Buscando producto con PLU: ${plu}`);
        console.log(
          `🔧 DEBUG: Productos disponibles con PLU:`,
          availableProducts
            .filter((p) => p.plu)
            .map((p) => ({ name: p.name, plu: p.plu, id: p.id }))
        );

        const productByPlu: any = availableProducts.find((p: any) => {
          if (p.plu === null || p.plu === undefined) return false;
          return Number(p.plu) === Number(plu);
        });

        if (productByPlu) {
          console.log("🔧 DEBUG: ✅ Producto encontrado por PLU:", {
            id: productByPlu.id,
            name: productByPlu.name,
            plu: productByPlu.plu,
            precio: productByPlu.pricePerUnit,
            cantidad: kgQuantity,
            formato: formatUsed,
            codigoOriginal: completeCode,
          });

          // 🛒 DEBUG: Estado del carrito ANTES de agregar PLU+peso
          const itemsBeforeAdd = cartState.getCurrentItems();
          console.log("🔧 DEBUG: Items en carrito ANTES de agregar PLU+peso:", {
            count: itemsBeforeAdd.length,
            items: itemsBeforeAdd.map((item) => ({
              id: item.id,
              name: item.name,
              cartId: item.cartId,
            })),
          });

          // ✅ USAR SOLO LA FUNCIÓN CENTRALIZADA - NO duplicar lógica
          console.log(
            "🔧 DEBUG: 🛒 Usando addScannedProduct para PLU+peso (función centralizada)"
          );
          addScannedProduct(productByPlu, kgQuantity);
          console.log(
            "🔧 DEBUG: ✅ Producto PLU+peso procesado a través de addScannedProduct"
          );

          // Limpiar input y buffer
          input.value = "";
          inputBuffer = "";
          if (inputTimeout) {
            clearTimeout(inputTimeout);
            inputTimeout = null;
          }

          // 🛒 DEBUG: Estado del carrito después de agregar PLU+peso
          setTimeout(() => {
            const itemsAfterAdd = cartState.getCurrentItems();
            console.log(
              "🔧 DEBUG: Items en carrito DESPUÉS de agregar PLU+peso:",
              {
                count: itemsAfterAdd.length,
                items: itemsAfterAdd.map((item) => ({
                  id: item.id,
                  name: item.name,
                  cartId: item.cartId,
                  cantidad: item.quantity,
                  subtotal: item.subtotal,
                })),
              }
            );
          }, 100);
        } else {
          console.log("🔧 DEBUG: ❌ No se encontró producto con PLU:", plu);
          console.log(`   - Formato usado: ${formatUsed}`);
          console.log(`   - Código original: ${completeCode}`);
          console.log(
            "🔧 DEBUG: Productos disponibles con PLU:",
            availableProducts
              .filter((p) => p.plu)
              .map((p) => ({ name: p.name, plu: p.plu, id: p.id }))
          );
        }
      } else {
        // ✅ NUEVO: Solo procesar códigos que tengan el formato correcto
        // NO procesar códigos parciales que coincidan con PLUs existentes
        console.log("🔧 DEBUG: ❌ No coincide con ningún patrón PLU + peso");
        console.log(`   - Código ingresado: ${completeCode}`);
        console.log(`   - Longitud: ${completeCode.length} dígitos`);
        console.log(
          `   - Formatos soportados: 12 dígitos (XXX + 8 dígitos peso + 1) o 13 dígitos (0 + XXX + 8 dígitos peso + 1)`
        );
        console.log(
          `   - ⚠️ Código ignorado: Solo se procesan códigos completos con formato PLU + peso`
        );
      }

      console.log("🔧 DEBUG: 🏁 FIN processCompleteCode");
    };

    const handleInputEvent = (e: Event) => {
      console.log("🔧 DEBUG: 📥 INICIO handleInputEvent");
      const value = (e.target as HTMLInputElement).value.trim();
      console.log(`🔧 DEBUG: Input event detectado, valor: "${value}"`);

      // ✅ CRÍTICO: Si estamos procesando un paste, ignorar este input event
      if (isProcessingPaste) {
        console.log("🔧 DEBUG: ⚠️ Input event ignorado - paste en progreso");
        return;
      }

      // Si el valor está vacío, limpiar buffer
      if (!value) {
        console.log(`🔧 DEBUG: Valor vacío, limpiando buffer`);
        inputBuffer = "";
        if (inputTimeout) {
          clearTimeout(inputTimeout);
          inputTimeout = null;
        }
        console.log("🔧 DEBUG: 🏁 FIN handleInputEvent (valor vacío)");
        return;
      }

      // Actualizar buffer con el valor completo
      inputBuffer = value;
      console.log(`🔧 DEBUG: Buffer actualizado: "${inputBuffer}"`);

      // Limpiar timeout anterior si existe
      if (inputTimeout) {
        clearTimeout(inputTimeout);
        console.log(`🔧 DEBUG: Timeout anterior limpiado`);
      }

      // Establecer nuevo timeout para procesar después de que se complete la entrada
      console.log(`🔧 DEBUG: Estableciendo nuevo timeout de 100ms`);
      inputTimeout = setTimeout(() => {
        console.log(
          `🔧 DEBUG: ⏰ Timeout completado, procesando buffer: "${inputBuffer}"`
        );
        processCompleteCode(inputBuffer);
        console.log(`🔧 DEBUG: Buffer limpiado después de procesar`);
        inputBuffer = "";
        inputTimeout = null;
      }, 100); // 100ms de delay para capturar el código completo

      console.log("🔧 DEBUG: 🏁 FIN handleInputEvent");
    };

    const handlePasteEvent = (e: ClipboardEvent) => {
      console.log("🔧 DEBUG: 📋 INICIO handlePasteEvent");
      const pastedText = e.clipboardData?.getData("text") || "";
      console.log(`🔧 DEBUG: Texto pegado: "${pastedText}"`);

      // ✅ CRÍTICO: Marcar que estamos procesando un paste para evitar duplicación
      isProcessingPaste = true;
      console.log("🔧 DEBUG: ✅ isProcessingPaste = true");

      // Limpiar buffer y timeout anteriores
      inputBuffer = "";
      if (inputTimeout) {
        clearTimeout(inputTimeout);
        inputTimeout = null;
      }

      // ✅ CAMBIO CRÍTICO: Solo procesar a través de processCompleteCode, no directamente
      console.log(
        `🔧 DEBUG: Procesando texto pegado a través de processCompleteCode`
      );
      setTimeout(() => {
        processCompleteCode(pastedText);
        // ✅ CRÍTICO: Después de procesar, permitir input events nuevamente
        setTimeout(() => {
          isProcessingPaste = false;
          console.log(
            "🔧 DEBUG: ✅ Paste procesado, isProcessingPaste = false"
          );
        }, 200); // Delay para asegurar que el input event no interfiera
      }, 50);

      console.log("🔧 DEBUG: 🏁 FIN handlePasteEvent");
    };

    // Escuchar tanto 'input' como 'paste' para capturar pegado por Ctrl+V
    console.log("🔧 DEBUG: 🔗 Agregando event listeners");
    input.addEventListener("input", handleInputEvent);
    input.addEventListener("paste", handlePasteEvent);

    console.log("🔧 DEBUG: ✅ Listeners configurados correctamente");

    return () => {
      console.log("🔧 DEBUG: 🧹 Removiendo listeners");
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

      <CancelDialog
        open={cancelDialogOpen}
        onOpenChange={setCancelDialogOpen}
        onConfirm={handleCancelCart}
        isLoading={isProcessingPayment}
      />

      <PaymentDialog
        isOpen={afipPaymentDialogOpen}
        onClose={() => {
          console.log(
            "🚪 REFACTORED: Cerrando AFIP PaymentDialog - reseteando estados"
          );
          console.log("🚪 AFIP Estados ANTES de cerrar:", {
            afipIsProcessing: afipPaymentProcessor.isProcessingPayment,
            afipSelectedMethod: afipPaymentProcessor.selectedPaymentMethod,
          });

          setAfipPaymentDialogOpen(false);
          afipPaymentProcessor.resetPaymentState();
          // NO tocar los estados locales del flujo normal

          console.log("🚪 AFIP Estados DESPUÉS de cerrar: reseteados");
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
          console.log(
            "🚪 REFACTORED: Cerrando PaymentDialog - reseteando estados"
          );
          console.log("🚪 Estados ANTES de cerrar:", {
            isProcessingPayment,
            selectedPaymentMethod,
            hookIsProcessing: paymentProcessor.isProcessingPayment,
            hookSelectedMethod: paymentProcessor.selectedPaymentMethod,
          });

          // ✅ RESETEAR ESTADOS INMEDIATAMENTE
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);

          setPaymentDialogOpen(false);
          paymentProcessor.resetPaymentState();

          console.log("🚪 Estados DESPUÉS de cerrar:", {
            isProcessingPayment: false,
            selectedPaymentMethod: null,
          });

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
            console.log(
              "🚪 REFACTORED: Cerrando AFIP CashPaymentDialog - reseteando estados"
            );
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
          console.log("💰 AFIP: Confirmando pago redondeado");
          try {
            // Preparar datos de descuento en el formato correcto para processAfipPayment
            let afipDiscountData = null;
            if (afipPaymentProcessor.applyingDiscount && discountData) {
              afipDiscountData = {
                type: discountData.type,
                value: discountData.value,
                amount: discountData.amount, // ✅ AGREGADO: incluir amount
              };
              console.log(
                "💰 AFIP: Aplicando descuento en pago redondeado",
                afipDiscountData
              );
            }

            await afipPaymentProcessor.processAfipPayment(
              "efectivo",
              cartState.getCurrentItems(),
              afipPaymentProcessor.roundedAmount,
              afipDiscountData
            );
            console.log("✅ AFIP: Pago redondeado procesado exitosamente");
          } catch (error) {
            console.error("❌ AFIP: Error al procesar pago redondeado:", error);
          }
        }}
        onCancel={() => {
          console.log(
            "❌ REFACTORED: Cancelando AFIP CashPaymentDialog - reseteando estados"
          );
          afipPaymentProcessor.setRoundedAmountDialogOpen(false);
          afipPaymentProcessor.resetPaymentState();
        }}
      />

      {/* Diálogo de pago exacto para AFIP */}
      <ExactPaymentDialog
        open={afipPaymentProcessor.exactPaymentDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            console.log(
              "🚪 REFACTORED: Cerrando AFIP ExactPaymentDialog - reseteando estados"
            );
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
          console.log(
            "💰 AFIP EXACT PAYMENT: Confirmando desde ShoppingCartRefactored",
            {
              paidAmount,
              change,
              totalAmount: afipPaymentProcessor.roundedAmount,
            }
          );

          try {
            await afipPaymentProcessor.confirmAfipExactPayment(
              paidAmount,
              change,
              cartState.getCurrentItems(),
              discountData
            );

            console.log("✅ AFIP EXACT PAYMENT: Pago procesado exitosamente");

            focusSearchInput("AFIP exact payment confirmado");
          } catch (error) {
            console.error("❌ AFIP EXACT PAYMENT: Error al procesar:", error);
          }
        }}
        onCancel={() => {
          console.log(
            "❌ REFACTORED: Cancelando AFIP ExactPaymentDialog - reseteando estados"
          );
          afipPaymentProcessor.setExactPaymentDialogOpen(false);
          afipPaymentProcessor.resetPaymentState();
        }}
      />

      <CashPaymentDialog
        open={paymentProcessor.roundedAmountDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            console.log(
              "🚪 REFACTORED: Cerrando CashPaymentDialog - reseteando estados"
            );
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
          console.log(
            "❌ REFACTORED: Cancelando CashPaymentDialog - reseteando estados"
          );
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
            console.log(
              "🚪 REFACTORED: Cerrando ExactPaymentDialog - reseteando estados"
            );
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
          console.log(
            "💰 EXACT PAYMENT: Confirmando desde ShoppingCartRefactored",
            {
              paidAmount,
              change,
              totalAmount: paymentProcessor.roundedAmount,
            }
          );

          setIsProcessingPayment(true);
          try {
            await paymentProcessor.confirmExactPayment(
              paidAmount,
              change,
              cartState.getCurrentItems(),
              discountData
            );

            console.log("✅ EXACT PAYMENT: Pago procesado exitosamente");

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
          console.log(
            "❌ REFACTORED: Cancelando ExactPaymentDialog - reseteando estados"
          );
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
          console.log("🔧 ManualQrDialog (normal) onOpenChange:", open);
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
          console.log("🔧 ManualQrDialog (AFIP) onOpenChange:", open);
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
