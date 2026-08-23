# S0-05 码表文本编解码

## Goal

在 `wubilex-codec` 中实现社区码表文本与 `LexiconDocument` 之间的双向转换：导入时自动识别文本编码、6 种行方言和微软专用分支，导出时稳定产生 7 种既定文本布局，为后续编辑、格式转换和真实 fixture 回归提供可测试的格式合同。

## Background

- 权威行为来自 `docs/01-data-formats.md` 第 3/8 节和 `docs/modules/M1-lex-table.md` 的 `M1-PARSE-002..008` / `M1-XFORM-001..007`。旧 aardio 代码只用于核对控制流。
- `LexiconDocument` 已固定为保序、保留重复项的 entry stream；`LexCode` 限定 1..=4 位小写 ASCII，`Weight` 为 1..=65535 的可选显式权重。
- `TextEncoding` / `DetectedTextEncoding` 已固定 UTF-8、UTF-16LE、UTF-16BE、GBK 及 BOM 合同，本任务负责首次实现探测与严格解码。
- 旧解析器对未匹配行静默忽略；新实现保留兼容导入，但必须把未知非空行作为带行号的可见警告返回。已识别语法的非法字段仍是错误，非空正文零有效条目也是错误。
- 根目录 `resource/` 只有用户的 `.lex` / `.lex.lzma`，没有文本码表样本；该目录继续保持只读、不引用、不提交。

## In Scope

1. 从原始字节探测并严格解码 UTF-8（有/无 BOM）、带 BOM 的 UTF-16LE/BE 和 GBK；BOM 优先，返回 `DetectedTextEncoding`。
2. 按固定顺序处理 YAML front matter、`#` 注释、`[Text]` 描述头和极点五笔标记。
3. 按 A 到 F 优先级解析 6 种主循环行格式，并实现 `[Text]` 后的微软“词前码后、一行多码”专用分支。
4. 实现显式/缺省权重、降序权重 `65535 - source` 及最小值平移到 5000 的归一化合同，并删除 `codeWeight[0]` 死分支。
5. 实现码表和短语文本可共用的 ASCII 空白转义：`%20/%09/%0A/%0D/%0B/%0C`，修复旧实现编码/解码字符集不对称。
6. 用显式枚举暴露 7 种输出：码前词后、码前聚合、码前升序权重、词前码后、词前聚合、词前降序权重、短语升序号。
7. 输出前按 code 字典序及同 code 有效权重稳定排序；缺省权重按前项 +1 补齐，保留重复项和等权项的稳定顺序。
8. 所有输出词条应用空白转义，固定分隔符和 `\r\n`；同时提供固定 UTF-16LE 的字节输出边界。
9. 返回解码结果、探测编码和诊断列表。未匹配非空行产生可见警告；已匹配但字段非法、或非空正文零有效条目时返回错误。
10. 在解码前检查输入字节上限，每次拆分前检查展开条目上限；任何错误不返回部分 document。

## Out Of Scope

- CSV / JSON 码表容器、`.lex.lzma` / `.lex.zst`、路径和文件系统 I/O。
- 短语 P1..P6、`$[...]`、时间变量、词频、拆字表和 8 种码表方案探测。
- 码表合并、倒排索引、统计、精简、调频和简繁转换；这些属于 `wubilex-core`。
- Tauri、Windows、Tokio、网络逻辑，以及对根目录 `resource/` 的任何访问。
- 真实可复现 fixture 与全局 90% 覆盖率出口；集中到 `s0-fixtures-regressions`。

## Requirements

| ID | Requirement |
|---|---|
| LT-R01 | `wubilex_codec::text` 提供纯同步、内存到内存的码表文本解码/格式化 API；新依赖仅限纯 Rust 文本编码 crate，必须精确 pin 并审计。 |
| LT-R02 | decode 在探测前检查完整输入上限；BOM 优先识别 UTF-8/UTF-16LE/BE，无 BOM 输入在 UTF-8 与 GBK 间可确定重放地探测，不使用系统 locale。 |
| LT-R03 | 非法字节返回 `InvalidTextEncoding` 及最接近的零起字节偏移，不得 lossy 解码。 |
| LT-R04 | 预处理顺序固定为 YAML -> `#` 注释 -> `[Text]` -> 极点/微软检测；合法空行不产生条目。 |
| LT-R05 | 微软分支仅在 `[Text]` 正文首行符合特征时启用，一词多码展开为无权重 entry，之后提前返回。 |
| LT-R06 | A..F 按文档优先级尝试；A/B 产生显式升序权重，C/E 拆分一码多词，D 产生反转后权重，F 展开一词多码。 |
| LT-R07 | 每次追加 entry 或 warning 前共用 `max_expanded_entries` 产出预算；超限错误携带确切 `limit/actual` 和当前 1 起行号，不得用未知行绕过内存上限。 |
| LT-R08 | A/B 拒绝 0 和超出 `u16` 的权重；D 的 `65535 - source` 必须落在 1..=65535，并在全文完成后执行基线 5000 平移。 |
| LT-R09 | 极点标志触发后清空权重，删除 `^/$/!` 开头词条，对 `~` 去掉首字符，并删除变空的编码组。 |
| LT-R10 | 空白编解码在 `%20/%09/%0A/%0D/%0B/%0C` 上对称，只解码这些大写转义，不臆测其他 `%xx`。 |
| LT-R11 | 7 种输出用枚举标识，固定分隔符和 CRLF；格式化返回 `String`，字节边界可确定生成 UTF-16LE。 |
| LT-R12 | 格式化不修改 document；code 排序、缺省权重补齐、同 code 权重稳定排序、聚合格式的相邻重复折叠及短语重编号用独立测试固定。 |
| LT-R13 | 解析失败返回结构化 kind 及 1 起 line/column，不 panic、不静默截断、不返回部分 document；格式化失败不伪造文本位置。 |
| LT-R14 | 合成测试覆盖四编码、六方言、微软/极点分支、七输出、权重归一化、空白不对称回归、错误位置和资源上限；不得依赖 `resource/`。 |
| LT-R15 | decode 结果携带有序诊断。每个未匹配非空行产生一条结构化 warning，保留 1 起行号和最多 160 个 Unicode scalar 的预览；已识别格式中的非法 code/weight/空文本仍立即失败，未知 `%xx` 按 LT-R10 保持字面值，非空正文零有效条目不得伪装为成功。 |

## Acceptance Criteria

- [ ] UTF-8（有/无 BOM）、UTF-16LE/BE BOM 和 GBK 样本均解码为同一 document，返回正确编码/BOM 元数据；非法序列给出字节偏移。
- [ ] YAML、注释、`[Text]`、微软分支和极点清理按固定顺序生效，且对每个删除/保留分支有定向断言。
- [ ] A..F 每种行格式均有独立样本，混合文档按优先级产生预期 code/text/weight/顺序/重复项。
- [ ] D 权重反转与 5000 平移保留相对顺序，零、溢出和缺省权重边界有结构化断言。
- [ ] 7 种格式的完整字符串与手写预期字节级一致，包括 CRLF、分隔符、聚合去重、缺省权重和短语重编号。
- [ ] 包含空格、Tab、CR、LF、VT、FF 的词条在格式化后重新解析完全一致，并有旧 `%0B/%0C` 不对称的失败到通过回归。
- [ ] 空文档、重复词条、等权项、最大权重、展开条目边界和错误路径均不 panic，且位置/上限证据准确。
- [ ] 未匹配非空行按源顺序返回可见 warning 且不阻断其他合法条目；已匹配格式的损坏字段精确失败，非空正文零有效条目返回错误而非空码表。
- [ ] 自动化测试不读取、修改或提交 `resource/`；真实文本 fixture 与七格式旧产物对照留给 `s0-fixtures-regressions`。
- [ ] `cargo fmt`、严格 Clippy、codec/workspace 测试、Rustdoc、依赖审计和 `git diff --check` 通过，production source 无 `unwrap()` / `expect()` / unsafe / lossy 文本解码。

## Risks And Deferred Items

- 无 BOM 的短纯 ASCII 文本同时是合法 UTF-8 与 GBK，固定归为 UTF-8；探测不能依赖机器 locale。
- 原项目用文本作权重 map key，重复词条会共享权重。新模型是 per-entry 权重，需用稳定排序和独立测试固定不丢重复的改进行为。
- 字面 `%20` 与被转义空格在旧协议中无法区分；本任务只修复文档明确的 `%0B/%0C` 不对称，不擅自引入不兼容的 `%25`。
