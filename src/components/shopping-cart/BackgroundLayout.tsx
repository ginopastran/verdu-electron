import { ReactNode } from "react";
import iselinLogo from "../../assets/iselin-logo.png";
import andextechLogo from "../../assets/andextech-black.png";

interface BackgroundLayoutProps {
  children: ReactNode;
}

export const BackgroundLayout = ({ children }: BackgroundLayoutProps) => {
  return (
    <div className="min-h-screen bg-white-cream h-screen relative overflow-hidden">
      {/* Background logos */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-0">
        <div className="relative w-full h-full flex flex-col items-center justify-center gap-10">
          <img
            src={iselinLogo}
            alt="Iselin Logo"
            className="w-[540px] opacity-[.07] select-none"
          />
          <img
            src={andextechLogo}
            alt="Andextech Logo"
            className="w-[440px] opacity-[.07] select-none"
          />
        </div>
      </div>
      <div className="w-full mx-auto flex h-full flex-col px-8 py-6 relative z-10">
        {children}
      </div>
    </div>
  );
};
