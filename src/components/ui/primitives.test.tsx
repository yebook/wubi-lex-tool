// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";

import { i18n } from "../../i18n";
import { I18nextProvider } from "react-i18next";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Input,
  Kbd,
  OverlayProvider,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from ".";

beforeAll(() => {
  Object.defineProperty(HTMLElement.prototype, "hasPointerCapture", {
    configurable: true,
    value: () => false,
  });
  Object.defineProperty(HTMLElement.prototype, "setPointerCapture", {
    configurable: true,
    value: () => {},
  });
  Object.defineProperty(HTMLElement.prototype, "releasePointerCapture", {
    configurable: true,
    value: () => {},
  });
  Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
    configurable: true,
    value: () => {},
  });
  globalThis.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
});

beforeEach(() => {
  const root = document.createElement("div");
  root.id = "overlay-root";
  document.body.append(root);
});

afterEach(() => {
  cleanup();
  document.getElementById("overlay-root")?.remove();
});

function renderUi(node: React.ReactNode) {
  return render(
    <I18nextProvider i18n={i18n}>
      <OverlayProvider>{node}</OverlayProvider>
    </I18nextProvider>,
  );
}

describe("basic UI primitives", () => {
  it("keeps button bounds and native disabled/busy semantics stable", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    const { rerender } = render(
      <Button variant="danger" onClick={onClick}>
        删除
      </Button>,
    );
    const button = screen.getByRole("button", { name: "删除" });
    expect(button.getAttribute("type")).toBe("button");
    expect(button.className).toContain("min-h-control");
    expect(button.className).toContain("bg-danger");
    await user.click(button);
    expect(onClick).toHaveBeenCalledOnce();
    button.focus();
    await user.keyboard("{Enter}");
    expect(onClick).toHaveBeenCalledTimes(2);

    rerender(<Button busy>保存</Button>);
    const busy = screen.getByRole("button", { name: "保存" });
    expect((busy as HTMLButtonElement).disabled).toBe(true);
    expect(busy.getAttribute("aria-busy")).toBe("true");
  });

  it("preserves label, invalid, disabled and read-only input semantics", () => {
    render(
      <>
        <label htmlFor="scheme">方案</label>
        <Input id="scheme" aria-invalid readOnly />
        <Input aria-label="禁用输入" disabled />
      </>,
    );
    const input = screen.getByLabelText("方案");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect((input as HTMLInputElement).readOnly).toBe(true);
    expect(
      (screen.getByLabelText("禁用输入") as HTMLInputElement).disabled,
    ).toBe(true);
  });

  it("renders Kbd as noninteractive semantic text", () => {
    render(<Kbd>Ctrl+K</Kbd>);
    const shortcut = screen.getByText("Ctrl+K");
    expect(shortcut.tagName).toBe("KBD");
    expect(shortcut.getAttribute("role")).toBeNull();
  });
});

describe("overlay UI primitives", () => {
  it("portals a dialog, closes with Escape and restores trigger focus", async () => {
    const user = userEvent.setup();
    renderUi(
      <Dialog>
        <DialogTrigger asChild>
          <Button>打开详情</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogTitle>配置详情</DialogTitle>
          <DialogDescription>检查当前配置。</DialogDescription>
        </DialogContent>
      </Dialog>,
    );
    const trigger = screen.getByRole("button", { name: "打开详情" });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog", { name: "配置详情" });
    expect(document.getElementById("overlay-root")?.contains(dialog)).toBe(
      true,
    );
    expect(screen.getByRole("button", { name: "关闭对话框" })).toBeTruthy();
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: "关闭对话框" }),
    );

    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(document.activeElement).toBe(trigger);
  });

  it("supports menu keyboard selection, Escape and disabled items", async () => {
    const user = userEvent.setup();
    const first = vi.fn();
    const disabled = vi.fn();
    renderUi(
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline">更多操作</Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuLabel>编辑</DropdownMenuLabel>
          <DropdownMenuGroup>
            <DropdownMenuItem onSelect={first}>打开</DropdownMenuItem>
            <DropdownMenuItem disabled onSelect={disabled}>
              删除
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
        </DropdownMenuContent>
      </DropdownMenu>,
    );

    const trigger = screen.getByRole("button", { name: "更多操作" });
    await user.click(trigger);
    const menu = await screen.findByRole("menu");
    expect(document.getElementById("overlay-root")?.contains(menu)).toBe(true);
    expect(
      screen
        .getByRole("menuitem", { name: "删除" })
        .getAttribute("data-disabled"),
    ).not.toBeNull();
    await user.keyboard("{ArrowDown}{Enter}");
    expect(first).toHaveBeenCalledOnce();
    expect(disabled).not.toHaveBeenCalled();

    await user.click(trigger);
    await screen.findByRole("menu");
    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(document.activeElement).toBe(trigger);
  });

  it("opens supplementary tooltip text from hover and keyboard focus", async () => {
    const user = userEvent.setup();
    renderUi(
      <Tooltip delayDuration={0}>
        <TooltipTrigger asChild>
          <Button variant="ghost">帮助</Button>
        </TooltipTrigger>
        <TooltipContent>查看详细说明</TooltipContent>
      </Tooltip>,
    );
    const trigger = screen.getByRole("button", { name: "帮助" });

    await user.hover(trigger);
    const hovered = await screen.findByRole("tooltip");
    expect(document.getElementById("overlay-root")?.contains(hovered)).toBe(
      true,
    );
    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("tooltip")).toBeNull());

    trigger.blur();
    await user.tab();
    expect(document.activeElement).toBe(trigger);
    expect(await screen.findByRole("tooltip")).toBeTruthy();
  });
});
