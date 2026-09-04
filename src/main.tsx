import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";

import { App } from "./app/app";
import { AppRuntimeProvider } from "./app/providers/app-runtime-provider";
import { UiPreferencesProvider } from "./app/providers/ui-preferences-provider";
import { WindowControlsProvider } from "./app/providers/window-controls-provider";
import { OverlayProvider } from "./components/ui";
import { i18n } from "./i18n";
import "./styles/theme.css";
import "./styles/runtime-status.css";
import "./styles/shell.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("WubiLex root element is missing");
}

createRoot(root).render(
  <StrictMode>
    <I18nextProvider i18n={i18n}>
      <UiPreferencesProvider>
        <OverlayProvider>
          <AppRuntimeProvider>
            <WindowControlsProvider>
              <App />
            </WindowControlsProvider>
          </AppRuntimeProvider>
        </OverlayProvider>
      </UiPreferencesProvider>
    </I18nextProvider>
  </StrictMode>,
);
