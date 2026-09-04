# S1 路由与应用外壳

## Goal

在已完成的 Tauri 窗口、配置、feature catalog 与 UI foundation 上建立第一个可长期扩展的桌面应用壳：七个一级领域均有稳定入口和内部路径，启动参数与第二实例可以导航，侧栏状态可持久化，概览保留真实运行诊断，设置页提供已经可用的外观控制，其余未交付能力以一致且诚实的占位呈现。本任务只完成路由与外壳，不提前实现动作注册表、快捷键系统、任务反馈或 S2+ 领域功能。

## User Value

- 用户可以从固定侧栏进入概览、码表、短语、反查、字根、学习和设置，不再停留在临时运行状态页。
- 启动或第二实例传入合法内部路径时会到达同一个页面；未知路径有可见说明，不会白屏或落入死链。
- 未实现领域仍然可发现，但明确显示能力、阶段和不可用状态，不会触发不存在的 command 或伪造系统数据。
- 主题、密度和侧栏折叠可以在真实设置界面即时应用并沿用现有事务配置保存。
- 键盘用户在路由切换、返回和折叠导航中保持可预测焦点，固定应用栏和状态栏不会遮挡内容。

## Background And Confirmed Facts

- `s1-runtime-lifecycle`、`s1-config-features`、`s1-window-tray` 与 `s1-ui-foundation` 已完成。当前页面仍由 `src/main.tsx` 直接承载 runtime snapshot、launch event、窗口 Hook 与临时状态布局。
- Rust 已提供 `LaunchRequest.navigationPath`、`--navigate <path>`、`app://launch-requested` 和 `app_runtime_snapshot`。传输层已限制路径必须以 `/` 开头、最多 256 个 Unicode 标量、不含控制字符、查询、片段、反斜杠、空段、`.` 或 `..`；路由层只需验证产品路径，无需修改 Rust IPC。
- `UiConfig.sidebarCollapsed`、`config_update_ui`、`config://changed` 和 `UiPreferencesProvider` 的串行全组更新已经存在；本任务只扩展 provider 的侧栏 setter，不建立第二 store 或 Web Storage。
- `AppFeatureCatalog` 与 Zustand feature store 是未来能力可用性的唯一来源。当前 12 个 generated feature ID 覆盖 S2-S8；路由和占位不得维护 Vite flag 或从 command failure 推断可用性。
- `theme.css`、zh-CN i18n、Button/Input/Kbd/Dialog/Menu/Tooltip/Overlay、Lucide 窄出口与 44x44 窗口控制已建立，必须复用。
- 父任务固定七领域 IA、左侧可折叠导航、轻量应用栏、常驻状态栏、`Esc` / `Alt+Left` 返回语义、稳定深链接和三种占位层级。
- 2026-09-03 官方 npm metadata 显示 `react-router@8.3.1` 要求 Node `>=22.22.0`、React/React DOM `>=19.2.7`，兼容本项目 Node `24.18.1` 与 React `19.2.8`；`react-router-dom@8.3.1` 不存在。
- UI 检索中可采用的是平面、低动效、清晰边界、完整键盘和深链接建议；其 landing hero、青绿色板、在线字体与营销结构不适用于本离线 Windows 工具，继续以仓库墨蓝/朱砂令牌为准。
- 根目录 `resource/` 不是输入；ImTip 永久禁止。

## In Scope

### 1. Router Dependency And Canonical Catalog

- 使用项目 Volta pnpm 精确增加唯一运行时依赖 `react-router@8.3.1`，不增加 `react-router-dom`、第二 Router、查询库或历史状态库。
- 在 `src/app/router/` 建立单一 route catalog，冻结七个 frontend-owned route ID、路径、i18n key、Lucide icon、顺序和可选 generated feature requirement：

| Route ID | Path | Label | Feature requirement |
|---|---|---|---|
| `overview` | `/overview` | 概览 | shell-owned |
| `lexicons` | `/lexicons` | 码表 | `lexiconRead` |
| `phrases` | `/phrases` | 短语 | `phraseRead` |
| `lookup` | `/lookup` | 反查 | `reverseLookup` |
| `radicals` | `/radicals` | 字根 | `radicalReference` |
| `learning` | `/learning` | 学习 | `selfLearning` |
| `settings` | `/settings` | 设置 | shell-owned |

- `/` 只作为 replace 到 `/overview` 的入口别名；未知路径不进入 catalog。Route ID 当前不跨 IPC，因此由 catalog 的 `as const` 推导；下一任务若把 route target 放进 Rust action descriptor，必须把同一枚举提升为 Rust-owned generated contract。
- 生产 WebView 使用 hash history，使打包资源刷新不依赖服务器 fallback；内部/启动路径保持上表的无 hash 形式。测试通过同一 route objects 创建 memory router，不复制路由表。

### 2. Launch And Deep-Link Consumption

- 把现有 runtime listener-first/snapshot merge 从 `main.tsx` 收敛到可测试的 app runtime provider/hook，继续保留 privilege、recovery、launch notice、window notice、refresh 与错误状态。
- 初次 Router 创建前按“启动期间最后一个带路径的 secondary launch > snapshot latest secondary > primary launch > 当前 hash > `/overview`”消费 `navigationPath`，避免先展示错误页面再跳转；无路径 secondary 不重置已选页面。
- Router 就绪后的 `app://launch-requested` 使用同一产品路径 validator；合法路径只更新前端导航，feature 不可用仍进入相应页面占位。窗口还原/置前继续由 Rust 现有 single-instance handler 与 WindowCoordinator 独占，前端不重复调用窗口 command。
- `/` 归一到 `/overview`；未知或非 canonical path 回到 `/overview` 并显示有界中文 warning。传输 envelope 已由 Rust 拒绝的路径继续沿用 launch notice，不在前端重复解析原始 argv。
- 浏览器地址中的未知 hash 同样回到概览并显示 warning，不产生白屏、错误堆栈或外部导航。

### 3. Application Shell

- `src/main.tsx` 仅保留 styles、i18n、app providers 和 root composition；外壳、路由与 runtime bootstrap 移入各自 owner。
- 页面结构固定为唯一 WindowTitleBar/AppBar、Sidebar、RouteOutlet、StatusBar 与已有 app-level Overlay root。现有 WindowTitleBar 直接演进为唯一顶部应用栏，继续独占 native window intents 和 drag-region 约束；不叠加第二条固定 header。
- 唯一应用栏显示 WubiLex 品牌、当前页面标题与原生窗口控制。当前系统码表方案尚无 S1 权威数据，全局搜索/命令面板由 `s1-actions-keymap` 所有；本任务不伪造方案徽标、不创建临时搜索实现。
- 左侧导航按七领域顺序显示，设置固定在底部。展开态显示 icon + label，折叠态保留 icon、accessible name、tooltip 和选中状态；折叠按钮是 44x44 icon button。
- `sidebarCollapsed` 从 UiPreferencesProvider 读取并通过其完整 UI group 队列即时投影、事务保存和失败回滚，不创建独立 Zustand slice 或 localStorage。
- 底部状态栏只投影当前已有的真实 runtime/配置/导航 warning 与就绪状态。后台任务进度和输入法状态分别等 `s1-task-feedback` 与领域能力提供后接入，不显示假值。
- 布局在 Tauri 最小 `1024x640`、常用 `1440x900`、standard/compact、light/dark/system、200% 根字号和 reduced-motion 下无横向滚动、遮挡或焦点覆盖。

### 4. Overview And Settings

- 概览页接管当前 runtime surface 的真实 privilege、异常会话、最近启动请求和 notices；保留 loading、failure、retry、listener warning 与 refresh 行为。
- 概览状态项采用平铺重复项或无框 section，不嵌套卡片。系统码表、短语、备份、输入法和健康数据在 S2/S3 前不虚构；需要预告时使用 feature-backed section/inline placeholder。
- 设置页建立输入法、五笔行为、候选窗口、快捷键、外观、网络、数据、关于八组信息架构。外观组是本任务唯一完整可操作组，提供 native radio/segmented semantics 的主题三选一与密度二选一，并沿用 provider warning。
- 侧栏按钮与外观组的 native checkbox/toggle 复用同一个 `setSidebarCollapsed`，两处同步反映当前状态；`zh-CN` 是唯一 locale，不渲染无意义的单项语言选择器。
- 其余设置组只显示与真实 feature/task owner 对应的 section placeholder，不增加开关、表单、保存按钮或假默认值。即时保存沿用现有 provider；本任务不暴露 window close action、keymap 或配置导入导出 UI。

### 5. Feature Placeholder Contract

- 新建共享 `FeaturePlaceholder`，支持 `page`、`section`、`inline` 三种 variant；每种都使用统一“功能暂未完善”文案、具体能力说明、真实 milestone 和非颜色状态。
- `FeatureGate` 只消费 Zustand catalog 中的 generated `AppFeatureId`。loading 保持稳定 skeleton/busy 状态；catalog failure 显示有界可重试错误；unavailable 显示 placeholder；available 渲染真实 children，绝不捕获 `command not found`。
- 五个未来领域 route 使用 page gate；概览或设置中的未来区块使用 section gate；未来操作槽可用 inline gate。导航项保持可点击，并根据 catalog 显示“开发中”状态而不禁用 route。
- shell 后续子任务（搜索、快捷键、任务反馈）没有伪造的 backend feature ID；本任务通过明确 out-of-scope 或静态任务归属处理，不扩展 Cargo feature catalog。
- Feature placeholder 不能读取根目录资源、注册命令、执行系统写入或创建假业务数据。

### 6. Navigation And Accessibility

- 侧栏使用语义 `nav` 和链接；当前项通过 `aria-current="page"` 及非颜色样式表达。DOM 顺序与视觉顺序一致，icon 均经 `src/icons/` 窄 Lucide 出口。
- 普通 push 导航后把焦点移到页面 `h1`；browser/hash POP 返回优先恢复到离开该历史项时的触发元素，目标不存在时回退页面标题。
- `Alt+Left` 使用同一内部 history 返回并阻止 WebView 离开应用；`Esc` 仅在存在内部返回项、事件未被 overlay 消费且焦点不在输入/编辑控件时返回。顶层无 history 时两者安全 no-op。
- Dialog/Menu 的 Escape 优先级高于路由返回；输入、textarea、select、contenteditable 与未来编辑器不会因输入 Escape 被强制离页。
- 固定应用栏/状态栏配合内容 scroll padding，保证 keyboard focus 不被完全遮挡；所有折叠/导航控件保持可见 focus 和 44x44 target。

### 7. Testing And Verification

- route catalog 测试覆盖七项顺序、唯一 ID/path、`/` 归一、未知 path、feature mapping 与未来扩展的 fail-closed 行为。
- Router 测试覆盖 sidebar push、hash/memory parity、primary/secondary deep link、未知路径 warning、feature-unavailable page、catalog loading/failure/retry 与 available children。
- 焦点/键盘测试覆盖 route h1、POP 触发器恢复、fallback focus、`Esc`、`Alt+Left`、顶层 no-op、输入控件和打开 overlay 不回退。
- layout/settings 测试覆盖 expanded/collapsed 导航、tooltip/accessible name、sidebar config queue/rollback、theme/density native radio semantics、status warning 和 runtime regression。
- 浏览器视觉检查覆盖七路由、两主题、两密度、两 viewport、200% 字号、长中文、reduced-motion、折叠侧栏、占位三 variant、loading/failure 与无横向 overflow。
- Windows smoke 增加 primary `--navigate /settings`、hidden combined launch、secondary navigation、unknown product path fallback、tray/close/exit regression；不读取真实用户配置。

## Out Of Scope

- `Ctrl+1..7`、`Ctrl+K`、动作注册表、frontend dispatcher、命令面板、快捷键录制/冲突、全局热键和领域托盘扩展；由 `s1-actions-keymap` 实现。
- 后台 task registry、任务进度、取消、toast、错误详情、确认、拖放、空状态和首次知情说明；由 `s1-task-feedback` 实现。
- 码表、短语、反查、字根、学习、系统设置、资源、迁移或输入法状态的真实读取、编辑、写入和业务组件。
- 码表库/编辑器、设置分组等二级/动态 route；本任务冻结七个一级 path，后续真实 consumer 按 central catalog 规则扩展。
- 新 config 字段、schema migration、新 Tauri command/event、Rust route enum、CSP/capability 放宽、query/cache library 或 Web Storage。
- 当前系统码表方案徽标、可用的全局搜索框和后台任务/输入法状态；没有权威数据或 owning task 前不显示假值。
- 根目录 `resource/`、在线字体、GSAP、decorative animation、React Router framework mode、SSR 或网络路由服务。
- ImTip 的任何 route、导航项、占位、搜索结果、设置项、动作、依赖或资源。

## Acceptance Criteria

- [x] `AC-RSH-01` 精确依赖 `react-router@8.3.1` 且无 `react-router-dom`、第二 Router 或其它 package-manager/version source；frozen install 通过，audit 经用户于 2026-09-04 明确决定跳过。
- [x] `AC-RSH-02` 单一 typed route catalog 包含且只包含七个一级 route ID/path，顺序和 feature mapping 与本 PRD 一致；`/` replace 到 `/overview`。
- [x] `AC-RSH-03` 生产 hash router 与测试 memory router 复用同一 route objects/validator；刷新不依赖服务器 fallback，未知 URL 不白屏。
- [x] `AC-RSH-04` primary launch、启动期间 secondary launch、运行中 `app://launch-requested` 和浏览器 hash 均进入同一产品路径 validator；未知 path 回概览并显示 warning，known unavailable path 进入对应占位。
- [x] `AC-RSH-05` AppShell 由唯一 WindowTitleBar/AppBar、可折叠侧栏、route outlet、常驻状态栏和既有 overlay root 组成；不出现第二顶部栏，native titlebar、drag region、44x44 controls、tray/window behavior 不回归。
- [x] `AC-RSH-06` 七个导航入口均可通过鼠标与键盘访问，当前项有 `aria-current` 和非颜色状态；折叠后 label 不占布局但 accessible name、tooltip、选中态和 target 保持。
- [x] `AC-RSH-07` `sidebarCollapsed` 通过 UiPreferencesProvider 乐观应用、串行保存、事件合并和失败回滚；无第二 store、localStorage、sessionStorage 或 cookie。
- [x] `AC-RSH-08` 概览页保留现有 runtime loading/error/retry、privilege、recovery、launch 与 notices 的真实数据流；不伪造系统码表、短语、备份、输入法或健康数据。
- [x] `AC-RSH-09` 设置页建立八组 IA，外观组可即时设置 light/dark/system、standard/compact 与 sidebar collapsed/expanded，单一 zh-CN 不渲染伪选择器；其它组无假表单或系统副作用。
- [x] `AC-RSH-10` FeaturePlaceholder 的 page/section/inline、说明、milestone 和 accessible state 均有测试；FeatureGate 对 loading/failed/unavailable/available fail closed，并只消费 backend catalog。
- [x] `AC-RSH-11` route push 后聚焦页面 h1；POP 返回恢复触发元素或回退标题；`Esc` / `Alt+Left` 复用内部 history，顶层、输入控件和 overlay 场景不会误导航或让 WebView 离开应用。
- [x] `AC-RSH-12` light/dark/system、standard/compact、`1024x640`、`1440x900`、200% 字号、reduced-motion、长文案和折叠侧栏下无横向 overflow、文本遮挡、焦点遮挡或 incoherent overlap。
- [x] `AC-RSH-13` frontend format/typecheck/lint/test/build、bindings/docs、task validation、静态搜索和 Windows runtime smoke 全部通过；dependency audit 经用户于 2026-09-04 明确决定跳过，Router 变更未产生 generated binding diff。
- [x] `AC-RSH-14` 产品代码、manifest、路由、导航、占位、设置、文案和依赖不读取根目录 `resource/`，不含 ImTip surface，不触发 S2/S3 command 或系统写入。

## Key Decisions

- 使用 `react-router@8.3.1` 单包 API；不安装不存在的同版本 `react-router-dom`。
- 生产采用 hash history，外部 `--navigate` 继续传无 hash canonical path；这同时满足打包 WebView 刷新和稳定内部 URL，不要求 Tauri 提供 SPA server fallback。
- route ID 在本任务尚未跨 IPC，先由 frontend catalog 推导；等 Rust action descriptor 首次真正消费时再提升为 generated Rust contract，避免为未来接口预建无消费者 wire type。
- 复用现有 launch event/snapshot；不新增 Rust route command、event 或配置 schema。
- 第二实例窗口激活继续由 Rust WindowCoordinator 负责；前端 route bridge 只消费 navigationPath，避免重复 show/focus command 和双 owner。
- 现有 WindowTitleBar 直接承担唯一顶部应用栏，不为当前页面标题另建第二条 header。
- 外壳只显示有权威数据的内容。方案徽标、全局搜索、任务进度和输入法状态保持空缺或由 owning task 后续接入，不用假数据填满设计稿。
- `Esc` 返回必须尊重 overlays 与编辑控件，`Alt+Left` 必须截获为内部返回；两者不与下一任务的快捷键注册重复。
- UI 搜索结果只采纳适合桌面工具的 flat/low-motion、keyboard、focus 与 deep-link 规则；拒绝 landing、青绿色板、Google Fonts 和营销结构。

## Risks And Deferred Items

- `LaunchRequestedEvent` 没有 backend revision。runtime provider 必须用本地单调序号区分新 event 与 snapshot refresh，并让同 path 导航幂等，避免重复 history entry。
- Hash history、primary deep link 与 Router 创建顺序若处理不当会短暂显示概览。实现必须在初始 runtime path 判定后创建 Router，浏览器无 Tauri snapshot 时使用当前 hash 或 `/overview`。
- 全局 Escape 可能与 Radix Dialog/Menu 冲突。监听必须在未被消费、无打开 overlay 且目标非编辑控件时才返回，并以真实 user-event 测试验证。
- Feature catalog 在 `--all-features` 测试构建中可能标记未来能力 available；FeatureGate 必须在 available 时渲染 supplied children，领域 route 只提供无假数据的 shell-owned结构，真实页面由后续阶段替换。
- 父任务要求的命令面板、快捷入口、完整状态栏和托盘领域动作跨越后续子任务，本任务只冻结可供它们消费的 route catalog 与导航入口。
