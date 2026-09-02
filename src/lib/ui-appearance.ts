import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Theme } from "@tauri-apps/api/window";

import type {
  Density,
  ThemePreference,
  UiConfig,
} from "../types/generated/bindings";

const DARK_QUERY = "(prefers-color-scheme: dark)";

export type ResolvedUiConfig = Required<UiConfig>;

export interface AppearanceMediaQuery {
  readonly matches: boolean;
  addEventListener(type: "change", listener: () => void): void;
  removeEventListener(type: "change", listener: () => void): void;
}

export interface AppearanceEnvironment {
  root: HTMLElement;
  matchMedia?: (query: string) => AppearanceMediaQuery;
  setNativeTheme?: (theme: Theme | null) => Promise<void>;
}

export function normalizeUiConfig(ui: UiConfig | undefined): ResolvedUiConfig {
  return {
    theme: ui?.theme ?? "system",
    density: ui?.density ?? "standard",
    locale: ui?.locale ?? "zh-CN",
    sidebarCollapsed: ui?.sidebarCollapsed ?? false,
    onboardingVersion: ui?.onboardingVersion ?? 0,
  };
}

export function readBootstrapUi(root: HTMLElement): ResolvedUiConfig {
  return normalizeUiConfig({
    theme: readTheme(root.dataset.theme),
    density: readDensity(root.dataset.density),
    locale: root.lang === "zh-CN" ? root.lang : "zh-CN",
  });
}

export function applyUiAppearance(
  ui: ResolvedUiConfig,
  environment: AppearanceEnvironment,
): void {
  const systemDark =
    ui.theme === "system" &&
    (environment.matchMedia?.(DARK_QUERY).matches ?? false);
  const dark = ui.theme === "dark" || systemDark;

  environment.root.dataset.theme = ui.theme;
  environment.root.dataset.density = ui.density;
  environment.root.lang = ui.locale;
  environment.root.classList.toggle("dark", dark);
  environment.root.style.colorScheme =
    ui.theme === "system" ? "light dark" : ui.theme;
}

export function synchronizeUiAppearance(
  ui: ResolvedUiConfig,
  environment: AppearanceEnvironment,
): () => void {
  applyUiAppearance(ui, environment);
  if (ui.theme !== "system" || !environment.matchMedia) {
    return () => {};
  }

  const media = environment.matchMedia(DARK_QUERY);
  const applySystemTheme = () => applyUiAppearance(ui, environment);
  media.addEventListener("change", applySystemTheme);
  return () => media.removeEventListener("change", applySystemTheme);
}

export function browserAppearanceEnvironment(): AppearanceEnvironment {
  const environment: AppearanceEnvironment = {
    root: document.documentElement,
    matchMedia: window.matchMedia?.bind(window),
  };
  return isTauri()
    ? {
        ...environment,
        setNativeTheme: (theme) => getCurrentWindow().setTheme(theme),
      }
    : environment;
}

export function nativeThemeForPreference(theme: ThemePreference): Theme | null {
  return theme === "system" ? null : theme;
}

function readTheme(value: string | undefined): ThemePreference {
  return value === "light" || value === "dark" ? value : "system";
}

function readDensity(value: string | undefined): Density {
  return value === "compact" ? value : "standard";
}
