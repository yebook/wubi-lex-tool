import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import { createContext, useContext } from "react";
import type { ReactNode } from "react";

const OverlayRootContext = createContext<HTMLElement | null>(null);

export function OverlayProvider({
  children,
  container,
}: {
  children: ReactNode;
  container?: HTMLElement | null;
}) {
  const overlayRoot =
    container ??
    (typeof document === "undefined"
      ? null
      : document.getElementById("overlay-root"));

  return (
    <OverlayRootContext.Provider value={overlayRoot}>
      <TooltipPrimitive.Provider delayDuration={300} skipDelayDuration={100}>
        {children}
      </TooltipPrimitive.Provider>
    </OverlayRootContext.Provider>
  );
}

export function useOverlayRoot(): HTMLElement | null {
  return useContext(OverlayRootContext);
}
