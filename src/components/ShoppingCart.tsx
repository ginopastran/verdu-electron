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
  Plus,
  X,
  History,
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
import {
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
} from "@/components/ui/table";
import { Select, SelectItem, SelectContent } from "@/components/ui/select";
// Importar las imágenes como recursos desde assets
import iselinLogo from "../assets/iselin-logo.png";
import andextechLogo from "../assets/andextech-black.png";
// Importar QRCode para generación de QR en cliente
import QRCode from "qrcode";

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
  const [splitPaymentDialogOpen, setSplitPaymentDialogOpen] = useState(false);
  const [cashAmount, setCashAmount] = useState<string>("");
  const [secondPaymentMethod, setSecondPaymentMethod] =
    useState<string>("tarjeta");
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
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<
    string | null
  >(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [closingDialogOpen, setClosingDialogOpen] = useState(false);
  const [lastInputTime, setLastInputTime] = useState<number>(0);
  const [barcodeBuffer, setBarcodeBuffer] = useState<string>("");
  const [closeResultDialogOpen, setCloseResultDialogOpen] = useState(false);
  const [closeResultData, setCloseResultData] = useState<any>(null);
  const [isClosing, setIsClosing] = useState(false);
  // Nuevos estados para el sistema de redondeo
  const [businessInfo, setBusinessInfo] = useState<any>(null);
  const [roundedAmountDialogOpen, setRoundedAmountDialogOpen] = useState(false);
  const [originalAmount, setOriginalAmount] = useState<number>(0);
  const [roundedAmount, setRoundedAmount] = useState<number>(0);
  // Agregar un estado para evitar múltiples adiciones
  const [isAddingToCart, setIsAddingToCart] = useState(false);
  // Modificar la función handleCashPayment para establecer un estado que indique si se está aplicando descuento o no
  const [applyingDiscount, setApplyingDiscount] = useState(false);

  // Nuevos estados para múltiples pantallas
  const [activeScreen, setActiveScreen] = useState(0);
  const [screens, setScreens] = useState<{ id: number; items: Product[] }[]>([
    { id: 0, items: [] },
  ]);

  // Agregar una referencia para rastrear la última solicitud
  const lastAddRequestRef = useRef<string>("");

  const appId =
    window.electron?.process?.argv
      ?.find((arg) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null;
  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  const weight = useScaleWeight();

  // Cargar información del negocio con businessId 1
  useEffect(() => {
    const fetchBusinessInfo = async () => {
      try {
        console.log("🏢 Iniciando carga de información del negocio");
        const response = await fetch(`${API_URL}/api/business/1`, {
          headers,
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
        } else {
          console.log("✅ Sistema de pago configurado:", data.sistemaPago);
          // Verificación explícita para asegurar que sea "redondeo"
          if (data.sistemaPago !== "redondeo") {
            console.warn(
              `⚠️ Sistema de pago '${data.sistemaPago}' no es 'redondeo', el redondeo no se aplicará`
            );
          } else {
            console.log("✅ Sistema de redondeo activado");
          }
        }

        // Verificar descuento en efectivo
        if (data.descuentoEfectivo) {
          console.log(
            `✅ Descuento en efectivo configurado: ${data.descuentoEfectivo}%`
          );
        } else {
          console.log("ℹ️ No hay descuento en efectivo configurado");
        }

        // Guardar en el estado con un resumen
        console.log("🧮 Resumen de configuración:", {
          sistemaPago: data.sistemaPago,
          redondeoActivo: data.sistemaPago === "redondeo",
          descuentoEfectivo: data.descuentoEfectivo || "No configurado",
        });

        setBusinessInfo(data);
      } catch (error) {
        console.error("❌ Error al cargar información del negocio:", error);
        // En caso de error, establecer un valor por defecto para evitar problemas
        setBusinessInfo({ sistemaPago: "redondeo" });
      }
    };

    // Ejecutar la carga de información
    fetchBusinessInfo();
  }, [API_URL]);

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
      // Filtrar productos que coinciden con la búsqueda
      const filtered = availableProducts.filter((product) =>
        product.name.toLowerCase().includes(searchQuery.toLowerCase())
      );

      // Ordenar los resultados: primero por relevancia, luego alfabéticamente
      const sortedResults = filtered.sort((a, b) => {
        // 1. Priorizar coincidencias exactas al inicio
        const aStartsWithQuery = a.name
          .toLowerCase()
          .startsWith(searchQuery.toLowerCase());
        const bStartsWithQuery = b.name
          .toLowerCase()
          .startsWith(searchQuery.toLowerCase());

        if (aStartsWithQuery && !bStartsWithQuery) return -1;
        if (!aStartsWithQuery && bStartsWithQuery) return 1;

        // 2. Priorizar coincidencias de ID/código (números al inicio)
        const aStartsWithNumber =
          /^\d+/.test(a.name) &&
          a.name.toLowerCase().includes(searchQuery.toLowerCase());
        const bStartsWithNumber =
          /^\d+/.test(b.name) &&
          b.name.toLowerCase().includes(searchQuery.toLowerCase());

        if (aStartsWithNumber && !bStartsWithNumber) return -1;
        if (!aStartsWithNumber && bStartsWithNumber) return 1;

        // 3. Orden alfabético para misma relevancia
        return a.name.localeCompare(b.name);
      });

      setSearchResults(sortedResults);
      setShowResults(true);

      // Seleccionar automáticamente el primer resultado
      setSelectedIndex(sortedResults.length > 0 ? 0 : -1);
    } else {
      setSearchResults([]);
      setShowResults(false);
      setSelectedIndex(-1);
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

  // Modificar la función addToCart para implementar una verificación de duplicación basada en huella única
  const addToCart = () => {
    // Crear un identificador único para esta solicitud específica
    const currentRequestId = `${selectedProduct?.id}-${Date.now()}-${quantity}`;

    console.log("🔄 DEBUG CRITICAL: Nueva solicitud de adición", {
      requestId: currentRequestId,
      lastRequestId: lastAddRequestRef.current,
      timeDiff: lastAddRequestRef.current
        ? Date.now() - parseInt(lastAddRequestRef.current.split("-")[1] || "0")
        : "Primera solicitud",
      productName: selectedProduct?.name,
      productUnit: selectedProduct?.unit,
    });

    // Si esta solicitud es idéntica a la última dentro de 1 segundo, es un duplicado
    if (
      lastAddRequestRef.current &&
      lastAddRequestRef.current.split("-")[0] ===
        currentRequestId.split("-")[0] &&
      Date.now() - parseInt(lastAddRequestRef.current.split("-")[1] || "0") <
        1000
    ) {
      console.log("🚫 BLOQUEADO: Solicitud duplicada detectada", {
        current: currentRequestId,
        last: lastAddRequestRef.current,
      });
      return;
    }

    // Actualizar la referencia con la solicitud actual
    lastAddRequestRef.current = currentRequestId;

    console.log("🔍 DEBUG: Iniciando addToCart", {
      selectedProduct: selectedProduct?.name,
      isAddingToCart,
      quantity,
      time: new Date().toISOString(),
    });

    // Protección contra adiciones duplicadas
    if (isAddingToCart) {
      console.log(
        "🛑 DEBUG: Bloqueando adición - ya hay una adición en progreso"
      );
      return;
    }

    if (!selectedProduct) {
      console.log(
        "🛑 DEBUG: Bloqueando adición - no hay producto seleccionado"
      );
      return;
    }

    // Marcar el inicio del proceso de adición
    setIsAddingToCart(true);

    try {
      let finalQuantity: number;

      // Validación específica para productos de tipo Kg
      if (selectedProduct.unit === "Kg") {
        if (useManualWeight) {
          if (!quantity) {
            console.log("❌ DEBUG: Cantidad faltante para producto de peso");
            throw new Error(
              "Se requiere especificar una cantidad para este producto"
            );
          }
          finalQuantity = parseFloat(quantity) / 1000; // Convertir gramos a kilos
        } else {
          finalQuantity = weight / 1000; // Convertir gramos a kilos
        }
      }
      // Validación específica para productos de tipo Unidad
      else {
        if (!quantity) {
          console.log("❌ DEBUG: Cantidad faltante para producto unitario");
          throw new Error(
            "Se requiere especificar una cantidad para este producto"
          );
        }
        finalQuantity = parseFloat(quantity);
      }

      // Validación adicional de la cantidad
      if (isNaN(finalQuantity) || finalQuantity <= 0) {
        console.log("❌ DEBUG: Cantidad inválida:", finalQuantity);
        throw new Error("La cantidad debe ser un número mayor que cero");
      }

      // Crear un ID verdaderamente único (mejor entropia)
      const uniqueId = `${selectedProduct.id}-${Date.now()}-${Math.random()
        .toString(36)
        .substring(2, 10)}-${Math.random().toString(36).substring(2, 10)}`;

      console.log("✓ DEBUG: ID generado:", uniqueId);

      // Crear el objeto a añadir con información completa para debugging
      const newItem: Product = {
        id: selectedProduct.id,
        cartId: uniqueId,
        name: selectedProduct.name,
        quantity: finalQuantity,
        unit: selectedProduct.unit,
        pricePerUnit: selectedProduct.pricePerUnit,
        subtotal: selectedProduct.pricePerUnit * finalQuantity,
        costo: selectedProduct.costo,
      };

      console.log("✓ DEBUG: Añadiendo producto al carrito:", newItem);

      // Actualizar la hoja activa con el nuevo item
      setScreens(
        screens.map((screen, index) => {
          if (index === activeScreen) {
            const existingItem = screen.items.find(
              (item) => item.id === newItem.id
            );
            if (existingItem) {
              // Si el item ya existe, actualizar la cantidad
              return {
                ...screen,
                items: screen.items.map((item) =>
                  item.id === newItem.id
                    ? {
                        ...item,
                        quantity: item.quantity + newItem.quantity,
                        subtotal: item.subtotal + newItem.subtotal,
                      }
                    : item
                ),
              };
            } else {
              // Si es un nuevo item, agregarlo
              return {
                ...screen,
                items: [...screen.items, newItem],
              };
            }
          }
          return screen;
        })
      );

      // Cerramos el diálogo y limpiamos el estado
      setDialogOpen(false);
      setSelectedProduct(null);
      setQuantity("");

      console.log("✅ DEBUG: Producto añadido con éxito, limpiando estados");
    } catch (error: any) {
      console.error("❌ ERROR:", error.message);
      toast.error(error.message || "Error al añadir producto");
    } finally {
      // Aseguramos que el estado isAddingToCart se resetea después de un tiempo suficiente
      // para evitar problemas de carrera y dar tiempo a los estados a actualizarse
      setTimeout(() => {
        setIsAddingToCart(false);
        console.log("✓ DEBUG: Estado de adición reiniciado después de timeout");
        // Enfocar el input de búsqueda
        if (searchInputRef.current) {
          searchInputRef.current.focus();
        }
      }, 500); // Permitimos 500ms para que todos los estados se actualicen
    }
  };

  const removeFromCart = (cartId: string) => {
    console.log("Eliminando producto con cartId:", cartId);
    setCartItems((prev) => {
      const updatedItems = prev.filter((item) => item.cartId !== cartId);
      console.log("Items restantes:", updatedItems.length);
      return updatedItems;
    });
  };

  const calculateTotal = () => {
    const currentScreen = screens[activeScreen];
    return currentScreen.items.reduce((total, item) => {
      const subtotal = item.pricePerUnit * item.quantity;
      return total + subtotal;
    }, 0);
  };

  const handleCancelCart = () => {
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen ? { ...screen, items: [] } : screen
      )
    );
    setCancelDialogOpen(false);
    setTimeout(() => {
      searchInputRef.current?.focus();
    }, 0);
    toast.success("Carrito cancelado", {});
  };

  // Función para redondear a los 50 pesos más cercanos hacia abajo
  const roundToNearest50 = (amount: number): number => {
    // Primero asegurarse de que estamos trabajando con un número entero
    // Redondear a 2 decimales primero para evitar problemas de punto flotante
    const amountFixed = parseFloat(amount.toFixed(2));

    // Obtener la parte entera
    const integerPart = Math.floor(amountFixed);

    // Calcular el resto solo con la parte entera
    const remainder = integerPart % 50;

    // Obtener la parte decimal
    const decimalPart = amountFixed - integerPart;

    // Si el resto es 0 y no hay decimales, ya está redondeado a 50
    if (remainder === 0 && decimalPart === 0) {
      console.log("🧮 Ya está redondeado a 50:", amountFixed);
      return amountFixed;
    }

    // Redondear hacia abajo a múltiplo de 50
    const roundedDown = integerPart - remainder;

    console.log("🧮 DEBUG Redondeo:", {
      original: amount,
      redondeadoA2Decimales: amountFixed,
      parteEntera: integerPart,
      parteDecimal: decimalPart,
      resto: remainder,
      redondeadoA50: roundedDown,
    });

    return roundedDown;
  };

  // Reemplazar completamente la función handleCashPayment para forzar siempre un diálogo
  const handleCashPayment = (withDiscount = false) => {
    console.log(
      "🛒 EFECTIVO: Iniciando proceso de pago en efectivo",
      withDiscount ? "con descuento" : ""
    );

    // Establecer el estado de descuento
    setApplyingDiscount(withDiscount);

    // Calcular los importes para cualquier caso
    const originalTotal = Number(calculateTotal().toFixed(2));
    let finalTotal = originalTotal;

    // Aplicar descuento si es necesario
    if (withDiscount && businessInfo?.descuentoEfectivo) {
      const discountPercentage = Number(businessInfo.descuentoEfectivo);
      const discountAmount = (originalTotal * discountPercentage) / 100;
      finalTotal = originalTotal - discountAmount;

      console.log("💰 DESCUENTO: Cálculos:", {
        originalTotal,
        discountPercentage,
        discountAmount,
        finalTotal,
      });
    }

    // Si el sistema de pago es redondeo, calcular el monto redondeado
    let roundedTotal = finalTotal;
    if (businessInfo?.sistemaPago === "redondeo") {
      roundedTotal = roundToNearest50(finalTotal);
      console.log("🧮 EFECTIVO: Cálculos de redondeo:", {
        finalTotal,
        roundedTotal,
        diferencia: finalTotal - roundedTotal,
        sistemaRedondeo: businessInfo?.sistemaPago,
      });
    } else {
      console.log("💰 EFECTIVO: No hay redondeo, usando monto original:", {
        finalTotal,
        sistemaRedondeo: businessInfo?.sistemaPago,
      });
    }

    // Guardar los montos calculados en el estado
    setOriginalAmount(originalTotal);
    setRoundedAmount(roundedTotal);

    // Establecer efectivo como método seleccionado
    setSelectedPaymentMethod("efectivo");

    // SIEMPRE cerrar el diálogo de pago y abrir el diálogo de redondeo
    console.log("🔄 EFECTIVO: Mostrando diálogo de confirmación");

    // Cerrar primero el diálogo de pagos
    setPaymentDialogOpen(false);

    // Abrir diálogo de redondeo o confirmación con un pequeño retraso
    setTimeout(() => {
      // Es crucial que esto siempre se ejecute
      setRoundedAmountDialogOpen(true);
      console.log("🔄 EFECTIVO: Diálogo de confirmación abierto");
    }, 100);
  };

  // Modificar la función handlePayment para manejar el método de pago QR
  const handlePayment = async (method: string) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    console.log("🔄 handlePayment llamado con método:", method);

    // Si es efectivo, siempre usar nuestra función especializada
    if (method === "efectivo") {
      console.log("🔄 Redirigiendo a handleCashPayment");
      handleCashPayment();
      return;
    }

    // Re-habilitar el flujo de Mercado Pago para QR, ahora con generación local de QR
    if (method === "qr") {
      console.log(
        "🔄 Iniciando flujo de pago con QR de Mercado Pago (generación local)"
      );
      generateQRPayment();
      return;
    }

    // Para otros métodos, continuar con el flujo normal
    console.log("🔄 Estado actual:", {
      isProcessingPayment,
      selectedPaymentMethod,
      roundedAmountDialogOpen,
    });

    // Prevenir procesamiento duplicado
    if (isProcessingPayment || selectedPaymentMethod) {
      console.log(
        "⚠️ Procesamiento bloqueado - ya está procesando o hay método seleccionado"
      );
      return;
    }

    // Establecer el método seleccionado y marcar como procesando
    setSelectedPaymentMethod(method);
    setIsProcessingPayment(true);

    // Procesar directamente los métodos que no son efectivo
    console.log("🔄 Procesando pago con:", method);
    await processPayment(method, Number(calculateTotal().toFixed(2)));
  };

  // Función separada para procesar el pago
  const processPayment = async (method: string, finalTotal: number) => {
    setIsProcessingPayment(true);

    const currentScreen = screens[activeScreen];
    const orderItems = currentScreen.items.map((item) => ({
      productoId: item.id,
      cantidad: item.quantity,
      subtotal: Number(item.subtotal.toFixed(2)),
      precioHistorico: item.pricePerUnit,
      costo: Number(item.costo),
      nombre: item.name,
    }));

    // Asegurar que user no sea null
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      return;
    }

    // Para métodos de pago únicos, usar metodoPago directamente
    const orderData = {
      metodoPago: method, // Campo único para pagos simples
      total: finalTotal,
      items: orderItems,
      vendedorId: user.id,
      sucursalId: user.sucursalId,
      vendedor: user.nombre,
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
        toast.info("Imprimiendo ticket...", {
          duration: 3000,
          description: "Enviando datos a la impresora",
        });

        const result = await ipcRenderer.invoke("print-ticket", orderData);
        console.log("Resultado de impresión:", result);

        if (result.success) {
          toast.success("Ticket impreso correctamente");
          console.log(
            "Detalles de impresión:",
            result.details || "No hay detalles adicionales"
          );

          // Mostrar si se imprimió el logo
          if (result.logoStatus) {
            console.log("Estado del logo:", result.logoStatus);
            if (result.logoStatus === "success") {
              console.log("✅ Logo impreso correctamente");
            } else {
              console.log("❌ Error al imprimir logo:", result.logoError);
            }
          }
        } else {
          throw new Error(result.message || "Error desconocido al imprimir");
        }
      } catch (printError: any) {
        console.error("Error detallado al imprimir:", printError);
        toast.error(`Error al imprimir el ticket: ${printError.message}`);
      }

      // Limpiar todos los estados relacionados con el pago
      setPaymentDialogOpen(false);
      setRoundedAmountDialogOpen(false);
      setScreens(
        screens.map((screen, index) =>
          index === activeScreen ? { ...screen, items: [] } : screen
        )
      );
      setSelectedPaymentMethod(null);
      setIsProcessingPayment(false);
      setOriginalAmount(0);
      setRoundedAmount(0);

      toast.success("Orden completada exitosamente");
      // Devolver el foco al input de búsqueda
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al procesar la orden");

      // También limpiar estados en caso de error
      setPaymentDialogOpen(false);
      setRoundedAmountDialogOpen(false);
      setSelectedPaymentMethod(null);
      setIsProcessingPayment(false);
      setOriginalAmount(0);
      setRoundedAmount(0);
    }
  };

  // Función para confirmar pago con monto redondeado
  const confirmRoundedPayment = () => {
    console.log("🔄 Iniciando confirmRoundedPayment");
    console.log("🔄 Estado actual:", {
      isProcessingPayment,
      selectedPaymentMethod,
      roundedAmount,
      originalAmount,
    });

    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso, ignorando solicitud");
      return;
    }

    console.log("✅ Procesando pago:", roundedAmount);

    // Siempre procesar como "efectivo", independientemente de si tiene descuento o no
    processPayment("efectivo", roundedAmount);
  };

  const handlePaymentClick = () => {
    const currentScreen = screens[activeScreen];
    if (currentScreen.items.length === 0) {
      toast.error("No hay productos en el carrito", {
        description: "Agrega al menos un producto antes de continuar",
      });
      return;
    }
    setPaymentDialogOpen(true);
  };

  const handleCancelClick = () => {
    const currentScreen = screens[activeScreen];
    if (currentScreen.items.length === 0) {
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

    // Activar estado de carga
    setIsClosing(true);

    try {
      // Removing unused API call for last closing since we're not using it anymore

      let startDate;
      if (period === "mañana") {
        startDate = new Date();
        startDate.setHours(6, 0, 0, 0);
      } else if (period === "tarde") {
        // Usar mediodía del día actual en lugar de la fecha del último cierre
        // para evitar incluir transacciones de días anteriores
        startDate = new Date();
        startDate.setHours(12, 0, 0, 0);
      } else {
        startDate = new Date();
        startDate.setHours(0, 0, 0, 0);
      }

      // Preparar datos para el cierre - simplificado porque el backend ahora hace los cálculos
      const closingData = {
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        fechaInicio: startDate,
        fechaCierre: new Date(),
        periodo: period,
      };

      console.log("🔄 Enviando solicitud de cierre con datos:", closingData);

      // Enviar datos al endpoint de cierres
      const cierreResponse = await fetch(`${API_URL}/api/cierres`, {
        method: "POST",
        headers,
        body: JSON.stringify(closingData),
      });

      // Obtener los datos de la respuesta
      const responseData = await cierreResponse.json();

      if (!cierreResponse.ok) {
        // Verificar si es el error específico de cierre de mañana requerido
        if (responseData.error === "ERROR_CIERRE_MAÑANA_REQUERIDO") {
          toast.error(
            "No puedes realizar un cierre de tarde sin haber realizado el cierre de mañana del día actual.",
            {
              duration: 5000,
              description: "Primero debes realizar el cierre de mañana",
            }
          );
        } else {
          // Otros errores
          throw new Error(
            responseData.message || "Error al registrar el cierre"
          );
        }
        return;
      }

      // Los datos devueltos ahora tienen toda la información necesaria
      const cierreData = await responseData;
      console.log("✅ Datos de cierre recibidos:", cierreData);

      // Imprimir ticket de cierre usando Electron IPC
      try {
        const { ipcRenderer } = window.require("electron");
        const printingToast = toast.loading("Imprimiendo ticket de cierre...");

        // Log de representación del ticket que se imprimirá
        console.log("====== SIMULACIÓN DEL TICKET DE CIERRE ======");
        console.log(`CIERRE DE CAJA - ${period.toUpperCase()}`);
        console.log(`Fecha inicio: ${new Date(startDate).toLocaleString()}`);
        console.log(`Fecha cierre: ${new Date().toLocaleString()}`);
        console.log("-------------------------------------");
        console.log("VENTAS POR MÉTODO DE PAGO:");

        // Mostrar ventas por método de pago
        if (cierreData.ventasPorMetodo) {
          Object.entries(cierreData.ventasPorMetodo).forEach(
            ([metodo, total]) => {
              console.log(`${metodo}: $${Number(total).toLocaleString()}`);
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
                    `  ${metodo}: $${Number(total).toLocaleString()}`
                  );
                }
              );
            }
          });
        }

        console.log("-------------------------------------");
        console.log(
          `TOTAL: $${Number(cierreData.totalVentas).toLocaleString()} (${
            cierreData.cantidadVentas
          } ventas)`
        );
        console.log("======================================");

        // Usar directamente los datos del backend sin manipulación adicional
        const result = await ipcRenderer.invoke("print-closing", cierreData);

        toast.dismiss(printingToast);

        if (result.success && !result.printerError) {
          toast.success("Ticket de cierre impreso correctamente");
        } else if (result.printerError) {
          toast.error(`No se pudo imprimir: ${result.printerError}`);
        }
      } catch (printError: any) {
        toast.error(`Error al imprimir: ${printError.message}`);
      }

      toast.success(`Cierre de ${period} realizado correctamente`);
      setClosingDialogOpen(false);
    } catch (error: any) {
      toast.error(`Error: ${error.message || "Error al realizar el cierre"}`);
    } finally {
      // Desactivar estado de carga independientemente del resultado
      setIsClosing(false);
    }
  };

  // Modificar el manejador de eventos de teclado
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
        const currentScreen = screens[activeScreen];
        if (e.key === "F1") {
          if (currentScreen.items.length === 0) {
            toast.error("No hay productos en el carrito", {
              description: "El carrito ya está vacío",
            });
            return;
          }
          handleCancelClick();
        } else {
          if (currentScreen.items.length === 0) {
            toast.error("No hay productos en el carrito", {
              description: "Agrega al menos un producto antes de continuar",
            });
            return;
          }
          handlePaymentClick();
        }
        return;
      }

      // Para el resto de atajos, verificar que no estemos en un input
      if (isInputElement) return;

      // Solo procesar atajos de cierre si el usuario tiene permiso y no está cargando
      if (
        closingDialogOpen &&
        user?.permisos?.cierreDeCajaEnabled &&
        !isClosing
      ) {
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

      // Para el diálogo de pago, verificar los estados antes de procesar las teclas
      if (paymentDialogOpen) {
        console.log("⌨️ TECLADO: Tecla presionada en diálogo de pago:", e.key);
        console.log("⌨️ TECLADO: Estados actuales:", {
          isProcessingPayment,
          selectedPaymentMethod,
          roundedAmountDialogOpen,
          businessInfo: businessInfo?.sistemaPago,
          descuentoEfectivo: businessInfo?.descuentoEfectivo,
        });

        // Si ya hay un método seleccionado o se está procesando, evitar nuevas selecciones
        if (isProcessingPayment || selectedPaymentMethod) {
          console.log(
            "⌨️ TECLADO: Tecla ignorada - ya está procesando o hay método seleccionado"
          );
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
            // Usar el nuevo manejador especializado para efectivo
            console.log(
              "⌨️ TECLADO: Tecla 3 detectada - Iniciando flujo de efectivo"
            );
            handleCashPayment(false);
            break;
          case "4":
            e.preventDefault();
            console.log(
              "⌨️ TECLADO: Tecla 4 detectada - Iniciando flujo de pago mixto"
            );
            handleSplitPayment();
            break;
        }
      }
    };

    window.addEventListener("keydown", handleGlobalKeyPress);
    return () => window.removeEventListener("keydown", handleGlobalKeyPress);
  }, [
    closingDialogOpen,
    paymentDialogOpen,
    screens,
    activeScreen,
    isProcessingPayment,
    selectedPaymentMethod,
    isClosing,
    businessInfo,
    calculateTotal,
    roundedAmountDialogOpen,
    applyingDiscount,
  ]);

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
  };

  const handleAddProduct = (product: Product) => {
    const currentScreen = screens[activeScreen];
    const existingItem = currentScreen.items.find(
      (item) => item.id === product.id
    );

    if (existingItem) {
      const updatedItems = currentScreen.items.map((item) =>
        item.id === product.id ? { ...item, quantity: item.quantity + 1 } : item
      );
      setScreens(
        screens.map((screen, index) =>
          index === activeScreen ? { ...screen, items: updatedItems } : screen
        )
      );
    } else {
      setScreens(
        screens.map((screen, index) =>
          index === activeScreen
            ? {
                ...screen,
                items: [...screen.items, { ...product, quantity: 1 }],
              }
            : screen
        )
      );
    }
  };

  const handleRemoveProduct = (productId: number) => {
    const currentScreen = screens[activeScreen];
    const updatedItems = currentScreen.items.filter(
      (item) => item.id !== productId
    );
    setScreens(
      screens.map((screen, index) =>
        index === activeScreen ? { ...screen, items: updatedItems } : screen
      )
    );
  };

  const [deleteScreenDialogOpen, setDeleteScreenDialogOpen] = useState(false);
  const [screenToDelete, setScreenToDelete] = useState<number | null>(null);

  const handleAddScreen = () => {
    if (screens.length >= 4) {
      toast.error("No se pueden crear más de 4 pantallas");
      return;
    }
    const newId = screens.length;
    setScreens([...screens, { id: newId, items: [] }]);
    setActiveScreen(newId);
  };

  const handleDeleteScreen = (screenId: number) => {
    if (screens.length <= 1) {
      toast.error("No se puede eliminar la última pantalla");
      return;
    }
    setScreenToDelete(screenId);
    setDeleteScreenDialogOpen(true);
  };

  const confirmDeleteScreen = () => {
    if (screenToDelete === null) return;

    const newScreens = screens.filter((screen) => screen.id !== screenToDelete);
    setScreens(newScreens);

    // Si la pantalla activa es la que se eliminó, cambiar a la primera pantalla
    if (activeScreen === screenToDelete) {
      setActiveScreen(0);
    }

    setDeleteScreenDialogOpen(false);
    setScreenToDelete(null);
    toast.success("Pantalla eliminada correctamente");
  };

  // Agregar estados necesarios para el diálogo de órdenes recientes
  const [ordersDialogOpen, setOrdersDialogOpen] = useState(false);
  const [recentOrders, setRecentOrders] = useState<any[]>([]);
  const [isLoadingOrders, setIsLoadingOrders] = useState(false);
  const [isPrinting, setIsPrinting] = useState(false);

  // Función para cargar órdenes recientes
  const loadRecentOrders = async () => {
    if (!user) {
      toast.error("Debes iniciar sesión para ver órdenes");
      return;
    }

    setIsLoadingOrders(true);

    try {
      // Usar el nuevo endpoint específico para órdenes del vendedor
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

  // Abrir diálogo de órdenes y cargar datos
  const handleOrdersButtonClick = () => {
    setOrdersDialogOpen(true);
    loadRecentOrders();
  };

  // Agregar los estados necesarios para la integración con Mercado Pago QR
  const [qrData, setQrData] = useState<any>(null);
  const [qrDialogOpen, setQrDialogOpen] = useState(false);
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(
    null
  );
  const [paymentStatus, setPaymentStatus] = useState<string | null>(null);

  // Actualizar la función generateQRPayment para ajustar el manejo de la respuesta
  const generateQRPayment = async () => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso");
      return;
    }

    setIsProcessingPayment(true);
    setSelectedPaymentMethod("qr");

    try {
      const currentScreen = screens[activeScreen];
      const orderItems = currentScreen.items.map((item) => ({
        productoId: item.id,
        nombre: item.name,
        cantidad: item.quantity,
        subtotal: Number(item.subtotal.toFixed(2)),
        precioHistorico: item.pricePerUnit,
        costo: Number(item.costo),
      }));

      const orderData = {
        monto: Number(calculateTotal().toFixed(2)),
        descripcion: `Compra de ${orderItems.length} productos`,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        externalPosId: import.meta.env.VITE_POS_ID, // El POS ID que ya está configurado
        items: orderItems,
      };

      console.log("🔄 Enviando solicitud para generar QR:", orderData);

      // Cerrar diálogo de pagos y mostrar cargando
      setPaymentDialogOpen(false);
      toast.loading("Generando código QR...", { id: "qr-loading" });

      // Realizar la solicitud para generar el QR
      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
        body: JSON.stringify(orderData),
      });

      if (!response.ok) {
        throw new Error("Error al generar el código QR");
      }

      const data = await response.json();
      console.log("✅ QR generado:", data);

      // Log all properties for debugging
      console.log("QR response properties:", Object.keys(data));
      for (const key of Object.keys(data)) {
        console.log(`QR response ${key}:`, data[key]);
      }

      // Generar la imagen QR del lado del cliente usando la biblioteca QRCode
      // en lugar de usar la URL de imagen proporcionada por la API
      let qrImageDataUrl;
      try {
        if (data.qrData) {
          // Generar QR a partir de los datos recibidos
          qrImageDataUrl = await QRCode.toDataURL(data.qrData, {
            errorCorrectionLevel: "H",
            margin: 1,
            width: 256,
            color: {
              dark: "#000000", // Color de los puntos del QR
              light: "#FFFFFF", // Color de fondo
            },
          });
          console.log("✅ QR generado localmente con éxito");
        } else {
          console.error("❌ No se recibieron datos de QR del servidor");
          throw new Error("Datos de QR no disponibles");
        }
      } catch (qrError: any) {
        console.error("❌ Error al generar QR local:", qrError);
        // Si falla la generación local, intentar usar la URL proporcionada por la API
        qrImageDataUrl = data.qrImageUrl || null;
        if (!qrImageDataUrl) {
          throw new Error("No se pudo generar ni obtener imagen QR");
        }
      }

      // Agregar el dataUrl generado a los datos del QR
      setQrData({
        ...data,
        qrImageUrl: qrImageDataUrl, // Usar la imagen generada localmente
        monto: orderData.monto,
        items: orderItems,
      });

      toast.dismiss("qr-loading");
      setQrDialogOpen(true);

      // Iniciar el polling para verificar el estado del pago
      startPaymentStatusPolling(data.orderId);
    } catch (error: any) {
      console.error("❌ Error al generar QR:", error);
      toast.dismiss("qr-loading");
      toast.error(`Error al generar QR: ${error.message}`);
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    }
  };

  // Agregar función para iniciar el polling del estado del pago
  const startPaymentStatusPolling = (orderId: number) => {
    console.log(
      "🔄 Iniciando polling para verificar estado del pago:",
      orderId
    );

    setPaymentStatus("PENDIENTE");

    // Limpiar cualquier intervalo existente
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }

    // Crear intervalo de polling (cada 3 segundos)
    const interval = setInterval(async () => {
      try {
        console.log("🔄 Verificando estado del pago...");
        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status?orderId=${orderId}`,
          {
            headers,
          }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        console.log("🔄 Estado actual del pago:", statusData);

        setPaymentStatus(statusData.status);

        // Si el pago se completó o canceló, detener el polling
        if (statusData.isCompleted || statusData.isCancelled) {
          clearInterval(interval);
          setPollingInterval(null);

          if (statusData.isCompleted) {
            console.log("✅ Pago completado exitosamente");

            // Set a timeout to automatically close the dialog after 5 seconds
            toast.success("¡Pago completado! Cerrando en 5 segundos...");
            setTimeout(() => {
              setQrDialogOpen(false);
              // Clear cart items
              setScreens(
                screens.map((screen, index) =>
                  index === activeScreen ? { ...screen, items: [] } : screen
                )
              );
              console.log(
                "✅ Diálogo cerrado automáticamente después del pago"
              );
              setIsProcessingPayment(false);
              setSelectedPaymentMethod(null);
              // Return focus to search input
              setTimeout(() => {
                searchInputRef.current?.focus();
              }, 100);
            }, 5000);

            // Capturar una copia de los datos necesarios para la impresión antes de cualquier limpieza
            const currentQrData = qrData;

            // Crear la orden en el sistema y limpiar el carrito
            if (
              currentQrData &&
              currentQrData.items &&
              currentQrData.items.length > 0
            ) {
              const ticketItems = [...currentQrData.items];
              const ticketMonto = currentQrData.monto;

              // Añadir los datos capturados al statusData para usarlos en finalizeMPPayment
              statusData.ticketItems = ticketItems;
              statusData.ticketMonto = ticketMonto;
              console.log("✅ Datos del ticket capturados correctamente:", {
                items: ticketItems.length,
                monto: ticketMonto,
              });
            } else {
              console.error(
                "❌ No se pudieron capturar los datos del ticket en el momento del pago",
                currentQrData
              );
            }

            finalizeMPPayment(statusData);
          } else {
            console.log("❌ Pago cancelado o rechazado");
            toast.error("El pago ha sido cancelado o rechazado");
            setQrDialogOpen(false);
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
          }
        }
      } catch (error: any) {
        console.error("❌ Error al verificar estado:", error);
      }
    }, 3000);

    setPollingInterval(interval);

    // Establecer un tiempo máximo de espera (5 minutos)
    setTimeout(() => {
      if (pollingInterval) {
        clearInterval(pollingInterval);
        setPollingInterval(null);
        console.log("⏱️ Tiempo de espera agotado");
        toast.error("Tiempo de espera agotado. Intente nuevamente.");
        setQrDialogOpen(false);
        setIsProcessingPayment(false);
        setSelectedPaymentMethod(null);
      }
    }, 5 * 60 * 1000);
  };

  // Función para finalizar el pago después de que MP confirme
  const finalizeMPPayment = async (paymentData: any) => {
    try {
      console.log("🔄 Finalizando pago con datos:", paymentData);
      console.log("🔄 Estado de qrData:", qrData);

      // Verificar que user no sea null
      if (!user) {
        toast.error("Se perdió la sesión. Por favor inicia sesión nuevamente.");
        setQrDialogOpen(false);
        setIsProcessingPayment(false);
        setSelectedPaymentMethod(null);
        return;
      }

      // Imprimir ticket solo si el pago fue completado
      if (paymentData.isCompleted) {
        try {
          console.log("💰 Pago completado, preparando para crear orden en BD");

          // Primero intentar usar los datos capturados en statusData
          let ticketItems = paymentData.ticketItems;
          let ticketMonto = paymentData.ticketMonto;

          // Si no hay datos capturados, intentar usar qrData como respaldo
          if (!ticketItems && qrData && qrData.items) {
            console.log("⚠️ Usando datos de respaldo de qrData");
            ticketItems = qrData.items;
            ticketMonto = qrData.monto;
          }

          // Verificar que tenemos los datos necesarios
          if (!ticketItems || ticketItems.length === 0) {
            console.error(
              "❌ No se encontraron datos de items para el ticket",
              {
                ticketItems,
                ticketMonto,
                paymentData,
                qrData,
              }
            );

            // Como último recurso, crear un item genérico para imprimir al menos el total
            console.log("⚠️ Creando item genérico para el ticket");
            ticketItems = [
              {
                nombre: "Pago con QR",
                cantidad: 1,
                subtotal: paymentData.total || 0,
                precioHistorico: paymentData.total || 0,
                costo: 0,
              },
            ];
            ticketMonto = paymentData.total || 0;
          }

          // Crear los datos para la orden en BD
          const orderData = {
            metodoPago: "qr",
            total: paymentData.total || ticketMonto,
            items: ticketItems.map((item: any) => ({
              productoId: item.productoId || item.id || 0,
              cantidad: item.cantidad || item.quantity || 1,
              subtotal: Number((item.subtotal || 0).toFixed(2)),
              precioHistorico: item.precioHistorico || item.pricePerUnit || 0,
              costo: Number((item.costo || 0).toFixed(2)),
              nombre: item.nombre || item.name || "Producto",
            })),
            vendedorId: user.id,
            sucursalId: user.sucursalId,
            vendedor: user.nombre,
            createdAt: new Date().toISOString(),
            referencia: paymentData.orderId?.toString() || "unknown",
          };

          console.log("💾 Guardando orden en BD:", orderData);

          // Crear la orden en la BD
          const orderResponse = await fetch(`${API_URL}/api/ordenes`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              ...(appId && { "X-App-ID": appId }),
            },
            body: JSON.stringify(orderData),
          });

          if (!orderResponse.ok) {
            console.error(
              "❌ Error al crear la orden en BD:",
              await orderResponse.text()
            );
            throw new Error("Error al crear la orden en base de datos");
          }

          console.log("✅ Orden creada correctamente en BD");

          // Ahora imprimir el ticket
          console.log(
            "🖨️ Imprimiendo ticket con datos:",
            JSON.stringify(orderData)
          );

          const { ipcRenderer } = window.require("electron");
          toast.loading("Imprimiendo ticket...", { id: "print-ticket" });

          try {
            const result = await ipcRenderer.invoke("print-ticket", orderData);
            console.log("🖨️ Resultado de impresión:", result);

            toast.dismiss("print-ticket");
            if (result.success) {
              toast.success("Ticket impreso correctamente");
            } else {
              console.error("❌ Error al imprimir:", result.message);
              toast.error(`Error al imprimir: ${result.message}`);
            }
          } catch (innerError: any) {
            console.error("❌ Error en invoke print-ticket:", innerError);
            toast.dismiss("print-ticket");
            toast.error(`Error al invocar impresión: ${innerError.message}`);
          }

          // Esperar un poco antes de continuar para asegurar que la impresión se complete
          await new Promise((resolve) => setTimeout(resolve, 1000));
        } catch (printError: any) {
          console.error("❌ Error general al procesar/imprimir:", printError);
          toast.error(`Error: ${printError.message}`);
        }
      } else {
        console.log(
          "⚠️ El pago no está marcado como completado, no se creará orden ni imprimirá ticket"
        );
      }

      // Ahora que la impresión ha terminado (o falló), podemos limpiar estados
      console.log("🧹 Limpiando estados después del pago");
      setQrDialogOpen(false);
      setQrData(null);
      setPaymentStatus(null);
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);

      // Limpiar el carrito
      setScreens(
        screens.map((screen, index) =>
          index === activeScreen ? { ...screen, items: [] } : screen
        )
      );

      toast.success("Pago completado exitosamente");

      // Devolver el foco al input de búsqueda
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
    } catch (error: any) {
      console.error("❌ Error al finalizar pago:", error);
      toast.error(`Error al finalizar el pago: ${error.message}`);
      setQrDialogOpen(false);
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
    }
  };

  // Cancelar el pago con QR
  const cancelQRPayment = () => {
    // Limpiar el intervalo de polling
    if (pollingInterval) {
      clearInterval(pollingInterval);
      setPollingInterval(null);
    }

    // Cerrar el diálogo y limpiar estados
    setQrDialogOpen(false);
    setQrData(null);
    setPaymentStatus(null);
    setIsProcessingPayment(false);
    setSelectedPaymentMethod(null);

    console.log("❌ Pago con QR cancelado por el usuario");
  };

  // Limpiar intervalos cuando se desmonte el componente
  useEffect(() => {
    return () => {
      if (pollingInterval) {
        clearInterval(pollingInterval);
      }
    };
  }, [pollingInterval]);

  // Función para manejar el pago mixto
  const handleSplitPayment = () => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso");
      return;
    }

    // Establecer el método seleccionado
    setSelectedPaymentMethod("split");

    // Inicializar el efectivo con un valor vacío
    setCashAmount("");

    // Establecer tarjeta como método secundario por defecto
    setSecondPaymentMethod("tarjeta");

    // Cerrar el diálogo de pago y abrir el diálogo de pago mixto
    setPaymentDialogOpen(false);
    setSplitPaymentDialogOpen(true);
  };

  // Función para procesar el pago mixto
  const processSplitPayment = async () => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Validar que el monto de efectivo sea válido
    const cashAmountValue = parseFloat(cashAmount);
    if (isNaN(cashAmountValue) || cashAmountValue <= 0) {
      toast.error("Ingresa un monto válido para el pago en efectivo");
      return;
    }

    const totalAmount = calculateTotal();

    // Validar que el monto de efectivo no sea mayor al total
    if (cashAmountValue > totalAmount) {
      toast.error("El monto en efectivo no puede ser mayor al total");
      return;
    }

    // Calcular el monto restante para el segundo método de pago
    const secondAmount = parseFloat((totalAmount - cashAmountValue).toFixed(2));

    // Validar que el monto restante sea positivo
    if (secondAmount <= 0) {
      toast.error("El monto para el segundo método debe ser mayor a cero");
      return;
    }

    // Si el segundo método es QR, generar el QR
    if (secondPaymentMethod === "qr") {
      generateSplitQRPayment(cashAmountValue, secondAmount);
      return;
    }

    // Si es tarjeta, continuar con el flujo normal
    setIsProcessingPayment(true);

    const currentScreen = screens[activeScreen];
    const orderItems = currentScreen.items.map((item) => ({
      productoId: item.id,
      cantidad: item.quantity,
      subtotal: Number(item.subtotal.toFixed(2)),
      precioHistorico: item.pricePerUnit,
      costo: Number(item.costo),
      nombre: item.name,
    }));

    // Crear la estructura de pagos múltiples siguiendo el formato API
    const orderData = {
      total: Number(totalAmount.toFixed(2)),
      items: orderItems,
      vendedorId: user.id,
      sucursalId: user.sucursalId,
      vendedor: user.nombre,
      createdAt: new Date().toISOString(),
      // Array de pagos con los dos métodos
      pagos: [
        {
          metodoPago: "efectivo",
          monto: cashAmountValue,
        },
        {
          metodoPago: secondPaymentMethod,
          monto: secondAmount,
        },
      ],
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
        toast.info("Imprimiendo ticket...", {
          duration: 3000,
          description: "Enviando datos a la impresora",
        });

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

      // Limpiar todos los estados relacionados con el pago
      setSplitPaymentDialogOpen(false);
      setScreens(
        screens.map((screen, index) =>
          index === activeScreen ? { ...screen, items: [] } : screen
        )
      );
      setSelectedPaymentMethod(null);
      setIsProcessingPayment(false);
      setCashAmount("");
      setSecondPaymentMethod("tarjeta");

      toast.success("Orden completada exitosamente");
      // Devolver el foco al input de búsqueda
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 100);
    } catch (error) {
      console.error("Error:", error);
      toast.error("Error al procesar la orden");

      // También limpiar estados en caso de error
      setSplitPaymentDialogOpen(false);
      setSelectedPaymentMethod(null);
      setIsProcessingPayment(false);
      setCashAmount("");
      setSecondPaymentMethod("tarjeta");
    }
  };

  // Función para generar QR en caso de pago mixto con QR
  const generateSplitQRPayment = async (
    cashAmountValue: number,
    qrAmount: number
  ) => {
    if (!user) {
      toast.error("Debes iniciar sesión para realizar una orden");
      return;
    }

    // Prevenir procesamiento duplicado
    if (isProcessingPayment) {
      console.log("⚠️ Ya hay un pago en proceso");
      return;
    }

    setIsProcessingPayment(true);

    try {
      const currentScreen = screens[activeScreen];
      // Guardar los items originales para usarlos después en la orden final
      const originalOrderItems = currentScreen.items.map((item) => ({
        productoId: item.id,
        nombre: item.name,
        cantidad: item.quantity,
        subtotal: Number(item.subtotal.toFixed(2)),
        precioHistorico: item.pricePerUnit,
        costo: Number(item.costo),
      }));

      // Guardar información del pago en efectivo para usarla después
      const splitPaymentInfo = {
        cashAmount: cashAmountValue,
        total: calculateTotal(),
        qrAmount: qrAmount,
        originalItems: originalOrderItems,
      };

      localStorage.setItem(
        "splitPaymentInfo",
        JSON.stringify(splitPaymentInfo)
      );

      // Acceder a las propiedades del usuario de forma segura
      const userId = user.id;
      const sucursalId = user.sucursalId;

      // Usar el primer producto de la lista como base para el pago parcial
      // para asegurarnos de tener un productoId válido
      const firstProduct = currentScreen.items[0];

      // Crear un único item que representa el pago parcial
      // para que el monto total coincida con el monto del QR
      const singleQrItem = {
        productoId: firstProduct.id, // Usar un ID de producto válido
        nombre: "Pago parcial",
        cantidad: 1,
        subtotal: qrAmount,
        precioHistorico: qrAmount,
        costo: 0,
      };

      console.log("🔄 Creando solicitud QR para pago parcial de:", qrAmount);

      const orderData = {
        monto: qrAmount, // Solo el monto para QR
        descripcion: `Pago parcial de $${qrAmount.toFixed(2)}`,
        vendedorId: userId,
        sucursalId: sucursalId,
        externalPosId: import.meta.env.VITE_POS_ID,
        // Enviar solo un item que representa el pago parcial
        items: [singleQrItem],
        // Metadatos adicionales
        isSplitPayment: true,
        cashAmount: cashAmountValue,
        originalItems: originalOrderItems,
      };

      console.log(
        "🔄 Enviando solicitud para generar QR (pago mixto):",
        orderData
      );

      // Cerrar diálogo de pago mixto y mostrar cargando
      setSplitPaymentDialogOpen(false);
      toast.loading("Generando código QR...", { id: "qr-loading" });

      // Realizar la solicitud para generar el QR
      const response = await fetch(`${API_URL}/api/mercadopago/generate-qr`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(appId && { "X-App-ID": appId }),
        },
        body: JSON.stringify(orderData),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        console.error("Error detallado del servidor:", errorData);
        throw new Error(errorData.message || "Error al generar el código QR");
      }

      const data = await response.json();
      console.log("✅ QR generado (pago mixto):", data);

      // Generar la imagen QR del lado del cliente
      let qrImageDataUrl;
      try {
        if (data.qrData) {
          qrImageDataUrl = await QRCode.toDataURL(data.qrData, {
            errorCorrectionLevel: "H",
            margin: 1,
            width: 256,
            color: {
              dark: "#000000",
              light: "#FFFFFF",
            },
          });
          console.log("✅ QR generado localmente con éxito");
        } else {
          console.error("❌ No se recibieron datos de QR del servidor");
          throw new Error("Datos de QR no disponibles");
        }
      } catch (qrError: any) {
        console.error("❌ Error al generar QR local:", qrError);
        qrImageDataUrl = data.qrImageUrl || null;
        if (!qrImageDataUrl) {
          throw new Error("No se pudo generar ni obtener imagen QR");
        }
      }

      // Agregar información del pago mixto a los datos del QR
      setQrData({
        ...data,
        qrImageUrl: qrImageDataUrl,
        monto: qrAmount,
        items: originalOrderItems,
        isSplitPayment: true,
        cashAmount: cashAmountValue,
      });

      toast.dismiss("qr-loading");
      setQrDialogOpen(true);

      // Iniciar el polling para verificar el estado del pago
      startSplitPaymentStatusPolling(data.orderId, cashAmountValue);
    } catch (error: any) {
      console.error("❌ Error al generar QR para pago mixto:", error);
      toast.dismiss("qr-loading");
      toast.error(`Error al generar QR: ${error.message}`);
      setIsProcessingPayment(false);
      setSelectedPaymentMethod(null);
      setCashAmount("");
      setSecondPaymentMethod("tarjeta");
    }
  };

  // Función para verificar el estado del pago con QR en caso de pago mixto
  const startSplitPaymentStatusPolling = (
    orderId: number,
    cashAmount: number
  ) => {
    console.log(
      "🔄 Iniciando polling para verificar estado del pago mixto:",
      orderId
    );

    setPaymentStatus("PENDIENTE");

    // Limpiar cualquier intervalo existente
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }

    // Crear intervalo de polling (cada 3 segundos)
    const interval = setInterval(async () => {
      try {
        console.log("🔄 Verificando estado del pago mixto...");
        const response = await fetch(
          `${API_URL}/api/mercadopago/check-status?orderId=${orderId}`,
          {
            headers,
          }
        );

        if (!response.ok) {
          throw new Error("Error al verificar estado del pago");
        }

        const statusData = await response.json();
        console.log("🔄 Estado actual del pago mixto:", statusData);

        setPaymentStatus(statusData.status);

        // Si el pago se completó o canceló, detener el polling
        if (statusData.isCompleted || statusData.isCancelled) {
          clearInterval(interval);
          setPollingInterval(null);

          if (statusData.isCompleted) {
            console.log("✅ Pago mixto QR completado exitosamente");

            // Set a timeout to automatically close the dialog after 5 seconds
            toast.success("¡Pago completado! Cerrando en 2 segundos...");
            setTimeout(() => {
              setQrDialogOpen(false);
              // Clear cart items
              setScreens(
                screens.map((screen, index) =>
                  index === activeScreen ? { ...screen, items: [] } : screen
                )
              );
              console.log(
                "✅ Diálogo cerrado automáticamente después del pago mixto"
              );
              setIsProcessingPayment(false);
              setSelectedPaymentMethod(null);
              setCashAmount("");
              setSecondPaymentMethod("tarjeta");
              // Return focus to search input
              setTimeout(() => {
                searchInputRef.current?.focus();
              }, 100);
            }, 2000);

            finalizeSplitMPPayment(statusData, cashAmount);
          } else {
            console.log("❌ Pago mixto QR cancelado o rechazado");
            toast.error("El pago ha sido cancelado o rechazado");
            setQrDialogOpen(false);
            setIsProcessingPayment(false);
            setSelectedPaymentMethod(null);
            setCashAmount("");
            setSecondPaymentMethod("tarjeta");
          }
        }
      } catch (error: any) {
        console.error("❌ Error al verificar estado:", error);
      }
    }, 3000);

    setPollingInterval(interval);

    // Establecer un tiempo máximo de espera (5 minutos)
    setTimeout(() => {
      if (pollingInterval) {
        clearInterval(pollingInterval);
        setPollingInterval(null);
        console.log("⏱️ Tiempo de espera agotado");
        toast.error("Tiempo de espera agotado. Intente nuevamente.");
        setQrDialogOpen(false);
        setIsProcessingPayment(false);
        setSelectedPaymentMethod(null);
        setCashAmount("");
        setSecondPaymentMethod("tarjeta");
      }
    }, 5 * 60 * 1000);
  };

  // Función para finalizar el pago mixto después de que QR sea confirmado
  const finalizeSplitMPPayment = async (
    paymentData: any,
    cashAmount: number
  ) => {
    try {
      console.log("🔄 Finalizando pago mixto con datos:", paymentData);
      console.log("🔄 Monto en efectivo:", cashAmount);

      // Verificar que user no sea null
      if (!user) {
        toast.error(
          "Sesión no disponible. Por favor inicia sesión nuevamente."
        );
        return;
      }

      const totalAmount = calculateTotal();
      const qrAmount = totalAmount - cashAmount;

      const currentScreen = screens[activeScreen];
      const orderItems = currentScreen.items.map((item) => ({
        productoId: item.id,
        cantidad: item.quantity,
        subtotal: Number(item.subtotal.toFixed(2)),
        precioHistorico: item.pricePerUnit,
        costo: Number(item.costo),
        nombre: item.name,
      }));

      // Crear la orden con los dos métodos de pago
      // Usar el formato correcto para pagos múltiples
      const orderData = {
        total: Number(totalAmount.toFixed(2)),
        items: orderItems,
        vendedorId: user.id,
        sucursalId: user.sucursalId,
        vendedor: user.nombre,
        createdAt: new Date().toISOString(),
        pagos: [
          {
            metodoPago: "efectivo",
            monto: cashAmount,
          },
          {
            metodoPago: "qr",
            monto: qrAmount,
            referencia: paymentData.orderId?.toString() || "unknown",
          },
        ],
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
          console.log(
            "Enviando datos para impresión de pago mixto:",
            orderData
          );
          toast.loading("Imprimiendo ticket...", { id: "print-ticket" });

          const result = await ipcRenderer.invoke("print-ticket", orderData);
          console.log("Resultado de impresión:", result);

          toast.dismiss("print-ticket");
          if (result.success) {
            toast.success("Ticket impreso correctamente");
          } else {
            console.error("❌ Error al imprimir:", result.message);
            toast.error(`Error al imprimir: ${result.message}`);
          }
        } catch (printError: any) {
          console.error("❌ Error al imprimir:", printError);
          toast.error(`Error al imprimir: ${printError.message}`);
        }

        // Limpiar todos los estados relacionados con el pago
        setQrDialogOpen(false);
        setQrData(null);
        setPaymentStatus(null);
        setScreens(
          screens.map((screen, index) =>
            index === activeScreen ? { ...screen, items: [] } : screen
          )
        );
        setSelectedPaymentMethod(null);
        setIsProcessingPayment(false);
        setCashAmount("");
        setSecondPaymentMethod("tarjeta");

        toast.success("Pago mixto completado exitosamente");

        // Devolver el foco al input de búsqueda
        setTimeout(() => {
          searchInputRef.current?.focus();
        }, 100);
      } catch (error: any) {
        console.error("❌ Error al finalizar pago mixto:", error);
        toast.error(`Error al finalizar el pago: ${error.message}`);

        // Limpiar estados en caso de error
        setQrDialogOpen(false);
        setQrData(null);
        setPaymentStatus(null);
        setSelectedPaymentMethod(null);
        setIsProcessingPayment(false);
        setCashAmount("");
        setSecondPaymentMethod("tarjeta");
      }
    } catch (error: any) {
      console.error("❌ Error general en finalizeSplitMPPayment:", error);
      toast.error(`Error al procesar el pago mixto: ${error.message}`);

      // Limpiar estados en caso de error
      setQrDialogOpen(false);
      setQrData(null);
      setPaymentStatus(null);
      setSelectedPaymentMethod(null);
      setIsProcessingPayment(false);
      setCashAmount("");
      setSecondPaymentMethod("tarjeta");
    }
  };

  console.log(user);
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
        <div className="flex w-full justify-between items-center mb-4">
          <div className="relative w-full">
            <div className="flex items-center gap-2">
              <Input
                type="text"
                placeholder="Buscar productos..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={handleKeyPress}
                className="w-[300px]"
                ref={searchInputRef}
              />
              <div className="flex items-center gap-2">
                {screens.map((screen) => (
                  <div key={screen.id} className="flex items-center gap-1">
                    <Button
                      variant={
                        activeScreen === screen.id ? "default" : "outline"
                      }
                      size="sm"
                      onClick={() => setActiveScreen(screen.id)}
                      className="h-8"
                    >
                      Orden {screen.id + 1}
                    </Button>
                    {screen.id > 0 && (
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 w-8 p-0 hover:bg-red-100 hover:text-red-600"
                        onClick={() => handleDeleteScreen(screen.id)}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                ))}
                {screens.length < 4 && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8"
                    onClick={handleAddScreen}
                  >
                    <Plus className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </div>

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
            {/* Botón de Órdenes recientes */}
            <Button
              className="bg-emerald-gradient text-white hover:text-white text-base [&_svg]:size-6"
              onClick={handleOrdersButtonClick}
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

        <div className="flex-1 space-y-2 overflow-auto mb-4">
          {screens[activeScreen].items.map((item) => (
            <Card
              key={item.cartId}
              className="bg-background border p-4 flex items-center justify-between shadow-sm rounded-xl"
            >
              <div className="flex justify-between w-full items-center">
                <div className="flex justify-between items-end gap-10">
                  <span className="text-2xl font-medium">{item.name}</span>
                  <div className="text-muted-foreground">
                    Cantidad: {item.quantity} {item.unit}
                  </div>
                  <div className="text-muted-foreground">
                    {item.unit === "Kg" ? "$/Kg" : "$/U"}: $
                    {item.pricePerUnit.toLocaleString()}
                  </div>
                  <div className="text-muted-foreground">
                    SUBTOTAL: ${item.subtotal.toLocaleString()}
                  </div>
                </div>

                <Button
                  variant="destructive"
                  size="icon"
                  className="h-8 w-8 rounded-lg bg-cancel-gradient"
                  onClick={() => handleRemoveProduct(item.id)}
                >
                  <Trash2 className="h-6 w-6" />
                </Button>
              </div>
            </Card>
          ))}
        </div>

        <Dialog
          open={dialogOpen}
          onOpenChange={(open) => {
            console.log("🔍 DEBUG: Cambio de estado del diálogo:", open);
            if (!open) {
              setDialogOpen(false);
              setIsAddingToCart(false); // Asegurar que se resetea el estado al cerrar

              // Devolver el foco al input de búsqueda cuando se cierra el diálogo
              setTimeout(() => {
                if (searchInputRef.current) {
                  searchInputRef.current.focus();
                  console.log("✓ DEBUG: Foco devuelto al input de búsqueda");
                }
              }, 100);
            } else {
              setDialogOpen(open);
            }
          }}
        >
          <DialogContent
            onKeyDown={(e) => {
              console.log(
                "🔑 DEBUG: Tecla en diálogo:",
                e.key,
                "isAddingToCart:",
                isAddingToCart
              );

              // Asegurar que sólo procesamos un evento a la vez
              if (e.key === "Enter" && !isAddingToCart) {
                e.preventDefault();
                e.stopPropagation(); // Esto es crítico - detiene la propagación del evento

                // Usar un setTimeout para asegurar que la ejecución se separa del evento
                setTimeout(() => {
                  console.log("🔍 DEBUG: Procesando tecla Enter para adición");

                  if (selectedProduct?.unit === "Kg" && !useManualWeight) {
                    console.log("⚖️ DEBUG: Adición por peso automático");
                    addToCart();
                  } else if (quantity) {
                    console.log(
                      "📦 DEBUG: Adición con cantidad manual:",
                      quantity
                    );
                    addToCart();
                  } else {
                    console.log(
                      "⚠️ DEBUG: Enter presionado pero faltan datos para añadir"
                    );
                  }
                }, 0);
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
                autoFocus
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
                tabIndex={2}
              >
                Cancelar
              </Button>
              <Button
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation(); // Prevenir propagación de eventos

                  // Usar un setTimeout para asegurar que la ejecución se desacopla del evento
                  setTimeout(() => {
                    console.log(
                      "🖱️ DEBUG: Clic en botón de agregar al carrito"
                    );

                    // Solo procesar si no estamos ya añadiendo
                    if (!isAddingToCart) {
                      addToCart();
                    } else {
                      console.log(
                        "⚠️ DEBUG: Clic ignorado, ya hay una adición en progreso"
                      );
                    }
                  }, 0);
                }}
                className="bg-emerald-gradient text-white hover:text-white text-base"
                type="button" // Cambiar a button para mejor control
                disabled={isAddingToCart}
                autoFocus={!(selectedProduct?.unit !== "Kg" || useManualWeight)}
                tabIndex={1}
              >
                {isAddingToCart ? (
                  <div className="flex items-center gap-2">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>Agregando...</span>
                  </div>
                ) : (
                  "Agregar al carrito"
                )}
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

        <Dialog
          open={paymentDialogOpen}
          onOpenChange={(open) => {
            if (!open) {
              // Resetear el método seleccionado al cerrar el diálogo
              setSelectedPaymentMethod(null);
              setIsProcessingPayment(false);
              // Devolver el foco al input de búsqueda cuando se cierra el diálogo
              setTimeout(() => {
                searchInputRef.current?.focus();
              }, 100);
            }
            setPaymentDialogOpen(open);
          }}
        >
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
                onClick={() => handlePayment("qr")}
                className={`h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8 ${
                  selectedPaymentMethod === "qr"
                    ? "bg-emerald-100 border-emerald-600 border-2"
                    : ""
                }`}
                variant="outline"
                disabled={
                  isProcessingPayment ||
                  (selectedPaymentMethod !== null &&
                    selectedPaymentMethod !== "qr")
                }
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment && selectedPaymentMethod === "qr" ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <QrCode className="h-12 w-12" />
                  )}
                </div>
                <span>Transferencia (1)</span>
              </Button>
              <Button
                onClick={() => handlePayment("tarjeta")}
                className={`h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8 ${
                  selectedPaymentMethod === "tarjeta"
                    ? "bg-emerald-100 border-emerald-600 border-2"
                    : ""
                }`}
                variant="outline"
                disabled={
                  isProcessingPayment ||
                  (selectedPaymentMethod !== null &&
                    selectedPaymentMethod !== "tarjeta")
                }
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment &&
                  selectedPaymentMethod === "tarjeta" ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <CreditCard className="h-12 w-12" />
                  )}
                </div>
                <span>Tarjeta (2)</span>
              </Button>
              <Button
                onClick={() => handleCashPayment()}
                className={`h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8 ${
                  selectedPaymentMethod === "efectivo"
                    ? "bg-emerald-100 border-emerald-600 border-2"
                    : ""
                }`}
                variant="outline"
                disabled={
                  isProcessingPayment ||
                  (selectedPaymentMethod !== null &&
                    selectedPaymentMethod !== "efectivo")
                }
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment &&
                  selectedPaymentMethod === "efectivo" ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <Wallet className="h-12 w-12" />
                  )}
                </div>
                <span>Efectivo (3)</span>
              </Button>
              <Button
                onClick={() => handleSplitPayment()}
                className={`h-32 flex flex-col items-center justify-center space-y-2 [&_svg]:size-8 ${
                  selectedPaymentMethod === "split"
                    ? "bg-emerald-100 border-emerald-600 border-2"
                    : ""
                }`}
                variant="outline"
                disabled={
                  isProcessingPayment ||
                  (selectedPaymentMethod !== null &&
                    selectedPaymentMethod !== "split")
                }
              >
                <div className="h-12 flex items-center justify-center">
                  {isProcessingPayment && selectedPaymentMethod === "split" ? (
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900" />
                  ) : (
                    <div className="flex items-center gap-1">
                      <Wallet className="h-10 w-10" />
                      <span className="text-xl">+</span>
                      <CreditCard className="h-10 w-10" />
                    </div>
                  )}
                </div>
                <span>Pago Mixto (4)</span>
              </Button>
            </div>
          </DialogContent>
        </Dialog>

        <Dialog
          open={closingDialogOpen}
          onOpenChange={(open) => {
            if (!open) {
              setClosingDialogOpen(false);
              // Devolver el foco al input de búsqueda cuando se cierra el diálogo
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
                      {new Date(closeResultData.fechaInicio).toLocaleString()} -{" "}
                      {new Date(closeResultData.fechaCierre).toLocaleString()}
                    </p>
                  </div>
                </div>

                <div className="rounded-lg border p-4">
                  <h3 className="text-lg font-semibold mb-2">
                    Resumen General
                  </h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <p className="text-sm text-muted-foreground">
                        Total Ventas
                      </p>
                      <p className="text-2xl font-bold text-emerald-600">
                        $
                        {Number(closeResultData.totalVentas).toLocaleString(
                          "es-AR",
                          { minimumFractionDigits: 2, maximumFractionDigits: 2 }
                        )}
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

                {closeResultData.ventasPorMetodo &&
                  closeResultData.ventasPorMetodo.ventasPorMetodo && (
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
                          {Object.entries(
                            closeResultData.ventasPorMetodo.ventasPorMetodo
                          ).map(([metodo, monto]: [string, any]) => (
                            <TableRow key={metodo}>
                              <TableCell className="font-medium capitalize">
                                {metodo}
                              </TableCell>
                              <TableCell className="text-right">
                                $
                                {Number(monto).toLocaleString("es-AR", {
                                  minimumFractionDigits: 2,
                                  maximumFractionDigits: 2,
                                })}
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  )}

                {closeResultData.ventasPorMetodo &&
                  closeResultData.ventasPorMetodo.ventasPorVendedor && (
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
                          {closeResultData.ventasPorMetodo.ventasPorVendedor.map(
                            (vendedor: any) => (
                              <TableRow key={vendedor.id}>
                                <TableCell className="font-medium">
                                  {vendedor.nombre}
                                </TableCell>
                                <TableCell>{vendedor.email}</TableCell>
                                <TableCell className="text-right">
                                  $
                                  {Number(vendedor.totalVentas).toLocaleString(
                                    "es-AR",
                                    {
                                      minimumFractionDigits: 2,
                                      maximumFractionDigits: 2,
                                    }
                                  )}
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

        {/* Diálogo para mostrar monto redondeado */}
        <Dialog
          open={roundedAmountDialogOpen}
          onOpenChange={(open) => {
            console.log(
              "🔄 Estado del diálogo de confirmación cambiado a:",
              open
            );
            if (!open) {
              console.log("🔄 Cerrando diálogo de confirmación manualmente");

              // Limpiar TODOS los estados relacionados con el pago
              setRoundedAmountDialogOpen(false);
              setPaymentDialogOpen(false);
              setSelectedPaymentMethod(null);
              setIsProcessingPayment(false);
              setOriginalAmount(0);
              setRoundedAmount(0);

              console.log("🔄 Estados limpiados después de cerrar diálogo");

              // Devolver el foco al input de búsqueda cuando se cierra el diálogo
              setTimeout(() => {
                if (searchInputRef.current) {
                  console.log("🔄 Devolviendo foco al input de búsqueda");
                  searchInputRef.current.focus();
                } else {
                  console.log("⚠️ searchInputRef.current es null");
                }
              }, 100);
            }
          }}
        >
          <DialogContent
            className=""
            onKeyDown={(e) => {
              console.log(
                "🔑 Tecla presionada en diálogo de confirmación:",
                e.key
              );
              if (e.key === "Enter" && !isProcessingPayment) {
                e.preventDefault();
                console.log("🔄 ENTER detectado - Confirmando pago");
                confirmRoundedPayment();
              }
              if (e.key === "Escape") {
                console.log(
                  "🔄 ESC detectado - Cerrando diálogo de confirmación"
                );
                setRoundedAmountDialogOpen(false);
                setSelectedPaymentMethod(null);
                setIsProcessingPayment(false);
                setApplyingDiscount(false);
              }
              // Agregar manejo de F5 para aplicar descuento
              if (
                e.key === "F5" &&
                !applyingDiscount &&
                businessInfo?.descuentoEfectivo &&
                !isProcessingPayment
              ) {
                e.preventDefault();
                console.log("🔄 F5 detectado - Aplicando descuento");
                // Cerrar el diálogo actual
                setRoundedAmountDialogOpen(false);
                // Pequeña pausa para asegurar que se cierre primero
                setTimeout(() => {
                  // Volver a abrir con descuento
                  handleCashPayment(true);
                }, 100);
              }
            }}
          >
            <DialogHeader>
              <DialogTitle className="text-2xl">
                {applyingDiscount
                  ? "Efectivo con descuento"
                  : businessInfo?.sistemaPago === "redondeo"
                  ? "Redondeo de pago en efectivo"
                  : "Confirmar pago en efectivo"}
              </DialogTitle>
              <DialogDescription className="text-lg">
                {applyingDiscount
                  ? `Se aplicará un descuento del ${businessInfo?.descuentoEfectivo}% por pago en efectivo.`
                  : businessInfo?.sistemaPago === "redondeo"
                  ? "El sistema de redondeo ha ajustado el monto para facilitar el pago en efectivo."
                  : "Por favor confirma el pago en efectivo."}
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-4">
              {applyingDiscount ||
              (businessInfo?.sistemaPago === "redondeo" &&
                originalAmount !== roundedAmount) ? (
                <>
                  <div className="flex justify-between items-center">
                    <span className="text-base text-muted-foreground">
                      Monto original:
                    </span>
                    <span className="text-xl">
                      ${originalAmount.toLocaleString()}
                    </span>
                  </div>

                  {applyingDiscount && (
                    <div className="flex justify-between items-center">
                      <span className="text-base text-muted-foreground">
                        Con descuento ({businessInfo?.descuentoEfectivo}%):
                      </span>
                      <span className="text-xl text-blue-600">
                        $
                        {(
                          originalAmount -
                          (originalAmount *
                            Number(businessInfo?.descuentoEfectivo)) /
                            100
                        ).toLocaleString()}
                      </span>
                    </div>
                  )}

                  <div className="flex justify-between items-center">
                    <span className="text-lg font-medium">Monto a cobrar:</span>
                    <span className="text-3xl font-bold text-emerald-600">
                      ${roundedAmount.toLocaleString()}
                    </span>
                  </div>

                  <div className="flex justify-between items-center">
                    <span className="text-base text-muted-foreground">
                      {applyingDiscount
                        ? "Ahorro total:"
                        : "Descuento del redondeo:"}
                    </span>
                    <span className="text-lg text-emerald-700">
                      ${(originalAmount - roundedAmount).toLocaleString()}
                      {applyingDiscount &&
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
                    ${originalAmount.toLocaleString()}
                  </span>
                </div>
              )}
            </div>

            <DialogFooter className="flex space-x-2 justify-end">
              <Button
                variant="outline"
                onClick={() => {
                  setRoundedAmountDialogOpen(false);
                  setSelectedPaymentMethod(null);
                  setIsProcessingPayment(false);
                  setApplyingDiscount(false);
                }}
                tabIndex={3}
                className="text-base py-5 px-4"
              >
                Cancelar
              </Button>

              {!applyingDiscount && businessInfo?.descuentoEfectivo && (
                <Button
                  variant="outline"
                  className="bg-blue-50 border-blue-200 hover:bg-blue-100 hover:text-blue-700 text-base py-5 px-4"
                  onClick={() => {
                    // Cerrar el diálogo actual
                    setRoundedAmountDialogOpen(false);
                    // Pequeña pausa para asegurar que se cierre primero
                    setTimeout(() => {
                      // Volver a abrir con descuento
                      handleCashPayment(true);
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
                disabled={isProcessingPayment}
                autoFocus
                tabIndex={1}
              >
                {isProcessingPayment ? (
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

        <Dialog
          open={deleteScreenDialogOpen}
          onOpenChange={setDeleteScreenDialogOpen}
        >
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>¿Eliminar pantalla?</DialogTitle>
              <DialogDescription>
                ¿Estás seguro de que deseas eliminar esta pantalla? Se perderán
                todos los productos en ella.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter className="flex space-x-2 justify-end">
              <Button
                variant="outline"
                onClick={() => setDeleteScreenDialogOpen(false)}
              >
                Cancelar
              </Button>
              <Button variant="destructive" onClick={confirmDeleteScreen}>
                Eliminar
              </Button>
            </DialogFooter>
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
              TOTAL: ${calculateTotal().toLocaleString()}
            </div>
          </div>
        </div>

        <div className="text-sm text-muted-foreground mt-2">
          Atajos: ↑↓ para navegar, Enter para seleccionar, F2 para pagar, F1
          para cancelar, F4 para cerrar sesión
        </div>

        {/* Diálogo de órdenes recientes */}
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

        {/* Diálogo para mostrar QR de Mercado Pago */}
        <Dialog
          open={qrDialogOpen}
          onOpenChange={(open) => {
            if (!open) {
              cancelQRPayment();
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
              {qrData ? (
                <div className="space-y-4 text-center">
                  <img
                    src={qrData.qrImageUrl}
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
                    {qrData.isSplitPayment ? (
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
                            ${Number(qrData.cashAmount || 0).toLocaleString()}
                          </span>
                        </div>
                        <div className="flex justify-between items-center font-semibold">
                          <span className="flex items-center">
                            <QrCode className="h-4 w-4 mr-1" />
                            QR:
                          </span>
                          <span>
                            ${Number(qrData.monto || 0).toLocaleString()}
                          </span>
                        </div>
                      </div>
                    ) : (
                      <span>
                        Monto a pagar: $
                        {Number(qrData.monto || 0).toLocaleString()}
                      </span>
                    )}
                  </div>

                  <div className="space-y-2">
                    <div
                      className={`text-center py-2 px-4 rounded-full font-medium ${
                        paymentStatus === "PENDIENTE"
                          ? "bg-yellow-100 text-yellow-800"
                          : paymentStatus === "COMPLETADA"
                          ? "bg-green-100 text-green-800"
                          : paymentStatus === "CANCELADA"
                          ? "bg-red-100 text-red-800"
                          : "bg-gray-100 text-gray-800"
                      }`}
                    >
                      Estado:{" "}
                      {paymentStatus === "PENDIENTE"
                        ? "Esperando pago..."
                        : paymentStatus === "COMPLETADA"
                        ? "¡Pago completado!"
                        : paymentStatus === "CANCELADA"
                        ? "Pago cancelado"
                        : "Desconocido"}
                      {paymentStatus === "PENDIENTE" && (
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
                onClick={cancelQRPayment}
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
              setSelectedPaymentMethod(null);
              setIsProcessingPayment(false);
              setCashAmount("");
              setSecondPaymentMethod("tarjeta");

              // Devolver el foco al input de búsqueda
              setTimeout(() => {
                searchInputRef.current?.focus();
              }, 100);
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
                const isButtonEnabled = !(
                  isProcessingPayment ||
                  !cashAmount ||
                  isNaN(parseFloat(cashAmount)) ||
                  parseFloat(cashAmount) <= 0 ||
                  parseFloat(cashAmount) >= calculateTotal()
                );

                // Solo disparar la acción si el botón estaría habilitado
                if (isButtonEnabled) {
                  console.log(
                    "⌨️ TECLADO: Enter detectado en diálogo de pago mixto"
                  );
                  // Ejecutar la función según el método de pago seleccionado
                  if (secondPaymentMethod === "qr") {
                    // Para QR, generar QR con el monto restante
                    const cashAmountValue = parseFloat(cashAmount);
                    const totalAmount = calculateTotal();
                    const secondAmount = parseFloat(
                      (totalAmount - cashAmountValue).toFixed(2)
                    );
                    generateSplitQRPayment(cashAmountValue, secondAmount);
                  } else {
                    // Para tarjeta, procesar el pago mixto directamente
                    processSplitPayment();
                  }
                } else {
                  console.log(
                    "⌨️ TECLADO: Enter detectado pero el botón está deshabilitado"
                  );
                }
              }
            }}
          >
            <DialogHeader>
              <DialogTitle className="text-xl">Pago mixto</DialogTitle>
              <DialogDescription>
                Ingresa el monto a pagar con efectivo y selecciona el método
                para el resto
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-6 py-4">
              <div className="text-center">
                <div className="text-lg text-muted-foreground">
                  Total a pagar
                </div>
                <div className="text-3xl font-bold text-emerald-600 mt-1">
                  ${calculateTotal().toLocaleString()}
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
                      value={cashAmount}
                      onChange={(e) => setCashAmount(e.target.value)}
                      className="w-40 text-right"
                      step="0.01"
                      min="0"
                      max={calculateTotal().toString()}
                      disabled={isProcessingPayment}
                      autoFocus
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Segundo método</label>
                  <div className="grid grid-cols-2 gap-3">
                    <Button
                      variant={
                        secondPaymentMethod === "tarjeta"
                          ? "default"
                          : "outline"
                      }
                      className={
                        secondPaymentMethod === "tarjeta"
                          ? "bg-emerald-600 hover:bg-emerald-700"
                          : ""
                      }
                      onClick={() => setSecondPaymentMethod("tarjeta")}
                      disabled={isProcessingPayment}
                    >
                      <CreditCard className="mr-2 h-4 w-4" />
                      Tarjeta
                    </Button>
                    <Button
                      variant={
                        secondPaymentMethod === "qr" ? "default" : "outline"
                      }
                      className={
                        secondPaymentMethod === "qr"
                          ? "bg-emerald-600 hover:bg-emerald-700"
                          : ""
                      }
                      onClick={() => setSecondPaymentMethod("qr")}
                      disabled={isProcessingPayment}
                    >
                      <QrCode className="mr-2 h-4 w-4" />
                      Transferencia
                    </Button>
                  </div>
                </div>

                <div className="flex justify-between pt-2">
                  <div className="flex items-center">
                    <div className="text-sm font-medium">
                      {secondPaymentMethod === "tarjeta" ? (
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
                    {!cashAmount || isNaN(parseFloat(cashAmount))
                      ? calculateTotal().toLocaleString()
                      : (
                          calculateTotal() -
                          Math.min(parseFloat(cashAmount), calculateTotal())
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
                  setSelectedPaymentMethod(null);
                }}
                disabled={isProcessingPayment}
              >
                Cancelar
              </Button>
              <Button
                onClick={processSplitPayment}
                disabled={
                  isProcessingPayment ||
                  !cashAmount ||
                  isNaN(parseFloat(cashAmount)) ||
                  parseFloat(cashAmount) <= 0 ||
                  parseFloat(cashAmount) >= calculateTotal()
                }
                className="bg-emerald-gradient"
              >
                {isProcessingPayment ? (
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
    </div>
  );
}
