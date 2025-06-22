# 🚀 Integración de Facturación AFIP con Electron

## Descripción General

Este documento explica cómo usar los endpoints de facturación AFIP desde una aplicación Electron para crear facturas B (consumidores finales) utilizando la misma infraestructura del sistema web.

## 📋 Requisitos Previos

1. **Business con facturación habilitada**: El campo `facturacionHabilitada` debe estar en `true`
2. **Configuración AFIP**: Business debe tener configuración AFIP válida
3. **Servidor ejecutándose**: El API del sistema debe estar corriendo (normalmente en puerto 3000)

## 🏗️ Arquitectura de la Solución

### 1. Schema Actualizado

El modelo `Factura` ahora es compatible con `OrdenCompra`:

```prisma
model Factura {
  // Campos compatibles con OrdenCompra
  sucursalId       Int              // Obligatorio
  vendedorId       Int?             // Opcional
  metodoPago       String?          // Método de pago usado
  idReal           Int              // ID único por business

  // Campos específicos de facturación
  numero           String
  tipoFactura      String           @default("B")
  cae              String?
  // ... otros campos AFIP
}
```

### 2. idReal Único por Business

Cada business mantiene contadores separados:

- `ultimoIdRealOrden`: Para órdenes
- `ultimoIdRealFactura`: Para facturas

Esto garantiza IDs únicos y secuenciales por business.

## 🔌 Uso desde Electron

### Ejemplo Completo: Crear Factura B

**IMPORTANTE**: Para facturas B (consumidores finales), los precios de los productos deben estar con IVA incluido. El sistema NO calculará IVA adicional.

```javascript
// En tu aplicación Electron (renderer process)
const { ipcRenderer } = require("electron");

// Función para crear factura B desde Electron
async function crearFacturaB(productos, sucursalId, vendedorId = null) {
  try {
    const facturaData = {
      // Campos requeridos
      sucursalId: sucursalId,
      tipoFactura: "B", // Factura B para consumidores finales
      esConsumidorFinal: true,

      // Campos opcionales
      vendedorId: vendedorId,
      metodoPago: "efectivo", // o "tarjeta", "transferencia", etc.

      // Productos
      productos: productos.map((item) => ({
        productoId: item.id,
        cantidad: item.cantidad,
        precio: item.precio, // o precioUnitario
      })),

      // Opcional
      observaciones: "Venta mostrador",
    };

    const response = await fetch(
      "http://localhost:3000/api/facturas/crear-afip",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Cookie: document.cookie, // Para autenticación
        },
        credentials: "include",
        body: JSON.stringify(facturaData),
      }
    );

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Error al crear factura");
    }

    const result = await response.json();

    console.log("✅ Factura creada exitosamente:", {
      id: result.factura.id,
      numero: result.factura.numero,
      idReal: result.factura.idReal,
      total: result.factura.total,
      cae: result.afip.cae,
    });

    return result;
  } catch (error) {
    console.error("❌ Error creando factura:", error);
    throw error;
  }
}

// Ejemplo de uso
const productosVenta = [
  {
    id: 1,
    cantidad: 2,
    precio: 150.0,
  },
  {
    id: 5,
    cantidad: 1,
    precio: 250.0,
  },
];

// Crear factura
crearFacturaB(productosVenta, 1, 2)
  .then((factura) => {
    console.log("Factura creada:", factura);
    // Aquí puedes imprimir, guardar en BD local, etc.
  })
  .catch((error) => {
    console.error("Error:", error);
  });
```

### Configuración de Electron (main process)

```javascript
// main.js - Configurar CORS y cookies
const { app, BrowserWindow, session } = require("electron");

function createWindow() {
  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      webSecurity: false, // Para permitir requests cross-origin
    },
  });

  // Configurar session para cookies
  session.defaultSession.webRequest.onBeforeSendHeaders((details, callback) => {
    details.requestHeaders["User-Agent"] = "ElectronApp";
    callback({ requestHeaders: details.requestHeaders });
  });

  mainWindow.loadFile("index.html");
}
```

## 📊 Historial Unificado

El historial ahora muestra tanto órdenes como facturas cuando `facturacionHabilitada = true`:

### Acceder al Historial

```javascript
async function obtenerHistorial(filtros = {}) {
  const response = await fetch("/api/business/me");
  const business = await response.json();

  if (business.facturacionHabilitada) {
    console.log("✅ Business con facturación habilitada");
    console.log("📋 El historial mostrará órdenes y facturas");
  } else {
    console.log("📋 El historial solo mostrará órdenes");
  }
}
```

## 🎯 Casos de Uso Específicos

### Caso 1: Cliente que NO factura (órdenes normales)

```javascript
// Usar el endpoint de órdenes existente
const ordenData = {
  sucursalId: 1,
  vendedorId: 2,
  metodoPago: "efectivo",
  total: 500.0,
  items: productos,
};

const response = await fetch("/api/ordenes", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify(ordenData),
});
```

### Caso 2: Cliente que SÍ factura (facturas AFIP)

```javascript
// Usar el endpoint de facturas AFIP
const facturaData = {
  sucursalId: 1,
  vendedorId: 2,
  metodoPago: "efectivo",
  tipoFactura: "B",
  esConsumidorFinal: true,
  productos: productos,
};

const response = await fetch("/api/facturas/crear-afip", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify(facturaData),
});
```

## 🔄 Flujo Recomendado

1. **Verificar configuración del business**:

   ```javascript
   const business = await fetch("/api/business/me").then((r) => r.json());
   const puedeFacturar =
     business.facturacionHabilitada && business.afipHabilitado;
   ```

2. **Mostrar opción al usuario**:

   ```javascript
   if (puedeFacturar) {
     // Mostrar checkbox "¿Generar factura?"
     const generarFactura = confirm("¿Desea generar factura AFIP?");
   }
   ```

3. **Crear transacción según elección**:
   ```javascript
   if (generarFactura) {
     await crearFacturaB(productos, sucursalId, vendedorId);
   } else {
     await crearOrden(productos, sucursalId, vendedorId);
   }
   ```

## 🛠️ Funciones Auxiliares

### Obtener Sucursales

```javascript
async function obtenerSucursales() {
  const response = await fetch("/api/sucursales");
  return await response.json();
}
```

### Obtener Productos

```javascript
async function obtenerProductos() {
  const response = await fetch("/api/productos");
  return await response.json();
}
```

### Validar Autenticación

```javascript
async function validarAuth() {
  try {
    const response = await fetch("/api/business/me");
    return response.ok;
  } catch {
    return false;
  }
}
```

## 📈 Ventajas de esta Implementación

1. **Reutilización de código**: Mismos endpoints para web y Electron
2. **Consistencia**: Mismo formato de datos y validaciones
3. **Mantenibilidad**: Un solo lugar para lógica de facturación
4. **Historial unificado**: Órdenes y facturas en el mismo lugar
5. **idReal único**: Numeración secuencial por business
6. **Flexibilidad**: Cada cliente puede elegir si facturar o no
7. **Precios finales**: Para facturas B, los precios ya incluyen IVA (no se calcula adicional)

## 🚨 Consideraciones Importantes

1. **Autenticación**: Asegúrate de manejar cookies/tokens correctamente
2. **Red**: El servidor debe estar accesible desde Electron
3. **Errores**: Implementa manejo robusto de errores
4. **Performance**: Considera caching local para productos/sucursales
5. **Offline**: Planifica estrategia para trabajar sin conexión
6. **Precios con IVA**: Para facturas B, usa precios finales (con IVA incluido). El sistema no agregará IVA adicional

## 🎉 ¡Listo para usar!

Con estos cambios, puedes:

- ✅ Crear facturas B desde Electron usando los mismos endpoints
- ✅ Mantener historial unificado de órdenes y facturas
- ✅ Usar idReal único por business para mejor organización
- ✅ Conservar flexibilidad entre clientes que facturan y los que no
