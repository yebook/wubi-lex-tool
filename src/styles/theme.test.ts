/// <reference types="node" />

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const runtimeCss = readFileSync(
  new URL("runtime-status.css", import.meta.url),
  "utf8",
);
const shellCss = readFileSync(new URL("shell.css", import.meta.url), "utf8");
const themeCss = readFileSync(new URL("theme.css", import.meta.url), "utf8");

const COLOR_TOKENS = [
  "primary",
  "primary-hover",
  "primary-subtle",
  "on-primary",
  "surface-1",
  "surface-2",
  "surface-3",
  "border",
  "border-strong",
  "text-1",
  "text-2",
  "text-3",
  "success",
  "warning",
  "danger",
  "on-danger",
  "info",
  "focus",
  "zone-1",
  "zone-2",
  "zone-3",
  "zone-4",
  "zone-5",
] as const;

function ruleBody(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = themeCss.match(new RegExp(`${escaped}\\s*\\{([^}]+)\\}`));
  expect(match, `missing ${selector} token rule`).not.toBeNull();
  return match?.[1] ?? "";
}

function tokenValue(body: string, name: string): string {
  const match = body.match(new RegExp(`--wl-${name}:\\s*(#[0-9a-f]{6})`, "i"));
  expect(match, `missing --wl-${name}`).not.toBeNull();
  return match?.[1] ?? "#000000";
}

function relativeLuminance(hex: string): number {
  const channels = hex
    .slice(1)
    .match(/.{2}/g)
    ?.map((channel) => Number.parseInt(channel, 16) / 255) ?? [0, 0, 0];
  const linear = channels.map((channel) =>
    channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4,
  );
  return (
    0.2126 * (linear[0] ?? 0) +
    0.7152 * (linear[1] ?? 0) +
    0.0722 * (linear[2] ?? 0)
  );
}

function contrastRatio(first: string, second: string): number {
  const [lighter, darker] = [
    relativeLuminance(first),
    relativeLuminance(second),
  ].sort((left, right) => right - left);
  return ((lighter ?? 0) + 0.05) / ((darker ?? 0) + 0.05);
}

describe("Tailwind v4 theme contract", () => {
  it("uses the CSS-first inline theme and class/data variants", () => {
    expect(themeCss).toContain('@import "tailwindcss"');
    expect(themeCss).toContain("@theme inline");
    expect(themeCss).toContain("@custom-variant dark");
    expect(themeCss).toContain("@custom-variant compact");
    expect(themeCss).toContain(':root[data-density="compact"]');
    expect(themeCss).toContain("prefers-reduced-motion: reduce");
  });

  it("defines every semantic color in light and dark themes", () => {
    const light = ruleBody(":root");
    const dark = ruleBody(":root.dark");
    const systemDark = ruleBody(
      ':root:not([data-theme="light"]):not([data-theme="dark"])',
    );

    for (const token of COLOR_TOKENS) {
      expect(tokenValue(light, token)).toMatch(/^#[0-9a-f]{6}$/i);
      expect(tokenValue(dark, token)).toMatch(/^#[0-9a-f]{6}$/i);
      expect(tokenValue(systemDark, token)).toBe(tokenValue(dark, token));
    }
  });

  it.each([
    [":root", "text-1", "surface-1", 4.5],
    [":root", "text-2", "surface-1", 4.5],
    [":root", "text-3", "surface-1", 4.5],
    [":root", "on-primary", "primary", 4.5],
    [":root", "on-danger", "danger", 4.5],
    [":root", "focus", "surface-1", 3],
    [":root.dark", "text-1", "surface-1", 4.5],
    [":root.dark", "text-2", "surface-1", 4.5],
    [":root.dark", "text-3", "surface-1", 4.5],
    [":root.dark", "on-primary", "primary", 4.5],
    [":root.dark", "on-danger", "danger", 4.5],
    [":root.dark", "focus", "surface-1", 3],
  ])(
    "keeps %s %s legible against %s",
    (selector, foreground, background, minimum) => {
      const body = ruleBody(selector);
      expect(
        contrastRatio(
          tokenValue(body, foreground),
          tokenValue(body, background),
        ),
      ).toBeGreaterThanOrEqual(minimum);
    },
  );

  it.each([
    [":root", "border-strong", "surface-1"],
    [":root.dark", "border-strong", "surface-2"],
  ])("keeps %s %s visible against %s", (selector, border, surface) => {
    const body = ruleBody(selector);
    expect(
      contrastRatio(tokenValue(body, border), tokenValue(body, surface)),
    ).toBeGreaterThanOrEqual(3);
  });

  it("keeps the runtime surface on the shared token vocabulary", () => {
    expect(runtimeCss).not.toMatch(
      /var\(--(?:page|surface|surface-muted|text|text-muted|border|accent|positive|critical|focus)\b/,
    );
    expect(runtimeCss).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    expect(runtimeCss).not.toContain("prefers-color-scheme");
  });

  it("keeps shell dimensions and colors on the shared token vocabulary", () => {
    for (const token of [
      "sidebar-expanded",
      "sidebar-collapsed",
      "statusbar-min-height",
      "route-max-width",
      "placeholder-min-height",
    ]) {
      expect(themeCss).toContain(`--spacing-${token}: var(--wl-${token});`);
      expect(shellCss).toContain(`var(--wl-${token})`);
    }
    expect(shellCss).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    expect(shellCss).not.toMatch(/letter-spacing:\s*-/);
    expect(shellCss).not.toContain("prefers-color-scheme");
  });
});
