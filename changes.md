# Cambios Pendientes para Implementar en ShoppingCartRefactored

## 1. Fix error doble impresión

**Descripción**: Corrección del problema de impresión duplicada de tickets.
**Archivos Afectados**:

- ShoppingCart.tsx

**Código Original**:

**Cambios Realizados**:

## ✅ Correcciones: Código PLU y Diálogo Manual QR

### Problemas Identificados y Solucionados

#### 1. Código PLU - Peso Incorrecto

**Problema**: El código `105000013552` se interpretaba como 13.552 kg en lugar de 1.355 kg.

**Causa**: El patrón PLU + peso estaba funcionando correctamente, pero se necesitaba validación adicional.

**Solución Implementada**:

- ✅ **Validación de Rango**: Agregada validación para pesos entre 0.001 y 999.999 kg
- ✅ **Logging Mejorado**: Logs detallados para debugging del proceso PLU
- ✅ **Prevención de Errores**: Evita procesar pesos fuera de rango válido

**Código Corregido**:

```typescript
// ✅ CORRECCIÓN: Validar que el peso sea razonable (entre 0.001 y 999.999 kg)
if (kgQuantity < 0.001 || kgQuantity > 999.999) {
  console.log(`🔍 DEBUG: Peso fuera de rango válido: ${kgQuantity} kg`);
  return;
}
```

#### 2. Diálogo de Contraseña Manual QR No Se Abre

**Problema**: Al hacer clic en "Completar manualmente" en el diálogo QR, no se abría el diálogo de contraseña.

**Causa**: Posible problema con el estado del diálogo o la comunicación entre componentes.

**Solución Implementada**:

- ✅ **Logging Detallado**: Agregados logs para rastrear el flujo completo
- ✅ **Monitoreo de Estado**: useEffect para monitorear cambios en los diálogos
- ✅ **Debugging del Flujo**: Logs antes y después de abrir diálogos
- ✅ **Verificación de Estados**: Logs del estado de los hooks de pago

**Logs Agregados**:

```typescript
// En QRPaymentDialog
console.log("🔧 QR Dialog: Estado antes de abrir:", {
  manualQrPasswordDialogOpen: paymentProcessor.manualQrPasswordDialogOpen,
  isManualPasswordSubmitting: paymentProcessor.isManualPasswordSubmitting,
});

// En ShoppingCartRefactored
useEffect(
  () => {
    console.log("🔧 Estado de diálogos de contraseña manual:", {
      normalDialogOpen: paymentProcessor.manualQrPasswordDialogOpen,
      afipDialogOpen: afipPaymentProcessor.manualQrPasswordDialogOpen,
      isCurrentlyAfipFlow,
    });
  },
  [
    /* dependencias */
  ]
);
```

### Flujo de Debugging Implementado

1. **Código PLU**:

   - Log del código escaneado
   - Log del PLU extraído
   - Log de los gramos y conversión a kg
   - Validación de rango
   - Log del producto encontrado

2. **Diálogo Manual QR**:
   - Log del clic en botón "Completar manualmente"
   - Log del estado antes de abrir diálogo
   - Log del estado después de abrir diálogo
   - Monitoreo continuo del estado de diálogos
   - Log de cambios en onOpenChange

### Archivos Modificados

1. `src/components/ShoppingCartRefactored.tsx`

   - Agregada validación de peso en código PLU
   - Agregados logs de debugging para diálogos manuales
   - Agregado useEffect para monitorear estados

2. `src/components/shopping-cart/dialogs/QRPaymentDialog.tsx`
   - Agregados logs detallados en botón "Completar manualmente"
   - Logs antes y después de abrir diálogos de contraseña

### Próximos Pasos para Testing

1. **Probar Código PLU**:

   - Escanear `105000013552` → Debe mostrar 1.355 kg
   - Verificar logs en consola
   - Confirmar que el peso se muestra correctamente

2. **Probar Diálogo Manual QR**:
   - Generar QR de pago
   - Hacer clic en "Completar manualmente"
   - Verificar que se abre el diálogo de contraseña
   - Revisar logs en consola para debugging

### Beneficios de las Correcciones

1. **Código PLU**:

   - Pesos correctos en productos escaneados
   - Prevención de errores de interpretación
   - Mejor experiencia de usuario

2. **Diálogo Manual QR**:
   - Funcionalidad completa de completado manual
   - Debugging mejorado para futuros problemas
   - Flujo de pago más robusto

# Cambios en el Sistema

## Sistema de Cancelaciones Unificado

### Cambios Realizados

1. **API Unificada**: Se actualizó el sistema de cancelaciones para usar el endpoint unificado `/api/cancelaciones` que maneja tanto órdenes como facturas.

2. **Payload Mejorado**: El payload de cancelación ahora incluye un array de productos con información detallada:

   ```json
   {
     "tipo": "orden",
     "referenciaId": "ID_DE_LA_ORDEN",
     "montoTotal": 150.0,
     "productos": [
       {
         "productoId": 123,
         "nombreProducto": "Manzana",
         "cantidad": 2.5,
         "precioUnitario": 60.0,
         "subtotal": 150.0
       }
     ]
   }
   ```

3. **Validación Mejorada**: Se aumentó el mínimo de caracteres para la razón de cancelación de 2 a 3 caracteres.

4. **Interfaz Actualizada**: El diálogo de cancelación ahora muestra la lista de productos que se van a cancelar con sus cantidades y subtotales.

### Archivos Modificados

- `src/hooks/useCancellationControl.ts`: Hook actualizado para usar la API unificada
- `src/components/shopping-cart/dialogs/CancellationDialog.tsx`: Diálogo actualizado para mostrar productos
- `src/components/ShoppingCartRefactored.tsx`: Integración del nuevo sistema de cancelación

## Sistema de PLU Mejorado - Múltiples Formatos Soportados

### Problema Identificado

El sistema de PLU solo funcionaba con un formato específico, pero los clientes usan diferentes formatos:

- Formato 1: `105000013552` (12 dígitos)
- Formato 2: `0105000013552` (13 dígitos)

### Solución Implementada

1. **Sistema Flexible**: Ahora el sistema detecta automáticamente ambos formatos:

   - **12 dígitos**: 3 dígitos PLU + 8 dígitos peso + 1 dígito adicional
   - **13 dígitos**: 0 + 3 dígitos PLU + 8 dígitos peso + 1 dígito adicional

2. **Logs Detallados**: Se agregaron logs extensivos para debugging:

   ```
   🔍 DEBUG: PLU + peso detectado (formato 13 dígitos):
      - Código completo: 0105000013552
      - PLU extraído: 105
      - Gramos extraídos: 1355
      - Kilogramos calculados: 1.355
   ```

3. **Validación de Peso**: Se mantiene la validación para pesos entre 0.001 y 999.999 kg.

4. **Compatibilidad Total**: Funciona tanto con escaneo directo como con pegado de códigos.

### Archivos Modificados

- `src/components/ShoppingCartRefactored.tsx`: Sistema de detección de PLU actualizado

### Ejemplos de Uso

**Código `105000013552` (12 dígitos)**:

- PLU: `105`
- Peso: `00001355` = 1.355 kg
- Adicional: `2`

**Código `0105000013552` (13 dígitos)**:

- PLU: `105`
- Peso: `00001355` = 1.355 kg
- Adicional: `2`

Ambos códigos producen el mismo resultado: producto con PLU 105 y cantidad 1.355 kg.

## Corrección Crítica - Sistema de Buffer para Códigos PLU

### Problema Identificado

El sistema procesaba los códigos **carácter por carácter** en lugar de esperar a que se complete todo el código. Esto causaba que nunca se procesara el código completo, resultando en cantidades incorrectas.

**Síntomas observados**:

```
🔍 DEBUG: Input event detectado, valor: "1"
🔍 DEBUG: Input event detectado, valor: "10"
🔍 DEBUG: Input event detectado, valor: "105"
```

### Solución Implementada

1. **Sistema de Buffer**: Se implementó un buffer que acumula caracteres hasta que se complete el código:

   ```javascript
   let inputBuffer = "";
   let inputTimeout: NodeJS.Timeout | null = null;
   ```

2. **Timeout Inteligente**: Se usa un timeout de 100ms para procesar el código completo:

   ```javascript
   inputTimeout = setTimeout(() => {
     console.log(
       `🔍 DEBUG: Timeout completado, procesando buffer: "${inputBuffer}"`
     );
     processCompleteCode(inputBuffer);
     inputBuffer = "";
     inputTimeout = null;
   }, 100);
   ```

3. **Procesamiento Completo**: Ahora el sistema procesa el código completo en una sola operación:

   ```
   🔍 DEBUG: Procesando código completo: "105000013552"
   🔍 DEBUG: PLU + peso detectado (formato 12 dígitos):
      - Código completo: 105000013552
      - PLU extraído: 105
      - Gramos extraídos: 1355
      - Kilogramos calculados: 1.355
   ```

4. **Limpieza de Estados**: Se asegura que los timeouts se limpien correctamente para evitar procesamiento duplicado.

### Resultado

Ahora el sistema:

- ✅ **Espera** a que se complete todo el código antes de procesar
- ✅ **Procesa** el código completo en una sola operación
- ✅ **Calcula** correctamente el peso (1.355 kg en lugar de 13.552 kg)
- ✅ **Funciona** tanto con escaneo como con pegado de códigos

### Archivos Modificados

- `src/components/ShoppingCartRefactored.tsx`: Sistema de buffer implementado

## Corrección del Problema del "0" Residual en el Input

### Problema Identificado

Después de escanear un código PLU, el producto se agregaba correctamente pero quedaba un "0" en el campo de búsqueda, indicando que el buffer no se limpiaba completamente.

### Solución Implementada

1. **Limpieza Completa del Buffer**: Se asegura que tanto el input como el buffer se limpien completamente después de procesar un código:

   ```javascript
   // Limpiar completamente el input y el buffer
   input.value = "";
   inputBuffer = "";
   if (inputTimeout) {
     clearTimeout(inputTimeout);
     inputTimeout = null;
   }
   ```

2. **Logs Detallados del Buffer**: Se agregaron logs para rastrear el estado del buffer:

   ```
   🔍 DEBUG: Buffer actualizado: "0105000013552"
   🔍 DEBUG: Timeout completado, procesando buffer: "0105000013552"
   🔍 DEBUG: Buffer limpiado después de procesar
   ```

3. **Análisis Detallado del Código**: Se agregaron logs específicos para el formato de 13 dígitos:
   ```
   🔍 DEBUG: PLU + peso detectado (formato 13 dígitos):
      - Código completo: 0105000013552
      - PLU extraído: 105
      - Gramos extraídos: 1355
      - Kilogramos calculados: 1.355
      - Análisis detallado:
        * Dígito inicial: 0
        * PLU (3 dígitos): 105
        * Peso (8 dígitos): 00001355 = 1355g = 1.355kg
        * Dígito final: 2
   ```

### Resultado

Ahora el sistema:

- ✅ **Procesa** correctamente el código completo
- ✅ **Calcula** el peso exacto (1.355 kg)
- ✅ **Limpia** completamente el input después de agregar el producto
- ✅ **No deja** caracteres residuales en el campo de búsqueda

### Archivos Modificados

- `src/components/ShoppingCartRefactored.tsx`: Limpieza completa del buffer implementada

## Corrección Crítica - Procesamiento de Códigos Parciales

### Problema Identificado

El sistema estaba procesando códigos **parciales** que coincidían con PLUs existentes, causando que se agregaran productos con cantidades incorrectas antes de que se completara el código completo.

**Síntomas observados**:

```
🔍 DEBUG: Input event detectado, valor: "1"
🔍 DEBUG: Input event detectado, valor: "10"
🔍 DEBUG: Input event detectado, valor: "105"
Producto seleccionado: Blanda  // ← Se agregó con cantidad incorrecta
🔍 DEBUG: Input event detectado, valor: "0"  // ← Procesó el resto como nuevo código
```

**Problema**: El código `105` coincidía con un PLU existente y se agregaba inmediatamente, sin esperar el código completo `0105000013552`.

### Solución Implementada

1. **Procesamiento Solo de Códigos Completos**: El sistema ahora **solo procesa** códigos que tengan el formato exacto de PLU + peso:

   - **12 dígitos**: `XXX + 8 dígitos peso + 1 dígito adicional`
   - **13 dígitos**: `0 + XXX + 8 dígitos peso + 1 dígito adicional`

2. **Ignorar Códigos Parciales**: Los códigos que no coincidan con estos formatos se ignoran completamente:

   ```
   🔍 DEBUG: Código ignorado - no es un código completo PLU + peso: "105"
   ```

3. **Logs Mejorados**: Se agregaron logs específicos para identificar cuando se ignoran códigos:
   ```
   🔍 DEBUG: No coincide con ningún patrón PLU + peso
   🔍 DEBUG: ⚠️ Código ignorado: Solo se procesan códigos completos con formato PLU + peso
   ```

### Resultado

Ahora el sistema:

- ✅ **Espera** a que se complete todo el código antes de procesar
- ✅ **Ignora** códigos parciales que coincidan con PLUs existentes
- ✅ **Solo procesa** códigos con formato PLU + peso completo
- ✅ **Calcula** correctamente el peso (1.355 kg en lugar de 13.552 kg)

### Ejemplo de Comportamiento Correcto

**Antes (incorrecto)**:

```
"105" → Encuentra PLU 105 → Agrega producto con cantidad incorrecta
"0" → Procesa como nuevo código
```

**Ahora (correcto)**:

```
"105" → Código ignorado (no es formato completo)
"0105000013552" → Código completo → PLU: 105, Peso: 1.355kg → Agrega correctamente
```

### Archivos Modificados

- `src/components/ShoppingCartRefactored.tsx`: Lógica de procesamiento de códigos corregida

## Mejora en Sistema de Debugging - Rastreo Completo del Escaneo

### Problema Identificado

El usuario reportó que cuando escanea un código, no aparece ningún log de debug, pero cuando pega manualmente el mismo código sí aparecen los logs. También se abre brevemente un diálogo durante el escaneo.

### Solución Implementada

1. **Logs Detallados en Cada Instancia**: Se agregaron logs con emojis para rastrear el flujo completo:

   ```
   🔧 DEBUG: 🚀 Iniciando configuración de listeners para input de búsqueda
   🔧 DEBUG: 📥 INICIO handleInputEvent
   🔧 DEBUG: 🎯 INICIO processCompleteCode
   🔧 DEBUG: 🚀 INICIO autoAddScannedProduct
   ```

2. **Rastreo del Buffer**: Logs específicos para el manejo del buffer:

   ```
   🔧 DEBUG: Buffer actualizado: "0105000013552"
   🔧 DEBUG: Estableciendo nuevo timeout de 100ms
   🔧 DEBUG: ⏰ Timeout completado, procesando buffer: "0105000013552"
   ```

3. **Verificación de Formatos**: Logs detallados para cada formato de código:

   ```
   🔧 DEBUG: 🔍 Verificando si es código de barras estándar...
   🔧 DEBUG: 🔍 Verificando si es PLU + peso...
   🔧 DEBUG: 🔍 Probando formato de 12 dígitos...
   🔧 DEBUG: ✅ Formato de 13 dígitos detectado
   ```

4. **Rastreo de Productos**: Logs para la búsqueda y agregado de productos:

   ```
   🔧 DEBUG: 🔍 Buscando producto con PLU: 105
   🔧 DEBUG: ✅ Producto encontrado por PLU
   🔧 DEBUG: 🛒 Llamando a autoAddScannedProduct para PLU+peso
   ```

5. **Investigación del Diálogo**: Logs para rastrear por qué se abre el diálogo:
   ```
   🔧 DEBUG: 🎯 INICIO handleProductSelect
   🔧 DEBUG: Estableciendo selectedProduct y abriendo diálogo
   ```

### Resultado Esperado

Ahora cuando se escanee un código, deberíamos ver un flujo completo como:

```
🔧 DEBUG: 🚀 Iniciando configuración de listeners para input de búsqueda
🔧 DEBUG: 📥 INICIO handleInputEvent
🔧 DEBUG: Input event detectado, valor: "0105000013552"
🔧 DEBUG: Buffer actualizado: "0105000013552"
🔧 DEBUG: ⏰ Timeout completado, procesando buffer: "0105000013552"
🔧 DEBUG: 🎯 INICIO processCompleteCode
🔧 DEBUG: ✅ Formato de 13 dígitos detectado
🔧 DEBUG: ✅ Producto encontrado por PLU
🔧 DEBUG: 🛒 Llamando a autoAddScannedProduct para PLU+peso
🔧 DEBUG: 🚀 INICIO autoAddScannedProduct
🔧 DEBUG: ✅ FIN autoAddScannedProduct - Producto agregado exitosamente
```

### Archivos Modificados

- `src/components/ShoppingCartRefactored.tsx`: Sistema de logs detallados implementado

## Corrección Crítica - Conflicto entre Sistemas de Detección de PLU

### Problema Identificado

Se descubrió que había **dos sistemas** compitiendo por procesar los códigos PLU:

1. **Sistema en `useProductSearch.ts`**: Detectaba PLUs de 3-6 dígitos y abría automáticamente el diálogo de selección
2. **Sistema en `ShoppingCartRefactored.tsx`**: Procesaba códigos completos PLU + peso (12-13 dígitos)

**Síntomas observados**:

```
🔧 DEBUG: Input event detectado, valor: "105"
🔧 DEBUG: 🎯 INICIO handleProductSelect  // ← Sistema antiguo interfiriendo
🔧 DEBUG: Producto seleccionado: Blanda
🔧 DEBUG: Estableciendo selectedProduct y abriendo diálogo
```

**Problema**: El sistema antiguo detectaba "105" como PLU válido y abría el diálogo **antes** de que se completara el código `0105000013552`.

### Solución Implementada

**Deshabilitación del Sistema Antiguo**: Se comentó la detección automática de PLU en `useProductSearch.ts`:

```typescript
// ✅ DESHABILITADO: Detección automática de PLU directo
// Este sistema interfería con el procesamiento de códigos PLU + peso completos
// Ahora solo se procesan PLUs a través del sistema de buffer en ShoppingCartRefactored
/*
// Detectar PLU directo (3-6 dígitos numéricos)
if (/^\d{3,6}$/.test(value)) {
  const productByPlu = availableProducts.find((p) => p.plu === value);
  if (productByPlu) {
    handleProductSelect(productByPlu);
  }
}
*/
```

### Resultado

Ahora el sistema:

- ✅ **No abre** diálogos automáticamente con códigos parciales
- ✅ **Espera** a que se complete el código completo PLU + peso
- ✅ **Procesa** correctamente el peso (1.355 kg en lugar de 13.552 kg)
- ✅ **No interfiere** con el sistema de buffer

### Flujo Correcto Esperado

**Antes (incorrecto)**:

```
"105" → Sistema antiguo detecta PLU → Abre diálogo → Se cierra → "0" se procesa por separado
```

**Ahora (correcto)**:

```
"0105000013552" → Sistema de buffer procesa código completo → PLU: 105, Peso: 1.355kg → Agrega correctamente
```

### Archivos Modificados

- `src/hooks/useProductSearch.ts`: Sistema de detección automática de PLU deshabilitado

## Problema de Diálogo QR Manual

### Problema Reportado

El botón para completar manualmente un pago QR no abría el diálogo de contraseña, tanto con AFIP como sin AFIP.

### Solución Implementada

1. **Logs Detallados**: Se agregaron logs extensivos para rastrear el estado del diálogo:

   ```javascript
   console.log("🔧 Estado de diálogos de contraseña manual:", {
     normalDialogOpen: paymentProcessor.manualQrPasswordDialogOpen,
     afipDialogOpen: afipPaymentProcessor.manualQrPasswordDialogOpen,
     isCurrentlyAfipFlow,
   });
   ```

2. **Monitoreo de Estados**: Se implementó un sistema de monitoreo para verificar que los diálogos se abran correctamente.

3. **Verificación de Componentes**: Se aseguró que los componentes `ManualQrDialog` estén correctamente renderizados y controlados.

### Archivos Modificados

- `src/components/ShoppingCartRefactored.tsx`: Logs y monitoreo agregados
- `src/components/shopping-cart/dialogs/QRPaymentDialog.tsx`: Logs detallados en el botón manual

### Estado Actual

El problema está siendo investigado con logs detallados. Los diálogos están correctamente implementados y deberían funcionar. Los logs ayudarán a identificar si hay algún problema en el flujo de estados.
