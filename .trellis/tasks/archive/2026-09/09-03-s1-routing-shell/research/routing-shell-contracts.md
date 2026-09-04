# S1 路由与应用外壳研究

## 1. 研究范围

本研究只回答 `s1-routing-shell` 的实现边界：路由依赖、七个一级路径、启动/第二实例导航、应用壳组合、侧栏配置、功能占位、返回与焦点语义，以及现有质量门禁。它不设计动作注册表、命令面板、任务反馈或任何 S2+ 领域 command。

## 2. 仓库现状

### 2.1 前端入口与运行时

- `src/main.tsx` 当前同时负责 provider 挂载、feature store 初始化、`app://launch-requested` listener-first 订阅、`app_runtime_snapshot` 获取、启动事件合并、窗口 Hook 和临时运行状态页面。
- `src/runtime-view.ts` 已把 privilege、recovery、launch 和 notice 转换为可测试的展示模型；概览页应复用这些 helper，而不是重新解释 generated payload。
- `src/hooks/use-window-controls.ts` 已建立 listener-first、revision merge、异步 cleanup、warning 可见的 Hook 模式。
- `src/components/window-title-bar/WindowTitleBar.tsx` 已拥有品牌、版本、drag region、最小化/最大化/关闭和 44x44 控件。路由壳应扩展这一条标题栏为唯一顶部应用栏，不叠加第二条固定 header。
- `index.html` 已有 `#root` 和 app-level `#overlay-root`；Dialog、Dropdown Menu 和 Tooltip 均通过 `OverlayProvider` portal 到后者。

结论：把 runtime bootstrap 提取为 app provider/hook，把现有运行状态内容迁入 Overview；`main.tsx` 最终只保留样式、i18n、provider 和 root composition。

### 2.2 配置与功能目录

- generated `UiConfig` 已包含 `sidebarCollapsed?: boolean`；`normalizeUiConfig` 已把它解析为必填 boolean。
- `UiPreferencesProvider` 已实现 bootstrap projection、listener-first snapshot、revision merge、单队列完整 UI group 更新、失败 rollback 和 512 Unicode 标量 warning，但 `UiPreferencePatch` 与 context setter 还只覆盖 theme、density、locale。
- `src/stores/features.ts` 是 feature availability 的唯一前端来源，初态为 loading，并提供 `initialize`、`retry`、完整 catalog replacement、typed `feature`/`isAvailable`。
- generated `AppFeatureId` 已包含本任务所需的 `lexiconRead`、`phraseRead`、`reverseLookup`、`radicalReference` 和 `selfLearning`；catalog record 自带真实 `targetMilestone`。

结论：侧栏状态只扩展现有 provider patch/setter；FeatureGate 只选择现有 store，不创建 Vite flag、第二 store 或 Web Storage。

### 2.3 启动路径与 native 激活

- `src-tauri/src/launch/mod.rs` 的 transport envelope 接受 `/` 开头、最多 256 Unicode 标量的路径，并拒绝控制字符、`\\`、`?`、`#`、空段、`.` 和 `..`。
- 该 parser 只证明路径适合传输，不证明它属于产品 route。generated `LaunchRequest.navigationPath` 的注释也明确把 route ownership 留给 routing layer。
- `RuntimeSnapshot` 同时包含 `primaryLaunch` 与 `latestSecondaryLaunch`；Rust state 会保留 setup 期间抢先到达的 secondary launch。
- `handle_secondary_launch` 先更新 runtime state，再通过既有 `WindowCoordinator.restore()` 还原并置前窗口，随后发送 `app://launch-requested`。前端不得为深链接重复调用窗口 command。
- 当前 Windows smoke 用 `/settings/runtime` 验证 transport handoff。新 catalog 只冻结一级 `/settings`，所以该值将变为产品层 unknown-path 用例；已知路径 smoke 应改用 `/settings` 或另一个 canonical path。

结论：所有入口经过一个 frontend product-path validator；native 继续独占窗口激活，frontend 只更新 route 和可见 warning。

## 3. 路由依赖调查

2026-09-03 从 npm 官方 registry 读取的 `react-router@8.3.1` metadata：

| Field | Value | Project compatibility |
|---|---|---|
| version | `8.3.1` | 精确 pin |
| license | MIT | 当前许可策略可接受 |
| engines.node | `>=22.22.0` | 项目 Volta Node `24.18.1` 满足 |
| peer react | `>=19.2.7` | 项目 React `19.2.8` 满足 |
| peer react-dom | `>=19.2.7` | 项目 React DOM `19.2.8` 满足 |
| unpacked size | `2,800,441` bytes | 不需要额外 router package |
| exports | `.`, `./dom`, `./package.json` 等 | browser/data router API 由单包提供 |

同一 registry 返回 `react-router-dom@8.3.1` `ERR_PNPM_PACKAGE_NOT_FOUND`。因此本任务只增加 `react-router@8.3.1`；不安装不存在的同版本 DOM package，也不引入第二套路由或 history library。

选定公共 API 形态：

- 生产使用 hash router，避免 Tauri 打包资源刷新依赖 SPA server fallback。
- 测试使用 memory router。
- 两者接收同一个 route object factory、route catalog 和 product-path validator，不维护平行路由表。
- 不采用 framework mode、SSR、loader 网络数据层或 query/cache library。

## 4. Canonical Route Contract

| ID | Path | Owner | Feature requirement |
|---|---|---|---|
| `overview` | `/overview` | shell | none |
| `lexicons` | `/lexicons` | future domain | `lexiconRead` |
| `phrases` | `/phrases` | future domain | `phraseRead` |
| `lookup` | `/lookup` | future domain | `reverseLookup` |
| `radicals` | `/radicals` | future domain | `radicalReference` |
| `learning` | `/learning` | future domain | `selfLearning` |
| `settings` | `/settings` | shell | none |

Rules:

- Catalog 是导航顺序、label key、Lucide icon、path 与 feature mapping 的唯一来源。
- `/` 是 replace-only alias，归一到 `/overview`，不作为第八个 catalog record。
- 只接受精确 canonical path；大小写变化、尾斜杠、二级段、query 或 fragment 均为 unknown product path。
- Unknown path replace 到 `/overview` 并保留 bounded warning；feature unavailable 仍停留在其 canonical path 并渲染 page placeholder。
- Route ID 当前不跨 IPC，可由 TypeScript `as const` 推导；等下一任务的 Rust action descriptor 成为真实跨层 consumer 时再提升为 Rust-owned generated enum。

## 5. Initial And Live Navigation Order

首次可渲染路径按以下优先级确定：

1. runtime bootstrap 期间最后一个带非空 `navigationPath` 的 secondary launch；
2. primary launch 的非空 `navigationPath`；
3. 当前 hash 中的内部路径；
4. `/overview`。

没有 navigation path 的 secondary launch 只负责 native 激活和 runtime 诊断，不把已有/待定路径重置为 Overview。初始输入在 RouterProvider 首次绘制前完成 product validation；未知输入直接以 `/overview` 初始化并附 warning，避免错误页闪现。

Router 就绪后：

- 带 canonical path 的 secondary event 使用 push 导航，保留内部返回语义；目标等于当前 path 时使用 replace，避免重复 history entry。
- 未知 path replace 到 Overview 并显示 warning。
- 无 path event 不导航。
- runtime snapshot refresh 只补权威诊断，不因旧 snapshot 重放相同 route。

## 6. Shell And Accessibility Evidence

Repository requirements establish:

- `UX-IA-001..010`：七领域、左侧可折叠边栏、轻量应用栏、常驻状态栏、最小 `1024x640`、内部返回和深链接。
- `UX-INTERACT-013`：入口保持可见，占位必须说明能力和阶段，不能空白或通过 command failure 探测。
- `NFR-A11Y-001..007`：键盘完成、focus visible、对比度、非颜色状态、accessible name、reduced motion 和 200% 字号。
- `.trellis/spec/frontend/tailwind-v4-tokens.md`：唯一 token source、44px target、4/8 spacing、无第二色板/字体/阴影/动效系统。
- `.trellis/spec/frontend/component-guidelines.md`：语义原语、Lucide 窄出口、Radix overlay focus/Escape 归 owner。

本轮 `ui-ux-pro-max` 检索返回的可用规则是：深链接反映当前视图、语义 `nav`/button/link、键盘完整可达、焦点可见、固定 header/footer 不完全遮挡 focus。生成器同时给出了 landing hero、CTA、Google Fonts、绿色 accent 与 GSAP；这些与离线 Windows 工具、既有墨蓝/朱砂 token 及最小动效约束冲突，全部不采用。

## 7. History, Focus And Escape Contract

React Router 提供 URL/history transition，但产品还需维护当前 session 内可安全返回的 entry stack：

- 初始 entry 深度为 0；PUSH 追加并截断 forward entries；REPLACE 替换当前 entry；POP 只在已知 key 中移动 current index。
- `canGoBack` 只在 current index > 0 时成立，不能用 `window.history.length`，否则可能让 WebView 离开应用。
- 导航前记录当前 focused HTMLElement；POP 后若节点仍在 DOM 则恢复，否则聚焦新页面 `h1`。
- 普通 PUSH/REPLACE 后聚焦 route `h1`。焦点管理发生在 route commit 后，且不滚动到被固定栏遮挡的位置。
- `Alt+Left` 在 `canGoBack` 时 preventDefault 并执行内部 `navigate(-1)`；否则 no-op。
- `Esc` 仅在 `canGoBack`、event 未被消费、target 非 input/textarea/select/contenteditable、且没有 active app overlay 时返回。
- Radix Dialog/Menu/Tooltip 先处理 Escape；route handler 通过 `defaultPrevented`、overlay root/active element guard 尊重这一优先级。

## 8. Placeholder And Honest Data Rules

- FeatureGate state matrix：loading -> stable busy skeleton；failed -> bounded visible error + retry；ready but record absent -> fail-closed error；ready unavailable -> placeholder；ready available -> supplied children。
- milestone 只能来自 backend catalog record，不在 route table 复制。
- page/section/inline 三种 variant 共享标题、能力说明、真实阶段和非颜色状态；只有真正可操作的 disabled action 才使用 inline variant。
- Overview 只展示当前已有 privilege、abnormal session、latest launch、runtime/window/config notices 与 retry/refresh。
- Settings 建立八组 IA；theme、density、sidebarCollapsed 是唯一可操作项，使用 native radio/checkbox semantics 并即时保存。其它组只呈现 section placeholder。
- 顶部不伪造系统码表方案和搜索；状态栏不伪造后台任务或输入法状态。

## 9. Verification Consequences

- Frontend tests own catalog uniqueness/order, hash-memory parity, validation/fallback, runtime race, navigation stack/focus/Escape, provider sidebar queue/rollback, feature gate states, Overview regression and Settings semantics。
- Browser checks own two themes、two densities、`1024x640`、`1440x900`、200% root font、reduced motion、long Chinese text、collapsed sidebar、all route states and horizontal overflow。
- Windows smoke continues to own primary/secondary process handoff, hidden launch, window restore and close/tray regression; canonical and unknown product path visuals are additionally checked in the real window because the current PowerShell harness cannot inspect WebView DOM。
- No Rust IPC/config/capability/CSP change is expected. `cargo xtask bindings --check` must pass with zero generated diff。

## 10. Sources

- `package.json`
- `src/main.tsx`
- `src/runtime-view.ts`
- `src/app/providers/ui-preferences-provider.tsx`
- `src/hooks/use-window-controls.ts`
- `src/stores/features.ts`
- `src/components/window-title-bar/WindowTitleBar.tsx`
- `src/components/ui/overlay-provider.tsx`
- `src/types/generated/bindings.ts`
- `src-tauri/src/launch/mod.rs`
- `src-tauri/src/runtime/mod.rs`
- `src-tauri/src/lib.rs`
- `scripts/smoke-runtime.ps1`
- `docs/21-ui-ux.md`
- `docs/20-nonfunctional.md`
- `docs/modules/M7-app-shell.md`
- `.trellis/tasks/08-25-s1-shell-ui/{prd.md,design.md,implement.md}`
- `.trellis/spec/frontend/{directory-structure,ui-platform,component-guidelines,hook-guidelines,state-management,type-safety,tailwind-v4-tokens,quality-guidelines}.md`
- `.trellis/spec/backend/{window-coordinator,windows-system-integration,repository-quality-ci}.md`
- npm official registry metadata for `react-router@8.3.1` and `react-router-dom@8.3.1`, queried 2026-09-03
