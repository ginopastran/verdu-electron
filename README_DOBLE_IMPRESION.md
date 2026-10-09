# Funcionalidad de Doble Impresión

## Descripción

Se ha implementado la funcionalidad de doble impresión de tickets que permite imprimir 2 copias del mismo ticket cuando está habilitada en la configuración del negocio.

## Campo de configuración

La funcionalidad se controla mediante el campo `dobleImpresionEnabled` en la información del business obtenida desde la API:

- `true`: Se imprimen 2 tickets idénticos
- `false` o no definido: Se imprime 1 ticket (comportamiento original)

## Archivos modificados

### 1. `src/utils/businessHelpers.ts`

- ✅ Agregada función `getBusinessInfo()` para obtener información completa del business desde la API
- Esta función obtiene el businessId y hace la llamada a `/api/business/${businessId}`

### 2. `src/hooks/useTicketPrinting.ts`

- ✅ Modificada función `handleTicketPrinting()` para soportar doble impresión
- ✅ Agregado parámetro `API_URL` y `appId` para obtener configuración del business
- ✅ Implementada lógica de doble impresión con delay de 1 segundo entre impresiones
- ✅ Mensajes diferenciados en toast para doble impresión
- ✅ Manejo de errores específico para cada impresión

### 3. `src/hooks/usePaymentProcessing.ts`

- ✅ Actualizada función `handleTicketPrinting()` para usar el nuevo hook
- ✅ Agregada obtención del `appId` desde window.electron
- ✅ Eliminada declaración de tipos duplicada

### 4. `src/components/RecentOrdersDialog.tsx`

- ✅ Actualizado para usar el hook `useTicketPrinting`
- ✅ Modificada función `handleReprintClick()` para pasar parámetros necesarios
- ✅ Eliminado prop `handleTicketPrinting` del componente

### 5. `src/components/ShoppingCartRefactored.tsx`

- ✅ Removido prop `handleTicketPrinting` del componente `RecentOrdersDialog`

## Flujo de funcionamiento

1. **Al procesar un pago**: Se llama a `handleTicketPrinting(orderData, API_URL, appId)`
2. **Obtención de configuración**: Se consulta `/api/business/${businessId}` para obtener `dobleImpresionEnabled`
3. **Decisión de impresión**:
   - Si `dobleImpresionEnabled === true`: Imprime 2 tickets con delay
   - Si `dobleImpresionEnabled === false`: Imprime 1 ticket (comportamiento original)
4. **Feedback al usuario**: Toast diferenciado según el tipo de impresión

## Configuración del negocio

Para habilitar la doble impresión, el campo debe configurarse en la base de datos:

```sql
UPDATE business SET dobleImpresionEnabled = true WHERE id = ?;
```

## Logs y debugging

El sistema incluye logs detallados:

- `🏢 Obteniendo información del business para doble impresión...`
- `🖨️🖨️ DOBLE IMPRESIÓN HABILITADA - Imprimiendo 2 tickets`
- `🖨️ IMPRESIÓN SIMPLE - Imprimiendo 1 ticket`
- `📄 Realizando primera impresión...`
- `📄 Realizando segunda impresión...`

## Compatibilidad

- ✅ Funciona con todos los métodos de pago (efectivo, QR, tarjeta, mixto)
- ✅ Compatible con reimpresión desde historial de órdenes
- ✅ No afecta la funcionalidad existente cuando está deshabilitado
- ✅ Manejo de errores robusto
- ❌ NO aplicado al cierre de caja (según requerimiento)
