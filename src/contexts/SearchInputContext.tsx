import React, { createContext, useContext, useRef, RefObject } from "react";

interface SearchInputContextType {
  searchInputRef: RefObject<HTMLInputElement | null>;
  focusSearchInput: (source: string) => void;
}

const SearchInputContext = createContext<SearchInputContextType | undefined>(
  undefined
);

export function SearchInputProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const searchInputRef = useRef<HTMLInputElement>(null);

  const focusSearchInput = (source: string) => {
    // console.log(`🔍 Intentando enfocar la barra de búsqueda desde: ${source}`);
    // Usar un delay más agresivo para asegurar que el DOM esté listo
    setTimeout(() => {
      if (searchInputRef.current) {
        try {
          searchInputRef.current.focus();
          searchInputRef.current.scrollIntoView({
            behavior: "smooth",
            block: "center",
          });
          // console.log(`🎯 Foco establecido desde: ${source}`);
        } catch (error) {
          console.warn(`⚠️ Error al enfocar desde ${source}:`, error);
        }
      } else {
        console.warn(
          `⚠️ No se pudo enfocar desde ${source}: searchInputRef.current es nulo.`
        );
      }
    }, 200);
  };

  return (
    <SearchInputContext.Provider value={{ searchInputRef, focusSearchInput }}>
      {children}
    </SearchInputContext.Provider>
  );
}

export function useSearchInput() {
  const context = useContext(SearchInputContext);
  if (context === undefined) {
    throw new Error("useSearchInput must be used within a SearchInputProvider");
  }
  return context;
}
