// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";

import { UiPreferencesProvider } from "../../app/providers/ui-preferences-provider";
import { i18n } from "../../i18n";
import type { UiConfigClient } from "../../lib/config-client";
import type { AppearanceEnvironment } from "../../lib/ui-appearance";
import type { ConfigSnapshot, UiConfig } from "../../types/generated/bindings";
import { SettingsRoute } from "./SettingsRoute";

afterEach(cleanup);

const snapshot = (revision: number, ui: UiConfig): ConfigSnapshot => ({
  revision,
  config: { schemaVersion: 1, ui },
  persistence: "ready",
  notices: [],
});

describe("SettingsRoute", () => {
  it("keeps eight groups honest and persists only real appearance controls", async () => {
    const user = userEvent.setup();
    let revision = 1;
    const updateUi = vi.fn(async (ui: UiConfig) => snapshot(++revision, ui));
    const client: UiConfigClient = {
      listenChanged: vi.fn(async () => vi.fn()),
      fetchSnapshot: vi.fn(async () =>
        snapshot(revision, {
          theme: "system",
          density: "standard",
          locale: "zh-CN",
          sidebarCollapsed: false,
        }),
      ),
      updateUi,
    };
    const appearance: AppearanceEnvironment = {
      root: document.createElement("html"),
    };
    render(
      <I18nextProvider i18n={i18n}>
        <UiPreferencesProvider
          client={client}
          appearanceEnvironment={appearance}
        >
          <SettingsRoute />
        </UiPreferencesProvider>
      </I18nextProvider>,
    );
    await waitFor(() =>
      expect(
        (
          screen.getByRole("radio", {
            name: "跟随系统",
          }) as HTMLInputElement
        ).checked,
      ).toBe(true),
    );

    expect(
      screen
        .getAllByRole("heading", { level: 2 })
        .map((heading) => heading.textContent),
    ).toEqual([
      "输入法",
      "五笔行为",
      "候选窗口",
      "快捷键",
      "外观",
      "网络",
      "数据",
      "关于",
    ]);
    expect(screen.getAllByRole("radio")).toHaveLength(5);
    expect(screen.getAllByRole("checkbox")).toHaveLength(1);
    expect(screen.queryByRole("button", { name: /保存/ })).toBeNull();
    expect(screen.queryByRole("combobox")).toBeNull();

    await user.click(screen.getByRole("radio", { name: "深色" }));
    await user.click(screen.getByRole("radio", { name: "紧凑" }));
    await user.click(screen.getByRole("checkbox", { name: /折叠侧栏/ }));
    await waitFor(() => expect(updateUi).toHaveBeenCalledTimes(3));
    expect(updateUi).toHaveBeenLastCalledWith({
      theme: "dark",
      density: "compact",
      locale: "zh-CN",
      sidebarCollapsed: true,
      onboardingVersion: 0,
    });
  });
});
