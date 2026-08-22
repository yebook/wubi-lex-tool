# S0-01 workspace 与工具链

## Goal

建立固定工具链下可重复安装、编译和静态检查的 Rust、Tauri 与前端最小工程壳，为后续 S0 codec 子任务提供统一入口；本任务不提前交付 S1 的可运行应用。

## Background

- `docs/22-roadmap.md` 将 S0 定义为“可测试的编解码内核 + 工程基础设施”，并明确不产出可运行应用。
- `docs/02-architecture.md` 的 D10、D17 与第 9 节已经定案 workspace 成员、工具链来源和待创建配置；当前仓库只有目录、README 与占位文件，没有可编译配置或源码。
- 开发机已实测 Volta `2.0.2`、Node `24.18.1`、全局 pnpm `11.18.0`、Rust/Cargo `1.97.1`，目标为 `x86_64-pc-windows-msvc`。
- pnpm 11 官方配置合同已核对：项目设置放在 `pnpm-workspace.yaml`，`.npmrc` 只用于认证和 registry；根目录是唯一前端包时可以省略 `packages`。
- 后续 `s0-xtask-ci` 子任务负责真实 `xtask` 子命令、CI、覆盖率和依赖审计闸门；本任务只保证这些能力有可扩展的工程入口。

## Requirements

| ID | Requirement |
|---|---|
| WST-R01 | 根 `Cargo.toml` 必须是 virtual workspace，成员恰好为 `crates/wubilex-codec`、`crates/wubilex-core`、`crates/wubilex-winime`、`crates/wubilex-resource`、`src-tauri`、`xtask`。 |
| WST-R02 | `crates/wubilex-learn` 保留现有目录和 README，但在 S8 前不得有成员身份；本任务不为它创建可编译 crate。 |
| WST-R03 | Rust 版本唯一来源是 `rust-toolchain.toml`：`1.97.1`、`rustfmt`、`clippy`、`x86_64-pc-windows-msvc`；Node 版本来自 `package.json.volta.node = 24.18.1`；pnpm 直接使用全局命令并匹配 `package.json.engines.pnpm = 11.18.0`，不使用 Volta 的项目级 pnpm pin。 |
| WST-R04 | 只使用全局 pnpm；不得引入 `.nvmrc`、`packageManager`、corepack、`volta.pnpm`、`VOLTA_FEATURE_PNPM`、npm、yarn 或 npx 命令。 |
| WST-R05 | 创建 `pnpm-workspace.yaml` 作为 pnpm 11 项目配置入口；根包是唯一前端包，因此不声明 `packages`，也不创建无认证/registry 用途的 `.npmrc`。 |
| WST-R06 | 六个 active Rust member 均可编译；四个库 crate 只有最小 `lib.rs`，`xtask` 只有可编译入口，均不包含 codec、领域、系统集成、资源或自动化业务。 |
| WST-R07 | `src-tauri` 只建立 Tauri v2 的 manifest、build script、合法配置和 compile-only library 壳；不得注册 command、创建窗口或加入产品启动入口。 |
| WST-R08 | 根前端工具链固定兼容版本，建立 Vite 8 + Tailwind CSS v4、TypeScript、ESLint 和 Vitest 配置；无测试时测试命令仍可成功退出。 |
| WST-R09 | 前端壳不得新增 `index.html`、`src/main.tsx`、路由、组件、store、IPC 类型或主题令牌；S1 负责首个可运行、可导航应用。 |
| WST-R10 | `pnpm-lock.yaml` 必须由符合 `engines.pnpm` 的全局 pnpm `11.18.0` 生成并入库，随后 `pnpm install --frozen-lockfile` 不得改写它。 |
| WST-R11 | 本任务的 Rust 与前端本地检查必须通过，且不得通过关闭严格检查、忽略 warning 或静默跳过命令来达成。 |

## Acceptance Criteria

- [ ] `cargo metadata --no-deps` 显示且只显示 WST-R01 的六个 workspace member；`wubilex-learn` 不在其中。
- [ ] `rustc --version`、`cargo --version`、`node --version`、`pnpm --version` 与 WST-R03 一致，配置中没有第二版本来源。
- [ ] `cargo fmt --all -- --check` 通过。
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过。
- [ ] `cargo test --workspace --all-features` 通过。
- [ ] `pnpm install --frozen-lockfile` 通过且 `pnpm-lock.yaml` 无变化。
- [ ] `pnpm typecheck`、`pnpm lint`、`pnpm test --run` 均通过。
- [ ] `vite.config.ts` 使用 `@tailwindcss/vite`，仓库没有 `tailwind.config.*` 或 `postcss.config.*`。
- [ ] 仓库没有新增 `.nvmrc`、`packageManager`、corepack、`volta.pnpm`、`VOLTA_FEATURE_PNPM`、npm、yarn 或 npx 项目口径。
- [ ] 没有新增 Rust command/业务模块，也没有 `src-tauri/src/main.rs`、`index.html`、`src/main.tsx` 或可运行 UI。
- [ ] `git diff --check` 通过，改动仅覆盖本任务的工程配置、最小编译占位、实施期工具链 Trellis 规范/索引、架构口径校正和 Trellis 任务文件。

## Out Of Scope

- S1 的应用启动入口、窗口、导航、主题、组件、状态、IPC command 和占位 UI。
- codec 模型、错误类型、解析器、序列化器或任何其他业务实现。
- `xtask resources`、`fixtures`、`licenses`、`bindings`、`check-docs` 的真实实现。
- GitHub Actions、`cargo-deny`、`cargo-llvm-cov`、覆盖率阈值和生成物新鲜度闸门。
- 安装或修改开发机全局工具、Volta package 身份或 Cargo 扩展。
- `tauri-specta` / `ts-rs` 的兼容性选择；没有 IPC 合同时不提前引入。

## Risks And Deferred Items

- 全局 pnpm 版本是开发环境前置条件；仓库只校验 `engines.pnpm`，不自动安装、卸载或改动用户全局 pnpm，也不添加 Volta 项目 pin。
- TypeScript 最新版 `7.0.2` 超出 `typescript-eslint@8.67.0` 的 `<6.1.0` 范围，因此本任务使用兼容的 TypeScript `6.0.3`，后续升级必须重新验证 peer contract。
- Tauri、Vite 和前端依赖以开工时已核对的兼容版本落锁；版本升级不与本任务混做。
- 完整 Tauri/Vite bundle 与 WebView 启动验证延后到 S1，因为本任务刻意没有应用入口。
