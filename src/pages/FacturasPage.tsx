import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { FacturaList } from "@/components/facturas/FacturaList";
import { AgregarPagoDialog } from "@/components/facturas/AgregarPagoDialog";
import { HistorialPagosDialog } from "@/components/facturas/HistorialPagosDialog";
import FacturaModal from "@/components/facturas/FacturaModal";
import FacturaForm from "@/components/facturas/FacturaForm";
import { useFacturas } from "@/hooks/useFacturas";
import { Factura, AddPagoData } from "@/types/factura";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";

interface FacturasPageProps {
  // Removido onBack - ahora se maneja con navegación
}

export const FacturasPage: React.FC<FacturasPageProps> = ({}) => {
  const navigate = useNavigate();
  const {
    facturas,
    loading,
    pagination,
    filters,
    fetchFacturas,
    addPago,
    downloadPDF,
    printFactura,
    updateFilters,
    goToPage,
  } = useFacturas();

  const handleBack = () => {
    navigate("/dashboard");
  };

  const [selectedFactura, setSelectedFactura] = useState<Factura | null>(null);
  const [showPagoDialog, setShowPagoDialog] = useState(false);
  const [showHistorialDialog, setShowHistorialDialog] = useState(false);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);

  const handleVer = (factura: Factura) => {
    // Esta función ya no se usa, la navegación se maneja directamente en FacturaCard
  };

  const handleEditar = (factura: Factura) => {
    setSelectedFactura(factura);
    setShowEditModal(true);
  };

  const handleDescargar = async (id: string) => {
    await downloadPDF(id);
  };

  const handleEliminar = async (id: string) => {
    if (confirm("¿Estás seguro de que quieres eliminar esta factura?")) {
      // Implementar eliminación
    }
  };

  const handleVerPagos = (factura: Factura) => {
    setSelectedFactura(factura);
    setShowHistorialDialog(true);
  };

  const handleAgregarPago = (factura: Factura) => {
    setSelectedFactura(factura);
    setShowPagoDialog(true);
  };

  const handleCreateNew = () => {
    setShowCreateForm(true);
  };

  const handleFormClose = () => {
    setShowCreateForm(false);
  };

  const handleFormSuccess = () => {
    setShowCreateForm(false);
    fetchFacturas();
  };

  const handleAddPagoSubmit = async (data: AddPagoData): Promise<boolean> => {
    return await addPago(data);
  };

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Header */}
      <header className="p-6 border-b border-gray-200 bg-emerald-gradient">
        <div className="flex items-center gap-4 justify-between">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleBack}
              className="text-white hover:text-gray-900 hover:bg-gray-100"
            >
              <ArrowLeft className="w-4 h-4 mr-2" />
              Volver
            </Button>
            <h1 className="text-3xl font-semibold text-white">
              Cuenta Corriente
            </h1>
          </div>
          <div className="flex items-center gap-2">
            <img
              src="/andexmarket-logo.png"
              alt="logo"
              className="w-40 h-auto"
            />
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 p-6 overflow-auto bg-white">
        <FacturaList
          facturas={facturas}
          loading={loading}
          pagination={pagination}
          filters={filters}
          onVer={handleVer}
          onEditar={handleEditar}
          onDescargar={handleDescargar}
          onEliminar={handleEliminar}
          onVerPagos={handleVerPagos}
          onAgregarPago={handleAgregarPago}
          onCreateNew={handleCreateNew}
          onUpdateFilters={updateFilters}
          onGoToPage={goToPage}
        />
      </main>

      {/* Dialogs */}
      {selectedFactura && showPagoDialog && (
        <AgregarPagoDialog
          factura={selectedFactura}
          open={showPagoDialog}
          onOpenChange={setShowPagoDialog}
          onSuccess={() => {
            setShowPagoDialog(false);
            fetchFacturas();
          }}
          onSubmit={handleAddPagoSubmit}
        />
      )}

      {selectedFactura && showHistorialDialog && (
        <HistorialPagosDialog
          factura={selectedFactura}
          open={showHistorialDialog}
          onOpenChange={setShowHistorialDialog}
        />
      )}

      {/* Modales de facturas */}
      {showCreateForm && (
        <FacturaForm
          mode="create"
          onClose={handleFormClose}
          onSuccess={handleFormSuccess}
        />
      )}

      {showEditModal && selectedFactura && (
        <FacturaModal
          factura={selectedFactura}
          mode="edit"
          onClose={() => setShowEditModal(false)}
          onUpdate={() => {
            setShowEditModal(false);
            setSelectedFactura(null);
            fetchFacturas();
          }}
        />
      )}
    </div>
  );
};
