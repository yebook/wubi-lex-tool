# Design - S0-04 EUDP 二进制编解码

## 1. Module And API Boundary

新增 `crates/wubilex-codec/src/eudp/`，通过 `lib.rs` 暴露格式命名空间：

```rust
pub fn decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<PhraseDocument, CodecError>;

pub fn encode(
    document: &PhraseDocument,
    timestamp: i32,
) -> Result<Vec<u8>, CodecError>;
```

API 只接收内存字节、格式文档和显式 timestamp。路径打开、v1/v2 择新/双写、系统时钟、Windows 版本、TSF 编排和备份恢复均不进入本模块。

实现按职责拆为 `eudp/mod.rs`、`eudp/decode.rs` 和 `eudp/encode.rs`。格式常量与少量本模块共享的 checked helper 放在 `eudp/mod.rs`；优先复用已经稳定的公共错误/limits 合同，不为两个 binary module 建立过早的通用 reader 抽象。

## 2. Decode Data Flow

```text
input bytes
  -> input-size preflight
  -> fixed 64-byte header
  -> signed count/structural offset validation
  -> declared wire-entry limit
  -> signed relative offset table + end sentinel
  -> bounded record slices
  -> cbSize / text-offset / candidate / UTF-16 / NUL validation
  -> code-order validation
  -> skip structurally valid deleted records
  -> ordered PhraseDocument
```

所有线上 `INT32` 先读为 `i32`。每次切片、加法、乘法、容量和窄化转换前做 checked 验证。偏移表先验证首项 0、严格递增和相对条目区范围，再追加 `phraseEnd - phraseStart` 哨兵；第 i 条记录只在 `[phraseStart + offsets[i], phraseStart + offsets[i + 1])` 内解析。

记录在所有字段和字符串都验证通过后，才根据 deleted 决定是否构造 `PhraseEntry`。这保证 tombstone 不出现在结果中，同时损坏 tombstone 也不会绕过格式检查。声明的 wire count 在分配偏移数组前受 `max_expanded_entries` 限制，避免 deleted 记录被用来规避资源上限。

## 3. Header Compatibility Policy

严格字段是解释边界所必需的结构：

- magic 必须是 `mschxudp`；
- `phraseOffsetStart` 必须不早于 64 字节头；
- `phraseOffsetStart + count * 4 == phraseStart`；
- `phraseStart <= phraseEnd == input.len()`；
- count 和所有偏移不得为负，所有运算必须可表示。

`magic2`、version、timestamp 和 reserved 不参与记录边界计算；原读取端也不以它们拒绝文件。decode 对这些值容错。encode 不保存这些非语义元数据，而是固定写出 `0x00600002`、version 1、显式 timestamp 和零 reserved。

`count == 0` 时，结构完整的 64 字节规范文件解为合法 empty `PhraseDocument`。这遵守已经冻结的公共模型；长度不足 64 的空/只有部分头输入仍是结构化 EOF 错误。

## 4. Offset Table And Record Contract

偏移表的第一个元素为 0，后续严格递增。最后一条的结束位置来自条目区总长度哨兵，因此每个记录范围都必须完整、非重叠且至少容纳：16 字节头、一个 code unit + NUL、一个 text code unit + NUL。

记录解析顺序：

1. 读取 16 字节头，`cbSize != 16` 返回 `UnsupportedFormat { format: "eudp", ... }`；
2. 校验 text offset 是偶数，满足 `16 < offset < record_len`，且为 code 区提供至少一个字符和 NUL；
3. 校验 candidate 非零；
4. 严格解码 `[16, offset - 2)` 为非空小写 ASCII code，拒绝内嵌 NUL，并校验 `offset - 2` 的 NUL；
5. 严格解码 `[offset, record_len - 2)` 为非空 text，拒绝内嵌 NUL，并校验记录末尾 NUL；
6. 校验 code 对前一 wire 记录非降序；
7. deleted 为 0 时构造 entry，否则丢弃该已验证 tombstone。

`cbSize2`、unknown 和 reserved 在 decode 时不作为拒绝条件，因为它们不参与边界解释且旧读取端未校验；canonical encode 固定写规范值。任何非法 UTF-16 都定位到原始代理项字节，不使用 replacement character。

物理记录只要求按 code 排序。同 code 内保留 wire 顺序，不强制 candidate 单调，因为 candidate 字段本身承载候选位序，旧 writer 也只按 code 排序。这样可兼容显式 candidate 行顺序，同时不会丢失重复记录。

## 5. Canonical Encode

encode 建立条目引用列表，使用稳定排序仅按 `PhraseCode::as_str()` 升序，不修改原文档。相同 code 的输入顺序、candidate 和完全重复项保持不变。

写出分三步：

1. checked 计算 `phraseStart = 64 + count * 4`，预留规范头和偏移表；
2. 对每条 entry 记录当前相对位置，写 16 字节规范头、UTF-16LE code + NUL、UTF-16LE text + NUL；
3. checked 转换并回填所有相对 `i32` offset、`phraseEnd`、count 与调用方 timestamp。

code 的 text offset 使用 `16 + (code_utf16_units + 1) * 2`，必须能表示为 `u16`。因为 `PhraseCode` 已限制为 ASCII，每字符正好一个 UTF-16 unit；仍使用 `encode_utf16()` 统一计算并显式检查。text 没有独立 u16 长度字段，但完整文件和相对 offset 必须落在正 `i32` 范围内；若格式中立模型包含内嵌 `U+0000`，encode 在 format boundary 明确拒绝，避免产生有歧义的终止字符串。

同一 `PhraseDocument` 与同一 timestamp 的输出完全确定。codec 不自行取当前时间；未来系统安装层负责提供 Unix 秒并把同一结果原样写到 v1/v2 两个路径。

## 6. Error Mapping

- magic 不符使用 `MagicMismatch`；
- 字段或完整记录不足使用 `UnexpectedEof`；
- 负数、越界、倒序或不连续的结构偏移使用 `InvalidOffset` 或 `MalformedField`；
- `cbSize != 16` 使用 `UnsupportedFormat`；
- candidate、terminator、code/order 等固定合同使用 `MalformedField`；
- 非法代理项使用 `InvalidUtf16`；
- checked 运算或窄化失败使用 `IntegerOverflow`；
- 输入或声明条目上限使用 `ResourceLimitExceeded`。

每个 decode 错误附最接近失败字段的 `SourceLocation::ByteOffset`。encode 错误来自内存模型或表示上限，不虚构 source location。测试匹配 kind/location，不解析 `Display` 文本。

## 7. Verification Strategy

测试首先用独立手写的规范字节固定 header、offset table、record 和 timestamp，再验证公开 round trip。后续损坏用例只对这份已知字节做单点修改，避免测试 helper 复制完整生产 writer。

覆盖层次：

- 精确字节：empty header、多 code、同 code 重复、emoji 代理对、换行和时间变量文本；
- 语义：未排序 document、同 code 稳定顺序、candidate 边界、deleted 跳过；
- 兼容：非规范但非结构 metadata 被接受后 canonical encode 归一化；
- 错误：header、count、offset table、record header、candidate、code/text、UTF-16 和终止符；
- 健壮性：所有截断前缀、匹配声明长度的记录区截断、input/entry limits 和 checked 边界；
- 范围审计：测试及生产代码不读取 `resource/`，依赖树不新增 crate。

真实 Windows EUDP 样本、文本方言端到端和系统双写验证留给后续任务，不能用不存在的 fixture 伪造完成声明。

## 8. Rollback And Follow-Up

产品代码改动局限于 `wubilex-codec/src/eudp/`、`lib.rs` 和对应测试。若 timestamp API、容错元数据策略或严格损坏策略评审失败，可回退本子任务而不改变已冻结模型与 `.lex` codec。

后续 `s0-phrase-aux` 只依赖公开 PhraseDocument/EUDP API，实现文本方言、`$[...]`、多行与时间变量。v1/v2 文件协调、备份和写后校验由 Windows/application 层承担，不访问 eudp 私有 reader/writer helper。
