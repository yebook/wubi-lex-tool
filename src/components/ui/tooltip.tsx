import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import type { ComponentProps } from "react";

import { cn } from "../../lib/cn";
import { useOverlayRoot } from "./overlay-provider";

export const Tooltip = TooltipPrimitive.Root;
export const TooltipTrigger = TooltipPrimitive.Trigger;

export function TooltipContent({
  className,
  sideOffset = 8,
  ...props
}: ComponentProps<typeof TooltipPrimitive.Content>) {
  const container = useOverlayRoot();
  return (
    <TooltipPrimitive.Portal container={container}>
      <TooltipPrimitive.Content
        className={cn(
          "z-tooltip max-w-[calc(var(--wl-space-16)*5)] rounded-sm border border-border-strong bg-surface-2 px-[var(--wl-space-3)] py-[var(--wl-space-2)] [font-size:var(--wl-font-size-xs)] text-text-1 shadow-overlay",
          className,
        )}
        sideOffset={sideOffset}
        {...props}
      />
    </TooltipPrimitive.Portal>
  );
}
