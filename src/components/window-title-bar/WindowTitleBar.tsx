import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import type {
  WindowControlIntent,
  WindowStateSnapshot,
} from "../../types/generated/bindings";
import { Copy, Minus, Square, X } from "../../icons/window-controls";

interface WindowTitleBarProps {
  iconUrl: string;
  version: string;
  pageTitle?: string;
  snapshot: WindowStateSnapshot | null;
  onControl: (intent: WindowControlIntent) => void;
}

export function WindowTitleBar({
  iconUrl,
  version,
  pageTitle,
  snapshot,
  onControl,
}: WindowTitleBarProps) {
  const { t } = useTranslation("window");
  const maximized = snapshot?.maximized ?? false;
  const exiting = snapshot?.visibility === "exiting";
  const maximizeLabel = maximized ? t("restore") : t("maximize");
  const MaximizeIcon = maximized ? Copy : Square;

  return (
    <header className="window-title-bar">
      <div className="window-drag-region" data-tauri-drag-region="deep">
        <img
          className="window-brand-icon"
          src={iconUrl}
          alt=""
          width="24"
          height="24"
        />
        <strong className="window-brand-name">WubiLex</strong>
        <span className="window-version">v{version}</span>
        {pageTitle ? (
          <>
            <span className="window-title-separator" aria-hidden="true" />
            <span className="window-page-title">{pageTitle}</span>
          </>
        ) : null}
      </div>
      <div className="window-controls" aria-label={t("controls")}>
        <WindowButton
          label={t("minimizeToTray")}
          disabled={exiting}
          onClick={() => onControl("minimizeToTray")}
        >
          <Minus aria-hidden="true" strokeWidth={1.8} />
        </WindowButton>
        <WindowButton
          label={maximizeLabel}
          disabled={exiting}
          onClick={() => onControl("toggleMaximize")}
        >
          <MaximizeIcon aria-hidden="true" strokeWidth={1.8} />
        </WindowButton>
        <WindowButton
          label={t("close")}
          className="window-close-button"
          disabled={exiting}
          onClick={() => onControl("close")}
        >
          <X aria-hidden="true" strokeWidth={1.8} />
        </WindowButton>
      </div>
    </header>
  );
}

function WindowButton({
  label,
  className = "",
  disabled,
  onClick,
  children,
}: {
  label: string;
  className?: string;
  disabled: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      className={`window-control-button ${className}`.trim()}
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
