# Implementation Plan - S0-03 .lex 二进制编解码

## 1. Baseline And Failure Tests

- [x] 记录现有 codec 公共 API、依赖树与 workspace 门禁基线。
- [x] 新增公开 API 编译合同和手写最小 `.lex` 字节测试，先确认目标测试在实现前失败。
- [x] 增加头部、索引、记录、UTF-16、截断与 limits 的失败断言，匹配结构化 kind/location。

## 2. Decoder

- [x] 建立 `lex` 模块常量和有界小端读取 helper，不使用 ABI 映射或 unsafe。
- [x] 实现输入限制、头部魔数、结构偏移和实际文件大小校验。
- [x] 实现记录解析、严格 UTF-16、模型构造和逐条展开上限检查。
- [x] 实现编码顺序及 26 项字母索引的记录边界/分区一致性检查。
- [x] 运行 decoder 定向测试并补齐每个失败分支的准确字节偏移。

## 3. Encoder

- [x] 实现条目引用的稳定编码排序和同码顺序/重复项保留。
- [x] 实现显式/缺省权重状态机、UTF-16 记录和全部 checked 长度转换。
- [x] 实现规范 1.1 头部、字母索引回填和确定性字节输出。
- [x] 通过空文档、索引空洞、非 BMP、重复项、未排序输入与权重边界往返测试。

## 4. Robustness And Scope Review

- [x] 对已知规范样本的全部截断前缀执行无 panic 错误测试。
- [x] 确认 production source 无 `unwrap()` / `expect()` / unsafe / transmute，且未引入新依赖。
- [x] 确认未实现文件 I/O、压缩、文本格式、方案探测或 core 领域操作。
- [x] 只读验证 `resource/微软五笔86( 完整 ).lex` 的记录数和 decode/encode SHA-256，一律不修改、不暂存该目录。

## 5. Validation

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo test -p wubilex-codec --all-features
cargo test --workspace --all-features
cargo doc -p wubilex-codec --all-features --no-deps
cargo tree -p wubilex-codec
rg -n 'unwrap\(|expect\(|unsafe|transmute' crates/wubilex-codec/src
git diff --check
git status --short
```

- [x] `wubilex-codec` 定向测试和 workspace 全量门禁通过。
- [x] Rustdoc 在 warnings 与 missing docs denied 条件下通过。
- [x] 依赖树仍只有标准库与直接依赖 `thiserror 2.0.20`。
- [x] Trellis check 逐条核对 LB-R01..R12 与 Acceptance Criteria。

## 6. Finish

- [x] 用真实 `.lex` parser/writer 证据更新 backend 规范，保持 `00-bootstrap-guidelines` 继续进行直至 S0 集成。
- [ ] 提交并归档子任务，在父任务中把 `s0-lex-binary` 标为完成。

## Rollback Points

- decoder 与 encoder 可分别调试，但必须作为同一线格式合同通过全量检查后提交。
- 若容错策略证明会接受歧义布局，收紧对应结构字段并补回归测试，不用 lossy 或 partial-success 绕过。
- 若真实样本不能字节往返，先定位规范化差异并更新设计评审；不得修改或提交用户样本来掩盖差异。
