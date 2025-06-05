import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableHeader,
  TableRow,
  TableHead,
  TableBody,
  TableCell,
} from "@/components/ui/table";

interface CloseResultDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  result: {
    periodo: string;
    fechaInicio: string;
    fechaCierre: string;
    totalVentas: number;
    cantidadVentas: number;
    ventasPorMetodo: Record<string, number>;
    ventasPorVendedor: Array<{
      id: string;
      nombre: string;
      email: string;
      totalVentas: number;
      cantidadVentas: number;
    }>;
  } | null;
  formatDate: (date: string) => string;
}

export function CloseResultDialog({
  open,
  onOpenChange,
  result,
  formatDate,
}: CloseResultDialogProps) {
  if (!result) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Resultado del Cierre de Caja</DialogTitle>
          <DialogDescription>
            Resumen de ventas del período seleccionado
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <h3 className="text-sm font-medium">Período</h3>
              <p className="text-xl font-bold">
                {result.periodo.toUpperCase()}
              </p>
            </div>
            <div>
              <h3 className="text-sm font-medium">Fecha</h3>
              <p>
                {formatDate(result.fechaInicio)} - {formatDate(result.fechaCierre)}
              </p>
            </div>
          </div>

          <div className="rounded-lg border p-4">
            <h3 className="text-lg font-semibold mb-2">Resumen General</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-sm text-muted-foreground">Total Ventas</p>
                <p className="text-2xl font-bold text-emerald-600">
                  ${Number(result.totalVentas).toLocaleString()}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">
                  Cantidad de Ventas
                </p>
                <p className="text-2xl font-bold">{result.cantidadVentas}</p>
              </div>
            </div>
          </div>

          {result.ventasPorMetodo && (
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
                  {Object.entries(result.ventasPorMetodo).map(
                    ([metodo, monto]) => (
                      <TableRow key={metodo}>
                        <TableCell className="font-medium capitalize">
                          {metodo}
                        </TableCell>
                        <TableCell className="text-right">
                          ${Number(monto).toLocaleString()}
                        </TableCell>
                      </TableRow>
                    )
                  )}
                </TableBody>
              </Table>
            </div>
          )}

          {result.ventasPorVendedor && result.ventasPorVendedor.length > 0 && (
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
                  {result.ventasPorVendedor.map((vendedor) => (
                    <TableRow key={vendedor.id}>
                      <TableCell className="font-medium">
                        {vendedor.nombre}
                      </TableCell>
                      <TableCell>{vendedor.email}</TableCell>
                      <TableCell className="text-right">
                        ${Number(vendedor.totalVentas).toLocaleString()}
                      </TableCell>
                      <TableCell className="text-right">
                        {vendedor.cantidadVentas}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>Cerrar</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
