# S0-03 .lex 二进制编解码

## Goal

在 `wubilex-codec` 中实现微软五笔原始 `.lex` 字节与已冻结 `LexiconDocument` 之间的安全双向转换，为后续系统码表读取、导出和真实 fixture 回归提供确定、可测试的格式合同。

## Background

- 权威布局来自 `docs/01-data-formats.md`：小端、`imscwubi` 魔数、64 字节头、26 项字母索引、UTF-16LE 变长记录。
- `s0-codec-model` 已冻结保序且保留重复项的 `LexiconDocument`、1 至 4 位小写 ASCII `LexCode`、非零 `Weight`、结构化 `CodecError` 和 `DecodeLimits`。
- 用户提供的 `resource/微软五笔86( 完整 ).lex` 是只读本机样本。2026-08-22 审计确认其 SHA-256 为 `969BAE11DAA3C3D9A66D50C26A3EC5F47AAB629EB5076D8D7BC0A4777C3898DB`，大小 4,467,680 字节，包含 207,055 条记录和 193,261 个编码，头部、字母索引、排序、填充、终止符及 UTF-16 均符合规范。
- 根目录 `resource/` 是用户数据，不是仓库 fixture；自动化测试不得依赖、修改或提交该目录。

## In Scope

1. 在 `crates/wubilex-codec/src/lex/` 提供原始 `.lex` 的公开 decode/encode API。
2. 显式解析和写出头部、字母索引与变长记录，不把外部字节映射为 Rust ABI 结构体。
3. 严格检查边界、长度、偏移、索引、排序、UTF-16、编码、权重、填充与终止符，并返回带字节偏移的结构化错误。
4. decode 使用 `DecodeLimits` 限制输入字节数与展开记录数；encode 对所有算术和线格式上限做 checked 校验。
5. encode 生成确定性的规范化 1.1 文件，稳定保留同编码内的词条顺序、重复项和显式权重，并实现缺省权重的隐式递增规则。
6. 用手写合成字节、往返、边界、损坏输入和本机真实样本审计验证合同。

## Out Of Scope

- `.lex.lzma` 解压、`.lex.zst` 读写及任何文件系统或网络 I/O；压缩属于 `wubilex-resource` 的 archive 边界。
- 文本码表、CSV、JSON、EUDP、词频、拆字、空白转义和码表方案探测。
- 码表合并、统计、倒排索引、同码词条重排等 `wubilex-core` 领域操作。
- 将根目录 `resource/` 复制为测试 fixture，或在本任务内建立 8 种方案的可复现下载集。
- 保留任意输入头部的非语义元数据；encode 始终输出规范化头部。

## Requirements

| ID | Requirement |
|---|---|
| LB-R01 | `wubilex_codec::lex` 提供纯同步、内存到内存的 decode/encode API；生产代码只使用标准库和现有 `thiserror`，不引入平台、异步、网络、序列化或压缩依赖。 |
| LB-R02 | decode 必须在读取字段或分配记录前检查完整输入大小，并在每次加入记录前检查展开条目上限。 |
| LB-R03 | decode 严格校验 8 字节 `imscwubi` 魔数；按线上的有符号偏移读取并结构化校验 `indexOffset`、`tableOffset` 和 `fileSize`，且 `fileSize` 必须等于实际输入长度。 |
| LB-R04 | 为兼容既有文件，decode 不因 `majorVersion`、`minorVersion`、`unknown` 或 `reserved` 的非规范值单独拒绝输入；encode 固定输出 1.1、`0x40`、`0xA8`、`0x78563412` 与零保留区。 |
| LB-R05 | 26 项字母索引必须非负、单调、位于记录区范围内、落在记录边界，并与按编码首字母划分的实际记录区间一致；记录编码必须按字典序非降序。 |
| LB-R06 | 每条记录必须满足偶数字节长度且至少 16 字节、非零权重、1 至 4 位小写 ASCII 编码、未使用编码位为零、非空且严格合法的 UTF-16LE 文本以及零终止符。 |
| LB-R07 | decode 输出的每条二进制记录都带 `Some(Weight)`，保持线记录顺序和重复项，不排序、不去重、不建立索引。 |
| LB-R08 | encode 对条目副本按编码做稳定升序排列；同编码内保持输入顺序和重复项。显式权重替换当前权重，缺省权重在当前值上加一；首次缺省为 1，溢出返回结构化错误。 |
| LB-R09 | encode 必须按 UTF-16 code unit 计算长度，拒绝超出 `u16` 记录长度或 `i32` 文件/索引范围的模型，并生成完整且确定性的字母索引。 |
| LB-R10 | 任一损坏输入都返回 `CodecErrorKind` 的可匹配字段证据和最接近问题字段的零起字节偏移；不得 panic、静默截断、lossy 解码或返回部分成功文档。 |
| LB-R11 | 规范化 `.lex` 必须满足 `decode -> encode` 字节级一致；非规范但结构有效且被容错接受的头部元数据允许在 encode 时规范化，只保证语义一致。 |
| LB-R12 | 测试必须独立断言公开行为，覆盖空码表、索引空洞、重复项、非 BMP 文本、缺省权重、所有关键损坏字段、截断前缀、资源限制和算术边界；不得通过复刻生产编码算法制造自证式断言。 |

## Acceptance Criteria

- [x] 手写规范字节经 decode 得到预期条目、权重、顺序和重复项，再 encode 回完全相同的字节。
- [x] 未排序文档经 encode 后编码稳定升序，同编码内顺序与重复项不变，26 项索引精确指向对应记录边界。
- [x] 空码表、首字母空洞、最大四码、非 BMP 文本以及显式/缺省权重混合均往返通过。
- [x] 魔数、文件大小、负数/越界偏移、损坏索引、乱序编码、记录长度、零权重、编码长度/内容/填充、空文本、非法代理项和终止符错误均返回正确 kind、字段和字节偏移。
- [x] 对每个规范合成样本的所有截断前缀，decode 只返回结构化错误且不 panic；自定义输入/条目上限在边界值准确生效。
- [x] 本机只读验证中，用户提供的 4,467,680 字节样本 decode 为 207,055 条记录，并 encode 回相同 SHA-256；自动化测试和提交均不包含 `resource/`。
- [x] `cargo fmt`、严格 Clippy、workspace check/test、Rustdoc、依赖树检查和 `git diff --check` 全部通过，production source 无 `unwrap()` / `expect()`。

## Risks And Deferred Items

- 非规范头部元数据不会进入 `LexiconDocument`，因此只能语义往返；这是格式中立模型边界的有意结果。
- 8 种方案的真实、可复现 fixture 与属性测试集中到后续 `s0-fixtures-regressions`，本任务只建立可被其复用的 raw codec。
- `.lex.lzma` 样本可在未来 resource/archive 集成时用于验证 LZMA alone 解压，当前任务不读取其内容。
