import type { ReactNode } from "react";

import type { WindowControlIntent, WindowStateSnapshot } from "../../types/generated/bindings";
import { Copy, Minus, Square, X } from "../../icons/window-controls";

interface WindowTitleBarProps {
  iconUrl: string;
  version: string;
  snapshot: WindowStateSnapshot | null;
  onControl: (intent: WindowControlIntent) => void;
}

export function WindowTitleBar({
  iconUrl,
  version,
  snapshot,
  onControl,
}: WindowTitleBarProps) {
  const maximized = snapshot?.maximized ?? false;
  const exiting = snapshot?.visibility === "exiting";
  const maximizeLabel = maximized ? "还原窗口" : "最大化窗口";
  const MaximizeIcon = maximized ? Copy : Square;

  return (
    <header className="window-title-bar">
      <div className="window-drag-region" data-tauri-drag-region="deep">
        <img className="window-brand-icon" src={iconUrl} alt="" width="24" height="24" />
        <strong className="window-brand-name">WubiLex</strong>
        <span className="window-version">v{version}</span>
      </div>
      <div className="window-controls" aria-label="窗口控制">
        <WindowButton
          label="最小化到托盘"
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
          label="关闭窗口"
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
