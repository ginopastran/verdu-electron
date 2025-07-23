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
import { useCartSidebar } from "@/contexts/CartSidebarContext";
import { useSearchInput } from "@/contexts/SearchInputContext";
import { useCartState } from "@/hooks/useCartState";
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

// Importar el nuevo componente de diálogo de órdenes recientes
import { RecentOrdersDialog } from "./RecentOrdersDialog";

// Tipos
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
    calculateTotal: cartState.calculateTotal,
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
    calculateTotal: cartState.calculateTotal,
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
    if (!product) return;
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

    setAfipPaymentDialogOpen(true);
    setPaymentDialogOpen(false); // Asegurar que el diálogo normal esté cerrado
  };

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
        afipPaymentProcessor.handleAfipCashPayment(businessInfo);
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
            cartState.getCurrentItems()
          );

          setAfipPaymentDialogOpen(false);
          paymentLockRef.current = false;
          return;
        }

        // Si MP está habilitado, generar QR de MercadoPago + AFIP
        console.log("🧾📱 AFIP: MP habilitado - generando QR de MercadoPago");
        const qrData = await afipPaymentProcessor.handleAfipQrPayment(
          cartState.getCurrentItems()
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
        cartState.getCurrentItems()
      );

      // Los estados se limpian en el hook afipPaymentProcessor
    } catch (error: any) {
      console.error("❌ Error en pago AFIP:", error);
      toast.error(`Error en factura AFIP: ${error.message}`);
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
      // NO establecer isProcessingPayment aquí, solo cuando se confirme el pago
      setSelectedPaymentMethod("efectivo");
      paymentProcessor.handleCashPayment(businessInfo);
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

          // Limpiar carrito y estados locales
          cartState.clearCart();
          setIsProcessingPayment(false);
          setSelectedPaymentMethod(null);
          setPaymentDialogOpen(false);

          toast.success("Orden completada exitosamente");
          paymentLockRef.current = false;
        } catch (error: any) {
          console.error("❌ Error en flujo QR/MP deshabilitado:", error);
          toast.error(`Error al procesar la orden: ${error.message}`);
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
      paymentProcessor.generateQRPayment(cartState.getCurrentItems());
      // No cerramos inmediatamente el diálogo: permanecerá mostrando spinner
      // El diálogo se cerrará automáticamente cuando se abra el diálogo QR
      return;
    }

    // Para otros métodos (tarjeta), procesar directamente
    try {
      await paymentProcessor.processPayment(
        method,
        Number(cartState.calculateTotal().toFixed(2)),
        cartState.getCurrentItems()
      );
      // ✅ MEJORADO: Estados se limpian en processPayment, pero asegurar limpieza local
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    } catch (error: any) {
      console.error("❌ Error al procesar pago:", error);
      // ✅ MEJORADO: Limpiar estados en caso de error
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
      await paymentProcessor.processPayment(
        "efectivo",
        paymentProcessor.roundedAmount,
        cartState.getCurrentItems()
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
    }
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
      }
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
      const product = availableProducts.find(
        (p: any) => p.codigoBarras === value
      );
      if (product) {
        // Agregar automáticamente producto por código de barras (1 unidad)
        autoAddScannedProduct(product, 1);
        setBarcodeBuffer("");
        return;
      }
    }

    // Detectar formato 0 + 3 dígitos PLU + 8 dígitos peso(g) + 1 dígito checksum (13 dígitos)
    const pluWeightRegex = /^0(\d{3})(\d{8})\d$/;
    const match = value.match(pluWeightRegex);
    if (match) {
      const plu = match[1];
      const grams = parseInt(match[2], 10);
      const kgQuantity = grams / 1000; // convertir a kilos

      const productByPlu: any = availableProducts.find((p: any) => {
        if (p.plu === null || p.plu === undefined) return false;
        // Comparar numéricamente para ignorar ceros a la izquierda ("002" vs "2")
        return Number(p.plu) === Number(plu);
      });
      if (productByPlu) {
        autoAddScannedProduct(productByPlu, kgQuantity);
        setBarcodeBuffer("");
        return;
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

  // Función auxiliar para añadir producto escaneado directamente al carrito
  const autoAddScannedProduct = (prod: any, qty: number) => {
    try {
      if (qty <= 0) return;

      // 🛒 DEBUG: Estado del carrito ANTES de agregar en autoAddScannedProduct
      const itemsBeforeAdd = cartState.getCurrentItems();
      console.log("🛒 DEBUG: [autoAddScannedProduct] Estado carrito ANTES:", {
        count: itemsBeforeAdd.length,
        items: itemsBeforeAdd.map((item) => ({
          id: item.id,
          name: item.name,
          cartId: item.cartId,
        })),
      });

      console.log("🛒 DEBUG: [autoAddScannedProduct] Agregando producto:", {
        id: prod.id,
        name: prod.name,
        quantity: qty,
        pricePerUnit: prod.pricePerUnit,
        unit: prod.unit,
      });

      const uniqueId = `${prod.id}-${Date.now()}-${Math.random()
        .toString(36)
        .substring(2, 10)}`;

      const newItem: Product = {
        id: prod.id,
        cartId: uniqueId,
        name: prod.name,
        quantity: qty,
        unit: prod.unit || "Unidad",
        pricePerUnit: prod.pricePerUnit,
        subtotal: Number((prod.pricePerUnit * qty).toFixed(2)),
        costo: prod.costo,
      };

      console.log("🛒 DEBUG: [autoAddScannedProduct] Objeto creado:", newItem);

      cartState.addToCart(newItem);

      // 🛒 DEBUG: Estado del carrito DESPUÉS de agregar en autoAddScannedProduct
      setTimeout(() => {
        const itemsAfterAdd = cartState.getCurrentItems();
        console.log(
          "🛒 DEBUG: [autoAddScannedProduct] Estado carrito DESPUÉS:",
          {
            count: itemsAfterAdd.length,
            items: itemsAfterAdd.map((item) => ({
              id: item.id,
              name: item.name,
              cartId: item.cartId,
            })),
          }
        );
      }, 50);

      // Limpiar input y restablecer foco
      setSearchQuery("");
      focusSearchInput("producto escaneado");
    } catch (err) {
      console.error("❌ Error auto-add producto:", err);
    }
  };

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

  // Limpiar intervalos al desmontar o cuando cambia el estado del diálogo QR
  useEffect(() => {
    if (!qrDialogOpen) {
      paymentProcessor.cleanupPolling();
    }
    return () => {
      paymentProcessor.cleanupPolling();
    };
  }, [qrDialogOpen]);

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
      console.log("🔍 DEBUG: searchInputRef.current es null");
      return;
    }

    console.log("🔍 DEBUG: Configurando listeners para input de búsqueda");
    console.log("🔍 DEBUG: Productos disponibles:", availableProducts.length);

    const handleInputEvent = (e: Event) => {
      const value = (e.target as HTMLInputElement).value.trim();
      console.log(`🔍 DEBUG: Input event detectado, valor: "${value}"`);

      // Usar setTimeout para procesar después de que se complete el paste
      setTimeout(() => {
        const currentValue = (e.target as HTMLInputElement).value.trim();
        console.log(
          `🔍 DEBUG: Procesando valor después de timeout: "${currentValue}"`
        );

        if (!currentValue) {
          console.log("🔍 DEBUG: Valor vacío, no procesando");
          return;
        }

        // Verificar que tengamos productos cargados
        if (!availableProducts || availableProducts.length === 0) {
          console.log("🔍 DEBUG: Productos aún no cargados, esperando...");
          return;
        }

        // 🛒 DEBUG: Estado actual del carrito antes de procesar
        const currentItems = cartState.getCurrentItems();
        console.log("🛒 DEBUG: Estado del carrito antes de procesar:", {
          itemsCount: currentItems.length,
          items: currentItems.map((item) => ({
            id: item.id,
            name: item.name,
            cartId: item.cartId,
          })),
        });

        // 1. Código de barras estándar (8-15 dígitos) - EAN-8, UPC-A, EAN-13, Code 128, etc.
        if (/^\d{8,15}$/.test(currentValue)) {
          console.log("🔍 DEBUG: Patrón de código de barras detectado");
          console.log(
            "🔍 DEBUG: Productos con código de barras:",
            availableProducts
              .filter((p) => p.codigoBarras)
              .map((p) => ({ name: p.name, codigoBarras: p.codigoBarras }))
          );

          // Buscar coincidencia exacta
          let product = availableProducts.find(
            (p: any) => p.codigoBarras === currentValue
          );

          // Si no hay coincidencia exacta, buscar si algún código contiene el valor o viceversa
          if (!product) {
            console.log(
              "🔍 DEBUG: No encontrado exacto, buscando coincidencias parciales..."
            );
            product = availableProducts.find(
              (p: any) =>
                p.codigoBarras &&
                (p.codigoBarras.includes(currentValue) ||
                  currentValue.includes(p.codigoBarras))
            );
          }

          if (product) {
            console.log("🔍 DEBUG: Producto encontrado por código de barras:", {
              id: product.id,
              name: product.name,
              codigoBarras: product.codigoBarras,
              precio: product.pricePerUnit,
              cantidad: 1,
            });

            // 🛒 DEBUG: Estado del carrito ANTES de agregar código de barras
            const itemsBeforeAdd = cartState.getCurrentItems();
            console.log(
              "🛒 DEBUG: Items en carrito ANTES de agregar código de barras:",
              {
                count: itemsBeforeAdd.length,
                items: itemsBeforeAdd.map((item) => ({
                  id: item.id,
                  name: item.name,
                  cartId: item.cartId,
                })),
              }
            );

            autoAddScannedProduct(product, 1);
            (e.target as HTMLInputElement).value = "";

            // 🛒 DEBUG: Estado del carrito después de agregar código de barras
            setTimeout(() => {
              const itemsAfterAdd = cartState.getCurrentItems();
              console.log(
                "🛒 DEBUG: Items en carrito DESPUÉS de agregar código de barras:",
                {
                  count: itemsAfterAdd.length,
                  items: itemsAfterAdd.map((item) => ({
                    id: item.id,
                    name: item.name,
                    cartId: item.cartId,
                  })),
                }
              );
            }, 100);
            return;
          } else {
            console.log(
              "🔍 DEBUG: No se encontró producto con código de barras:",
              currentValue
            );
          }
        }

        // 2. PLU + peso: 3 dígitos PLU + 8 dígitos gramos + 1 dígito adicional (12 dígitos total)
        const pluWeightRegex = /^(\d{3})(\d{8})(\d{1})$/;
        const match = currentValue.match(pluWeightRegex);
        if (match) {
          const plu = match[1];
          const grams = parseInt(match[2], 10);
          const kgQuantity = grams / 1000; // convertir a kilos

          console.log(
            `🔍 DEBUG: PLU + peso detectado - PLU: ${plu}, gramos: ${grams}, kg: ${kgQuantity}`
          );

          // ✅ CORRECCIÓN: Validar que el peso sea razonable (entre 0.001 y 999.999 kg)
          if (kgQuantity < 0.001 || kgQuantity > 999.999) {
            console.log(
              `🔍 DEBUG: Peso fuera de rango válido: ${kgQuantity} kg`
            );
            return;
          }

          const productByPlu: any = availableProducts.find((p: any) => {
            if (p.plu === null || p.plu === undefined) return false;
            return Number(p.plu) === Number(plu);
          });

          if (productByPlu) {
            console.log("🔍 DEBUG: Producto encontrado por PLU:", {
              id: productByPlu.id,
              name: productByPlu.name,
              plu: productByPlu.plu,
              precio: productByPlu.pricePerUnit,
              cantidad: kgQuantity,
            });

            // 🛒 DEBUG: Estado del carrito ANTES de agregar PLU+peso
            const itemsBeforeAdd = cartState.getCurrentItems();
            console.log(
              "🛒 DEBUG: Items en carrito ANTES de agregar PLU+peso:",
              {
                count: itemsBeforeAdd.length,
                items: itemsBeforeAdd.map((item) => ({
                  id: item.id,
                  name: item.name,
                  cartId: item.cartId,
                })),
              }
            );

            autoAddScannedProduct(productByPlu, kgQuantity);
            (e.target as HTMLInputElement).value = "";

            // 🛒 DEBUG: Estado del carrito después de agregar PLU+peso
            setTimeout(() => {
              const itemsAfterAdd = cartState.getCurrentItems();
              console.log(
                "🛒 DEBUG: Items en carrito DESPUÉS de agregar PLU+peso:",
                {
                  count: itemsAfterAdd.length,
                  items: itemsAfterAdd.map((item) => ({
                    id: item.id,
                    name: item.name,
                    cartId: item.cartId,
                  })),
                }
              );
            }, 100);
          } else {
            console.log("🔍 DEBUG: No se encontró producto con PLU:", plu);
            console.log(
              "🔍 DEBUG: Productos disponibles con PLU:",
              availableProducts
                .filter((p) => p.plu)
                .map((p) => ({ name: p.name, plu: p.plu }))
            );
          }
        } else {
          console.log("🔍 DEBUG: No coincide con patrón PLU + peso");
        }
      }, 10);
    };

    const handlePasteEvent = (e: ClipboardEvent) => {
      console.log("🔍 DEBUG: Paste event detectado");
      const pastedText = e.clipboardData?.getData("text") || "";
      console.log(`🔍 DEBUG: Texto pegado: "${pastedText}"`);

      setTimeout(() => {
        const inputElement = e.target as HTMLInputElement;
        const currentValue = inputElement.value.trim();
        console.log(`🔍 DEBUG: Valor después de paste: "${currentValue}"`);

        // Procesar el valor pegado
        if (currentValue) {
          handleInputEvent({ target: inputElement } as any);
        }
      }, 50);
    };

    // Escuchar tanto 'input' como 'paste' para capturar pegado por Ctrl+V
    input.addEventListener("input", handleInputEvent);
    input.addEventListener("paste", handlePasteEvent);

    console.log("🔍 DEBUG: Listeners configurados correctamente");

    return () => {
      console.log("🔍 DEBUG: Removiendo listeners");
      input.removeEventListener("input", handleInputEvent);
      input.removeEventListener("paste", handlePasteEvent);
    };
  }, [availableProducts, searchInputRef]);

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
            onChangeScreen={cartState.setActiveScreen}
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

        {cartState.getCurrentItems().map((item) => (
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
        total={cartState.calculateTotal()}
        onCancel={handleCancelClick}
        onCheckout={
          businessInfo?.facturacionHabilitada
            ? handleAfipPaymentClick
            : handlePaymentClick
        }
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
            afipPaymentProcessor.handleAfipCashPayment(businessInfo, true);
          }, 100);
        }}
        onConfirm={async () => {
          console.log("💰 AFIP: Confirmando pago redondeado");
          try {
            await afipPaymentProcessor.processAfipPayment(
              "efectivo",
              cartState.getCurrentItems(),
              afipPaymentProcessor.roundedAmount
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
              cartState.getCurrentItems()
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
              cartState.getCurrentItems()
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
      />

      {/* NUEVO: Diálogo de pago mixto para AFIP */}
      <AfipSplitPaymentDialog
        open={afipSplitPaymentDialogOpen}
        onOpenChange={setAfipSplitPaymentDialogOpen}
        afipPaymentProcessor={afipPaymentProcessor}
        cartState={cartState}
        businessInfo={businessInfo}
        searchInputRef={searchInputRef}
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
}

function getAppId() {
  return (
    (window as any).electron?.process?.argv
      ?.find((arg: string) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null
  );
}
