# Implementation Plan - S0-04 EUDP 二进制编解码

## 1. Baseline And Failure Tests

- [x] 记录现有 codec 公共 API、依赖树与 workspace 门禁基线。
- [x] 新增公开 API 编译合同和独立手写最小 EUDP 字节测试，先确认目标测试在实现前失败。
- [x] 增加头部、偏移表、记录、UTF-16、deleted、截断与 limits 的失败断言，匹配结构化 kind/location。

## 2. Decoder

- [x] 建立 `eudp` 模块常量和有界小端读取 helper，不使用 ABI 映射、unsafe 或 lossy UTF-16。
- [x] 实现输入限制、64 字节头、signed count/offset、实际文件大小和 empty document 校验。
- [x] 实现偏移表首项/单调性/范围/记录边界验证及末尾哨兵。
- [x] 实现记录头、`cbSize` variant、text offset、candidate、code/text UTF-16、内嵌 NUL、终止符和 code 顺序校验。
- [x] 在完整记录验证后跳过 deleted 条目，保持 active wire 顺序和重复项，并验证声明 wire count 的资源上限。
- [x] 运行 decoder 定向测试并补齐每个失败分支的准确字节偏移。

## 3. Encoder

- [x] 实现条目引用仅按 code 的稳定排序，保持同 code 顺序、candidate 和重复项。
- [x] 实现显式 timestamp 的规范 64 字节头、offset table 预留/回填和 canonical metadata。
- [x] 实现 UTF-16LE code/text + NUL 条目、内嵌 NUL 拒绝、规范 16 字节记录头与全部 checked 长度/offset/count/file-size 转换。
- [x] 通过 empty、未排序输入、同码重复、candidate 1/255、长 code、emoji、换行和时间变量文本精确往返。

## 4. Robustness And Scope Review

- [x] 验证非结构 metadata 容错后只发生 canonical normalization，不改变 PhraseDocument 语义。
- [x] 对已知规范样本的全部截断前缀执行无 panic 错误测试，并覆盖匹配 `phraseEnd` 的记录区截断。
- [x] 确认损坏 deleted 条目不会绕过验证，任一失败都不返回部分 document。
- [x] 确认 production source 无 `unwrap()` / `expect()` / unsafe / transmute，且未引入新依赖。
- [x] 确认未实现系统时钟、文件 I/O、v1/v2 路径、文本方言、`$[...]`、Windows/TSF 或 core 领域操作。
- [x] 确认自动化测试和代码不读取、修改或暂存根目录 `resource/`。

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
- [x] Trellis check 逐条核对 EU-R01..R13 与 Acceptance Criteria。

## 6. Finish

- [x] 用真实 EUDP parser/writer 证据更新 backend 规范，保持 `00-bootstrap-guidelines` 继续进行直至 S0 集成。
- [x] 提交并归档子任务，在父任务中把 `s0-eudp` 标为完成。

## Rollback Points

- decoder 与 encoder 可分别调试，但必须作为同一 wire 合同通过全量检查后提交。
- 若真实样本证明非结构字段影响兼容性，先用可复现证据收紧或放宽对应字段并补回归测试，不猜测格式变体。
- 若需要兼容“损坏条目跳过”，必须先设计显式 warning/partial-result API 并重新评审；不得在当前 `Result<PhraseDocument, CodecError>` 中静默吞错。
- 若 timestamp 的调用方注入方式证明不够用，调整 format API 即可，不向 PhraseDocument 加入非语义头元数据或引入系统时钟。
