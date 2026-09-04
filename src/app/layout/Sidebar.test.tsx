// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createMemoryRouter, RouterProvider } from "react-router";

import { OverlayProvider } from "../../components/ui";
import { i18n } from "../../i18n";
import type { UiConfigClient } from "../../lib/config-client";
import type { AppearanceEnvironment } from "../../lib/ui-appearance";
import type { ConfigSnapshot, UiConfig } from "../../types/generated/bindings";
import { UiPreferencesProvider } from "../providers/ui-preferences-provider";
import { NavigationProvider } from "../router/navigation-provider";
import { Sidebar } from "./Sidebar";

afterEach(cleanup);

const snapshot = (revision: number, ui: UiConfig): ConfigSnapshot => ({
  revision,
  config: { schemaVersion: 1, ui },
  persistence: "ready",
  notices: [],
});

function Providers({
  children,
  client,
}: {
  children: ReactNode;
  client: UiConfigClient;
}) {
  const appearance: AppearanceEnvironment = {
    root: document.createElement("html"),
  };
  return (
    <I18nextProvider i18n={i18n}>
      <UiPreferencesProvider client={client} appearanceEnvironment={appearance}>
        <OverlayProvider container={document.body}>{children}</OverlayProvider>
      </UiPreferencesProvider>
    </I18nextProvider>
  );
}

describe("Sidebar", () => {
  it("keeps seven links accessible and synchronizes the collapsed preference", async () => {
    const user = userEvent.setup();
    const updateUi = vi.fn(async (ui: UiConfig) => snapshot(2, ui));
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () =>
        snapshot(1, { sidebarCollapsed: false }),
      ),
      updateUi,
    };
    const router = createMemoryRouter(
      [
        {
          path: "*",
          element: (
            <NavigationProvider>
              <Sidebar />
              <h1 data-route-heading tabIndex={-1}>
                页面
              </h1>
            </NavigationProvider>
          ),
        },
      ],
      { initialEntries: ["/overview"] },
    );
    render(
      <Providers client={client}>
        <RouterProvider router={router} />
      </Providers>,
    );

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "折叠侧栏" })).toBeTruthy(),
    );
    expect(screen.getAllByRole("link")).toHaveLength(7);
    const overview = screen.getByRole("link", { name: "概览" });
    const settings = screen.getByRole("link", { name: "设置" });
    expect(overview.getAttribute("aria-current")).toBe("page");
    expect(overview.classList.contains("sidebar-link-active")).toBe(true);
    expect(settings.classList.contains("sidebar-link-settings")).toBe(true);

    const initialLocationKey = router.state.location.key;
    await user.click(overview);
    expect(router.state.location.key).toBe(initialLocationKey);

    await user.click(screen.getByRole("button", { name: "折叠侧栏" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "展开侧栏" })).toBeTruthy(),
    );
    expect(screen.getByRole("link", { name: "码表" })).toBeTruthy();
    expect(
      document.querySelector(".shell-sidebar")?.hasAttribute("data-collapsed"),
    ).toBe(true);
    expect(updateUi).toHaveBeenCalledWith({
      theme: "system",
      density: "standard",
      locale: "zh-CN",
      sidebarCollapsed: true,
      onboardingVersion: 0,
    });
  });
});
