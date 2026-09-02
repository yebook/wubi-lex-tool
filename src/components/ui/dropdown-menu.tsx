import * as DropdownPrimitive from "@radix-ui/react-dropdown-menu";
import type { ComponentProps } from "react";

import { cn } from "../../lib/cn";
import { useOverlayRoot } from "./overlay-provider";

export const DropdownMenu = DropdownPrimitive.Root;
export const DropdownMenuTrigger = DropdownPrimitive.Trigger;
export const DropdownMenuGroup = DropdownPrimitive.Group;

export function DropdownMenuContent({
  className,
  sideOffset = 4,
  ...props
}: ComponentProps<typeof DropdownPrimitive.Content>) {
  const container = useOverlayRoot();
  return (
    <DropdownPrimitive.Portal container={container}>
      <DropdownPrimitive.Content
        className={cn(
          "z-dropdown min-w-[calc(var(--wl-control-size)*4)] overflow-hidden rounded-md border border-border-strong bg-surface-2 p-[var(--wl-space-1)] text-text-1 shadow-overlay",
          className,
        )}
        sideOffset={sideOffset}
        {...props}
      />
    </DropdownPrimitive.Portal>
  );
}

export function DropdownMenuItem({
  className,
  ...props
}: ComponentProps<typeof DropdownPrimitive.Item>) {
  return (
    <DropdownPrimitive.Item
      className={cn(
        "relative flex min-h-control cursor-default items-center rounded-sm px-[var(--wl-space-3)] py-[var(--wl-space-2)] [font-size:var(--wl-font-size-sm)] outline-none select-none focus:bg-primary-subtle focus:text-text-1 data-disabled:pointer-events-none data-disabled:text-text-3 data-disabled:opacity-[var(--wl-opacity-disabled)]",
        className,
      )}
      {...props}
    />
  );
}

export function DropdownMenuLabel({
  className,
  ...props
}: ComponentProps<typeof DropdownPrimitive.Label>) {
  return (
    <DropdownPrimitive.Label
      className={cn(
        "px-[var(--wl-space-3)] py-[var(--wl-space-2)] [font-size:var(--wl-font-size-xs)] [font-weight:var(--wl-weight-strong)] text-text-2",
        className,
      )}
      {...props}
    />
  );
}

export function DropdownMenuSeparator({
  className,
  ...props
}: ComponentProps<typeof DropdownPrimitive.Separator>) {
  return (
    <DropdownPrimitive.Separator
      className={cn("my-[var(--wl-space-1)] h-px bg-border", className)}
      {...props}
    />
  );
}
