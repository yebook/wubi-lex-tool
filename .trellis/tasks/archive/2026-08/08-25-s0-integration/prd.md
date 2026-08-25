# S0 集成验收

## Goal

以可审计、可重复的证据完成 S0 最终集成验收，关闭父任务 `S0 地基`
的全部标准，并把 Trellis bootstrap 从空模板状态收敛为“已建立合同”或
“有明确边界的待实现证据”，为创建 S1 任务提供可信入口。

## Background

- 父任务定义的十个前置子任务均已归档，最后一项风险预研于 2026-08-25
  完成，四项判定全部通过。
- 当前仓库已具备 Rust workspace、codec、真实 fixtures、xtask、Windows CI、
  生成 bindings、文档校验与前端门禁；2026-08-25 用户撤销“用户级 pnpm”
  决策，要求由 `package.json.volta.pnpm = 11.18.0` 固定项目级 pnpm、删除
  `engines.pnpm`、不添加 `packageManager` 或 Corepack，并重新安装依赖。
- 规划复核时，项目目录中的 `pnpm --version` 为 `11.19.0`，而
  `volta list pnpm` 记录 `11.18.0`；必须由项目 pin 消除这一分歧后重新验收。
- 实施时确认 Volta `2.0.2` 是当前最新正式版；官方文档仍将 pnpm 支持标为
  experimental，并要求 `VOLTA_FEATURE_PNPM=1`。未设置该变量时，
  `volta pin pnpm@11.18.0` 以 `Only node and yarn can be pinned in a project`
  失败且不修改 manifest。
- 用户已将 Windows 用户环境中的 `VOLTA_FEATURE_PNPM` 设置为 `1`；复核确认
  该前置已满足。实施前项目尚未 pin，实际 pnpm 为 `11.23.0`；随后已按本
  任务完成迁移和强制冻结重装。
- 本任务开始时，父任务部分复选框尚未按已交付证据回填，五份尚无对应产品
  实现的 spec 仍保留 `(To be filled by the team)` 模板；当前未提交的集成变更
  已按 S0-R09 用真实证据或明确 pending 边界完成收口，并已同步新的 pnpm 决策
  和完整复验结果。
- 完整同树门禁与独立 Phase 2.2 通过后，唯一剩余问题是缺少七种文本输出的
  aardio/原项目 golden。2026-08-25 用户明确批准“跳过逐字节比对”，将 S0
  出口标准改为现有 canonical 完整字符串、真实 fixture 确定性投影、编码、
  空白转义与缺陷回归证据；本任务不声称 legacy golden 存在。

## Requirements

| ID | Requirement |
|---|---|
| INT-R01 | 审计父任务列出的十个 S0 子任务，确认任务已归档、交付物可定位、结果无未处置 blocker，并形成一份统一证据矩阵。 |
| INT-R02 | 逐条验证父任务 S0-R01..R09 与全部 Acceptance Criteria；结果必须引用当前代码、测试、spec 或归档任务证据，不得仅凭复选框宣称完成。 |
| INT-R03 | 在当前最终树上执行与 CI 一致的完整 Rust、fixture、覆盖率、依赖、bindings、docs 和前端门禁；使用用户确认的项目级 pnpm 入口，默认镜像无 audit endpoint 时仅允许命令级官方 registry 复验。 |
| INT-R04 | 移除 backend/frontend spec 中的空模板占位。已有 S0 实现的合同补真实路径与可执行检查；尚未实现的数据库、日志、组件、Hook 和状态管理只记录已批准边界、明确未选型项及未来更新触发条件，不伪造代码模式。 |
| INT-R05 | 更新 `00-bootstrap-guidelines` 的完成清单和父任务的验收/执行清单，使其与证据一致；任何未通过项必须保持未勾选并阻止归档。 |
| INT-R06 | 复核四项风险预研的最终 PASS 证据和恢复/清理结果；本任务不重新执行 Windows live 或可见 Edge，也不读取根目录 `resource/`。 |
| INT-R07 | 集成验收通过后，按 Trellis 顺序提交集成变更，再归档 `s0-integration`、`00-bootstrap-guidelines` 和父任务 `s0-foundation`，最后记录 journal；失败时不得提前创建 S1 实现任务。 |
| INT-R08 | 在 `VOLTA_FEATURE_PNPM=1` 前置下运行 `volta pin pnpm@11.18.0`，以 `package.json.volta.pnpm` 作为 pnpm 唯一版本源并删除 `engines.pnpm`；不得添加 `packageManager`、Corepack 或另一份 pnpm 版本来源。使用固定版本强制重装依赖，并同步 D17、父任务 S0-R01、CI、workflow contract 测试、Trellis 工具链规范及所有可执行命令。 |
| INT-R09 | 应用 2026-08-25 用户明确批准的 S0 出口标准变更：七种文本输出由 canonical 完整字符串测试、真实 fixture 确定性投影、严格编码、空白转义和缺陷回归提供证据；只移除 aardio/原项目 golden 独立逐字节对照义务，不改变 `.lex` 或 EUDP 字节级要求，也不得声称该 golden 已取得。 |

## Acceptance Criteria

- [x] 十个前置子任务均为 `completed` 且位于 archive，统一证据矩阵覆盖其交付、关键测试和 S0 要求映射。
- [x] 八种真实 `.lex` fixtures 均通过完整性、严格解码、方案识别和字节级重编码；码表文本行为规格、EUDP、短语/辅助格式及既有缺陷回归全部通过。
- [x] `wubilex-codec` 行覆盖率不低于 90%，迁移后的完整 Rust/前端/xtask/依赖/文档门禁全部通过。
- [x] 七种码表文本输出的 canonical 完整字符串测试、真实 fixture 确定性投影、严格编码、空白转义与缺陷回归全部通过；按 2026-08-25 用户决定，不要求 aardio/原项目 golden 独立逐字节对照。
- [x] TSF、ACL、Task Scheduler COM 和 300,000 行虚拟滚动最终报告均为 PASS，且 Windows 恢复/清理证据与 Edge 原始 JSON 可审计。
- [x] Windows 用户环境和 CI 均启用 `VOLTA_FEATURE_PNPM=1`；`package.json.volta.pnpm` 唯一固定 pnpm `11.18.0`，`engines.pnpm` 与 `packageManager` 均不存在；项目目录中的 `pnpm --version` 为 `11.18.0`，CI 只依赖 Volta 项目 pin 准备 Node/pnpm。
- [x] 依赖已通过 `pnpm install --frozen-lockfile --force` 重装，`pnpm-lock.yaml` 未改写，随后 audit、typecheck、lint 和 Vitest 全部通过。
- [x] backend/frontend spec 不再含空模板或旧的 pnpm 版本源口径；每份文件准确标注 established、baseline 或 pending evidence，且未把未来 S1/S2 设计冒充现有惯例。
- [x] `00-bootstrap-guidelines` 三项完成条件有真实 spec/代码证据支撑并可归档。
- [x] 父任务 S0-R01..R09、Acceptance Criteria、Ordered Child Work 和 Global Review Gates 已同步项目级 pnpm 与文本输出出口标准，并按现有可执行证据回填。
- [x] 最终集成报告包含迁移环境、命令、结果、限制与 S1 入口结论；`cargo xtask check-docs`、父/当前任务 Trellis validate 和 `git diff --check` 通过。

## Out Of Scope

- 实现 S1 应用外壳、Tauri 产品命令、数据库、日志框架、Zustand store、正式组件或 Hook 约定。
- 修改 codec、Windows probe 或虚拟滚动阈值来绕过失败验收。
- 重新读取根目录 `resource/`、运行系统变更型 live 探针或重复可见浏览器性能测试。
- 改写已归档子任务中的历史 pnpm 决策，或安装、卸载任何全局包管理器。
- 创建 S1 实现任务；本任务只给出是否允许进入 S1 的集成结论。

## Risks And Deferred Items

- 数据库、应用日志和正式前端模式没有实现证据，必须以明确 pending 状态进入 S1，不能在本任务中选择库或制造示例。
- 本地默认 pnpm 镜像可能不提供 audit endpoint；该情况只说明镜像能力不足，必须使用命令级官方 registry 得到真实审计结论。
- `volta pin` 会写入项目 manifest，并可能补齐 Volta 工具缓存；它不得安装用户级 pnpm，也不得改写 Node pin 或 lockfile。
- `VOLTA_FEATURE_PNPM=1` 会启用 Volta 的实验 pnpm 解析能力，并影响同一用户
  环境下的其他 Volta pnpm 调用；用户已接受并设置该前置，CI 必须显式设置
  同一变量。当前 Codex 父进程启动早于设置动作，实施命令需从用户环境读取
  并注入当前 PowerShell 进程。
- Windows/Edge 结论基于当前开发机环境；跨机器兼容性仍由后续产品集成与 Windows CI 持续验证。
- 仓库和当前开发机均无 aardio 运行时或原项目 golden，且不宣称该证据
  存在。2026-08-25 用户已明确批准跳过该独立逐字节对照，因此它不再阻止
  S0 归档和 S1 入口；现有 canonical 与真实 fixture 测试仍全部保留。
