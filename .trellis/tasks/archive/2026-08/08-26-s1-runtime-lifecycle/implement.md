# Implementation Plan - S1 运行时与生命周期

## Entry Gate

- [x] 用户在本子任务最终规划摘要之后明确批准开始实施。
- [x] 获批后运行 `python ./.trellis/scripts/task.py start .trellis/tasks/08-26-s1-runtime-lifecycle`。
- [x] 实施前再次确认 `VOLTA_FEATURE_PNPM=1` 以及 Node、pnpm、Cargo 项目 pin。
- [x] 保护现有父任务文档改动，不读取根目录 `resource/`。

## Ordered Work

- [x] 1. Runtime dependency and feature wiring
  - 精确固定经审计的 single-instance、serde、tracing、subscriber、appender 和测试依赖。
  - 为 Wry desktop binary 与 mock bindings export 建立清晰 feature 边界。
  - 更新 Cargo lockfile，验证 cargo-deny 和 workspace metadata。

- [x] 2. Launch parser
  - 实现纯 `OsString` parser、`LaunchRequest`、稳定 notice codes 和内部 path validator。
  - 覆盖 normal、`/tray`、navigate、组合、重复、缺值、未知、非 Unicode 与长度边界。

- [x] 3. Native privilege and manifest
  - 在 `wubilex-winime` 实现当前进程 token elevation probe及可注入 adapter tests。
  - 新增保留 Common Controls v6 的 `requireAdministrator` manifest，并由 `build.rs` 必须成功嵌入。
  - 配置产品名称、图标、版本、公司与版权 metadata。

- [x] 4. Logging and session evidence
  - 建立每日 JSON tracing writer、7 天/7 文件双重清理、development stderr 和 worker guard 生命周期。
  - 安装不记录 payload 的 panic hook。
  - 实现注入式 session marker service、会话独占文件、精确所有权清理和异常状态 snapshot。

- [x] 5. Tauri runtime composition
  - 新增 desktop `main.rs` 与 library-owned `run()` / builder composition。
  - 首先注册 single-instance plugin，建立 runtime managed state、typed snapshot command/event 和窗口置前请求。
  - 复用唯一 specta registry，生成并校验 TypeScript bindings。

- [x] 6. Minimal React bootstrap
  - 新增 `index.html`、`src/main.tsx`、最小 status view 和外部 CSS。
  - 使用 generated IPC contract 展示 elevation、异常会话与启动警告；不建立最终设计系统或 route/store 假约定。
  - 添加 Vite/Tauri dev/build scripts 与聚焦 Vitest 测试。

- [x] 7. Security and Windows smoke
  - 增加最小 main capability、严格 production CSP 和本地-only development CSP。
  - 验证普通/隐藏启动、第二实例、无效参数、强制终止恢复提示与干净退出。
  - 运行生产根、manifest 和 capability 的禁止集成搜索。

## Validation Commands

```powershell
$env:VOLTA_FEATURE_PNPM = [Environment]::GetEnvironmentVariable('VOLTA_FEATURE_PNPM', 'User')
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ($env:VOLTA_FEATURE_PNPM -ne '1') { throw 'VOLTA_FEATURE_PNPM must be 1' }
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p wubilex-winime --locked
cargo test -p wubilex-app --all-features --locked
$env:RUSTDOCFLAGS='-D warnings'; cargo doc -p wubilex-app -p wubilex-winime --all-features --no-deps --locked
cargo xtask bindings --check
cargo xtask check-docs
cargo deny check

pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run build

python ./.trellis/scripts/task.py validate .trellis/tasks/08-26-s1-runtime-lifecycle
git diff --check
```

Windows smoke commands will be committed as a reusable script or package command before final acceptance; temporary manual steps do not replace the checked-in gate.

## Review Gates

- [x] Single-instance is registered first and no callback duplicates parser or routing logic.
- [x] Runtime snapshot remains authoritative when events arrive before frontend subscription.
- [x] Session cleanup deletes only the current session's unique marker and cannot erase stale evidence from another session.
- [x] Logging never records full argv, navigation target, panic payload, user text, lexicon or phrase data.
- [x] Manifest elevation and runtime token detection are independently tested.
- [x] Wry feature wiring does not break mock binding generation or `xtask`.
- [x] Capability and CSP contain no broad fallback permissions or remote content.
- [x] Product roots contain no ImTip integration or generic companion-tool substitute.

## Rollback Points

- Dependency/feature wiring, launch parser, privilege/manifest, logging/session, Tauri runtime and frontend bootstrap remain separable review units.
- A plugin incompatibility rolls back the plugin integration and its dependency, not the security boundary.
- A session marker format change before downstream consumption can be replaced directly; after consumption it requires schema migration.
- A Windows smoke failure blocks the child task even when mock/unit tests pass.
