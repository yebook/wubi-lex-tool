# Design — WubiLex 需求文档集

本任务的「技术设计」是**文档集自身的结构设计**：文档边界、编号契约、可追溯性机制。

## 1. 文档集架构

```
docs/
├── README.md                    索引 + 阅读路径 + 文档维护约定
├── 00-overview.md               总览：背景 / 目标 / 原项目现状 / 模块地图 / 术语 / 优先级定义
├── 01-data-formats.md           数据格式规格（跨模块共享的技术契约）
├── 02-architecture.md           Rust+Tauri 架构映射 + 依赖选型 + 风险登记册
├── modules/
│   ├── M1-lex-table.md          码表管理
│   ├── M2-phrase.md             短语词库
│   ├── M3-reverse-lookup.md     反查拆字
│   ├── M4-ime-control.md        输入法控制与系统设置
│   ├── M5-etymon-help.md        字根图与帮助资料
│   ├── M6-resource-sync.md      资源分发与网络同步
│   └── M7-app-shell.md          应用外壳
├── 20-nonfunctional.md          非功能需求
├── 21-ui-ux.md                  UI/UX 需求
└── 22-roadmap.md                优先级汇总 + 里程碑
```

### 模块边界的划分依据

原项目的目录结构（`dlg/` 界面 + `lib/` 逻辑）是 **UI 驱动**的，不适合直接搬为新项目的模块边界。本文档集改按**领域能力**切分：

| 模块 | 领域职责 | 原项目对应文件 |
|---|---|---|
| M1 | 码表的解析、编辑、变换、安装 | `lib/wubi/lexFile.aardio`、`dlg/dict/lex.aardio`、`dlg/dict/wordWeight.aardio`、`lib/wubi/weightData.aardio`、`lib/wubi/text.aardio`、`lib/app/lexContents.aardio` |
| M2 | 短语词库的解析、编辑、安装 | `lib/wubi/phrase.aardio`、`dlg/dict/phrase.aardio`、`dlg/dict/phraseHelp.aardio` |
| M3 | 编码反查、拆字、笔顺、虚拟键盘 | `dlg/spelling.aardio`、`lib/wubi/spellingTable.aardio`、`lib/wubi/table.aardio`、`lib/wubi/fonts.aardio` |
| M4 | 输入法启停、双拼、注册表设置、TSF 重启 | `lib/tsfInput.aardio`、`lib/tsfUtil.aardio`、`dlg/setting.aardio`、`lib/ui/doublePinyinMenu.aardio`、`lib/winex/msCandidate.aardio` |
| M5 | 字根图、帮助文本、关于与更新入口 | `dlg/help/*.aardio`、`lib/wubi/table.aardio`（字根歌诀数据） |
| M6 | 在线目录、下载、解压、缓存 | `lib/app/lexNetContents.aardio`、`lib/app/lexContents.aardio`（下载分支）、各模块的资源下载逻辑 |
| M7 | 单实例、窗口、托盘、热键、配置、事件总线 | `main.aardio`、`lib/config.aardio`、`lib/style.aardio`、`dlg/dict/dict.aardio`、`dlg/help/help.aardio` |

`lib/wubi/table.aardio` 同时被 M3（虚拟键盘字根）与 M5（字根歌诀展示）使用 —— 数据定义放在 `01-data-formats.md`，两个模块各自引用，不重复正文。

### 为什么把数据格式单独成文

`.lex` 与 EUDP 的二进制布局是 M1/M2/M3 共同依赖的**跨模块契约**，且是原项目最有价值、最难重新推导的技术资产。放入独立文档可以：
- 避免在三个模块文档里重复描述
- 让 Rust 实现方把它当作 codec crate 的验收规格直接使用
- 与需求条目解耦：格式变更与功能变更走不同的评审路径

## 2. 需求 ID 契约

格式：`<模块>-<域>-<三位序号>`，例如 `M1-PARSE-004`、`M4-REG-011`。

- 模块段：`M1`..`M7`
- 域段：模块内的功能分组缩写（如 M1 的 `LIST` / `PARSE` / `EDIT` / `XFORM` / `SLIM` / `WEIGHT` / `COIN` / `SPLIT` / `INSTALL` / `IO`）
- 序号：模块内域段下**永不复用**，删除的需求保留 ID 并标注「已废弃」

ID 一经写入即冻结，`22-roadmap.md` 的汇总表以 ID 为主键做交叉引用。

## 3. 需求条目结构

每条需求统一为表格行，列固定为：

| 列 | 含义 |
|---|---|
| ID | 唯一标识 |
| 需求 | 一句话可验收描述（动宾结构） |
| P | P0 / P1 / P2 |
| 来源 | 原项目文件（+ 函数名或行号） |
| 备注 / 风险 | 边界条件、Rust 侧实现难点、依赖 |

复杂需求（如码表文本方言解析）在表格下方补充「详述」小节，用有序列表描述算法步骤。

## 4. 优先级判定规则

| 级别 | 判据 |
|---|---|
| **P0** | 缺失则产品不成立。覆盖：码表解析/编辑/安装主链路、短语读写、反查基础编码查询、输入法启停、应用外壳最小闭环 |
| **P1** | 显著影响体验但可延后。覆盖：格式转换、精简、词频优化、造词、笔顺动画、字根图、双拼、注册表高级设置、在线资源目录 |
| **P2** | 锦上添花或依赖外部不可控资源。覆盖：手写输入、语音输入、百度联想 API、Windows 使用技巧文本、ImTip 跳转、鼠标左右键反转 |

已废弃项单列 **P3（不实现）**，仅作历史记录。

## 5. 可追溯性机制

- 每条需求的「来源」列必须指向 `wubi-lex/` 下的真实文件；行为描述以源码实际逻辑为准，不采信 README 的宣传措辞（README 与代码冲突时以代码为准，并在备注中记录冲突）。
- `00-overview.md` 提供一张**源文件覆盖表**：32 个 `.aardio` 文件 × 归属模块，确保零遗漏。
- 三个未接线/已废弃文件在覆盖表中显式标 P3，不作为遗漏处理。

## 6. 兼容性与回滚

文档集为纯新增，不修改 `wubi-lex/` 任何文件，不影响现有仓库状态。回滚 = 删除 `docs/` 目录。

## 7. 本设计不做的事

- 不定义 Rust 模块的具体函数签名（属实现期决策，架构文档只到 crate 与 command 边界）
- 不产出 UI 视觉稿，`21-ui-ux.md` 只到信息架构与交互规范层
- 不评估工作量人天，`22-roadmap.md` 只做里程碑排序与依赖关系
