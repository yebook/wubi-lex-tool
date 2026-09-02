# S1 UI 基础技术契约

## 研究范围

本文只收敛 `s1-ui-foundation`：Tailwind v4 令牌、首帧外观、配置同步、简体中文资源、最小 UI primitives、当前 runtime/titlebar 迁移和质量门禁。路由/侧栏/设置页面、动作/快捷键、任务反馈、领域组件、在线/PUA 资源、ImTip 与根目录 `resource/` 均不在范围内。

## 仓库基线

- `vite.config.ts` 已接入 `@tailwindcss/vite@4.3.3`，但产品 CSS 尚未 `@import "tailwindcss"`；当前 `runtime-status.css` 用临时变量和大量字面样式承载全部页面与标题栏。
- `src/main.tsx` 直接挂载运行状态页，初始化 feature store、runtime snapshot 和 window Hook；没有 provider tree、i18n 或 overlay root。
- `src-tauri/src/config/model.rs` 已定义 `ThemePreference = system|light|dark`、`Density = standard|compact`、`AppLocale = zh-CN`，配置服务已有事务保存、revision 和 `config://changed`。
- `src-tauri/src/lib.rs` 在一次 `config_service.snapshot()` 后只取 window config，再通过 `WebviewWindowBuilder` 创建不可见窗口。该位置可以同时取 ui config 并接入 document-start script，无需新 command/schema。
- 当前 Tauri CSP 禁止任意 inline script/style；Webview builder initialization script 是 native 注入，不要求放宽 CSP。
- titlebar 和 `useWindowControls` 已固定 44x44、Lucide、listener-first、revision、防旧 snapshot、异步 unlisten 和 warning 可见性；UI foundation 必须延续，不另建冲突模式。

## 设计事实与搜索结论

### 仓库权威设计

- `docs/21-ui-ux.md`：墨蓝 `#1E3A5F/#7DA7D9`、朱砂 danger、冷灰 neutral、surface 1..3、border 1..2、text 1..3、五区色、4/8 间距、4/8/12 圆角、120/200ms 动效。
- `docs/20-nonfunctional.md`：键盘完整、focus 可见、正文 4.5:1、大字号 3:1、不只靠颜色、reduced motion、系统字号、简体中文和文案外置。
- `.trellis/spec/frontend/tailwind-v4-tokens.md`：只允许 `src/styles/theme.css`、`@theme inline`、CSS-first、dark/compact custom variants；禁止 Tailwind v3 配置面。

### `ui-ux-pro-max`

- 检测到 React 19 + Tailwind v4 + Windows desktop tool；有效结果是 Minimal/Swiss、低 variance、低 motion、高 density、清晰边界/focus 和两主题独立验证。
- 搜索结果的 landing hero/CTA、在线 Google Fonts、青绿品牌色和 GSAP scroll reveal 与产品/离线桌面环境不匹配，按技能的 fit verification 规则弃用。
- shadcn stack 搜索确认：TooltipProvider 应放在 app 层而不是每实例；tooltip 不能只靠 title；menu 按真实分组使用 separator。
- React stack 搜索确认：theme/locale 适合 context/provider，context value 必须 memo；高频领域数据不得放入该 context。

## 依赖研究

2026-09-02 通过项目 pnpm 查询 npm metadata：

| Package | Selected | Reason |
|---|---:|---|
| `i18next` | `26.4.0` | 架构文档已实测并固定；支持 TypeScript 6 |
| `react-i18next` | `17.0.13` | React >=16.8、i18next >=26.2，兼容当前栈 |
| `@radix-ui/react-dialog` | `1.1.23` | 只为真实 Dialog primitive |
| `@radix-ui/react-dropdown-menu` | `2.1.24` | 只为真实 menu primitive |
| `@radix-ui/react-tooltip` | `1.2.16` | 只为真实 tooltip primitive |
| `@radix-ui/react-slot` | `1.3.3` | 仅供 Button `asChild` 组合；不引入通用 polymorphic framework |
| `class-variance-authority` | `0.7.1` | Button 等有限 variant |
| `clsx` | `2.1.1` | 条件 class |
| `tailwind-merge` | `3.6.0` | 合并 Tailwind overrides |
| `prettier` | `3.9.6` | 前端稳定格式门禁 |
| `prettier-plugin-tailwindcss` | `0.8.1` | v4 `tailwindStylesheet` 类名排序 |

- 所有 Radix packages peer range包含 React 19；均为 MIT。CVA 为 Apache-2.0，其余 helper 为 MIT。
- 不选择 `radix-ui@1.6.7`，因为它覆盖远多于本任务使用的 primitive。
- 不选择 `react-router@8.3.1`；它虽兼容当前 React 19.2.8，但没有本任务消费者，路由子任务再精确引入并验收。
- 不增加 `shadcn` CLI 为 dev/runtime dependency；项目拥有复制并评审后的源组件。

## First-Frame Bootstrap

Tauri 2.11.5 的 `WebviewWindowBuilder::initialization_script` 在 global object 创建后、HTML 解析和页面脚本前执行；Windows 会在 subframe 也执行，因此脚本必须限制 top-level context。

### Native input

Rust setup 只读取一次 config snapshot：

```text
ConfigSnapshot
  -> window config -> placement
  -> ui config -> UiBootstrap(theme, density, locale)
```

配置读取失败时两者都使用各自默认值，并沿用可见 runtime notice。不能 window config 默认、ui config 再做第二次 snapshot，否则启动期间 event/revision 可能分裂。

### Document projection

固定枚举映射：

| Input | Root projection |
|---|---|
| light | `data-theme=light`, remove `.dark`, `color-scheme: light` |
| dark | `data-theme=dark`, add `.dark`, `color-scheme: dark` |
| system | `data-theme=system`, `.dark = matchMedia(...)`, `color-scheme: light dark` |
| standard/compact | `data-density=<value>` |
| zh-CN | `lang=zh-CN` |

- document-start 时 `document.documentElement` 可能尚不存在；脚本用一次性 observer/ready helper 在 `<html>` 创建的首个 microtask 应用，然后断开。
- 不把配置 JSON、路径、用户文案或任意字符串插入脚本；Rust helper 只选择固定片段并测试禁止控制字符/动态输入。
- explicit theme 同时调用 WebviewWindowBuilder native theme，减少创建背景与 CSS 首帧不一致。
- 前端 provider 接管后复用同一 DOM projection helper，并为 system preference 注册 `matchMedia` change listener。

## UI Preferences State Flow

```text
document bootstrap attributes
        -> provider initial render
listen config://changed
        -> config_snapshot
        -> merge by revision
        -> apply root class/data/lang

setTheme/setDensity/setLocale
        -> optimistic DOM projection
        -> serialized config_update_ui(latest full group)
        -> confirmed ConfigSnapshot
        -> config://changed may arrive before response
        -> merge by revision
failure -> restore last confirmed UiConfig + visible bounded warning
```

- `config_update_ui` 替换完整 group，因此 update API 接收 patch 但在单一队列中基于最新 pending/confirmed value 生成 full `UiConfig`；禁止多个组件各自 read-modify-write。
- equal revision 可接受 command/event 的权威 snapshot，lower revision 永不覆盖。
- provider 不存储到 Web Storage，不创建第二份 schema，不把 backend config 复制为 handwritten wire type。
- 当前 runtime page 把 provider warning 合并到现有可见 warning 区；routing shell 后续接到状态栏。

## Token Contract

`src/styles/theme.css` 结构：

```css
@import "tailwindcss";
@custom-variant dark (...);
@custom-variant compact (...);

@theme inline {
  --color-primary: var(--wl-primary);
  /* semantic colors, surfaces, text, zones */
}

:root { /* light values + non-color tokens */ }
.dark { /* dark values */ }
```

- theme variables use `--wl-*`; Tailwind public tokens use `--color-*`, `--font-*`, `--radius-*`, `--shadow-*` and named motion/spacing values.
- `letter-spacing: 0`; body uses system offline UI stack. Mono and etymon stacks are declared even when optional fonts are not installed.
- surface 1 is page background, surface 2 is repeated item/panel background, surface 3 is hover/header. Sections remain unframed; cards are only repeated items or true framed tools.
- focus token must remain >=3:1 against adjacent surfaces. Text-1/text-2 pairs are tested >=4.5:1; disabled text is not used as essential standalone content.
- density changes spacing/line-height variables while preserving required accessible control hit areas; later virtual tables may define denser noninteractive rows.
- reduced-motion overrides nonessential transitions/animations without removing focus or loading-state meaning.

## i18n Contract

- `src/i18n/resources/zh-CN.ts` is the first real resource; namespaces separate common/window/runtime/ui.
- i18next initializes from bundled resources with no backend/network loader. React interpolation keeps `escapeValue=false` because React escapes output.
- components use `useTranslation`; pure projection helpers receive a narrow translation function and stay deterministic.
- brand `WubiLex`, version values, codes/paths and backend `AppError.message/detail` remain data, not translation keys.
- static search permits Chinese only inside locale resources, tests/fixtures, generated/backend payload assertions, and product/domain content explicitly excluded by NFR-I18N-004.

## Primitive Contract

### Button / Input / Kbd

- Button defaults to native `type=button`, supports primary/secondary/outline/ghost/danger and default/icon variants, forwards React 19 ref semantics, and exposes busy through disabled + `aria-busy` without changing bounds.
- `asChild` uses Radix Slot only where semantic child ownership is explicit; no generic polymorphic type framework.
- Input remains a native input; label/help/error ownership stays with form composition. `aria-invalid`, disabled and read-only states are visually distinct.
- Kbd is presentation text, not a button. Shortcut recording behavior remains out of scope.

### Dropdown / Dialog / Tooltip

- Wrappers re-export only the reviewed surface needed by later tasks, not the entire Radix namespace.
- One app-level TooltipProvider supplies delay/skip behavior; every icon-only trigger still has an accessible name.
- Dialog has title/description/close affordance, Escape, focus trap and trigger focus restore; destructive confirmation semantics are not invented here.
- Dropdown menu groups related items and supports separator; disabled item uses real disabled state.
- Portal container is the stable `#overlay-root`; z-index and scrim are tokens. No overlay lives inside a card or clipped scroll container.

## Formatting And Quality

- Prettier config points Tailwind plugin to `src/styles/theme.css`; generated bindings are excluded to preserve `cargo xtask bindings` byte ownership.
- `format:check` joins typecheck/lint/test/build gates and Windows CI. Formatting should touch only frontend-owned files and config, not Rust/docs wholesale.
- Contrast/token tests parse the controlled CSS structure, verify required variables per theme and calculate sRGB contrast for required pairs.
- Browser checks use real rendered pixels for both themes/densities and system text scaling; jsdom is not evidence for color/layout.

## Explicit Non-Selections

- No React Router, page routes, sidebar, settings UI, command palette, toast/task system or placeholder components.
- No Tailwind v3 config, PostCSS, CSS-in-JS theme, localStorage, packageManager field, Corepack/npm/yarn/npx.
- No Google Fonts, GSAP, theme plugin, unified Radix bundle, unreviewed shadcn generated tree or decorative animation.
- No root resource reads, PUA asset import, ImTip, S2/S3 command, network or system write.
