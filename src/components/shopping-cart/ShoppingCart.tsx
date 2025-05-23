import { useState } from "react";
import { useAuth } from "@/contexts/AuthContext";
import { useProductSearch } from "./hooks";
import { ProductSearch } from "./product-list";
import { PaymentDialog, SplitPaymentDialog } from "./dialogs";
import {
  CartProvider,
  PaymentProvider,
  BusinessProvider,
  WeightProvider,
} from "./contexts";
import {
  CartHeader,
  CartFooter,
  ProductQuantityInput,
  QRPayment,
} from "./components";
import { CartList } from "./product-list";
import { AvailableProduct } from "./types";

export default function ShoppingCart() {
  const { user } = useAuth();
  const API_URL = import.meta.env.VITE_API_URL;
  const appId =
    window.electron?.process?.argv
      ?.find((arg) => arg.startsWith("--app-id="))
      ?.split("=")[1] || null;
  const headers = {
    "Content-Type": "application/json",
    ...(appId && { "X-App-ID": appId }),
  };

  // Estados
  const [paymentDialogOpen, setPaymentDialogOpen] = useState(false);
  const [splitPaymentDialogOpen, setSplitPaymentDialogOpen] = useState(false);
  const [qrDialogOpen, setQrDialogOpen] = useState(false);
  const [cashAmount, setCashAmount] = useState<string>("");
  const [secondPaymentMethod, setSecondPaymentMethod] =
    useState<string>("tarjeta");

  // Hooks personalizados
  const productSearch = useProductSearch({ API_URL, headers });

  // Manejador de producto seleccionado
  const handleProductSelect = (product: AvailableProduct) => {
    productSearch.handleProductSelect(product);
  };

  return (
    <BusinessProvider API_URL={API_URL} headers={headers}>
      <WeightProvider>
        <CartProvider>
          <PaymentProvider API_URL={API_URL} headers={headers}>
            <div className="flex flex-col h-full">
              <div className="flex-none p-4">
                <div className="flex flex-col gap-4">
                  <CartHeader />
                  <ProductSearch
                    searchQuery={productSearch.searchQuery}
                    onSearchChange={productSearch.handleSearchChange}
                    searchResults={productSearch.searchResults}
                    selectedIndex={productSearch.selectedIndex}
                    onProductSelect={handleProductSelect}
                    showResults={productSearch.showResults}
                  />
                  {productSearch.selectedProduct && (
                    <ProductQuantityInput
                      product={productSearch.selectedProduct}
                      useManualWeight={false}
                      onAdd={() => {
                        productSearch.setSelectedProduct(null);
                      }}
                    />
                  )}
                </div>
              </div>

              <CartList />

              <CartFooter
                onPaymentClick={() => setPaymentDialogOpen(true)}
                onSplitPaymentClick={() => setSplitPaymentDialogOpen(true)}
              />

              <PaymentDialog
                open={paymentDialogOpen}
                onClose={() => setPaymentDialogOpen(false)}
              />

              <SplitPaymentDialog
                open={splitPaymentDialogOpen}
                onClose={() => setSplitPaymentDialogOpen(false)}
                cashAmount={cashAmount}
                onCashAmountChange={setCashAmount}
                secondPaymentMethod={secondPaymentMethod}
                onSecondPaymentMethodChange={setSecondPaymentMethod}
              />

              <QRPayment
                open={qrDialogOpen}
                onClose={() => setQrDialogOpen(false)}
                isSplitPayment={secondPaymentMethod === "qr"}
                cashAmount={Number(cashAmount)}
              />
            </div>
          </PaymentProvider>
        </CartProvider>
      </WeightProvider>
    </BusinessProvider>
  );
}
