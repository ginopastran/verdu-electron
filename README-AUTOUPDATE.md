# 🚀 Sistema de Auto-Actualizaciones

Tu aplicación VerduSoft ahora cuenta con un sistema de auto-actualizaciones similar a Discord. Los usuarios instalan la app una vez y reciben actualizaciones automáticas.

## 🔧 ¿Cómo Funciona?

### Para los Usuarios

1. **Instalación única**: El usuario instala la app normalmente
2. **Verificación automática**: La app verifica actualizaciones cada vez que se abre
3. **Notificación**: Si hay una nueva versión, aparece una notificación elegante
4. **Descarga opcional**: El usuario puede descargar la actualización cuando quiera
5. **Instalación automática**: Al reiniciar, la nueva versión se instala automáticamente

### Para el Desarrollador

#### 1. **Configurar Repositorio en GitHub**

```bash
# Tu repositorio debe estar en GitHub para usar el auto-updater
# Ejemplo: https://github.com/tu-usuario/verdu-electron
```

#### 2. **Configurar Token de GitHub**

- Ve a: https://github.com/settings/tokens
- Crea un nuevo token con permisos de `repo` (para repos privados) o `public_repo`
- Configura la variable de entorno: `GH_TOKEN=tu-token-aqui`

#### 3. **Publicar Nueva Versión**

```bash
# Actualizar version en package.json
npm version patch  # o minor, major

# Publicar release
npm run release

# O crear un draft para revisar antes
npm run release:draft
```

## 📋 Flujo de Actualización

### 1. **Detección Automática**

- La app verifica actualizaciones 5 segundos después de cargar
- También verifica al hacer clic en "Verificar actualizaciones" (si agregas botón)

### 2. **Notificación al Usuario**

- Aparece una tarjeta elegante en la esquina inferior derecha
- Muestra la nueva versión y notas de la release
- Botones: "Más tarde" y "Descargar"

### 3. **Descarga**

- Se descarga en segundo plano
- Indicador de progreso visual
- No interfiere con el uso de la app

### 4. **Instalación**

- Se ofrece botón "Reiniciar e instalar"
- Al reiniciar, se aplica la actualización automáticamente
- La app se abre con la nueva versión

## 🛠️ Componentes Implementados

### Hook: `useAutoUpdater`

```typescript
const {
  updateAvailable, // boolean: hay actualización disponible
  updateInfo, // objeto con version, fecha, notas
  downloading, // boolean: descargando actualización
  downloaded, // boolean: actualización descargada
  checking, // boolean: verificando actualizaciones
  checkForUpdates, // función: verificar manualmente
  downloadUpdate, // función: descargar actualización
  installUpdate, // función: instalar y reiniciar
} = useAutoUpdater();
```

### Componente: `UpdateNotification`

- Interfaz visual para el sistema de actualizaciones
- Se muestra automáticamente cuando hay actualizaciones
- Diseño consistente con la app (colores emerald)

## 🔒 Configuración de Seguridad

### Code Signing (Recomendado para Producción)

```json
// En package.json -> build
"win": {
  "certificateFile": "path/to/certificate.p12",
  "certificatePassword": "password",
  "signtoolOptions": {
    "signingHashAlgorithms": ["sha256"]
  }
}
```

### Verificación de Actualizaciones

- Todas las actualizaciones se descargan desde GitHub Releases
- electron-updater verifica checksums automáticamente
- Solo se instalan releases oficiales del repositorio configurado

## 📦 Providers Alternativos

### Amazon S3

```json
"publish": {
  "provider": "s3",
  "bucket": "tu-bucket",
  "region": "us-east-1"
}
```

### Servidor Propio

```json
"publish": {
  "provider": "generic",
  "url": "https://tu-servidor.com/updates/"
}
```

## 🚨 Troubleshooting

### "No se pueden verificar actualizaciones"

- Verificar conexión a internet
- Revisar configuración de `publish` en package.json
- Verificar que el repositorio y releases existan

### "Error al descargar"

- Verificar permisos de escritura
- Revisar espacio en disco
- Comprobar que el release tenga los archivos correctos

### En Desarrollo

- El auto-updater solo funciona en builds de producción
- En desarrollo se muestran logs pero no verifica actualizaciones reales

## 🎯 Próximos Pasos

1. **Configurar tu repositorio en GitHub**
2. **Actualizar la configuración `publish` con tu usuario/repo real**
3. **Configurar tu `GH_TOKEN`**
4. **Hacer tu primer release: `npm run release:draft`**
5. **Probar el sistema con usuarios beta**

¡Tu app ahora se actualiza como las aplicaciones profesionales! 🎉
