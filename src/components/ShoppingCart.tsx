import { useState, useEffect, useRef } from "react";
import {
  Trash2,
  CreditCard,
  Wallet,
  QrCode,
  Receipt,
  Landmark,
  User,
  Store,
  Sun,
  Moon,
  Calendar,
  LogOut,
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { toast } from "sonner";
import { useAuth } from "@/contexts/AuthContext";
import { UserMenu } from "@/components/user-menu";
import { cn } from "@/lib/utils";
import { useScaleWeight } from "@/hooks/useScaleWeight";

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

interface AvailableProduct {
  id: number;
  name: string;
  pricePerUnit: number;
  unit: string;
  costo: number;
  codigoBarras: string | null;
}

export default function ShoppingCart() {
  const { user, refreshUserData, logout } = useAuth();
  const API_URL = import.meta.env.VITE_API_URL;

  // Add detailed console logging for debugging
  useEffect(() => {
    console.log("User data:", user);
    console.log("User permissions:", user?.permisos);
    console.log("pesoManualEnabled:", user?.permisos?.pesoManualEnabled);
  }, [user]);

  const [searchQuery, setSearchQuery] = useState("");
  const [showResults, setShowResults] = useState(false);
  const [cartItems, setCartItems] = useState<Product[]>([]);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [cancelDialogOpen, setCancelDialogOpen] = useState(false);
  const [paymentDialogOpen, setPaymentDialogOpen] = useState(false);
  const [selectedProduct, setSelectedProduct] =
    useState<AvailableProduct | null>(null);
  const [quantity, setQuantity] = useState<string>("");
  const [searchResults, setSearchResults] = useState<AvailableProduct[]>([]);
  const [availableProducts, setAvailableProducts] = useState<
    AvailableProduct[]
  >([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [isProcessingPayment, setIsProcessingPayment] = useState(false);
  const [useManualWeight, setUseManualWeight] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [closingDialogOpen, setClosingDialogOpen] = useState(false);
  const [lastInputTime, setLastInputTime] = useState<number>(0);
  const [barcodeBuffer, setBarcodeBuffer] = useState<string>("");

  const appId =
    window.electron?.process?.argv
      ?.find((arg) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null;
  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  const weight = useScaleWeight();

  // Initialize and update useManualWeight based on user permissions
  useEffect(() => {
    console.log("User permissions:", user?.permisos);
    // Set weight mode based on user permissions
    if (user?.permisos?.pesoManualEnabled === true) {
      setUseManualWeight(true);
    } else {
      setUseManualWeight(false);
    }
  }, [user]);

  // Cargar productos al montar el componente
  useEffect(() => {
    const fetchProducts = async () => {
      try {
        const response = await fetch(`${API_URL}/api/productos`, {
          headers,
        });
        if (!response.ok) {
          throw new Error("Error al cargar productos");
        }
        const data = await response.json();

        const transformedProducts = data.map((p: any) => ({
          id: p.id,
          name: p.nombre,
          pricePerUnit: p.precio,
          unit: p.tipoMedida,
          costo: p.costo,
          codigoBarras: p.codigoBarras,
        }));

        setAvailableProducts(transformedProducts);
      } catch (error) {
        console.error("Error al cargar productos:", error);
        toast.error("Error al cargar los productos");
      }
    };

    fetchProducts();
  }, [API_URL]);

  // Filtrar productos según búsqueda
  useEffect(() => {
    if (searchQuery) {
      const results = availableProducts.filter((product) =>
        product.name.toLowerCase().includes(searchQuery.toLowerCase())
      );
      setSearchResults(results);
      setShowResults(true);
    } else {
      setSearchResults([]);
      setShowResults(false);
    }
  }, [searchQuery, availableProducts]);

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (searchResults.length === 0) return;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < searchResults.length - 1 ? prev + 1 : prev
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
        break;
      case "Enter":
        e.preventDefault();
        if (selectedIndex >= 0) {
          handleProductSelect(searchResults[selectedIndex]);
        } else if (searchResults.length > 0) {
          handleProductSelect(searchResults[0]);
        }
        break;
      case "Escape":
        setShowResults(false);
        setSearchQuery("");
        setSelectedIndex(-1);
        break;
    }
  };

  const handleQuantityKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && quantity) {
      addToCart();
    }
  };

  const handleProductSelect = (product: AvailableProduct) => {
    setSelectedProduct(product);
    // Update the useManualWeight state based on current permissions when dialog opens
    if (user?.permisos?.pesoManualEnabled === true) {
      setUseManualWeight(true);
    } else {
      setUseManualWeight(false);
    }
    setDialogOpen(true);
    setShowResults(false);
    setSearchQuery("");
  };

  // Modificar addToCart para manejar peso manual
  const addToCart = () => {
    if (!selectedProduct) return;

    let finalQuantity: number;

    if (selectedProduct.unit === "Kg") {
      if (useManualWeight) {
        if (!quantity) return;
        finalQuantity = parseFloat(quantity) / 1000; // Convertir gramos a kilos
      } else {
        finalQuantity = weight / 1000; // Convertir gramos a kilos
      }
    } else {
      if (!quantity) return;
      finalQuantity = parseFloat(quantity);
    }

    const newItem: Product = {
      id: selectedProduct.id,
      cartId: `${selectedProduct.id}-${Date.now()}`,
      name: selectedProduct.name,
      quantity: finalQuantity,
      unit: selectedProduct.unit,
      pricePerUnit: selectedProduct.pricePerUnit,
      subtotal: selectedProduct.pricePerUnit * finalQuantity,
      costo: selectedProduct.costo,
    };

    setCartItems((prev) => [...prev, newItem]);
    setDialogOpen(false);
    setSelectedProduct(null);
    setQuantity("");
    setTimeout(() => {
      searchInputRef.current?.focus();
    }, 0);
  };

  const removeFromCart = (cartId: string) => {
    setCartItems((prev) => prev.filter((item) => item.cartId !== cartId));
  };

  const total = cartItems.reduce((sum, item) => sum + item.subtotal, 0);

  const handleCancelCart = () => {
    setCartItems([]);
    setCancelDialogOpen(false);
    setTimeout(() => {
      searchInputRef.current?.focus();
    }, 0);
    toast.success("Carrito cancelado", {});
  };

  const handlePayment = async (method: string) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    if (isProcessingPayment) return;
    setIsProcessingPayment(true);

    const orderItems = cartItems.map((item) => ({
      productoId: item.id,
      cantidad: item.quantity,
      subtotal: Number(item.subtotal.toFixed(2)),
      precioHistorico: item.pricePerUnit,
      costo: Number(item.costo),
      nombre: item.name,
    }));

    const orderData = {
      metodoPago: method,
      total: Number(total.toFixed(2)),
      items: orderItems,
      vendedorId: user.id,
      sucursalId: user.sucursalId,
      createdAt: new Date().toISOString(),
    };

    try {
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
        throw new Error("Error al crear la orden");
      }

      // Imprimir ticket usando Electron IPC
      try {
        const { ipcRenderer } = window.require("electron");
        console.log("Enviando datos para impresión:", orderData);
        const result = await ipcRenderer.invoke("print-ticket", orderData);
        console.log("Resultado de impresión:", result);

        if (result.success) {
          toast.success("Ticket impreso correctamente");
        } else {
          throw new Error(result.message || "Error desconocido al imprimir");
        }
      } catch (printError: any) {
        console.error("Error detallado al imprimir:", printError);
        toast.error(`Error al imprimir el ticket: ${printError.message}`);
      }

      setPaymentDialogOpen(false);
      setCartItems([]);
      toast.success("Orden completada exitosamente");
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al procesar la orden");
    } finally {
      setIsProcessingPayment(false);
    }
  };

  const handlePaymentClick = () => {
    if (cartItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de continuar",
      });
      return;
    }
    setPaymentDialogOpen(true);
  };

  const handleCancelClick = () => {
    if (cartItems.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "El carrito ya está vacío",
      });
      return;
    }
    setCancelDialogOpen(true);
  };

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

    try {
      const lastCloseResponse = await fetch(
        `${API_URL}/api/cierres?vendedorId=${user.id}&last=true`,
        { headers }
      );
      const lastClose = await lastCloseResponse.json();

      let startDate;
      if (period === "mañana") {
        startDate = new Date();
        startDate.setHours(6, 0, 0, 0);
      } else if (period === "tarde") {
        startDate = lastClose ? new Date(lastClose.fechaCierre) : new Date();
      } else {
        startDate = new Date();
        startDate.setHours(0, 0, 0, 0);
      }

      const ventasResponse = await fetch(`${API_URL}/api/ordenes/ventas`, {
        method: "POST",
        headers,
        body: JSON.stringify({
          vendedorId: user.id,
          sucursalId: user.sucursalId,
          fechaInicio: startDate,
          fechaCierre: new Date(),
        }),
      });

      const ventas = await ventasResponse.json();

      const closingData = {
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        fechaInicio: startDate,
        fechaCierre: new Date(),
        periodo: period,
        totalVentas: ventas.total,
        cantidadVentas: ventas.cantidad,
        ventasPorMetodo: ventas.ventasPorMetodo,
      };

      const cierreResponse = await fetch(`${API_URL}/api/cierres`, {
        method: "POST",
        headers,
        body: JSON.stringify(closingData),
      });

      if (!cierreResponse.ok) {
        throw new Error("Error al registrar el cierre");
      }

      // Imprimir ticket de cierre
      try {
        const { ipcRenderer } = window.require("electron");
        const result = await ipcRenderer.invoke("print-closing", closingData);
        if (result.success) {
          toast.success("Ticket de cierre impreso correctamente");
        }
      } catch (printError: any) {
        console.error("Error al imprimir cierre:", printError);
        toast.error(
          `Error al imprimir el ticket de cierre: ${printError.message}`
        );
      }

      console.log("Datos del cierre:", {
        periodo: period,
        fechaInicio: startDate,
        fechaCierre: new Date(),
        ventasPorMetodo: ventas.ventasPorMetodo,
        totalVentas: ventas.total,
        cantidadVentas: ventas.cantidad,
      });

      toast.success(`Cierre de ${period} realizado correctamente`);
      setClosingDialogOpen(false);
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al realizar el cierre de caja");
    }
  };

  useEffect(() => {
    const handleGlobalKeyPress = (e: KeyboardEvent) => {
      // Handle F4 for logout
      if (e.key === "F4") {
        e.preventDefault();
        handleLogout();
        return;
      }

      // Evitar que se active cuando se está escribiendo en un input
      const isInputElement = e.target instanceof HTMLInputElement;

      // Manejar F1 y F2 incluso en inputs
      if (e.key === "F1" || e.key === "F2") {
        e.preventDefault();
        if (e.key === "F2") {
          handlePaymentClick();
        } else {
          handleCancelClick();
        }
        return;
      }

      // Para el resto de atajos, verificar que no estemos en un input
      if (isInputElement) return;

      // Solo procesar atajos de cierre si el usuario tiene permiso
      if (closingDialogOpen && user?.permisos?.cierreDeCajaEnabled) {
        switch (e.key) {
          case "1":
            e.preventDefault();
            handleClosing("mañana");
            break;
          case "2":
            e.preventDefault();
            handleClosing("tarde");
            break;
          case "3":
            e.preventDefault();
            handleClosing("todo");
            break;
        }
        return;
      }

      if (paymentDialogOpen) {
        switch (e.key) {
          case "1":
            e.preventDefault();
            handlePayment("transferencia");
            break;
          case "2":
            e.preventDefault();
            handlePayment("mercadoPago");
            break;
          case "3":
            e.preventDefault();
            handlePayment("tarjeta");
            break;
          case "4":
            e.preventDefault();
            handlePayment("efectivo");
            break;
        }
      }
    };

    window.addEventListener("keydown", handleGlobalKeyPress);
    return () => window.removeEventListener("keydown", handleGlobalKeyPress);
  }, [closingDialogOpen, paymentDialogOpen, cartItems.length]);

  // Función para cerrar sesión
  const handleLogout = () => {
    toast.success("Cerrando sesión...");
    setTimeout(() => {
      logout();
    }, 1000);
  };

  // Modificar la función de búsqueda para incluir código de barras
  const handleSearchInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const currentTime = Date.now();
    const value = e.target.value;
    setSearchQuery(value);

    // Si el tiempo entre teclas es menor a 50ms, probablemente sea un scanner
    if (currentTime - lastInputTime < 50) {
      setBarcodeBuffer((prev) => prev + value.slice(-1));
    } else {
      setBarcodeBuffer(value);
    }

    setLastInputTime(currentTime);

    // Si detectamos un patrón de código de barras (números y longitud específica)
    if (/^\d{8,13}$/.test(value)) {
      const product = availableProducts.find((p) => p.codigoBarras === value);
      if (product) {
        handleProductSelect(product);
      }
    }

    // Actualizar resultados de búsqueda normal
    if (value) {
      const results = availableProducts.filter(
        (product) =>
          product.name.toLowerCase().includes(value.toLowerCase()) ||
          product.codigoBarras === value
      );
      setSearchResults(results);
      setShowResults(true);
    } else {
      setSearchResults([]);
      setShowResults(false);
    }
  };

  console.log(user);
  return (
    <div className="min-h-screen bg-white-cream h-screen">
      <div className="w-full mx-auto flex h-full flex-col px-8 py-6">
        <div className="flex w-full justify-between items-center mb-4">
          <div className="relative w-full">
            <Input
              ref={searchInputRef}
              autoFocus
              type="search"
              placeholder="Buscar productos..."
              value={searchQuery}
              onChange={handleSearchInputChange}
              onKeyDown={handleKeyPress}
              className="bg-background max-w-2xl md:text-base rounded-xl"
            />

            {showResults && searchResults.length > 0 && (
              <Card className="absolute w-full mt-1 bg-background border shadow-lg p-2 z-10 max-w-2xl rounded-xl">
                {searchResults.map((product, index) => (
                  <div
                    key={product.id}
                    className={cn(
                      "p-3 cursor-pointer rounded-lg transition-colors",
                      index === selectedIndex
                        ? "bg-emerald-100 text-emerald-900 font-medium"
                        : "hover:bg-muted"
                    )}
                    onClick={() => handleProductSelect(product)}
                  >
                    <div className="text-base flex items-center justify-between">
                      <span>{product.name}</span>
                      <span className="text-emerald-600">
                        ${product.pricePerUnit.toLocaleString()}/{product.unit}
                      </span>
                    </div>
                  </div>
                ))}
              </Card>
            )}
          </div>

          <div className="w-full flex justify-end items-center gap-4">
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

        <div className="flex-1 space-y-2 overflow-auto mb-4">
          {cartItems.map((item) => (
            <Card
              key={item.cartId}
              className="bg-background border p-4 flex items-center justify-between shadow-sm rounded-xl"
            >
              <div className="flex justify-between w-full items-center">
                <div className="flex justify-between items-end gap-10">
                  <span className="text-2xl font-medium">{item.name}</span>
                  <div className=" text-muted-foreground">
                    Cantidad: {item.quantity} {item.unit}
                  </div>
                  <div className=" text-muted-foreground">
                    {item.unit === "Kg" ? "$/Kg" : "$/U"}: $
                    {item.pricePerUnit.toLocaleString()}
                  </div>
                  <div className=" text-muted-foreground">
                    SUBTOTAL: ${item.subtotal.toLocaleString()}
                  </div>
                </div>

                <Button
                  variant="destructive"
                  size="icon"
                  className="h-8 w-8 rounded-lg bg-cancel-gradient"
                  onClick={() => removeFromCart(item.cartId)}
                >
                  <Trash2 className="h-6 w-6" />
                </Button>
              </div>
            </Card>
          ))}
        </div>

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogContent
            onKeyDown={(e) => {
              if (
                e.key === "Enter" &&
                selectedProduct?.unit === "Kg" &&
                !useManualWeight
              ) {
                e.preventDefault();
                addToCart();
              }
            }}
          >
            <DialogHeader>
              <DialogTitle className="text-emerald-gradient font-bold text-2xl">
                Agregar producto
              </DialogTitle>
              <DialogDescription>
                {selectedProduct?.name} - ${selectedProduct?.pricePerUnit}/{" "}
                {selectedProduct?.unit}
              </DialogDescription>
            </DialogHeader>

            {selectedProduct?.unit === "Kg" && (
              <div className="flex items-center space-x-4 py-2">
                {user?.permisos?.pesoManualEnabled === true && (
                  <>
                    <Switch
                      id="weight-mode"
                      checked={useManualWeight}
                      onCheckedChange={setUseManualWeight}
                      className="data-[state=checked]:bg-emerald-700"
                    />
                    <label
                      htmlFor="weight-mode"
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      Peso manual
                    </label>
                  </>
                )}
                {user?.permisos?.pesoManualEnabled !== true && (
                  <p className="text-sm text-gray-500">
                    Modo de peso automático
                  </p>
                )}
              </div>
            )}

            {selectedProduct?.unit !== "Kg" || useManualWeight ? (
              <Input
                type="number"
                placeholder={`Cantidad ${
                  selectedProduct?.unit === "Kg" ? "en gramos" : ""
                }`}
                value={quantity}
                onChange={(e) => setQuantity(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    addToCart();
                  }
                }}
                step={selectedProduct?.unit === "Kg" ? "1" : "1"}
                min="0"
              />
            ) : (
              <div className="text-start py-2">
                <p className="text-3xl font-bold">{weight} g</p>
                <p className="text-sm text-gray-500">Peso de la balanza</p>
              </div>
            )}

            <DialogFooter>
              <Button
                variant="outline"
                className="bg-cancel-gradient text-white hover:text-white text-base"
                onClick={() => setDialogOpen(false)}
                type="button"
              >
                Cancelar
              </Button>
              <Button
                onClick={addToCart}
                className="bg-emerald-gradient text-white hover:text-white text-base"
                type="submit"
              >
                Agregar al carrito
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog open={cancelDialogOpen} onOpenChange={setCancelDialogOpen}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>¿Cancelar orden?</DialogTitle>
              <DialogDescription>
                ¿Estás seguro de que deseas cancelar la orden? Se eliminarán
                todos los productos del carrito.
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

        <Dialog open={paymentDialogOpen} onOpenChange={setPaymentDialogOpen}>
          <DialogContent
            className="sm:max-w-md"
            onKeyDown={(e) => {
              if (
                e.key === "Enter" &&
                selectedProduct?.unit === "Kg" &&
                !useManualWeight
              ) {
                e.preventDefault();
                addToCart();
              }
            }}
          >
            <DialogHeader>
              <DialogTitle>Seleccionar método de pago</DialogTitle>
              <DialogDescription>
                Presiona el número correspondiente al método de pago o haz clic
                en el botón
              </DialogDescription>
            </DialogHeader>
            <div className="grid grid-cols-2 gap-4">
              <Button
                onClick={() => handlePayment("transferencia")}
                className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
                variant="outline"
                disabled={isProcessingPayment}
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <Landmark className="h-12 w-12" />
                  )}
                </div>
                <span>Transferencia Bancaria (1)</span>
              </Button>
              <Button
                onClick={() => handlePayment("mercadoPago")}
                className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
                variant="outline"
                disabled={isProcessingPayment}
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <img
                      src="logos/mercadopago.webp"
                      alt="Mercado Pago"
                      width={48}
                      height={48}
                      className="object-contain"
                    />
                  )}
                </div>
                <span>Mercado Pago (2)</span>
              </Button>
              <Button
                onClick={() => handlePayment("tarjeta")}
                className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
                variant="outline"
                disabled={isProcessingPayment}
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <CreditCard className="h-12 w-12" />
                  )}
                </div>
                <span>Tarjeta (3)</span>
              </Button>
              <Button
                onClick={() => handlePayment("efectivo")}
                className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
                variant="outline"
                disabled={isProcessingPayment}
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <Wallet className="h-12 w-12" />
                  )}
                </div>
                <span>Efectivo (4)</span>
              </Button>
            </div>
          </DialogContent>
        </Dialog>

        <Dialog open={closingDialogOpen} onOpenChange={setClosingDialogOpen}>
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
              >
                <Sun />
                <span className="text-base">Mañana (1)</span>
              </Button>
              <Button
                onClick={() => handleClosing("tarde")}
                className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
                variant="outline"
              >
                <Moon />
                <span className="text-base">Tarde (2)</span>
              </Button>
              <Button
                onClick={() => handleClosing("todo")}
                className="h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8"
                variant="outline"
              >
                <Calendar />
                <span className="text-base">Todo el día (3)</span>
              </Button>
            </div>
          </DialogContent>
        </Dialog>

        <div className="sticky bottom-0">
          <div className="flex items-center justify-between">
            <div className="space-x-2">
              <Button
                variant="outline"
                className=" text-white px-14 py-8 text-xl rounded-xl bg-cancel-gradient hover:text-white"
                onClick={handleCancelClick}
              >
                CANCELAR
              </Button>
              <Button
                className="bg-emerald-gradient text-primary-foreground px-14 py-8 text-xl rounded-xl"
                onClick={handlePaymentClick}
              >
                PAGAR
              </Button>
            </div>
            <div className="text-4xl font-semibold bg-white border px-8 py-6 shadow-sm rounded-xl">
              TOTAL: ${total.toLocaleString()}
            </div>
          </div>
        </div>

        <div className="text-sm text-muted-foreground mt-2">
          Atajos: ↑↓ para navegar, Enter para seleccionar, F2 para pagar, F1
          para cancelar, F4 para cerrar sesión
        </div>
      </div>
    </div>
  );
}
