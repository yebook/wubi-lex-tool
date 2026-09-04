// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../i18n";
import type { RuntimeClient } from "../../lib/runtime-client";
import type {
  LaunchRequestedEvent,
  RuntimeSnapshot,
} from "../../types/generated/bindings";
import {
  AppRuntimeProvider,
  resolveInitialNavigation,
  useAppRuntime,
} from "./app-runtime-provider";
import type { AppRuntimeContextValue } from "./app-runtime-provider";

afterEach(cleanup);

const launch = (path: string | null): LaunchRequestedEvent => ({
  request: { startHidden: false, navigationPath: path },
  notices: [],
});

const snapshot = (
  primary = "/overview",
  secondary: string | null = null,
): RuntimeSnapshot => ({
  privilege: { state: "elevated", failure: null },
  previousAbnormalSessionCount: 0,
  primaryLaunch: launch(primary),
  latestSecondaryLaunch: secondary ? launch(secondary) : null,
  notices: [],
});

function wrapper(client: RuntimeClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <I18nextProvider i18n={i18n}>
        <AppRuntimeProvider client={client}>{children}</AppRuntimeProvider>
      </I18nextProvider>
    );
  };
}

describe("AppRuntimeProvider", () => {
  it("listens before snapshot and preserves an event received in flight", async () => {
    const order: string[] = [];
    let listener: ((event: LaunchRequestedEvent) => void) | undefined;
    let resolveSnapshot: ((value: RuntimeSnapshot) => void) | undefined;
    const pending = new Promise<RuntimeSnapshot>((resolve) => {
      resolveSnapshot = resolve;
    });
    const client: RuntimeClient = {
      listenLaunch: vi.fn(async (next) => {
        order.push("listen");
        listener = next;
        return vi.fn();
      }),
      fetchSnapshot: vi.fn(() => {
        order.push("snapshot");
        return pending;
      }),
    };
    const { result } = renderHook(() => useAppRuntime(), {
      wrapper: wrapper(client),
    });

    await waitFor(() => expect(order).toEqual(["listen", "snapshot"]));
    act(() => listener?.(launch("/settings")));
    await act(async () => {
      resolveSnapshot?.(snapshot("/overview"));
      await pending;
    });
    await waitFor(() => expect(result.current.loadState.status).toBe("ready"));

    expect(client.fetchSnapshot).toHaveBeenCalledOnce();
    expect(result.current.latestLaunch?.sequence).toBe(1);
    expect(
      result.current.latestNavigationLaunch?.event.request.navigationPath,
    ).toBe("/settings");
    expect(
      result.current.loadState.status === "ready"
        ? result.current.loadState.snapshot.latestSecondaryLaunch?.request
            .navigationPath
        : null,
    ).toBe("/settings");
  });

  it("cleans a listener that resolves after unmount", async () => {
    let resolveListener: ((stop: () => void) => void) | undefined;
    const pending = new Promise<() => void>((resolve) => {
      resolveListener = resolve;
    });
    const stop = vi.fn();
    const client: RuntimeClient = {
      listenLaunch: vi.fn(() => pending),
      fetchSnapshot: vi.fn(async () => snapshot()),
    };
    const { unmount } = renderHook(() => useAppRuntime(), {
      wrapper: wrapper(client),
    });
    unmount();
    await act(async () => {
      resolveListener?.(stop);
      await pending;
    });
    expect(stop).toHaveBeenCalledOnce();
    expect(client.fetchSnapshot).not.toHaveBeenCalled();
  });

  it("ignores an older refresh that resolves after a newer request", async () => {
    const pending: Array<{
      promise: Promise<RuntimeSnapshot>;
      resolve: (value: RuntimeSnapshot) => void;
    }> = [];
    const client: RuntimeClient = {
      listenLaunch: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(() => {
        let resolve: (value: RuntimeSnapshot) => void = () => {};
        const promise = new Promise<RuntimeSnapshot>((next) => {
          resolve = next;
        });
        pending.push({ promise, resolve });
        return promise;
      }),
    };
    const { result } = renderHook(() => useAppRuntime(), {
      wrapper: wrapper(client),
    });

    await waitFor(() => expect(pending).toHaveLength(1));
    await act(async () => {
      pending[0]?.resolve(snapshot("/overview"));
      await pending[0]?.promise;
    });

    let olderRefresh: Promise<void> | undefined;
    let newerRefresh: Promise<void> | undefined;
    act(() => {
      olderRefresh = result.current.refresh(false);
      newerRefresh = result.current.refresh(false);
    });
    await waitFor(() => expect(pending).toHaveLength(3));
    await act(async () => {
      pending[2]?.resolve(snapshot("/settings"));
      await newerRefresh;
    });
    await act(async () => {
      pending[1]?.resolve(snapshot("/lexicons"));
      await olderRefresh;
    });

    expect(
      result.current.loadState.status === "ready"
        ? result.current.loadState.snapshot.primaryLaunch.request.navigationPath
        : null,
    ).toBe("/settings");
  });

  it("bounds an initial snapshot failure before exposing it", async () => {
    const client: RuntimeClient = {
      listenLaunch: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () => {
        throw new Error("错".repeat(600));
      }),
    };
    const { result } = renderHook(() => useAppRuntime(), {
      wrapper: wrapper(client),
    });

    await waitFor(() => expect(result.current.loadState.status).toBe("error"));
    expect(
      result.current.loadState.status === "error"
        ? [...result.current.loadState.message]
        : [],
    ).toHaveLength(512);
  });
});

describe("resolveInitialNavigation", () => {
  function runtime(
    overrides: Partial<AppRuntimeContextValue>,
  ): AppRuntimeContextValue {
    return {
      loadState: {
        status: "ready",
        snapshot: snapshot("/lexicons", "/phrases"),
      },
      latestLaunch: null,
      latestNavigationLaunch: null,
      listenerWarning: null,
      refreshWarning: null,
      refresh: vi.fn(),
      ...overrides,
    };
  }

  it("uses the latest startup event with a path before snapshot and hash paths", () => {
    const result = resolveInitialNavigation(
      runtime({
        latestLaunch: { sequence: 2, event: launch(null) },
        latestNavigationLaunch: { sequence: 1, event: launch("/settings") },
      }),
      "/learning",
    );
    expect(result).toEqual({
      path: "/settings",
      warning: null,
      consumedLaunchSequence: 2,
    });
  });

  it("falls back unknown product paths to overview with a warning", () => {
    const result = resolveInitialNavigation(
      runtime({ loadState: { status: "error", message: "offline" } }),
      "/unknown",
    );
    expect(result.path).toBe("/overview");
    expect(result.warning).toContain("/unknown");
  });
});
