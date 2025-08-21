declare global {
  interface Window {
    electron?: {
      process: {
        argv: string[];
      };
      ipcRenderer: {
        invoke: (channel: string, ...args: any[]) => Promise<any>;
      };
    };
    printer?: {
      printTicket: (orderData: any) => Promise<any>;
      printClosing: (closingData: any) => Promise<any>;
      printFacturaTicket: (facturaData: any) => Promise<any>;
    };
    require?: (module: string) => any;
  }
}

export {};
