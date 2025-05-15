import { useState, useEffect, useRef } from "react";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Receipt, Wallet, CreditCard, QrCode } from "lucide-react";
import {
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
} from "@/components/ui/table";
import { Input } from "@/components/ui/input";

import { useAuth } from "@/contexts/AuthContext";
import { useCartState } from "@/hooks/useCartState";
import { usePaymentProcessing } from "@/hooks/usePaymentProcessing";
import { AvailableProduct } from "@/hooks/useProductSearch";

import {
  AddProductDialog,
  CartItem,
  CartSummary,
  CartTabs,
  HeaderActions,
  ProductSearch,
  PaymentDialog,
} from "./shopping-cart";

// Importar las imágenes como recursos desde assets
import iselinLogo from "../assets/iselin-logo.png";
import andextechLogo from "../assets/andextech-black.png";

export default function ShoppingCartRefactored() {
  const { user, logout } = useAuth();
  const API_URL = import.meta.env.VITE_API_URL;
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Estados para diálogos y UI
  const [dialogOpen, setDialogOpen] = useState(false);
  const [cancelDialogOpen, setCancelDialogOpen] = useState(false);
  const [paymentDialogOpen, setPaymentDialogOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] =
    useState<AvailableProduct | null>(null);

  // Estados para el diálogo de órdenes recientes
  const [ordersDialogOpen, setOrdersDialogOpen] = useState(false);
  const [recentOrders, setRecentOrders] = useState<any[]>([]);
  const [isLoadingOrders, setIsLoadingOrders] = useState(false);
  const [isPrinting, setIsPrinting] = useState(false);

  // Estado para información del negocio
  const [businessInfo, setBusinessInfo] = useState<any>(null);

  // Agregar estados para QR
  const [qrDialogOpen, setQrDialogOpen] = useState(false);
  const [splitPaymentDialogOpen, setSplitPaymentDialogOpen] = useState(false);

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
  });

  // Cargar información del negocio
  useEffect(() => {
    const fetchBusinessInfo = async () => {
      try {
        console.log("🏢 Iniciando carga de información del negocio");
        const response = await fetch(`${API_URL}/api/business/1`, {
          headers: {
            "Content-Type": "application/json",
            ...(appId && { "X-App-ID": appId }),
          },
        });

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

  // Limpiar intervalos al desmontar
  useEffect(() => {
    return () => {
      paymentProcessor.cleanupPolling();
    };
  }, []);

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
  const handlePayment = (method: string) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Si es efectivo, manejar con el sistema de redondeo
    if (method === "efectivo") {
      paymentProcessor.handleCashPayment(businessInfo);
      setPaymentDialogOpen(false);
      return;
    }

    // Si es QR, generar el QR
    if (method === "qr") {
      paymentProcessor.generateQRPayment(cartState.getCurrentItems());
      setPaymentDialogOpen(false);
      return;
    }

    // Si es pago dividido, preparar pago mixto
    if (method === "split") {
      paymentProcessor.handleSplitPayment();
      setPaymentDialogOpen(false);
      return;
    }

    // Para otros métodos (tarjeta), procesar directamente
    if (
      paymentProcessor.isProcessingPayment ||
      paymentProcessor.selectedPaymentMethod
    ) {
      return;
    }

    paymentProcessor.processPayment(
      method,
      Number(cartState.calculateTotal().toFixed(2)),
      cartState.getCurrentItems()
    );
    setPaymentDialogOpen(false);
  };

  // Confirmar pago redondeado en efectivo
  const confirmRoundedPayment = () => {
    if (paymentProcessor.isProcessingPayment) {
      return;
    }

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

  // Manejadores de diálogos
  const handleClosingDialog = () => {
    // Por implementar
  };

  // Funciones para manejar órdenes recientes
  const handleOrdersDialog = () => {
    setOrdersDialogOpen(true);
    loadRecentOrders();
  };

  const loadRecentOrders = async () => {
    if (!user) {
      toast.error("Debes iniciar sesión para ver órdenes");
      return;
    }

    setIsLoadingOrders(true);

    try {
      // Usar el endpoint específico para órdenes del vendedor
      const response = await fetch(
        `${API_URL}/api/ordenes/vendedor/${user.id}?limit=5`,
        {
          headers,
        }
      );

      if (!response.ok) {
        throw new Error("Error al cargar órdenes recientes");
      }

      const data = await response.json();
      console.log("Órdenes recientes cargadas:", data);
      setRecentOrders(data);
    } catch (error) {
      console.error("Error al cargar órdenes:", error);
      toast.error("Error al cargar las órdenes recientes");
    } finally {
      setIsLoadingOrders(false);
    }
  };

  // Función para reimprimir un ticket
  const handleReprintTicket = async (order: any) => {
    if (isPrinting) return;

    setIsPrinting(true);

    try {
      const { ipcRenderer } = window.require("electron");

      toast.info("Reimprimiendo ticket...", {
        duration: 3000,
        description: "Enviando datos a la impresora",
      });

      console.log("Reimprimiendo ticket para orden:", order);

      const result = await ipcRenderer.invoke("print-ticket", order);

      if (result.success) {
        toast.success("Ticket reimpreso correctamente");
      } else {
        throw new Error(result.message || "Error desconocido al reimprimir");
      }
    } catch (error: any) {
      console.error("Error al reimprimir ticket:", error);
      toast.error(`Error al reimprimir: ${error.message}`);
    } finally {
      setIsPrinting(false);
    }
  };

  // Asignar las funciones de control de diálogos al procesador de pagos
  // Muy importante: esto debe estar antes de limpiar el poll de QR en el useEffect
  paymentProcessor.setQrDialogOpen = setQrDialogOpen;
  paymentProcessor.setSplitPaymentDialogOpen = setSplitPaymentDialogOpen;
  paymentProcessor.qrDialogOpen = qrDialogOpen; // Añadir acceso al estado actual del diálogo

  // Limpiar intervalos al desmontar o cuando cambia el estado del diálogo QR
  useEffect(() => {
    if (!qrDialogOpen) {
      paymentProcessor.cleanupPolling();
    }
    return () => {
      paymentProcessor.cleanupPolling();
    };
  }, [qrDialogOpen]);

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

          <div className="ml-auto flex items-center">
            <HeaderActions
              userName={user?.nombre || ""}
              userEmail={user?.email || ""}
              canCloseCashDesk={user?.permisos?.cierreDeCajaEnabled || false}
              onOpenOrders={handleOrdersDialog}
              onOpenClosing={handleClosingDialog}
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

      <Dialog open={cancelDialogOpen} onOpenChange={setCancelDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>¿Cancelar orden?</DialogTitle>
            <DialogDescription>
              ¿Estás seguro de que deseas cancelar la orden? Se eliminarán todos
              los productos del carrito.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="flex space-x-2 justify-end">
            <Button
              variant="outline"
              onClick={() => setCancelDialogOpen(false)}
            >
              No, mantener productos
            </Button>
            <Button className="bg-cancel-gradient" onClick={handleCancelCart}>
              Sí, cancelar orden
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <PaymentDialog
        isOpen={paymentDialogOpen}
        onClose={() => {
          setPaymentDialogOpen(false);
          paymentProcessor.resetPaymentState();
          setTimeout(() => searchInputRef.current?.focus(), 100);
        }}
        onSelectPayment={handlePayment}
        isProcessingPayment={paymentProcessor.isProcessingPayment}
        selectedPaymentMethod={paymentProcessor.selectedPaymentMethod}
      />

      <Dialog
        open={paymentProcessor.roundedAmountDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            paymentProcessor.setRoundedAmountDialogOpen(false);
            paymentProcessor.resetPaymentState();
            setTimeout(() => searchInputRef.current?.focus(), 100);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="text-2xl">
              {paymentProcessor.applyingDiscount
                ? "Efectivo con descuento"
                : businessInfo?.sistemaPago === "redondeo"
                ? "Redondeo de pago en efectivo"
                : "Confirmar pago en efectivo"}
            </DialogTitle>
            <DialogDescription className="text-lg">
              {paymentProcessor.applyingDiscount
                ? `Se aplicará un descuento del ${businessInfo?.descuentoEfectivo}% por pago en efectivo.`
                : businessInfo?.sistemaPago === "redondeo"
                ? "El sistema de redondeo ha ajustado el monto para facilitar el pago en efectivo."
                : "Por favor confirma el pago en efectivo."}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {paymentProcessor.applyingDiscount ||
            (businessInfo?.sistemaPago === "redondeo" &&
              paymentProcessor.originalAmount !==
                paymentProcessor.roundedAmount) ? (
              <>
                <div className="flex justify-between items-center">
                  <span className="text-base text-muted-foreground">
                    Monto original:
                  </span>
                  <span className="text-xl">
                    ${paymentProcessor.originalAmount.toLocaleString()}
                  </span>
                </div>

                {paymentProcessor.applyingDiscount && (
                  <div className="flex justify-between items-center">
                    <span className="text-base text-muted-foreground">
                      Con descuento ({businessInfo?.descuentoEfectivo}%):
                    </span>
                    <span className="text-xl text-blue-600">
                      $
                      {(
                        paymentProcessor.originalAmount -
                        (paymentProcessor.originalAmount *
                          Number(businessInfo?.descuentoEfectivo)) /
                          100
                      ).toLocaleString()}
                    </span>
                  </div>
                )}

                <div className="flex justify-between items-center">
                  <span className="text-lg font-medium">Monto a cobrar:</span>
                  <span className="text-3xl font-bold text-emerald-600">
                    ${paymentProcessor.roundedAmount.toLocaleString()}
                  </span>
                </div>

                <div className="flex justify-between items-center">
                  <span className="text-base text-muted-foreground">
                    {paymentProcessor.applyingDiscount
                      ? "Ahorro total:"
                      : "Descuento del redondeo:"}
                  </span>
                  <span className="text-lg text-emerald-700">
                    $
                    {(
                      paymentProcessor.originalAmount -
                      paymentProcessor.roundedAmount
                    ).toLocaleString()}
                    {paymentProcessor.applyingDiscount &&
                      businessInfo?.sistemaPago === "redondeo" && (
                        <span className="ml-1 text-sm">
                          (descuento + redondeo)
                        </span>
                      )}
                  </span>
                </div>
              </>
            ) : (
              <div className="flex justify-between items-center">
                <span className="text-lg font-medium">Monto a cobrar:</span>
                <span className="text-3xl font-bold text-emerald-600">
                  ${paymentProcessor.originalAmount.toLocaleString()}
                </span>
              </div>
            )}
          </div>

          <DialogFooter className="flex space-x-2 justify-end">
            <Button
              variant="outline"
              onClick={() => {
                paymentProcessor.setRoundedAmountDialogOpen(false);
                paymentProcessor.resetPaymentState();
              }}
              tabIndex={3}
              className="text-base py-5 px-4"
            >
              Cancelar
            </Button>

            {!paymentProcessor.applyingDiscount &&
              businessInfo?.descuentoEfectivo && (
                <Button
                  variant="outline"
                  className="bg-blue-50 border-blue-200 hover:bg-blue-100 hover:text-blue-700 text-base py-5 px-4"
                  onClick={() => {
                    // Cerrar el diálogo actual
                    paymentProcessor.setRoundedAmountDialogOpen(false);
                    // Pequeña pausa para asegurar que se cierre primero
                    setTimeout(() => {
                      // Volver a abrir con descuento
                      paymentProcessor.handleCashPayment(businessInfo, true);
                    }, 100);
                  }}
                  tabIndex={2}
                >
                  Aplicar D (F5)
                </Button>
              )}

            <Button
              className="bg-emerald-gradient text-lg py-5 px-6"
              onClick={confirmRoundedPayment}
              disabled={paymentProcessor.isProcessingPayment}
              autoFocus
              tabIndex={1}
            >
              {paymentProcessor.isProcessingPayment ? (
                <div className="flex items-center gap-2">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  <span>Procesando...</span>
                </div>
              ) : (
                "Confirmar pago"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Agregar el diálogo de órdenes recientes */}
      <Dialog open={ordersDialogOpen} onOpenChange={setOrdersDialogOpen}>
        <DialogContent className="sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle className="text-xl">Órdenes recientes</DialogTitle>
            <DialogDescription>
              Últimas 5 órdenes realizadas por {user?.nombre}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 max-h-[60vh] overflow-y-auto py-2">
            {isLoadingOrders ? (
              <div className="flex justify-center items-center h-40">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
              </div>
            ) : recentOrders.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No se encontraron órdenes recientes
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Fecha</TableHead>
                    <TableHead>Método de pago</TableHead>
                    <TableHead className="text-right">Total</TableHead>
                    <TableHead className="text-center">Acciones</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {recentOrders.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell>
                        {new Date(order.fecha).toLocaleString("es-AR", {
                          day: "2-digit",
                          month: "2-digit",
                          year: "2-digit",
                          hour: "2-digit",
                          minute: "2-digit",
                        })}
                      </TableCell>
                      <TableCell className="capitalize">
                        {order.metodoPago}
                      </TableCell>
                      <TableCell className="text-right font-medium">
                        ${Number(order.total).toLocaleString()}
                      </TableCell>

                      <TableCell className="text-center">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleReprintTicket(order)}
                          disabled={isPrinting}
                          className="hover:bg-blue-50 hover:text-blue-600"
                        >
                          {isPrinting ? (
                            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-current"></div>
                          ) : (
                            <Receipt className="h-4 w-4 mr-1" />
                          )}
                          Reimprimir
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setOrdersDialogOpen(false)}
            >
              Cerrar
            </Button>
            <Button onClick={loadRecentOrders} disabled={isLoadingOrders}>
              {isLoadingOrders ? (
                <div className="flex items-center gap-2">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  <span>Cargando...</span>
                </div>
              ) : (
                "Actualizar"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Diálogo de QR */}
      <Dialog
        open={qrDialogOpen}
        onOpenChange={(open) => {
          console.log("🔄 QR Dialog onOpenChange:", open);
          if (!open) {
            console.log("🔄 Cerrando diálogo QR, cancelando pago");
            paymentProcessor.cancelQRPayment();
          }
          setQrDialogOpen(open);
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
                  cartState.calculateTotal()
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
                  cartState.calculateTotal()
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
    </div>
  );
}
