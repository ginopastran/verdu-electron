import { useState, useEffect } from "react";
import { toast } from "sonner";
import { facturasService } from "@/services/facturas";
import { Cliente, Producto } from "@/types/factura";

export const useClientes = () => {
  const [clientes, setClientes] = useState<Cliente[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchClientes = async () => {
    setLoading(true);
    try {
      const data = await facturasService.getClientes();
      setClientes(data);
    } catch (error) {
      console.error("Error fetching clientes:", error);
      toast.error("Error al cargar los clientes");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchClientes();
  }, []);

  return {
    clientes,
    loading,
    fetchClientes,
  };
};

export const useProductos = () => {
  const [productos, setProductos] = useState<Producto[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchProductos = async () => {
    setLoading(true);
    try {
      const data = await facturasService.getProductos();
      setProductos(data);
    } catch (error) {
      console.error("Error fetching productos:", error);
      toast.error("Error al cargar los productos");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchProductos();
  }, []);

  return {
    productos,
    loading,
    fetchProductos,
  };
};
