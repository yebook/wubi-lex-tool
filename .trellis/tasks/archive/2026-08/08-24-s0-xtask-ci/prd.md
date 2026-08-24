# S0 xtask 与 CI 门禁

## Goal

把已经在本地稳定运行的 S0 Rust、codec fixture/覆盖率、文档、IPC 绑定和前端检查固化成可复现、可快速失败的 GitHub Actions 门禁，使 pull request 与 `main` 推送无法绕过编译、测试、覆盖率、依赖安全和生成物新鲜度合同。

## Background

- 父任务 `S0-R07` 与 `NFR-MAINT-006` 要求 CI 覆盖 fmt、Clippy、测试、覆盖率、依赖审计、bindings/docs 和前端静态检查。
- 仓库当前没有 `.github/workflows/`。`xtask` 已有经过独立复核的 `fixtures [--check]`，但没有 `bindings` 或 `check-docs`。
- 八份真实 fixture 已由 manifest 固定并在本地通过；codec 130 项测试的实测行覆盖率为 90.12%，`cargo-llvm-cov 0.9.0 --fail-under-lines 90` 已通过。
- `src-tauri` 仍是无 command 的编译壳，`src/types/generated/` 仍为空。`tauri-specta 2.0.0-rc.25` 官方合同支持 Tauri 2、空 command/event 集和泛型 runtime，因此可建立真实空绑定基线而不制造假 command。
- 当前需求计数为模块 414、NFR 101、UX 115、总计 630；内部锚点检查为 0 处失效。
- Node、全局 pnpm 与 Rust 的唯一版本来源分别是 `package.json.volta.node`、`package.json.engines.pnpm` 和 `rust-toolchain.toml`。CI 不复制这些版本。

## Requirements

| ID | Requirement |
|---|---|
| CI-R01 | 扩展 `cargo xtask` 为严格命令面：`fixtures [--check]`、`bindings [--check]`、`check-docs`；未知命令、重复/多余参数和非 Unicode 参数必须失败并显示完整 usage。 |
| CI-R02 | `cargo xtask bindings` 必须从 `src-tauri/src/bindings/` 的 Rust 单一注册表生成提交到 `src/types/generated/` 的 TypeScript；`--check` 必须非修改地比较规范输出并在过期时失败。 |
| CI-R03 | IPC 空基线必须由兼容的 `tauri-specta` 真实导出产生。收集器对 `tauri::Runtime` 泛型，检查路径使用 mock runtime，不启用 `wry`、不注册虚假 command；生成文件固定 LF，禁止手改。 |
| CI-R04 | `cargo xtask check-docs` 必须从仓库根执行同一套跨平台校验：定义 ID 语法/唯一性、414/101/115/630 计数、悬空引用、`TBD`/`待补充` 占位符，并复用已验证的 anchor 检查；一次运行应尽量汇总所有问题。 |
| CI-R05 | 新增一个 Windows GitHub Actions quality workflow，触发 `pull_request`、`main` push 和手动运行；使用最小 `contents: read` 权限与同分支并发取消，不执行发布、签名或系统写入。 |
| CI-R06 | CI 必须从仓库字段读取 Node/pnpm/Rust 版本。pnpm 由 `pnpm/action-setup` 按 `engines.pnpm` 准备为全局命令；不得新增 `volta.pnpm`、corepack、`packageManager`、npm/yarn/npx 命令或第二份工具链版本。 |
| CI-R07 | 第三方 GitHub Actions 必须固定到审核过的完整 commit SHA，并用注释保留对应 release；Cargo 辅助工具版本必须精确固定，且本地验证只能安装到忽略的 `target/tools`，不修改开发机全局 Cargo 工具。 |
| CI-R08 | CI 按快速失败顺序执行 Rust fmt、check、严格 Clippy、fixture 准备/离线确认、workspace tests、warnings-denied Rustdoc、codec 90% 行覆盖率、Cargo 依赖审计、bindings/docs 新鲜度，再执行 frozen pnpm install、前端漏洞审计、typecheck、lint 和 Vitest。 |
| CI-R09 | fixture 缓存键必须包含 runner OS 与 manifest 摘要；Cargo/pnpm 缓存键必须来自对应 lockfile。缓存只能提速，命中后仍须执行完整校验，缓存缺失不得造成测试静默跳过。 |
| CI-R10 | 新增 `deny.toml` 并用 `cargo-deny 0.20.2` 同时执行 advisories、licenses、bans 与 sources 检查；只允许当前依赖树实际需要的兼容许可证，例外必须精确到 crate/version 并写原因。 |
| CI-R11 | 前端漏洞门禁使用全局 `pnpm audit --audit-level high`。本机镜像若无 audit endpoint，应报告环境原因并用命令级官方 registry 做本地诊断，不创建或修改仓库 `.npmrc`。干净 CI runner 使用默认官方 registry。 |
| CI-R12 | workflow 中的命令必须与本地已通过命令一致；任何检查失败都必须使 job 非零退出，不使用 `continue-on-error`、静默 fallback、空成功或弱化 warning/覆盖率阈值。 |
| CI-R13 | 本任务不得实现 `xtask resources`、`xtask licenses`、产品资源下载、Tauri 业务 command、Windows 系统集成、发布/上传/签名或 S1 UI。 |
| CI-R14 | 更新 backend/frontend 工具链与质量规范，记录 bindings、docs、审计、缓存和 CI 失败合同；父任务只关闭 `s0-xtask-ci`，风险预研与 S0 integration 继续开放。 |

## Acceptance Criteria

- [x] `cargo xtask bindings` 生成已提交的真实空 TypeScript 基线；连续生成字节一致，`bindings --check` 通过，篡改产物的回归测试会失败且不改写文件。
- [x] `cargo xtask check-docs` 对当前仓库输出 414/101/115/630、0 个悬空 ID、0 个真实占位符和 0 个失效锚点；针对重复定义、计数漂移、悬空引用、占位符与 anchor 子进程失败的测试均能阻止成功。
- [x] `cargo xtask fixtures` 与 `cargo xtask fixtures --check` 在八方案缓存上通过；真实 fixture 不入 Git，根 `resource/` 未被读取。
- [x] `cargo fmt --all -- --check`、workspace Cargo check、严格 Clippy、全量 tests 与 warnings-denied Rustdoc 通过。
- [x] 精确 `cargo-llvm-cov 0.9.0` 对 `wubilex-codec` 的 `--fail-under-lines 90` 门槛通过，数值不低于已验证的 90.12% 基线。
- [x] 精确 `cargo-deny 0.20.2 check` 通过 advisories/licenses/bans/sources；未以宽泛 skip 或全局 allow 掩盖问题。
- [x] 全局 pnpm 11.18.0 的 frozen install、`audit --audit-level high`、typecheck、lint、test 均通过，`pnpm-lock.yaml` 未改写。
- [x] `.github/workflows/ci.yml` 通过语法/静态审查，所有 action 固定完整 SHA，版本读取、权限、并发、缓存键、步骤顺序和失败传播符合 CI-R05..R12。
- [x] `cargo xtask bindings --check`、`cargo xtask check-docs`、Trellis validate、anchor 检查、forbidden-pattern scan 与 `git diff --check` 全部通过。
- [x] 独立 Trellis check 逐项复核 CI-R01..R14 与所有验收标准，并直接修复发现后重跑受影响门禁。

## Out Of Scope

- `xtask resources`、`xtask licenses`、许可页面生成和产品资源 manifest。
- S1 的应用入口、真实 IPC command/event、前端消费者或 Tauri/Wry 窗口运行。
- Windows IME/TSF/ACL/Task Scheduler 的真实或 dry-run 集成测试；这些属于 `s0-risk-spikes` 和后续阶段。
- 发布构建、安装包、产物上传、代码签名、自动更新、Dependabot 和 release workflow。
- 独立 aardio golden 生成与 S0 最终集成结论。

## Risks And Deferred Items

- 上游 fixture 网络可能短暂失败；manifest-keyed cache 降低重复下载，但空缓存 runner 仍必须显式失败，不能改为跳过。
- 本机 `npmmirror` 没有 npm audit endpoint；这是环境诊断，不是仓库配置理由。官方 registry 基线已验证为无已知漏洞。
- Cargo advisory 数据库会随时间变化；新增 advisory 必须审查并修依赖，不能预先加入宽泛 ignore。
- 绑定基线目前为空，但生成器和 freshness gate 必须真实工作。S1 新增 command/event 时扩展同一泛型收集器。
- GitHub-hosted runner 与外部 action 会演进；完整 SHA 固定保证可复现，升级留给后续独立维护。
