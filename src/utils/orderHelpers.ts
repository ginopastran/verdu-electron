export interface OrderItemInput {
  productoId?: number;
  id?: number;
  cantidad?: number;
  quantity?: number;
  subtotal?: number;
  precioHistorico?: number;
  pricePerUnit?: number;
  costo?: number;
  nombre?: string;
  [key: string]: any;
}

export interface OrderPayloadInput {
  items: OrderItemInput[];
  total?: number;
  pagos?: { metodoPago: string; monto: number }[];
  metodoPago?: string;
  descuento?: {
    tipo: 'percentage' | 'fixed';
    valor: number;
    montoDescuento: number;
  };
  [key: string]: any;
}

// Construye un payload válido para el endpoint POST /api/ordenes asegurando que
// cada item tenga productoId, cantidad, subtotal, costo, precioHistorico
// Además ajusta total y pagos si fuese necesario.
export function createValidOrderPayload(payload: OrderPayloadInput) {
  const items = (payload.items || []).map((raw) => {
    const productoId =
      typeof raw.productoId === "number"
        ? raw.productoId
        : typeof raw.id === "number"
        ? raw.id
        : undefined;

    const cantidad =
      typeof raw.cantidad === "number"
        ? raw.cantidad
        : typeof raw.quantity === "number"
        ? raw.quantity
        : 0;

    let precioHistorico =
      typeof raw.precioHistorico === "number"
        ? raw.precioHistorico
        : typeof raw.pricePerUnit === "number"
        ? raw.pricePerUnit
        : undefined;

    let subtotal = typeof raw.subtotal === "number" ? raw.subtotal : undefined;

    // Intentar deducir valores faltantes
    if (precioHistorico === undefined && subtotal !== undefined && cantidad) {
      // price = subtotal / cantidad
      precioHistorico = parseFloat((subtotal / cantidad).toFixed(2));
    }

    if (
      (subtotal === undefined || subtotal === 0) &&
      precioHistorico !== undefined
    ) {
      subtotal = parseFloat((precioHistorico * cantidad).toFixed(2));
    }

    const costo = typeof raw.costo === "number" ? raw.costo : 0;

    // Si aún no tenemos valores válidos, marcar item como inválido y será filtrado
    if (
      productoId === undefined ||
      !cantidad ||
      precioHistorico === undefined ||
      subtotal === undefined
    ) {
      return null; // indicador de item inválido
    }

    const rawLid = raw.listaPrecioId;
    const lidParsed =
      rawLid != null && rawLid !== "" ? Number(rawLid) : NaN;
    const listaPrecioId = Number.isFinite(lidParsed) ? lidParsed : null;

    const out: Record<string, unknown> = {
      productoId,
      cantidad,
      subtotal,
      costo,
      precioHistorico,
      nombre: raw.nombre,
      listaPrecioId,
    };

    if (typeof raw.listaPrecioNombre === "string" && raw.listaPrecioNombre) {
      out.listaPrecioNombre = raw.listaPrecioNombre;
    }

    return out as {
      productoId: number;
      cantidad: number;
      subtotal: number;
      costo: number;
      precioHistorico: number;
      nombre?: string;
      listaPrecioId: number | null;
      listaPrecioNombre?: string;
    };
  });

  // Filtrar items inválidos (null)
  const validItems = items.filter(
    (it): it is Exclude<typeof it, null> => it !== null
  );

  if (validItems.length === 0) {
    throw new Error("No valid items to create order payload");
  }

  const calculatedTotal = parseFloat(
    validItems.reduce((acc, cur) => acc + (cur.subtotal || 0), 0).toFixed(2)
  );

  const total =
    typeof payload.total === "number" ? payload.total : calculatedTotal;

  const tieneDescuento = Boolean(
    payload.tieneDescuento ||
      (payload.montoDescuento != null && Number(payload.montoDescuento) > 0) ||
      payload.discountData
  );

  const finalTotal =
    tieneDescuento && typeof payload.total === "number"
      ? payload.total
      : Math.abs(total - calculatedTotal) < 0.01
        ? calculatedTotal
        : total;

  // Construir arreglo de pagos si falta
  let pagos = payload.pagos;
  if (!pagos || pagos.length === 0) {
    pagos = [
      {
        metodoPago: payload.metodoPago || "efectivo",
        monto: finalTotal,
      },
    ];
  }

  // Asegurar que la suma de montos de pagos cubra el total exacto
  const pagado = pagos.reduce((acc, cur) => acc + cur.monto, 0);
  if (Math.abs(pagado - finalTotal) >= 0.01) {
    // Ajustar el primer pago para que complete
    pagos[0].monto = parseFloat(
      (pagos[0].monto + (finalTotal - pagado)).toFixed(2)
    );
  }

  return {
    ...payload,
    items: validItems,
    total: finalTotal,
    pagos,
    ...(payload.descuento && { descuento: payload.descuento }),
  };
}
