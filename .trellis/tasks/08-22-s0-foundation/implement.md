# Implementation Plan - S0 地基

## Ordered Child Work

- [x] 0. 完成并归档 `s0-docs-spec-alignment`。
- [x] 1. 创建并完成 `s0-workspace-toolchain`；验证 D10/D17、Volta 项目级 Node/pnpm pin 与最小 Rust/Tauri/前端检查链。
- [x] 2. 创建并完成 `s0-codec-model`；冻结核心模型、错误和输入限制后再拆格式实现。
- [x] 3. 创建 `s0-lex-binary`、`s0-eudp`、`s0-lex-text`、`s0-phrase-aux`；依赖模型任务，格式间不得通过内部实现细节耦合。
  - [x] `s0-lex-binary`
  - [x] `s0-eudp`
  - [x] `s0-lex-text`
  - [x] `s0-phrase-aux`
- [x] 4. 创建并完成 `s0-fixtures-regressions`；补真实样本、属性测试、损坏输入和阶段归属正确的缺陷回归。
- [x] 5. 创建并完成 `s0-xtask-ci`；把本地已通过的检查固化到 CI。
- [x] 6. 在步骤 1 后并行推进 `s0-risk-spikes`，结果作为 S1 入口门槛。
- [ ] 7. 完成 `s0-integration`；全量验证与集成评审已通过，待按顺序提交并归档当前任务、bootstrap 与父任务。

## Global Review Gates

- [x] G1：任何代码前，当前子任务状态必须为 `in_progress` 且已加载相关 Trellis 规范。
- [x] G2：公共模型冻结前，不并行实现多个格式，避免重复类型和错误模型漂移。
- [x] G3：每个格式子任务必须先有失败测试或 fixture 断言，再实现通过。
- [x] G4：CI 只接入已经能在本地稳定运行的命令。
- [x] G5：技术预研未全部通过或完成架构复评，不创建 S1 实现任务。

## Final Validation

```powershell
$env:VOLTA_FEATURE_PNPM = [Environment]::GetEnvironmentVariable('VOLTA_FEATURE_PNPM', 'User')
if ($env:VOLTA_FEATURE_PNPM -ne '1') { throw 'VOLTA_FEATURE_PNPM must be 1' }
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
$env:RUSTDOCFLAGS='-D warnings'; cargo doc --workspace --all-features --no-deps --locked
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov clean --workspace
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov --package wubilex-codec --all-features --summary-only --fail-under-lines 90
& .\target\tools\cargo-deny-0.20.2\bin\cargo-deny.exe check
cargo xtask fixtures --check
cargo xtask bindings --check
cargo xtask check-docs
& .\target\tools\actionlint-1.7.7\bin\actionlint.exe .github/workflows/ci.yml
pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run typecheck
pnpm run lint
pnpm run test --run
python ./.trellis/scripts/task.py validate .trellis/tasks/08-22-s0-foundation
python ./.trellis/scripts/task.py validate .trellis/tasks/08-25-s0-integration
git status --short
```

具体命令名可在 workspace 子任务中按实际工具能力收敛，但不得降低 PRD 的验收指标。

## Rollback Points

- workspace/toolchain、各 codec 格式、fixture/CI、四项预研分别提交。
- fixture 获取失败时保留 manifest 与校验逻辑，不提交不完整或来源不明的二进制。
- Windows 原型失败时先恢复测试状态并记录结果，再讨论替代设计。
