declare global {
  interface Window {
    electron?: {
      process: {
        argv: string[];
      };
    };
  }
}

export {};
