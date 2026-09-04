// @vitest-environment jsdom

import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createMemoryRouter,
  Outlet,
  RouterProvider,
  useLocation,
} from "react-router";

import { AppRuntimeProvider } from "../providers/app-runtime-provider";
import { OverlayProvider } from "../../components/ui";
import { i18n } from "../../i18n";
import type { RuntimeClient } from "../../lib/runtime-client";
import type {
  LaunchRequestedEvent,
  RuntimeSnapshot,
} from "../../types/generated/bindings";
import { NavigationProvider, useAppNavigation } from "./navigation-provider";
import { RuntimeNavigationBridge } from "./runtime-navigation-bridge";

afterEach(cleanup);

function launch(path: string | null): LaunchRequestedEvent {
  return {
    request: { startHidden: false, navigationPath: path },
    notices: [],
  };
}

function snapshot(secondary: LaunchRequestedEvent | null): RuntimeSnapshot {
  return {
    privilege: { state: "elevated", failure: null },
    previousAbnormalSessionCount: 0,
    primaryLaunch: launch("/overview"),
    latestSecondaryLaunch: secondary,
    notices: [],
  };
}

function NavigationProbe() {
  const location = useLocation();
  const navigation = useAppNavigation();
  return (
    <>
      <output aria-label="当前路径">{location.pathname}</output>
      <output aria-label="导航警告">{navigation.warning}</output>
      <Outlet />
    </>
  );
}

function Page({ children }: { children: ReactNode }) {
  return (
    <h1 data-route-heading tabIndex={-1}>
      {children}
    </h1>
  );
}

describe("RuntimeNavigationBridge", () => {
  it("pushes canonical launches, ignores empty and same-path launches, and fails closed", async () => {
    let listener: ((event: LaunchRequestedEvent) => void) | undefined;
    let currentSnapshot = snapshot(null);
    const client: RuntimeClient = {
      listenLaunch: vi.fn(async (next) => {
        listener = next;
        return vi.fn();
      }),
      fetchSnapshot: vi.fn(async () => currentSnapshot),
    };
    const router = createMemoryRouter(
      [
        {
          element: (
            <NavigationProvider>
              <RuntimeNavigationBridge consumedLaunchSequence={0} />
              <NavigationProbe />
            </NavigationProvider>
          ),
          children: [
            { path: "/overview", element: <Page>概览</Page> },
            { path: "/settings", element: <Page>设置</Page> },
          ],
        },
      ],
      { initialEntries: ["/overview"] },
    );
    const overlay = document.createElement("div");
    const view = render(
      <I18nextProvider i18n={i18n}>
        <OverlayProvider container={overlay}>
          <AppRuntimeProvider client={client}>
            <RouterProvider router={router} />
          </AppRuntimeProvider>
        </OverlayProvider>
      </I18nextProvider>,
    );

    await waitFor(() => expect(listener).toBeTypeOf("function"));
    const emit = (path: string | null) => {
      const event = launch(path);
      currentSnapshot = snapshot(event);
      act(() => listener?.(event));
    };

    emit("/settings");
    await waitFor(() =>
      expect(screen.getByLabelText("当前路径").textContent).toBe("/settings"),
    );
    const settingsKey = router.state.location.key;

    emit("/settings");
    await waitFor(() => expect(router.state.location.key).toBe(settingsKey));
    emit(null);
    expect(screen.getByLabelText("当前路径").textContent).toBe("/settings");

    emit("/settings/runtime");
    await waitFor(() =>
      expect(screen.getByLabelText("当前路径").textContent).toBe("/overview"),
    );
    expect(screen.getByLabelText("导航警告").textContent).toContain(
      "/settings/runtime",
    );

    view.unmount();
  });
});
