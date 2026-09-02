import { Slot } from "@radix-ui/react-slot";
import { cva } from "class-variance-authority";
import type { VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "../../lib/cn";

const buttonVariants = cva(
  "inline-flex min-h-control items-center justify-center gap-[var(--wl-space-2)] whitespace-nowrap rounded-md border px-[var(--wl-space-4)] [font-size:var(--wl-font-size-sm)] [font-weight:var(--wl-weight-medium)] transition-colors duration-fast ease-standard focus-visible:outline-focus disabled:pointer-events-none disabled:opacity-[var(--wl-opacity-disabled)]",
  {
    variants: {
      variant: {
        primary:
          "border-primary bg-primary text-on-primary hover:border-primary-hover hover:bg-primary-hover",
        secondary:
          "border-border-strong bg-surface-3 text-text-1 hover:bg-primary-subtle",
        outline:
          "border-border-strong bg-transparent text-text-1 hover:bg-surface-3",
        ghost:
          "border-transparent bg-transparent text-text-1 hover:bg-surface-3",
        danger:
          "border-danger bg-danger text-on-danger hover:border-danger hover:bg-danger",
      },
      size: {
        default: "py-[var(--wl-space-2)] compact:px-[var(--wl-space-3)]",
        icon: "size-control p-0",
      },
    },
    defaultVariants: { variant: "primary", size: "default" },
  },
);

export interface ButtonProps
  extends ComponentProps<"button">, VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  busy?: boolean;
}

export function Button({
  asChild = false,
  busy = false,
  className,
  disabled,
  type,
  variant,
  size,
  ...props
}: ButtonProps) {
  const Component = asChild ? Slot : "button";
  return (
    <Component
      type={asChild ? undefined : (type ?? "button")}
      className={cn(buttonVariants({ variant, size }), className)}
      aria-busy={busy || undefined}
      disabled={asChild ? undefined : disabled || busy}
      {...props}
    />
  );
}
