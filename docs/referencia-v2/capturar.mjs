import { spawn } from "node:child_process";
import http from "node:http";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const OUT = path.dirname(fileURLToPath(import.meta.url));
const PROJECT = path.resolve(OUT, "../../design/redise-o-profesional-de-app/project");
const PAGE = "Andex POS.dc.html";
const EDGE = process.env.EDGE || "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe";
const CDP_PORT = 9333;
const SIZES = [[1920, 1080], [1366, 768]];
const ONLY = process.argv[2] ? new RegExp(process.argv[2]) : null;

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".png": "image/png", ".css": "text/css", ".svg": "image/svg+xml", ".json": "application/json" };

const HELPERS = `(() => {
  const el = document.querySelector('[data-screen-label]');
  if (!el || !customElements.get('sl-icon')) return false;
  const key = Object.keys(el).find(k => k.startsWith('__reactFiber'));
  let f = el[key];
  while (f && !(f.stateNode && f.stateNode.logic && typeof f.stateNode.logic.renderVals === 'function')) f = f.return;
  if (!f) return false;
  const c = window.__c = f.stateNode.logic;
  window.__set = p => new Promise(r => c.setState(p, r));
  window.__flags = o => { const base = Object.getPrototypeOf(c).flags; c.flags = function () { return { ...base.call(this), ...o }; }; return new Promise(r => c.__host.forceUpdate(r)); };
  window.__p = q => { const s = c.state.q; c.state.q = q; const r = c.results(); c.state.q = s; return r[0]; };
  window.__add = (q, qty, li) => { const p = __p(q); c.addItem(p, qty, li == null ? null : p.listas[li]); };
  window.__cart = () => { __add('tomate redondo', 1.245); __add('papa', 2.1); __add('lechuga', 2); __add('banana', 0.85, 0); __add('huevos', 1); __add('palta', 3); return new Promise(r => setTimeout(r, 50)); };
  window.__still = (w = 1245) => { clearInterval(c._w); return __set({ weight: w, wStable: true }); };
  window.__quiet = () => { clearTimeout(c._x); clearTimeout(c._qr); clearTimeout(c._p); };
  window.__fact = id => c.state.facturas.find(f => f.id === id);
  window.__vals = () => c.renderVals();
  return true;
})()`;

const S = [];
const add = (id, name, desc, js, wait = 500) => S.push({ id, name, desc, js, wait });

add("01", "config-inicial", "Configuración inicial vacía. pantallaInicial=admin (screen \"admin\").", `await __set({screen:'admin',biz:null,user:null});`);
add("02", "config-inicial-error", "Configuración inicial con error de validación: email cargado, contraseña vacía y submitAdmin() (botón Configurar negocio / Enter).", `await __set({screen:'admin',biz:null,user:null,ad:{...__c.state.ad,email:'martin@donjose.com.ar'}}); __c.submitAdmin();`);
add("03", "config-inicial-cargando", "Configuración inicial enviando: ad.loading=true (estado que pone submitAdmin mientras espera).", `await __set({screen:'admin',biz:null,user:null,ad:{...__c.state.ad,email:'martin@donjose.com.ar',password:'demo1234',loading:true}});`);
add("04", "quien-vende", "¿Quién está vendiendo? Grilla de vendedores (pantallaInicial=usuarios).", `await __set({screen:'users',user:null});`);
add("05", "quien-vende-cargando", "¿Quién está vendiendo? cargando: skeleton de vendedores (vLoading=true, como queda tras configurar el negocio).", `await __set({screen:'users',user:null,vLoading:true});`);
add("06", "modal-contrasena-vendedor", "Modal Ingresá tu contraseña: click en la tarjeta de Lucía Fernández (pickVendor) en Quién está vendiendo.", `await __set({screen:'users',user:null}); __vals().vendors[0].pick();`);
add("07", "modal-contrasena-vendedor-error", "Modal contraseña con error: 2 caracteres y submitPw() (Enter).", `await __set({screen:'users',user:null}); __vals().vendors[0].pick(); await __set({m:{...__c.state.m,value:'12'}}); __c.submitPw();`);
add("08", "modal-contrasena-vendedor-cargando", "Modal contraseña ingresando: m.loading=true (estado de submitPw mientras valida).", `await __set({screen:'users',user:null}); __vals().vendors[0].pick(); await __set({m:{...__c.state.m,value:'demo',loading:true}});`);
add("09", "modal-confirmar-reconfigurar", "Confirmación ¿Reconfigurar el punto de venta?: openReconfig() desde Quién está vendiendo.", `await __set({screen:'users',user:null}); __vals().openReconfig();`);
add("10", "cuenta-administrador", "Pantalla Esta es una cuenta de administrador: vendedor con rol ADMIN (Martín Álvarez) que ingresa contraseña (screen \"adminMsg\").", `await __set({screen:'adminMsg'});`);
add("11", "pos-orden-vacia", "Punto de venta con la orden vacía (pantallaInicial=pos, afip y facturación on).", ``);
add("12", "pos-con-items", "Punto de venta con 6 productos (kg, unidad y uno con lista Mayorista) cargados con addItem; el último queda resaltado.", `await __cart();`, 1900);
add("13", "pos-buscador-resultados", "Buscador con resultados: se escribe \"to\" en el buscador (state.q).", `await __cart(); await __set({q:'to',sel:0});`, 1900);
add("14", "pos-buscador-segundo-resultado", "Buscador con el segundo resultado elegido con flecha abajo (sel=1).", `await __set({q:'pa',sel:1});`);
add("15", "pos-buscador-sin-resultados", "Buscador sin resultados: se escribe \"xyz\".", `await __set({q:'xyz'});`);
add("16", "pos-menu-usuario", "Menú de usuario abierto (click en el avatar del header: toggleMenu).", `await __cart(); __vals().toggleMenu();`, 1900);
add("17", "pos-varias-pestanas", "Tres órdenes en espera (botón Nueva orden dos veces), la Orden 2 activa con productos.", `await __cart(); __vals().addTab(); await new Promise(r=>setTimeout(r,50)); __add('manzana',1.56); __add('lechuga',1); await new Promise(r=>setTimeout(r,50)); __vals().addTab(); await new Promise(r=>setTimeout(r,50)); await __set({active:1});`, 1900);
add("18", "pos-sin-afip", "Punto de venta con afip=false: F2 Cobrar sin factura y sin acciones F3/F8.", `await __flags({afip:false}); await __cart();`, 1900);
add("19", "pos-sin-facturacion", "Punto de venta con facturacion=false: sin Cuenta corriente en header ni en acciones (F6).", `await __flags({fact:false}); await __cart();`, 1900);
add("20", "pos-sin-afip-sin-facturacion", "Punto de venta con afip=false y facturacion=false.", `await __flags({afip:false,fact:false}); await __cart();`, 1900);
add("21", "pos-cancelacion-controlada", "Punto de venta con cancelacionControlada=true: los ítems no tienen botón de quitar.", `await __flags({ctrl:true}); await __cart();`, 1900);
add("22", "pos-header-angosto", "Header angosto (ventana < 1240 px): botones del header solo con ícono y avatar sin nombre (state.wide=false).", `await __cart(); await __set({wide:false});`, 1900);
add("23", "modal-cancelar-orden", "Confirmación ¿Cancelar la orden? con la lista de productos: F1 / Cancelar orden (cancelSale).", `await __cart(); __c.cancelSale();`, 1900);
add("24", "modal-cancelar-controlada", "Cancelar y registrar con motivo (cancelacionControlada=true): F1 y se escribe el motivo.", `await __flags({ctrl:true}); await __cart(); __c.cancelSale(); await __set({m:{...__c.state.m,reason:'El cliente cambió de opinión'}});`, 1900);
add("25", "modal-cerrar-pestana", "Confirmación ¿Cerrar la Orden 2? al tocar la X de una pestaña con productos (delTab).", `__vals().addTab(); await new Promise(r=>setTimeout(r,50)); await __cart(); __c.delTab(1);`, 1900);
add("26", "modal-agregar-kg", "Agregar producto por kilo (Tomate redondo) con listas de precios y peso estable: Enter sobre el resultado del buscador (openAdd).", `__c.openAdd(__p('tomate redondo')); await __still(1245);`);
add("27", "modal-agregar-kg-leyendo", "Agregar producto por kilo mientras la balanza estabiliza (wStable=false).", `__c.openAdd(__p('banana')); clearInterval(__c._w); await __set({weight:812,wStable:false});`);
add("28", "modal-agregar-kg-manual", "Agregar producto por kilo con Ingresar peso manualmente activado y 1250 g escritos.", `__c.openAdd(__p('papa')); await __still(); await __set({m:{...__c.state.m,manual:true,qty:'1250'}});`);
add("29", "modal-agregar-kg-sin-peso-manual", "Agregar producto por kilo con pesoManual=false (sin el switch) y sin listas de precios (Zanahoria).", `await __flags({manual:false}); __c.openAdd(__p('zanahoria')); await __still(2380);`);
add("30", "modal-agregar-unidad", "Agregar producto por unidad sin listas (Lechuga criolla), 2 unidades.", `__c.openAdd(__p('lechuga')); await __set({m:{...__c.state.m,qty:'2'}});`);
add("31", "modal-agregar-unidad-lista", "Agregar producto por unidad con lista Mayorista elegida (Palta Hass).", `const p=__p('palta'); __c.openAdd(p); await __set({m:{...__c.state.m,qty:'3',lista:p.listas[0]}});`);
add("32", "modal-agregar-error", "Agregar producto por unidad con cantidad vacía y confirmar: error.", `__c.openAdd(__p('ajo')); await __set({m:{...__c.state.m,qty:''}}); __c.confirmAdd();`);
add("33", "modal-descuento", "Aplicar descuento vacío (F7 Cobrar con descuento: openDiscount(true)).", `await __cart(); __c.openDiscount(true);`, 1900);
add("34", "modal-descuento-porcentaje", "Aplicar descuento con preset 10% elegido.", `await __cart(); __c.openDiscount(true); await __set({m:{...__c.state.m,value:'10'}});`, 1900);
add("35", "modal-descuento-monto-fijo", "Aplicar descuento en pestaña Monto fijo con $2.000 (F8 Descuento sin factura: openDiscount(false)).", `await __cart(); __c.openDiscount(false); await __set({m:{tipo:'cantidad',value:'2000'}});`, 1900);
add("36", "modal-metodo-pago-factura", "Método de pago Cobrar con factura AFIP (F2 con afip=true).", `await __cart(); __c.openPay(true,false);`, 1900);
add("37", "modal-metodo-pago-sin-factura", "Método de pago Cobrar orden sin factura (F3).", `await __cart(); __c.openPay(false,false);`, 1900);
add("38", "modal-metodo-pago-con-descuento", "Método de pago después de aplicar 10% de descuento (subtotal tachado).", `await __cart(); __c.openDiscount(true); await __set({m:{...__c.state.m,value:'10'}}); __c.applyDiscount();`, 1900);
add("39", "modal-metodo-pago-procesando", "Método de pago procesando Tarjeta (tecla 2: proc=tarjeta).", `await __cart(); __c.openPay(true,false); await __set({proc:'tarjeta'});`, 1900);
add("40", "modal-efectivo", "Cobro en efectivo con redondeo (tecla 3 en Método de pago).", `await __cart(); __c.openPay(true,false); __c.choosePay('efectivo');`, 1900);
add("41", "modal-efectivo-vuelto", "Cobro en efectivo con billete de $20.000 elegido: muestra el vuelto.", `await __cart(); __c.openPay(true,false); __c.choosePay('efectivo'); await __set({m:{...__c.state.m,recibido:'20000'}});`, 1900);
add("42", "modal-efectivo-descuento", "Cobro en efectivo con Descuento efectivo 10% aplicado (F5).", `await __cart(); __c.openPay(true,false); __c.choosePay('efectivo'); __c.cashDisc();`, 1900);
add("43", "modal-mixto", "Pago mixto con $5.000 en efectivo y el resto con tarjeta (tecla 4).", `await __cart(); __c.openPay(true,false); __c.choosePay('split'); await __set({m:{...__c.state.m,cash:'5000'}});`, 1900);
add("44", "modal-mixto-error", "Pago mixto confirmado sin monto: error de validación.", `await __cart(); __c.openPay(true,false); __c.choosePay('split'); __c.completeSplit();`, 1900);
add("45", "modal-qr-esperando", "Cobro con QR esperando el pago (tecla 1 con mercadoPago=true).", `await __cart(); __c.openPay(true,false); await __set({modal:'qr',proc:null,m:{status:'pend'}});`, 1900);
add("46", "modal-qr-acreditado", "Cobro con QR con el pago acreditado.", `await __cart(); __c.openPay(true,false); await __set({modal:'qr',proc:null,m:{status:'ok'}});`, 1900);
add("47", "modal-venta-completada-factura", "Venta cobrada en efectivo con vuelto, Factura B y CAE, ticket impreso (complete).", `await __cart(); __c.openPay(true,false); __c.choosePay('efectivo'); const r=__c.cashAmounts().rounded; __c.complete('efectivo',{final:r,vuelto:20000-r});`, 1900);
add("48", "modal-venta-completada-imprimiendo", "Venta cobrada sin factura con tarjeta, mientras imprime el ticket.", `await __cart(); __c.openPay(false,false); __c.complete('tarjeta'); __quiet();`, 1900);
add("49", "modal-venta-completada-mixto", "Venta cobrada con pago mixto (efectivo + transferencia).", `await __cart(); __c.openPay(false,false); __c.complete('split',{cash:5000,second:'qr'});`, 1900);
add("50", "modal-ordenes-cargando", "Transacciones del día cargando (botón Órdenes del header: openOrders).", `__c.openOrders(); __quiet();`);
add("51", "modal-ordenes", "Transacciones del día del vendedor con KPIs y tabla.", `__c.openOrders();`, 1300);
add("52", "modal-ordenes-paginacion", "Transacciones del día con 12 órdenes: paginación de 6 por página.", `const extra=__c.state.orders.filter(o=>o.vendedorId===1).map((o,i)=>({...o,id:3000+i,fecha:new Date(o.fecha.getTime()+16200000)})); await __set({orders:[...extra,...__c.state.orders]}); __c.openOrders();`, 1300);
add("53", "modal-ordenes-vacio", "Transacciones del día sin órdenes del vendedor.", `await __set({orders:[]}); __c.openOrders();`, 1300);
add("54", "modal-cierre-caja", "Cierre de caja: elegir período (botón Cierre de caja del header).", `__vals().headerBtns.find(b=>b.label==='Cierre de caja').run();`);
add("55", "modal-cierre-caja-procesando", "Cierre de caja procesando Todo el día (tecla 3).", `__vals().headerBtns.find(b=>b.label==='Cierre de caja').run(); await new Promise(r=>setTimeout(r,50)); __c.chooseClosing('todo'); __quiet();`);
add("56", "modal-cierre-manual", "Cierre manual con fechas y horas (tecla 4 en Cierre de caja).", `await __set({modal:'closing',m:{busy:false}}); __c.chooseClosing('manual');`);
add("57", "modal-cierre-manual-varios-dias", "Cierre manual de varios días (desde ayer): aviso amarillo.", `await __set({modal:'closing',m:{busy:false}}); __c.chooseClosing('manual'); const d=new Date(); d.setDate(d.getDate()-1); const y=d.getFullYear()+'-'+String(d.getMonth()+1).padStart(2,'0')+'-'+String(d.getDate()).padStart(2,'0'); await __set({m:{...__c.state.m,d1:y,t1:'08:00'}});`);
add("58", "modal-cierre-manual-error", "Cierre manual con hora de fin anterior a la de inicio: error.", `await __set({modal:'closing',m:{busy:false}}); __c.chooseClosing('manual'); await __set({m:{...__c.state.m,t1:'18:00',t2:'09:00'}});`);
add("59", "modal-resultado-cierre", "Resultado del cierre de Todo el día (KPIs, métodos de pago y vendedores).", `await __set({modal:'result',m:{data:__c.closingData('todo')}});`);
add("60", "modal-impresion", "Impresión: impresora y ancho de papel (menú de usuario > Impresión).", `__vals().menuItems[0].run();`);
add("61", "modal-impresion-dropdown", "Impresión con el desplegable de impresoras abierto.", `__vals().menuItems[0].run(); await new Promise(r=>setTimeout(r,50)); __vals().pr.dd.toggle();`);
add("62", "modal-balanza", "Balanza en modo Puerto serie con lectura en vivo (menú de usuario > Balanza).", `__vals().menuItems[1].run(); await __still(1245);`);
add("63", "modal-balanza-dropdown", "Balanza con el desplegable Velocidad abierto.", `__vals().menuItems[1].run(); await __still(1245); __vals().bz.selects[0].dd.toggle();`);
add("64", "modal-balanza-archivo", "Balanza en modo Archivo.", `__vals().menuItems[1].run(); await __still(1245); await __set({scale:{...__c.state.scale,modo:'archivo'}});`);
add("65", "modal-cuenta-corriente-desde-orden", "Enviar a cuenta corriente desde la orden (F6) con cliente elegido y tipo Remito.", `await __cart(); __c.openFactura('cart'); await __set({m:{...__c.state.m,clienteId:1,cq:'Restaurante La Huerta'}});`, 1900);
add("66", "modal-factura-nueva-clientes", "Nueva factura con el buscador de clientes abierto (Nueva factura en Cuenta corriente).", `await __set({screen:'cc',ccView:'list'}); __c.openFactura('new'); await __set({m:{...__c.state.m,cliOpen:true}});`);
add("67", "modal-factura-nueva-productos", "Nueva factura con cliente y tipo Factura A, buscador de productos abierto.", `await __set({screen:'cc',ccView:'list'}); __c.openFactura('new'); await __set({m:{...__c.state.m,clienteId:2,cq:'Hotel Aconcagua',tipo:'a',pOpen:true,pq:'',items:[{pid:13,name:'Banana Ecuador',unit:'Kg',qty:18,price:2400}]}});`);
add("68", "modal-factura-editar", "Editar factura existente (acción Editar de una fila de Cuenta corriente).", `await __set({screen:'cc',ccView:'list'}); __c.openFactura('edit',__fact('f1'));`);
add("69", "modal-factura-error", "Nueva factura guardada sin cliente: error.", `await __set({screen:'cc',ccView:'list'}); __c.openFactura('new'); __c.saveFactura();`);
add("70", "cc-lista", "Cuenta corriente: KPIs, filtros y tabla (botón Cuenta corriente del header).", `__vals().headerBtns.find(b=>b.label==='Cuenta corriente').run();`);
add("71", "cc-filtro-con-saldo", "Cuenta corriente filtrada por Con saldo.", `await __set({screen:'cc',ccView:'list',festado:'pend',fpage:1});`);
add("72", "cc-busqueda", "Cuenta corriente buscando \"hotel\".", `await __set({screen:'cc',ccView:'list',fq:'hotel',fpage:1});`);
add("73", "cc-dropdown-tipo", "Cuenta corriente con el desplegable de tipo de comprobante abierto.", `await __set({screen:'cc',ccView:'list'}); __vals().ddTipo.toggle();`);
add("74", "cc-filtro-tipo-remito", "Cuenta corriente filtrada por tipo Remito.", `await __set({screen:'cc',ccView:'list',ftipo:'remito',fpage:1});`);
add("75", "cc-vacio", "Cuenta corriente sin resultados (búsqueda \"zzz\").", `await __set({screen:'cc',ccView:'list',fq:'zzz',fpage:1});`);
add("76", "cc-paginacion-2", "Cuenta corriente en la página 2 (8 por página).", `await __set({screen:'cc',ccView:'list',fpage:2});`);
add("77", "cc-detalle-parcial", "Detalle de factura con pagos parciales (click en la fila de la Factura A de Hotel Aconcagua).", `await __set({screen:'cc',ccView:'detail',ccId:'f1'});`);
add("78", "cc-detalle-sin-pagos", "Detalle de factura sin pagos (Remito de Comedor San José).", `await __set({screen:'cc',ccView:'detail',ccId:'f2'});`);
add("79", "cc-detalle-pagada", "Detalle de factura pagada (Factura B de Bar El Patio).", `await __set({screen:'cc',ccView:'detail',ccId:'f4'});`);
add("80", "modal-agregar-pago", "Agregar pago a una factura con saldo (botón Pago de la fila).", `await __set({screen:'cc',ccView:'list'}); __c.openPago(__fact('f2'));`);
add("81", "modal-agregar-pago-completo", "Agregar pago con el saldo completo, transferencia y referencia.", `await __set({screen:'cc',ccView:'list'}); __c.openPago(__fact('f2')); await __set({m:{...__c.state.m,monto:String(__c.state.m.saldo),metodo:'transferencia',ref:'TRF 90544'}});`);
add("82", "modal-agregar-pago-error", "Agregar pago con monto mayor al saldo: error.", `await __set({screen:'cc',ccView:'list'}); __c.openPago(__fact('f0')); await __set({m:{...__c.state.m,monto:'999999'}}); __c.savePago();`);
add("83", "modal-historial-pagos", "Historial de pagos de una factura con dos pagos (acción Historial de la fila).", `await __set({screen:'cc',ccView:'list'}); await __set({modal:'hist',m:{fid:'f0'}});`);
add("84", "modal-historial-pagos-vacio", "Historial de pagos de una factura sin pagos.", `await __set({screen:'cc',ccView:'list'}); await __set({modal:'hist',m:{fid:'f2'}});`);
add("85", "modal-confirmar-eliminar-factura", "Confirmación ¿Eliminar la factura? (acción Eliminar de la fila).", `await __set({screen:'cc',ccView:'list'}); __vals().fRows[0].del({stopPropagation(){}});`);
add("86", "toast-exito", "Toast de éxito al escanear un código (scan del código de Tomate redondo).", `await __cart(); __c.scan('7790001000213');`, 450);
add("87", "toast-exito-balanza", "Toast de éxito al escanear una etiqueta de balanza (Banana 1,245 kg, código 201000012450).", `__c.scan('201000012450');`, 450);
add("88", "toast-error", "Toast de error al cobrar con la orden vacía (F2 sin productos).", `__c.openPay(true,false);`, 450);
add("89", "toast-info", "Toast informativo Sesión cerrada (F4 / Cerrar sesión: logout).", `__c.logout();`, 450);

let edge, server, ws;
const sleep = ms => new Promise(r => setTimeout(r, ms));

function serve() {
  server = http.createServer(async (req, res) => {
    const rel = decodeURIComponent(new URL(req.url, "http://x").pathname).replace(/^\/+/, "");
    const file = path.resolve(PROJECT, rel);
    if (!file.startsWith(PROJECT)) { res.writeHead(403).end(); return; }
    try {
      const body = await fs.readFile(file);
      res.writeHead(200, { "Content-Type": TYPES[path.extname(file).toLowerCase()] || "application/octet-stream" }).end(body);
    } catch { res.writeHead(404).end(); }
  });
  return new Promise(r => server.listen(0, "127.0.0.1", () => r(server.address().port)));
}

async function cdpTarget() {
  for (let i = 0; i < 100; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${CDP_PORT}/json/list`)).json();
      const page = list.find(t => t.type === "page");
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(200);
  }
  throw new Error("Edge no abrió el puerto de depuración");
}

let seq = 0;
const pending = new Map();
function send(method, params = {}) {
  const id = ++seq;
  ws.send(JSON.stringify({ id, method, params }));
  return new Promise((resolve, reject) => pending.set(id, { resolve, reject, method }));
}

async function evaluate(expression) {
  const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
  return r.result.value;
}

async function load(url) {
  await send("Page.navigate", { url });
  for (let i = 0; i < 150; i++) {
    await sleep(100);
    try { if (await evaluate(HELPERS)) break; } catch {}
    if (i === 149) throw new Error("El prototipo no terminó de montar");
  }
  const fonts = await evaluate(`(async () => {
    await Promise.all(['400 16px Inter','500 16px Inter','600 16px Inter','700 16px Inter','400 13px "Geist Mono"','500 13px "Geist Mono"'].map(f => document.fonts.load(f)));
    await document.fonts.ready;
    await Promise.all([...document.images].map(i => i.complete ? 0 : new Promise(r => { i.onload = i.onerror = r; })));
    return [...new Set([...document.fonts].filter(f => f.status === 'loaded').map(f => f.family.replace(/"/g, '')))];
  })()`);
  if (!fonts.includes("Inter") || !fonts.includes("Geist Mono")) throw new Error("No cargaron las fuentes de Google: " + fonts.join(", "));
  return fonts;
}

async function main() {
  const port = await serve();
  const url = `http://127.0.0.1:${port}/${encodeURIComponent(PAGE)}`;
  const profile = await fs.mkdtemp(path.join(os.tmpdir(), "andex-capturas-"));
  edge = spawn(EDGE, ["--headless=new", `--remote-debugging-port=${CDP_PORT}`, "--remote-allow-origins=*", `--user-data-dir=${profile}`, "--no-first-run", "--no-default-browser-check", "--disable-extensions", "--hide-scrollbars", "--force-device-scale-factor=1", "--lang=es-AR", "about:blank"], { stdio: "ignore" });
  ws = new WebSocket(await cdpTarget());
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
  ws.onmessage = e => {
    const msg = JSON.parse(e.data);
    const p = msg.id && pending.get(msg.id);
    if (!p) return;
    pending.delete(msg.id);
    msg.error ? p.reject(new Error(p.method + ": " + msg.error.message)) : p.resolve(msg.result);
  };
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Emulation.setTimezoneOverride", { timezoneId: "America/Argentina/Mendoza" });

  const done = [], failed = [];
  for (const [w, h] of SIZES) {
    await send("Emulation.setDeviceMetricsOverride", { width: w, height: h, deviceScaleFactor: 1, mobile: false });
    for (const s of S) {
      const file = `${s.id}-${s.name}-${w}.png`;
      if (ONLY && !ONLY.test(file)) continue;
      try {
        const fonts = await load(url);
        await evaluate(`(async () => { ${s.js} })()`);
        await sleep(s.wait);
        const shot = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
        await fs.writeFile(path.join(OUT, file), Buffer.from(shot.data, "base64"));
        done.push(file);
        console.log("ok", file, fonts.join("+"));
      } catch (e) {
        failed.push(`${file}: ${e.message}`);
        console.log("FALLÓ", file, e.message);
      }
    }
  }

  if (!ONLY) {
    const lines = ["# Referencia v2: capturas del POS AndexMarket", "", `Prototipo: \`design/redise-o-profesional-de-app/project/${PAGE}\`. Cada captura existe en 1920x1080 (\`-1920.png\`) y 1366x768 (\`-1366.png\`). Se regeneran con \`node docs/referencia-v2/capturar.mjs\` (Edge headless + CDP; acepta una regex para capturar solo algunas).`, "", "Props del prototipo por defecto: pantallaInicial=pos, afip=true, facturacion=true, mercadoPago=true, cancelacionControlada=false, pesoManual=true. Vendedor logueado: Lucía Fernández.", ""];
    for (const s of S) lines.push(`- \`${s.id}-${s.name}-{1920,1366}.png\`: ${s.desc}`);
    await fs.writeFile(path.join(OUT, "INDICE.md"), lines.join("\n") + "\n");
  }
  console.log(`\n${done.length} capturas, ${failed.length} fallidas`);
  failed.forEach(f => console.log("  " + f));
  return failed.length;
}

let code = 1;
try { code = await main(); } catch (e) { console.error(e); }
finally {
  try { await send("Browser.close"); } catch {}
  try { ws?.close(); } catch {}
  await sleep(500);
  try { edge?.kill(); } catch {}
  server?.close();
}
process.exit(code ? 1 : 0);
