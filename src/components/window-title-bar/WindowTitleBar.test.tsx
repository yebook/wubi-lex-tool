// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../i18n";
import type { WindowStateSnapshot } from "../../types/generated/bindings";
import { WindowTitleBar } from "./WindowTitleBar";

afterEach(cleanup);

const visible: WindowStateSnapshot = {
  revision: 1,
  visibility: "visible",
  maximized: false,
};

function renderWithI18n(element: ReactElement) {
  return render(element, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
    ),
  });
}

describe("WindowTitleBar", () => {
  it("renders accessible native controls outside the drag region", () => {
    const { container } = renderWithI18n(
      <WindowTitleBar
        iconUrl="/icon.ico"
        version="0.1.0"
        snapshot={visible}
        onControl={() => {}}
      />,
    );

    const minimize = screen.getByRole("button", { name: "最小化到托盘" });
    const maximize = screen.getByRole("button", { name: "最大化窗口" });
    const close = screen.getByRole("button", { name: "关闭窗口" });
    expect(minimize.getAttribute("title")).toBe("最小化到托盘");
    expect(maximize.getAttribute("title")).toBe("最大化窗口");
    expect(close.getAttribute("title")).toBe("关闭窗口");
    expect(minimize.classList.contains("window-control-button")).toBe(true);
    expect(minimize.hasAttribute("data-tauri-drag-region")).toBe(false);
    expect(maximize.hasAttribute("data-tauri-drag-region")).toBe(false);
    expect(close.hasAttribute("data-tauri-drag-region")).toBe(false);
    expect(
      container
        .querySelector(".window-drag-region")
        ?.getAttribute("data-tauri-drag-region"),
    ).toBe("deep");
    expect(screen.getByText("v0.1.0")).toBeTruthy();
  });

  it("submits all intents from keyboard-operable buttons", async () => {
    const user = userEvent.setup();
    const onControl = vi.fn();
    renderWithI18n(
      <WindowTitleBar
        iconUrl="/icon.ico"
        version="0.1.0"
        snapshot={visible}
        onControl={onControl}
      />,
    );

    screen.getByRole("button", { name: "最小化到托盘" }).focus();
    await user.keyboard("{Enter}");
    screen.getByRole("button", { name: "最大化窗口" }).focus();
    await user.keyboard(" ");
    await user.click(screen.getByRole("button", { name: "关闭窗口" }));
    expect(onControl.mock.calls.map(([intent]) => intent)).toEqual([
      "minimizeToTray",
      "toggleMaximize",
      "close",
    ]);
  });

  it("switches to restore semantics and disables controls while exiting", () => {
    const { rerender } = renderWithI18n(
      <WindowTitleBar
        iconUrl="/icon.ico"
        version="0.1.0"
        snapshot={{ ...visible, revision: 2, maximized: true }}
        onControl={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: "还原窗口" })).toBeTruthy();

    rerender(
      <WindowTitleBar
        iconUrl="/icon.ico"
        version="0.1.0"
        snapshot={{ ...visible, revision: 3, visibility: "exiting" }}
        onControl={() => {}}
      />,
    );
    for (const button of screen.getAllByRole("button")) {
      expect((button as HTMLButtonElement).disabled).toBe(true);
    }
  });
});
