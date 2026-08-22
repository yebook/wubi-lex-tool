# S0-02 codec 公共模型

## Goal

为 `wubilex-codec` 冻结后续 `.lex`、EUDP、文本码表、短语与辅助格式共同依赖的格式文档模型、结构化错误和输入资源限制，使各格式子任务共享同一组可测试合同，而不提前实现任何具体解析器或序列化器。

## Background

- `wubilex-codec` 的职责是无状态、纯同步、平台无关的“字节/文本与强类型格式文档互转”；`wubilex-core` 才拥有带倒排索引和编辑操作的常驻领域模型。
- `.lex` 必须保留重复词条、候选顺序和可选权重语义；EUDP 必须保留候选位序，并以 UTF-16 code unit 而非 Unicode scalar 处理 emoji 长度。
- 后续四个格式子任务依赖同一模型和错误位置合同。若在各格式内分别定义，会造成类型、验证规则和错误语义漂移。
- 完整码表通常为数 MB、数十万条；`NFR-PERF-010` 要求打开一份完整码表后常驻内存不超过 300 MB。

## Requirements

| ID | Requirement |
|---|---|
| CM-R01 | 在 `wubilex-codec` 内提供格式中立的 `LexiconDocument` / `LexiconEntry` 与 `PhraseDocument` / `PhraseEntry` 公共模型；模型使用有序 `Vec` 保存条目，必须保留源顺序和重复项。 |
| CM-R02 | `LexiconEntry` 保存小写 ASCII 编码、非空文本和可选非零 `u16` 权重；`.lex` 编码长度为 1..=4，超长编码必须明确拒绝，不得静默截断。 |
| CM-R03 | `PhraseEntry` 保存非空小写 ASCII 编码、非空文本和 1 起的非零 `u8` 候选位序；emoji 等文本长度按需通过 `encode_utf16().count()` 派生，不存冗余字符数字段。 |
| CM-R04 | 空 `LexiconDocument` / `PhraseDocument` 合法，以覆盖只有文件头或 `count == 0`；单条记录的空编码、空文本、零权重和零候选位序非法。 |
| CM-R05 | 提供 8 种码表方案的 `LexScheme` 公共枚举；郑码构词表状态必须只能附着于郑码方案，不允许出现其他方案的无效 formation 组合。 |
| CM-R06 | 提供文本编码探测结果类型，区分 UTF-8、UTF-16LE、UTF-16BE、GBK 及 BOM 是否存在；本任务只定义合同，不实现探测。 |
| CM-R07 | 提供库内结构化 `CodecError`：错误 kind 与来源位置分离；位置至少支持 0 起字节偏移和 1 起文本行/列。错误保持技术语义，不在 codec 层生成 Tauri `AppError` 或 UI 中文文案。 |
| CM-R08 | 错误 kind 至少覆盖无效输入、魔数不匹配、意外 EOF、无效 UTF-16/文本编码、不支持格式、整数溢出和资源上限超限；调用方可稳定模式匹配，不依赖展示字符串。 |
| CM-R09 | 提供可覆盖的 `DecodeLimits`，默认最大输入 64 MiB、最大展开条目数 500,000。未来解析器必须在切片访问、容量预分配和 `$[...]` 展开前检查相应上限。 |
| CM-R10 | 格式自身的硬边界仍由对应格式实现检查，包括 `.lex` 的 `u16` 记录长度、EUDP 的 offset/candidate 宽度和所有 `u16/u32/usize` 转换；通用上限不得替代格式校验。 |
| CM-R11 | 仅新增本任务需要的 `thiserror 2.0.20`；不得引入 serde、Tauri、Windows、Tokio、网络、正则、编码探测或格式解析依赖。 |
| CM-R12 | 本任务不得实现 `.lex`、EUDP、文本方言、版本/编码探测、空白转义、词频或拆字解析，也不得在 `wubilex-core` 建立索引模型。 |

## Acceptance Criteria

- [ ] 公共模型的构造与只读访问 API 可由集成测试使用，条目顺序和重复项保持不变。
- [ ] `.lex` 编码 1/4 字符边界成功，空值、大写、非 ASCII 和 5 字符编码返回可模式匹配的 validation 错误。
- [ ] 可选权重允许缺省与 `1..=65535`，拒绝 0；候选位序允许 `1..=255`，拒绝 0。
- [ ] 短语文本的 UTF-16 单元计数对 BMP 字符和 emoji 代理对返回正确结果，且不存储冗余长度状态。
- [ ] 空文档合法；空编码或空文本记录非法；重复记录不被自动去重。
- [ ] `LexScheme` 覆盖 86、98、06、091、092、郑码（含 formation）、小鹤音形和表形码，不能构造非郑码 formation 状态。
- [ ] `CodecError` 可分别携带 byte offset 或 line/column，测试通过 kind 与 location 断言，不依赖错误字符串。
- [ ] `DecodeLimits::default()` 精确为 64 MiB / 500,000，且可用更小自定义值测试 input/entry limit 超限。
- [ ] `wubilex-codec` 的依赖树不含 Tauri、Windows、Tokio、网络或未进入本任务范围的解析依赖。
- [ ] `cargo fmt --all -- --check`、严格 Clippy 和 workspace 全量测试通过，生产路径没有 `unwrap()` / `expect()`。

## Out Of Scope

- 任何具体二进制或文本格式的 parse/encode 实现。
- `wubilex-core` 的常驻可编辑模型、倒排索引、合并、排序、统计或变换。
- 文件系统、Tauri IPC、UI 错误文案、进度回调、取消和异步执行。
- serde/TypeScript 绑定；codec 模型不是 IPC 模型。
- 真实 fixture、属性测试框架、覆盖率和 CI 接入；由后续 S0 子任务完成。

## Risks And Deferred Items

- 64 MiB / 500,000 是基于当前“数 MB/数十万条”资料和 300 MB 常驻内存指标的防御性默认值；真实 fixture 阶段若证明不足，必须用测量结果调整默认值并保留可配置覆盖能力。
- 格式文档模型刻意不含倒排索引，后续 core 模型转换必须测量额外内存，避免同时持有两份完整字符串数据。
- 展示错误的中文本地化属于 `wubilex-app` 的 `AppError` 转换层，本任务只保证技术错误和位置不丢失。
