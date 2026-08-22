# Implementation Plan - S0-01 workspace 与工具链

## 1. Baseline And Version Sources

- [x] 记录 `rustc`、`cargo`、`volta`、`node`、`pnpm` 的实际版本与可执行文件解析结果。
- [x] 确认现有仓库没有根 Cargo/pnpm/Tauri 配置，且待改目录只有 README / `.gitkeep`。
- [x] 搜索 `.nvmrc`、`packageManager`、corepack、npm、yarn、npx，区分历史文档说明与将执行的项目命令。

## 2. Rust Workspace Shell

- [x] 创建根 virtual `Cargo.toml`，显式写入六个 members、resolver `3` 与 edition `2024`；不重复 `rust-toolchain.toml` 的 Rust 版本。
- [x] 创建 `rust-toolchain.toml`，固定 Rust `1.97.1`、minimal profile、rustfmt、clippy 和 MSVC x64 target。
- [x] 为 codec/core/winime/resource 创建最小 `Cargo.toml` 与 `src/lib.rs`，只保留 crate 文档，不创建业务模块。
- [x] 为 `xtask` 创建最小 manifest 与 no-op `src/main.rs`，不提前实现任何子命令。
- [x] 为 `src-tauri` 创建 manifest、`build.rs`、最小合法 `tauri.conf.json`、任务自有的 Windows 编译占位图标与 compile-only `src/lib.rs`。
- [x] 验证没有为 `wubilex-learn` 创建 manifest，也没有新增 `src-tauri/src/main.rs`。

## 3. Frontend Tooling Shell

- [x] 创建私有根 `package.json`，写入 Volta Node pin、全局 pnpm engine 合同和 typecheck/lint/test 脚本；不写 `volta.pnpm`、`packageManager`、dev 或 build 产品脚本。
- [x] 使用核对后的兼容版本声明 React/Tauri 基础依赖与 TypeScript/Vite/Tailwind/ESLint/Vitest 开发依赖。
- [x] 创建不含 `packages` 的 `pnpm-workspace.yaml`；不创建 `.npmrc`。
- [x] 创建严格 `tsconfig.json`、Node config、ESLint flat config 和 `vite.config.ts`，只接入 Tailwind v4 Vite 插件。
- [x] 添加最小非产品 TypeScript 声明输入；不创建 `index.html`、React mount、route、component、store、IPC 类型或主题令牌。
- [x] 用 pnpm `11.18.0` 生成 `pnpm-lock.yaml`，再用 frozen install 验证 lockfile 可复现。

## 4. Validation

```powershell
rustc --version
cargo --version
volta --version
node --version
pnpm --version
Get-Command pnpm

cargo metadata --no-deps --format-version 1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test --run

$package = Get-Content -Raw package.json | ConvertFrom-Json
if ($null -ne $package.volta.pnpm -or $null -ne $package.packageManager) { throw 'project-level pnpm pin found' }
rg -n 'VOLTA_FEATURE_PNPM|corepack|npm (install|run)|yarn |npx ' package.json pnpm-workspace.yaml vite.config.ts tsconfig*.json eslint.config.js
Get-ChildItem -Force . -Filter '.nvmrc'
Get-ChildItem -Force . -Filter 'tailwind.config.*'
Get-ChildItem -Force . -Filter 'postcss.config.*'
Test-Path 'crates/wubilex-learn/Cargo.toml'
Test-Path 'src-tauri/src/main.rs'
Test-Path 'index.html'
Test-Path 'src/main.tsx'
git diff --check
git status --short
```

- [x] Cargo metadata 的 member 集合与 WST-R01 完全相等。
- [x] Rust fmt、Clippy、测试全部通过，Clippy warnings 被拒绝。
- [x] frozen install 不改写 lockfile，TypeScript、ESLint、Vitest 全部通过。
- [x] 禁止项搜索无项目配置命中，四个推迟文件的 `Test-Path` 均为 `False`。
- [x] diff 不包含业务逻辑、CI、真实 xtask 子命令或全局环境修改；只额外包含实施期工具链 Trellis 规范/索引与架构口径校正。

## 5. Review And Rollback

- [x] 按 WST-R01..R11 与每条 Acceptance Criteria 做逐项复核。
- [x] 检查新增 manifest/config 的版本来源只有 `package.json.volta.node`、`package.json.engines.pnpm` 与 `rust-toolchain.toml`。
- [x] 同步工具链七节规范、guides 索引、前端质量规范和 `docs/02-architecture.md` D17/R55 口径。
- [x] 检查新增最小源码没有形成后续任务必须继承的虚假业务模式。
- [x] 任一工具链不兼容时，只调整本任务新增配置和 lockfile；不通过降低 lint/test 强度绕过。
- [x] Trellis check 已通过，无遗留代码或规范缺陷。
- [x] 提交并归档子任务，再在父任务中把 `s0-workspace-toolchain` 标为完成。
