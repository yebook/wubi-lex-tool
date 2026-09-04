// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../i18n";
import type { UiConfigClient } from "../../lib/config-client";
import type { AppearanceEnvironment } from "../../lib/ui-appearance";
import type { ConfigSnapshot, UiConfig } from "../../types/generated/bindings";
import {
  UiPreferencesProvider,
  useUiPreferences,
} from "./ui-preferences-provider";

afterEach(cleanup);

function snapshot(revision: number, ui: UiConfig): ConfigSnapshot {
  return {
    revision,
    config: { schemaVersion: 1, ui },
    persistence: "ready",
    notices: [],
  };
}

function environment(): AppearanceEnvironment {
  return { root: document.createElement("html") };
}

function wrapper(
  client: UiConfigClient,
  appearanceEnvironment: AppearanceEnvironment,
) {
  return function Wrapper({ children }: { children: ReactNode }) {
    const content = (
      <I18nextProvider i18n={i18n}>
        <UiPreferencesProvider
          client={client}
          appearanceEnvironment={appearanceEnvironment}
        >
          {children}
        </UiPreferencesProvider>
      </I18nextProvider>
    );
    return content;
  };
}

describe("UiPreferencesProvider", () => {
  it("does not persist a full UI group before an authoritative snapshot", async () => {
    let resolveSnapshot: ((value: ConfigSnapshot) => void) | undefined;
    const pendingSnapshot = new Promise<ConfigSnapshot>((resolve) => {
      resolveSnapshot = resolve;
    });
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(() => pendingSnapshot),
      updateUi: vi.fn(),
    };
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, environment()),
    });

    await act(async () => result.current.setSidebarCollapsed(true));
    expect(result.current.ui.sidebarCollapsed).toBe(false);
    expect(result.current.warning).toBe("界面设置读取完成前无法保存更改。");
    expect(client.updateUi).not.toHaveBeenCalled();

    await act(async () => {
      resolveSnapshot?.(snapshot(1, { sidebarCollapsed: true }));
      await pendingSnapshot;
    });
  });

  it("listens before snapshot and rejects an older snapshot", async () => {
    const order: string[] = [];
    let listener: ((incoming: ConfigSnapshot) => void) | undefined;
    let resolveSnapshot: ((value: ConfigSnapshot) => void) | undefined;
    const pendingSnapshot = new Promise<ConfigSnapshot>((resolve) => {
      resolveSnapshot = resolve;
    });
    const client: UiConfigClient = {
      listenChanged: vi.fn(async (next) => {
        order.push("listen");
        listener = next;
        return vi.fn();
      }),
      fetchSnapshot: vi.fn(() => {
        order.push("snapshot");
        return pendingSnapshot;
      }),
      updateUi: vi.fn(),
    };
    const appearance = environment();
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, appearance),
    });

    await waitFor(() => expect(order).toEqual(["listen", "snapshot"]));
    act(() => listener?.(snapshot(2, { theme: "dark", density: "compact" })));
    await waitFor(() => expect(result.current.ui.theme).toBe("dark"));
    await act(async () => {
      resolveSnapshot?.(snapshot(1, { theme: "light" }));
      await pendingSnapshot;
    });

    expect(result.current.status).toBe("ready");
    expect(result.current.ui).toMatchObject({
      theme: "dark",
      density: "compact",
      locale: "zh-CN",
    });
    expect(appearance.root.classList.contains("dark")).toBe(true);
  });

  it("serializes full-group updates while applying patches optimistically", async () => {
    const pendingUpdates: Array<{
      ui: UiConfig;
      resolve: (value: ConfigSnapshot) => void;
    }> = [];
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () =>
        snapshot(1, {
          theme: "light",
          density: "standard",
          locale: "zh-CN",
          sidebarCollapsed: true,
          onboardingVersion: 3,
        }),
      ),
      updateUi: vi.fn(
        (ui) =>
          new Promise<ConfigSnapshot>((resolve) => {
            pendingUpdates.push({ ui, resolve });
          }),
      ),
    };
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, environment()),
    });
    await waitFor(() => expect(result.current.status).toBe("ready"));

    let themeUpdate: Promise<void> | undefined;
    let densityUpdate: Promise<void> | undefined;
    act(() => {
      themeUpdate = result.current.setTheme("dark");
      densityUpdate = result.current.setDensity("compact");
    });
    expect(result.current.ui).toMatchObject({
      theme: "dark",
      density: "compact",
    });
    await waitFor(() => expect(pendingUpdates).toHaveLength(1));
    expect(pendingUpdates[0]?.ui).toEqual({
      theme: "dark",
      density: "standard",
      locale: "zh-CN",
      sidebarCollapsed: true,
      onboardingVersion: 3,
    });

    await act(async () => {
      pendingUpdates[0]?.resolve(snapshot(2, pendingUpdates[0].ui));
      await themeUpdate;
    });
    await waitFor(() => expect(pendingUpdates).toHaveLength(2));
    expect(pendingUpdates[1]?.ui).toEqual({
      theme: "dark",
      density: "compact",
      locale: "zh-CN",
      sidebarCollapsed: true,
      onboardingVersion: 3,
    });
    await act(async () => {
      pendingUpdates[1]?.resolve(snapshot(3, pendingUpdates[1].ui));
      await densityUpdate;
    });
    expect(result.current.ui).toMatchObject({
      theme: "dark",
      density: "compact",
    });
  });

  it("rolls a failed update back to the latest confirmed UI", async () => {
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () => snapshot(4, { theme: "light" })),
      updateUi: vi.fn(async () => {
        throw new Error("保存通道不可用");
      }),
    };
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, environment()),
    });
    await waitFor(() => expect(result.current.status).toBe("ready"));

    await act(async () => result.current.setTheme("dark"));
    expect(result.current.ui.theme).toBe("light");
    expect(result.current.warning).toBe("保存通道不可用");
  });

  it("persists sidebar state through the same full-group queue", async () => {
    const updateUi = vi.fn(async (ui: UiConfig) => snapshot(2, ui));
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () =>
        snapshot(1, {
          theme: "dark",
          density: "compact",
          locale: "zh-CN",
          sidebarCollapsed: false,
          onboardingVersion: 4,
        }),
      ),
      updateUi,
    };
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, environment()),
    });
    await waitFor(() => expect(result.current.status).toBe("ready"));

    await act(async () => result.current.setSidebarCollapsed(true));
    expect(result.current.ui.sidebarCollapsed).toBe(true);
    expect(updateUi).toHaveBeenCalledWith({
      theme: "dark",
      density: "compact",
      locale: "zh-CN",
      sidebarCollapsed: true,
      onboardingVersion: 4,
    });
  });

  it("synchronizes explicit and system preferences to the native window theme", async () => {
    const setNativeTheme = vi.fn(async () => {});
    const appearance = {
      ...environment(),
      setNativeTheme,
    } satisfies AppearanceEnvironment;
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () => snapshot(1, { theme: "dark" })),
      updateUi: vi.fn(async (ui) => snapshot(2, ui)),
    };
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, appearance),
    });

    await waitFor(() =>
      expect(setNativeTheme).toHaveBeenLastCalledWith("dark"),
    );
    await act(async () => result.current.setTheme("system"));
    await waitFor(() => expect(setNativeTheme).toHaveBeenLastCalledWith(null));
  });

  it("keeps a native theme synchronization failure visible and bounded", async () => {
    const appearance = {
      ...environment(),
      setNativeTheme: vi.fn(async (theme) => {
        if (theme === "dark") {
          throw new Error("x".repeat(600));
        }
      }),
    } satisfies AppearanceEnvironment;
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () => snapshot(1, { theme: "dark" })),
      updateUi: vi.fn(),
    };
    const { result } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, appearance),
    });

    await waitFor(() => expect(result.current.warning).toHaveLength(512));
  });

  it("cleans every StrictMode subscription without leaving a live listener", async () => {
    const stops: Array<ReturnType<typeof vi.fn>> = [];
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => {
        const stop = vi.fn();
        stops.push(stop);
        return stop;
      }),
      fetchSnapshot: vi.fn(async () => snapshot(1, {})),
      updateUi: vi.fn(),
    };
    const { unmount } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, environment()),
      reactStrictMode: true,
    });
    await waitFor(() => expect(stops).toHaveLength(2));
    unmount();
    expect(stops.every((stop) => stop.mock.calls.length === 1)).toBe(true);
  });

  it("cleans a listener that resolves after unmount", async () => {
    let resolveListener: ((stop: () => void) => void) | undefined;
    const listener = new Promise<() => void>((resolve) => {
      resolveListener = resolve;
    });
    const stop = vi.fn();
    const client: UiConfigClient = {
      listenChanged: vi.fn(() => listener),
      fetchSnapshot: vi.fn(async () => snapshot(1, {})),
      updateUi: vi.fn(),
    };
    const { unmount } = renderHook(() => useUiPreferences(), {
      wrapper: wrapper(client, environment()),
    });

    unmount();
    await act(async () => {
      resolveListener?.(stop);
      await listener;
    });
    expect(stop).toHaveBeenCalledOnce();
    expect(client.fetchSnapshot).not.toHaveBeenCalled();
  });
});
