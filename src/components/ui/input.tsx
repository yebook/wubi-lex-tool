import type { ComponentProps } from "react";

import { cn } from "../../lib/cn";

export function Input({ className, ...props }: ComponentProps<"input">) {
  return (
    <input
      className={cn(
        "duration-fast min-h-control w-full rounded-sm border border-border-strong bg-surface-1 px-[var(--wl-space-3)] py-[var(--wl-space-2)] [font-size:var(--wl-font-size-sm)] text-text-1 transition-colors ease-standard placeholder:text-text-3 read-only:bg-surface-2 hover:border-primary focus-visible:border-primary focus-visible:outline-focus disabled:cursor-not-allowed disabled:bg-surface-3 disabled:text-text-3 disabled:opacity-[var(--wl-opacity-disabled)] aria-invalid:border-danger aria-invalid:outline-danger",
        className,
      )}
      {...props}
    />
  );
}
