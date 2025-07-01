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
} from "./shopping-cart";

// Componentes de diálogo
import { CashPaymentDialog, CancelDialog } from "./shopping-cart/dialogs";
import { ExactPaymentDialog } from "./shopping-cart/dialogs/ExactPaymentDialog";

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

  // Estados de procesamiento de pago (manejados en este componente)
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<
    string | null
  >(null);

  // Utilizar los hooks personalizados
  const cartState = useCartState();
  const { businessInfo, loading: businessInfoLoading } = useBusinessInfo(
    API_URL,
    getAppId()
  );
  const { availableProducts } = useProducts(API_URL, getAppId());
  const closing = useClosing(user, API_URL, getAppId());
  const { handleTicketPrinting, formatFechaArgentina } = useTicketPrinting();

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
  });

  // 🔄 SYNC: Mantener los estados locales de pago en línea con el hook
  useEffect(() => {
    console.log(
      "🔄 SYNC: Actualizando isProcessingPayment local:",
      paymentProcessor.isProcessingPayment
    );
    setIsProcessingPayment(paymentProcessor.isProcessingPayment);
  }, [paymentProcessor.isProcessingPayment]);

  useEffect(() => {
    console.log(
      "🔄 SYNC: Actualizando selectedPaymentMethod local:",
      paymentProcessor.selectedPaymentMethod
    );
    setSelectedPaymentMethod(paymentProcessor.selectedPaymentMethod);
  }, [paymentProcessor.selectedPaymentMethod]);

  // 🆕 NUEVO: Effect para limpiar estados locales cuando el hook se resetea
  useEffect(() => {
    // Si el hook no tiene método seleccionado ni está procesando, limpiar estados locales
    if (
      !paymentProcessor.selectedPaymentMethod &&
      !paymentProcessor.isProcessingPayment
    ) {
      console.log("🧹 SYNC: Hook reseteado, limpiando estados locales");
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
      !cashDialogOpen &&
      !exactPaymentDialogOpen
    ) {
      // Si todos los diálogos están cerrados pero aún hay estados activos, limpiar
      if (isProcessingPayment || selectedPaymentMethod) {
        console.log(
          "🧹 SYNC CLEANUP: Diálogos cerrados pero estados activos, limpiando:",
          {
            qrDialogOpen,
            paymentDialogOpen,
            splitPaymentDialogOpen,
            cashDialogOpen,
            exactPaymentDialogOpen,
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
    cashDialogOpen,
    exactPaymentDialogOpen,
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
  const handleCancelCart = () => {
    cartState.clearCart();
    setCancelDialogOpen(false);

    // Enfocar el input de búsqueda
    focusSearchInput("carrito cancelado");

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

    console.log("💰 handlePaymentClick - Artículos en carrito:", currentItems);

    // Resetear estados del flujo AFIP al abrir diálogo normal
    console.log("🧹 Reseteando estados del flujo AFIP antes de pago normal");
    afipPaymentProcessor.resetPaymentState();

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

    console.log("🧾 Abriendo diálogo de pago AFIP");

    // Resetear estados del flujo normal al abrir AFIP
    console.log("🧹 Reseteando estados del flujo normal antes de AFIP");
    paymentProcessor.resetPaymentState();
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);

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
        setPaymentDialogOpen(false);
        paymentLockRef.current = false;
        return;
      }

      // Para otros métodos, procesar directamente
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
      console.log("🔄 Iniciando pago mixto desde ShoppingCartRefactored");
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
            // Imprimir ticket
            await handleTicketPrinting(enrichedOrderData, API_URL, getAppId());

            // 🆕 DOBLE IMPRESIÓN QR/MP DESHABILITADO: Si está habilitada, imprimir segunda vez
            if (businessInfo?.dobleImpresionEnabled === true) {
              console.log(
                "🖨️🖨️ QR/MP DESHABILITADO DOBLE IMPRESIÓN: Imprimiendo segunda copia..."
              );
              try {
                await handleTicketPrinting(
                  enrichedOrderData,
                  API_URL,
                  getAppId()
                );
                console.log(
                  "✅ QR/MP DESHABILITADO DOBLE IMPRESIÓN: Segunda copia impresa exitosamente"
                );
              } catch (error) {
                console.error(
                  "❌ QR/MP DESHABILITADO DOBLE IMPRESIÓN: Error en segunda copia:",
                  error
                );
                // No fallar la orden si la segunda impresión falla
              }
            }
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
        return String(p.plu) === plu;
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

      const uniqueId = `${prod.id}-${Date.now()}-${Math.random()
        .toString(36)
        .substring(2, 10)}`;

      const newItem: Product = {
        id: prod.id,
        cartId: uniqueId,
        name: prod.nombre,
        quantity: qty,
        unit: prod.tipoMedida || "Unidad",
        pricePerUnit: prod.precio,
        subtotal: Number((prod.precio * qty).toFixed(2)),
        costo: prod.costo,
      };

      cartState.addToCart(newItem);

      // Limpiar input y restablecer foco
      setSearchQuery("");
      focusSearchInput("producto escaneado");
    } catch (err) {
      console.error("Error auto-add producto:", err);
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

      <ClosingDialog
        open={closing.closingDialogOpen}
        onOpenChange={closing.setClosingDialogOpen}
        onHandleClosing={closing.handleClosing}
        isClosing={closing.isClosing}
        searchInputRef={searchInputRef}
      />

      <QRPaymentDialog
        open={qrDialogOpen}
        onOpenChange={setQrDialogOpen}
        paymentProcessor={paymentProcessor}
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
        onSubmit={paymentProcessor.handleManualQrPasswordSubmit}
      />

      <SplitPaymentDialog
        open={splitPaymentDialogOpen}
        onOpenChange={setSplitPaymentDialogOpen}
        paymentProcessor={paymentProcessor}
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
