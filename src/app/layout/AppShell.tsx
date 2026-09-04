import { useTranslation } from "react-i18next";
import { Outlet, useLocation } from "react-router";
import packageMetadata from "../../../package.json";

import { WindowTitleBar } from "../../components/window-title-bar/WindowTitleBar";
import { latestRuntimeNotice } from "../../runtime-view";
import { useAppRuntime } from "../providers/app-runtime-provider";
import { useUiPreferences } from "../providers/ui-preferences-provider";
import { useAppWindowControls } from "../providers/window-controls-provider";
import { routeForPath } from "../router/catalog";
import { useAppNavigation } from "../router/navigation-provider";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

const appIconUrl = new URL("../../../src-tauri/icons/icon.ico", import.meta.url)
  .href;

export function AppShell() {
  const { t } = useTranslation("shell");
  const location = useLocation();
  const runtime = useAppRuntime();
  const preferences = useUiPreferences();
  const windowControls = useAppWindowControls();
  const navigation = useAppNavigation();
  const route = routeForPath(location.pathname);
  const pageTitle = route ? t(route.labelKey) : t("routes.overview");
  const snapshot =
    runtime.loadState.status === "ready" ? runtime.loadState.snapshot : null;
  const runtimeNotice = latestRuntimeNotice(snapshot, windowControls.notices);
  const warning =
    navigation.warning ??
    preferences.warning ??
    runtime.listenerWarning ??
    runtime.refreshWarning ??
    windowControls.warning ??
    runtimeNotice?.detail ??
    runtimeNotice?.summary ??
    null;

  return (
    <div className="app-shell">
      <WindowTitleBar
        iconUrl={appIconUrl}
        version={packageMetadata.version}
        pageTitle={pageTitle}
        snapshot={windowControls.snapshot}
        onControl={(intent) => void windowControls.control(intent)}
      />
      <div className="shell-body">
        <Sidebar />
        <main id="main-content" className="shell-route-main" tabIndex={-1}>
          <div className="route-content">
            <Outlet />
          </div>
        </main>
      </div>
      <StatusBar
        loading={
          runtime.loadState.status === "loading" ||
          preferences.status === "loading"
        }
        warning={warning}
      />
    </div>
  );
}
