# Design - S0-02 codec 公共模型

## 1. Boundary

本任务只建立 `wubilex-codec` 的格式合同层：格式模块把外部字节或文本解析为保序文档模型，再从同一模型写回。`wubilex-core` 后续把文档转换为适合编辑、索引和变换的领域模型；codec 不持有倒排索引或应用 session。

```text
external bytes/text
  -> future format parser
  -> codec document model (this task)
  -> future core indexed model

CodecError + DecodeLimits are shared by every future format parser.
```

## 2. Module Layout

```text
crates/wubilex-codec/src/
  lib.rs
  error.rs
  limits.rs
  model/
    mod.rs
    lexicon.rs
    phrase.rs
    scheme.rs
    text_encoding.rs
```

`lib.rs` 只公开稳定模块与必要 re-export。格式子目录仍由后续任务创建，不能把占位 parser API 放进本任务。

## 3. Document Models

`LexiconDocument` 与 `PhraseDocument` 各自拥有 `Vec<Entry>`。选择 entry stream 而不是 `HashMap` / `BTreeMap`，原因是：

- 二进制往返和文本兼容需要保留源顺序与重复记录。
- 同一编码下候选顺序有业务语义，map 会诱导隐式去重或重排。
- 索引是 core 的派生状态，不应污染格式合同。

`LexCode` 是 1..=4 字符的小写 ASCII 新类型。`PhraseCode` 是非空小写 ASCII 新类型，不人为套用 `.lex` 的四码硬上限；EUDP 编码器以后按自身 offset 宽度检查。`Weight` 和 `Candidate` 使用非零整数语义。文档允许空，entry 不允许空 code/text。

`LexiconEntry.weight` 为可选值，以保留无显式权重的文本输入；具体 encoder 或 core 排序逻辑以后按“前项 +1”合同补齐。`PhraseEntry.candidate` 在进入文档模型前必须已经分配，因此为必填非零值。

UTF-16 长度是 `PhraseEntry` 的派生方法，不存储在结构体中，避免文本修改后长度失效。

## 4. Scheme And Encoding Contracts

`LexScheme` 使用八个语义分支；郑码分支内部携带 `formation: bool`，从类型形状上禁止“86 + formation”一类无效组合。

`TextEncoding` 只表示 UTF-8、UTF-16LE、UTF-16BE 和 GBK；`DetectedTextEncoding` 另带 `has_bom`。探测置信度和解码实现不进入本任务，避免提前绑定 `chardetng` API。

## 5. Error Contract

`CodecError` 由 `CodecErrorKind` 与可选 `SourceLocation` 组成。`SourceLocation` 为：

- `ByteOffset(u64)`：0 起，面向二进制和原始字节。
- `Text { line, column }`：line/column 1 起，column 可缺省。

模型构造器产生无位置 validation 错误；未来 parser 在知道来源后附加位置。错误 kind 使用公开枚举和结构化字段，测试与上层转换匹配 kind/location，不解析 `Display` 文本。

codec 错误不包含中文 UI 文案、不依赖 Tauri，也不吞掉底层技术原因。未来 app 层负责映射为统一 `AppError`。

## 6. Resource Limits

`DecodeLimits` 默认：

| Limit | Default | Rationale |
|---|---:|---|
| input bytes | 64 MiB | 当前完整码表为数 MB，为异常输入留余量，同时约束解码放大 |
| expanded entries | 500,000 | 覆盖数十万条真实码表，并防止 `$[...]` 小输入展开为无界记录 |

字段公开或提供构造器以允许调用方下调/调整。零上限合法，便于拒绝全部非空输入和编写边界测试。

future parser 的检查顺序固定为：输入总量 -> header/count/offset 算术 -> 容量预分配 -> 字段切片/解码 -> 展开条目。所有加乘法使用 checked arithmetic，所有窄化使用 `try_from`。

## 7. Dependency And Compatibility

新增直接依赖仅为 `thiserror = 2.0.20`。公共模型不派生 serde，不与 IPC 合同耦合。Rust 仍来自 `rust-toolchain.toml`，不新增 feature 或平台条件编译。

本任务不改变 C1/C2/C5/C6/C9/C10/C12 行为；它只把后续实现共同需要的状态表示和失败语义固定下来。超长 `.lex` 编码从原项目静默截断改为明确拒绝，以避免不可逆数据损坏。

## 8. Rollback Shape

改动仅限 `wubilex-codec` manifest/source/tests 和任务产物。若 API 评审不通过，可整体回退该任务，不影响已完成 workspace。后续格式任务开始后再修改公共类型必须单独做兼容评审，不能在单个 parser 内私自分叉。
