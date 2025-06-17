declare global {
  interface Window {
    electron?: {
      process: {
        argv: string[];
      };
    };
    require?: (module: string) => any;
  }
}

export {};
