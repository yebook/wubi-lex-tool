# Implementation Plan - S1 路由与应用外壳

## Entry Gate

- [ ] 用户审阅本任务最终规划摘要，并在后续消息中明确批准实施。
- [ ] 获批后运行 `python ./.trellis/scripts/task.py start .trellis/tasks/09-03-s1-routing-shell`；未 start 前不改产品代码、不安装依赖。
- [ ] 实施前加载 `trellis-before-dev`，完整读取本任务 manifests 注入的 frontend/backend/guides/research。
- [ ] 核对工作树，保留用户 `.vscode/`；不读取或修改根目录 `resource/`。
- [ ] 设置/验证 `VOLTA_FEATURE_PNPM=1`，`node --version` 与 `pnpm --version` 分别匹配 `package.json.volta.node/pnpm`；不增加 `packageManager` 或其它 pnpm version source。

## 1. Add The Single Router Dependency

- [ ] 以项目 Volta pnpm 精确增加 `react-router@8.3.1`，更新 `package.json` 和 `pnpm-lock.yaml`。
- [ ] 不增加 `react-router-dom`、history、query/cache、framework mode、SSR、Google Fonts、GSAP 或其它 runtime dependency。
- [ ] 检查 lockfile closure、license、peer/engine compatibility 和 `packageManager` absence。

```powershell
$env:VOLTA_FEATURE_PNPM = '1'
pnpm add --save-exact react-router@8.3.1
pnpm install --frozen-lockfile
pnpm audit --audit-level high --registry https://registry.npmjs.org/
```

Rollback point：只回退 manifest/lock 的 router change；不切换 registry 配置、不添加第二 package manager。

## 2. Freeze Route Catalog And Product Path Validator

- [ ] 在 `src/app/router/` 建立七项 readonly catalog，包含 ID、canonical path、shell i18n key、Lucide icon、顺序和可选 generated feature ID。
- [ ] 从 catalog 推导 RouteId/CanonicalRoutePath；不手写平行 union，不复制 backend milestone。
- [ ] 实现纯 product-path validator：`/` replace alias、七 exact paths、unknown/query/fragment/case/trailing slash fail closed 到 Overview + bounded warning。
- [ ] 建立 shared route objects；production hash factory 与 test memory factory 复用它。
- [ ] Production factory 在 RouterProvider 首帧前 replace validated hash，不依赖 server fallback。
- [ ] 添加 catalog/path/factory 单测。

Focused gate：

```powershell
pnpm run test -- src/app/router
pnpm run typecheck
```

Rollback point：route catalog + factory 为一个边界；不能改用 BrowserRouter 或在 sidebar/pages 各维护路径。

## 3. Extract Runtime Bootstrap And Seed Initial Navigation

- [ ] 把 `main.tsx` 的 runtime state、listener-first/snapshot merge、refresh 和 warning 提取到可注入 client 的 AppRuntimeProvider/Hook。
- [ ] 保留 `mergeLatestLaunch`、runtime error bounding、feature store bootstrap 和现有诊断数据；不重新定义 generated payload。
- [ ] 为 launch event 增加 frontend local sequence，覆盖 snapshot in-flight race 和同 path 幂等消费。
- [ ] 按“最后带 path 的 startup secondary > snapshot latest secondary > primary > current hash > overview”产生一次 initial navigation。
- [ ] Runtime terminal outcome 前渲染 neutral bootstrap frame；复用一次 window Hook/WindowTitleBar，不绘制错误 route。
- [ ] 添加 listener order、race、failure、refresh、cleanup、path precedence 测试。

Focused gate：

```powershell
pnpm run test -- src/app/providers src/runtime-view.test.ts
pnpm run lint
```

Rollback point：若提取破坏 runtime diagnostics，恢复现有 `main.tsx` owner；不得新增 Rust revision/event/command 绕过。

## 4. Implement Runtime Navigation, History And Focus

- [ ] 建立 NavigationProvider，按 `location.key` 追踪当前 session 的 PUSH/REPLACE/POP entries/index 和 `canGoBack`。
- [ ] 提供 app-owned navigation helper 与 focus record；Sidebar link 激活前记录 trigger。
- [ ] PUSH/REPLACE 后聚焦 route `h1`；POP 恢复仍在 DOM 的 trigger，否则聚焦目标 `h1`。
- [ ] 建立 runtime bridge：新 canonical event push，same path replace，无 path no-op，unknown replace Overview + warning；不调用 window command。
- [ ] 实现 `Alt+Left` 内部返回与顶层 prevent-external no-op。
- [ ] 实现 `Esc` guard：top-level、defaultPrevented、editable target、active overlay 均不回退。
- [ ] 添加真实 user-event 测试，包含 Dialog/Menu Escape priority 与 forward-stack truncation。

Focused gate：

```powershell
pnpm run test -- src/app/router src/components/ui
pnpm run typecheck
```

Rollback point：history/focus manager 可独立移除；不能用 `window.history.length`、全局 `history.back()` 或 private router fields。

## 5. Build The Single App Bar And Shell Layout

- [ ] 扩展现有 WindowTitleBar 显示当前 route title，使其同时承担唯一 app bar；保留 icon/name/version、drag region、44x44 window controls 和 existing tests。
- [ ] 建立 AppShell grid：one title/app bar、Sidebar、scrollable Outlet、persistent StatusBar、existing overlay root。
- [ ] Sidebar 完整投影 catalog；settings 位于底部，expanded/collapsed 均保持 semantic link、aria-current、non-color selection、44x44 target。
- [ ] 折叠按钮使用 Lucide、Tooltip、accessible label/expanded state；不以 emoji 或文字胶囊代替熟悉 icon。
- [ ] StatusBar 只展示真实 ready/loading/warning；按 navigation/config/runtime/window/backend notice 优先级显示一条 bounded 信息。
- [ ] 布局使用 existing tokens 与 `minmax(0,1fr)`；不动画 width/height、不嵌套 cards、不加入方案徽标/搜索/任务/IME 假值。
- [ ] 添加 shell/sidebar/status/titlebar component tests。

Focused gate：

```powershell
pnpm run test -- src/app/layout src/components/window-title-bar
pnpm run build
```

Rollback point：shell styles/tokens 与 titlebar props 一起回退；不建立第二 fixed header 或第二视觉系统。

## 6. Extend Sidebar Preference Through The Existing Queue

- [ ] 把 `sidebarCollapsed` 加入 UiPreferencesProvider patch type/context，并新增 `setSidebarCollapsed`。
- [ ] Sidebar button 与 Settings checkbox 复用该 setter；initial/bootstrap/config event 继续使用 normalized value。
- [ ] 测试 optimistic projection、theme/density/sidebar sibling queue、event merge、failed job rollback、warning bound 和 StrictMode cleanup。
- [ ] 静态确认无 localStorage/sessionStorage/cookie/Zustand persist/第二 config client。

Focused gate：

```powershell
pnpm run test -- src/app/providers/ui-preferences-provider.test.tsx src/lib/ui-appearance.test.ts
```

Rollback point：只回退新增 patch/setter/consumers；stored config 字段已存在，无 schema migration 要撤销。

## 7. Implement Feature Placeholder And Gate

- [ ] 在 `src/components/feature-placeholder/` 建立 page/section/inline variants，共享“功能暂未完善”、能力说明、真实 milestone 和文本状态。
- [ ] FeatureGate 使用 Zustand selector 读取 generated AppFeatureId；不创建 handwritten feature union。
- [ ] 覆盖 loading stable busy、failed retry、missing record fail-closed、unavailable placeholder、available children。
- [ ] 五个 future domain routes 使用 page gate；Settings future groups 使用 section presentation；inline contract 用 test component 固定。
- [ ] 确保 placeholder 不调用 command、不读取根资源、不产生系统写入或 fake data。

Focused gate：

```powershell
pnpm run test -- src/components/feature-placeholder src/stores/features.test.ts
pnpm run lint
```

Rollback point：若 gate 与 store contract 冲突，回退 component 并复用 store selector；不以 command failure 或 Vite flag 替代。

## 8. Migrate Overview And Build Minimal Settings

- [ ] 把现有 runtime loading/error/retry/status/launch/notices 移到 `src/routes/overview/`，继续调用 `runtime-view.ts` helper。
- [ ] Overview 保留真实 privilege、recovery、launch、runtime/window/config/navigation warnings；无系统码表、短语、备份、IME、健康或快捷动作假数据。
- [ ] Settings 建立输入法、五笔行为、候选窗口、快捷键、外观、网络、数据、关于八组顺序。
- [ ] 外观组用 native fieldset/radio styled segmented controls 操作 theme/density，用 labeled checkbox/toggle 操作 sidebarCollapsed；即时保存，无 Save button。
- [ ] 其它七组只显示 honest section placeholder，不创建 fake form/control/default/command；不渲染单一 zh-CN selector。
- [ ] 七 route screens 都有唯一 `h1[data-route-heading]` 和稳定 title。
- [ ] 添加 Overview regression、Settings semantics/queue 和 five route gate tests。

Focused gate：

```powershell
pnpm run test -- src/routes src/runtime-view.test.ts
pnpm run typecheck
```

Rollback point：Overview 与 Settings 可分别回退；不得用扩大领域实现来填补空白。

## 9. Complete i18n, Icons And Styling

- [ ] 在 bundled zh-CN 增加单一 `shell` namespace，覆盖 route labels、sidebar、status、navigation warning、placeholder 和 Settings；frontend visible copy 不硬编码。
- [ ] 只在 `src/icons/ui.ts` 重导出已使用 Lucide icons，保持一致 stroke/size；无 emoji、手写 SVG、第二 icon package。
- [ ] 新增/调整 shell CSS 与必要 semantic tokens；复用现有 palette/font/radius/focus/motion/density。
- [ ] 200% text 与长中文可 wrap，buttons/tiles dimensions stable；main/footer/titlebar/sidebar 无 horizontal overflow 或 focus occlusion。
- [ ] 更新 i18n/CSS contract tests 和 formatter coverage。

Focused gate：

```powershell
pnpm run format
pnpm run test -- src/i18n src/styles
pnpm run build
```

Rollback point：新增 shell token 必须随 consumers 回退；不 hardcode palette、负 letter spacing 或 viewport-scaled font。

## 10. Browser And Windows Verification

- [ ] 运行 Vite，使用已锁定 Playwright-core 检查七 route、root alias、unknown hash warning、hash updates、Sidebar states 和 Settings controls。
- [ ] 对 light/dark x standard/compact 在 `1024x640` 与 `1440x900` 截图并做 canvas/pixel + DOM overflow 检查。
- [ ] 额外覆盖 200% root font、long Chinese warning、reduced-motion、keyboard-only focus、overlay Escape、page/section/inline placeholder、feature loading/failure。
- [ ] 截图与临时 browser profile 只放 ignored `target/`，不提交。
- [ ] 更新 Windows smoke 的已知 route 为 `/settings`，加入 hidden combined primary、secondary canonical navigation 和 unknown product path handoff，保留 tray/close/exit cleanup。
- [ ] 从管理员终端运行 real smoke，并在可见窗口确认 canonical route 与 unknown warning；不读取真实用户配置。

```powershell
pnpm run dev
pnpm run smoke:runtime
```

Rollback point：smoke 路径变化独立于 frontend route code；失败时保留日志/截图证据，不能删除 native lifecycle assertions。

## 11. Full Quality Gate And Static Review

- [x] Frontend frozen install、format check、typecheck、ESLint、Vitest、build 全绿；official audit 经用户于 2026-09-04 明确决定跳过。
- [ ] Rust fmt/check/clippy/test/doc 回归全绿；记录本机 admin-manifest binary harness、coverage/actionlint 等真实环境限制，不伪报通过。
- [ ] Bindings freshness 与 docs validation 通过，generated bindings 无 diff。
- [ ] 静态搜索无 ImTip、根 resource 读取、Web Storage、Vite feature flag、第二 router/package manager、在线 font/GSAP、fake domain command。
- [ ] `ui-ux-pro-max` 结束检查覆盖 focus、keyboard、deep-link、contrast、density、text scaling、reduced motion、stable dimensions 和 overlap。
- [ ] Task validation、`git diff --check`、`git status --short` 通过并保护 `.vscode/`。

```powershell
$env:VOLTA_FEATURE_PNPM = '1'
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }

pnpm install --frozen-lockfile
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run format:check
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run build

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --all-features --no-deps --locked
cargo xtask bindings --check
cargo xtask check-docs

python ./.trellis/scripts/task.py validate .trellis/tasks/09-03-s1-routing-shell
git diff --check
git status --short
```

## 12. Phase 3 Closure

- [x] 使用 `trellis-check` 做全任务独立检查并修复 verified findings。
- [x] 使用 `trellis-update-spec` 记录真实形成的 route catalog、runtime bootstrap、history/focus、feature placeholder、shell/settings 模式，不以规划代替实现证据。
- [x] 更新 PRD acceptance 与本文件 verification record。
- [ ] 按 planning / product implementation / spec coherent units 提交；只有用户明确要求时 push。
- [ ] 使用 `trellis-finish-work` 归档子任务、更新父任务与 journal，然后进入 `s1-actions-keymap`。

## Verification Record - 2026-09-03

- Toolchain and dependency: Node `24.18.1`, project Volta pnpm `11.18.0`, exact
  `react-router@8.3.1`, frozen install passed, and lock SHA-256 remained
  `91DC2EFFA2DD010CE284B5395329798915E366E8EEC2D26C9CC107EA4006B392`.
- Frontend: format check, typecheck, ESLint, 22 Vitest files / 101 tests, and
  production Vite build passed.
- Browser: seven routes, root alias, unknown fallback, hash/focus behavior,
  expanded/collapsed Sidebar, light/dark/system, standard/compact,
  `1024x640`, `1440x900`, reduced motion, 200% text, and a long encoded Unicode
  warning passed 13 screenshot, nonblank, overlap, and horizontal-overflow
  checks. Evidence is retained only under ignored `target/`.
- Rust and repository: fmt, workspace check, strict Clippy, all-feature tests,
  warnings-denied Rustdoc, binding freshness, docs validation, task validation,
  PowerShell 5.1 parsing, static forbidden-pattern search, and
  `git diff --check` passed. Generated bindings remained unchanged.
- Windows runtime smoke: all four isolated elevated stages passed, including
  hidden `/settings`, canonical and unknown secondary handoff, tray restore,
  abnormal-session detection, and owned cleanup.
- User waiver: official npm registry audit did not run. After the sandbox and
  escalation refusal exposed that it would upload the locked dependency graph,
  the user explicitly chose to skip the audit on 2026-09-04. No audit success
  is claimed; `AC-RSH-01` and `AC-RSH-13` close under that scope decision.

## Risky Files And Rollback Matrix

| Boundary | Expected files | Rollback trigger |
|---|---|---|
| Dependency/router | `package.json`, lock, `src/app/router/**` | hash refresh/fallback fails, parallel route source appears |
| Runtime bootstrap | `src/main.tsx`, `src/app/app.tsx`, runtime provider | event lost, wrong initial flash, diagnostics/retry regress |
| History/focus | navigation provider/bridge | WebView can leave app, overlay/editable Escape misroutes, focus lost |
| Shell/titlebar | `src/app/layout/**`, WindowTitleBar, styles | second header, drag/window regression, overlap/overflow |
| UI preferences | provider/tests | sibling overwrite, optimistic rollback failure, second persistence source |
| Feature state | placeholder/gate/routes | fake availability, command probing, missing loading/failure state |
| Overview/settings | `src/routes/**`, i18n | fake data/control, runtime loss, inaccessible native form semantics |
| Smoke | `scripts/smoke-runtime.ps1` | process/marker cleanup scope broadens or lifecycle coverage is removed |

任何 boundary 失败都局部回退；不得通过 CSP/capability 放宽、Rust 新 IPC、第二 store/router、localStorage、root resource、ImTip 或 S2/S3 行为绕过。
