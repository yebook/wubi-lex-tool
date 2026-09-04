import { useTranslation } from "react-i18next";

import { AlertTriangle, CheckCircle2, LoaderCircle } from "../../icons/ui";
import { cn } from "../../lib/cn";
import { boundVisibleText } from "../../lib/visible-text";

export function StatusBar({
  loading,
  warning,
}: {
  loading: boolean;
  warning: string | null;
}) {
  const { t } = useTranslation("shell");
  const message = warning
    ? boundVisibleText(warning)
    : loading
      ? t("status.loading")
      : t("status.ready");
  const Icon = warning ? AlertTriangle : loading ? LoaderCircle : CheckCircle2;

  return (
    <footer
      className={cn("shell-status-bar", warning && "shell-status-warning")}
      role="status"
      aria-live="polite"
    >
      <Icon aria-hidden="true" strokeWidth={1.8} />
      <span>{message}</span>
    </footer>
  );
}
