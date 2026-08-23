# S0-04 EUDP 二进制编解码

## Goal

在 `wubilex-codec` 中实现微软五笔 EUDP 原始字节与已冻结 `PhraseDocument` 之间的安全双向转换，为后续短语文本方言、系统短语导入安装和真实 fixture 回归提供确定、可测试的格式合同。

## Background

- 权威布局来自 `docs/01-data-formats.md`：小端、`mschxudp` 魔数、64 字节头、相对条目区的偏移表，以及 UTF-16LE 终止字符串条目。
- `s0-codec-model` 已冻结保序且保留重复项的 `PhraseDocument`、非空小写 ASCII `PhraseCode`、1 起的 `Candidate`、结构化 `CodecError` 和 `DecodeLimits`。
- `ChsWubiEUDPv1.lex` 与 `ChsWubiEUDPv2.lex` 只有系统路径和择新策略不同；原项目把同一组字节复制到两个路径，codec 不存在两套 v1/v2 wire 实现。
- 头部 timestamp 是写入时刻，但 codec 必须保持纯函数；编码 API 由调用方显式传入 `i32` Unix timestamp，不读取系统时钟。同一文档和 timestamp 必须生成相同字节。
- 原 aardio 在条目文本范围为负时静默跳过，但当前 `NFR-REL-008` 和 backend error contract 要求损坏短语返回精确错误且不得部分成功。本任务采用严格失败；`deleted != 0` 仅在条目结构完整后作为合法 tombstone 跳过。
- 根目录 `resource/` 目前只有用户提供的 `.lex` / `.lex.lzma`，没有 EUDP 样本。它仍是只读用户数据，不是测试 fixture，本任务不得修改、引用或提交它。

## In Scope

1. 在 `crates/wubilex-codec/src/eudp/` 提供原始 EUDP 的公开 decode/encode API。
2. 显式解析和写出头部、偏移表与变长条目，不把外部字节映射为 Rust ABI 结构体。
3. 严格检查输入边界、结构偏移、记录范围、`cbSize`、candidate、代码/文本终止符、UTF-16、编码内容和编码顺序，并返回带零起字节偏移的结构化错误。
4. decode 在完整验证 wire 条目后跳过 `deleted != 0`，对有效条目保持线顺序和重复项，不建立映射或去重。
5. encode 生成规范化 EUDP 字节：稳定按 code 升序，同 code 内保持输入顺序、candidate 和重复项，固定写出规范头/条目常量，并使用调用方 timestamp。
6. decode 使用 `DecodeLimits` 限制完整输入和声明的 wire 条目数；encode 对 offset、count、UTF-16 长度、文件大小与所有整数转换做 checked 校验。
7. 用独立手写字节、精确往返、emoji、换行文本、重复项、deleted、边界、截断和损坏输入测试固定合同。

## Out Of Scope

- v1/v2 系统路径定位、最后写入时间择新、双写/复制、Windows 版本判断、注册表、TSF、备份、回滚与任何文件系统 I/O。
- 短语文本 P1-P6 方言、`$[...]` 展开/压缩、时间变量别名、空白转义和默认短语库；这些属于后续 `s0-phrase-aux`。本任务只保证其最终产生的 emoji、换行和 `%...%` 普通字符串可被 raw EUDP 编解码。
- EUDP 拖放分派缺陷、郑码造词和反向索引；分别属于 S4 或 `wubilex-core`/应用层。
- 将根目录 `resource/` 作为测试输入，或在缺少来源和完整性信息时提交任意真实 EUDP 文件。
- 保留输入中的 timestamp、reserved 或其他非语义元数据；decode 输出格式中立模型，encode 输出规范值和显式传入的 timestamp。

## Requirements

| ID | Requirement |
|---|---|
| EU-R01 | `wubilex_codec::eudp` 提供纯同步、内存到内存的 `decode(input, limits)` 与显式 timestamp 的 `encode(document, timestamp)`；不引入平台、异步、网络、文件系统、时钟或新第三方依赖。 |
| EU-R02 | decode 在读取任何字段或分配偏移/条目集合前检查完整输入大小，并在按 `count` 分配或遍历前检查声明的 wire 条目数上限；deleted 条目同样计入此防御性上限。 |
| EU-R03 | decode 严格校验 8 字节 `mschxudp` 魔数；所有线上 `INT32` 先保留为有符号值，再校验 offset table、`phraseStart`、`phraseEnd`、`count` 和实际输入长度的 checked 关系。 |
| EU-R04 | 固定头为 64 字节；offset table 从 `phraseOffsetStart` 开始并恰好在 `phraseStart` 结束；`phraseEnd` 必须等于实际输入长度。`count == 0` 的结构完整文件合法并解为 empty document。 |
| EU-R05 | 偏移表首项必须为 0，后续项严格递增，所有项位于条目区并形成无重叠的完整记录范围；任一负值、越界、倒序、重复或落入非记录边界的 offset 均失败。 |
| EU-R06 | 每条记录必须至少容纳 16 字节头、非空 code 及其 NUL、非空 text 及其 NUL；`cbSize` 必须为 16，否则返回 `UnsupportedFormat`。text offset 必须为偶数、位于记录范围内并精确指向 code 终止符之后。 |
| EU-R07 | 每条 wire 记录的 candidate 必须在 1..=255；code 必须是非空小写 ASCII，code/text 必须是严格合法 UTF-16LE、不得内嵌 `U+0000`，且各自以单个零 code unit 结束。不得 lossy 解码、静默截断或返回部分文档。 |
| EU-R08 | `deleted != 0` 的 wire 条目在完整结构验证后不进入 `PhraseDocument`；合法 active 条目保持 wire 顺序与重复项。记录 code 必须按字典序非降序；同 code 的物理顺序不强制按 candidate 排列，candidate 字段本身决定候选位序。 |
| EU-R09 | 为兼容相同布局的既有文件，decode 不因 `magic2`、`version`、timestamp、`cbSize2`、unknown 或 reserved 的非规范值单独拒绝；encode 固定输出 `0x00600002`、version 1、`cbSize/cbSize2 = 16`、unknown 6/0、deleted 0 和零 reserved。 |
| EU-R10 | encode 对条目引用按 code 做稳定升序，不改变输入文档，不重排同 code 记录，不去重；offset table 必须精确指向每条记录，所有字符串使用 UTF-16LE + NUL。 |
| EU-R11 | encode 按 UTF-16 code unit 计算 code/text 长度，显式校验 code text-offset 的 `u16` 上限、候选 `u8` 合同、count/相对 offset/文件大小的 `i32` 上限及 `usize` 算术，任何不可表示值返回结构化错误而非截断。 |
| EU-R12 | 任一损坏输入返回可匹配的 `CodecErrorKind` 和最接近问题字段的 `SourceLocation::ByteOffset`；规范输入在提供相同 timestamp 时满足 `decode -> encode` 字节级一致，容错元数据输入只保证语义一致并被规范化。 |
| EU-R13 | 测试必须独立断言公开行为，覆盖空文档、重复项、非 BMP emoji、换行/时间变量文本、deleted、非规范元数据、所有关键损坏字段、全部截断前缀、资源上限和算术边界；不得复刻生产 encoder 制造自证 fixture，也不得依赖根目录 `resource/`。 |

## Acceptance Criteria

- [x] 手写规范 EUDP 字节经 decode 得到预期 code、text、candidate、顺序和重复项，再用相同 timestamp encode 回完全相同的字节。
- [x] 未排序 `PhraseDocument` 经 encode 后 code 稳定升序，同 code 内输入顺序、candidate 和重复项不变，offset table 精确指向每条记录。
- [x] 空文档、最大 candidate、长 code 边界、emoji 代理对、换行文本和 `%yyyy%` 等普通文本均通过往返。
- [x] `deleted != 0` 条目在结构完整时被跳过；损坏的 deleted 条目仍返回错误，不产生部分成功文档。
- [x] 魔数、负数/越界头字段、错误 count/phraseStart/phraseEnd、损坏偏移表、错误 `cbSize`、记录/text offset、零 candidate、非法 code、空或内嵌 NUL text、非法代理项和终止符错误均返回正确 kind、字段和字节偏移。
- [x] 对规范合成样本的每个截断前缀，decode 只返回结构化错误且不 panic；输入/条目上限在边界值准确生效。
- [x] v1/v2 兼容性以“同一 codec 字节可原样写入两个路径”的合同固定，不创建虚构的第二套线格式或平台依赖。
- [x] `cargo fmt`、严格 Clippy、workspace check/test、Rustdoc、依赖树检查和 `git diff --check` 全部通过，production source 无 `unwrap()` / `expect()` / unsafe / transmute。

## Risks And Deferred Items

- 当前没有真实 EUDP 样本，无法在本任务声称 Windows 产物逐字节回归；可复现真实 fixture 和实际系统兼容验证留给 `s0-fixtures-regressions` / 后续 Windows 集成任务。
- 旧实现对负 text range 的静默跳过不保留，因为它无法同时满足精确错误位置和禁止部分成功的可靠性合同；若真实 Windows 样本证明存在可恢复的此类条目，必须用样本和明确的 warning/partial-result API 重新评审，不能在现有 `Result<PhraseDocument, CodecError>` 中静默吞错。
- timestamp 是非语义头字段，`PhraseDocument` 不保存它；调用方负责提供当前 Unix 秒或测试固定值。该选择避免 codec 依赖时钟，并使测试与重放确定。
- `$[...]`、多行方言和时间别名的端到端文本往返要到 `s0-phrase-aux` 才能关闭；本任务只覆盖其原始字符串在 EUDP wire 中不丢失。
