# Implementation Plan - S1 配置与功能目录

## Entry Gate

- [x] 用户在本轮最终规划摘要之后明确批准进入实施。
- [x] 仅在获批后运行 `python ./.trellis/scripts/task.py start .trellis/tasks/08-27-s1-config-features`。
- [x] 实施前使用 `trellis-before-dev` 完整加载 backend/frontend 目标规范和跨层指南。
- [x] 确认工作树只包含本任务规划改动；保护其他窗口和用户改动。
- [x] 设置进程内 `VOLTA_FEATURE_PNPM=1`，确认 Node/pnpm 与 `package.json.volta` 一致；不增加其他版本源。
- [x] 不读取、修改或测试根目录 `resource/`；生产目录不得出现 ImTip。

## Work Checklist

- [x] 1. 冻结依赖和公共类型
  - 精确增加 Rust `toml 1.1.4`、测试用 `tempfile 3.27.0` 及 Zustand `5.0.15`；为既有 `windows 0.61.3` 开启存储 API feature，不增加第二套 Windows bindings，并更新两个 lockfile。
  - 定义 schema v1、defaults、validation、`AppError`、config snapshot/notice/event 和 feature catalog 类型。
  - 验收：所有跨层类型可由 `specta` 导出，无手写 TypeScript wire type。

- [x] 2. 实现 bounded TOML codec 和 migration registry
  - 先读 `schemaVersion`，再执行相邻 migration，最后反序列化当前模型。
  - 实现 1 MiB、unknown field、enum/range、keymap 数量/字符串限制和规范化输出。
  - schema v1 不伪造 v0；测试 missing/zero/future 和空 transition registry。
  - 验收：canonical roundtrip、默认字段、非法输入和确定性序列化测试通过。

- [x] 3. 实现 Windows 持久化端口与 `ConfigService`
  - 在 `wubilex-winime` 封装同目录独占 staging、no-clobber/write-through 初次安装和 `ReplaceFileW(flags=0)`；在 app config 层实现 unique backup、corrupt 保留和归属式 cleanup。
  - 实现 missing/live-backup/valid/corrupt/future/I/O 启动状态机、只读降级、revision 和串行事务。
  - 对 native 1177 的 old-target-at-backup 拓扑执行显式 no-clobber restore；恢复失败时保留 backup 和组合错误证据。
  - 用真实临时目录和 stage-failing wrapper 覆盖每个故障点。
  - 验收：任何失败都不丢失最后有效字节，非所有者不删除 temp/backup，future 文件原位保留。

- [x] 4. 实现整份导入、导出和分组更新
  - 分组更新/恢复默认先形成完整候选，再走同一事务。
  - 导入 parse/migrate/default/validate 成功后整份替换，不与当前值合并。
  - 导出仅包含 canonical `AppConfig`，拒绝 config-owned alias destination。
  - 验收：失败 rollback 的文件、snapshot、revision 完全相同；成功只增加一次 revision。

- [x] 5. 接入 Tauri runtime、commands、events 与 bindings
  - setup 中解析 app config path 并 manage service；失败以只读 defaults 继续启动。
  - I/O command 使用 `spawn_blocking`；成功提交后 emit 完整 `config://changed` snapshot。
  - 注册全部 config commands、`app_features` 和 event 到唯一 binding registry。
  - 验收：event 丢失可由 snapshot 恢复，emit 失败不回滚已提交文件，bindings freshness 通过。

- [x] 6. 建立稳定 Cargo feature catalog
  - 声明 12 个未来能力 Cargo features，默认 S1 build 全部关闭。
  - const catalog 固定 ID、顺序、里程碑和 typed unavailable reason。
  - 验收：cfg 与 availability 一一对应；无 raw Cargo name 泄漏、无 ImTip/通用相关工具 entry。

- [x] 7. 实现首个 Zustand store
  - 建立 injected feature client、store factory、typed selectors 和 production store。
  - 实现 loading/ready/failed、in-flight 去重、失败重试和 full snapshot replace；不使用 persistence/devtools。
  - 在当前 frontend bootstrap 触发初始化，不添加最终路由或 placeholder UI。
  - 验收：并发初始化只调用一次 command，retry 和 stale-entry removal 测试通过。

- [x] 8. 聚焦与跨层测试
  - Rust 覆盖 codec、migration、storage、service、commands projection、feature catalog 和 serialization。
  - Vitest 覆盖 client/store 状态、selector、StrictMode 时序和失败恢复。
  - 更新 generated bindings，并检查前后端字段、command result 和 event name 一致。

- [x] 9. 规范回填和最终审查
  - 用真实实现更新 backend directory/error/quality 和 frontend state/type/quality/index 证据。
  - 运行 `trellis-check`，修复全部已验证问题后重跑受影响门禁。
  - 对生产根目录复查 ImTip、Vite flags、browser persistence、package manager 和非原子 fallback。

## Validation Commands

```powershell
$env:VOLTA_FEATURE_PNPM = '1'
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --all-features --no-deps --locked
cargo xtask bindings --check
cargo xtask check-docs
cargo deny check

pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run build

python ./.trellis/scripts/task.py validate .trellis/tasks/08-27-s1-config-features
git diff --check
```

验证说明：仓库正式 CI 的测试命令为 `cargo test --workspace --all-features`，本任务以附加 `--locked` 的等价命令通过。额外执行 `cargo test --workspace --all-targets --all-features --locked` 时，47 个 app library tests 先全部通过，随后非提升终端在启动带 `requireAdministrator` 的桌面 bin 时由 Windows 返回 740；完整 `--lib --tests --examples` 套件已通过。依赖图最终冻结后，官方 registry audit 返回零已知漏洞；独立检查后的改动未修改 manifest 或 lockfile。

## Review Gates

- [x] `AC-CONFIG-01..11` 均有实现和测试证据并在 PRD 勾选。
- [x] 整份导入已验证缺失字段取导入 schema 默认值，不继承当前配置。
- [x] future schema 原文件保持不动，普通保存不会解除只读保护。
- [x] 所有磁盘故障点保持内存/revision 不提交，最后有效字节位于 live 或 owned backup；1177 恢复分支已验证，无 delete-live-then-rename。
- [x] command/event/Error/feature 类型全部来自 Rust registry，bindings 无漂移。
- [x] feature store 无 Vite flag、localStorage/sessionStorage 或手写 ID union。
- [x] S2/S3 行为、旧 aardio 数据迁移、resource 读取和 ImTip 均未进入实现。
- [x] 独立 `trellis-check` 无未解决的 verified finding。

## Rollback Points

- schema/model/codec、storage/service、IPC/catalog、frontend store 分成可独立审查的逻辑批次。
- Windows 原子替换无法覆盖普通失败与 1177 backup/restore 合同时停止并回到设计，不用非原子降级绕过。
- bindings 或 Zustand bootstrap 回归时回退所属批次，不添加前端第二 feature source。
- schema v1 被后续消费者采用后只允许新增版本和相邻 migration，不静默改写 v1 含义。
