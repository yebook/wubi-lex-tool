import { useEffect, useRef, useState } from "react";
import { RouterProvider } from "react-router";
import packageMetadata from "../../package.json";

import { WindowTitleBar } from "../components/window-title-bar/WindowTitleBar";
import { featuresStore } from "../stores/features";
import {
  resolveBrowserInitialNavigation,
  useAppRuntime,
} from "./providers/app-runtime-provider";
import type { InitialNavigation } from "./providers/app-runtime-provider";
import { useAppWindowControls } from "./providers/window-controls-provider";
import { createHashAppRouter } from "./router/router";

const appIconUrl = new URL("../../src-tauri/icons/icon.ico", import.meta.url)
  .href;

export function App() {
  const runtime = useAppRuntime();
  const initialNavigation = useRef<InitialNavigation | null>(null);

  useEffect(() => {
    void featuresStore.getState().initialize();
  }, []);

  if (runtime.loadState.status === "loading" && !initialNavigation.current) {
    return <BootstrapShell />;
  }

  if (!initialNavigation.current) {
    initialNavigation.current = resolveBrowserInitialNavigation(runtime);
  }

  return <AppRouterHost initialNavigation={initialNavigation.current} />;
}

function AppRouterHost({
  initialNavigation,
}: {
  initialNavigation: InitialNavigation;
}) {
  const [router, setRouter] = useState<ReturnType<
    typeof createHashAppRouter
  > | null>(null);

  useEffect(() => {
    const nextRouter = createHashAppRouter(initialNavigation);
    setRouter(nextRouter);
    return () => nextRouter.dispose();
  }, [initialNavigation]);

  return router ? <RouterProvider router={router} /> : <BootstrapShell />;
}

function BootstrapShell() {
  const windowControls = useAppWindowControls();
  return (
    <div className="app-shell bootstrap-shell">
      <WindowTitleBar
        iconUrl={appIconUrl}
        version={packageMetadata.version}
        snapshot={windowControls.snapshot}
        onControl={(intent) => void windowControls.control(intent)}
      />
      <main className="bootstrap-main" aria-busy="true">
        <span className="loading-indicator" aria-hidden="true" />
      </main>
    </div>
  );
}
