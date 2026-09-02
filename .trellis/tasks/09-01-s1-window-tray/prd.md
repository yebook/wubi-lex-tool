# S1 窗口与托盘

## Goal

在已完成的 Tauri runtime 与事务配置基础上，建立可靠、可恢复、键盘可用的 Windows 主窗口和系统托盘生命周期。用户可以通过自定义标题栏、第二实例或托盘安全地隐藏、恢复、置前、最大化和退出；窗口状态在 DPI/显示器变化后仍恢复到可见工作区，任何窗口或配置失败都不阻止应用继续运行。

## User Value

- 关闭或最小化主窗口后，应用仍可从托盘明确恢复或退出，不会出现“进程还在但入口消失”。
- 重启后保留合法窗口位置、尺寸和最大化状态；显示器拔插或缩放变化不会把窗口恢复到屏幕外。
- 无边框窗口提供清晰一致的最小化到托盘、最大化/还原和关闭策略，鼠标与键盘都能操作。
- `/tray` 后台启动和第二实例唤起行为稳定，不闪现主窗口，也不会产生重复窗口或托盘图标。

## Background And Confirmed Facts

- `s1-runtime-lifecycle` 已建立单实例、`/tray` 隐藏启动、第二实例激活队列和唯一 `main` 窗口；当前窗口由 Rust `WebviewWindowBuilder` 创建。
- `s1-config-features` 已提供 `WindowConfig { bounds, maximized, closeAction }`、事务保存、revision 和完整快照事件；默认关闭行为是 `minimizeToTray`。
- 当前窗口仍为有边框 `960x680`、最小 `720x520`，与父任务要求的最小 `1024x640` 不一致；本任务负责收敛。
- Tauri 2.11.5 的核心 `tray-icon` feature 提供 `TrayIconBuilder`、左键事件、菜单、`tray_by_id` 和归属式移除；不需要引入 tray plugin。窗口 API 提供 `CloseRequested`、Moved/Resized/ScaleFactorChanged、monitor 查询和 `set_skip_taskbar`。
- 旧版在首次最小化或 `/tray` 延迟启动时创建托盘，恢复后保留图标，进程退出时删除；这只作为行为证据，不迁移旧 UI 或 ImTip。
- UI/UX 规则要求原生语义按钮、可见焦点、视觉顺序 Tab、可访问名称、至少 44x44 控制尺寸、Lucide 图标、reduced-motion 和最小 `1024x640` 自适应布局。
- 根目录 `resource/` 不是输入；ImTip 在窗口、托盘、菜单、动作、能力和依赖中永久禁止。

## In Scope

### 1. Window Coordinator

- 在 Rust 应用层建立主窗口协调边界，统一处理创建、隐藏到托盘、恢复/置前、最大化切换、关闭请求和显式退出。
- 标题栏最小化、任务栏触发的原生最小化和默认关闭策略统一为隐藏到托盘；`closeAction=exit` 与托盘显式退出才结束进程。
- 第二实例和托盘恢复复用同一原生激活流程：取消任务栏跳过、解除最小化、显示并聚焦；失败进入结构化、可见且有界的 runtime notice。
- 读取当前 `closeAction`：`minimizeToTray` 拦截关闭并隐藏，`exit` 允许干净退出；退出路径仍清理 session marker 和托盘资源。
- 正常可见窗口不得重复创建；重复事件、重复隐藏/恢复和重复托盘创建保持幂等。

### 2. Bounds Persistence And Display Correction

- 只持久化最后一个正常态窗口矩形、最大化状态和采样时 scale factor；最小化或最大化产生的瞬时矩形不能覆盖正常态 bounds。
- Moved/Resized/ScaleFactorChanged 使用有界合并与后台持久化，不在 UI/event-loop 线程执行可能超过 100ms 的配置事务。
- 启动恢复先把已存逻辑 bounds 投影到当前显示器集合，再按工作区求交、限幅和最小尺寸校正；无有效交集时在可用显示器工作区内居中，至少保证标题栏和可操作区域可见。
- 配置只读或保存失败不阻止移动/缩放；保留当前原生状态并产生脱敏 notice，不伪造保存成功。

### 3. Frameless Title Bar

- 主窗口改为无边框，并在现有 React runtime surface 上增加紧凑标题栏：产品图标/名称、版本，以及最小化、最大化/还原、关闭按钮。
- 使用 Tauri drag region；按钮和其他交互控件不属于拖动区。标题栏双击遵循最大化/还原语义。
- 窗口按钮使用 Lucide 熟悉图标、原生 `button`、中文 accessible name、tooltip、可见 focus ring 和稳定 44x44 命中区域；最大化图标与状态一致。
- 不在本任务建立最终侧栏、应用栏、主题系统、路由或设置页面；标题栏样式只建立可复用的窗口控制约束。

### 4. Tray Lifecycle

- 普通可见启动不预先创建托盘；首次隐藏时创建一个固定 ID 的托盘图标，此后恢复仍保留，进程退出时归属式清理。
- `/tray` 启动保持主窗口从创建起不可见，并按需求延迟 3 秒创建托盘；期间第二实例到达时取消等待并立即恢复主窗口。
- 左键释放恢复并置前主窗口；右键使用原生菜单；显式“退出”绕过 minimize-to-tray 拦截并走干净退出。
- 本任务的右键菜单只提供必要且真实可用的“显示 WubiLex”和“退出”；不建立动作投影接口，不复制领域 label、route ID、feature 判断或 disabled placeholder，后期具备统一动作目录后再扩展。
- 隐藏时从任务栏移除，恢复时重新进入任务栏；不调用 `EmptyWorkingSet`，内存回收优化在有测量证据前保持延期。

### 5. Testing And Visibility

- 窗口/托盘状态转换放入可注入或纯状态机边界，覆盖关闭策略、幂等、延迟托盘、第二实例竞态、保存合并和失败恢复。
- monitor 校正使用纯矩形算法测试单屏、多屏、负坐标、不同 scale、显示器拔插、过大/过小和完全离屏。
- 前端组件测试覆盖按钮名称、tooltip、最大化状态、键盘操作、拖动区隔离和失败可见性。
- Windows smoke 验证普通启动、`/tray` 无闪窗、最小化/恢复、任务栏状态、唯一托盘图标、关闭策略和退出清理，只操作本次启动拥有的进程与托盘资源。

## Out Of Scope

- 七领域路由、侧栏、最终应用栏、主题/密度/i18n provider 和设置界面；由后续 UI foundation / routing shell 任务实现。
- 全局动作注册表、命令面板、应用内快捷键、全局热键和可改绑 `window.hide`；由 `s1-actions-keymap` 实现。
- 完整领域托盘菜单、route/action 驱动的菜单投影和未实现功能占位；等统一动作目录与路由存在后扩展，本任务不建立临时第二事实源。
- S2/S3 领域命令、输入法状态查询、动态勾选菜单、码表方案 tooltip、系统设置/外部程序/URL 打开和任何系统写入。
- `M7-WIN-009` 的主动工作集清理；只有独立测量证明收益且无卡顿回归后才重新评审。
- ImTip 的任何菜单项、占位、动作、探测、启动或 URL；读取或转换根目录 `resource/`。

## Acceptance Criteria

- [x] `AC-WINDOW-01` 主窗口无边框，最小尺寸为 `1024x640`；标题栏图标/产品名/版本和三个窗口控制在鼠标、键盘及 200% 系统字号下均可用且不重叠。
- [x] `AC-WINDOW-02` 最小化会按项目约定隐藏到托盘，最大化/还原、关闭策略和标题栏双击具有一致 Windows 语义；按钮有可访问名称、tooltip、可见 focus，交互控件不会误触窗口拖动。
- [x] `AC-WINDOW-03` 默认关闭拦截为隐藏到托盘；`closeAction=exit` 和托盘显式退出会干净退出，session marker 与 owned tray 均被清理。
- [x] `AC-WINDOW-04` 普通启动不创建 tray；首次隐藏只创建一个固定 ID 图标，恢复后图标保留，重复隐藏/恢复不重复创建。
- [x] `AC-WINDOW-05` `/tray` 从首帧保持隐藏并延迟 3 秒创建 tray；等待期间第二实例会恢复窗口且不会遗留延迟创建或重复图标。
- [x] `AC-WINDOW-06` 托盘左键、第二实例和内部恢复命令复用同一 restore/foreground 状态机，并正确恢复任务栏可见性；失败产生有界、脱敏的可见 notice。
- [x] `AC-WINDOW-07` 正常态 bounds、maximized 和 scale factor 通过现有事务配置保存；窗口事件合并不会阻塞 UI 线程，保存失败不改变原生窗口状态且可见告警。
- [x] `AC-WINDOW-08` 单屏、多屏、负坐标、DPI 改变、显示器移除、完全离屏、过大/过小 bounds 均恢复到至少一个 monitor 的可见工作区内，且不会持久化最小化/最大化瞬时矩形。
- [x] `AC-WINDOW-09` native window/tray 操作、配置更新、事件 emit 任一失败均保持应用可用；退出意图不会被失败的状态保存永久阻断。
- [x] `AC-WINDOW-10` Rust/TypeScript 共享的 window state/command/event 类型只从唯一 bindings registry 生成，前端不直接复制 native 状态机或手写 wire type。
- [x] `AC-WINDOW-11` Windows smoke 覆盖普通启动、hidden `/tray`、close-to-tray、restore/foreground、taskbar 切换、唯一 tray 和显式退出清理，且不删除历史 session markers 或其他进程资源。
- [x] `AC-WINDOW-12` Rust、bindings/docs、pnpm、task validation 和生产静态搜索门禁通过；没有 S2/S3 行为、额外包管理器来源、tray plugin、root `resource/` 读取或 ImTip surface。

## Key Decisions

- 用户于 2026-09-01 选择最小必要托盘菜单：本任务只实现显示/置前与退出，完整领域菜单后期由统一动作目录扩展。
- 当前任务不为后续托盘扩展预建 descriptor、placeholder 或 route/action adapter；届时基于真实统一目录重构。
- 托盘使用 Tauri 核心 `tray-icon` feature，不增加 tray plugin 或前端托盘控制面。
- 托盘图标首次隐藏后持续存在至进程退出；普通可见启动不提前创建，`/tray` 启动遵守 3 秒延迟。
- Rust window coordinator 是 native 状态转换的唯一所有者；React 标题栏只发出生成的 typed intent 并渲染生成状态。
- 本任务不主动清理工作集，不为未实现路由/动作创建可点击或禁用的临时菜单项。

## Risks And Deferred Items

- 父计划把 window/tray 排在 action catalog 与 routing 前；本任务用最小真实菜单消除依赖，完整菜单明确推迟到统一动作目录存在之后。
- 无边框窗口会失去系统标题栏默认命中区与系统菜单行为，必须通过 Windows smoke 验证拖动、双击、DPI、任务栏和焦点，不只依赖组件测试。
- `/tray` 的 3 秒延迟来自旧版兼容行为；实现必须保证第二实例或显式退出能取消延迟任务，不能在状态变化后异步补建 tray。
- 配置保存是磁盘事务；窗口 move/resize 高频事件必须合并，否则会造成 I/O 风暴和大量 backup。
