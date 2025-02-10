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
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
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
}

export default function ShoppingCart() {
  const { user } = useAuth();
  const API_URL = import.meta.env.VITE_API_URL;

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
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [closingDialogOpen, setClosingDialogOpen] = useState(false);

  const appId =
    window.electron?.process?.argv
      ?.find((arg) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null;
  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  const weight = useScaleWeight();

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
    setDialogOpen(true);
    setShowResults(false);
    setSearchQuery("");
  };

  const addToCart = () => {
    if (selectedProduct && quantity) {
      const quantityNum = parseFloat(quantity);
      const finalQuantity =
        selectedProduct.unit === "Kg" ? quantityNum / 1000 : quantityNum;
      const newItem: Product = {
        id: selectedProduct.id,
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
    }
  };

  const removeFromCart = (id: number) => {
    setCartItems((prev) => prev.filter((item) => item.id !== id));
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

      if (closingDialogOpen) {
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
              onChange={(e) => {
                setSearchQuery(e.target.value);
                setShowResults(e.target.value.length > 0);
              }}
              onKeyDown={handleKeyPress}
              className="bg-background max-w-2xl md:text-base rounded-xl"
            />

            {showResults && searchResults.length > 0 && (
              <Card className="absolute w-full mt-1 bg-background border shadow-lg p-2 z-10 max-w-2xl rounded-xl">
                {searchResults.map((product, index) => (
                  <div
                    key={product.id}
                    className={cn(
                      "p-2 cursor-pointer",
                      index === selectedIndex ? "bg-muted" : "hover:bg-muted"
                    )}
                    onClick={() => handleProductSelect(product)}
                  >
                    <div className="text-base">
                      {product.name} - ${product.pricePerUnit.toLocaleString()}/
                      {product.unit}
                    </div>
                  </div>
                ))}
              </Card>
            )}
          </div>

          <div className="w-full flex justify-end items-center gap-4">
            <Button
              className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
              onClick={() => setClosingDialogOpen(true)}
            >
              <Store />
              Cierre de caja
            </Button>
            <UserMenu
              user={{ nombre: user?.nombre || "", email: user?.email || "" }}
            />
          </div>
        </div>

        <div className="flex-1 space-y-2 overflow-auto mb-4">
          {cartItems.map((item) => (
            <Card
              key={item.id}
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
                  onClick={() => removeFromCart(item.id)}
                >
                  <Trash2 className="h-6 w-6" />
                </Button>
              </div>
            </Card>
          ))}
        </div>

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogContent className="w-[90vw] md:w-full rounded-xl">
            <DialogHeader>
              <DialogTitle className="text-emerald-gradient font-bold text-2xl">
                Agregar {selectedProduct?.name}
              </DialogTitle>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <div className="text-center">
                  <div className="text-4xl font-bold">
                    {selectedProduct?.unit === "Kg" ? weight : quantity}{" "}
                    {selectedProduct?.unit === "Kg" ? "g" : "u"}
                  </div>
                  {selectedProduct?.unit !== "Kg" && (
                    <Input
                      type="number"
                      value={quantity}
                      onChange={(e) => setQuantity(e.target.value)}
                      onKeyPress={handleQuantityKeyPress}
                      placeholder="Cantidad"
                    />
                  )}
                </div>
              </div>
              <Button
                onClick={addToCart}
                className="w-full bg-emerald-gradient text-white hover:text-white"
              >
                Agregar al carrito
              </Button>
            </div>
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
          <DialogContent className="sm:max-w-md">
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
          para cancelar
        </div>
      </div>
    </div>
  );
}
