import type { ComponentProps } from "react";

import { cn } from "../../lib/cn";

export function Kbd({ className, ...props }: ComponentProps<"kbd">) {
  return (
    <kbd
      className={cn(
        "inline-flex min-h-[var(--wl-icon-lg)] items-center rounded-sm border border-border-strong bg-surface-3 px-[var(--wl-space-2)] font-mono [font-size:var(--wl-font-size-xs)] text-text-2 tabular-nums",
        className,
      )}
      {...props}
    />
  );
}
