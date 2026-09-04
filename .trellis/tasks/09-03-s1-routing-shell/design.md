# Design - S1 路由与应用外壳

## 1. Delivery Boundary

本任务把当前单页运行状态迁移为可扩展的七领域桌面壳，但不实现领域业务。唯一新增运行时依赖是 `react-router@8.3.1`；不修改 Rust IPC、配置 schema、capability、CSP 或 generated bindings。

核心边界：

```text
Rust launch parser + runtime snapshot/event + window activation
                              |
                              v
AppRuntimeProvider -> product path validator -> hash router
                              |                    |
                              |                    v
                              +----------> AppShell + route outlet
                                                   |
                  UiPreferencesProvider -----------+--> sidebar/theme/density
                  FeaturesStore -------------------+--> FeatureGate
                  WindowControls Hook -------------+--> one title/app bar
```

Rust 继续拥有启动参数 envelope、单实例与窗口还原；frontend 拥有产品路径、内部 history、route warning、焦点和渲染。Feature availability 与 durable UI preferences 继续分别由 backend catalog 和 TOML config 掌权。

## 2. Proposed Ownership

实现遵循 `.trellis/spec/frontend/directory-structure.md`：

| Location | Responsibility |
|---|---|
| `src/main.tsx` | styles、i18n、provider tree、React root |
| `src/app/app.tsx` | application bootstrap terminal state、window Hook 与 router host composition |
| `src/app/providers/app-runtime-provider.tsx` | runtime listener-first/snapshot merge、refresh、launch sequence 与 visible warnings |
| `src/app/router/catalog.ts` | 七项 typed route catalog、ID/path/label/icon/feature mapping |
| `src/app/router/path.ts` | canonical product-path validation、root alias 与 bounded warning result |
| `src/app/router/router.tsx` | shared route objects、hash/memory router factories、initial hash replacement |
| `src/app/router/navigation-provider.tsx` | history entries、safe back、focus restoration、keyboard routing、navigation warning |
| `src/app/router/runtime-navigation-bridge.tsx` | 把新的 runtime launch request 交给同一 validator/navigation service |
| `src/app/layout/` | AppShell、Sidebar、StatusBar |
| `src/routes/<domain>/` | 七个 route screen；Overview 与 Settings 是本任务真实页面，其余是 gated domain shell |
| `src/components/feature-placeholder/` | FeaturePlaceholder、FeatureGate 和 colocated tests |
| `src/i18n/resources/zh-CN.ts` | 新增 `shell` namespace 的 route、placeholder、warning、settings 文案 |
| `src/icons/ui.ts` | 仅重导出本任务实际使用的 Lucide icons |
| `src/styles/` | shell、route 与 placeholder layout；颜色/尺寸只引用现有 theme tokens，必要的新语义 token 仍归 `theme.css` |

文件可在实现时按真实耦合度合并，但 owner 不能漂移：route-specific behavior 不回到 `main.tsx`，产品 path validation 不进入 Rust transport parser，持久配置不进入 navigation store。

## 3. Provider And Bootstrap Composition

Provider 顺序保持既有合同：

```text
StrictMode
  I18nextProvider
    UiPreferencesProvider
      OverlayProvider
        AppRuntimeProvider
          App
```

`App` 调用现有 `useWindowControls` 一次。Runtime bootstrap 尚未获得首个 terminal outcome 时，渲染中性的 bootstrap frame：复用同一个 WindowTitleBar/窗口 Hook，显示 runtime loading，不挂载 route 页面。这样既保留窗口控制，也不会先绘制 Overview 再跳到启动目标。

Runtime snapshot 成功或失败后，bootstrap 产生一次 `InitialNavigation`：

```typescript
type InitialNavigation = {
  path: CanonicalRoutePath;
  warning: string | null;
  consumedLaunchSequence: number;
};
```

失败时不阻断应用：使用 current hash 或 `/overview`，把 runtime failure 留给 Overview 和 status bar。Router 实例只创建一次；后续 runtime events 经 bridge 导航，不重建 router。

Feature store 初始化与 runtime bootstrap 并行启动。Router 不等待 feature catalog，FeatureGate 自己渲染稳定 loading/failure 状态。

## 4. Route Catalog And Router Factory

### 4.1 Catalog

Catalog 是只读 tuple，并以 `satisfies` 校验字段：

```typescript
type RouteDefinition = {
  id: RouteId;
  path: CanonicalRoutePath;
  labelKey: ShellTranslationKey;
  icon: LucideIcon;
  feature?: AppFeatureId;
};
```

`RouteId` 与 `CanonicalRoutePath` 从 tuple 推导。测试显式断言：长度为 7、ID/path 唯一、顺序固定、settings 最后、五个 feature mapping 精确。Catalog 不重复 milestone；FeatureGate 从 backend record 读取它。

### 4.2 Product Path Validation

一个纯函数接收 launch path 或 URL route string，返回：

```typescript
type ProductPathResult =
  | { kind: "canonical"; path: CanonicalRoutePath }
  | { kind: "redirect"; path: "/overview"; warning: string };
```

规则：

- `/` 返回 `/overview`，无 warning。
- 七个 exact paths 返回自身。
- 其它输入，包括大小写变化、尾斜杠、二级段、query 与 fragment，返回 Overview + bounded zh-CN warning。
- validator 不解码 argv、不重复 Rust 的字符 envelope，也不访问 feature store。

Warning 的 frontend fallback 通过 i18n 生成并限制 512 Unicode 标量；输入路径只在已通过 Rust envelope或浏览器内部 URL 时显示，仍限制长度并作为文本渲染。

### 4.3 Hash And Memory Parity

`createRouteObjects()` 只生成一份 route tree：

```text
AppShell
  /                 -> replace /overview
  /overview         -> OverviewRoute
  /lexicons         -> gated LexiconsRoute
  /phrases          -> gated PhrasesRoute
  /lookup           -> gated LookupRoute
  /radicals         -> gated RadicalsRoute
  /learning         -> gated LearningRoute
  /settings         -> SettingsRoute
  *                 -> UnknownRoute -> replace /overview + warning
```

Production factory 在创建 hash router 前，以 `history.replaceState` 只替换 URL hash 为已校验 initial path，保留 origin/path/query，不增加 history entry；随后创建 hash router。Test factory 把同一 objects 和 initial path 传给 memory router。该 adapter 本身有单测，避免 RouterProvider 首帧错误 route。

`UnknownRoute` 通过 navigation context 写 warning，再 replace 到 Overview。它不渲染错误堆栈、空白页或外部 URL。

## 5. Runtime Navigation State Machine

`AppRuntimeProvider` 延续当前 listener-first race handling：

1. 注册 `app://launch-requested`。
2. 记录每个 event 的本地单调 `sequence`，立刻合并到已 ready snapshot。
3. 请求 `app_runtime_snapshot`。
4. 若请求期间收到 event，用 `mergeLatestLaunch` 保留较新的本地 event。
5. refresh 失败时保留最后有效 snapshot，显示 refresh warning。

Initial path precedence：bootstrap 期间最后一个带 path 的 secondary event > snapshot latest secondary 中带 path的 request > primary path > current hash > Overview。无 path secondary 不覆盖已选路径。

Router bridge 只消费 `sequence > consumedLaunchSequence` 的新事件：

| Event | Result |
|---|---|
| no navigationPath | runtime diagnostics update only |
| canonical, different from current | push target; clear navigation warning |
| canonical, same as current | replace/idempotent; do not grow history |
| unknown product path | replace Overview; set bounded warning |

Native `handle_secondary_launch` 已独立调用 `WindowCoordinator.restore()`；bridge 不调用 `window_control`，即使 route validation 失败也不改变 native 激活结果。

## 6. Safe Internal History And Focus

### 6.1 Session History

NavigationProvider 按 `location.key` 维护有界于当前 WebView session 的 entries 与 index：

- Initial：`entries=[initialKey]`, `index=0`。
- PUSH：截断 forward entries，追加新 key，`index += 1`。
- REPLACE：替换当前 key，不改变 index。
- POP：只在 known entry key 中移动 index；未知 key 重建为单项边界，防止误判可返回。

`canGoBack = index > 0`。不得使用 `window.history.length`、浏览器全局 back 或私有 React Router history fields，因为它们可能包含应用外 entry。

### 6.2 Focus Records

Sidebar link 等 app-owned 导航源在激活前调用 `rememberFocus(currentLocationKey, event.currentTarget)`。记录的是 HTMLElement reference；POP 后只有在 `document.contains(node)` 且节点仍可聚焦时才恢复，否则聚焦目标 route 的 `[data-route-heading]` `h1`。

Focus policy：

- PUSH/REPLACE：route commit 后聚焦新 `h1`。
- POP：优先恢复 destination entry 的 remembered trigger；fallback 到新 `h1`。
- 同路径 idempotent replace 不抢走当前合理焦点。
- 聚焦用 `preventScroll`，随后只在目标不在可见 content viewport 时调用最近的滚动容器定位。

### 6.3 Keyboard Back

Document bubble-phase keydown handler：

- `Alt+Left`：若 `canGoBack`，preventDefault 并调用 router `navigate(-1)`；否则 prevent WebView 外跳并 no-op。
- `Esc`：仅无修饰键、`canGoBack`、未 `defaultPrevented`、target 非 editable、overlay root 无 active portal 时返回。
- Editable 包含 input、textarea、select、contenteditable 及其 descendants。
- Overlay guard 复用 `useOverlayRoot()`；Dialog/Menu/Tooltip 的 Radix owner 先消费 Escape。路由层不写第二套 focus trap。

## 7. Application Shell

### 7.1 Layout

```text
row 1: WindowTitleBar (also the one app bar)
row 2: Sidebar | scrollable route main
row 3: persistent StatusBar
overlay: existing #overlay-root
```

`WindowTitleBar` 增加当前 route title 的轻量展示，同时保留 app icon/name/version、drag region 和原生控制。没有系统码表权威数据与全局搜索 action，因此方案徽标和搜索框不渲染，也不保留空壳控件。

Frame 使用 `min-height: 100vh`、`grid-template-rows: auto minmax(0,1fr) auto`；body 使用 expanded/collapsed 两个稳定 sidebar track 和 `minmax(0,1fr)` content。Main 是唯一 route scroll container，header/footer 不覆盖它，因此 focus 不会被固定层完全遮挡。宽度切换不做 width animation；允许的微交互只改变 color/opacity，并受 reduced-motion token 控制。

### 7.2 Sidebar

- 语义为 `<nav aria-label>` + `<NavLink>`；DOM/视觉顺序均来自 catalog。
- settings 通过 layout spacer 固定到底部，但 DOM 仍在前六项之后。
- 展开显示 icon + label；折叠保留 44x44 link、accessible name、Tooltip 和非颜色 selected indicator。
- 当前项由 `aria-current="page"`、文字字重/边界/indicator 表达，不能只靠颜色。
- Collapse control 使用 Lucide panel icon、44x44 button、`aria-expanded` 和动态 accessible label。
- Sidebar control 和 Settings 外观 checkbox 都调用 `setSidebarCollapsed`；provider 负责 optimistic projection、串行 full-group save、event merge 与 rollback。

### 7.3 Status Bar

StatusBar 只投影已存在的事实：ready/loading 状态，最高优先级 visible warning，以及最近 runtime/config/window/navigation notice。无 warning 时显示“应用已就绪”；不显示假 task percent、IME enabled state 或 system scheme。

Warning 使用 icon + 文本 + tone，`aria-live="polite"`，正文限制 512 Unicode 标量。完整 runtime details 仍在 Overview；status bar 不演进成错误中心或 toast。

## 8. Overview And Settings

### 8.1 Overview

Overview 迁移现有 `LoadingState`、`LoadError`、`RuntimeStatus`、`StatusCell` 与 `LaunchDetails`，继续使用 `runtime-view.ts`：

- privilege、previous abnormal session、latest launch 均来自 RuntimeSnapshot。
- launch/runtime/window/config/navigation warnings 保持可见且不翻译 backend message/detail。
- refresh/retry 仍调用 runtime provider；失败保留 prior valid snapshot。
- 布局使用平铺 status items 与无框 detail sections，不嵌套 cards。
- 不展示系统码表、短语、备份、输入法健康或快捷操作的假数据。

### 8.2 Settings

Settings 使用八组固定 IA：输入法、五笔行为、候选窗口、快捷键、外观、网络、数据、关于。

- 外观组是真实表单：theme 三选一与 density 二选一使用 `fieldset`/`legend`/native radio styled as segmented controls；sidebarCollapsed 使用 labeled native checkbox/toggle。
- 每个 control 直接调用 UiPreferencesProvider setter；没有 Save button。Provider loading/failed/warning 用既有 visible path 表达。
- 单一 `zh-CN` locale 不渲染选择器。
- 其它七组只包含 section-level FeaturePlaceholder 或静态 owner 说明；不创建开关、输入、默认值、command 或系统副作用。
- 本任务不增加二级 settings routes；同页 anchor/组内导航等到真实内容需要时再建立。

## 9. Feature Placeholder And Gate

`FeaturePlaceholder` 是纯展示组件：

```typescript
type FeaturePlaceholderProps = {
  variant: "page" | "section" | "inline";
  title: string;
  description: string;
  milestone?: TargetMilestone;
};
```

所有 variant 使用统一“功能暂未完善”、具体能力说明和非颜色“开发中”状态。Milestone 来自 AppFeature record；shell-owned deferred sections 可显示 owning child task 文案，但不得伪造 AppFeatureId。

`FeatureGate` 接收 generated `AppFeatureId`、placeholder copy、variant 和 children：

| Store state | Rendering |
|---|---|
| loading | same-size busy/skeleton state |
| failed | bounded alert + retry button |
| ready, record missing | fail-closed catalog error + retry |
| ready, unavailable | FeaturePlaceholder with record.targetMilestone |
| ready, available | children |

Gate 不 catch missing command，不维护 feature list，不触发业务 IPC。五个 future domain routes 使用 page gate；Settings/Overview future blocks 使用 section presentation；inline variant 先由 component contract/test 固定，实际 action consumer 等后续任务。

## 10. Errors And Visible Warning Priority

各 owner 保留自己的 state，不拼成新的全局 store。AppShell 只做 presentation priority：

1. navigation fallback warning；
2. UI preference persistence/listener warning；
3. runtime listener/refresh warning；
4. window command/listener warning；
5. latest backend runtime notice；
6. no warning -> ready。

Overview 可展示完整集合；StatusBar 显示最高优先级一条。成功 canonical navigation 清除 navigation warning；其它 owner 的 warning 只能由其自身成功路径或 clear API 清除。

## 11. Styling And Accessibility

- 复用 `theme.css` 的 semantic colors、spacing、radius、control、focus、motion 和 density tokens；若需 sidebar/status dimensions，只新增语义 `--wl-*` token 并在 `@theme inline` 暴露，组件中无 raw palette。
- Cards radius 不超过现有 8px `radius-md`；page sections 不做 floating cards，阴影只用于 overlays。
- Sidebar、route headings、radio、checkbox、retry、window controls 全键盘可达；DOM order 与视觉 order 一致。
- Icon-only controls 使用 Lucide narrow export、visible Tooltip 与独立 accessible name；不使用 emoji、手写 SVG 或在线 asset。
- long Chinese/200% text 使用 wrap、`minmax(0,1fr)`、`overflow-wrap:anywhere`；不能靠 viewport-width font scaling。
- `1024x640` 和 `1440x900` 下 route main 自己滚动，frame 无水平 overflow；light/dark/system、standard/compact 和 reduced-motion 均保持状态可辨。

## 12. Test Architecture

### 12.1 Pure And Component Tests

- catalog/path：七项、unique/order/mapping、root alias、exact match、unknown/query/fragment、warning bound。
- router factory：同 route objects 的 hash/memory parity、initial replace、wildcard fallback、no first-route flash contract。
- runtime provider：listener-first、event-during-snapshot、latest path precedence、refresh failure、cleanup、local sequence/idempotence。
- navigation：PUSH/REPLACE/POP stack、forward truncation、trigger restore、h1 fallback、same-path、Esc、Alt+Left、top-level、editable、overlay。
- shell：expanded/collapsed aria/tooltip/target/current item、one app bar、status priority。
- provider：sidebar patch participates in same serialized queue, sibling preservation, event merge and failure rollback。
- feature：loading/failed/retry/missing/unavailable/available and three variants。
- routes：Overview regression；Settings eight groups/radio/checkbox/immediate save/no locale selector/no fake controls。

Tests inject runtime/config clients and use Testing Library/user-event. Memory router is used for deterministic component tests; production hash adapter has targeted JSDOM tests。

### 12.2 Browser And Windows

- Vite + Playwright-core visual/DOM checks cover all seven paths, unknown hash, two themes, two densities, expanded/collapsed sidebar, `1024x640`, `1440x900`, 200% root font, long Chinese and reduced motion。
- Assert nonblank rendered pixels, active route/title, no horizontal overflow, no text/control overlap, visible focus, overlay Escape priority and hash update。
- Windows smoke changes path examples to canonical paths and adds an unknown product path handoff while preserving hidden/restore/tray/exit cleanup assertions。真实窗口手工检查 route 与 warning，因为 PowerShell process harness 不读取 WebView DOM。
- No generated binding change is expected；freshness check 和 `git diff -- src/types/generated/bindings.ts` 必须为空。

## 13. Compatibility, Rollout And Rollback

- Hash URLs are frontend-only and do not alter the external `--navigate /path` contract。
- Existing config already stores sidebarCollapsed, so no schema migration or rollback migration is required。
- Route IDs become compatibility-sensitive only after the next action registry consumes them；本任务先冻结 catalog 并用 tests 防漂移。
- Dependency/catalog/runtime extraction/navigation manager/shell/placeholders/routes-smoke 分独立 review boundary。每一步先过 focused tests 再继续。
- Router 失败时回退 dependency + router modules，保留现有 runtime page；不得以 BrowserRouter/server fallback 或第二 history package 绕过。
- Runtime extraction失败时恢复 current listener/snapshot logic，不修改 Rust event/command。
- Focus/Escape 失败时回退 navigation manager，不能手写 overlay focus trap、禁用 focus ring 或允许 WebView 外跳。
- Layout 失败时调整现有 token/grid，不加入营销 hero、第二色板、在线字体、GSAP、卡片嵌套或 viewport font scaling。
- 任一实现触及根 `resource/`、ImTip、S2/S3 command 或系统写入时，删除该实现并回到 placeholder boundary，不扩大任务。
