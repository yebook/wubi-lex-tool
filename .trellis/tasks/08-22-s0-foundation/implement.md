# Implementation Plan - S0 地基

## Ordered Child Work

- [x] 0. 完成并归档 `s0-docs-spec-alignment`。
- [x] 1. 创建并完成 `s0-workspace-toolchain`；验证 D10/D17、全局 pnpm 命令与最小 Rust/Tauri/前端检查链。
- [x] 2. 创建并完成 `s0-codec-model`；冻结核心模型、错误和输入限制后再拆格式实现。
- [ ] 3. 创建 `s0-lex-binary`、`s0-eudp`、`s0-lex-text`、`s0-phrase-aux`；依赖模型任务，格式间不得通过内部实现细节耦合。
- [ ] 4. 创建并完成 `s0-fixtures-regressions`；补真实样本、属性测试、损坏输入和阶段归属正确的缺陷回归。
- [ ] 5. 创建并完成 `s0-xtask-ci`；把本地已通过的检查固化到 CI。
- [ ] 6. 在步骤 1 后并行推进 `s0-risk-spikes`，结果作为 S1 入口门槛。
- [ ] 7. 完成 `s0-integration`；全量验证、真实示例回填规范、关闭 bootstrap、集成评审。

## Global Review Gates

- [ ] G1：任何代码前，当前子任务状态必须为 `in_progress` 且已加载相关 Trellis 规范。
- [ ] G2：公共模型冻结前，不并行实现多个格式，避免重复类型和错误模型漂移。
- [ ] G3：每个格式子任务必须先有失败测试或 fixture 断言，再实现通过。
- [ ] G4：CI 只接入已经能在本地稳定运行的命令。
- [ ] G5：技术预研未全部通过或完成架构复评，不创建 S1 实现任务。

## Final Validation

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 90
cargo deny check
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test --run
cargo xtask bindings --check
cargo xtask check-docs
python ./.trellis/scripts/check_anchors.py
git status --short
```

具体命令名可在 workspace 子任务中按实际工具能力收敛，但不得降低 PRD 的验收指标。

## Rollback Points

- workspace/toolchain、各 codec 格式、fixture/CI、四项预研分别提交。
- fixture 获取失败时保留 manifest 与校验逻辑，不提交不完整或来源不明的二进制。
- Windows 原型失败时先恢复测试状态并记录结果，再讨论替代设计。
