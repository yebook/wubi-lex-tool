# Design - S1 外壳与 UI 骨架

## 1. Delivery Shape

S1 采用一个父任务和八个可独立验收的子任务。实现顺序优先冻结跨层契约，再构建窗口与视觉壳，最后接入动作、任务和集成验收：

```text
runtime lifecycle
  -> config + features
  -> window + tray ------------------+
                                      +-> actions + keymap --+
ui foundation -> routing + shell ----+                     |
                       \-> task + feedback -----------------+
                                                             -> integration
```

父任务不承载产品代码；各子任务有自己的 PRD、设计、实现清单、检查上下文和提交。跨子任务变更必须先更新本设计中的共同合同。

## 2. Runtime And Security Boundary

`src-tauri` 新增真实 desktop entry，并保持 library entry 可供测试和 bindings 生成复用。应用启动顺序固定为：

1. 解析并校验启动参数。
2. 初始化最小滚动日志与 panic/异常退出记录。
3. 建立单实例处理；第二实例只转交参数并退出。
4. 检测管理员权限和未清理会话标记。
5. 加载/恢复配置，计算 backend feature set。
6. 注册窗口、托盘、动作和全局热键。
7. 创建 WebView，注入首帧主题，等待前端 `app-ready`。

Tauri capability 只开放实际使用的窗口、托盘、事件和必要 OS 能力；CSP 禁止 `eval`、内联业务脚本和非 HTTPS 网络源。首帧主题引导脚本必须是最小的、固定哈希或 Tauri 允许的受控入口，不能把 CSP 放宽为任意 inline script。

一期 bundle manifest 请求管理员权限，运行时仍独立检测实际 token，避免“manifest 存在”被误当作权限事实。所有后续领域系统操作在 S1 都不可用；命令面板可以显示禁用的 feature placeholder，窗口子任务的最小托盘只显示窗口与退出动作。

`M7-WIN-005` 是永久 P3 禁止项，不是 feature placeholder。应用不定义 ImTip route、action、tray item、setting、feature ID、command、capability、process adapter、URL 或 dependency，也不建立通用“相关工具”入口来绕过该决定。

## 3. Application State And IPC

Rust 侧扩展 `AppState`，只包含 S1 的共享状态：

- `ConfigService`：强类型配置、迁移、原子保存和备份。
- `FeatureSet`：从 Cargo features 计算的只读能力集合。
- `ActionRegistry` / `KeymapService`：动作定义、有效绑定和全局注册状态。
- `TaskRegistry`：任务状态、互斥组和 cancellation token。
- `SessionState`：当前运行标记、异常退出和恢复检查结果。
- `WindowState`：托盘/窗口协调需要的短生命周期状态。

所有 Tauri command、event payload 和跨层枚举先在 Rust 定义，再由现有 `tauri-specta` registry 生成 TypeScript。前端只能通过 `src/lib/` 的薄 IPC wrapper 使用生成类型，不在 store、组件或测试里复制 wire type。

建议的 S1 command 分组：

| Prefix | Responsibility |
|---|---|
| `app_*` | 应用信息、功能集合、会话/恢复状态、退出 |
| `config_*` | 获取快照、更新分组、导入、导出、恢复默认 |
| `window_*` | 最小化到托盘、还原、关闭策略和 bounds |
| `action_*` | 动作目录、执行、当前可用性 |
| `keymap_*` | 校验、应用、清除、恢复、导入导出 |
| `hotkey_*` | 全局绑定探测、注册结果和注销 |
| `task_*` | 列表、详情、取消 |

Command 返回统一 `AppError`；event 使用 `<domain>://<name>` 命名和强类型 payload。前端不依赖字符串错误或无类型 `unknown` 强转。

## 4. Configuration Contract

配置文件位于 Tauri 应用自有配置目录，格式为版本化 TOML。顶层至少包含：

- `schemaVersion`
- `window`：bounds、maximized、closeAction
- `ui`：theme、density、locale、sidebarCollapsed、onboardingVersion
- `keymap.bindings`

写入流程为：内存变更校验 -> 写同目录临时文件 -> flush -> 备份当前有效文件 -> 原子替换 -> 发布 `config://changed`。任何一步失败时内存状态和最后有效文件保持一致，并返回结构化错误。

读取流程为：不存在则创建默认配置；存在则解析并逐版本迁移；解析或迁移失败则把原文件重命名为带时间戳的 `.corrupt` 副本，加载默认值，并记录用户可见 warning。测试通过注入临时目录和 clock 保持确定性。

导入使用同一 parser/migrator/validator，在临时状态验证完全成功后才提交。导出不包含运行会话标记或未来的用户数据。

## 5. Features And Placeholder Contract

Cargo feature 是能力开关唯一来源。`app_features` 返回稳定 feature ID、availability、target milestone 和可选 reason；前端启动时一次拉取并写入 Zustand feature store。

占位渲染分三种：

- page：整个领域未实现。
- section：页面存在但某一能力后置。
- inline：单个动作或状态不可用。

占位必须包含能力名称、预计阶段和不产生副作用的状态。它不得通过捕获 `command not found` 推断，也不得在 Vite 环境变量维护第二套列表。已实现 feature 的 route/action 测试必须断言占位消失。

## 6. Window, Tray And Deep Links

主窗口最小内容尺寸为 `1024x640`。保存 bounds 时记录逻辑坐标和缩放信息；恢复时与当前 monitor work area 求交，不可见或过小则居中回退。最大化状态单独保存，不能把最大化后的物理 bounds 当作普通窗口 bounds。

关闭事件由统一 coordinator 处理：

- `minimize`：阻止关闭、隐藏任务栏窗口并保留托盘。
- `exit`：检查未保存状态和不可中断任务，完成热键/托盘/会话标记清理后退出。

窗口子任务先交付固定且真实可用的“显示 WubiLex / 退出”最小托盘，不建立领域 descriptor 或 disabled placeholder。统一动作目录与路由交付后，完整领域托盘菜单再从共享动作目录投影并替换该最小构建；动态状态只在真实模块交付后接入事件驱动缓存。

内部 route ID 与 URL path 建立一一映射。合法来源包括侧栏、命令面板、托盘、第二实例参数和 `navigate` event；所有来源进入同一 route validator。未知或当前 feature 不可用的深链接跳到对应占位页并显示 warning，而不是白屏。

## 7. Action And Keymap Contract

Rust 静态动作目录定义：`id`、i18n key、group、scope、default binding、feature requirement 和 execution target。前端得到生成的只读 descriptor；不可自行新增“仅 UI 知道”的全局动作。

执行分两类：

- frontend target：页面跳转、打开命令面板、切换主题等，由统一 frontend dispatcher 处理。
- native target：窗口、托盘、全局热键等，通过 typed command 执行。

全局热键触发后发出 `action://invoked`，再走同一 dispatcher。统一动作目录交付后，命令面板、扩展托盘、键盘和按钮只提交 action ID，不复制执行逻辑；此前窗口子任务的两项最小托盘由 native coordinator 直接所有。

绑定变更是事务：语法/保留键/应用内冲突校验 -> 尝试建立新全局注册 -> 更新内存 keymap -> 保存配置 -> 撤销旧注册。任一步失败都撤销新注册并恢复旧绑定。使用可替换的 global-shortcut adapter 做占用和 rollback 单测。

`Ctrl+Shift+H` 是 `window.hide` 默认值；`Ctrl+W` 不保留隐藏语义。`Ctrl+1..7`、`Ctrl+K`、`Ctrl+F2` 和返回键遵循 M7 新动作表。编辑器上下文动作虽在目录中可见，但在对应 feature 未交付前保持 unavailable。

## 8. Frontend Composition

前端入口只负责挂载 provider tree。Provider 顺序为：首帧 theme bootstrap -> i18n -> IPC/app bootstrap -> router -> shortcuts/overlays。错误边界包住应用壳，但不吞掉结构化错误详情。

页面结构固定为：

```text
AppShell
  AppBar
  Sidebar
  RouteOutlet
  StatusBar
  OverlayHost (command palette, dialogs, toast, error detail)
```

路由目录按七领域组织。概览状态卡片是平铺的重复数据项；设置分组是独立的低浮层区块，不嵌套卡片。页面 section 使用无框布局，浮层才使用阴影。桌面工具强调扫描与重复操作效率，不使用营销 hero、大标题、装饰渐变、圆球或卡片堆叠。

组件只使用 `theme.css` 暴露的语义令牌。交互控件优先采用语义原语；icon-only button 使用 Lucide、稳定的点击区域和 tooltip/accessible name。动效只用 transform/opacity，遵守 120/200 ms 令牌和 reduced-motion。

## 9. Task, Feedback And Error Flow

后台任务由 Rust registry 持有权威状态，前端 store 订阅 `task://started`、`task://progress`、`task://finished`。每条 event 带 task ID 和单调 revision；前端启动或丢事件后通过 `task_list` 重建，不把 event stream 当唯一持久事实。

取消是协作式的。任务 descriptor 明确 `cancellable` 和 `phaseCancellable`；UI 只在允许时显示可操作取消按钮。互斥冲突由 registry 拒绝并返回占用任务详情，不只依赖 disabled button。

进度 UI 共享同一数据模型，但提供状态栏 inline 和不可中断阶段 overlay 两种投影。S1 用合成的 12 阶段任务验证完整形态，不注册真实 TSF 操作；S3 只需把实际 task event 接到该组件。

`AppError` 分离稳定 code、中文 summary、cause、recovery、technical details 和可选 source。Toast 只用于成功或短暂信息；错误保留在状态栏/错误中心直到用户处理。复制详情默认脱敏，不包含用户输入、码表或短语内容。

## 10. Accessibility And Keyboard Model

- DOM 顺序与视觉顺序一致，侧栏折叠不改变可达性。
- 路由切换把焦点放到页面主标题；返回恢复到触发元素。
- 命令面板和对话框圈定焦点，`Esc` 关闭并恢复焦点。
- 所有 icon-only window control、tray-equivalent action 和状态图标有名称；状态不只靠颜色。
- 深浅主题各自验证正文 4.5:1、大字号/大图标 3:1；focus ring 在所有 surface 上可见。
- 200% 系统字号下关键命令和设置文字换行，不截断到不可理解。

## 11. Validation Architecture

- Rust unit/integration：配置迁移与损坏、session marker、参数解析、action/keymap、global hotkey adapter、task registry、window bounds 校正和 typed errors。
- Vitest + Testing Library：stores、router、占位、主题、命令面板、录制器、任务反馈、错误与焦点行为。
- 浏览器 Playwright：两种主题、两种密度、`1024x640` 与常见桌面尺寸的视觉/键盘回归；Tauri API 使用 typed test adapter。
- Windows Tauri integration/manual automation：单实例、UAC/权限提示、无边框窗口、托盘、全局热键、`/tray`、异常退出检测和 release 启动性能。
- Negative integration check：生产根目录、manifest、capability、route、action、tray、settings 和 feature snapshots 均不得包含 ImTip 集成。
- 生成契约：每次 Rust command/event 变化后运行 bindings 生成与 `--check`。

## 12. Compatibility, Rollback And Trade-offs

- S1 主验收环境是 Windows 11 x64；Win10 1703、ARM64 和安装器矩阵延后，但不得采用已知不兼容 API 或假定固定 DPI/单显示器。
- 子任务独立提交；配置 schema、action IDs、feature IDs 和 route IDs 一旦被后续子任务消费即视为兼容合同，修改需带迁移或集中适配。
- 若某 Tauri plugin 与锁定版本不兼容，先在所属子任务记录最小复现并评估直接 Win32/Tauri core API；不能以放宽 capability/CSP 作为绕过。
- UI 搜索脚本不可用不影响实现事实来源：设计以 `docs/21-ui-ux.md` 和已加载的 UI/UX 规则为准，视觉验收以实际截图、对比度和键盘测试为准。
