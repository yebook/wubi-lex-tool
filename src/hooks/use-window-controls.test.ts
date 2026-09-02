// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { createElement } from "react";
import type { ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../i18n";
import type { WindowClient } from "../lib/window-client";
import type {
  RuntimeNotice,
  WindowStateSnapshot,
} from "../types/generated/bindings";
import { mergeWindowState, useWindowControls } from "./use-window-controls";

afterEach(cleanup);

const initial: WindowStateSnapshot = {
  revision: 1,
  visibility: "visible",
  maximized: false,
};

function I18nTestProvider({ children }: { children: ReactNode }) {
  return createElement(I18nextProvider, { i18n }, children);
}

describe("useWindowControls", () => {
  it("registers listeners before snapshot and rejects a stale bootstrap response", async () => {
    const order: string[] = [];
    let stateListener: ((snapshot: WindowStateSnapshot) => void) | undefined;
    let resolveSnapshot: ((snapshot: WindowStateSnapshot) => void) | undefined;
    const fetchState = new Promise<WindowStateSnapshot>((resolve) => {
      resolveSnapshot = resolve;
    });
    const client: WindowClient = {
      listenState: vi.fn(async (listener) => {
        order.push("state-listener");
        stateListener = listener;
        return vi.fn();
      }),
      listenNotice: vi.fn(async () => {
        order.push("notice-listener");
        return vi.fn();
      }),
      fetchState: vi.fn(() => {
        order.push("snapshot");
        return fetchState;
      }),
      control: vi.fn(async () => initial),
    };

    const { result } = renderHook(() => useWindowControls(client), {
      wrapper: I18nTestProvider,
    });
    await waitFor(() =>
      expect(order).toEqual(["state-listener", "notice-listener", "snapshot"]),
    );
    act(() => {
      stateListener?.({ revision: 2, visibility: "hidden", maximized: true });
    });
    await act(async () => {
      resolveSnapshot?.(initial);
      await fetchState;
    });

    expect(result.current.snapshot).toEqual({
      revision: 2,
      visibility: "hidden",
      maximized: true,
    });
  });

  it("merges live notices, reports command failures and cleans up listeners", async () => {
    const stopState = vi.fn();
    const stopNotice = vi.fn();
    let noticeListener: ((notice: RuntimeNotice) => void) | undefined;
    const client: WindowClient = {
      listenState: vi.fn(async () => stopState),
      listenNotice: vi.fn(async (listener) => {
        noticeListener = listener;
        return stopNotice;
      }),
      fetchState: vi.fn(async () => initial),
      control: vi.fn(async () => {
        throw new Error("窗口命令失败");
      }),
    };
    const { result, unmount } = renderHook(() => useWindowControls(client), {
      wrapper: I18nTestProvider,
    });
    await waitFor(() => expect(result.current.snapshot).toEqual(initial));

    const notice: RuntimeNotice = {
      code: "trayUnavailable",
      summary: "托盘不可用",
      detail: "失败阶段：create_tray。",
    };
    act(() => {
      noticeListener?.(notice);
      noticeListener?.(notice);
    });
    await act(async () => result.current.control("close"));
    expect(result.current.notices).toEqual([notice]);
    expect(result.current.warning).toBe("窗口命令失败");

    unmount();
    expect(stopState).toHaveBeenCalledOnce();
    expect(stopNotice).toHaveBeenCalledOnce();
  });

  it("keeps the higher revision in the pure merge helper", () => {
    const newer = { ...initial, revision: 4, maximized: true };
    expect(mergeWindowState(newer, initial)).toBe(newer);
    expect(mergeWindowState(initial, newer)).toBe(newer);
  });
});
