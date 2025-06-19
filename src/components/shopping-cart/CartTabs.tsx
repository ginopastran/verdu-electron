import { Button } from "@/components/ui/button";
import { Plus, X } from "lucide-react";
import { CartScreen } from "@/hooks/useCartState";

interface CartTabsProps {
  screens: CartScreen[];
  activeScreen: number;
  onAddScreen: () => void;
  onDeleteScreen: (id: number) => void;
  onChangeScreen: (id: number) => void;
}

export function CartTabs({
  screens,
  activeScreen,
  onAddScreen,
  onDeleteScreen,
  onChangeScreen,
}: CartTabsProps) {
  return (
    <div className="flex items-center ml-0">
      {screens.map((screen) => (
        <div key={screen.id} className="flex items-center">
          <Button
            variant={activeScreen === screen.id ? "default" : "outline"}
            size="sm"
            onClick={() => onChangeScreen(screen.id)}
            className={`h-8 rounded-lg px-8 py-5 text-white ${
              screen.id > 0 ? "ml-4" : ""
            } ${activeScreen === screen.id ? "bg-black" : "bg-gray-300"}`}
          >
            Orden {screen.id + 1}
          </Button>
          {screen.id > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-7 p-0 rounded-l-none border-l-0 hover:bg-red-100 hover:text-red-600"
              onClick={() => onDeleteScreen(screen.id)}
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
      ))}
      {screens.length < 4 && (
        <Button
          variant="outline"
          size="sm"
          className="h-8 ml-0.5 px-2"
          onClick={onAddScreen}
        >
          <Plus className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
}
