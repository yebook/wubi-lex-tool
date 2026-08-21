# 02 — Rust + Tauri 架构映射

> 新项目 **WubiLexTool** 的分层边界、crate 划分、依赖选型，以及 aardio 能力 → Rust 的逐项映射表与**全项目风险登记册**。
>
> 不定义具体函数签名 —— 那是实现期决策。本文档只到 crate 与 command 边界。

## 项目标识

| 项 | 值 |
|---|---|
| 产品名 | **WubiLexTool** |
| 可执行文件 | `WubiLexTool.exe` |
| Crate 前缀 | `wubilex-` |
| 应用数据目录 | `%LOCALAPPDATA%\WubiLexTool\` |
| 配置目录 | `%APPDATA%\WubiLexTool\` |

## 目录

0. [**首要原则：旧项目是行为规格，不是实现范本**](#0-首要原则旧项目是行为规格不是实现范本)
1. [架构总览](#1-架构总览)
2. [Crate 划分](#2-crate-划分)
3. [Tauri 层设计](#3-tauri-层设计)
4. [aardio → Rust 能力映射](#4-aardio--rust-能力映射)
5. [依赖选型](#5-依赖选型)
6. [前端技术栈](#6-前端技术栈)
7. [关键设计决策](#7-关键设计决策)
8. [测试策略](#8-测试策略)
9. [目录结构建议](#9-目录结构建议)
10. [风险登记册](#10-风险登记册)

---

## 0. 首要原则：旧项目是**行为规格**，不是实现范本

> 这一条优先于本文档其余所有内容。

WubiLex 原项目在文档集中的角色只有一个：**说明「正确的行为是什么」**。它**不说明「该怎么写」**。

- 各模块文档「来源」列指向的代码，是**行为的权威定义**，不是要移植的实现
- [`03-source-index.md`](./03-source-index.md) 是**行为查证工具**，不是移植清单
- 遇到更优的实现方式，**直接用更优的**，不必与原实现保持形似
- 原项目的实现细节（数据结构、并发模型、错误处理、UI）**没有一处**具有被继承的正当性 —— 它们是 aardio 与 2021 年桌面开发条件下的产物

### 0.1 什么必须一致（行为契约）

改了会导致**用户数据出错**或**系统不工作**的部分。这些必须逐位对齐，且有测试固化。

| # | 契约 | 为什么不能改 | 规格出处 |
|---:|---|---|---|
| C1 | `.lex` 二进制布局 | 微软五笔 IME 直接读这个文件 | [`01#1`](./01-data-formats.md#1-lex-微软五笔码表二进制) |
| C2 | EUDP 二进制布局 | 同上 | [`01#2`](./01-data-formats.md#2-eudp-用户短语库二进制) |
| C3 | 注册表键名与取值语义 | 微软 IME 读这些键 | [`M4-REG-*`](./modules/M4-ime-control.md#3-五笔行为设置reg) |
| C4 | TSF 停机窗口的必要步骤 | 系统约束：不停服务就改不了文件 | [`M4-TSF-*`](./modules/M4-ime-control.md#5-tsf-重启与-acl-接管tsf) |
| C5 | 文本码表 6 种方言的解析规则 | 用户手上现有的码表文件按这些规则写成 | [`01#3`](./01-data-formats.md#3-文本码表方言) |
| C6 | 短语文本方言与 `$[...]` 语义 | 同上 | [`01#4`](./01-data-formats.md#4-短语文本方言) |
| C7 | 三种造词规则的取码位置 | 生成的编码要与其他五笔工具一致，否则用户换工具就乱套 | [`M1-COIN`](./modules/M1-lex-table.md#详述三种造词规则对比) |
| C8 | 键名占用的判定启发式 | 判错会让词条进错位置，导致候选排序异常 | [`M1-SPLIT-001`](./modules/M1-lex-table.md#详述键名占用的判定规则) |
| C9 | 版本探测的特征编码 | 判错会加载错误的字根表与拆字表 | [`01#7`](./01-data-formats.md#7-码表版本探测算法) |
| C10 | 空白字符转义映射 | 用户码表里存在这些转义序列 | [`01#8`](./01-data-formats.md#8-空白字符转义) |
| C11 | 五笔五区分组、字根歌诀、一级简码 | 这是**五笔规范**，不是原项目的发明 | [`M5 附录 A`](./modules/M5-etymon-help.md#3-字根歌诀数据附录-a) |
| C12 | 词频文件与拆字数据表格式 | 与上游资源兼容 | [`01#5`](./01-data-formats.md#5-词频文件)、[`01#6`](./01-data-formats.md#6-拆字数据表) |

**C8、C9 需要特别说明**：这两处的原实现质量不高（C8 是拍脑袋的经验规则，C9 有明确 bug）。保留它们**不是因为实现好，而是因为它们的输出定义了行为**。做法是：原样实现判定逻辑 + 修掉确认的 bug + 写测试固化，并在代码注释中标明其经验性质。

### 0.2 什么自由改进（实现细节）

| 维度 | 说明 |
|---|---|
| 数据结构与算法 | 随便换，只要输出符合契约 |
| 并发与任务模型 | 原项目的裸线程模型无参考价值 |
| 错误处理 | 原项目此处是**反面教材**，不要看 |
| UI 的全部 | 见 [`21-ui-ux.md`](./21-ui-ux.md) —— 已按「全新设计」重写 |
| 配置格式与存储位置 | |
| 缓存策略与目录组织 | |
| 网络层、重试、校验 | |
| 日志与可观测性 | 原项目完全没有 |
| 资源打包格式 | LZMA → zstd |
| 系统调用方式 | 命令行工具 → Win32/COM API |
| 测试 | 原项目无测试 |

### 0.3 已识别的更优实现

以下偏离已在本文档集中定案，实现时**按新做法**，不要回头照原项目写。

| # | 原项目做法 | 新做法 | 收益 |
|---:|---|---|---|
| 1 | 反查全表 O(n·m) 线性扫描 | 倒排索引 | 数百 ms → O(1)，见 [`M1-PARSE-017`](./modules/M1-lex-table.md#-性能红线) |
| 2 | 每次操作「文本 → 解析 → 变换 → 序列化 → 文本」全量往返 | 内存模型常驻 + 句柄式访问 | 省掉每次操作的解析与序列化，见 [D1](#d1码表常驻内存--句柄式访问) |
| 3 | 单个 Win32 `edit` 装数十万行 | 虚拟滚动表格 + 分页 command | 加载数分钟 → ≤ 5 s，见 [`UX-BIGDATA-*`](./21-ui-ux.md#7-超大数据量策略bigdata) |
| 4 | 调 `takeown` / `icacls` / `schtasks` 并解析文本输出 | `SetNamedSecurityInfo`、服务 API、Task Scheduler COM | 不依赖系统语言、错误码明确、无子进程开销 |
| 5 | 先删后写，无备份 | 备份 → 写入 → 校验 → 失败回滚 | 失败不丢数据 |
| 6 | 停机窗口无失败恢复 | RAII 守卫 + 崩溃后自恢复 | 不留系统中间态 |
| 7 | `eval()` 反序列化目录缓存 | `serde` 严格反序列化 | 消除 RCE 链路 |
| 8 | 明文 HTTP、无校验、更新无签名 | HTTPS + SHA-256 + minisign | |
| 9 | LZMA alone | zstd（新增资源） | 解压快 3–5 倍 |
| 10 | 业务逻辑写在 UI 回调里 | 分层 crate，逻辑层可脱离 UI 单测 | 可测试、可复用 |
| 11 | `thread.invoke` 裸线程，不可取消 | 任务注册表 + `CancellationToken` + 进度事件 | 可取消、可查询 |
| 12 | 5 帧图标动画当进度指示 | 真实百分比 + 阶段文字 + 剩余估算 | |
| 13 | 大量静默失败 | 结构化错误（阶段 + 错误码 + 可读描述 + 技术细节） | |
| 14 | 硬编码，仅 1 个热键可改 | 动作注册表 + 全量可改绑 | 见 [`M7-KEYMAP`](./modules/M7-app-shell.md#42-快捷键自定义keymap) |
| 15 | FontAwesome 码点硬编码 60+ 处 | 组件化图标库 | |
| 16 | 固定像素坐标布局 | 响应式 | |
| 17 | 缓存与用户数据混放同一目录 | 目录分离 | 清缓存不误删用户码表 |
| 18 | 逐行串行处理 | `rayon` 并行分片 | |
| 19 | 仅靠静态词频文件补空码 | 追加自学习（[M8](./modules/M8-self-learning.md)） | 覆盖个人高频词 |
| 20 | 无任何健康检查 | 概览页状态卡片 + 异常告警条 | 见 [`UX-SCREEN-006`](./21-ui-ux.md#51-概览新增) |
| 21 | 双拼方案串硬编码在代码里 | 作为配置数据管理 | 便于补充其他方案 |
| 22 | 短语只能整段文本编辑 | 表格 + 实时 `$[...]` 预览 | |
| 23 | 无日志、无诊断导出 | `tracing` 结构化日志 + 诊断导出 | |
| 24 | 无测试 | codec 层覆盖率 ≥ 90% + 真实码表回归集 | |

### 0.4 遇到未列出的分歧怎么办

```
这段原实现看起来不好，我能改吗？
  │
  ├─ 改了会影响 §0.1 表里的某条契约吗？
  │     ├─ 会  → 不能改行为。可以改实现方式，但输出必须逐位一致，并加测试
  │     └─ 不会 → 随便改，用你认为最好的方式
  │
  └─ 不确定算不算契约？
        → 判据：改了之后，用户现有的码表 / 短语 / 系统配置会出错吗？
          会 → 是契约。不会 → 不是。
```

---

## 1. 架构总览

```
┌──────────────────────────────────────────────────────────────┐
│                       前端 (WebView2)                         │
│      React + TypeScript + Tailwind CSS + Lucide              │
│  路由 · 虚拟滚动 · 状态管理 · 主题 · i18n · 占位状态            │
└─────────────────────────────┬────────────────────────────────┘
                     Tauri IPC │ commands / events
┌─────────────────────────────▼────────────────────────────────┐
│                    wubilex-app  (src-tauri)                   │
│   command 处理 · 事件广播 · 全局状态 · 任务调度 · 恢复守卫       │
└─┬──────────┬──────────┬───────────┬───────────┬──────────────┘
  │          │          │           │           │
┌─▼────────┐┌▼─────────┐┌▼─────────┐┌▼─────────┐┌▼────────────┐
│ wubilex- ││ wubilex- ││ wubilex- ││ wubilex- ││  wubilex-   │
│  codec   ││   core   ││  winime  ││ resource ││    learn    │
│          ││          ││          ││          ││             │
│.lex 编解码││码表模型   ││TSF/注册表 ││HTTP/解压  ││分词/新词发现 │
│EUDP 编解码││变换/精简  ││服务/ACL   ││缓存/校验  ││采集/计数/入库│
│文本方言   ││造词/索引  ││进程/计划  ││镜像/离线  ││    (M8)     │
│版本探测   ││统计/排序  ││          ││          ││             │
│          ││          ││          ││          ││             │
│  纯逻辑   ││  纯逻辑   ││Windows 专有││I/O + 网络││ 逻辑 + UIA  │
│  可单测   ││  可单测   ││ 需集成测试 ││  可 mock ││   可单测    │
└──────────┘└──────────┘└──────────┘└──────────┘└─────────────┘
```

### 分层原则

| 层 | 允许依赖 | 禁止 |
|---|---|---|
| `wubilex-codec` | 无平台依赖，仅标准库 + 解析/编码 crate | 文件系统之外的 I/O、Windows API、Tauri |
| `wubilex-core` | `wubilex-codec` | Windows API、Tauri、网络 |
| `wubilex-winime` | `windows` crate | Tauri、业务逻辑 |
| `wubilex-resource` | HTTP / 压缩 crate | Tauri、业务逻辑 |
| `wubilex-learn` | `wubilex-core`、分词 crate、UIA | Tauri、直接写系统文件（经 `core` 走短语库） |
| `wubilex-app` | 全部 | 把领域逻辑写在 command 里 |

**这套分层的核心目的**是修正原项目最大的结构性缺陷：**业务逻辑写在 UI 回调里**（`dlg/dict/lex.aardio` 的 1,455 行中，造词、格式转换、精简、词频优化全部内嵌在菜单回调中，无法测试也无法复用）。

> 实现任何一层时，用 [`03-source-index.md`](./03-source-index.md) 反查原项目对应代码位置。

---

## 2. Crate 划分

### 2.1 `wubilex-codec` — 编解码

**职责**：字节 ↔ 内存模型的双向转换。无状态、纯函数、零平台依赖。

| 能力 | 对应文档 |
|---|---|
| `.lex` 二进制读写 | [`01-data-formats.md#1`](./01-data-formats.md#1-lex-微软五笔码表二进制) |
| EUDP 二进制读写 | [`01-data-formats.md#2`](./01-data-formats.md#2-eudp-用户短语库二进制) |
| 文本码表方言解析（6 种行格式 + 微软码表分支） | [`01-data-formats.md#3`](./01-data-formats.md#3-文本码表方言) |
| 文本码表 7 种输出格式 | [`01-data-formats.md#3.5`](./01-data-formats.md#35-输出格式矩阵) |
| 短语文本方言解析与输出 | [`01-data-formats.md#4`](./01-data-formats.md#4-短语文本方言) |
| 词频文件读写 | [`01-data-formats.md#5`](./01-data-formats.md#5-词频文件) |
| 拆字数据表解析 | [`01-data-formats.md#6`](./01-data-formats.md#6-拆字数据表) |
| 码表版本探测 | [`01-data-formats.md#7`](./01-data-formats.md#7-码表版本探测算法) |
| 空白字符转义 | [`01-data-formats.md#8`](./01-data-formats.md#8-空白字符转义) |
| CSV / JSON 容器 | [`M1-PARSE-009/010`](./modules/M1-lex-table.md) |

**这是全项目最应该被测试覆盖的 crate** —— 目标覆盖率 90%+，含往返测试与真实码表回归集。

### 2.2 `wubilex-core` — 领域逻辑

**职责**：码表与短语的内存模型及其上的全部操作。

| 能力 | 对应需求 |
|---|---|
| 码表模型 + **倒排索引** | [`M1-PARSE-017`](./modules/M1-lex-table.md#-性能红线) |
| 合并 / 排序 / 统计 | `M1-PARSE-014/015/018` |
| 格式转换（9 项） | `M1-XFORM-*` |
| 精简（7 项） | `M1-SLIM-*` |
| 词频与权重优化（9 项） | `M1-WEIGHT-*` |
| 造词（3 种规则 + 空码造词） | `M1-COIN-*` |
| 短语分离 | `M1-SPLIT-*` |
| 拆字组合规则 | `M3-SPLIT-002` |
| 简繁转换 / GB2312 判定 / 拼音 | `M1-XFORM-008/009`、`M1-SLIM-004`、`M3-QUERY-005` |

### 2.3 `wubilex-winime` — Windows 输入法集成

**职责**：所有与 Windows 输入法子系统的交互。这是**唯一**允许直接调用 Win32/COM 的业务 crate。

| 能力 | 对应需求 |
|---|---|
| TIP 启停与状态查询 | `M4-TIP-*` |
| 双拼方案管理 | `M4-DPY-*` |
| 注册表设置读写 | `M4-REG-*` |
| **TSF 停机窗口编排 + RAII 恢复守卫** | `M4-TSF-*` |
| 服务控制 / 计划任务 / 进程终止 | `M4-TSF-002/003/004` |
| 文件所有权与 ACL 接管 | `M4-TSF-007` |
| 系统设置页跳转 / 开机自启 | `M4-SYS-*` |

**必须提供 dry-run 模式**：不实际修改系统，仅记录将执行的操作序列。用于开发调试与自动化测试。

### 2.4 `wubilex-resource` — 资源分发

**职责**：网络获取、解压、校验、缓存。

| 能力 | 对应需求 |
|---|---|
| 在线目录拉取与缓存 | `M6-CATALOG-*` |
| 下载（进度 / 取消 / 重试 / 断点续传） | `M6-DOWN-*` |
| LZMA / TAR 解压（含路径穿越防护） | `M6-ARCHIVE-*` |
| SHA-256 校验 | `M6-DOWN-011` |
| 缓存目录管理与清理 | `M6-CACHE-*` |
| 镜像源与离线包 | `M6-DOWN-013`、`M6-CACHE-005` |

网络层需可 mock（trait 抽象 HTTP 客户端），便于离线测试。

### 2.5 `wubilex-learn` — 自学习

**职责**：[M8](./modules/M8-self-learning.md) 的分词、新词发现、频次统计与入库决策。

| 能力 | 对应需求 |
|---|---|
| 语料导入与文本提取 | `M8-CORPUS-*` |
| 输入采集（UI Automation） | `M8-CAPTURE-*` |
| 中文分词 + N-gram 新词发现 | `M8-LEARN-001` |
| 频次计数 / 阈值判定 / 时间衰减 | `M8-LEARN-002..006` |
| 造词与入库决策 | `M8-APPLY-001..003` |
| 候选池持久化与管理 | `M8-MANAGE-*` |

**分层约束**：本 crate **不直接写系统文件**。入库时把结果交给 `wubilex-core`，由其经短语库路径写入（见 [`M8-APPLY-004`](./modules/M8-self-learning.md#为什么走短语库而不是系统码表)）。

采集部分（UIA 交互）应与学习逻辑分离为两个模块，使学习逻辑可脱离 Windows 单测。

### 2.6 `wubilex-app` — Tauri 应用

**职责**：把上述能力暴露为 command / event，管理全局状态与任务生命周期。

**禁止**在此 crate 写领域逻辑。command 函数应当是薄适配层：参数反序列化 → 调用下层 → 结果序列化。

---

## 3. Tauri 层设计

### 3.1 全局状态

```rust
struct AppState {
    config: RwLock<Config>,                              // M7-CONF
    keymap: RwLock<KeyMap>,                              // M7-KEYMAP
    lex_sessions: RwLock<HashMap<SessionId, LexTable>>,  // 已打开的码表（含倒排索引）
    system_lex: RwLock<Option<SystemLexInfo>>,           // 系统码表快照（方案、路径、统计）
    phrase_map: RwLock<Option<PhraseMap>>,               // 系统短语映射
    tasks: TaskRegistry,                                 // M7-TASK
    resources: ResourceManager,                          // M6
    learn: RwLock<LearnStore>,                           // M8 候选池与频次
    features: FeatureFlags,                              // 占位状态开关，见 UX-INTERACT-013
}
```

> **`lex_sessions` 是关键设计**：原项目每次操作都做「文本 → 解析 → 变换 → 序列化 → 文本」全量往返，是其卡顿的根因。新设计把码表**常驻内存**为句柄，所有变换在内存模型上原地进行，仅在需要展示或落盘时序列化。

### 3.2 Command 命名约定

`<模块前缀>_<动作>`，模块前缀对应文档模块：

| 前缀 | 模块 |
|---|---|
| `lex_` | [M1](./modules/M1-lex-table.md) |
| `phrase_` | [M2](./modules/M2-phrase.md) |
| `spelling_` | [M3](./modules/M3-reverse-lookup.md) |
| `ime_` / `dpy_` / `wubi_settings_` / `tsf_` | [M4](./modules/M4-ime-control.md) |
| `etymon_` / `help_` / `about_` | [M5](./modules/M5-etymon-help.md) |
| `resource_` / `update_` | [M6](./modules/M6-resource-sync.md) |
| `app_` / `config_` / `window_` / `hotkey_` / `keymap_` / `task_` | [M7](./modules/M7-app-shell.md) |
| `learn_` | [M8](./modules/M8-self-learning.md) |

各模块的 command 清单见对应模块文档的「对外接口草案」章节。

### 3.3 统一错误模型

```rust
#[derive(Serialize)]
struct AppError {
    kind: ErrorKind,        // Io | Parse | Network | Permission | System | Validation | Cancelled
    module: &'static str,   // "M1" | "M4" | ...
    message: String,        // 面向用户的可读描述（中文）
    detail: Option<String>, // 技术细节：系统错误码、行号、路径
    recoverable: bool,      // 是否可重试
}
```

**这是对原项目的关键改进**。原项目的错误处理模式是 `return null, "错误信息"`，且大量调用点直接丢弃错误信息，最终用户只能看到「安装失败请重试一次即可」。

**要求**：

- 每个 command 返回 `Result<T, AppError>`
- 任何 `unwrap()` / `expect()` 在生产路径上都是缺陷
- 系统 API 失败必须携带 `GetLastError()` 的错误码与文本

### 3.4 事件命名约定

`<域>://<事件名>`，如 `lex://progress`、`tsf://phase`。完整清单见 [`M7-BUS`](./modules/M7-app-shell.md#6-事件总线bus)。

### 3.5 长任务模式

```
前端 invoke("lex_transform", { session, op })
  → 后端立即返回 { task_id }
  → 后端 spawn_blocking 执行
  → 持续 emit "task://progress" { task_id, percent, message }
  → 完成时 emit "task://done" { task_id, result | error }
前端可 invoke("task_cancel", { task_id })
```

**取消语义**：通过 `CancellationToken` 在循环中协作式检查。[TSF 停机窗口内不可取消](./modules/M7-app-shell.md#需要可取消的长任务)。

---

## 4. aardio → Rust 能力映射

### 4.1 UI 与外壳

| aardio | 用途 | Rust / 前端方案 |
|---|---|---|
| `win.ui` / `win.form` | 窗口与控件 | Tauri window + WebView2 前端 |
| `win.ui.tabs` | Tab 容器 | 前端路由 |
| `win.util.tray` | 系统托盘 | `tauri::tray::TrayIcon` |
| `win.ui.popmenu` | 弹出菜单 | `tauri::menu::Menu`（支持 checkitem / submenu） |
| `win.ui.atom` | 单实例 | `tauri-plugin-single-instance` |
| `mainForm.reghotkey` | 全局热键 | `tauri-plugin-global-shortcut` 或直接 `RegisterHotKey` |
| `win.ui.accelerator` | 应用内快捷键 | 前端 keydown 监听 |
| `win.ui.tooltip` | 工具提示 | 前端组件 |
| `win.inputBox` / `fsys.dlg` | 输入框 / 文件对话框 | `tauri-plugin-dialog` |
| `win.dlg.message` | 消息框 | 前端组件（更可控）或 `tauri-plugin-dialog` |
| `publish` / `subscribe` | 事件总线 | Tauri `emit` / `listen` |
| `thread.invoke` | 后台线程 | `tokio::task::spawn_blocking` |
| `win.invoke` | 消息泵内执行 | 主线程 `run_on_main_thread` |
| `win.debounce` | 防抖 | 前端 debounce |
| `fsys.config` | 配置持久化 | `tauri-plugin-store` 或 `serde` + TOML |
| `fsys.update.simpleMain` | 自更新 | `tauri-plugin-updater`（内置签名验证） |
| `process.emptyWorkingSet` | 释放工作集 | `SetProcessWorkingSetSize`（效果待评估） |

### 4.2 文本与编码

| aardio | 用途 | Rust crate |
|---|---|---|
| `fsys.codepage.load` | 编码自动探测 | `chardetng` 探测 + `encoding_rs` 解码 |
| `string.toUnicode` / `readUnicode` | UTF-16 转换 | `widestring` / `String::from_utf16` |
| `string.conv.simplized` / `traditionalized` | 简繁转换 | `opencc-rust`，或内置转换表 |
| `string.conv.isGb2312` | GB2312 可编码判定 | `encoding_rs` GBK 编码尝试（无替换字符即可编码） |
| `string.conv.pinyin` | 汉字转拼音 | `pinyin` crate |
| `string.csv` | CSV | `csv` crate |
| `web.json` | JSON | `serde_json` |
| `string.match` / `replace`（正则） | 正则 | `regex` + `once_cell::Lazy` 预编译 |
| `string.len`（字符数） | Unicode 字符计数 | `.chars().count()`，注意与 UTF-16 单元数的区别 |
| `ustring.indexOf` | Unicode 感知查找 | `str::find` + 字符边界处理 |

> ⚠️ **UTF-16 vs 字符数**：原项目的 `.lex` / EUDP 格式以 **UTF-16 code unit** 计长，而 `string.len` 返回**字符数**。含 emoji（代理对）时两者不同。新实现必须区分，见 [`M2-PARSE-008`](./modules/M2-phrase.md)。

### 4.3 Windows 系统

| aardio | 用途 | Rust 方案 |
|---|---|---|
| `win.reg` | 注册表 | `windows::Win32::System::Registry` 或 `winreg` |
| `sys.input.disable` / `getEnabledLayoutOrTips` | 输入法启停与枚举 | WinRT `Windows.Globalization` + `Win32::UI::Input::KeyboardAndMouse` |
| `com.interface.ITfInputProcessorProfileMgr` | TSF Profile 激活 | `windows` crate，feature `Win32_UI_TextServices` |
| `service.manager` | 服务控制 | `windows::Win32::System::Services`（`OpenSCManager` / `ControlService` / `ChangeServiceConfig`） |
| `process.popen("schtasks …")` | 计划任务 | Task Scheduler 2.0 COM（`ITaskService`） |
| `sys.runAsTask` | 开机自启任务注册 | 同上 |
| `process.kill` | 进程终止 | `ToolHelp32Snapshot` + `OpenProcess` + `TerminateProcess` |
| `process.each` | 进程枚举 | 同上 |
| `fsys.acl.takeOwn` / `icacls` | 所有权与 ACL | `windows::Win32::Security::Authorization::SetNamedSecurityInfo` |
| `win.rt.bcp47` | 用户语言列表 | WinRT `Windows.System.UserProfile.GlobalizationPreferences` |
| `key.ime.changeRequest` | 请求输入法切换 | `PostMessage(HWND_BROADCAST, WM_INPUTLANGCHANGEREQUEST, …)` |
| `::User32.SystemParametersInfo` | 系统参数 | `windows::Win32::UI::WindowsAndMessaging::SystemParametersInfoW` |
| `::User32.SwapMouseButton` | 鼠标键反转 | 同上命名空间 |
| `process.control("ms-settings:…")` | 打开系统设置页 | `ShellExecuteW` |
| `winex.find` / `findEx` | 窗口查找 | `FindWindowExW` |
| `win.sendCopyData` | 跨进程消息 | `SendMessageW` + `WM_COPYDATA` |
| `process.admin.enableDropMsg` | UIPI 拖放放行 | `ChangeWindowMessageFilterEx` |

> **强烈建议全部改用 API 而非命令行工具**。原项目依赖 `takeown` / `icacls` / `schtasks` 的**文本输出**，在非中文/非英文系统上会失效，且错误码语义模糊。这是其错误处理薄弱的根因之一。

### 4.4 网络与压缩

| aardio | 用途 | Rust crate |
|---|---|---|
| `inet.http` | HTTP 请求 | `reqwest`（`rustls-tls`） |
| `web.rest.jsonLiteClient` | JSON API | `reqwest` + `serde_json` |
| `inet.downBox` | 带进度的下载 UI | `reqwest` 流式 + 前端进度组件 |
| `sevenZip.lzma.decodeFile` | LZMA 解压 | `lzma-rs::lzma_decompress`（**alone 格式，非 xz**） |
| `sevenZip.lzma.httpFile` | 下载并解压 | 组合上述 |
| tar 解包（`spelling.tar.lzma`） | TAR | `tar` crate（须防路径穿越） |

### 4.5 图像与字体

| aardio | 用途 | Rust / 前端方案 |
|---|---|---|
| `gdip.bitmap` / `drawImage` | GIF 拼接 | `image` + `gif` crate；或在前端用 Canvas 合成 |
| `fonts.addFamily` | 进程级字体注册 | 前端 `@font-face` + Tauri 自定义协议提供字体文件 |
| `plus` 控件的 `background` | 图片显示 | 前端 `<img>` + 自定义协议 |

### 4.6 无直接对应

| aardio | 用途 | 处置 |
|---|---|---|
| `InkEd.InkEdit`（ActiveX） | 手写输入 | **无法嵌入 WebView2**。建议移除，见 [`M3-INPUT-004`](./modules/M3-reverse-lookup.md#-m3-input-004-手写输入的替代方案) |
| `eval()` 反序列化 | 目录缓存 | **安全缺陷，必须移除**，改用 `serde_json`，见 [`M6-CATALOG-007`](./modules/M6-resource-sync.md) |

---

## 5. 依赖选型

### 5.1 Rust 侧

| 用途 | Crate | 备注 |
|---|---|---|
| 应用框架 | `tauri` v2 | |
| 单实例 | `tauri-plugin-single-instance` | |
| 全局热键 | `tauri-plugin-global-shortcut` | |
| 对话框 | `tauri-plugin-dialog` | |
| 配置存储 | `tauri-plugin-store` | 或自建 `serde` + TOML |
| 自更新 | `tauri-plugin-updater` | **内置 minisign 签名验证**，解决 `M6-UPDATE-005` |
| 日志 | `tauri-plugin-log` + `tracing` | |
| Windows API | `windows` | 按需启用 feature，避免全量编译 |
| 注册表 | `windows`（`Win32_System_Registry`） | |
| 序列化 | `serde` / `serde_json` | |
| 正则 | `regex` + `once_cell` | 预编译静态正则 |
| 编码探测 | `chardetng` | |
| 编码转换 | `encoding_rs` | UTF-16 / GBK |
| 宽字符 | `widestring` | |
| CSV | `csv` | |
| LZMA | `lzma-rs` | alone 格式 |
| TAR | `tar` | |
| 压缩（新资源） | `zstd` | 建议新分发改用 zstd |
| HTTP | `reqwest`（`rustls-tls`） | 不用 native-tls，减少系统依赖 |
| 哈希 | `sha2` | 资源校验 |
| 并行 | `rayon` | 大码表批量操作 |
| 异步 | `tokio` | Tauri 自带 |
| 错误 | `thiserror` + `anyhow` | 库用 thiserror，应用用 anyhow |
| 拼音 | `pinyin` | |
| 简繁转换 | `opencc-rust` | 或内置表；需评估体积 |
| 图像 | `image` + `gif` | GIF 拼接 |
| 依赖许可清单 | `cargo-about` | 生成 `M5-ABOUT-008` 的声明页 |
| 中文分词 | `jieba-rs` | M8 分词，内置词典约 5 MB |
| UI Automation | `windows`（`Win32_UI_Accessibility`） | M8 输入采集 |
| 本地加密 | `chacha20poly1305` 或 `age` | M8 采集数据加密（`M8-PRIV-003`） |

### 5.2 选型注意

| 项 | 说明 |
|---|---|
| `windows` crate feature | 需要 `Win32_UI_TextServices`、`Win32_System_Services`、`Win32_Security_Authorization`、`Win32_System_Registry`、`Win32_System_Diagnostics_ToolHelp` 等。逐项启用，全量启用会显著拖慢编译 |
| `opencc-rust` | 依赖 C++ 的 OpenCC 库，会增加构建复杂度与体积。若只需「简↔繁」单向映射，考虑内置字符映射表（原项目 `string.conv` 也是表驱动） |
| `lzma-rs` | 只支持解压，不支持压缩。若需要压缩（导出 `.lex.lzma`，`M1-IO-004`），需用 `xz2` 的 raw LZMA 编码器或 `sevenz-rust` |
| `reqwest` | 用 `rustls-tls` 而非 `native-tls`，避免依赖系统 Schannel 配置 |

---

## 6. 前端技术栈

### 6.1 技术栈

**已确定**（用户决策）：

| 层 | 选型 | 说明 |
|---|---|---|
| **样式** | **Tailwind CSS** | 设计令牌通过 `theme.extend` + CSS 变量暴露；`darkMode: 'class'`。约定见 [`21-ui-ux.md`](./21-ui-ux.md#tailwind-约定) |
| **图标** | **Lucide**（可换 Tabler） | 组件化引入，替代原项目的 FontAwesome 码点硬编码 |

**建议**：

| 层 | 选型 | 理由 |
|---|---|---|
| 框架 | React + TypeScript | 生态成熟，虚拟滚动与编辑器组件选择最多 |
| 构建 | Vite | Tauri 默认 |
| 组件库 | shadcn/ui | 无运行时依赖、基于 Tailwind、可完全定制，与已定的样式方案天然契合 |
| 状态 | Zustand | 轻量，适合桌面应用的中等复杂度 |
| 虚拟滚动 | TanStack Virtual | 码表表格视图的核心，见 `M1-EDIT-002` |
| 代码编辑器 | CodeMirror 6 | 「文本模式」编辑器 |
| 路由 | React Router | 一级 + 二级导航 |
| i18n | `i18next` | 见 [`20-nonfunctional.md`](./20-nonfunctional.md) |
| 类名排序 | `prettier-plugin-tailwindcss` | 统一顺序，减少 diff 噪音 |

> Vue / Svelte 同样可行（shadcn/ui 有对应移植）。**关键约束不是框架而是虚拟化能力** —— 必须能流畅渲染数十万行数据。

### 6.2 前端不做的事

- **不做数据变换**：格式转换、精简、造词全部在 Rust 侧完成。前端只发指令、收结果
- **不持有完整码表**：只通过分页 command 获取当前视口所需的数据窗口
- **不解析文件**：文件读取与解析在 Rust 侧

---

## 7. 关键设计决策

### D1：码表常驻内存 + 句柄式访问

**背景**：原项目每次操作都做「文本 ↔ 模型」全量往返，导致完整码表下每个菜单点击都要等待数十秒。

**决策**：`lex_open` 返回 session 句柄，码表模型（含倒排索引）常驻 `AppState`。所有变换在模型上原地进行，前端通过 `lex_query_page` 按需拉取视口数据。

**代价**：内存占用上升（数十万条目约需 50–150 MB）。可接受 —— 桌面应用，且用户通常同时只打开一份码表。

**对应需求**：`M1-EDIT-002`、`M1-PARSE-017`。

### D2：表格视图为主，文本编辑为辅

**背景**：见 [`M1-EDIT-002` 详述](./modules/M1-lex-table.md#详述m1-edit-002-大数据量编辑)。

**决策**：主视图为虚拟滚动表格（编码 / 词条 / 权重三列，支持排序与筛选）；保留「文本模式」供习惯原项目的用户使用，但文本模式下明确提示大码表的性能影响。

**代价**：与原项目的心智模型不同，需要迁移引导。

### D3：TSF 操作用 RAII 守卫

**背景**：原项目的停机窗口无任何失败恢复，崩溃会让输入法永久不可用。

**决策**：把「停服务 / 结束任务 / 接管 ACL」封装为 guard 类型，`Drop` 时无条件执行恢复。配合「进入停机窗口」的持久化标记，实现崩溃后自恢复。

**对应需求**：`M4-TSF-010`、`M7-INST-006`。

### D4：全部写入操作先备份

**背景**：原项目对系统码表与用户短语库都是「先删后写」，中途失败即数据丢失。短语库尤其严重 —— 那是用户手工积累的内容。

**决策**：

- 首次安装前备份「系统原始码表」，永不覆盖
- 每次写入前备份当前状态到 `last.*`
- 写入后校验（魔数 + 条目数），失败自动回滚
- UI 提供手动还原入口

**对应需求**：`M1-INSTALL-013/014`、`M2-INSTALL-010/011`。

### D5：资源全部走 HTTPS + 校验，核心资源内置

**背景**：原项目全链路明文 HTTP、无校验、单点依赖，软件更新包甚至无签名。

**决策**：

- 全部资源 HTTPS + SHA-256
- 更新走 `tauri-plugin-updater`（minisign 签名）
- 86/98 的拆字表、字根图、字根字体**内置于安装包**
- 支持自定义镜像源与离线资源包

**代价**：安装包体积上升（预估 +10–20 MB）。相对于原项目的 786 KB 是显著增长，但换取了断网可用与安全性。

**对应需求**：`M6-DOWN-011`、`M6-UPDATE-005`、`M6-CACHE-005`。

### D6：一期整进程提权，演进到辅助进程

**背景**：见 [`M7-INST-003`](./modules/M7-app-shell.md#m7-inst-003-管理员权限的取舍)。

**决策**：一期沿用原项目方案（整进程 `requireAdministrator`），在非功能需求中记录安全影响。二期拆分为「普通权限主进程 + 按需提权的辅助进程」。

### D7：移除手写输入与在线联想

**决策**：

- 手写输入（`InkEd.InkEdit` ActiveX）无法在 WebView2 复现，移除
- 百度联想接口（明文 HTTP + 上传用户输入 + 非公开契约）默认关闭，改用本地拼音库

**对应需求**：`M3-INPUT-003`、`M3-INPUT-004`。

---

## 8. 测试策略

| 层 | 类型 | 目标 |
|---|---|---|
| `wubilex-codec` | 单元 + 属性测试 | **覆盖率 90%+**。往返测试、6 种方言、8 种版本探测、边界与损坏输入 |
| `wubilex-codec` | 回归集 | 收集真实社区码表（86/98/06/091/092/郑码/小鹤/表形码 各 ≥1 份）建立快照测试 |
| `wubilex-core` | 单元 | 每个变换/精简/词频/造词操作的输入输出断言 |
| `wubilex-winime` | 集成 | **dry-run 模式**断言操作序列；真实执行在 Windows CI 的隔离 VM 中跑 |
| `wubilex-resource` | 单元 | mock HTTP；解压路径穿越防护测试 |
| `wubilex-learn` | 单元 | 分词/新词发现/阈值判定/衰减；采集层与学习层分离后可脱离 Windows 测试 |
| `wubilex-app` | 集成 | command 的序列化契约测试 |
| 前端 | 组件 + E2E | 关键流程：加载码表 → 编辑 → 安装；占位状态在特性开关关闭时正确呈现 |

### 必须覆盖的原项目缺陷回归

| 缺陷 | 出处 | 测试 |
|---|---|---|
| `XFXY` 大写导致版本探测失效 | `lexFile.aardio:1293` | 对新世纪码表断言探测结果为 `06` |
| 短语拖放二进制被文本覆盖 | `dlg/dict/phrase.aardio:130-135` | 拖放 EUDP 文件断言解析正确 |
| 郑码造词分支缺 `else` 导致重复 | `dlg/dict/phrase.aardio:191` | 郑码码表造词断言无重复输出 |
| 空白转义编解码字符集不对称 | `text.aardio:8-14` | 含 `\v` `\f` 的词条往返断言 |
| `codeWeight[0]` 死代码 | `lexFile.aardio:135-137` | 确认省略后行为不变 |
| `unique()` 循环无递增 | `lexFile.aardio:629` | 一码一词精简的正确性断言 |

---

## 9. 目录结构建议

```
WubiLexTool/
├── Cargo.toml                    # workspace
├── crates/
│   ├── wubilex-codec/
│   │   ├── src/
│   │   │   ├── lex/              # .lex 二进制
│   │   │   ├── eudp/             # EUDP 二进制
│   │   │   ├── text/             # 文本方言
│   │   │   ├── weight.rs         # 词频文件
│   │   │   ├── split_table.rs    # 拆字数据表
│   │   │   ├── escape.rs         # 空白转义
│   │   │   └── detect.rs         # 版本探测
│   │   └── tests/
│   │       └── fixtures/         # 真实码表回归集
│   ├── wubilex-core/
│   │   └── src/
│   │       ├── table.rs          # 码表模型 + 倒排索引
│   │       ├── transform/        # 格式转换
│   │       ├── slim/             # 精简
│   │       ├── weight/           # 词频优化
│   │       ├── coin/             # 造词
│   │       ├── split/            # 短语分离
│   │       └── convert/          # 简繁 / 拼音 / GB2312
│   ├── wubilex-winime/
│   │   └── src/
│   │       ├── tip.rs            # 输入法启停
│   │       ├── double_pinyin.rs
│   │       ├── settings.rs       # 注册表
│   │       ├── tsf/              # 停机窗口 + 恢复守卫
│   │       ├── service.rs
│   │       ├── schtask.rs
│   │       ├── acl.rs
│   │       └── dry_run.rs
│   ├── wubilex-resource/
│   │   └── src/
│   │       ├── catalog.rs
│   │       ├── download.rs
│   │       ├── archive.rs
│   │       ├── cache.rs
│   │       └── verify.rs
│   └── wubilex-learn/            # M8
│       └── src/
│           ├── corpus.rs         # 语料导入（路径 A）
│           ├── capture/          # 输入采集（路径 B，UIA）
│           ├── segment.rs        # 分词 + N-gram 新词发现
│           ├── store.rs          # 候选池与频次
│           └── apply.rs          # 造码与入库决策
├── src-tauri/                    # wubilex-app
│   ├── src/
│   │   ├── commands/             # 按模块分文件
│   │   ├── events.rs
│   │   ├── state.rs
│   │   ├── keymap.rs             # 动作注册表 + 绑定解析
│   │   ├── task.rs
│   │   ├── error.rs
│   │   ├── features.rs           # 占位状态特性开关
│   │   └── recovery.rs
│   ├── resources/                # 内置资源（字根图/拆字表/字体）
│   └── tauri.conf.json
├── src/                          # 前端
│   ├── routes/
│   │   ├── lex/                  # M1
│   │   ├── phrase/               # M2
│   │   ├── spelling/             # M3
│   │   ├── settings/             # M4（含 keymap 子页）
│   │   ├── help/                 # M5
│   │   └── learn/                # M8
│   ├── components/
│   │   ├── ui/                   # shadcn/ui
│   │   ├── virtual-table/        # UX-COMP-001
│   │   ├── wubi-keyboard/        # UX-COMP-002
│   │   ├── key-cap/              # UX-COMP-003
│   │   ├── hotkey-recorder/      # UX-COMP-004
│   │   └── feature-placeholder/  # UX-COMP-015
│   ├── stores/
│   ├── styles/
│   │   └── tokens.css            # CSS 变量（Tailwind 配置引用）
│   ├── i18n/
│   └── icons.ts                  # Lucide 统一出口
├── tailwind.config.ts
├── xtask/                        # 构建工具（资源打包、清单生成）
└── docs/                         # 本文档集
```

---

## 10. 风险登记册

全项目风险统一在此维护。模块文档中的风险表引用本表。

### 极高

| # | 风险 | 影响 | 缓解 | 需求 |
|---:|---|---|---|---|
| R1 | TSF 停机窗口中途失败或崩溃，输入法永久不可用 | 用户无法输入中文，需手工修复系统 | RAII 恢复守卫 + 持久化标记 + 启动时自恢复 | `M4-TSF-010`、`M7-INST-006` |
| R2 | 软件自更新无签名验证 | 中间人可投递任意可执行文件 | `tauri-plugin-updater`（minisign） | `M6-UPDATE-005` |
| R3 | 全链路明文 HTTP + 无完整性校验 | 码表/字体/更新包可被篡改 | HTTPS + SHA-256 | `M6-DOWN-011` |

### 高

| # | 风险 | 影响 | 缓解 | 需求 |
|---:|---|---|---|---|
| R4 | 大码表（数十万条）编辑器性能 | UI 冻结数分钟，产品不可用 | 虚拟化表格 + 内存模型原地操作 | `M1-EDIT-002` |
| R5 | 反查全表线性扫描 O(n·m) | 实时输入卡顿 | 倒排索引 | `M1-PARSE-017` |
| R6 | 系统码表写入失败导致丢失 | 五笔完全不可用 | 备份 + 校验 + 回滚 | `M1-INSTALL-013` |
| R7 | 用户短语库写入失败导致丢失 | 手工积累的内容永久丢失 | 同上 | `M2-INSTALL-010` |
| R8 | `TabletInputService` 未恢复 | 触摸键盘/手写面板永久失效 | 恢复守卫无条件恢复 | `M4-TSF-010` |
| R9 | 文件所有权停留在 Administrators | 系统文件权限异常 | 恢复守卫无条件归还 TrustedInstaller | `M4-TSF-007` |
| R10 | 依赖 `takeown`/`icacls`/`schtasks` 的文本输出 | 非中/英文系统上失效 | 改用 Win32 API | `M4-TSF-*` |
| R11 | 目录缓存用 `eval` 反序列化 | 远程代码执行链路 | 严格 JSON 反序列化 | `M6-CATALOG-007` |
| R12 | TAR 解包路径穿越 | 任意文件写入 | 路径规范化 + 拒绝 `..` | `M6-ARCHIVE-006` |
| R13 | 上游资源服务器单点 | 拆字/笔顺/字根图/码表全部失效 | 核心资源内置 + 镜像 + 离线包 | `M6-CACHE-005` |
| R14 | 手写输入 ActiveX 无法在 Tauri 复现 | 功能减项 | 移除，或用 WinRT Inking API | `M3-INPUT-004` |
| R15 | 百度联想接口：隐私 + 明文 + 非公开契约 | 用户输入外泄 | 默认关闭，改本地拼音库 | `M3-INPUT-003` |
| R16 | 整进程管理员权限扩大攻击面 | — | 一期接受并记录；二期拆辅助进程 | `M7-INST-003` |

### 中

| # | 风险 | 缓解 | 需求 |
|---:|---|---|---|
| R17 | 文本方言解析与原项目行为不一致 | 真实码表回归测试集 | `M1-PARSE-002` |
| R18 | 版本探测误判（含 `XFXY` 缺陷） | 修复 + 8 方案单测 | `M1-PARSE-013` |
| R19 | 键名占用启发式规则脆弱 | 原样移植 + 固化测试 | `M1-SPLIT-001` |
| R20 | 空码造词耗时数分钟 | 并行 + 进度 + 取消 | `M1-COIN-005` |
| R21 | 短语文本 6 种方言优先级歧义 | 严格按原顺序 + 逐格式单测 | `M2-PARSE-001` |
| R22 | 词组 GIF 拼接需帧级合成 | 一期对齐原行为（静态首帧） | `M3-ANIM-004` |
| R23 | PUA 字根字体在 WebView 中加载 | 自定义协议 + `@font-face` | `M3-FONT-001` |
| R24 | `ITfInputProcessorProfileMgr` COM 互操作复杂度 | 薄适配层 + 独立验证 | `M4-TIP-008` |
| R25 | 双拼方案串为未文档化格式 | 作为配置数据管理 | `M4-DPY-003` |
| R26 | Windows 版本差异导致注册表键语义变化 | 按版本分档 + 未知键不覆盖 | `M4-REG-*` |
| R27 | 缓存与用户数据混放，清理误删 | 目录分离 | `M6-CACHE-006` |
| R28 | 大量静默失败让用户无从排查 | 功能性资源必须上报 | `M6-DOWN-015` |
| R29 | 长任务不可取消 | 统一任务管理器 | `M7-TASK-003` |
| R30 | 托盘菜单动态状态在 Tauri 下的重建成本 | 弹出时刷新 | `M7-TRAY-004` |
| R31 | 配置文件损坏导致启动失败 | 校验 + 回退 + 备份 | `M7-CONF-004` |
| R32 | WebView2 运行时缺失 | 安装包内置 Bootstrapper | `M7-INST-*` |
| R33 | `opencc-rust` 增加构建复杂度与体积 | 评估内置转换表替代 | `M1-XFORM-008` |
| R34 | UTF-16 单元数与字符数混淆（emoji） | codec 层显式区分 + 测试 | `M2-PARSE-008` |
| R35 | 第三方内容（字体/词库）许可合规 | 补齐声明页 | `M5-ABOUT-008` |

### 低

| # | 风险 | 缓解 |
|---:|---|---|
| R36 | 简繁转换库与 aardio 内置表差异 | 抽取原项目转换结果做对照 |
| R37 | LZMA alone 与 xz 混淆 | 明确用 `lzma_decompress`；新资源改 zstd |
| R38 | 092 方案 `C` 键数据为空 | 保留为空并显示「—」，向上游确认 |
| R39 | 表形码无字根图 | 隐藏该方案的字根图页签 |
| R40 | `enableUsKeyboard` 无条件禁用副作用 | UI 明示 + 保守模式 |
| R41 | `emptyWorkingSet` 在 WebView2 下可能无效 | 评估后决定保留与否 |
| R42 | 一级简码动态构建可能出错 | 优先用内置静态表 |
| R43 | 用户绑定到系统保留组合（`Ctrl+Alt+Del` 等） | 绑定时拒绝并提示，见 `M7-KEYMAP-013` |
| R44 | 占位状态残留到已实现功能 | 占位由构建期特性开关驱动，见 [`UX-INTERACT-013`](./21-ui-ux.md#占位状态规范ux-interact-013) |

### M8 自学习（后置模块，`阶段 S8`）

本模块整体优先级 P2。以下风险在启动该模块开发前不构成阻塞，但**一旦开始实现路径 B（输入采集），R45/R46 即为准入门槛**。

| # | 风险 | 等级 | 缓解 | 需求 |
|---:|---|---|---|---|
| R45 | 输入采集记录用户在所有应用中的输入 | **极高** | 默认关闭 + 显式授权 + 只存分词结果 + 本地加密 + 一键清除；**先只做路径 A（语料导入）** | `M8-PRIV-001..005`、`NFR-PRIV-006` |
| R46 | 密码 / 敏感内容被误采 | **极高** | 密码框检测 + 进程黑名单 + 窗口标题黑名单 | `M8-CAPTURE-003/004/005` |
| R47 | UIA 采集覆盖面不完整，用户预期落空 | 中 | 文档与 UI 明示「尽力而为」，管理界面展示实际采集量 | `M8-CAPTURE-001` |
| R48 | 自动入库污染词库 | 中 | 默认确认模式 + 只占空码位 + 可撤回 | `M8-APPLY-002/003`、`M8-MANAGE-002` |
| R49 | 分词质量差导致垃圾候选 | 中 | jieba + N-gram 新词发现 + 多重过滤 + 人工确认 | `M8-LEARN-001/004/005` |
| R50 | 杀毒软件将输入监听判为可疑行为 | 中 | 代码签名 + 提交白名单；路径 A 不触发此问题 | `NFR-SEC-006` |
| R51 | 频繁写短语库触发输入法反复重载 | 中 | 批量入库 + 周期触发 | `M8-APPLY-005` |
| R52 | 采集拖慢用户输入 | 中 | 独立后台任务 + 事件队列解耦，延迟预算 < 1 ms | `M8-CAPTURE-006` |

---

## 下一步

- 非功能约束见 [`20-nonfunctional.md`](./20-nonfunctional.md)
- 界面设计约束见 [`21-ui-ux.md`](./21-ui-ux.md)
- 排期与里程碑见 [`22-roadmap.md`](./22-roadmap.md)
