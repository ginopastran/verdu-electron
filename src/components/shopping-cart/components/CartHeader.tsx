import { Button } from "@/components/ui/button";
import { Plus, X } from "lucide-react";
import { useCartContext } from "../contexts/CartContext";

export function CartHeader() {
  const { screens, activeScreen, setActiveScreen, addScreen, deleteScreen } =
    useCartContext();

  return (
    <div className="flex items-center gap-2">
      {screens.map((screen) => (
        <div key={screen.id} className="flex items-center gap-1">
          <Button
            variant={activeScreen === screen.id ? "default" : "outline"}
            size="sm"
            onClick={() => setActiveScreen(screen.id)}
            className="h-8"
          >
            Orden {screen.id + 1}
          </Button>
          {screen.id > 0 && (
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0 hover:bg-red-100 hover:text-red-600"
              onClick={() => deleteScreen(screen.id)}
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
      ))}
      {screens.length < 4 && (
        <Button variant="outline" size="sm" className="h-8" onClick={addScreen}>
          <Plus className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
}
