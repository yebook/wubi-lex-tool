// @vitest-environment jsdom

import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createMemoryRouter, RouterProvider } from "react-router";

import { OverlayProvider } from "../../components/ui";
import { i18n } from "../../i18n";
import type {
  ConfigSnapshot,
  RuntimeNotice,
  RuntimeSnapshot,
  WindowStateSnapshot,
} from "../../types/generated/bindings";
import type { RuntimeClient } from "../../lib/runtime-client";
import type { UiConfigClient } from "../../lib/config-client";
import type { WindowClient } from "../../lib/window-client";
import { OverviewRoute } from "../../routes/overview/OverviewRoute";
import { AppRuntimeProvider } from "../providers/app-runtime-provider";
import { UiPreferencesProvider } from "../providers/ui-preferences-provider";
import { WindowControlsProvider } from "../providers/window-controls-provider";
import { NavigationProvider } from "../router/navigation-provider";
import { AppShell } from "./AppShell";

afterEach(cleanup);

const configSnapshot: ConfigSnapshot = {
  revision: 1,
  config: {
    schemaVersion: 1,
    ui: {
      theme: "system",
      density: "standard",
      locale: "zh-CN",
      sidebarCollapsed: false,
      onboardingVersion: 0,
    },
  },
  persistence: "ready",
  notices: [],
};

describe("AppShell", () => {
  it("keeps one app bar and promotes the newest live runtime notice", async () => {
    const olderNotice: RuntimeNotice = {
      code: "loggingUnavailable",
      summary: "旧运行通知",
      detail: null,
    };
    const newerNotice: RuntimeNotice = {
      code: "windowPersistenceFailed",
      summary: "新运行通知",
      detail: "最新窗口状态无法保存",
    };
    const runtimeSnapshot: RuntimeSnapshot = {
      privilege: { state: "elevated", failure: null },
      previousAbnormalSessionCount: 0,
      primaryLaunch: {
        request: { startHidden: false, navigationPath: "/overview" },
        notices: [],
      },
      latestSecondaryLaunch: null,
      notices: [olderNotice],
    };
    const windowSnapshot: WindowStateSnapshot = {
      revision: 1,
      visibility: "visible",
      maximized: false,
    };
    let noticeListener: ((notice: RuntimeNotice) => void) | undefined;
    const runtimeClient: RuntimeClient = {
      listenLaunch: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () => runtimeSnapshot),
    };
    const configClient: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () => configSnapshot),
      updateUi: vi.fn(async () => configSnapshot),
    };
    const windowClient: WindowClient = {
      listenState: vi.fn(async () => vi.fn()),
      listenNotice: vi.fn(async (listener) => {
        noticeListener = listener;
        return vi.fn();
      }),
      fetchState: vi.fn(async () => windowSnapshot),
      control: vi.fn(async () => windowSnapshot),
    };
    const router = createMemoryRouter(
      [
        {
          path: "/",
          element: (
            <NavigationProvider>
              <AppShell />
            </NavigationProvider>
          ),
          children: [{ path: "overview", element: <OverviewRoute /> }],
        },
      ],
      { initialEntries: ["/overview"] },
    );
    const appearanceRoot = document.createElement("html");

    render(
      <I18nextProvider i18n={i18n}>
        <UiPreferencesProvider
          client={configClient}
          appearanceEnvironment={{ root: appearanceRoot }}
        >
          <OverlayProvider container={document.createElement("div")}>
            <AppRuntimeProvider client={runtimeClient}>
              <WindowControlsProvider client={windowClient}>
                <RouterProvider router={router} />
              </WindowControlsProvider>
            </AppRuntimeProvider>
          </OverlayProvider>
        </UiPreferencesProvider>
      </I18nextProvider>,
    );

    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toContain("旧运行通知"),
    );
    expect(document.querySelectorAll(".window-title-bar")).toHaveLength(1);
    expect(screen.getByText("WubiLex")).toBeTruthy();
    expect(screen.getAllByText("概览").length).toBeGreaterThan(0);
    expect(screen.getByText("已获得管理员权限")).toBeTruthy();

    act(() => noticeListener?.(newerNotice));
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toContain(
        "最新窗口状态无法保存",
      ),
    );
    expect(screen.getByRole("status").textContent).not.toContain("旧运行通知");
  });
});
