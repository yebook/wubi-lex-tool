# Design - S0-05 码表文本编解码

## 1. Module And API Boundary

新增两个纯同步模块：

```text
crates/wubilex-codec/src/
  text/
    mod.rs       # 公开类型和 API
    encoding.rs  # BOM/探测/严格字节解码
    decode.rs    # 预处理、六方言、诊断和权重归一化
    encode.rs    # 有效权重、稳定排序和七格式
  escape/
    mod.rs       # 码表/短语可共用的空白编解码
```

公开形状：

```rust
pub fn decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<DecodedLexiconText, CodecError>;

pub fn format(
    document: &LexiconDocument,
    format: LexiconTextFormat,
) -> Result<String, CodecError>;

pub fn encode_utf16le(
    document: &LexiconDocument,
    format: LexiconTextFormat,
) -> Result<Vec<u8>, CodecError>;
```

`DecodedLexiconText` 拥有 `LexiconDocument`、`DetectedTextEncoding` 和 `Vec<LexiconTextWarning>`。`LexiconTextFormat` 用 7 个语义分支代替旧版的 4 个布尔参数。路径打开、文件写入、CSV/JSON/压缩、编辑状态和领域索引不进入此 API。

## 2. Encoding Detection And Strict Decode

```text
input bytes
  -> complete input limit
  -> BOM selection (UTF-8 / UTF-16LE / UTF-16BE)
  -> otherwise strict UTF-8 fast path
  -> otherwise chardetng with cn hint
  -> narrow every non-UTF-8 legacy guess to strict GBK
  -> strict decode without replacement
  -> decoded text + DetectedTextEncoding
```

无 BOM 纯 ASCII 固定归为 UTF-8。`chardetng = 1.0.0` 仅用于无 BOM 的选择；它若返回 UTF-8 就严格按 UTF-8 解码，其他 legacy guess 在公开支持范围内统一窄化为 GBK，随后必须通过 strict GBK 解码。这不声称 detector 识别出的所有 legacy encoding 都是 GBK，而是 UTF-8/GBK 两分公开合同的确定性 narrowing。

UTF-8 用 `Utf8Error::valid_up_to()` 定位首个非法字节。UTF-16 先检查偶数长度，再按字节序组成 code unit 并严格校验代理项。GBK 使用 `encoding_rs = 0.8.35` 的 without-replacement streaming decoder，根据 consumed byte count 定位 malformed sequence。BOM 字节不进入文本也不计入文本列号，但字节错误位置始终相对原始输入。

`encode_utf16le` 输出 `FF FE` BOM，随后是 `format()` 结果的小端 UTF-16 code units。空 document 格式化为空字符串，字节输出仍只包含 BOM，便于后续无歧义重新探测。

## 3. Line Provenance And Preprocessing

解码后立即构建保留原始 1 起行号的 line records，不做整段破坏性替换后再重算位置。预处理依次：

1. 识别文档开头的 `---` 到 `...` YAML front matter；未闭合头是带行号错误。
2. 移除以 `#` 开头的注释行，保留空行位置信息。
3. 查找 `[Text]` 描述/正文边界；其前内容只用于极点标志检测。
4. 在 `[Text]` 正文的首个非空行上检测微软分支。

这种表示让 parse error、resource-limit error 和 warning 都能指向原始文档行号，即使 YAML 和描述区已被跳过。

## 4. Dialect Parser And Visible Warnings

解析器按 A..F 顺序执行返回 `NoMatch / Matched / Invalid` 的结构化 token parser。分隔符只使用显式 ASCII 空白集，同时保留 token 起始列；code、weight、escape 和空文本由显式 validator/模型构造器检查，避免把“已识别但损坏”误分类为未知行。该有限行语法不需要引入 regex 依赖。

- A/B 只允许 1..=65535 升序权重。
- C/E 的词条列按 ASCII 空白拆分，每个片段先解转义再构造 entry。
- D 允许源权重 0..=65534，转为 `65535 - source`；全部 D entry 的最小值大于 5000 时统一平移。
- F 及微软分支先验证词条，再对每个 code 逐条检查展开上限。

无法分类的非空行产生 `LexiconTextWarningKind::UnrecognizedLine`，保留 `SourceLocation::Text` 和最多 160 个 Unicode scalar 的 preview，按源顺序返回。warning 与 entry 共用 `max_expanded_entries` 产出预算，避免全未知行文件产生无界诊断内存。正文真正为空时可返回 empty document；正文有非空非注释行但最终零 entry 时返回 `MalformedField`，不返回“成功 + 全 warnings”。

## 5. Jidian Cleanup And Resource Limits

内部 parsed entry 在构造最终 document 前临时保留源行号。极点清理从已检查展开上限的条目流中删除 `^/$/!`，剔除 `~`，并把所有保留项权重改为 `None`。单独的 `~` 在剔除后变空，返回对应源行的结构化错误。

`DecodeLimits` 检查顺序为：input bytes -> 解码缓冲 checked capacity -> 每个候选 entry/warning 追加前的产出计数。即使 entry 后续被极点删除，或行只产生 warning，也不能绕过上限。

## 6. Canonical Formatting

格式化在 entry 引用副本上进行，不改变 document：

1. 稳定按 code 字典序分组。
2. 每个 code 组按源顺序计算 effective weight：显式权重替换当前值，`None` 使当前值 checked +1，首个缺省为 1。
3. 组内稳定按 effective weight 升序排序，等权和重复项保持源顺序。
4. 将词条的 6 种 ASCII 空白编码后，按格式写入。

码前聚合格式只折叠排序后的相邻重复词条。词前聚合格式按 canonical stream 中词条首次出现顺序分组，只折叠相邻重复 code。短语格式在每个 code 组的权重排序完成后重新编号 1..N，不写原始 effective weight。所有非空输出行以 `\r\n` 结束。

## 7. Whitespace Escape Contract

`escape` 按字符扫描，仅处理 ASCII space/Tab/LF/CR/VT/FF，分别写为 `%20/%09/%0A/%0D/%0B/%0C`。`unescape` 仅识别这 6 个大写序列；未知、小写、不完整或字面 `%` 原样保留。这修复旧 `%0B/%0C` 不对称，但不创造不兼容的 `%25` 扩展。

## 8. Errors And Diagnostics

- 字节解码损坏：`InvalidTextEncoding` + `ByteOffset`。
- YAML 未闭合、非空零 entry、格式字段损坏：`MalformedField` + text line/column。
- model code/text/weight 失败：保留 `InvalidInput` kind 并附 parser 拥有的 text location。
- input/expanded ceiling：`ResourceLimitExceeded`，条目上限附当前行。
- effective weight/capacity/UTF-16LE output size 溢出：`IntegerOverflow`，内存模型错误不伪造 location。
- 未知行：warning，不是 error，也不被静默丢弃。

## 9. Dependencies And Verification

直接依赖精确 pin 为 `chardetng = 1.0.0` 和 `encoding_rs = 0.8.35`，均为 MIT/Apache 兼容的纯 Rust crate，后者另含 BSD-3-Clause 许可部分。不开启 chardetng 的 Rayon feature；行语法使用小型 token parser，不把 workspace 中的传递 regex 升为 codec 直接依赖。codec 仍无平台、网络、文件系统、Tokio 依赖。

测试使用手写字符串/字节和单点损坏样本，独立断言 API、完整格式字符串、编码元数据、warning/error 位置和 limits。真实社区文本与旧 aardio 产物对照仍由 `s0-fixtures-regressions` 建立可复现来源后完成。

## 10. Rollback And Follow-Up

产品改动限于 codec manifest/lockfile、`text/`、`escape/`、公开导出与对应测试。若 warning API 或编码探测策略在真实 fixture 中证明不足，必须用可复现样本调整，不回退到静默忽略或 lossy 解码。后续 `s0-phrase-aux` 复用公开 escape 合同，不复制其内部实现。
