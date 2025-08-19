/**
 * Calcula el precio visual que debe mostrarse al usuario según las configuraciones de IVA
 *
 * @param precio - Precio base del producto
 * @param ivaIncluido - Si el producto ya tiene IVA incluido
 * @param ivaPorcentaje - Porcentaje de IVA específico del producto
 * @param ivaIncluidoEnPrecios - Configuración del business sobre si los precios incluyen IVA
 * @returns El precio que debe mostrarse visualmente
 */
export function calcularPrecioVisualConIVA(
  precio: number,
  ivaIncluido: boolean,
  ivaPorcentaje: number | null,
  ivaIncluidoEnPrecios: boolean
): number {
  // Si el business tiene ivaIncluidoEnPrecios en true, siempre mostrar el precio tal como viene
  if (ivaIncluidoEnPrecios) {
    return precio;
  }

  // Si el producto ya tiene IVA incluido, mostrar el precio tal como viene
  if (ivaIncluido) {
    return precio;
  }

  // Si no hay porcentaje de IVA definido, usar 21% por defecto (como en el backend)
  const porcentajeIva = ivaPorcentaje ?? 21;

  // Si el business NO incluye IVA en precios Y el producto NO tiene IVA incluido,
  // entonces calcular y mostrar el precio con IVA agregado
  const ivaAdicional = precio * (porcentajeIva / 100);
  return precio + ivaAdicional;
}

/**
 * Formatea un precio con IVA para mostrar en la interfaz
 *
 * @param precio - Precio base del producto
 * @param ivaIncluido - Si el producto ya tiene IVA incluido
 * @param ivaPorcentaje - Porcentaje de IVA específico del producto
 * @param ivaIncluidoEnPrecios - Configuración del business sobre si los precios incluyen IVA
 * @param decimales - Número de decimales a mostrar (default: 2)
 * @returns String formateado del precio
 */
export function formatearPrecioConIVA(
  precio: number,
  ivaIncluido: boolean,
  ivaPorcentaje: number | null,
  ivaIncluidoEnPrecios: boolean,
  decimales: number = 2
): string {
  const precioVisual = calcularPrecioVisualConIVA(
    precio,
    ivaIncluido,
    ivaPorcentaje,
    ivaIncluidoEnPrecios
  );

  return precioVisual.toFixed(decimales);
}

/**
 * Formatea un precio con separadores de miles y el símbolo de moneda
 *
 * @param precio - Precio base del producto
 * @param ivaIncluido - Si el producto ya tiene IVA incluido
 * @param ivaPorcentaje - Porcentaje de IVA específico del producto
 * @param ivaIncluidoEnPrecios - Configuración del business sobre si los precios incluyen IVA
 * @returns String formateado con separadores de miles y símbolo $
 */
export function formatearPrecioConIVAYMoneda(
  precio: number,
  ivaIncluido: boolean,
  ivaPorcentaje: number | null,
  ivaIncluidoEnPrecios: boolean
): string {
  const precioVisual = calcularPrecioVisualConIVA(
    precio,
    ivaIncluido,
    ivaPorcentaje,
    ivaIncluidoEnPrecios
  );

  return `$${precioVisual.toLocaleString()}`;
}
