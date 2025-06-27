Lógica de Búsqueda Propuesta
El sistema que te propongo funciona con el siguiente orden de prioridad:

1. Búsqueda por Código de Barra Directo

Busca primero si el código escaneado coincide exactamente con algún código de barra registrado
Para productos con código de barra tradicional (como productos empaquetados)

2. Detección de Formato PLU+PESO

Si tiene 13 dígitos y el PLU (primeros 4) existe en la base de datos como producto pesable
Extrae: PLU (4 dígitos) + Peso (8 dígitos) + Decimal (1 dígito)
Calcula automáticamente el precio basado en peso × precio por kg

3. Búsqueda por PLU Simple

Si no es código de barra ni PLU+PESO, busca como PLU directo
Útil para ingreso manual de productos

Características Clave
Flexibilidad: Maneja ambos tipos de códigos automáticamente
Validación: Verifica que el PLU exista antes de procesar como PLU+PESO
Cálculo automático: Calcula precio total para productos pesables
Extensible: Fácil de adaptar a diferentes formatos de código
Implementación Recomendada

En tu base de datos, asegúrate de tener el campo tipo_venta ('peso' o 'unidad')
En el scanner, usa esta función como intermediaria entre el escáner y tu sistema de ventas
Para debugging, el sistema devuelve el tipo de búsqueda utilizada

CODIGO DE EJEMPLO TYPESCRIPT:

```typescript
// Interfaces para tipado
interface Producto {
  plu: string;
  codigoBarra?: string; // opcional
  nombre: string;
  precioKg?: number;
  precioUnidad?: number;
  tipoVenta: "peso" | "unidad";
}

interface ResultadoBusqueda {
  producto: Producto;
  peso?: number;
  precioCalculado: number;
  tipoBusqueda: "codigo_barra_directo" | "plu_peso" | "plu_directo";
}

interface ValidacionCodigo {
  esValido: boolean;
  mensaje: string;
}

class BarcodeSearchSystem {
  private productosDB: Producto[];

  constructor(productosDB: Producto[]) {
    this.productosDB = productosDB;
  }

  /**
   * Función principal que determina el tipo de código y busca el producto
   */
  public buscarProducto(codigoEscaneado: string): ResultadoBusqueda | null {
    const codigo = codigoEscaneado.trim();

    // 1. Buscar primero como código de barra directo
    const productoDirecto = this.buscarCodigoBarraDirecto(codigo);
    if (productoDirecto) {
      return {
        producto: productoDirecto,
        peso: undefined,
        precioCalculado: this.calcularPrecioBase(productoDirecto),
        tipoBusqueda: "codigo_barra_directo",
      };
    }

    // 2. Si no encuentra, verificar si es código PLU+PESO
    if (this.esCodigoPluPeso(codigo)) {
      return this.procesarCodigoPluPeso(codigo);
    }

    // 3. Si no es ninguno, buscar por PLU simple
    const productoPlu = this.buscarPorPlu(codigo);
    if (productoPlu) {
      return {
        producto: productoPlu,
        peso: undefined,
        precioCalculado: this.calcularPrecioBase(productoPlu),
        tipoBusqueda: "plu_directo",
      };
    }

    return null;
  }

  /**
   * Busca producto por código de barra exacto
   */
  private buscarCodigoBarraDirecto(codigo: string): Producto | null {
    return (
      this.productosDB.find((producto) => producto.codigoBarra === codigo) ||
      null
    );
  }

  /**
   * Busca producto por PLU
   */
  private buscarPorPlu(plu: string): Producto | null {
    return this.productosDB.find((producto) => producto.plu === plu) || null;
  }

  /**
   * Determina si el código tiene formato PLU+PESO
   * Formato esperado: 4 dígitos PLU + 9 dígitos peso (último dígito es decimal)
   * Total: 13 dígitos
   */
  private esCodigoPluPeso(codigo: string): boolean {
    // Verificar longitud
    if (codigo.length !== 13) {
      return false;
    }

    // Verificar que todos sean dígitos
    if (!/^\d{13}$/.test(codigo)) {
      return false;
    }

    // Extraer PLU y verificar si existe en la base de datos
    const plu = codigo.substring(0, 4);
    const producto = this.buscarPorPlu(plu);

    // Solo es válido como PLU+PESO si el producto existe y es de tipo peso
    return producto !== null && producto.tipoVenta === "peso";
  }

  /**
   * Procesa código formato PLU+PESO
   * Formato: PPPPWWWWWWWWD
   * P = PLU (4 dígitos)
   * W = Peso (8 dígitos enteros)
   * D = Decimal (1 dígito)
   */
  private procesarCodigoPluPeso(codigo: string): ResultadoBusqueda | null {
    const plu = codigo.substring(0, 4);
    const pesoStr = codigo.substring(4, 12); // 8 dígitos del peso
    const decimalStr = codigo.substring(12, 13); // último dígito para decimal

    // Construir peso con decimal
    const peso = parseFloat(`${parseInt(pesoStr)}.${decimalStr}`);

    // Buscar producto por PLU
    const producto = this.buscarPorPlu(plu);

    if (!producto || !producto.precioKg) {
      return null;
    }

    // Calcular precio
    const precioCalculado = this.redondear(peso * producto.precioKg, 2);

    return {
      producto,
      peso,
      precioCalculado,
      tipoBusqueda: "plu_peso",
    };
  }

  /**
   * Calcula precio base del producto
   */
  private calcularPrecioBase(producto: Producto): number {
    if (producto.tipoVenta === "peso") {
      return producto.precioKg || 0;
    }
    return producto.precioUnidad || 0;
  }

  /**
   * Redondea un número a los decimales especificados
   */
  private redondear(numero: number, decimales: number): number {
    return (
      Math.round(numero * Math.pow(10, decimales)) / Math.pow(10, decimales)
    );
  }

  /**
   * Obtiene sugerencias cuando no se encuentra un producto
   */
  public obtenerSugerencias(codigoEscaneado: string): string[] {
    return [
      "Verificar que el producto esté cargado en el sistema",
      "Confirmar formato del código de barras",
      "Revisar configuración de PLU si es producto pesable",
      `Código escaneado: ${codigoEscaneado}`,
    ];
  }
}

/**
 * Valida el código antes de procesarlo
 */
function validarCodigoEntrada(codigo: string): ValidacionCodigo {
  const codigoLimpio = codigo.trim();

  if (!codigoLimpio) {
    return { esValido: false, mensaje: "Código vacío" };
  }

  if (codigoLimpio.length < 4) {
    return { esValido: false, mensaje: "Código muy corto" };
  }

  if (codigoLimpio.length > 15) {
    return { esValido: false, mensaje: "Código muy largo" };
  }

  return { esValido: true, mensaje: "Código válido" };
}

// Ejemplo de uso
function ejemploUso(): void {
  // Base de datos de productos ejemplo
  const productos: Producto[] = [
    {
      plu: "1234",
      codigoBarra: "7798123456789",
      nombre: "Leche Entera",
      precioUnidad: 150.0,
      tipoVenta: "unidad",
    },
    {
      plu: "2001",
      nombre: "Carne Picada",
      precioKg: 2500.0,
      tipoVenta: "peso",
    },
    {
      plu: "3005",
      nombre: "Pan Francés",
      precioKg: 800.0,
      tipoVenta: "peso",
    },
  ];

  const sistema = new BarcodeSearchSystem(productos);

  console.log("=== PRUEBAS DEL SISTEMA ===\n");

  // 1. Código de barra directo
  const resultado1 = sistema.buscarProducto("7798123456789");
  console.log("1. Código directo:", resultado1);

  // 2. Código PLU+PESO (PLU: 2001, Peso: 1.5kg)
  // Formato: 2001 + 00000015 + 0 = 2001000000150
  const resultado2 = sistema.buscarProducto("2001000000150");
  console.log("2. PLU+Peso:", resultado2);

  // 3. PLU directo
  const resultado3 = sistema.buscarProducto("3005");
  console.log("3. PLU directo:", resultado3);

  // 4. Código no encontrado
  const resultado4 = sistema.buscarProducto("9999999999999");
  console.log("4. No encontrado:", resultado4);

  // 5. Validación de código
  const validacion = validarCodigoEntrada("2001000000150");
  console.log("5. Validación:", validacion);
}

// Función helper para uso en React/Angular/Vue
function usarEnComponente() {
  const productos: Producto[] = []; // Tu array de productos
  const sistema = new BarcodeSearchSystem(productos);

  // Función que puedes llamar desde tu componente
  const procesarCodigoEscaneado = (
    codigo: string
  ): ResultadoBusqueda | null => {
    const validacion = validarCodigoEntrada(codigo);

    if (!validacion.esValido) {
      console.error("Código inválido:", validacion.mensaje);
      return null;
    }

    return sistema.buscarProducto(codigo);
  };

  return { procesarCodigoEscaneado };
}

// Exportar para uso en módulos
export {
  BarcodeSearchSystem,
  validarCodigoEntrada,
  usarEnComponente,
  type Producto,
  type ResultadoBusqueda,
  type ValidacionCodigo,
};

// Ejecutar ejemplo si se ejecuta directamente
if (require.main === module) {
  ejemploUso();
}
```
