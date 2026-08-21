# M3 — 反查拆字

> **模块职责**：由汉字 / 词组反向查出五笔编码，并可视化呈现字根拆分、按键方位与标准笔顺。
> 这是 WubiLex **对普通用户最有感知**的功能，也是学习五笔的核心工具。
>
> **只读模块**：不修改任何系统状态。

## 来源文件

| 文件 | 行数 | 角色 |
|---|---:|---|
| `dlg/spelling.aardio` | 590 | 反查主界面：输入、虚拟键盘、拆字、笔顺、热键 |
| `lib/wubi/table.aardio` | 273 | 字根静态数据：一级简码表、8 套字根歌诀、小鹤音形韵母映射 |
| `lib/wubi/spellingTable.aardio` | 64 | 拆字数据表按需下载与查询 |
| `lib/wubi/fonts.aardio` | 45 | 字根字体按需下载与注册 |

## 界面结构

```
┌──────────────────────────────────────────────────────────┐
│  [输入框: 汉字/拼音]                    [🎤 语音]  ┌────────┐│
│  ┌── 拼音候选下拉 ──┐                              │        ││
│  └──────────────────┘                              │ 手写   ││
│                                                     │ 输入   ││
│      编码显示  QWER , QWERT        [拆字图解 GIF]   │ 区     ││
│                                                     │        ││
│         [拆字字根: 亻田一󰀖]                        └────────┘│
│                                            [下载拆字图库]    │
│  [单键]  ┌─ 虚拟键盘 ────────────────────────────┐          │
│  [字根]  │ Q W E R T │ Y U I O P                 │          │
│  [图  ]  │  A S D F G │ H J K L                  │          │
│          │   Z X C V B │ N M                     │          │
│          └────────────────────────────────────────┘          │
│  字根歌诀 / 提示文字区                                        │
│                            [设置热键] [字根图]               │
└──────────────────────────────────────────────────────────┘
```

---

## 1. 输入（`INPUT`）

| ID | 需求 | P | 来源 | 备注 / 风险 |
|---|---|---|---|---|
| `M3-INPUT-001` | 提供文本输入框，接受汉字、词组或拼音 | P0 | `spelling.aardio:6` | |
| `M3-INPUT-002` | 输入纯英文字母时视为拼音，弹出汉字候选下拉列表 | P1 | `spelling.aardio:355-382` | 本地拼音库转换 |
| `M3-INPUT-003` | 本地拼音库无结果时，回退到在线联想接口获取候选 | P2 | `spelling.aardio:351-367` | **原项目依赖百度 `suggestion.baidu.com`**，见风险 |
| `M3-INPUT-004` | 提供手写输入区，识别结果自动填入输入框 | P2 | `spelling.aardio:9, 415-436` | 原项目用 `InkEd.InkEdit` ActiveX，**Tauri 无法直接嵌入**，见风险 |
| `M3-INPUT-005` | 提供语音输入按钮，触发系统语音听写（Win + H） | P2 | `spelling.aardio:577-581` | |
| `M3-INPUT-006` | 连续重复输入同一汉字时归一为单字（支持重复练习同一个字） | P1 | `spelling.aardio:407-410` | 用户反复敲同一字加深记忆 |
| `M3-INPUT-007` | 输入变化触发反查前做 100 ms 防抖 | P0 | `spelling.aardio:79, 340` | |
| `M3-INPUT-008` | 通过全局热键或托盘菜单唤起时，自动聚焦输入框 | P0 | `spelling.aardio:583-585`、`main.aardio:144, 265, 288` | 事件 `spellingFocus` |
| `M3-INPUT-009` | 输入清空时重置全部展示区（编码、拆字、笔顺图、虚拟键盘） | P0 | `spelling.aardio:393-405` | |

### ⚠️ `M3-INPUT-003` 第三方接口依赖

原项目直接调用 `http://suggestion.baidu.com/su?cb=&wd={0}`（`spelling.aardio:353`）。问题：

- 明文 HTTP，把用户输入内容发往第三方
- 无隐私声明
- 接口非公开契约，随时可能失效

**新实现要求**：默认**关闭**在线联想，改为纯本地拼音库（`pinyin` crate 覆盖足够）。若保留在线兜底，必须：HTTPS + 设置项默认关闭 + 首次开启时明确告知会上传输入内容。

### ⚠️ `M3-INPUT-004` 手写输入的替代方案

`InkEd.InkEdit` 是 Windows 的 ActiveX 手写控件，无法嵌入 Tauri 的 WebView2。可选方案：

| 方案 | 代价 |
|---|---|
| A. 调起系统触摸键盘的手写面板（`TabTip.exe`） | 体验割裂，用户需手工切换回本应用 |
| B. Canvas 手写 + 调用 Windows.UI.Input.Inking API 识别 | 需 WinRT 互操作，工作量中等 |
| C. **砍掉手写，保留语音输入**（建议） | 功能减项；但手写在桌面端使用率极低 |

建议采用 **C**，把 `M3-INPUT-004` 降为 P2 并默认不实现，视用户反馈再补。

---

## 2. 编码反查（`QUERY`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M3-QUERY-001` | 在当前系统码表中反查输入文本的全部编码 | P0 | `spelling.aardio:130`、`lexFile.aardio:454-466` | 依赖 [`M1-PARSE-017`](./M1-lex-table.md)；须走倒排索引 |
| `M3-QUERY-002` | 码表中查不到时，回退到系统短语库反查 | P1 | `spelling.aardio:131-133`、`phrase.aardio:106-115` | 依赖 [`M2-IO-008`](./M2-phrase.md) |
| `M3-QUERY-003` | 展示**最短编码**（主）与**最长编码**（次），格式 `QWER , QWERT` | P0 | `spelling.aardio:136, 164-166` | 编码按长度降序排列后取首尾 |
| `M3-QUERY-004` | 无任何编码时展示「缺少编码」 | P0 | `spelling.aardio:168-170` | |
| `M3-QUERY-005` | 展示输入文本的拼音 | P1 | `spelling.aardio:129` | |
| `M3-QUERY-006` | 系统码表变更时重新加载码表、重新探测方案、刷新全部展示 | P0 | `spelling.aardio:444-507` | 订阅 `system.lex.changed` |
| `M3-QUERY-007` | 展示当前系统码表方案名，并提示「第二码起可用 Z 键通配」 | P1 | `spelling.aardio:451-452` | |
| `M3-QUERY-008` | 系统短语库变更时重新加载短语映射表 | P1 | `spelling.aardio:440-442` | 订阅 `system.phrase.changed` |
| `M3-QUERY-009` | 首次进入页面时展示「正在加载字根数据库…」，加载完成前禁用页签 | P1 | `spelling.aardio:509-525` | 原项目用 5 帧动画 |

---

## 3. 虚拟键盘（`KBD`）

26 个键帽按 QWERTY 布局排列，实时反映当前码表方案的字根分布与本次反查的按键序列。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M3-KBD-001` | 展示 26 键 QWERTY 三排布局的虚拟键盘 | P1 | `spelling.aardio:10-35` | |
| `M3-KBD-002` | 每个键帽显示「字母 + 该键首字根」 | P1 | `spelling.aardio:494-501` | 数据源 `wubi.table[版本][键]` 的首字符 |
| `M3-KBD-003` | 当输入命中**一级简码**时，键帽改为显示对应的一级简码字 | P1 | `spelling.aardio:92-127` | 提示文字切换为「一级简码」 |
| `M3-KBD-004` | 反查出编码后，按 ①②③④ 顺序**逐键动画高亮**，间隔 500 ms | P1 | `spelling.aardio:144-161` | 仅编码长度 ≤ 4 时执行 |
| `M3-KBD-005` | 鼠标悬停某键时，在提示区展示该键的**完整字根歌诀** | P1 | `spelling.aardio:478-489` | 数据源 `wubi.table[版本][键]` |
| `M3-KBD-006` | 鼠标悬停某键时，同步展示该键的**单键字根图** | P1 | `spelling.aardio:56-74, 486` | 按需下载 `etymon/{版本}/{键}.gif` |
| `M3-KBD-007` | 键帽按五笔**五区**分色 | P1 | `spelling.aardio:10-35`（`bgcolor`） | 见下方配色表 |
| `M3-KBD-008` | 小鹤音形方案下，键帽附加显示该键的**韵母映射** | P2 | `spelling.aardio:503`、`table.aardio:245-274` | |
| `M3-KBD-009` | `Z` 键无背景色（通配键，不属任何区） | P1 | `spelling.aardio:35` | |

### 五区配色

原项目的键帽背景色按五笔的五个笔画区划分。RGB 值由 aardio COLORREF（`0x00BBGGRR`）换算：

| 区 | 笔画 | 键位 | 原值 | RGB | 观感 |
|---|---|---|---:|---|---|
| 1 区 | 横 | `G F D S A` | `12712703` | `#FFFAC1` | 淡黄 |
| 2 区 | 竖 | `H J K L M` | `14085594` | `#DAEDD6` | 淡绿 |
| 3 区 | 撇 | `T R E W Q` | `16509893` | `#C5EBFB` | 淡蓝 |
| 4 区 | 捺 | `Y U I O P` | `15853297` | `#F1E6F1` | 淡紫 |
| 5 区 | 折 | `N B V C X` | `14085631` | `#FFEDD6` | 淡橙 |
| — | 通配 | `Z` | 无 | 透明 | — |

> 这套配色**承载教学语义**（帮助记忆五笔分区），新 UI 重设计时必须保留分区区分度，仅可调整具体色值以适配深色主题。

### 详述：一级简码

一级简码是各键位对应的最高频汉字，单击一个字母键即可输入。原项目内置 6 套一级简码表（`table.aardio:4-34`）：

| 方案 | 一级简码 |
|---|---|
| `86` / `98` / `06` | Q我 W人 E有 R的 T和 Y主 U产 I不 O为 P这 A工 S要 D在 F地 G一 H上 J是 K中 L国 X经 C以 V发 B了 N民 M同 |
| `091` | Q我 W他 E有 R的 T和 Y就 U将 I小 O为 P着 A或 S要 D在 F都 G不 H上 J是 K中 L里 X给 C对 V她 B了 N已 M由 |
| `092` | Q与 W他 E有 R我 T和 Y就 U里 I小 O那 P着 A从 S中 D在 F你 G不 H上 J是 K的 L为 X来 C这 V她 B了 N以 M也 |
| `zhengma` | Q月 W这 E世 R多 T度 Y了 U为 I上 O个 P所 A一 S说 D的 F要 G在 H成 J中 K是 L用 X对 C现 V没 B地 N他 M我 **Z发** |
| `xhyx` | Q去 W我 E二 R人 T他 Y一 U是 I出 O哦 P平 A啊 S三 D的 F非 G个 H和 J就 K可 L了 X小 C才 V这 B不 N你 M没 **Z在** |

> 郑码与小鹤音形的 `Z` 键也有一级简码（不作通配用）。
> 表形码（`bxm`）**无**一级简码表，键帽退回显示字根。

**命中判定**（`spelling.aardio:92-103`）：若一级简码表未在缓存中，则按字母 `A`–`Z` 逐个取该字母编码在系统码表中的首个词条动态构建；随后判断输入文本是否出现在该表中。

---

## 4. 拆字（`SPLIT`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M3-SPLIT-001` | 从拆字数据表查询单字的字根序列并展示 | P1 | `spelling.aardio:194-196`、`spellingTable.aardio:19-36` | 数据格式见 [`01-data-formats.md#6`](../01-data-formats.md#6-拆字数据表) |
| `M3-SPLIT-002` | 词组拆字：先查整词，查不到则按字长规则组合各字字根 | P1 | `spelling.aardio:198-229` | 规则见下表 |
| `M3-SPLIT-003` | 郑码 / 表形码：直接查表，不做词组组合 | P2 | `spelling.aardio:176-180` | |
| `M3-SPLIT-004` | 小鹤音形：仅单字查表；字根数 > 1 时另在图区显示「首根 + 末根」 | P2 | `spelling.aardio:181-193` | |
| `M3-SPLIT-005` | 拆字数据表按需从服务器下载并缓存到本地 | P1 | `spellingTable.aardio:6-18` | 见 [M6](./M6-resource-sync.md) |
| `M3-SPLIT-006` | 数据表按码表方案分文件，切换方案时切换数据源 | P1 | `spellingTable.aardio:39-55` | 7 个版本各一份 |
| `M3-SPLIT-007` | 数据表缺失时静默降级（不显示拆字），不阻断编码反查 | P1 | `spellingTable.aardio:20` | |

### 词组拆字组合规则

记 `R[i][j]` 为第 `i` 个字字根序列的第 `j` 个字根：

| 词长 | 取根规则 | 前置条件 |
|---:|---|---|
| 2 | `R1[1] R1[2] R2[1] R2[2]` | 两字各 ≥ 2 根 |
| 3 | `R1[1] R2[1] R3[1] R3[2]` | 字 3 ≥ 2 根 |
| 4 | `R1[1] R2[1] R3[1] R4[1]` | 各字 ≥ 1 根 |
| >4 | `R1[1] R2[1] R3[1] R末[1]` | 各字 ≥ 1 根 |

任一字在数据表中缺失 → 整词不显示拆字（`spelling.aardio:207`）。

> 该规则与 [M1 造词](./M1-lex-table.md#详述三种造词规则对比) 的五笔规则**同构**（都是「一二一二 / 一一一二 / 一一一一 / 一一一末」），但作用对象不同：造词取的是**编码位**，拆字取的是**字根**。新实现可抽象为同一个「取位策略」函数。

---

## 5. 笔顺动画（`ANIM`）

展示标准书写笔顺的 GIF 动画。原项目彩蛋：动画播放结束后点击可重播。

| ID | 需求 | P | 来源 | 备注 / 风险 |
|---|---|---|---|---|
| `M3-ANIM-001` | 单字：展示该字的笔顺动画 GIF | P1 | `spelling.aardio:306-337` | |
| `M3-ANIM-002` | 本地无缓存时按需从服务器下载单字 GIF（文件名为字符的十六进制编码） | P1 | `spelling.aardio:314-336` | URL `download/spelling/{版本}/{hex}.gif` |
| `M3-ANIM-003` | 当前方案无该字 GIF 时，依次回退到 86 版、98 版 | P1 | `spelling.aardio:238-248, 326-332` | |
| `M3-ANIM-004` | 词组：把各字 GIF **横向拼接**为一张图 | P1 | `spelling.aardio:250-305` | 见下方布局表；需 GIF 帧级合成 |
| `M3-ANIM-005` | 提供「下载拆字详解图库」按钮，整包下载 `spelling.tar.lzma` 并解压 | P1 | `spelling.aardio:530-543` | 带进度条与取消，见 [M6](./M6-resource-sync.md) |
| `M3-ANIM-006` | 仅当本地缺少 86/98 图库**且**当前方案为 86/98/06 时显示下载按钮 | P1 | `spelling.aardio:454-461` | |
| `M3-ANIM-007` | 091 / 092 方案不展示笔顺图 | P1 | `spelling.aardio:231-235` | 上游无对应图库 |
| `M3-ANIM-008` | 动画播放完毕后可点击重播 | P2 | README 第 19 行 | |

### 词组 GIF 拼接布局

以首字 GIF 的宽度 `W` 为基准，其余字按比例叠加绘制：

| 词长 | 布局 |
|---:|---|
| 2 | 字1 在 `x=0`，字2 在 `x=W/2` |
| 3 | 字1 在 `x=0`，字2 在 `x=W/4`，字3 在 `x=W/2` |
| ≥4 | 字1 `x=0`，字2 `x=W/4`，字3 `x=W/2`，**末字** `x=3W/4` |

任一字 GIF 缺失 → 整词不展示（`spelling.aardio:256, 270, 288`）。

> **Rust 实现注意**：原项目用 GDI+ 把多张 GIF 的**首帧**绘制到一张位图上（`graphics.drawImage`），实际得到的是**静态拼接图**而非多字同步动画。新实现若要做真动画，需用 `image` + `gif` crate 逐帧合成，工作量显著上升。建议先对齐原行为（静态拼接），动画拼接列为后续增强。

---

## 6. 字根字体（`FONT`）

拆字结果中包含 Unicode 私用区（PUA）字符，标准字体无法渲染。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M3-FONT-001` | 按需下载字根字体并注册到应用进程，供拆字展示使用 | P1 | `fonts.aardio:7-40` | `wubi-lex-etymon-v5.otf` |
| `M3-FONT-002` | 092 方案使用专用字体 `wubi-lex-etymon-092.otf` | P2 | `fonts.aardio:8-11` | |
| `M3-FONT-003` | 字体下载失败时降级为默认字体，PUA 字符显示为缺字符号但不崩溃 | P1 | `fonts.aardio:31-33` | |

> **Tauri 特有约束**：原项目用 `fonts.addFamily` 做进程级字体注册（不装到系统）。Tauri 前端需通过 CSS `@font-face` 加载 —— 应把字体文件通过自定义协议或 base64 data URL 提供给 WebView，**不要**要求用户安装系统字体。

字体来源标注：基于 <https://github.com/yanhuacuo/qingg> 增删修改（`fonts.aardio:43`）。

---

## 7. 全局热键（`HOTKEY`）

反查功能的核心交互：五笔输入过程中忘了某字怎么打，按热键即时查询，再按热键返回原窗口继续输入。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M3-HOTKEY-001` | 注册全局热键唤起反查页并聚焦输入框，默认 `Ctrl + F2` | P0 | `main.aardio:271-294`、`config.aardio:13-15` | |
| `M3-HOTKEY-002` | 若当前前台窗口已是本应用，则按热键**最小化到托盘**（返回原窗口） | P0 | `main.aardio:279-283` | 「不影响继续输入」的关键 |
| `M3-HOTKEY-003` | 提供热键设置控件，允许用户自定义组合键 | P1 | `spelling.aardio:8, 545-565` | |
| `M3-HOTKEY-004` | 热键配置持久化，变更后立即重新注册 | P1 | `spelling.aardio:556-561`、`main.aardio:271-294` | 事件 `spellingHotkeyChange` |
| `M3-HOTKEY-005` | 托盘菜单展示当前反查热键组合 | P1 | `main.aardio:124-145` | |
| `M3-HOTKEY-006` | 提供「字根图」按钮，跳转到 [M5](./M5-etymon-help.md) 的字根图页 | P1 | `spelling.aardio:567-575` | 事件 `showEtymon` |

### 热键注册的健壮性要求（新增）

原项目直接调用 `reghotkey` 且**不检查返回值**。若热键已被其他程序占用（`Ctrl+F2` 在部分 IDE 中被占用），注册静默失败，用户无从得知。

**新实现要求**：注册失败时在设置界面明确提示「该热键已被占用，请更换」，并保留上一个可用热键。

---

## 8. 数据依赖

| 依赖 | 来源 | 说明 |
|---|---|---|
| 系统码表 | [M1](./M1-lex-table.md) | `M3-QUERY-001` 反查主数据源，需倒排索引 |
| 码表方案探测 | [M1](./M1-lex-table.md) | 决定字根表、拆字表、GIF 目录、字体的选择 |
| 系统短语映射表 | [M2](./M2-phrase.md) | `M3-QUERY-002` 回退数据源 |
| 字根静态数据 | 本模块（`table.aardio`） | 8 套字根歌诀 + 6 套一级简码 + 小鹤韵母映射 |
| 拆字数据表 | [M6](./M6-resource-sync.md) | 7 个版本，按需下载 |
| 笔顺 GIF | [M6](./M6-resource-sync.md) | 单字按需 + 整包下载 |
| 单键字根图 | [M6](./M6-resource-sync.md) | `etymon/{版本}/{键}.gif` |
| 字根字体 | [M6](./M6-resource-sync.md) | 2 个 OTF |
| 全局热键注册 | [M7](./M7-app-shell.md) | |
| 配置持久化 | [M7](./M7-app-shell.md) | `hotkey.spelling` |

---

## 9. 对外接口草案

### Tauri Commands

| Command | 用途 |
|---|---|
| `spelling_query` | 输入文本 → `{ codes: [短码, 长码], pinyin, split: [字根], level_one: bool }` |
| `spelling_keyboard` | 返回当前方案的 26 键字根数据（首字根 + 完整歌诀 + 分区 + 韵母） |
| `spelling_stroke_gif` | 返回笔顺 GIF（本地缓存或触发下载），词组返回拼接图 |
| `spelling_etymon_key_image` | 返回单键字根图 |
| `spelling_download_gif_pack` | 触发整包下载 |
| `spelling_pinyin_suggest` | 拼音 → 汉字候选 |
| `hotkey_get` / `hotkey_set` | 反查热键读写 |

### Events

| Event | 载荷 |
|---|---|
| `spelling://focus` | 无（唤起并聚焦输入框） |
| `spelling://resource-ready` | `{ kind: "split-table" \| "font" \| "gif", version }` |
| `spelling://progress` | `{ task_id, percent, message }` |

---

## 10. 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 反查全表线性扫描导致输入卡顿 | **高** | 倒排索引，见 [`M1-PARSE-017`](./M1-lex-table.md#-性能红线) |
| 手写输入 ActiveX 无法在 Tauri 复现 | **高** | 建议砍掉，或用 WinRT Inking API；见 `M3-INPUT-004` |
| 百度联想接口：隐私 + 明文 HTTP + 非公开契约 | **高** | 默认关闭，改用本地拼音库；见 `M3-INPUT-003` |
| 词组 GIF 拼接需帧级合成 | 中 | 一期对齐原行为（静态首帧拼接） |
| PUA 字根字体在 WebView 中的加载 | 中 | 自定义协议提供字体文件，`@font-face` 注册 |
| 上游资源服务器不可用则拆字/笔顺全部失效 | 中 | 图库随安装包内置或提供离线包，见 [M6](./M6-resource-sync.md) |
| 全局热键被占用且静默失败 | 中 | 检查注册返回值并提示 |
| 一级简码表动态构建依赖码表内容，异常码表可能构建出错误表 | 低 | 优先用内置静态表，动态构建仅作兜底 |

---

## 11. 源码索引

本模块的实现几乎全部集中在 `dlg/spelling.aardio` 一个文件里。完整反向索引见 [`03-source-index.md`](../03-source-index.md)。

| 域 | 核心逻辑 | 备注 |
|---|---|---|
| `INPUT` | `dlg/spelling.aardio:375-412`（输入分派）、`344-373`（拼音候选 + 百度接口）、`415-436`（InkEdit 手写）、`577-581`（语音） | `351-373` 与 `415-436` 均**不迁移** |
| `QUERY` | `dlg/spelling.aardio:79-170`（反查主体）；`lib/wubi/lexFile.aardio:454-466`（`spelling()`）；`lib/wubi/phrase.aardio:106-115`（短语回退） | 两处查询都需改倒排索引 |
| `KBD` | `dlg/spelling.aardio:4-48`（**26 键帽 + 五区配色**）、`92-127`（一级简码）、`136-166`（**按键动画**）、`477-505`（键帽内容与 hover）；`lib/wubi/table.aardio:4-274`（**字根数据**） | 五区配色见窗体定义的 `bgcolor` |
| `SPLIT` | `dlg/spelling.aardio:175-236`（含**词组组合规则** `198-229`）；`lib/wubi/spellingTable.aardio:19-55` | |
| `ANIM` | `dlg/spelling.aardio:238-248`（路径与回退）、`250-305`（**GIF 拼接**）、`306-337`（单字下载）、`530-543`（整包下载） | 拼接是 GDI+ 首帧合成，非真动画 |
| `FONT` | `lib/wubi/fonts.aardio:7-40` | |
| `HOTKEY` | `dlg/spelling.aardio:545-565`（录制与保存）、`567-575`（字根图跳转）；`main.aardio:271-294`（注册） | 自定义能力已扩展为 [`M7-KEYMAP-*`](./M7-app-shell.md#42-快捷键自定义keymap) |

### ⚠️ 不要照抄的位置

| 位置 | 问题 | 对应需求 |
|---|---|---|
| `dlg/spelling.aardio:351-373` | 直连 `suggestion.baidu.com`，明文上传用户输入 | `M3-INPUT-003` |
| `dlg/spelling.aardio:415-436` | `InkEd.InkEdit` ActiveX，Tauri 无法嵌入 | `M3-INPUT-004` |
| `lib/wubi/lexFile.aardio:454-466` | 全表 O(n·m) 扫描 | `M3-QUERY-001` |
| `lib/wubi/spellingTable.aardio:20` | 数据表缺失时静默返回 `null` | `M3-SPLIT-007` |

---

## 需求统计

| 域 | 条目数 | P0 | P1 | P2 |
|---|---:|---:|---:|---:|
| `INPUT` | 9 | 4 | 2 | 3 |
| `QUERY` | 9 | 4 | 5 | 0 |
| `KBD` | 9 | 0 | 8 | 1 |
| `SPLIT` | 7 | 0 | 5 | 2 |
| `ANIM` | 8 | 0 | 7 | 1 |
| `FONT` | 3 | 0 | 2 | 1 |
| `HOTKEY` | 6 | 2 | 4 | 0 |
| **合计** | **51** | **10** | **33** | **8** |
