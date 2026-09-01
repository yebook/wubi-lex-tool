# S1 配置与功能目录

## Goal

在现有可启动 Tauri/React runtime 上建立稳定、可迁移且失败不破坏有效数据的 S1 配置基础，并让后端 Cargo feature 通过生成的 IPC 契约成为前端功能可用性的唯一来源。该基础将被后续窗口、主题、路由、快捷键和任务反馈子任务直接复用。

## User Value

- 用户的外观、窗口、关闭行为、引导状态和快捷键配置在重启后保持一致。
- 旧配置、损坏配置或失败导入不会阻止应用启动，也不会覆盖最后一份有效配置。
- 未实现功能的入口由真实构建能力决定，不会出现可点击但实际缺少 command 的假可用状态。
- 用户可以导出配置用于备份或换机，并在完整校验后安全导入。

## Background And Confirmed Facts

- `s1-runtime-lifecycle` 已完成并提供真实 Tauri/React entry、应用状态基线、生成 bindings、日志和 session marker。
- 本任务承接 `M7-CONF-001..007`、架构 D11/D12/D16 和父任务 `AC-S1-05/07/17/18/19`。
- 配置技术已定为自建 `serde` + TOML，落在 WubiLex 应用自有目录，不使用 `tauri-plugin-store` 或 aardio 共享路径。
- Rust 是配置、错误、command/event 和 feature ID 的唯一跨层事实来源；TypeScript 由 `cargo xtask bindings` 生成。
- schema v1 是首个应用自有 TOML，不存在需要兼容的已发布 v0；旧 aardio 用户数据迁移属于 S7，而不是本任务的 schema migration。
- 当前尚未安装 Zustand。本任务是 feature store 的首个真实消费者，因此由本任务引入；后续 UI foundation 只引入剩余 UI 依赖。
- 根目录 `resource/` 不是本任务输入。ImTip 是永久禁止项，不得出现在配置或 feature catalog。

## In Scope

### 1. Versioned Configuration

- 定义 schema v1 的强类型 `window`、`ui` 和 `keymap` 分组，覆盖后续窗口、主题、密度、语言、引导、侧栏和快捷键子任务所需的持久字段。
- 首次运行创建完整默认配置；允许的缺失字段补 schema 默认值，未知字段、非法值和超限输入必须可见失败。
- 建立只接受显式相邻版本步骤的 migration 合同。v1 没有虚构前代；未来每次升级必须随同真实旧版 fixture 和迁移测试。
- 高于当前版本的文件不是损坏文件，必须原位保留；旧程序以只读默认状态继续启动，不能例行覆写未来数据。

### 2. Transactional Persistence And Recovery

- 分组更新、恢复默认和导入均先形成并验证完整候选配置，再原子保存；成功前不得改变内存快照或最后有效文件。
- 每次替换前保留最后一份已验证配置。写入、备份、替换或恢复失败时，内存配置和 revision 不提交，最后有效字节必须在 live 或明确报告的 owned backup 中可恢复。
- 解析、验证或 migration 失败时保留损坏副本并加载默认值；如果无法安全保留源文件，则源文件原位不动并进入只读降级。
- 配置故障不能阻止应用启动；快照必须携带持久化状态和可见 recovery notice。
- 临时文件和清理采用严格资源归属，不能删除其他进程、旧会话或未成功取得所有权的文件。

### 3. Typed IPC And Errors

- 提供配置快照、按完整分组更新、分组/全部恢复默认、导入、导出和 `app_features` command，并进入唯一 bindings registry。
- 每次成功配置变更增加 revision，并通过强类型 `config://changed` 发布完整快照；事件丢失后可重新读取快照恢复。
- 建立首个共享 `AppError` command 合同，保留稳定 code/category、需求模块、中文消息、技术详情和可恢复性。
- 解析、验证、版本、路径、保留、写入、导入和导出失败必须可区分；错误与日志不得包含完整 TOML 或快捷键值。

### 4. Feature Catalog And Store

- 为 S2..S8 的页面、区块和动作消费者冻结稳定、细粒度 feature ID、目标里程碑和不可用原因。
- Cargo feature 是 availability 的唯一来源；catalog 完整且稳定排序，不通过缺失 command 或异常推断能力。
- 前端启动时一次拉取 catalog，Zustand store 明确区分 loading、ready 和 failed，支持失败重试并在 React StrictMode 下去重并发初始化。
- 前端不维护 feature ID union、Vite flag 或 WebView 持久副本。完整 snapshot 替换不能残留旧 feature。
- S1 已实现外壳不伪装为 placeholder；ImTip 不得出现为不可用 feature 或通用“相关工具”占位。

### 5. Import And Export

- 导出生成规范化、带 schema version 的 UTF-8 TOML，仅包含可迁移配置，不含 revision、notice、日志、session marker 或领域用户数据。
- 用户已决定导入采用整份替换。导入文件代表完整配置快照；缺失的可默认字段按导入 schema 补齐，不继承当前值。
- 导入在独立候选状态中完成读取、迁移和验证，失败不得改变文件、内存、revision 或前端状态。
- 成功导入走与普通保存相同的事务并发布一次完整新快照。

## Out Of Scope

- S2 的真实码表、短语、反查数据和领域配置行为。
- S3 的输入法、服务、计划任务、ACL、系统文件写入或真实崩溃恢复动作。
- 窗口 bounds 的显示器校正、托盘关闭流程、主题渲染、路由、动作目录和全局热键注册；本任务只提供后续子任务要消费的配置字段和 feature 状态。
- 通用 schema 编辑器、云同步、配置历史浏览、多用户 profile、加密配置或 secrets 存储。
- 旧版 aardio 用户数据的一次性导入；`NFR-COMPAT-012` 属于后续迁移能力，不等同于本任务的新版配置导入。
- ImTip 的任何入口、feature、配置、命令、URL、进程探测或依赖。
- 读取或转换根目录 `resource/` 中的任何文件。

## Acceptance Criteria

- [x] `AC-CONFIG-01` 首次启动在应用自有配置目录创建规范化 schema v1 TOML，并返回与落盘一致的默认快照。
- [x] `AC-CONFIG-02` 分组更新验证成功后原子保存、增加 revision 并发布一次 `config://changed`；验证或 I/O 任一步失败时内存/revision 不提交，最后有效字节在 live 或 owned backup 中可恢复且恢复结果可见。
- [x] `AC-CONFIG-03` 当前版本、缺失可默认字段、每个受支持旧版本、未知字段、非法 enum/范围和高于当前版本的配置均有确定性测试。
- [x] `AC-CONFIG-04` 解析、验证或 migration 失败时保留损坏副本、加载默认值并返回可见 warning；应用仍可启动。
- [x] `AC-CONFIG-05` 写前备份、独占临时文件、sync、原子替换、错误 1177 后恢复和归属式 cleanup 均有成功及故障注入测试；失败不丢失最后有效配置。
- [x] `AC-CONFIG-06` 导出只包含完整配置 schema；导入复用同一 migration/validation/transaction pipeline 并整份替换，缺失可默认字段不继承当前值；失败 rollback，成功更新 snapshot 和 revision。
- [x] `AC-CONFIG-07` 所有新增 command、event、`AppError` 和 feature 类型均由 Rust registry 生成 bindings，`cargo xtask bindings --check` 通过，前端无手写 wire type。
- [x] `AC-CONFIG-08` `app_features` 由 Cargo feature 事实产生完整且稳定排序的 catalog；前端无 Vite feature flag 或第二份 feature ID 列表。
- [x] `AC-CONFIG-09` Zustand feature store 覆盖 loading、ready、failed、已启用和未启用选择器；snapshot 重取可恢复事件或启动时序缺口。
- [x] `AC-CONFIG-10` 生产根目录、manifests、capabilities、配置 schema 和 feature catalog 对 ImTip 的大小写不敏感搜索为零。
- [x] `AC-CONFIG-11` Rust fmt/check/Clippy/test/doc、bindings/docs、pnpm typecheck/lint/Vitest 和任务上下文验证全部通过。

## Key Decisions And Risks

- 整份导入会覆盖当前所有 S1 配置，这是用户明确选择；后续导入 UI 必须在提交前清楚提示，但 UI 不属于本任务。
- schema v1 一旦被后续子任务或测试用户使用，只能通过新版本和相邻 migration 演进，不能静默改写 v1 含义。
- Windows 原子替换必须同时满足写前备份和失败保留。`ReplaceFileW` 返回错误不代表路径一定未变，特别是 1177；实现必须显式恢复或保留并报告 backup，不能采用先删除有效文件的降级方案。
- future schema 以只读默认状态启动会暂时阻止普通配置保存，这是保护新版本数据的有意取舍；显式、完整且验证通过的导入可解除只读状态。
- frontend 仅建立 feature store，不提前实现配置 UI、route placeholder 或 keymap 业务语义。
