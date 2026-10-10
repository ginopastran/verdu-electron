import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SALIDA = path.dirname(fileURLToPath(import.meta.url));
const API = "https://market.andex.tech";
const ANCHO = 1200;
const ALTO = 800;

const ahora = new Date();
const haceHoras = (h, m = 0) => new Date(ahora.getTime() - (h * 60 + m) * 60000).toISOString();
const haceDias = (d) => new Date(ahora.getTime() - d * 86400000).toISOString();

function qrFalso() {
  const n = 29;
  let semilla = 7;
  const azar = () => ((semilla = (semilla * 1103515245 + 12345) % 2147483648) / 2147483648);
  const finder = (x, y, i, j) => {
    const dx = i - x, dy = j - y;
    if (dx < 0 || dy < 0 || dx > 6 || dy > 6) return null;
    return dx === 0 || dy === 0 || dx === 6 || dy === 6 || (dx >= 2 && dx <= 4 && dy >= 2 && dy <= 4);
  };
  let rects = "";
  for (let i = 0; i < n; i++)
    for (let j = 0; j < n; j++) {
      const f = finder(0, 0, i, j) ?? finder(n - 7, 0, i, j) ?? finder(0, n - 7, i, j);
      const separador = (i <= 7 && j <= 7) || (i >= n - 8 && j <= 7) || (i <= 7 && j >= n - 8);
      const negro = f ?? (separador ? false : azar() > 0.5);
      if (negro) rects += `<rect x="${i + 2}" y="${j + 2}" width="1" height="1"/>`;
    }
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${n + 4} ${n + 4}" shape-rendering="crispEdges"><rect width="100%" height="100%" fill="#fff"/><g fill="#000">${rects}</g></svg>`;
  return "data:image/svg+xml;base64," + Buffer.from(svg).toString("base64");
}

const productos = [
  [1, "Tomate perito", 2400, "Kg", "001", null, 10.5],
  [2, "Papa negra", 900, "Kg", "002", null, 10.5],
  [3, "Cebolla", 1100, "Kg", "003", null, 10.5],
  [4, "Banana Ecuador", 2200, "Kg", "004", null, 10.5],
  [5, "Manzana roja", 2800, "Kg", "005", null, 10.5],
  [6, "Naranja de jugo", 1500, "Kg", "006", null, 10.5],
  [7, "Mandarina", 1600, "Kg", "007", null, 10.5],
  [8, "Zapallo anco", 1200, "Kg", "008", null, 10.5],
  [9, "Zanahoria", 1000, "Kg", "009", null, 10.5],
  [10, "Limón", 2000, "Kg", "010", null, 10.5],
  [11, "Lechuga criolla", 2500, "Kg", "011", null, 10.5],
  [12, "Pimiento morrón rojo", 4500, "Kg", "012", null, 10.5],
  [13, "Batata", 1300, "Kg", "013", null, 10.5],
  [14, "Pera Williams", 2600, "Kg", "014", null, 10.5],
  [15, "Uva blanca", 4800, "Kg", "015", null, 10.5],
  [16, "Berenjena", 2000, "Kg", "016", null, 10.5],
  [17, "Zucchini", 2200, "Kg", "017", null, 10.5],
  [18, "Kiwi", 4000, "Kg", "018", null, 10.5],
  [19, "Palta Hass", 1500, "Unidad", null, "7791234500019", 10.5],
  [20, "Ananá", 3500, "Unidad", null, "7791234500026", 10.5],
  [21, "Huevos blancos maple x30", 6500, "Unidad", null, "7791234500033", 21],
  [22, "Ajo (cabeza)", 600, "Unidad", null, "7791234500040", 10.5],
  [23, "Perejil (atado)", 800, "Unidad", null, "7791234500057", 10.5],
  [24, "Choclo", 700, "Unidad", null, "7791234500064", 10.5],
  [25, "Frutilla bandeja 500 g", 3000, "Unidad", null, "7791234500071", 10.5],
  [26, "Acelga (atado)", 1200, "Unidad", null, "7791234500088", 10.5],
].map(([id, nombre, precio, tipoMedida, plu, codigoBarras, ivaPorcentaje]) => ({
  id,
  nombre,
  precio,
  tipoMedida,
  costo: Math.round(precio * 0.62),
  codigoBarras,
  plu,
  ivaIncluido: true,
  ivaPorcentaje,
  stock: tipoMedida === "Kg" ? 85 : 40,
  activo: true,
}));

const listasPrecios = [
  { id: 1, nombre: "Mayorista", descripcion: "Compras desde 10 kg", activa: true, _count: { productos: 6 } },
  { id: 2, nombre: "Comercios", descripcion: "Restaurantes y almacenes", activa: true, _count: { productos: 4 } },
];

const preciosLista = {
  1: { 1: 2100, 2: 780, 3: 950, 5: 2450, 20: 3100, 21: 5900 },
  2: { 1: 2250, 2: 830, 5: 2600, 21: 6100 },
};

const vendedores = [
  { id: 11, nombre: "Lucía Fernández", email: "lucia@lahuerta.test" },
  { id: 12, nombre: "Martín Gómez", email: "martin@lahuerta.test" },
  { id: 13, nombre: "Sofía Ramírez", email: "sofia@lahuerta.test" },
  { id: 14, nombre: "Diego Álvarez", email: "diego@lahuerta.test" },
  { id: 15, nombre: "Carla Benítez", email: "carla@lahuerta.test" },
];

const admin = { id: 1, nombre: "Roberto Sánchez", email: "admin@lahuerta.test" };

const sucursales = [
  { id: 1, nombre: "Casa central", ubicacion: "Av. San Martín 1450, Godoy Cruz, Mendoza", direccion: "Av. San Martín 1450, Godoy Cruz", telefono: "261 424-8890", esPrincipal: true, businessId: 7 },
  { id: 2, nombre: "Sucursal Mercado", ubicacion: "Mercado Cooperativo, Guaymallén", direccion: "Mercado Cooperativo, Guaymallén", telefono: "261 431-2207", esPrincipal: false, businessId: 7 },
];

const negocioBase = {
  id: 7,
  nombre: "Verdulería La Huerta",
  tipo: "verduleria",
  creadoEn: haceDias(400),
  sistemaPago: "redondeo",
  descuentoEfectivo: 10,
  mpEnabled: true,
  mpExternalPosId: "LAHUERTA01",
  mpUserId: "123456789",
  afipHabilitado: false,
  facturacionHabilitada: false,
  dobleImpresionEnabled: false,
  cancelacionHabilitada: false,
  ivaIncluidoEnPrecios: true,
  popupListaPreciosEnabled: false,
  listaPrecioPorDefectoId: null,
  etiquetaPrecioBase: "Precio catálogo",
  cuit: "30-71234567-8",
  sucursales,
  configuracionAfip: {
    id: 1,
    cuit: "30-71234567-8",
    produccion: false,
    puntoVenta: 2,
    ultimoNumeroA: 12,
    ultimoNumeroB: 212,
    ultimoNumeroC: 3,
    activo: true,
    condicionIva: "Responsable Inscripto",
    domicilioFiscal: "Av. San Martín 1450, Godoy Cruz, Mendoza",
    razonSocial: "La Huerta S.R.L.",
    tipoFactura: "B",
  },
};

const clientes = [
  { id: "c1", nombre: "Restaurante El Mesón", apellido: null, razonSocial: "El Mesón S.A.", cuit: "30-70987654-3", condicionFiscal: "Responsable Inscripto", email: "compras@elmeson.test", telefono: "261 555-0101", direccion: "Belgrano 320, Mendoza" },
  { id: "c2", nombre: "Almacén Don Pepe", apellido: null, razonSocial: "José Pérez", cuit: "20-23456789-1", condicionFiscal: "Monotributista", email: "donpepe@correo.test", telefono: "261 555-0102", direccion: "Perú 1120, Godoy Cruz" },
  { id: "c3", nombre: "María", apellido: "Quiroga", razonSocial: null, cuit: "27-30111222-4", condicionFiscal: "Consumidor Final", email: null, telefono: "261 555-0103", direccion: "Barrio Bombal, Godoy Cruz" },
  { id: "c4", nombre: "Hotel Aconcagua", apellido: null, razonSocial: "Hotelera Cuyana S.A.", cuit: "30-71555444-9", condicionFiscal: "Responsable Inscripto", email: "proveedores@aconcagua.test", telefono: "261 555-0104", direccion: "San Lorenzo 545, Mendoza" },
  { id: "c5", nombre: "Comedor Los Peques", apellido: null, razonSocial: "Asociación Civil Los Peques", cuit: "30-70222333-5", condicionFiscal: "Exento", email: null, telefono: "261 555-0105", direccion: "Lavalle 77, Las Heras" },
  { id: "c6", nombre: "Carlos", apellido: "Molina", razonSocial: null, cuit: "20-28999111-7", condicionFiscal: "Monotributista", email: "cmolina@correo.test", telefono: "261 555-0106", direccion: "Mitre 2301, Luján de Cuyo" },
];

const pr = (id) => productos.find((p) => p.id === id);
const detalle = (n, id, cantidad, precio = pr(id).precio) => ({
  id: n,
  productoId: id,
  descripcion: pr(id).nombre,
  cantidad,
  precioUnitario: precio,
  subtotal: Math.round(precio * cantidad * 100) / 100,
  producto: pr(id),
});

const facturas = [
  {
    id: "f101", numero: "R-0002-00000148", tipoFactura: "remito", puntoVenta: 2, fecha: haceDias(1), subtotal: 86500, impuestos: 0, total: 86500,
    estado: "pendiente", cliente: clientes[0], clienteId: "c1", esCuentaCorriente: true, observaciones: "Entrega por la mañana",
    detalles: [detalle(1, 1, 15, 2100), detalle(2, 2, 25, 780), detalle(3, 3, 10, 950), detalle(4, 12, 4)],
    pagos: [{ id: 501, monto: 30000, metodoPago: "efectivo", fecha: haceDias(1), referencia: "Seña", estado: "aprobado" }],
  },
  {
    id: "f102", numero: "A-0002-00000013", tipoFactura: "A", puntoVenta: 2, fecha: haceDias(3), subtotal: 57851.24, impuestos: 6074.38, total: 63925.62,
    estado: "pagada", cliente: clientes[3], clienteId: "c4", esCuentaCorriente: true, cae: "75412369874521", vencimientoCae: haceDias(-7),
    detalles: [detalle(5, 5, 8), detalle(6, 15, 3), detalle(7, 21, 4), detalle(8, 18, 2.5)],
    pagos: [
      { id: 502, monto: 40000, metodoPago: "transferencia", fecha: haceDias(3), referencia: "Transf. Banco Nación #88231", estado: "aprobado" },
      { id: 503, monto: 23925.62, metodoPago: "tarjeta", fecha: haceDias(2), referencia: "Visa débito", estado: "aprobado" },
    ],
  },
  {
    id: "f103", numero: "R-0002-00000147", tipoFactura: "remito", puntoVenta: 2, fecha: haceDias(4), subtotal: 41200, impuestos: 0, total: 41200,
    estado: "pendiente", cliente: clientes[1], clienteId: "c2", esCuentaCorriente: true,
    detalles: [detalle(9, 6, 10), detalle(10, 7, 8), detalle(11, 4, 6)],
    pagos: [],
  },
  {
    id: "f104", numero: "C-0002-00000004", tipoFactura: "C", puntoVenta: 2, fecha: haceDias(6), subtotal: 18600, impuestos: 0, total: 18600,
    estado: "pagada", cliente: clientes[2], clienteId: "c3", esCuentaCorriente: false, cae: "75412369870044", vencimientoCae: haceDias(4),
    detalles: [detalle(12, 25, 2), detalle(13, 19, 4), detalle(14, 23, 3)],
    pagos: [{ id: 504, monto: 18600, metodoPago: "efectivo", fecha: haceDias(6), estado: "aprobado" }],
  },
  {
    id: "f105", numero: "R-0002-00000146", tipoFactura: "remito", puntoVenta: 2, fecha: haceDias(8), subtotal: 128400, impuestos: 0, total: 128400,
    estado: "pendiente", cliente: clientes[4], clienteId: "c5", esCuentaCorriente: true, observaciones: "Pago a 30 días",
    detalles: [detalle(15, 2, 40, 780), detalle(16, 8, 20), detalle(17, 9, 15), detalle(18, 13, 12), detalle(19, 26, 10)],
    pagos: [
      { id: 505, monto: 50000, metodoPago: "transferencia", fecha: haceDias(7), referencia: "Transf. #10293", estado: "aprobado" },
      { id: 506, monto: 25000, metodoPago: "efectivo", fecha: haceDias(2), estado: "aprobado" },
    ],
  },
  {
    id: "f106", numero: "A-0002-00000012", tipoFactura: "A", puntoVenta: 2, fecha: haceDias(10), subtotal: 34710.74, impuestos: 3644.63, total: 38355.37,
    estado: "pendiente", cliente: clientes[0], clienteId: "c1", esCuentaCorriente: true, cae: "75412369861177", vencimientoCae: haceDias(0),
    detalles: [detalle(20, 1, 10), detalle(21, 11, 3), detalle(22, 16, 2.5)],
    pagos: [],
  },
].map((f) => {
  const pagado = Math.round(f.pagos.reduce((t, p) => t + p.monto, 0) * 100) / 100;
  return { ...f, pagoInicial: f.pagos[0]?.monto ?? 0, saldoPendiente: Math.round((f.total - pagado) * 100) / 100 };
});

const ordenesHoy = [
  { id: 9021, idReal: 9021, fecha: haceHoras(0, 12), metodoPago: "efectivo", total: 7650, tieneDescuento: true, tipoDescuento: "porcentual", valorDescuento: 10, montoDescuento: 850, subtotalSinDescuento: 8500, estado: "COMPLETADA", vendedor: { nombre: "Lucía Fernández" }, items: [{ productoId: 1, nombre: "Tomate perito", cantidad: 1.25, precioHistorico: 2400, subtotal: 3000 }, { productoId: 21, nombre: "Huevos blancos maple x30", cantidad: 1, precioHistorico: 5500, subtotal: 5500 }] },
  { id: 9020, idReal: 9020, fecha: haceHoras(0, 35), metodoPago: "qr", total: 12480, tieneDescuento: false, estado: "COMPLETADA", vendedor: { nombre: "Lucía Fernández" }, items: [{ productoId: 5, nombre: "Manzana roja", cantidad: 2.1, precioHistorico: 2800, subtotal: 5880 }, { productoId: 15, nombre: "Uva blanca", cantidad: 1.375, precioHistorico: 4800, subtotal: 6600 }] },
  { id: 9019, idReal: 9019, fecha: haceHoras(1, 5), metodoPago: "tarjeta", total: 23900, tieneDescuento: false, estado: "COMPLETADA", vendedor: { nombre: "Lucía Fernández" }, items: [{ productoId: 20, nombre: "Ananá", cantidad: 2, precioHistorico: 3500, subtotal: 7000 }, { productoId: 18, nombre: "Kiwi", cantidad: 1.2, precioHistorico: 4000, subtotal: 4800 }, { productoId: 12, nombre: "Pimiento morrón rojo", cantidad: 0.8, precioHistorico: 4500, subtotal: 3600 }, { productoId: 21, nombre: "Huevos blancos maple x30", cantidad: 1, precioHistorico: 6500, subtotal: 6500 }, { productoId: 22, nombre: "Ajo (cabeza)", cantidad: 3, precioHistorico: 600, subtotal: 1800 }, { productoId: 24, nombre: "Choclo", cantidad: 3, precioHistorico: 700, subtotal: 200 }] },
  { id: 9018, idReal: 9018, fecha: haceHoras(1, 40), metodoPago: "split", total: 15300, tieneDescuento: false, estado: "COMPLETADA", vendedor: { nombre: "Lucía Fernández" }, items: [{ productoId: 2, nombre: "Papa negra", cantidad: 5, precioHistorico: 900, subtotal: 4500 }, { productoId: 3, nombre: "Cebolla", cantidad: 3, precioHistorico: 1100, subtotal: 3300 }, { productoId: 8, nombre: "Zapallo anco", cantidad: 2.5, precioHistorico: 1200, subtotal: 3000 }, { productoId: 9, nombre: "Zanahoria", cantidad: 4.5, precioHistorico: 1000, subtotal: 4500 }] },
  { id: 9017, idReal: 9017, fecha: haceHoras(2, 15), metodoPago: "efectivo", total: 4350, tieneDescuento: false, estado: "COMPLETADA", vendedor: { nombre: "Lucía Fernández" }, items: [{ productoId: 4, nombre: "Banana Ecuador", cantidad: 1.5, precioHistorico: 2200, subtotal: 3300 }, { productoId: 23, nombre: "Perejil (atado)", cantidad: 1, precioHistorico: 800, subtotal: 800 }] },
  { id: 9016, idReal: 9016, fecha: haceHoras(3, 2), metodoPago: "tarjeta", total: 9800, tieneDescuento: true, tipoDescuento: "cantidad", valorDescuento: 500, montoDescuento: 500, subtotalSinDescuento: 10300, estado: "COMPLETADA", vendedor: { nombre: "Lucía Fernández" }, items: [{ productoId: 25, nombre: "Frutilla bandeja 500 g", cantidad: 2, precioHistorico: 3000, subtotal: 6000 }, { productoId: 10, nombre: "Limón", cantidad: 1, precioHistorico: 2000, subtotal: 2000 }, { productoId: 6, nombre: "Naranja de jugo", cantidad: 1.5, precioHistorico: 1500, subtotal: 2300 }] },
];

const facturasBHoy = [
  { id: "fb88", numero: "B-0002-00000212", fecha: haceHoras(0, 50), metodoPago: "tarjeta", total: 18750, tipoFactura: "B", cae: "75412369899001", vendedor: { nombre: "Lucía Fernández" }, detalles: [{ descripcion: "Manzana roja", cantidad: 3, precioUnitario: 2800, subtotal: 8400 }, { descripcion: "Pera Williams", cantidad: 2.5, precioUnitario: 2600, subtotal: 6500 }, { descripcion: "Palta Hass", cantidad: 2, precioUnitario: 1500, subtotal: 3000 }, { descripcion: "Ajo (cabeza)", cantidad: 1, precioUnitario: 600, subtotal: 850 }] },
  { id: "fb87", numero: "B-0002-00000211", fecha: haceHoras(2, 45), metodoPago: "efectivo", total: 6300, tipoFactura: "B", cae: "75412369898877", vendedor: { nombre: "Lucía Fernández" }, detalles: [{ descripcion: "Papa negra", cantidad: 3, precioUnitario: 900, subtotal: 2700 }, { descripcion: "Batata", cantidad: 2, precioUnitario: 1300, subtotal: 2600 }, { descripcion: "Choclo", cantidad: 1, precioUnitario: 700, subtotal: 1000 }] },
];

const itemCarrito = (id, cantidad, extra = {}) => {
  const p = pr(id);
  const precio = extra.precio ?? p.precio;
  return {
    id,
    cartId: `${id}-seed-${cantidad}`,
    name: p.nombre,
    quantity: cantidad,
    unit: p.tipoMedida,
    pricePerUnit: precio,
    subtotal: Math.round(precio * cantidad * 100) / 100,
    costo: p.costo,
    ivaIncluido: true,
    ivaPorcentaje: p.ivaPorcentaje,
    listaPrecioId: extra.listaPrecioId ?? null,
    listaPrecioNombre: extra.listaPrecioNombre ?? "Precio catálogo",
  };
};

const carritoSembrado = JSON.stringify({
  screens: [
    {
      id: 0,
      items: [
        itemCarrito(1, 1.245),
        itemCarrito(2, 2.5),
        itemCarrito(5, 0.98),
        itemCarrito(21, 1),
        itemCarrito(19, 3),
        itemCarrito(12, 0.415),
        itemCarrito(23, 2),
      ],
    },
    { id: 1, items: [itemCarrito(4, 1.1), itemCarrito(25, 1)] },
  ],
  activeScreen: 0,
});

const datos = {
  productos,
  listasPrecios,
  preciosLista,
  vendedores,
  admin,
  sucursales,
  negocioBase,
  clientes,
  facturas,
  ordenesHoy,
  facturasBHoy,
  qrImagen: qrFalso(),
};

function enPagina({ API, datos, escenario }) {
  const esc = escenario;
  window.__ESCENARIO__ = esc;
  const registro = (window.__MOCK_LOG__ = []);
  const registroIpc = (window.__IPC_LOG__ = []);

  const crearStorage = () => {
    const m = new Map();
    return {
      get length() {
        return m.size;
      },
      key: (i) => [...m.keys()][i] ?? null,
      getItem: (k) => (m.has(k) ? m.get(k) : null),
      setItem: (k, v) => void m.set(k, String(v)),
      removeItem: (k) => void m.delete(k),
      clear: () => m.clear(),
    };
  };
  const ls = crearStorage();
  for (const [k, v] of Object.entries(esc.localStorage || {})) ls.setItem(k, v);
  Object.defineProperty(window, "localStorage", { value: ls, configurable: true });
  Object.defineProperty(window, "sessionStorage", { value: crearStorage(), configurable: true });
  window.alert = (m) => registroIpc.push({ canal: "alert", args: [m] });
  window.confirm = (m) => (registroIpc.push({ canal: "confirm", args: [m] }), false);

  const negocio = () => {
    const n = JSON.parse(JSON.stringify({ ...datos.negocioBase, ...(esc.negocio || {}) }));
    return n;
  };
  const adminData = {
    id: datos.admin.id,
    nombre: datos.admin.nombre,
    email: datos.admin.email,
    businessId: datos.negocioBase.id,
    businessName: datos.negocioBase.nombre,
  };
  const store = new Map();
  if (esc.adminConfigurado) store.set("adminData", adminData);

  const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
  const respuestasIpc = {
    "store-get": (k) => store.get(k) ?? null,
    "store-set": (k, v) => void store.set(k, v),
    "store-delete": (k) => void store.delete(k),
    "store-has": (k) => store.has(k),
    "print-ticket": () => ({ success: true }),
    "print-afip-ticket": () => ({ success: true }),
    "print-closing": () => ({ success: true }),
    "print-factura-ticket": () => ({ success: true }),
    "get-available-printers": () => [{ name: "TP806L" }, { name: "Microsoft Print to PDF" }],
    "read-peso": () => ({ success: true, peso: esc.peso ?? 1245 }),
    "check-for-updates": () =>
      esc.actualizacion
        ? { available: true, info: { version: "4.1.0", releaseDate: new Date().toISOString(), releaseNotes: "Mejoras en la impresión de tickets y en la lectura de la balanza." } }
        : { available: false },
    "download-update": async () => (await dormir(esc.demoraDescarga ?? 4000), { success: true }),
    "install-update": () => null,
    "balanza-config-get": () => ({
      config: { modo: "serie", puerto: "COM3", baudRate: 9600, formato: "8N1", consulta: "enq", unidadEntera: "gramos" },
      estado: { conectada: true },
    }),
    "balanza-puertos": () => [
      { path: "COM3", manufacturer: "Prolific" },
      { path: "COM4", manufacturer: "FTDI" },
    ],
    "balanza-config-set": () => ({ ok: true }),
    "impresion-config-get": () => ({ anchoPapel: 80, tieneLogo: true, impresora: null, recurso: "TP806L" }),
    "impresora-elegir": (v) => ({ ok: true, recurso: v || "TP806L" }),
    "impresion-config-set": () => ({ ok: true }),
    "logo-negocio-guardar": () => null,
  };
  const ipcInvoke = async (canal, ...args) => {
    const fn = respuestasIpc[canal] || (canal.startsWith("print") ? () => ({ success: true }) : null);
    const r = fn ? await fn(...args) : null;
    registroIpc.push({ canal, args: canal === "store-set" ? [args[0]] : args, respuesta: r, mock: !!fn });
    return r;
  };
  const mocks = {
    electron: {
      ipcRenderer: { invoke: ipcInvoke, on: () => {}, removeListener: () => {} },
      process: { argv: [] },
    },
    electronStore: {
      get: (k) => ipcInvoke("store-get", k),
      set: (k, v) => ipcInvoke("store-set", k, v),
      delete: (k) => ipcInvoke("store-delete", k),
      has: (k) => ipcInvoke("store-has", k),
    },
    autoUpdater: {
      checkForUpdates: () => ipcInvoke("check-for-updates"),
      downloadUpdate: () => ipcInvoke("download-update"),
      installUpdate: () => ipcInvoke("install-update"),
    },
    pesoReader: { readPeso: () => ipcInvoke("read-peso") },
    printer: {
      printTicket: (d) => ipcInvoke("print-ticket", d),
      printAfipTicket: (d) => ipcInvoke("print-afip-ticket", d),
      printClosing: (d) => ipcInvoke("print-closing", d),
      getAvailablePrinters: () => ipcInvoke("get-available-printers"),
    },
  };
  for (const [k, v] of Object.entries(mocks))
    Object.defineProperty(window, k, { get: () => v, set: () => {}, configurable: true });

  const sesionDe = (usuario, rol) => ({
    user: {
      id: usuario.id,
      nombre: usuario.nombre,
      email: usuario.email,
      rol: { nombre: rol },
      businessId: datos.negocioBase.id,
      sucursalId: 1,
      sucursal: { id: 1, nombre: "Casa central", businessId: datos.negocioBase.id },
      pesoManualEnabled: !!esc.pesoManual,
      cierreDeCajaEnabled: esc.cierreDeCaja !== false,
    },
    expires: new Date(Date.now() + 86400000).toISOString(),
  });

  const paginar = (lista, url) => {
    const page = Number(url.searchParams.get("page") || 1);
    const limit = Number(url.searchParams.get("limit") || 12);
    const total = lista.length;
    const totalPages = Math.max(1, Math.ceil(total / limit));
    return {
      items: lista.slice((page - 1) * limit, page * limit),
      pagination: { page, limit, total, totalPages, hasNext: page < totalPages, hasPrev: page > 1 },
    };
  };

  const rutas = [
    ["POST", /^\/api\/login$/, "login", (m, u, b) => {
      if (esc.loginFalla) return [401, { message: "Credenciales inválidas" }];
      return [200, { token: b?.email === datos.admin.email ? "token-admin" : `token-${b?.email}` }];
    }],
    ["GET", /^\/api\/auth\/session$/, "sesion", (m, u, b, h) => {
      const token = (h.Authorization || h.authorization || "").replace("Bearer ", "");
      if (token === "token-admin") return [200, sesionDe(datos.admin, "ADMIN")];
      const email = token.replace("token-", "");
      const v = datos.vendedores.find((x) => x.email === email) || datos.vendedores[0];
      return [200, sesionDe(v, esc.rolSesion || "VENDEDOR")];
    }],
    ["GET", /^\/api\/usuarios\/vendedores$/, "vendedores", () =>
      esc.vendedoresFalla ? [500, { error: "Error interno" }] : [200, datos.vendedores]],
    ["GET", /^\/api\/usuarios$/, "usuarios", () => [200, [
      ...datos.vendedores.map((v) => ({ ...v, rol: { nombre: "VENDEDOR" }, sucursalId: 1 })),
      { ...datos.admin, rol: { nombre: "ADMIN" }, sucursalId: 1 },
    ]]],
    ["GET", /^\/api\/sucursales$/, "sucursales", () => [200, datos.sucursales]],
    ["GET", /^\/api\/business\/logo-comprobante$/, "logo", () => [404, { error: "Sin logo" }]],
    ["GET", /^\/api\/business\/(\d+)$/, "negocio", () => [200, negocio()]],
    ["GET", /^\/api\/productos$/, "productos", (m, u) => {
      const p = paginar(datos.productos, u);
      return [200, { productos: p.items, pagination: p.pagination }];
    }],
    ["GET", /^\/api\/productos\/(\d+)\/listas-precios$/, "productoListas", (m) => {
      const id = Number(m[1]);
      const filas = datos.listasPrecios
        .filter((l) => datos.preciosLista[l.id][id] !== undefined)
        .map((l) => ({
          id: l.id * 100 + id,
          listaPrecioId: l.id,
          productoId: id,
          precio: datos.preciosLista[l.id][id],
          activa: true,
          tipoMedida: datos.productos.find((p) => p.id === id).tipoMedida,
          listaPrecio: { id: l.id, nombre: l.nombre, activa: true, tipoMedida: datos.productos.find((p) => p.id === id).tipoMedida },
        }));
      return [200, { productos: filas }];
    }],
    ["GET", /^\/api\/productos\/(\d+)$/, "producto", (m) => {
      const p = datos.productos.find((x) => x.id === Number(m[1]));
      return p ? [200, p] : [404, { error: "No encontrado" }];
    }],
    ["GET", /^\/api\/listas-precios$/, "listasPrecios", () => [200, { listasPrecios: datos.listasPrecios }]],
    ["GET", /^\/api\/listas-precios\/(\d+)\/productos$/, "listaProductos", (m) =>
      [200, Object.entries(datos.preciosLista[m[1]] || {}).map(([pid, precio]) => ({ productoId: Number(pid), precio }))]],
    ["GET", /^\/api\/listas-precios\/(\d+)\/productos\/(\d+)$/, "listaProducto", (m) => {
      const precio = (datos.preciosLista[m[1]] || {})[m[2]];
      return precio !== undefined ? [200, { productoId: Number(m[2]), precio }] : [404, { error: "No está en la lista" }];
    }],
    ["GET", /^\/api\/clientes$/, "clientes", (m, u) => {
      const q = (u.searchParams.get("search") || "").toLowerCase();
      const lista = datos.clientes.filter((c) => !q || `${c.nombre} ${c.apellido} ${c.razonSocial} ${c.cuit}`.toLowerCase().includes(q));
      const p = paginar(lista, u);
      return [200, { clientes: p.items, pagination: p.pagination }];
    }],
    ["GET", /^\/api\/clientes\/([\w-]+)$/, "cliente", (m) => {
      const c = datos.clientes.find((x) => x.id === m[1]);
      return c ? [200, c] : [404, { error: "No encontrado" }];
    }],
    ["POST", /^\/api\/ordenes$/, "crearOrden", () => [200, { id: 9022, idReal: 9022, estado: "COMPLETADA" }]],
    ["GET", /^\/api\/ordenes\/vendedor\/(\d+)$/, "ordenesVendedor", () => [200, datos.ordenesHoy]],
    ["GET", /^\/api\/facturas\/vendedor\/(\d+)$/, "facturasVendedor", () => [200, datos.facturasBHoy]],
    ["POST", /^\/api\/facturas\/crear-afip$/, "crearAfip", () => [200, {
      success: true,
      factura: { id: "fb89", numero: "B-0002-00000213", tipoFactura: "B", puntoVenta: 2, fecha: new Date().toISOString() },
      afip: { cae: "75412369899123", vencimientoCae: new Date(Date.now() + 10 * 86400000).toISOString(), numeroComprobante: 213, puntoVenta: 2, tipoComprobante: 6 },
    }]],
    ["POST", /^\/api\/facturas\/agregar-pago$/, "agregarPago", (m, u, b) => [200, { success: true, pago: { id: 599, ...b, estado: "aprobado" } }]],
    ["GET", /^\/api\/facturas$/, "facturas", (m, u) => {
      const lista = esc.sinFacturas ? [] : datos.facturas;
      const p = paginar(lista, u);
      return [200, { facturas: p.items, pagination: { total: p.pagination.total, page: p.pagination.page, limit: p.pagination.limit, totalPages: p.pagination.totalPages } }];
    }],
    ["GET", /^\/api\/facturas\/([\w-]+)$/, "factura", (m) => {
      const f = datos.facturas.find((x) => x.id === m[1]);
      return f ? [200, f] : [404, { error: "No encontrada" }];
    }],
    ["PUT", /^\/api\/facturas\/([\w-]+)$/, "editarFactura", (m, u, b) => [200, { ...datos.facturas.find((x) => x.id === m[1]), ...b }]],
    ["POST", /^\/api\/facturas$/, "crearFactura", (m, u, b) => [200, { id: "f107", numero: "R-0002-00000149", ...b }]],
    ["POST", /^\/api\/mercadopago\/generate-qr$/, "generarQr", (m, u, b) => [200, {
      id: 9023,
      orderId: 9023,
      qrData: "00020101021243650016COM.MERCADOLIBRE0201306364fake-qr-lahuerta-9023",
      qrImageUrl: datos.qrImagen,
      status: "PENDING",
      monto: b?.monto,
      isSplitPayment: !!b?.isSplitPayment,
      cashAmount: b?.cashAmount || 0,
    }]],
    ["GET", /^\/api\/mercadopago\/check-status\/(\w+)$/, "estadoQr", () => [200, { status: "PENDING" }]],
    ["GET", /^\/api\/mercadopago\/check-status$/, "estadoQr2", () => [200, { status: "PENDING" }]],
    ["POST", /^\/api\/mercadopago\/manual-complete$/, "qrManual", () => [200, { success: true }]],
    ["POST", /^\/api\/cierres$/, "cierre", (m, u, b) => [200, {
      id: 311,
      periodo: b?.periodo || "mañana",
      fechaInicio: b?.fechaInicio,
      fechaCierre: b?.fechaCierre,
      totalVentas: 186450,
      cantidadVentas: 27,
      ventasPorMetodo: { efectivo: 84300, tarjeta: 61250, qr: 32400, transferencia: 8500 },
      ventasPorVendedor: [
        { vendedorId: 11, nombre: "Lucía Fernández", total: 112300, cantidad: 16 },
        { vendedorId: 12, nombre: "Martín Gómez", total: 74150, cantidad: 11 },
      ],
      vendedor: { nombre: "Lucía Fernández" },
      sucursal: { nombre: "Casa central" },
      totalFacturas: 2,
      montoFacturas: 25050,
    }]],
    ["POST", /^\/api\/cancelaciones$/, "cancelacion", () => [200, { success: true, id: 77 }]],
  ];

  const nativo = window.fetch.bind(window);
  const encabezados = (h) => {
    if (!h) return {};
    if (h instanceof Headers) return Object.fromEntries(h.entries());
    if (Array.isArray(h)) return Object.fromEntries(h);
    return { ...h };
  };
  const mockFetch = async (recurso, opciones = {}) => {
    const urlTexto = recurso instanceof Request ? recurso.url : String(recurso);
    if (urlTexto.startsWith("http://ipc.localhost/")) throw new TypeError("IPC por postMessage");
    const url = new URL(urlTexto, location.href);
    const esApi = urlTexto.startsWith(API) || url.pathname.startsWith("/api/");
    if (!esApi) return nativo(recurso, opciones);
    const metodo = (opciones.method || (recurso instanceof Request ? recurso.method : "GET")).toUpperCase();
    let cuerpo = null;
    try {
      cuerpo = opciones.body ? JSON.parse(opciones.body) : null;
    } catch {
      cuerpo = opciones.body;
    }
    const ruta = rutas.find(([mm, re]) => mm === metodo && re.test(url.pathname));
    let estado = 404;
    let respuesta = { error: "sin mock" };
    let nombre = null;
    if (ruta) {
      nombre = ruta[2];
      [estado, respuesta] = await ruta[3](url.pathname.match(ruta[1]), url, cuerpo, encabezados(opciones.headers));
    }
    registro.push({ metodo, ruta: url.pathname, query: url.search, nombre, patron: ruta ? ruta[1].source : null, cuerpoEnviado: cuerpo, estado, respuesta, mock: !!ruta });
    await dormir((esc.demoras && nombre && esc.demoras[nombre]) ?? 120);
    return new Response(JSON.stringify(respuesta), { status: estado, headers: { "Content-Type": "application/json" } });
  };
  Object.defineProperty(window, "fetch", { get: () => mockFetch, set: () => {}, configurable: true });
}

class Cdp {
  static async conectar() {
    const paginas = await (await fetch("http://127.0.0.1:9222/json")).json();
    const pagina = paginas.find((p) => p.type === "page");
    if (!pagina) throw new Error("sin página CDP");
    const c = new Cdp();
    c.ws = new WebSocket(pagina.webSocketDebuggerUrl);
    c.id = 0;
    c.pendientes = new Map();
    c.ws.addEventListener("message", (m) => {
      const d = JSON.parse(m.data);
      if (d.id && c.pendientes.has(d.id)) {
        const [ok, ko] = c.pendientes.get(d.id);
        c.pendientes.delete(d.id);
        d.error ? ko(new Error(JSON.stringify(d.error))) : ok(d.result);
      }
    });
    await new Promise((r) => c.ws.addEventListener("open", r));
    return c;
  }
  enviar(method, params = {}) {
    const id = ++this.id;
    this.ws.send(JSON.stringify({ id, method, params }));
    return new Promise((ok, ko) => this.pendientes.set(id, [ok, ko]));
  }
  async evaluar(expresion) {
    const r = await this.enviar("Runtime.evaluate", { expression: expresion, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
    return r.result.value;
  }
}

const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const capturas = [];
const indice = [];
const registros = [];
let c;
let guionId = null;

async function cargar(escenario, hash = "#/") {
  if (c && guionId) await c.enviar("Page.removeScriptToEvaluateOnNewDocument", { identifier: guionId });
  if (c) await recolectar();
  const fuente = `(${enPagina.toString()})(${JSON.stringify({ API, datos, escenario })})`;
  guionId = (await c.enviar("Page.addScriptToEvaluateOnNewDocument", { source: fuente })).identifier;
  await c.enviar("Page.navigate", { url: `http://localhost:5173/?carga=${Date.now()}${hash}` });
  await dormir(800);
  await esperar(`document.readyState === "complete" && !!window.__MOCK_LOG__ && document.querySelector("#root")?.children.length > 0`);
  await dormir(1200);
}

async function recolectar() {
  try {
    const r = await c.evaluar(`JSON.stringify({fetch: window.__MOCK_LOG__ || [], ipc: window.__IPC_LOG__ || []})`);
    const d = JSON.parse(r);
    registros.push(d);
    await c.evaluar(`window.__MOCK_LOG__ && (window.__MOCK_LOG__.length = 0, window.__IPC_LOG__.length = 0)`);
  } catch {}
}

async function esperar(cond, ms = 15000) {
  const fin = Date.now() + ms;
  while (Date.now() < fin) {
    try {
      if (await c.evaluar(`!!(${cond})`)) return true;
    } catch {}
    await dormir(150);
  }
  throw new Error("timeout esperando: " + cond);
}

const hayTexto = (t, sel = "body") =>
  `[...document.querySelectorAll(${JSON.stringify(sel)})].some(e => e.innerText && e.innerText.includes(${JSON.stringify(t)}))`;

const VK = { Enter: 13, Escape: 27, ArrowDown: 40, ArrowUp: 38, Tab: 9, Backspace: 8 };

async function tecla(k) {
  const f = /^F(\d+)$/.exec(k);
  let p;
  if (f) p = { key: k, code: k, windowsVirtualKeyCode: 111 + Number(f[1]) };
  else if (/^\d$/.test(k)) p = { key: k, code: "Digit" + k, windowsVirtualKeyCode: 48 + Number(k), text: k };
  else p = { key: k, code: k, windowsVirtualKeyCode: VK[k], ...(k === "Enter" ? { text: "\r" } : {}) };
  await c.enviar("Input.dispatchKeyEvent", { type: p.text ? "keyDown" : "rawKeyDown", ...p });
  await c.enviar("Input.dispatchKeyEvent", { type: "keyUp", ...p, text: undefined });
  await dormir(250);
}

async function escribir(t) {
  await c.enviar("Input.insertText", { text: t });
  await dormir(250);
}

async function clicEn(x, y) {
  await c.enviar("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
  await c.enviar("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: "left", clickCount: 1 });
  await c.enviar("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: 1 });
  await dormir(350);
}

async function clic(texto, sel = "button, [role=menuitem], [role=option], [role=tab], a, [role=combobox]", n = 0) {
  const r = await c.evaluar(`(() => {
    const t = ${JSON.stringify(texto)};
    const els = [...document.querySelectorAll(${JSON.stringify(sel)})].filter(e => {
      const r = e.getBoundingClientRect();
      return r.width > 0 && r.height > 0 && ((e.innerText || "").trim().includes(t) || e.getAttribute("aria-label") === t || e.getAttribute("title") === t || e.getAttribute("placeholder") === t);
    });
    const e = els.at(${n});
    if (!e) return null;
    e.scrollIntoView({block: "nearest"});
    const r = e.getBoundingClientRect();
    return [r.x + r.width / 2, r.y + r.height / 2];
  })()`);
  if (!r) throw new Error(`no encontré "${texto}" en ${sel}`);
  await clicEn(r[0], r[1]);
}

async function enfocar(sel) {
  await c.evaluar(`(() => { const e = document.querySelector(${JSON.stringify(sel)}); e.focus(); e.select && e.select(); })()`);
  await dormir(150);
}

async function captura(archivo, descripcion, como, espera = 450) {
  await dormir(espera);
  const r = await c.enviar("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: ANCHO, height: ALTO, scale: 1 }, captureBeyondViewport: false });
  fs.writeFileSync(path.join(SALIDA, archivo), Buffer.from(r.data, "base64"));
  capturas.push(archivo);
  indice.push({ archivo, descripcion, como });
  console.log("captura", archivo);
}

async function limpiarToasts() {
  await c.evaluar(`document.querySelectorAll("[data-sonner-toast]").forEach(t => t.remove())`).catch(() => {});
  await dormir(100);
}

async function entrar(indiceVendedor = 1, clave = "1234") {
  await c.evaluar(`location.hash = "#/user-login"`);
  await esperar(hayTexto(datos.vendedores[indiceVendedor - 1].nombre));
  await tecla(String(indiceVendedor));
  await esperar(`document.querySelector('input[type=password]')`);
  await escribir(clave);
  await tecla("Enter");
  await esperar(`location.hash.includes("/cart")`);
  await esperar(`document.querySelector('input[placeholder^="Buscar productos"]')`);
  await dormir(1500);
  await limpiarToasts();
}

const negocioAfip = {
  afipHabilitado: true,
  facturacionHabilitada: true,
  cancelacionHabilitada: true,
  sistemaPago: "pago-exacto",
  popupListaPreciosEnabled: true,
};

const bloques = {
  async admin() {
    await cargar({ adminConfigurado: false });
    await esperar(`location.hash.includes("admin-login")`);
    await captura("01-admin-login.png", "Configuración inicial (login de administrador) vacía", "Primer arranque sin adminData guardado: la ruta / redirige a #/admin-login");
    await enfocar("#email");
    await escribir("admin@lahuerta.test");
    await enfocar("#password");
    await escribir("clave-incorrecta");
    await c.evaluar(`__ESCENARIO__.loginFalla = true`);
    await clic("Configurar Negocio");
    await esperar(hayTexto("Credenciales inválidas"));
    await captura("02-admin-login-error.png", "Login de administrador con error (alerta roja en el formulario + toast de error arriba)", "Email y contraseña incorrectos → botón Configurar Negocio");
    await limpiarToasts();
    await c.evaluar(`__ESCENARIO__.loginFalla = false; __ESCENARIO__.demoras = { login: 60000 }`);
    await clic("Configurar Negocio");
    await esperar(hayTexto("Configurando..."));
    await captura("03-admin-login-cargando.png", "Login de administrador procesando (botón con spinner 'Configurando...', campos deshabilitados)", "Botón Configurar Negocio mientras espera la respuesta de /api/login", 300);
    await cargar({ adminConfigurado: false });
    await esperar(`location.hash.includes("admin-login")`);
    await enfocar("#email");
    await escribir("admin@lahuerta.test");
    await enfocar("#password");
    await escribir("1234");
    await clic("Recordar credenciales", "label");
    await clic("Configurar Negocio");
    await esperar(`location.hash.includes("user-login")`);
    await esperar(hayTexto("Lucía Fernández"));
    await captura("04-admin-login-exito.png", "Tras configurar el negocio: selector de vendedores con toast verde 'Configuración guardada'", "Login de administrador correcto → navega a #/user-login", 600);
  },

  async usuarios() {
    await cargar({ adminConfigurado: true, demoras: { vendedores: 60000 } });
    await esperar(`location.hash.includes("user-login")`);
    await captura("05-user-login-cargando.png", "Selector de vendedores cargando (spinner verde centrado)", "Arranque con negocio configurado mientras se piden /api/usuarios/vendedores", 600);
    await cargar({ adminConfigurado: true });
    await esperar(hayTexto("Lucía Fernández"));
    await captura("06-user-login.png", "Selector de vendedores '¿Quién está vendiendo?' con tarjetas numeradas 1-9, banda del negocio y botón Reconfigurar", "Arranque con negocio configurado (ruta / → #/user-login)");
    await tecla("1");
    await esperar(`document.querySelector('input[type=password]')`);
    await captura("07-user-login-contrasena.png", "Diálogo de contraseña del vendedor elegido", "Tecla 1 (o clic en la tarjeta) en el selector de vendedores");
    await escribir("12");
    await tecla("Enter");
    await esperar(hayTexto("al menos 4 caracteres"));
    await captura("08-user-login-contrasena-corta.png", "Diálogo de contraseña con error de validación (menos de 4 caracteres)", "Escribir 2 caracteres y Enter");
    await escribir("34");
    await c.evaluar(`__ESCENARIO__.loginFalla = true`);
    await tecla("Enter");
    await esperar(hayTexto("Credenciales inválidas"));
    await captura("09-user-login-error.png", "Contraseña incorrecta: toast rojo 'Error al iniciar sesión' con el mensaje del servidor", "Contraseña de 4+ caracteres rechazada por /api/login");
    await limpiarToasts();
    await c.evaluar(`__ESCENARIO__.loginFalla = false; __ESCENARIO__.demoras = { login: 60000 }`);
    await clic("Iniciar Sesión");
    await esperar(hayTexto("Iniciando sesión..."));
    await captura("10-user-login-iniciando.png", "Diálogo de contraseña enviando (botón 'Iniciando sesión...' con spinner)", "Enter con contraseña válida mientras responde /api/login", 300);
    await cargar({ adminConfigurado: true });
    await esperar(hayTexto("Lucía Fernández"));
    await clic("Reconfigurar");
    await esperar(hayTexto("Confirmar Reconfiguración"));
    await captura("11-user-login-reconfigurar.png", "Confirmación de reconfiguración del punto de venta", "Botón Reconfigurar (arriba a la derecha) del selector de vendedores");
    await cargar({ adminConfigurado: true, vendedoresFalla: true });
    await esperar(hayTexto("Error al cargar vendedores"));
    await captura("12-user-login-error-vendedores.png", "Selector sin vendedores y toast de error 'Error al cargar vendedores'", "Falla de /api/usuarios/vendedores", 500);
    await cargar({ adminConfigurado: true, rolSesion: "ADMIN" });
    await esperar(hayTexto("Lucía Fernández"));
    await tecla("1");
    await esperar(`document.querySelector('input[type=password]')`);
    await escribir("1234");
    await tecla("Enter");
    await esperar(`location.hash.includes("admin-message")`);
    await captura("13-admin-message.png", "Mensaje para usuarios ADMIN (no pueden vender desde el POS)", "Iniciar sesión con un usuario de rol ADMIN → #/admin-message", 600);
  },

  async carrito() {
    await cargar({ adminConfigurado: true });
    await entrar();
    await captura("14-carrito-vacio.png", "Pantalla principal con carrito vacío: sidebar verde con logo, buscador y atajos F1/F2/F7/F4, pestaña 'Orden 1', botones Órdenes y Cierre de caja, total $0", "Login de vendedor sin AFIP ni facturación (negocio simple)");
    await tecla("F2");
    await esperar(hayTexto("No hay productos en el carrito"));
    await captura("15-toast-error-carrito-vacio.png", "Toast de error 'No hay productos en el carrito'", "F2 (pagar) con el carrito vacío", 500);
    await limpiarToasts();
    await enfocar('input[placeholder^="Buscar productos"]');
    await escribir("ma");
    await esperar(hayTexto("Mandarina"));
    await captura("16-busqueda-resultados.png", "Resultados de búsqueda desplegados bajo el buscador (badge PLU, nombre, precio/unidad; primer resultado resaltado)", "Escribir 'ma' en el buscador del sidebar");
    await tecla("ArrowDown");
    await tecla("ArrowDown");
    await captura("17-busqueda-flechas.png", "Resultados de búsqueda con selección movida con flechas", "Flecha abajo dos veces");
    await tecla("Escape");
    await escribir("zapallo");
    await esperar(hayTexto("Zapallo anco"));
    await tecla("Enter");
    await esperar(hayTexto("Peso actual") + " || " + hayTexto("Agregar al carrito"));
    await captura("18-agregar-producto-kg-balanza.png", "Agregar producto por kilo tomando el peso de la balanza (sin listas de precios)", "Buscar 'zapallo' + Enter", 900);
    await tecla("Escape");
    await escribir("tomate");
    await esperar(hayTexto("Tomate perito"));
    await tecla("Enter");
    await esperar(hayTexto("Mayorista"));
    await captura("19-agregar-producto-con-listas.png", "Agregar producto por kilo con opciones de lista de precios (Precio catálogo / Mayorista / Comercios)", "Buscar 'tomate' + Enter (producto con listas de precios)", 900);
    await tecla("Escape");
    await escribir("palta");
    await esperar(hayTexto("Palta Hass"));
    await tecla("Enter");
    await esperar(`document.querySelector('[role=dialog] input')`);
    await escribir("3");
    await captura("20-agregar-producto-unidad.png", "Agregar producto por unidad con cantidad escrita", "Buscar 'palta' + Enter y escribir 3", 700);
    await tecla("Escape");
  },

  async pesoManual() {
    await cargar({ adminConfigurado: true, pesoManual: true });
    await entrar();
    await enfocar('input[placeholder^="Buscar productos"]');
    await escribir("zapallo");
    await esperar(hayTexto("Zapallo anco"));
    await tecla("Enter");
    await esperar(`document.querySelector('[role=dialog]')`);
    await dormir(800);
    await escribir("1750");
    await captura("21-agregar-producto-peso-manual.png", "Agregar producto por kilo con peso manual en gramos (usuario con pesoManualEnabled)", "Vendedor con permiso de peso manual: buscar 'zapallo' + Enter y escribir 1750", 500);
  },

  async pagos() {
    await cargar({ adminConfigurado: true, localStorage: { cartScreens: carritoSembrado } });
    await entrar();
    await captura("22-carrito-con-items.png", "Carrito con varios ítems (por kilo y por unidad), dos pestañas de órdenes y total con botones Cancelar / Pagar", "Cargar productos al carrito (aquí sembrados en cartScreens)");
    await clic("Orden 2");
    await captura("23-pestanas-carritos.png", "Segunda pestaña de orden activa", "Clic en la pestaña 'Orden 2'");
    await clic("Orden 1");
    await tecla("F1");
    await esperar(`document.querySelector('[role=dialog], [role=alertdialog]')`);
    await captura("24-cancelar-orden.png", "Confirmación para cancelar el carrito", "F1 con productos en el carrito");
    await tecla("Escape");
    await tecla("F2");
    await esperar(`document.querySelector('[role=dialog]')`);
    await captura("25-pago-metodos.png", "Diálogo de pago: Transferencia (1, abre el QR de Mercado Pago), Tarjeta (2), Efectivo (3), Pago Mixto (4)", "F2 o botón Pagar con productos en el carrito");
    await tecla("3");
    await esperar(hayTexto("Confirmar"));
    await captura("26-pago-efectivo-redondeo.png", "Pago en efectivo con redondeo a $50 (monto original tachado vs. monto a cobrar, botón 'Aplicar D (F5)' para el descuento por efectivo)", "En el diálogo de pago, tecla 3 (sistemaPago = redondeo)", 600);
    await clic("Aplicar D");
    await dormir(600);
    await captura("27-pago-efectivo-con-descuento.png", "Pago en efectivo con el descuento por efectivo (10%) aplicado", "Botón 'Aplicar D (F5)' en el diálogo de efectivo", 500);
    await tecla("Escape");
    await tecla("F2");
    await esperar(`document.querySelector('[role=dialog]')`);
    await tecla("4");
    await esperar(`document.querySelector('input[placeholder="Monto en efectivo"]')`);
    await captura("28-pago-dividido.png", "Pago mixto: monto en efectivo + segundo método", "En el diálogo de pago, tecla 4", 600);
    await enfocar('input[placeholder="Monto en efectivo"]');
    await escribir("10000");
    await captura("29-pago-dividido-con-monto.png", "Pago mixto con $10.000 en efectivo y el resto calculado", "Escribir el monto en efectivo", 400);
    await tecla("Escape");
    await tecla("F2");
    await esperar(`document.querySelector('[role=dialog]')`);
    await tecla("1");
    await esperar(`document.querySelector('[role=dialog] img')`);
    await captura("30-pago-qr-mercadopago.png", "Pago QR de Mercado Pago pendiente (imagen QR, monto, estado y botón para completar manualmente)", "En el diálogo de pago, tecla 1 (mpEnabled = true)", 900);
    await clic("manual");
    await esperar(`document.querySelector('input[type=password]')`);
    await captura("31-pago-qr-manual.png", "Contraseña para completar manualmente un pago QR", "Botón de completar manualmente en el diálogo QR", 500);
    await cargar({ adminConfigurado: true, localStorage: { cartScreens: carritoSembrado } });
    await entrar();
    await tecla("F7");
    await esperar(`document.querySelector('[role=dialog]')`);
    await captura("32-descuento.png", "Diálogo de descuento (porcentaje o monto fijo) sobre el subtotal", "F7 (pagar con descuento)");
    await escribir("10");
    await captura("33-descuento-con-valor.png", "Diálogo de descuento con 10% escrito y el total resultante", "Escribir 10 en el diálogo de descuento", 400);
    await clic("Cantidad Fija");
    await enfocar('[role=dialog] input');
    await escribir("1500");
    await captura("33b-descuento-cantidad-fija.png", "Diálogo de descuento en modo Cantidad Fija con $1.500", "Pestaña Cantidad Fija del diálogo de descuento y escribir 1500", 400);
    await clic("Porcentaje");
    await enfocar('[role=dialog] input');
    await escribir("10");
    await tecla("Enter");
    await esperar(`document.querySelector('[role=dialog]')`);
    await captura("34-pago-con-descuento.png", "Diálogo de pago mostrando subtotal, descuento y total", "Enter en el diálogo de descuento", 600);
    await tecla("2");
    await esperar(hayTexto("Orden completada exitosamente"));
    await esperar(`!document.querySelector("[role=dialog]")`);
    await captura("35-toast-exito-orden.png", "Toast verde 'Orden completada exitosamente' y carrito vacío después del pago con tarjeta", "Tecla 2 (tarjeta) en el diálogo de pago (impresión interceptada)", 100);
  },

  async ordenesYCierre() {
    await cargar({ adminConfigurado: true });
    await entrar();
    await clic("Órdenes");
    await esperar(hayTexto("Reimprimir"));
    await captura("36-ordenes-recientes.png", "Órdenes del día del vendedor (tabla con hora, método, descuento, total y reimpresión)", "Botón Órdenes (arriba a la derecha)", 800);
    await tecla("Escape");
    await clic("Cierre de caja");
    await esperar(hayTexto("Seleccionar período de cierre"));
    await captura("37-cierre-caja.png", "Selección del período de cierre de caja: Mañana (1), Tarde (2), Todo el día (3), Cierre Manual (4)", "Botón Cierre de caja (usuario con cierreDeCajaEnabled)");
    await c.evaluar(`__ESCENARIO__.demoras = { cierre: 60000 }`);
    await clic("Mañana (1)");
    await dormir(400);
    await captura("38-cierre-caja-procesando.png", "Cierre de caja procesando (spinners en los botones)", "Clic en Mañana mientras responde /api/cierres", 200);
    await cargar({ adminConfigurado: true });
    await entrar();
    await clic("Cierre de caja");
    await esperar(hayTexto("Seleccionar período de cierre"));
    await clic("Mañana (1)");
    await esperar(`document.querySelectorAll("[data-sonner-toast]").length > 0`);
    await captura("39-cierre-caja-resultado.png", "Resultado del cierre: toast 'Cierre de mañana realizado correctamente' (el ticket se imprime aparte)", "Clic en Mañana (1) con /api/cierres respondiendo bien (impresión interceptada)", 700);
  },

  async cierrePersonalizado() {
    await cargar({ adminConfigurado: true });
    await entrar();
    await clic("Cierre de caja");
    await esperar(hayTexto("Seleccionar período de cierre"));
    await clic("Cierre Manual");
    await dormir(800);
    await captura("40-cierre-manual.png", "Cierre manual: fechas/horas de inicio y fin, vendedor y sucursal", "En el diálogo de cierre, botón Cierre Manual (4)", 500);
  },

  async actualizacion() {
    await cargar({ adminConfigurado: true, actualizacion: true });
    await entrar();
    await esperar(hayTexto("Nueva versión disponible") + " || " + hayTexto("4.1.0"), 20000);
    await captura("41-actualizacion-disponible.png", "Aviso de actualización disponible (toast + tarjeta de UpdateNotification)", "5 s después de arrancar, checkForUpdates devuelve available = true", 600);
    await clic("Descargar", "button", -1);
    await dormir(600);
    await captura("42-actualizacion-descargando.png", "Actualización descargándose", "Botón Descargar del aviso", 200);
    await esperar(hayTexto("Actualización descargada") + " || " + hayTexto("Reiniciar"), 15000);
    await captura("43-actualizacion-descargada.png", "Actualización descargada, lista para reiniciar", "Fin de downloadUpdate", 600);
  },

  async afip() {
    await cargar({ adminConfigurado: true, negocio: negocioAfip, localStorage: { cartScreens: carritoSembrado } });
    await entrar();
    await captura("44-carrito-afip.png", "Carrito con AFIP y facturación habilitados: atajos F1, F2 pagar con AFIP, F7, F3 pagar orden, F8, F6 cuenta corriente, F4; botón Cuenta Corriente", "Negocio con afipHabilitado + facturacionHabilitada");
    await tecla("F2");
    await esperar(`document.querySelector('[role=dialog]')`);
    await captura("45-pago-afip.png", "Diálogo de pago en modo AFIP (factura electrónica)", "F2 con AFIP habilitado");
    await tecla("3");
    await esperar(`document.querySelector('[role=dialog] input')`);
    await captura("46-pago-afip-efectivo-exacto.png", "Pago exacto en efectivo (AFIP): campo 'paga con' y vuelto", "Tecla 3 en el pago AFIP (sistemaPago = pago-exacto)", 600);
    await escribir("50000");
    await captura("47-pago-afip-efectivo-vuelto.png", "Pago exacto con monto entregado y vuelto calculado", "Escribir 50000", 400);
    await tecla("Escape");
    await tecla("F2");
    await esperar(`document.querySelector('[role=dialog]')`);
    await tecla("4");
    await esperar(`document.querySelector('input[placeholder="Monto en efectivo"]')`);
    await captura("48-pago-afip-dividido.png", "Pago mixto con factura AFIP", "Tecla 4 en el pago AFIP", 600);
    await enfocar('input[placeholder="Monto en efectivo"]');
    await escribir("15000");
    await captura("49-pago-afip-dividido-con-monto.png", "Pago mixto AFIP con $15.000 en efectivo", "Escribir el monto en efectivo", 400);
    await tecla("Escape");
    await tecla("F2");
    await esperar(`document.querySelector('[role=dialog]')`);
    await tecla("1");
    await esperar(`document.querySelector('[role=dialog] img')`);
    await captura("50-pago-afip-qr.png", "Pago QR con factura AFIP", "Tecla 1 en el pago AFIP", 900);
    await cargar({ adminConfigurado: true, negocio: negocioAfip, localStorage: { cartScreens: carritoSembrado } });
    await entrar();
    await tecla("F3");
    await esperar(`document.querySelector('[role=dialog]')`);
    await tecla("3");
    await esperar(`document.querySelector('[role=dialog] input')`);
    await captura("51-pago-efectivo-exacto.png", "Pago exacto en efectivo sin factura (orden común)", "F3 (pagar orden) y tecla 3, con sistemaPago = pago-exacto", 600);
    await tecla("Escape");
    await tecla("F7");
    await esperar(`document.querySelector('[role=dialog]')`);
    await escribir("5");
    await tecla("Enter");
    await esperar(`document.querySelector('[role=dialog]')`);
    await captura("52-pago-afip-con-descuento.png", "Pago AFIP con descuento del 5%", "F7 con AFIP, escribir 5 y Enter", 600);
    await tecla("2");
    await esperar(hayTexto("Factura AFIP creada"), 15000);
    await captura("53-pago-afip-procesando.png", "Pago AFIP en curso: método Tarjeta con spinner y toast 'Factura AFIP creada exitosamente' con el CAE", "Tecla 2 en el pago AFIP (crear-afip mockeado, impresión interceptada)", 300);
    await esperar(`!document.querySelector("[role=dialog]")`, 15000);
    await captura("53b-pago-afip-terminado.png", "Fin del pago AFIP: diálogo cerrado, carrito vaciado y toasts finales", "Esperar a que termine el pago AFIP", 100);
  },

  async cancelacion() {
    await cargar({ adminConfigurado: true, negocio: negocioAfip, localStorage: { cartScreens: carritoSembrado } });
    await entrar();
    await tecla("F1");
    await esperar(`document.querySelector('[role=dialog], [role=alertdialog]')`);
    await clic("Sí", "button");
    await esperar(hayTexto("otivo"));
    await captura("54-cancelacion-motivo.png", "Cancelación controlada: motivo obligatorio y detalle de lo que se cancela", "F1 y confirmar, con cancelacionHabilitada = true", 600);
  },

  async escaneo() {
    await cargar({ adminConfigurado: true, negocio: negocioAfip });
    await entrar();
    await enfocar('input[placeholder^="Buscar productos"]');
    await escribir("7791234500026");
    await esperar(hayTexto("Mayorista"));
    await captura("55-escanear-lista-precios.png", "Elegir lista de precios para un producto escaneado (popupListaPreciosEnabled)", "Escanear el código de barras del Ananá (7791234500026)", 800);
  },

  async facturaDesdeCarrito() {
    await cargar({ adminConfigurado: true, negocio: negocioAfip, localStorage: { cartScreens: carritoSembrado } });
    await entrar();
    await tecla("F6");
    await esperar(hayTexto("Tomate perito", "[role=dialog]"));
    await captura("56-factura-form-desde-carrito.png", "Formulario de factura (cuenta corriente) precargado con los productos del carrito", "F6 con facturación habilitada", 1000);
    await clic("cliente");
    await esperar(hayTexto("El Mesón"));
    await captura("57-selector-cliente.png", "Selector de cliente con búsqueda y paginación", "Botón de seleccionar cliente en el formulario de factura", 700);
    await clic("El Mesón", "[role=dialog] *:not(:has(*))");
    await dormir(600);
    await captura("58-factura-form-con-cliente.png", "Formulario de factura con cliente elegido", "Elegir 'Restaurante El Mesón' en el selector", 500);
    await c.evaluar(`document.querySelector("[role=dialog]").scrollTop = 99999`);
    await captura("56b-factura-form-totales.png", "Parte inferior del formulario de factura: ítems, totales (subtotal, IVA, total), observaciones y botones", "Desplazar hacia abajo el formulario de factura", 500);
    await c.evaluar(`document.querySelector("[role=dialog]").scrollTop = 0`);
    await clic("Agregar Producto");
    await esperar(hayTexto("Seleccionar Producto") + " && " + hayTexto("Cebolla"));
    await captura("59-selector-producto.png", "Selector de producto para la factura", "Botón Agregar producto en el formulario de factura", 700);
    await clic("Cebolla", "[role=dialog] *:not(:has(*))");
    await esperar(hayTexto("antidad"));
    await captura("60-cantidad-producto.png", "Diálogo de cantidad del producto elegido", "Elegir 'Cebolla' en el selector de producto", 700);
  },

  async facturas() {
    await cargar({ adminConfigurado: true, negocio: negocioAfip, demoras: { facturas: 60000 } });
    await entrar();
    await clic("Cuenta Corriente");
    await esperar(`location.hash.includes("/facturas")`);
    await captura("61-facturas-cargando.png", "Cuenta corriente cargando", "Botón Cuenta Corriente mientras responde /api/facturas", 600);
    await cargar({ adminConfigurado: true, negocio: negocioAfip });
    await entrar();
    await clic("Cuenta Corriente");
    await esperar(hayTexto("R-0002-00000148"));
    await captura("62-facturas-lista.png", "Cuenta corriente: filtros y tarjetas de facturas/remitos con estado, total, pagado y saldo", "Botón Cuenta Corriente del carrito (o #/facturas)", 600);
    await clic("Todos los estados");
    await dormir(400);
    await captura("63-facturas-filtro-estado.png", "Filtro de estado desplegado", "Clic en el selector de estado", 300);
    await tecla("Escape");
    await clic("Pagos", "button", 0);
    await esperar(hayTexto("Historial"));
    await captura("64-historial-pagos.png", "Historial de pagos de una factura", "Botón Pagos/Ver pagos de la primera tarjeta", 600);
    await tecla("Escape");
    await clic("Agregar", "button", 0);
    await esperar(`document.querySelector('[role=dialog] input')`);
    await captura("65-agregar-pago.png", "Agregar pago a una factura en cuenta corriente", "Botón Agregar pago de la primera tarjeta", 600);
    await tecla("Escape");
    await clic("Editar", "button", 0);
    await esperar(`document.querySelector('[role=dialog], .fixed')`);
    await captura("66-factura-editar.png", "Modal de edición de factura", "Botón Editar de la primera tarjeta", 900);
    await c.evaluar(`location.hash = "#/facturas/f102"`);
    await esperar(hayTexto("A-0002-00000013"));
    await captura("67-factura-detalle.png", "Detalle de factura (A) con cliente, ítems, totales, CAE y pagos", "Botón Ver de una tarjeta (navega a #/facturas/:id)", 800);
    await c.evaluar(`location.hash = "#/facturas/f101"`);
    await esperar(hayTexto("R-0002-00000148"));
    await captura("68-factura-detalle-remito.png", "Detalle de un remito con saldo pendiente", "Ver de un remito con pagos parciales", 800);
  },

  async facturasVacio() {
    await cargar({ adminConfigurado: true, negocio: negocioAfip, sinFacturas: true });
    await entrar();
    await clic("Cuenta Corriente");
    await esperar(hayTexto("No hay facturas registradas"));
    await captura("69-facturas-vacio.png", "Cuenta corriente sin facturas", "Cuenta Corriente cuando /api/facturas devuelve una lista vacía", 500);
    await clic("Crear primera factura");
    await dormir(1200);
    await captura("70-factura-form-nueva.png", "Formulario de factura nueva (vacío)", "Botón Crear primera factura", 500);
  },

  async menuUsuario() {
    await cargar({ adminConfigurado: true });
    await entrar();
    const r = await c.evaluar(`(() => { const b = [...document.querySelectorAll("button")].find(b => b.querySelector("svg.lucide-user")); if (!b) return null; const r = b.getBoundingClientRect(); return [r.x + r.width/2, r.y + r.height/2]; })()`);
    if (!r) return console.log("menú de usuario no montado (HeaderActions lo tiene comentado): se conservan las capturas 71-73 anteriores");
    await clicEn(r[0], r[1]);
    await esperar(`document.querySelector('[role=menu]')`);
    await captura("71-menu-usuario.png", "Menú de usuario abierto (nombre, email, Impresión, Balanza, Cerrar sesión)", "Botón con ícono de usuario en la cabecera (hoy comentado en HeaderActions: se habilitó temporalmente para la captura)", 400);
    await clic("Impresión", "[role=menuitem]");
    await esperar(hayTexto("TP806L"));
    await captura("72-impresion.png", "Configuración de impresión (impresora, ancho de papel, logo, ticket de prueba)", "Menú de usuario → Impresión", 600);
    await tecla("Escape");
    await clicEn(r[0], r[1]);
    await esperar(`document.querySelector('[role=menu]')`);
    await clic("Balanza", "[role=menuitem]");
    await esperar(hayTexto("COM3"));
    await captura("73-balanza.png", "Configuración de balanza (modo, puerto, baudios, formato, estado y peso leído)", "Menú de usuario → Balanza", 600);
  },
};

async function principal() {
  c = await Cdp.conectar();
  await c.enviar("Page.enable");
  await c.enviar("Runtime.enable");
  await c.enviar("Emulation.setDeviceMetricsOverride", { width: ANCHO, height: ALTO, deviceScaleFactor: 1, mobile: false });
  const pedidos = process.argv.slice(2);
  const nombres = pedidos.length ? pedidos : Object.keys(bloques);
  const fallos = [];
  for (const n of nombres) {
    try {
      await bloques[n]();
    } catch (e) {
      fallos.push({ bloque: n, error: e.message });
      console.log("FALLO", n, e.message);
      const r = await c.enviar("Page.captureScreenshot", { format: "png" }).catch(() => null);
      if (r) fs.writeFileSync(path.join(SALIDA, `zz-fallo-${n}.png`), Buffer.from(r.data, "base64"));
    }
  }
  await recolectar();
  if (guionId) await c.enviar("Page.removeScriptToEvaluateOnNewDocument", { identifier: guionId });
  await c.enviar("Emulation.clearDeviceMetricsOverride");
  escribirEntregables(fallos);
  console.log(JSON.stringify({ capturas: capturas.length, fallos }, null, 1));
  c.ws.close();
}

const legible = (fuente) =>
  fuente
    .replace(/^\^|\$$/g, "")
    .replace(/\\\//g, "/")
    .replace(/\(\\d\+\)|\(\\w\+\)|\(\[\\w-\]\+\)/g, ":id");

function escribirEntregables(fallos) {
  const endpoints = new Map();
  const ipc = new Map();
  for (const r of registros) {
    for (const f of r.fetch) {
      const ruta = f.patron ? legible(f.patron) : f.ruta;
      const clave = `${f.metodo} ${ruta}`;
      if (!endpoints.has(clave)) endpoints.set(clave, { metodo: f.metodo, ruta, ejemplos: [] });
      const e = endpoints.get(clave);
      if (!e.ejemplos.some((x) => x.estado === f.estado && JSON.stringify(x.respuesta) === JSON.stringify(f.respuesta)) && e.ejemplos.length < 3)
        e.ejemplos.push({ rutaReal: f.ruta + (f.query || ""), cuerpoEnviado: f.cuerpoEnviado, estado: f.estado, respuesta: f.respuesta });
    }
    for (const i of r.ipc) {
      if (!ipc.has(i.canal)) ipc.set(i.canal, { canal: i.canal, ejemploArgs: i.args, respuesta: i.respuesta });
    }
  }
  const leer = (archivo, defecto) => (fs.existsSync(path.join(SALIDA, archivo)) ? fs.readFileSync(path.join(SALIDA, archivo), "utf8") : defecto);
  const previoMock = JSON.parse(leer("mock.json", '{"endpoints":[],"ipc":[]}'));
  for (const e of previoMock.endpoints) if (!endpoints.has(`${e.metodo} ${e.ruta}`)) endpoints.set(`${e.metodo} ${e.ruta}`, e);
  for (const i of previoMock.ipc) if (!ipc.has(i.canal)) ipc.set(i.canal, i);
  const lineasCaptura = new Map(
    leer("INDICE.md", "")
      .split("\n")
      .map((l) => [/^- `([^`]+\.png)` → /.exec(l)?.[1], l])
      .filter(([a]) => a && fs.existsSync(path.join(SALIDA, a)))
  );
  for (const i of indice) lineasCaptura.set(i.archivo, `- \`${i.archivo}\` → ${i.descripcion}. Cómo se llega: ${i.como}.`);
  const { qrImagen, ...datosSinQr } = datos;
  fs.writeFileSync(
    path.join(SALIDA, "mock.json"),
    JSON.stringify(
      {
        api: API,
        generado: new Date().toISOString(),
        nota: "Respuestas falsas servidas a la UI React durante las capturas. Las fechas son relativas al momento de la corrida. qrImageUrl es un QR de mentira en SVG (data URL).",
        escenarios: {
          simple: { negocio: "negocioBase", usuario: { pesoManualEnabled: false, cierreDeCajaEnabled: true } },
          afip: { negocio: { ...negocioBase, ...negocioAfip } },
          pesoManual: { usuario: { pesoManualEnabled: true } },
          carritoSembrado: JSON.parse(carritoSembrado),
        },
        datos: datosSinQr,
        endpoints: [...endpoints.values()],
        ipc: [...ipc.values()],
      },
      null,
      2
    )
  );
  const lineas = [
    "# Capturas de referencia de la UI del POS (React)",
    "",
    `Capturas PNG de ${ANCHO}x${ALTO} de la UI React de verdu-tauri, con la API y el IPC de Tauri simulados (\`mock.json\`). Se generan con \`node capturar.mjs\` contra la app corriendo en modo dev con \`--remote-debugging-port=9222\`.`,
    "",
    "## Capturas",
    "",
    ...[...lineasCaptura.entries()].sort(([a], [b]) => a.localeCompare(b, "es", { numeric: true })).map(([, l]) => l),
    "",
    "## Notas",
    "",
    ...NOTAS.map((n) => `- ${n}`),
    ...(fallos.length ? ["", "## Fallaron en la última corrida", "", ...fallos.map((f) => `- ${f.bloque}: ${f.error}`)] : []),
    "",
  ];
  fs.writeFileSync(path.join(SALIDA, "INDICE.md"), lineas.join("\n"));
}

const NOTAS = [
  "El menú de usuario (`UserMenu`, con Impresión y Balanza) está comentado en `src/components/shopping-cart/HeaderActions.tsx`, así que en la UI actual no se puede abrir. Para las capturas 71-73 se descomentó temporalmente esa línea y se volvió a comentar al terminar.",
  "`CloseResultDialog` (resultado del cierre de caja en un diálogo) no está montado en ningún lado: el resultado real del cierre es el toast de la captura 39 y el ticket impreso.",
  "No se capturaron por no estar montados en ninguna ruta: `ClienteSelector` y `ProductoSelector` (las versiones sin diálogo; las usadas son `ClienteSelectorDialog` y `ProductoSelectorDialog`), `CancelOrderDialog`, `BackgroundLayout`, `QrStatusWatcher`, `ElectronDebug`, `PrinterDebug` y la página `pages/Login.tsx` (no está en el router).",
  "Algunas confirmaciones usan diálogos nativos del navegador (`window.confirm` al borrar una pestaña de orden o una factura, `alert` en validaciones de Agregar pago y Agregar producto): no son parte de la UI React y no se capturaron.",
  "Los `<select>` nativos (lista de precios en Agregar producto y en Escanear lista) abren un desplegable del sistema operativo que la captura del WebView no incluye; se muestran cerrados.",
  "A 1200 px de ancho, con dos pestañas de órdenes y los tres botones de la cabecera (Órdenes, Cuenta Corriente, Cierre de caja) la cabecera desborda y aparece scroll horizontal (capturas 44 en adelante); con muchos ítems y total grande, Pagar baja debajo de Cancelar (22). Es el comportamiento real de la UI.",
  "El método 1 del diálogo de pago dice 'Transferencia (1)' aunque abre el flujo de QR de Mercado Pago cuando `mpEnabled` es true.",
  "Variantes: negocio simple (sin AFIP ni facturación, `sistemaPago` = redondeo) en 14-43; negocio con AFIP + facturación + cancelación controlada + popup de listas de precios y `sistemaPago` = pago-exacto en 44-70; vendedor con peso manual en 21.",
  "Impresión, balanza, actualizaciones y almacenamiento (`window.printer`, `pesoReader`, `autoUpdater`, `electronStore`, `electron.ipcRenderer`) se reemplazaron en memoria: no se imprimió nada ni se tocó la configuración real; localStorage y sessionStorage también fueron en memoria.",
];

await principal();
