# Design - S0 地基

## 1. Delivery Structure

S0 使用父/子任务树。父任务持有跨子任务合同与最终验收；每个子任务独立规划、实现、检查和归档。依赖顺序为：

```text
docs/spec alignment
  -> workspace/toolchain
  -> codec model
  -> binary/text/aux codecs (parallel-capable)
  -> fixtures + CI
  -> integration

risk spikes run in parallel after workspace/toolchain and gate S1
```

## 2. Workspace Boundary

根 `Cargo.toml` 是 virtual workspace，members 固定为：

- `crates/wubilex-codec`
- `crates/wubilex-core`
- `crates/wubilex-winime`
- `crates/wubilex-resource`
- `src-tauri`
- `xtask`

`crates/wubilex-learn` 保留目录但在 S8 前不加入 members。S0 的业务实现集中在 `wubilex-codec`；其他 member 只建立能支撑统一检查的最小边界，不提前实现后续阶段功能。

## 3. Codec Boundary

`wubilex-codec` 是纯同步、平台无关的字节/文本转换库：

- 输入：受限大小的字节或文本、显式解析选项。
- 输出：强类型码表/短语/辅助数据模型或结构化错误。
- 禁止：Tauri、Win32、网络、全局可变状态、对外部字节做 transmute。
- 可取消和并行化由未来的 `wubilex-app` 编排，不把 Tokio 引入 codec。

解析与写出共享同一模型，但保留格式级模块边界，避免一个巨型解析器同时承担探测、解析、规范化和序列化。

## 4. Validation Strategy

- 单元测试覆盖字段、分支、边界和错误码。
- 往返测试同时区分字节一致与语义一致。
- 属性测试覆盖任意有效模型的 encode/decode 不变量。
- 真实 fixture 由 `xtask fixtures` 获取并用 SHA-256 校验，避免不透明的本机依赖。
- 覆盖率只计算 codec 业务源文件；生成代码和不可达平台适配不用于稀释指标。
- 已知缺陷测试按功能所属阶段落地，不为满足 S0 数字而提前实现 S4 功能。

## 5. Toolchain And CI

Node 版本来自 `package.json.volta.node`，全局 pnpm 的期望版本来自 `package.json.engines.pnpm`，Rust 版本来自 `rust-toolchain.toml`。CI 不重复硬编码版本。日常命令只使用 `cargo`、全局 `pnpm` 与 `cargo xtask`；不用 Volta 的项目级 pnpm pin、corepack、npm、yarn 或 npx。

CI 顺序遵守快速失败原则：格式 -> lint/type -> 单测 -> 覆盖率 -> 依赖审计 -> 生成物/文档一致性。Windows 专属集成测试与不需要真实系统状态的单元测试分离。

## 6. Risk Spike Isolation

TSF、ACL、Task Scheduler 原型只操作隔离测试目标，并记录恢复前后状态；不得替换真实码表或短语文件。虚拟滚动原型使用合成数据，只验证帧率和内存行为，不演化为 S1 页面实现。

每项原型输出：环境、步骤、最小代码、测量结果、判定、清理/恢复说明。任一失败都阻止直接进入 S1，先更新 `docs/02-architecture.md` 的决策或风险处置。

## 7. Rollback Shape

每个子任务独立提交。配置、codec 和原型不混在同一提交中；某个方案失败时只回退对应子任务，不撤销已通过的前序地基。fixture 下载产物必须可重新生成，不把临时系统状态作为交付物。
