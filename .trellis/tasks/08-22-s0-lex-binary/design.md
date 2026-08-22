# Design - S0-03 .lex 二进制编解码

## 1. Module And API Boundary

新增 `crates/wubilex-codec/src/lex/`，通过 `lib.rs` 暴露格式命名空间：

```rust
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<LexiconDocument, CodecError>;
pub fn encode(document: &LexiconDocument) -> Result<Vec<u8>, CodecError>;
```

API 只接收内存字节和格式文档。路径打开、压缩解包、系统码表定位、领域索引和排序操作不进入本模块。`DecodeLimits` 按值传入；它是 `Copy`，调用方可显式使用默认值或自定义上限。

实现按职责拆为 `lex/mod.rs`、`lex/decode.rs` 和 `lex/encode.rs`。格式常量与仅供本模块共享的 checked helper 放在 `lex/mod.rs`，不新增通用抽象。

## 2. Decode Data Flow

```text
input bytes
  -> input-size preflight
  -> fixed header fields
  -> structural offsets and exact file-size validation
  -> 26 relative alpha offsets
  -> bounded record walk from tableOffset to fileSize
  -> strict field / UTF-16 / order validation
  -> alpha-index-to-record-boundary validation
  -> ordered LexiconDocument
```

读取器以切片和显式位置工作。每次读取先检查剩余字节，再做小端转换；所有线上的 `INT32` 先读为 `i32`，避免把负偏移误解释为巨大正数。记录只有在字段全部通过且条目上限允许时才加入结果，因此错误不会返回部分文档。

UTF-16 helper 先扫描代理项配对并定位第一个错误 code unit，再调用严格 `String::from_utf16`。这样 `InvalidUtf16` 同时包含代理项数值和原始字节偏移，不使用 replacement character。

解析记录时保存每条记录的相对起点与首字母。完成后重建规范字母分区，逐项核对线索引；这同时验证索引落在记录边界、空字母区间和实际首字母分组。任何编码逆序都会在对应记录编码位置提前失败。

## 3. Header Compatibility Policy

魔数和结构字段决定能否安全解释输入，必须严格：

- `indexOffset` 至少位于固定头之后，并能容纳 26 个 `i32`；
- `tableOffset` 不早于索引末尾；
- `fileSize` 等于传入切片长度；
- 所有加法、乘法和整数转换使用 checked 操作。

版本号、marker 和 reserved 不参与布局计算。decode 对其容错，以兼容相同布局的既有文件；encode 不保存这些非语义元数据，而是固定生成文档规定的 1.1 规范头。规范输入可字节往返，容错输入只承诺语义往返。

## 4. Record Contract

记录起点处的 `length` 覆盖完整记录。decode 按以下顺序验证：

1. 固定头和最小 16 字节；
2. 偶数长度与记录末尾不越过 `fileSize`；
3. 非零权重与 `codeLength` 1..=4；
4. 使用中的编码单元是小写 ASCII，未使用的 4 单元槽为零；
5. `length - 16` 对应非空、严格合法的 UTF-16LE 文本；
6. 最后一个 `u16` 为零终止符。

二进制总是携带权重，因此 decode 构造 `Some(Weight)`。重复的 code/text 记录不会折叠。

## 5. Canonical Encode

encode 建立条目引用列表并使用稳定排序按 `LexCode::as_str()` 升序排列，不改变原文档，也不重排同编码候选。

写出时先保留 168 字节规范头与索引空间，再逐条追加记录。每遇到新的首字母，补齐此前没有记录的字母索引；结尾把剩余字母索引指向记录区末尾。最后 checked 转换文件大小并回填头部与索引。

每个编码组的权重状态从 0 开始：

- `Some(weight)`：写该值并把当前状态设为该值；
- `None`：当前状态 checked 加一后写出；
- 65535 后再出现 `None`：返回 `IntegerOverflow`。

文本先收集 UTF-16 units，再 checked 计算 `16 + units * 2`。最大可编码文本为 32,759 个 UTF-16 units；更长文本返回结构化字段错误，不截断。

## 6. Error Mapping

- 固定值不符使用 `MagicMismatch` 或 `MalformedField`；
- 读取不足使用 `UnexpectedEof`；
- 负数或越界偏移使用 `InvalidOffset`；
- 非法代理项使用 `InvalidUtf16`；
- checked 运算和整数转换失败使用 `IntegerOverflow`；
- 输入或条目上限使用已有 `ResourceLimitExceeded`。

每个 decode 错误附最接近失败字段的 `SourceLocation::ByteOffset`。测试匹配 kind/location，不解析 `Display` 文本。

## 7. Verification Strategy

测试先用独立手写的最小规范字节固定 wire contract，再验证公开 round trip。其他测试从这份已知字节做单点破坏，避免测试 helper 复制完整生产 encoder 后形成自证。

覆盖层次：

- 精确字节：空文件头、跨字母索引、重复记录、UTF-16 代理对；
- 语义：未排序文档、同码稳定顺序、显式/缺省权重；
- 错误：每个头/索引/记录字段和所有截断前缀；
- 资源：输入大小与展开条目边界；
- 本机审计：读取用户样本后重新编码并比对长度与 SHA-256，但不把该路径写进自动化测试。

## 8. Rollback And Follow-Up

改动局限于 `wubilex-codec/src/lex/`、`lib.rs` 和对应测试。若公共 API 或容错策略评审失败，可回退本子任务而不改变已冻结模型。

真实 8 方案 fixture、压缩容器、文本互转和方案探测分别由后续子任务承担；它们只依赖本任务公开 API，不访问其私有 reader/writer helper。
