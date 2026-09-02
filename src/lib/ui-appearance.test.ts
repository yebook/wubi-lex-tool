// @vitest-environment jsdom

import { afterEach, describe, expect, it, vi } from "vitest";

import {
  applyUiAppearance,
  nativeThemeForPreference,
  normalizeUiConfig,
  readBootstrapUi,
  synchronizeUiAppearance,
} from "./ui-appearance";
import type { AppearanceMediaQuery } from "./ui-appearance";

afterEach(() => {
  document.documentElement.className = "";
  document.documentElement.removeAttribute("data-theme");
  document.documentElement.removeAttribute("data-density");
  document.documentElement.lang = "";
  document.documentElement.style.colorScheme = "";
});

describe("UI appearance projection", () => {
  it("maps explicit and system preferences to the native window contract", () => {
    expect(nativeThemeForPreference("light")).toBe("light");
    expect(nativeThemeForPreference("dark")).toBe("dark");
    expect(nativeThemeForPreference("system")).toBeNull();
  });

  it("normalizes missing generated fields and reads constrained bootstrap attributes", () => {
    const root = document.documentElement;
    root.dataset.theme = "dark";
    root.dataset.density = "compact";
    root.lang = "zh-CN";

    expect(readBootstrapUi(root)).toEqual({
      theme: "dark",
      density: "compact",
      locale: "zh-CN",
      sidebarCollapsed: false,
      onboardingVersion: 0,
    });
    expect(normalizeUiConfig(undefined)).toMatchObject({
      theme: "system",
      density: "standard",
      locale: "zh-CN",
    });
  });

  it("projects explicit themes without consulting the system preference", () => {
    const matchMedia = vi.fn();
    const environment = { root: document.documentElement, matchMedia };

    applyUiAppearance(normalizeUiConfig({ theme: "dark" }), environment);
    expect(environment.root.classList.contains("dark")).toBe(true);
    expect(environment.root.dataset.theme).toBe("dark");
    expect(environment.root.style.colorScheme).toBe("dark");

    applyUiAppearance(normalizeUiConfig({ theme: "light" }), environment);
    expect(environment.root.classList.contains("dark")).toBe(false);
    expect(matchMedia).not.toHaveBeenCalled();
  });

  it("tracks and cleans up the system color-scheme listener", () => {
    let matches = true;
    const listeners = new Set<() => void>();
    const media: AppearanceMediaQuery = {
      get matches() {
        return matches;
      },
      addEventListener: (_type, listener) => listeners.add(listener),
      removeEventListener: (_type, listener) => listeners.delete(listener),
    };
    const environment = {
      root: document.documentElement,
      matchMedia: vi.fn(() => media),
    };

    const stop = synchronizeUiAppearance(
      normalizeUiConfig({ theme: "system", density: "compact" }),
      environment,
    );
    expect(environment.root.classList.contains("dark")).toBe(true);
    expect(environment.root.dataset.density).toBe("compact");
    expect(listeners.size).toBe(1);

    matches = false;
    for (const listener of listeners) listener();
    expect(environment.root.classList.contains("dark")).toBe(false);

    stop();
    expect(listeners.size).toBe(0);
  });
});
