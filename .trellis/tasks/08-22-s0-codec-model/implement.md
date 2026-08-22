# Implementation Plan - S0-02 codec 公共模型

## 1. Baseline

- [x] 记录当前 `wubilex-codec` manifest、最小 `lib.rs`、Cargo lock 中 `thiserror 2.0.20` 和 workspace 检查结果。
- [x] 确认没有已存在的 codec 模型、错误或 limits helper，避免重复 API。

## 2. Error And Limits

- [x] 在 crate manifest 增加唯一直接依赖 `thiserror 2.0.20`。
- [x] 实现 `CodecError`、`CodecErrorKind` 与 `SourceLocation`，支持无位置错误附加 byte/text 位置。
- [x] 实现 `DecodeLimits`、64 MiB / 500,000 默认值及 input/entry 检查 helper。
- [x] 为错误 kind/location 和默认/自定义 limits 编写集成测试。

## 3. Public Models

- [x] 实现 `LexCode`、`Weight`、`LexiconEntry`、`LexiconDocument`，保留 entry 顺序、重复项和可选权重。
- [x] 实现 `PhraseCode`、`Candidate`、`PhraseEntry`、`PhraseDocument`，提供 UTF-16 code unit 派生计数。
- [x] 实现 `LexScheme` 与 `TextEncoding` / `DetectedTextEncoding` 公共合同。
- [x] 用集成测试固定空/边界/非法输入、顺序、重复项、emoji 和 formation 不变量。

## 4. Scope Review

- [x] 检查未创建 lex/eudp/text/detect/escape 等实现模块，未引入 core 索引、serde、Tauri、Windows、Tokio、网络或解析依赖。
- [x] 检查 production source 无 `unwrap()` / `expect()`，公开类型有必要文档且错误测试不匹配展示字符串。
- [x] 检查新 API 不存储可派生的字符数、索引或排序缓存。

## 5. Validation

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p wubilex-codec --all-features
cargo test --workspace --all-features
cargo tree -p wubilex-codec
rg -n 'unwrap\(|expect\(' crates/wubilex-codec/src
git diff --check
git status --short
```

- [x] `wubilex-codec` 和 workspace 门禁全部通过。
- [x] 依赖树只有标准库与 `thiserror` 直接合同，没有越层依赖。
- [x] Trellis check 逐条核对 CM-R01..R12 与 Acceptance Criteria。

## 6. Finish

- [x] 用首批真实 codec 类型更新 backend error/quality/directory 规范示例，保持 `00-bootstrap-guidelines` 继续进行直至 S0 集成。
- [ ] 提交并归档子任务，在父任务中把 `s0-codec-model` 标为完成。

## Rollback Points

- 错误/limits 与模型可分两个实现提交，但归档前必须作为一套公共合同通过全量检查。
- 若类型形状评审失败，只回退本任务新增文件和 codec manifest/lockfile；不得修改已归档 workspace 工具链提交。
- 若真实 fixture 后续证明默认上限不足，使用测量结果单独调整 limits，不绕过或删除资源检查。
