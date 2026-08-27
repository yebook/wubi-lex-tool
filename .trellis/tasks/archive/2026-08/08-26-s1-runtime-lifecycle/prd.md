# S1 运行时与生命周期

## Goal

把 S0 的编译型 Tauri 脚手架升级为第一个可开发启动、可构建、可见且可诊断的 WubiLex Windows 应用，为后续配置、窗口托盘、路由和动作子任务提供稳定的进程生命周期入口。

## User Value

- 用户可以启动一个真实的 WubiLex 应用，而不是只有编译占位。
- 重复启动不会产生多个主窗口；合法参数会交给已有实例，非法参数会得到可见中文提示。
- 应用能说明当前管理员权限和异常退出状态，为后续安全操作与恢复流程建立可信基础。
- 启动失败和运行异常留下脱敏日志，不要求用户复现后才能定位。

## Confirmed Facts

- 父任务 `.trellis/tasks/08-25-s1-shell-ui` 已获批准并进入 `in_progress`；本子任务是其第一个执行批次。
- 当前 `src-tauri` 只有 library 与空 IPC registry，没有 desktop binary、Wry runtime、command、window 或运行状态；`src/` 没有 React entry，根目录也没有 `index.html`。
- Tauri 版本基线为 `2.11.5`，生成绑定使用唯一的通用 `tauri-specta` registry；运行时必须复用该 registry。
- `M7-INST-001..006`、`NFR-SEC-001/002/009/010/011`、`NFR-REL-003`、`NFR-OBS-001/002` 和父任务合同共同约束本批次。
- 一期采用整进程 `requireAdministrator`；普通权限主进程加提权辅助进程是后续演进，不在本任务内。
- 官方 `tauri-plugin-single-instance 2.4.3` 支持 Tauri 2，并要求在其他插件之前注册。
- `tauri-build 2.6.3` 原生支持自定义 Windows application manifest；自定义清单必须保留 Common Controls v6 依赖。
- `tracing-appender 0.2.5` 支持每日滚动和 `max_log_files`；配合启动时按日期清理可实现默认 7 天日志保留合同。
- Node 和 pnpm 只由 `package.json.volta` 固定；当前 `VOLTA_FEATURE_PNPM=1`，Node `24.18.1`、pnpm `11.18.0`、Cargo `1.97.1` 均匹配项目 pin。

## In Scope

### 1. Runnable Application

- 新增真实 `index.html`、React mount entry、最小运行状态页面和 Vite dev/build scripts。
- 新增 `src-tauri/src/main.rs` desktop entry，同时保留 library entry 供测试和 bindings 生成使用。
- 为 desktop binary 单独启用 Wry；`xtask` 的 mock binding export 不得因为应用可运行而构造窗口或依赖运行时 registry 副本。
- 配置 Tauri build/dev URL、frontend dist、主窗口基线、产品名称、版本、图标、公司和版权元数据。

### 2. Launch Argument Contract

- 纯 Rust parser 接受普通启动、大小写不敏感且只能出现一次的 `/tray`，以及内部 `--navigate <path>` 参数。
- 内部 path 必须以 `/` 开头、长度有界且不包含查询、fragment、反斜杠、父目录段或控制字符；具体 route vocabulary 由 `s1-routing-shell` 冻结。
- `/tray` 与 `--navigate` 可以组合，表示后台启动并保存一个待处理导航目标。
- 重复、缺值、非 Unicode 或未知参数返回结构化 launch notice；应用继续启动并显示警告，不 panic，也不静默忽略。

### 3. Single Instance

- 使用 `tauri-plugin-single-instance` 作为第一个 Tauri plugin；第二实例不得创建第二个产品窗口。
- 第二实例参数必须经过同一个 parser，结果写入主实例的 runtime state，并通过 Rust-owned typed event 通知前端。
- 无论第二实例参数是否合法，已有主窗口都被请求还原、显示并置前；非法参数只产生可见 warning，不影响主实例继续运行。
- 运行时 snapshot command 与 event 都注册到现有唯一 `tauri-specta` builder，并重新生成 TypeScript bindings。

### 4. Privilege And Security

- Windows application manifest 声明 `requireAdministrator`，并保留 Tauri 默认 Common Controls v6 依赖。
- `wubilex-winime` 拥有实际 token elevation 检测；Tauri 层只消费结果，不直接散落 Win32 调用。
- 初始运行状态页显示管理员原因、当前实际权限和权限不足恢复提示；首次只显示一次的持久化行为在配置与反馈子任务完成。
- Tauri capability 只开放当前主窗口实际需要的 IPC/event 权限；CSP 禁止 `eval`、任意 inline script、远程页面和不必要网络源，开发态只增加本地 Vite/HMR 所需来源。

### 5. Logging And Session Marker

- 应用层初始化结构化 `tracing` 日志，输出每日滚动 JSON 文件，默认只保留最近 7 天且最多 7 份；开发态可额外输出到 stderr。
- 日志字段至少覆盖时间、级别、target、事件名、阶段、进程 ID 与版本；不得记录用户输入、码表词条、短语、完整启动原文或 panic payload。
- 在应用数据目录为每个唯一 session ID 创建独占运行标记。启动时发现其他会话标记即报告异常会话，再用 `create_new` 创建当前标记。
- 只有正常 Tauri 退出事件才删除当前会话自己创建的标记；panic、强制终止和启动中止保留标记。
- S1 只显示恢复警告和检查入口，不执行 TSF、服务、ACL、系统文件或注册表恢复。

## Out Of Scope

- 无边框标题栏、窗口 bounds 持久化、关闭行为和托盘生命周期；这些由 `s1-window-tray` 实现。
- TOML 配置、schema migration、首次引导持久化和完整 `AppError` 模型；这些由后续 S1 子任务实现。
- 七领域 route table、route ID 最终集合和导航执行；本任务只传递经过语法验证的内部 path。
- 命令面板、动作目录、快捷键、任务中心、完整错误详情和正式视觉系统。
- S2 数据读取、S3 系统写入或真实恢复动作，以及根目录 `resource/` 的任何读取。
- ImTip 的任何入口、标识、命令、事件、URL、进程探测、capability 或依赖。

## Acceptance Criteria

- [x] `AC-RUNTIME-01` `pnpm run dev` 与 `pnpm run build` 成功；Tauri dev 和 Windows release build 均能创建真实应用窗口并加载 React entry。
- [x] `AC-RUNTIME-02` desktop binary 和 library entry 共存；runtime 与 `cargo xtask bindings` 复用同一 registry，`bindings --check` 通过。
- [x] `AC-RUNTIME-03` 普通、`/tray`、`--navigate`、组合、重复、缺值、未知和非 Unicode 参数的纯测试覆盖成功与失败分支；所有失败均不 panic。
- [x] `AC-RUNTIME-04` 第二实例不创建新窗口，主实例收到解析后的 request/notice，并尝试显示、还原和置前已有窗口。
- [x] `AC-RUNTIME-05` application manifest 同时包含 Common Controls v6 与 `requireAdministrator`；运行时 token 探测结果进入 typed runtime snapshot。
- [x] `AC-RUNTIME-06` 权限不足、未知参数和异常会话在初始 React 页面上呈现可执行中文提示，不只写入日志。
- [x] `AC-RUNTIME-07` 正常退出只删除当前 session marker；预存标记、panic 模拟和强制终止路径保留可检测证据，且不执行任何系统恢复动作。
- [x] `AC-RUNTIME-08` 日志为每日滚动的结构化文件，启动后不存在超过 7 天的自有日志且最多保留 7 份；测试确认敏感字段和原始启动参数不进入记录。
- [x] `AC-RUNTIME-09` production CSP/capability 只允许当前启动壳所需能力，不含 remote URL、shell opener、文件系统、进程启动或后续领域权限。
- [x] `AC-RUNTIME-10` 产品名称、版本、图标、公司、版权和 executable metadata 可从构建产物或 Tauri context 验证。
- [x] `AC-RUNTIME-11` 聚焦 Rust/TypeScript/Vitest 测试、fmt、check、Clippy、Rustdoc、bindings/docs、依赖审计和 Trellis validation 全部通过。
- [x] `AC-RUNTIME-12` 对 `src/`、`src-tauri/`、`crates/`、manifests 与 capability 做大小写不敏感搜索，不存在 ImTip 集成。

## Risks And Deferred Items

- 整进程管理员权限扩大 WebView 攻击面；本任务以严格 CSP、无远程内容和最小 capability 缓解，但不会消除架构风险。
- `/tray` 在本任务可创建隐藏 WebView，但托盘图标和最终无闪现验收属于 `s1-window-tray`；该中间状态仅用于生命周期合同测试。
- 单实例回调可能早于前端 listener 就绪，因此 runtime state 必须保留 snapshot，event 只能作为增量通知而不是唯一事实来源。
- 首次权限说明的“只显示一次”依赖后续配置 schema；本任务先保证说明内容和权限状态可见。
