# Design - S0-06 短语与辅助文本编解码

## 1. Module And API Boundary

在 `wubilex-codec` 内新增三个文本格式模块和一个探测模块，并把现有编码解码器提升为 crate 内共享实现：

```text
crates/wubilex-codec/src/
  text/encoding.rs   # crate 内共享 BOM/严格解码
  text/phrase/       # 短语文本解析、warning、规范输出
  text/auxiliary.rs  # 辅助文本共享的严格两列解析
  weight/            # 词频 UTF-8 文本编解码
  split_table/       # 拆字数据表 UTF-8 文本编解码
  detect/            # LexiconDocument -> LexScheme
  model/             # 辅助表的保序强类型文档
```

公开 API 的目标形状：

```rust
pub fn phrase_text::decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<DecodedPhraseText, CodecError>;

pub fn phrase_text::format(
    document: &PhraseDocument,
) -> Result<String, CodecError>;

pub fn weight::decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<WordFrequencyDocument, CodecError>;

pub fn weight::format(
    document: &WordFrequencyDocument,
) -> Result<String, CodecError>;

pub fn split_table::decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<SplitTableDocument, CodecError>;

pub fn split_table::format(
    document: &SplitTableDocument,
) -> Result<String, CodecError>;

pub fn detect::scheme(document: &LexiconDocument) -> LexScheme;
```

格式 API 不打开路径，不下载资源，也不产生 EUDP 字节。现有 `text` 和 `text::phrase` 共用 `text/encoding.rs` 与公开 `escape`，crate 根通过 `phrase_text` facade 暴露短语 API；两者保持各自的公开结果和 warning 类型，避免把格式特有语义揉成一个巨型 parser。

## 2. Shared Strict Encoding

把 `text/encoding.rs` 提升为 `text` 子树内的 crate 共享模块，现有码表文本行为保持不变。短语文本完整复用：input limit -> BOM 优先 -> strict UTF-8 fast path -> chardetng `cn` narrowing -> strict GBK/UTF-16 decode，并返回 `DetectedTextEncoding`。

词频和拆字表的文档合同明确为无 BOM UTF-8，因此走更窄的共享入口：先检查输入上限，拒绝 BOM，再用 `Utf8Error::valid_up_to()` 严格解码。格式不允许的 BOM 返回 `UnsupportedFormat`，非法 UTF-8 返回 `InvalidTextEncoding` 及原始 byte offset。

共享重构只移动实现和可见性，不改变码表文本的已发布 API、编码选择、错误 kind 或偏移。

## 3. Phrase Preprocessing And State Machine

短语解码先构建带原始 1 起行号的 records。注释扫描器逐字符寻找最近的 `/*` 与后续第一个 `*/`，将注释内容替换为空白但保留换行和列宽；未闭合注释在起始 delimiter 报错。随后按行执行 tri-state parser：`NoMatch / Matched / Invalid`。

P1..P6 的识别顺序固定。每个 parser 先确认结构轮廓，再验证字段；一旦轮廓匹配，非法数字、code 或空文本返回 `Invalid`，不能继续尝试较宽松方言。空行忽略。

multiline 是显式状态：

```text
P2/P3 with empty text -> pending(code, candidate, source)
next recognized record -> finalize pending, then process record
next nonempty unrecognized line -> append with '\n'
EOF -> finalize pending
```

pending 最终仍为空时返回原始起始行的字段错误。注释剥离后的空行不人为产生换行内容，保持旧解析器忽略空行的行为。

## 4. Phrase Expansion And Candidate Contract

每个 code 维护当前最大 candidate，而不是最后出现值。显式候选先验证 `1..=255` 再更新最大值；缺省候选 checked `max + 1`。

`$[...]` 仅对无显式候选的完整字段启用：

- 内容包含 ASCII space：连续 space 归一化，去除空 token，按 token 展开。
- 不含 ASCII space：按 Rust `char`（Unicode scalar）展开，不拆代理对或 UTF-8 字节。
- 数组候选始终从 1 编到 N，不叠加已有最大值；展开后把该 code 最大值更新为 `max(old, N)`。空数组或超过 255 个元素时，在数组字段位置失败。

P2 的 `#` 前缀只移除一次。时间变量扫描 `$[A-Za-z_]+`，只替换文档列出的九个别名，未知名称保持原样。entry 构造前调用共享 `unescape_whitespace`。

每个 entry 和 warning 在 retain 前共用 `max_expanded_entries` 预算。任何失败丢弃本次解析的全部临时结果。

## 5. Phrase Diagnostics And Canonical Output

不在 multiline 状态且所有 P1..P6 都 `NoMatch` 的非空行产生 `PhraseTextWarningKind::UnrecognizedLine`。warning 复用码表文本的边界：原始行号、最多 160 Unicode scalar 预览、明确截断标志。若原始正文非空但最终没有 entry，返回错误而非成功加 warnings。

formatter 不修改 document：稳定按 code 字典序，再按 candidate 升序排列，重复 candidate 保持源序。每个 code group 只有在 `len > 1`、候选恰为 `1..=len` 且每条 text 的 UTF-16 长度 `<= 2` 时输出：

```text
code<TAB>$[escaped1 escaped2 ...]<CRLF>
```

否则每条输出：

```text
code<TAB>escaped-text<TAB>candidate<CRLF>
```

数组项先做空白转义，因此分隔用的 ASCII space 不与 item 内容混淆。emoji 的 UTF-16 长度为 2，可以压缩；BMP 两字符也可以，BMP 加 emoji 长度 3 不可压缩。

## 6. Auxiliary Table Models

新增保序、保重复的值对象：

- `WordFrequencyEntry { word, weight }` / `WordFrequencyDocument`
- `SplitTableEntry { term, roots }` / `SplitTableDocument`

word、term、roots 必须非空且不包含 Unicode whitespace。为公共模型增加明确的 `InvalidInputReason::ContainsWhitespace { index, character }`，使直接构造无歧义且 formatter 不会生成不可重读文本。文档只拥有 `Vec<Entry>`，不内建 HashMap、去重或覆盖策略。

两个 decoder 都逐行接受 CRLF/LF、忽略纯空白行，并要求恰好两个 Unicode-whitespace token。词频第二字段严格十进制 `1..=65535`；拆字 roots 原样保留 PUA 和非 BMP scalar。任何多余字段或损坏字段严格报 line/column。

两个 formatter 保持 document 顺序和重复项，以 TAB 分隔并使用 LF。空 document 输出空字符串。

## 7. Scheme Detection

探测器单次扫描 `LexiconDocument`，只记录文档定义的少量 `(code, text)` 特征位，不构建完整索引，也不看 weight。重复 entry 不会重复加分，因为每项测试是布尔命中。

扫描后先按固定顺序检查：

1. `q/月 + e/世` -> 郑码
2. `aakk/啊 + hedn/鹤` -> 小鹤音形
3. `qv/月 + ev/世` -> 郑码构词表
4. `sr/版 + ks/吃 + ms/见` -> 092
5. `hodd/够 + opto/啊` -> 表形码

未命中时按九项表更新 86/98/06/091 的 `i8` 分数。`xfxy` 使用小写 code，98、06、091 只有严格高于其余三者才返回；其他情况返回 86。这精确保留旧版优先级和平局语义，同时关闭大写缺陷。

## 8. Errors And Resource Limits

- 字节损坏：`InvalidTextEncoding + ByteOffset`。
- 不支持的辅助表 BOM：`UnsupportedFormat` at byte 0。
- 注释未闭合、已识别 phrase 字段损坏、辅助表列数或数字损坏：结构化 kind + 原始 text line/column。
- 公共值非法：保留 `InvalidInput` kind 并附 parser 拥有的位置。
- input/expanded ceiling：`ResourceLimitExceeded`，行级增长附当前 source location。
- candidate、计数和格式化容量溢出：`IntegerOverflow`，内存模型失败不伪造 source location。

warning 和 entry 共用产出上限；注释、空行和 scheme detection 不产生额外保留记录。格式化使用 checked count/capacity，生产路径不使用 `unwrap`、`expect`、unsafe 或 lossy conversion。

## 9. Verification Strategy

测试按公共 API、内部边界和回归三层组织：

- `phrase_text.rs`：手写四编码字节、P1..P6、状态转换、arrays、aliases、warnings、完整格式字符串和 round trip。
- `auxiliary_text.rs`：词频/拆字 good-base-bad、PUA/non-BMP、重复项、BOM/UTF-8、位置和 limits。
- `scheme_detection.rs`：五个直接分支、四个 scored 分支、优先级、平局、重复项、`xfxy` 失败到通过。
- 现有码表文本全量测试证明共享 encoding 重构无回归；workspace 门禁证明无跨层漂移。

真实八方案样本和大文件测量仍由后续 fixture 任务建立可复现来源。根目录 `resource/` 不进入命令、测试或文档依赖。

## 10. Rollback And Follow-Up

共享 encoding 可见性提升、phrase text、aux models/codecs 和 scheme detection 是可独立定位的实现块，但作为一个 S0 文本合同统一通过检查后提交。若真实 fixture 证明社区方言或辅助表存在未记录变体，先把样本来源和期望写入 `s0-fixtures-regressions`，再扩展 parser；不回退到静默丢损坏字段或 lossy 解码。
