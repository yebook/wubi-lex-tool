# S0 真实夹具与回归

## Goal

为已经完成的 `wubilex-codec` 建立可复现、不可静默跳过的真实回归层：从固定 HTTPS 来源获取八种方案的代表性 `.lex.lzma`，校验压缩与解压字节完整性，验证真实 `.lex` 字节往返和方案探测，并用属性、跨格式和损坏输入测试把 codec 覆盖率提升并实测到至少 90%，为下一子任务接入 CI 提供稳定命令。

## Background

- `.lex`、EUDP、码表文本、短语文本、词频、拆字和八方案探测均已有合成合同测试；当前 codec 共 117 项测试通过。
- `docs/02-architecture.md` 固定由 `cargo xtask fixtures` 把真实码表拉到 `crates/wubilex-codec/tests/fixtures/`，二进制不入库，测试不得依赖开发机隐式样本。
- 旧版 `wubi-lex/lib/app/lexNetContents.aardio` 列出八类在线码表。2026-08-24 只读探测确认 `wubi.aardio.com` 的对应静态文件均可通过 HTTPS 访问；旧目录内的 HTTP URL 不得照搬。
- 首选最小或常规样本为 `ChsWubi86.min`、`ChsWubi98.min`、`06.min`、`091`、`092wb`、`zhengma6.6`、`xhyx.min` 和 `bxm.wei`。HEAD 响应显示八份压缩文件合计约 4.3 MiB，其中 091 约 2.2 MiB。
- 根目录 `resource/` 已被 `.gitignore` 排除，只能作为用户本机人工核对材料；自动化、manifest 和测试不得读取或依赖它。
- 上游 `aardio/wubi-lex` 代码仓库使用 MIT License，但在线目录没有为每份第三方码表提供独立许可证或不可变版本 URL。本任务不再分发二进制，只提交来源、归属说明和固定哈希。
- 当前机器未安装 `cargo-llvm-cov`，也没有可调用的 aardio 编译器/运行时。实现阶段必须显式建立或记录这两个验证前提，不能把未执行检查描述为通过。

## In Scope

1. 建立包含八份代表性码表的可审计 fixture manifest：方案、显示名、固定 HTTPS URL、压缩大小、压缩 SHA-256、解压 SHA-256、目标文件名、预期探测结果、来源和许可证说明。
2. 实现 `cargo xtask fixtures`，支持按 manifest 下载、HTTPS/重定向约束、临时文件、压缩哈希校验、LZMA-alone 有界解压、解压哈希校验、原子落盘、缓存复用和离线 `--check`。
3. 忽略下载后的 `.lex` / `.lex.lzma`，但提交 manifest、README 和目录级 ignore 规则；缺失或损坏 fixture 必须给出可操作错误，测试门禁不得把缺失当成功。
4. 对八份真实 `.lex` 逐一执行严格 decode、预期方案探测和 `decode -> encode` 字节级一致断言；真实输入暴露的兼容差异必须先定位，不能降级为 lossy 或语义-only 通过。
5. 增加 `.lex`、EUDP、空白转义和适用文本格式的属性测试，生成受模型合同约束的数据并固定缩减后的失败种子。
6. 增加跨 codec 端到端测试：真实 `.lex` 到七种规范文本输出及可适用的语义回读，短语文本到 EUDP 再回短语文本，覆盖 emoji、多行、`$[...]`、时间变量和重复项。
7. 增加有界任意字节、截断和结构损坏的 no-panic/错误回归，补齐覆盖报告识别出的真实分支缺口，不写只为命中行数的无行为断言。
8. 明确固定 S0 三项缺陷回归：小写 `xfxy`、六种空白转义对称性，以及移除 `codeWeight[0]` 后 Microsoft/通用文本分支仍保留合法记录。
9. 使用 `cargo llvm-cov` 对 `wubilex-codec` 业务源测量 line coverage，达到至少 90%，并产出下一任务可直接接入 CI 的稳定命令。

## Out Of Scope

- 把八份二进制 fixture、根目录 `resource/` 或临时覆盖率产物提交到 Git。
- 实现通用产品下载器、缓存、镜像设置、断点续传或 `wubilex-resource` 的 LZMA/archive API；这里的下载和解压只服务于仓库 `xtask fixtures`。
- 在本任务内创建 GitHub Actions、`cargo-deny`、bindings/docs CI 闸门；由后续 `s0-xtask-ci` 接入已稳定命令。
- 获取或运行 aardio 编译器、生成独立 legacy golden，或声称七种文本格式已与真实 aardio 进程产物比较；该独立证据留给 `s0-integration`，本任务使用既有手写逐字节合同和真实码表语义投影。
- EUDP 拖放分发、郑码造词重复和 `unique()` 循环三个 S4 缺陷。
- 将上游未声明的第三方码表许可证推断为 MIT，或把按需测试下载视为产品再分发授权。
- 为达到覆盖数字修改生产行为、放宽严格解析或保留部分损坏文档。

## Requirements

| ID | Requirement |
|---|---|
| FR-R01 | fixture manifest 必须覆盖 86、98、06、091、092、郑码、小鹤音形和表形码各一份，使用固定 HTTPS URL，并同时固定压缩与解压 SHA-256；实现不得信任目录实时返回的可变内容。 |
| FR-R02 | `cargo xtask fixtures` 必须在任何落盘前校验状态、大小和哈希，拒绝 HTTP 及降级重定向，使用同目录临时文件并在完整成功后替换目标；失败不得留下被测试误用的半成品。 |
| FR-R03 | LZMA-alone 解压必须有压缩和解压大小上限，算术和写入使用 checked 路径；xtask 错误必须指出 fixture 名、阶段和预期/实际证据，但不泄露凭据。 |
| FR-R04 | 已存在 fixture 只有在解压 SHA-256 匹配时才可复用；`--check` 不访问网络，只验证八个本地目标并在缺失或损坏时失败。 |
| FR-R05 | 自动化 fixture 测试必须通过 manifest 明确找到文件；缺失不得静默 skip。快速合成测试与真实 fixture 门禁可分命令执行，但最终 S0 门禁必须显式运行两者。 |
| FR-R06 | 每个真实 `.lex` 必须严格 decode，`detect::scheme` 等于 manifest 预期，随后规范 encode 与原始解压字节完全一致；权重、顺序、重复项和非 BMP 文本不得丢失。 |
| FR-R07 | 属性测试必须生成模型合同内的有效值并验证规范化后的不变量；无效字节测试必须限制规模与 case 数，失败可复现且不得 panic、OOM 或无限运行。 |
| FR-R08 | 七种码表文本格式使用完整手写期望和真实文档投影验证确定性、CRLF、排序、权重、相邻去重和候选规则；不能用生产 formatter 自己生成同一断言的 golden。 |
| FR-R09 | 短语跨格式回归必须验证 `phrase_text -> PhraseDocument -> EUDP -> PhraseDocument -> phrase_text` 的规范语义，使用固定 timestamp，并覆盖数组、多行、变量、空白、emoji、候选缺口和重复项。 |
| FR-R10 | `xfxy` 与空白转义继续由现有失败到通过测试固定；新增 `codeWeight[0]` 回归必须证明该无效旧分支被省略后，匹配其前后的合法文本方言均被保留。 |
| FR-R11 | 覆盖率门禁只统计 `wubilex-codec` 业务源，line coverage 必须 `>= 90%`；排除第三方、生成物、fixture、测试源码和平台壳，未安装工具或未运行真实 fixture 时不得报告达标。 |
| FR-R12 | 新增测试依赖和 xtask 依赖必须职责最小、版本固定、许可证可接受；不得把 Tokio、Tauri、Windows 或网络依赖引入 `wubilex-codec` 生产依赖。 |
| FR-R13 | 所有自动化命令使用全局 pnpm；不得使用 `volta pnpm`、corepack、npm、yarn 或 npx，也不得读取根目录 `resource/`。 |

## Acceptance Criteria

- [ ] 八个 HTTPS fixture 均能从空缓存通过一条 `cargo xtask fixtures` 命令获取，有效缓存重复执行不下载，篡改任一压缩或解压文件会被哈希校验拒绝。
- [ ] fixture 目录只提交 manifest、说明与 ignore 规则；Git 状态不出现下载二进制、临时文件或覆盖率报告。
- [ ] 八份真实 `.lex` 均通过严格 decode、正确方案探测和字节级 re-encode；结果表记录文件大小、条目数和 SHA-256。
- [ ] 七种码表文本格式的确定性与语义投影通过，短语文本/EUDP 端到端往返通过，且测试期望不由待测 formatter 自生成。
- [ ] 属性测试覆盖 `.lex`、EUDP、转义及适用文本合同；代表性任意/截断/损坏输入不会 panic，失败种子可重放。
- [ ] S0 三项 defect regression 均有明确测试名和失败原因：`xfxy`、`%0B/%0C` 对称性、`codeWeight[0]` 死分支省略。
- [ ] `cargo llvm-cov` 实测 `wubilex-codec` line coverage 至少 90%，报告中的低覆盖分支已经用行为测试补齐或有明确不可达解释。
- [ ] `cargo fmt`、严格 Clippy、codec/workspace tests、Rustdoc、全局 pnpm、Trellis validation、anchors、lockfile/依赖审计和 `git diff --check` 通过。
- [ ] 自动化未读取根目录 `resource/`，未引入 codec 的平台、异步、网络或文件系统生产依赖。

## Risks And Deferred Items

- 上游静态 URL 没有版本号且目录不发布哈希；manifest 固定哈希可检测漂移，但无法阻止上游替换。发生漂移时必须人工审查后更新，不能自动信任新字节。
- 在线码表的逐项许可证不明确。本任务按需下载且不提交二进制，并记录来源；产品再分发需要在 S5 资源目录任务中单独解决。
- 真实 091 样本明显大于其他七份，属性与真实测试必须分层，避免每个随机 case 重复解析大文件。
- `cargo-llvm-cov` 缺失需要实现阶段用可审计方式提供本地验证工具；CI 安装与缓存仍由 `s0-xtask-ci` 负责。
- 七种文本格式在本任务只获得手写逐字节合同和真实码表语义投影证据；独立 aardio 运行时/golden 对照由 `s0-integration` 关闭，不能在本任务的完成报告中扩大表述。
- 真实 EUDP 样本仍不可复现获取，本任务只能做合成端到端 wire 回归；Windows 实际系统兼容保留给后续隔离集成。
