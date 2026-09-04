// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import {
  createMemoryRouter,
  Outlet,
  RouterProvider,
  useLocation,
} from "react-router";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
  OverlayProvider,
} from "../../components/ui";
import { NavigationProvider, useAppNavigation } from "./navigation-provider";

afterEach(() => {
  cleanup();
  document
    .querySelectorAll("[data-navigation-overlay]")
    .forEach((element) => element.remove());
});

function TestLayout() {
  const navigation = useAppNavigation();
  return (
    <>
      <button
        type="button"
        onClick={(event) => {
          navigation.rememberFocus(event.currentTarget);
          navigation.navigateProductPath("/settings");
        }}
      >
        打开设置
      </button>
      <button
        type="button"
        onClick={() => navigation.navigateProductPath("/unknown")}
      >
        打开未知路径
      </button>
      <Dialog>
        <DialogTrigger asChild>
          <button type="button">打开对话框</button>
        </DialogTrigger>
        <DialogContent>
          <DialogTitle>确认操作</DialogTitle>
          <DialogDescription>对话框优先处理退出键。</DialogDescription>
        </DialogContent>
      </Dialog>
      <output>{navigation.warning}</output>
      <Outlet />
    </>
  );
}

function Page({
  title,
  editable = false,
}: {
  title: string;
  editable?: boolean;
}) {
  const location = useLocation();
  return (
    <main className="shell-route-main">
      <h1 data-route-heading tabIndex={-1}>
        {title}
      </h1>
      <span>{location.pathname}</span>
      {editable ? <input aria-label="编辑内容" /> : null}
    </main>
  );
}

function renderNavigation() {
  const overlay = document.createElement("div");
  overlay.dataset.navigationOverlay = "true";
  document.body.append(overlay);
  const router = createMemoryRouter(
    [
      {
        element: (
          <NavigationProvider>
            <TestLayout />
          </NavigationProvider>
        ),
        children: [
          { path: "/overview", element: <Page title="概览" /> },
          { path: "/settings", element: <Page title="设置" editable /> },
        ],
      },
    ],
    { initialEntries: ["/overview"] },
  );
  const result = render(
    <OverlayProvider container={overlay}>
      <RouterProvider router={router} />
    </OverlayProvider>,
  );
  return { ...result, overlay, router };
}

describe("NavigationProvider", () => {
  it("focuses headings after push and restores the remembered trigger on pop", async () => {
    const user = userEvent.setup();
    renderNavigation();
    const trigger = screen.getByRole("button", { name: "打开设置" });
    const routeMain = document.querySelector<HTMLElement>(".shell-route-main");
    if (routeMain) {
      routeMain.scrollTop = 180;
    }
    await user.click(trigger);
    await waitFor(() =>
      expect(document.activeElement).toBe(
        screen.getByRole("heading", { name: "设置" }),
      ),
    );
    expect(routeMain?.scrollTop).toBe(0);

    await user.keyboard("{Alt>}{ArrowLeft}{/Alt}");
    await waitFor(() => expect(screen.getByText("/overview")).toBeTruthy());
    expect(document.activeElement).toBe(trigger);
  });

  it("does not navigate on Escape from an editor or while an overlay is active", async () => {
    const user = userEvent.setup();
    const { overlay } = renderNavigation();
    await user.click(screen.getByRole("button", { name: "打开设置" }));
    const input = screen.getByRole("textbox", { name: "编辑内容" });
    input.focus();
    await user.keyboard("{Escape}");
    expect(screen.getByText("/settings")).toBeTruthy();

    const portal = document.createElement("div");
    overlay.append(portal);
    screen.getByRole("heading", { name: "设置" }).focus();
    await user.keyboard("{Escape}");
    expect(screen.getByText("/settings")).toBeTruthy();

    portal.remove();
    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByText("/overview")).toBeTruthy());
  });

  it("lets a real dialog consume Escape before the route history", async () => {
    const user = userEvent.setup();
    renderNavigation();
    await user.click(screen.getByRole("button", { name: "打开设置" }));
    await user.click(screen.getByRole("button", { name: "打开对话框" }));
    expect(screen.getByRole("dialog")).toBeTruthy();

    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(screen.getByText("/settings")).toBeTruthy();

    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.getByText("/overview")).toBeTruthy());
  });

  it("falls back to the destination heading when no trigger was recorded", async () => {
    const user = userEvent.setup();
    const { router } = renderNavigation();
    await router.navigate("/settings");
    await waitFor(() =>
      expect(document.activeElement).toBe(
        screen.getByRole("heading", { name: "设置" }),
      ),
    );

    await user.keyboard("{Alt>}{ArrowLeft}{/Alt}");
    await waitFor(() =>
      expect(document.activeElement).toBe(
        screen.getByRole("heading", { name: "概览" }),
      ),
    );
  });

  it("prevents Alt+Left from leaving the app at the session boundary", () => {
    renderNavigation();
    const event = new KeyboardEvent("keydown", {
      key: "ArrowLeft",
      altKey: true,
      bubbles: true,
      cancelable: true,
    });
    document.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    expect(screen.getByText("/overview")).toBeTruthy();
  });

  it("keeps a fallback warning on overview and clears it on later canonical navigation", async () => {
    const user = userEvent.setup();
    const { router } = renderNavigation();
    await user.click(screen.getByRole("button", { name: "打开未知路径" }));
    expect(screen.getByText(/无法识别内部路径/)).toBeTruthy();
    expect(screen.getByText("/overview")).toBeTruthy();

    await router.navigate("/settings");
    await waitFor(() => expect(screen.getByText("/settings")).toBeTruthy());
    await waitFor(() =>
      expect(screen.queryByText(/无法识别内部路径/)).toBeNull(),
    );
  });
});
