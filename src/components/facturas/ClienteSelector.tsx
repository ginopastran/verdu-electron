import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Search, User, Building, Hash } from "lucide-react";
import { Cliente } from "@/types/factura";

interface ClienteSelectorProps {
  clientes: Cliente[];
  onSelect: (cliente: Cliente) => void;
  onClose: () => void;
}

export const ClienteSelector: React.FC<ClienteSelectorProps> = ({
  clientes,
  onSelect,
  onClose,
}) => {
  const [searchTerm, setSearchTerm] = useState("");

  const filteredClientes = clientes.filter(
    (cliente) =>
      cliente.nombre?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      cliente.razonSocial?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      cliente.cuit?.includes(searchTerm)
  );

  const getClienteDisplayName = (cliente: Cliente) => {
    if (cliente.razonSocial) {
      return cliente.razonSocial;
    }
    return `${cliente.nombre} ${cliente.apellido || ""}`.trim();
  };

  const getCondicionFiscalLabel = (condicion: string) => {
    const labels: Record<string, string> = {
      consumidor_final: "Consumidor Final",
      responsable_inscripto: "Responsable Inscripto",
      monotributista: "Monotributista",
      exento: "Exento",
    };
    return labels[condicion] || condicion.replace("_", " ");
  };

  return (
    <Dialog open={true} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <User className="w-5 h-5" />
            Seleccionar Cliente
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="search-cliente">Buscar cliente</Label>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
              <Input
                id="search-cliente"
                placeholder="Buscar por nombre, razón social o CUIT..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>

          <div className="max-h-64 overflow-y-auto space-y-2">
            {filteredClientes.map((cliente) => (
              <Card
                key={cliente.id}
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => onSelect(cliente)}
              >
                <CardContent className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        {cliente.razonSocial ? (
                          <Building className="w-4 h-4 text-muted-foreground" />
                        ) : (
                          <User className="w-4 h-4 text-muted-foreground" />
                        )}
                        <span className="font-medium">
                          {getClienteDisplayName(cliente)}
                        </span>
                      </div>
                      {cliente.cuit && (
                        <div className="flex items-center gap-1 text-sm text-muted-foreground mb-1">
                          <Hash className="w-3 h-3" />
                          <span>CUIT: {cliente.cuit}</span>
                        </div>
                      )}
                      <Badge variant="secondary" className="text-xs">
                        {getCondicionFiscalLabel(cliente.condicionFiscal)}
                      </Badge>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>

          {filteredClientes.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">
              <User className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p>No se encontraron clientes</p>
              <p className="text-sm">
                {searchTerm
                  ? "Intenta con otros términos de búsqueda"
                  : "No hay clientes registrados"}
              </p>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};
