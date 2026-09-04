// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it } from "vitest";

import { i18n } from "../../i18n";
import { StatusBar } from "./StatusBar";

afterEach(cleanup);

function renderWithI18n(element: ReactElement) {
  return render(element, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
    ),
  });
}

describe("StatusBar", () => {
  it("distinguishes ready, loading, and bounded warning states", () => {
    const { rerender } = renderWithI18n(
      <StatusBar loading={false} warning={null} />,
    );
    expect(screen.getByRole("status").textContent).toContain("应用已就绪");

    rerender(<StatusBar loading warning={null} />);
    const loading = screen.getByRole("status");
    expect(loading.textContent).toContain("正在准备应用");
    expect(loading.querySelector(".lucide-loader-circle")).not.toBeNull();

    rerender(<StatusBar loading={false} warning={"警".repeat(600)} />);
    const status = screen.getByRole("status");
    expect(status.classList.contains("shell-status-warning")).toBe(true);
    expect([...(status.textContent ?? "")]).toHaveLength(512);
  });
});
