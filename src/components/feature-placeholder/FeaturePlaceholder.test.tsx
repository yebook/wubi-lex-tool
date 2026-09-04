// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";
import { I18nextProvider } from "react-i18next";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../i18n";
import { featuresStore } from "../../stores/features";
import type { AppFeature } from "../../types/generated/bindings";
import { FeatureGate } from "./FeatureGate";
import { FeaturePlaceholder } from "./FeaturePlaceholder";

const initialFeatureState = featuresStore.getInitialState();

afterEach(() => {
  cleanup();
  featuresStore.setState(initialFeatureState, true);
});

function renderWithI18n(element: ReactElement) {
  return render(element, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
    ),
  });
}

const record = (available: boolean): AppFeature => ({
  id: "lexiconRead",
  available,
  targetMilestone: "s2",
  unavailableReason: available ? null : "notIncludedInBuild",
});

function gate() {
  return (
    <FeatureGate
      feature="lexiconRead"
      variant="page"
      title="码表读取"
      description="查看已安装码表"
    >
      <div>真实能力内容</div>
    </FeatureGate>
  );
}

describe("FeaturePlaceholder", () => {
  it("renders page, section, and inline contracts with non-color status", () => {
    renderWithI18n(
      <>
        {(["page", "section", "inline"] as const).map((variant) => (
          <FeaturePlaceholder
            key={variant}
            variant={variant}
            title={`${variant} 能力`}
            description="能力说明"
            milestone="s4"
          />
        ))}
      </>,
    );
    expect(screen.getAllByText("功能暂未完善")).toHaveLength(3);
    expect(screen.getAllByText("计划阶段 S4")).toHaveLength(3);
    expect(
      document.querySelectorAll("[data-placeholder-variant]"),
    ).toHaveLength(3);
  });
});

describe("FeatureGate", () => {
  it("renders stable loading and unavailable states", () => {
    featuresStore.setState({ status: "loading", catalog: { features: [] } });
    const { rerender } = renderWithI18n(gate());
    expect(screen.getByRole("status").getAttribute("aria-busy")).toBe("true");

    featuresStore.setState({
      status: "ready",
      catalog: { features: [record(false)] },
      error: null,
    });
    rerender(gate());
    expect(screen.getByText("功能暂未完善")).toBeTruthy();
    expect(screen.getByText("计划阶段 S2")).toBeTruthy();
  });

  it("fails closed for errors and missing records and can retry", async () => {
    const user = userEvent.setup();
    const retry = vi.fn(async () => {});
    featuresStore.setState({
      status: "failed",
      catalog: { features: [] },
      error: { message: "目录暂时不可用" },
      retry,
    });
    const { rerender } = renderWithI18n(gate());
    expect(screen.getByRole("alert").textContent).toContain("目录暂时不可用");
    await user.click(screen.getByRole("button", { name: "重新读取" }));
    expect(retry).toHaveBeenCalledOnce();

    featuresStore.setState({ status: "ready", error: null });
    rerender(gate());
    expect(screen.getByRole("alert").textContent).toContain("lexiconRead");
  });

  it("renders supplied children only when the backend record is available", () => {
    featuresStore.setState({
      status: "ready",
      catalog: { features: [record(true)] },
      error: null,
    });
    renderWithI18n(gate());
    expect(screen.getByText("真实能力内容")).toBeTruthy();
    expect(screen.queryByText("功能暂未完善")).toBeNull();
  });
});
