import * as DialogPrimitive from "@radix-ui/react-dialog";
import type { ComponentProps } from "react";
import { useTranslation } from "react-i18next";

import { X } from "../../icons/ui";
import { cn } from "../../lib/cn";
import { useOverlayRoot } from "./overlay-provider";

export const Dialog = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
export const DialogClose = DialogPrimitive.Close;

export function DialogContent({
  className,
  children,
  ...props
}: ComponentProps<typeof DialogPrimitive.Content>) {
  const { t } = useTranslation("ui");
  const container = useOverlayRoot();

  return (
    <DialogPrimitive.Portal container={container}>
      <DialogPrimitive.Overlay className="fixed inset-0 z-dialog bg-scrim" />
      <DialogPrimitive.Content
        className={cn(
          "fixed top-1/2 left-1/2 z-dialog grid w-[calc(100%_-_var(--wl-space-8))] max-w-[calc(var(--wl-page-max-width)/2)] -translate-x-1/2 -translate-y-1/2 gap-[var(--wl-space-4)] rounded-lg border border-border-strong bg-surface-2 p-[var(--wl-space-6)] text-text-1 shadow-overlay focus:outline-none",
          className,
        )}
        {...props}
      >
        {children}
        <DialogPrimitive.Close
          className="duration-fast absolute top-[var(--wl-space-3)] right-[var(--wl-space-3)] inline-grid size-control place-items-center rounded-md border border-transparent bg-transparent p-0 text-text-2 transition-colors ease-standard hover:bg-surface-3 hover:text-text-1 focus-visible:outline-focus"
          aria-label={t("dialogClose")}
        >
          <X className="size-icon-md" aria-hidden="true" />
        </DialogPrimitive.Close>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

export function DialogTitle({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      className={cn(
        "m-0 pr-[var(--wl-control-size)] [font-size:var(--wl-font-size-lg)] [font-weight:var(--wl-weight-strong)]",
        className,
      )}
      {...props}
    />
  );
}

export function DialogDescription({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Description>) {
  return (
    <DialogPrimitive.Description
      className={cn(
        "m-0 [font-size:var(--wl-font-size-sm)] text-text-2",
        className,
      )}
      {...props}
    />
  );
}
