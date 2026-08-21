# M2 — 短语词库

> **模块职责**：微软五笔用户自定义短语库（EUDP）的读取、解析、编辑与写入。
> WubiLex 在系统原生能力之上做了显著扩展：兼容多种短语文本方言、支持 `$[...]` 符号数组语法、支持多行短语。
>
> **不做**：不直接操作 TSF / 服务（委托 [M4](./M4-ime-control.md)）。

## 来源文件

| 文件 | 行数 | 角色 |
|---|---:|---|
| `lib/wubi/phrase.aardio` | 439 | 短语引擎核心：EUDP 二进制读写、文本方言解析、默认短语库 |
| `dlg/dict/phrase.aardio` | 286 | 短语编辑界面 |
| `dlg/dict/phraseHelp.aardio` | 22 | 短语格式说明弹窗（纯静态文本） |
| `lib/wubi/text.aardio` | 19 | 空白字符转义（与 M1 共享） |
| `lib/tsfUtil.aardio` | 235 | 安装编排（与 M1、M4 共享） |

## 背景：什么是 EUDP

**EUDP** = End User Defined Phrase，微软输入法的用户自定义短语机制。用户可以在 Windows「设置 → 时间和语言 → 输入法 → 微软五笔 → 用户自定义短语」中手工添加，但系统界面：

- 一次只能加一条，无批量导入
- 不支持 `$[...]` 数组语法
- 编辑后会**固化时间变量**，使其失去动态性

WubiLex 直接读写底层文件绕过了这些限制。

**文件位置**：

```
%APPDATA%\Microsoft\InputMethod\Chs\ChsWubiEUDPv1.lex
%APPDATA%\Microsoft\InputMethod\Chs\ChsWubiEUDPv2.lex
```

完整二进制格式见 [`01-data-formats.md#2`](../01-data-formats.md#2-eudp-用户短语库二进制)。

---

## 1. 读写（`IO`）

| ID | 需求 | P | 来源 | 备注 / 风险 |
|---|---|---|---|---|
| `M2-IO-001` | 定位当前生效的系统短语文件：v1/v2 只存在其一时取存在者，都存在时取**最后写入时间较新**者 | P0 | `phrase.aardio:368-391` | |
| `M2-IO-002` | 解析 EUDP 二进制为「编码 → [(文本, 候选位序, 字符数)]」映射表 | P0 | `phrase.aardio:11-104` | |
| `M2-IO-003` | 将 EUDP 二进制转换为可编辑的文本格式 | P0 | `phrase.aardio:125-176` | 含 `$[...]` 压缩，见 `M2-PARSE-008` |
| `M2-IO-004` | 读写前校验系统版本 ≥ Win10 1703（Build 15063），否则拒绝并给出明确提示 | P0 | `dlg/dict/phrase.aardio:21-30, 64-73` | 低版本无 EUDP 能力 |
| `M2-IO-005` | 将编辑器文本另存为 `.phrase.txt` | P1 | `dlg/dict/phrase.aardio:220-229` | |
| `M2-IO-006` | 从 `.phrase.txt` / `.txt` 加载短语到编辑器 | P1 | `dlg/dict/phrase.aardio:230-236` | |
| `M2-IO-007` | 拖放文件加载：自动识别是 EUDP 二进制（魔数 `mschxudp`）还是文本 | P1 | `dlg/dict/phrase.aardio:108-137, 248-253` | 见下方缺陷说明 |
| `M2-IO-008` | 在短语映射表中按文本反查编码 | P1 | `phrase.aardio:106-115` | [M3](./M3-reverse-lookup.md) 依赖此能力 |
| `M2-IO-009` | 空文件、`count == 0`、损坏偏移表等异常给出可读的错误信息而非崩溃 | P0 | `phrase.aardio:22-57` | 原项目已有 5 类错误分支，须保留 |

### ⚠️ 原项目缺陷：`M2-IO-007` 二进制分支被无条件覆盖

`dlg/dict/phrase.aardio:130-135`：

```aardio
if(file.read(8)=="mschxudp"){
    file.close();
    winform.editPhrase.text = wubi.phrase.load(path)     // ← 解析二进制
}
winform.editPhrase.text = ..fsys.codepage.load(path);    // ← 无条件覆盖！
```

第二行缺少 `else`，导致**二进制短语文件被当作文本重新读取**，前一行的解析结果被丢弃，编辑器显示乱码。

**新实现要求**：正确分支，并补测试。

### ⚠️ `M2-IO-008` 性能

`findCodeFromMap` 是对整个映射表的线性扫描（`phrase.aardio:106-115`）。短语库通常只有数十到数百条，影响可忽略，但 M3 的实时反查会高频调用 —— 建议在加载时同步建立反向索引。

---

## 2. 解析（`PARSE`）

完整方言规格见 [`01-data-formats.md#4`](../01-data-formats.md#4-短语文本方言)。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M2-PARSE-001` | 解析 6 种短语行格式（`P1`–`P6`），按固定优先级依次尝试 | P0 | `phrase.aardio:234-288` | |
| `M2-PARSE-002` | 剥离 `/* ... */` 注释（非贪婪、跨行） | P0 | `phrase.aardio:224` | |
| `M2-PARSE-003` | 支持多行短语：等号后立即换行时，后续行以 `\n` 连接追加 | P1 | `phrase.aardio:258, 271-277` | |
| `M2-PARSE-004` | 解析 `$[...]` 符号数组：含空格按空格分割（多空格归一），不含空格按字符分割 | P1 | `phrase.aardio:292-301` | **WubiLex 独有扩展**，系统界面不支持 |
| `M2-PARSE-005` | 未指定候选位序时按「该编码已用最大序号 + 1」自动分配 | P0 | `phrase.aardio:302-303` | |
| `M2-PARSE-006` | 将 `$year` / `$month` 等 9 个别名映射为微软的 `%yyyy%` / `%M%` 等时间变量 | P1 | `phrase.aardio:244-254` | 变量由输入法在上屏时求值，本项目只原样写入 |
| `M2-PARSE-007` | 写入前解码空白转义 `%20` `%09` `%0D` `%0A` | P0 | `phrase.aardio:351` | |
| `M2-PARSE-008` | EUDP → 文本时，若某编码的全部候选序号连续且每条长度 ≤ 2 个 UTF-16 单元，压缩输出为 `$[...]` | P1 | `phrase.aardio:146-162` | 长度阈值 2 是为了兼容 emoji 代理对 |
| `M2-PARSE-009` | 读取时跳过 `deleted != 0` 的条目 | P0 | `phrase.aardio:89` | |
| `M2-PARSE-010` | 读取时校验每条 `cbSize == 16`，不符视为格式不支持并中止 | P0 | `phrase.aardio:75-78` | |
| `M2-PARSE-011` | 写入前按编码字典序升序排序 | P0 | `phrase.aardio:314` | 系统读取依赖此顺序 |

### 时间变量对照

| `$` 别名 | 微软变量 | 含义 |
|---|---|---|
| `$year` | `%yyyy%` | 4 位年 |
| `$year_yy` | `%yy%` | 2 位年 |
| `$month_mm` | `%MM%` | 2 位月 |
| `$month` | `%M%` | 月（不补零） |
| `$day` / `$day_dd` | `%dd%` | 2 位日 |
| `$fullhour` | `%HH%` | 24 小时制时 |
| `$minute` | `%mm%` | 分 |
| `$second` | `%ss%` | 秒 |

> **需在 UI 中明示的陷阱**：用户若在 Windows 系统短语设置界面编辑过短语，时间变量会被固化为编辑时刻的具体时间，永久失去动态性（`phraseHelp.aardio:15`）。

---

## 3. 编辑（`EDIT`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M2-EDIT-001` | 提供短语文本编辑器，无长度限制，带横纵向滚动 | P0 | `dlg/dict/phrase.aardio:11, 259` | 短语库体量远小于码表，无 M1 的性能压力 |
| `M2-EDIT-002` | Tab 键在编辑器内插入制表符 | P1 | `dlg/dict/phrase.aardio:268-276` | |
| `M2-EDIT-003` | 维护修改脏标记，加载/覆盖前二次确认 | P0 | `dlg/dict/phrase.aardio:37-41, 113-117` | |
| `M2-EDIT-004` | 编辑器内容持久化到应用配置，重启后恢复 | P1 | `dlg/dict/phrase.aardio:61-62, 260, 263-266` | 配置键 `phrase.editorText` |
| `M2-EDIT-005` | 提供「恢复到范例」，回填内置默认短语库 | P1 | `dlg/dict/phrase.aardio:238-244` | 见 [§6 默认短语库](#6-默认短语库) |
| `M2-EDIT-006` | 提供右键上下文菜单：造词 / 保存文件 / 加载文件 / 恢复范例 | P1 | `dlg/dict/phrase.aardio:153-246` | |
| `M2-EDIT-007` | 提供「格式说明」弹窗，图文说明 3 类写法与 2 条注意事项 | P1 | `dlg/dict/phraseHelp.aardio` | 新 UI 建议改为可折叠的内联帮助 |
| `M2-EDIT-008` | 保存时若内容为空，回填默认范例并提示「短语内容不能为空」 | P1 | `dlg/dict/phrase.aardio:75-80` | |
| `M2-EDIT-009` | 长任务期间置编辑器只读并展示进度指示 | P0 | `dlg/dict/phrase.aardio:139-150` | |

---

## 4. 安装到系统（`INSTALL`）

| ID | 需求 | P | 来源 | 备注 / 风险 |
|---|---|---|---|---|
| `M2-INSTALL-001` | 将编辑器文本解析并写入系统短语库 | P0 | `dlg/dict/phrase.aardio:91-105`、`tsfUtil.aardio:208-220` | |
| `M2-INSTALL-002` | 同时写入 v1 与 v2 两个文件，内容完全一致 | P0 | `phrase.aardio:205-214, 364` | v2 完整写入后复制为 v1 |
| `M2-INSTALL-003` | 写入前循环终止 `ctfmon.exe` / `ChsIME.exe` 并删除旧文件，最多重试 10 次（每次间隔 1 秒） | P0 | `phrase.aardio:194-203` | 需检查最终是否成功，原项目未检查 |
| `M2-INSTALL-004` | 写入前自动置注册表 `Enable Wubi EUDP = 1` 并广播设置变更 | P0 | `dlg/dict/phrase.aardio:86-89` | 委托 [M4](./M4-ime-control.md) |
| `M2-INSTALL-005` | 写入走 TSF 重启流程（停服务 → 写入 → 恢复服务 → 重启输入法） | P0 | `tsfUtil.aardio:208-220` | 委托 [M4](./M4-ime-control.md) |
| `M2-INSTALL-006` | 写入成功后广播 `system.phrase.changed` 事件 | P0 | `tsfUtil.aardio:213-215` | |
| `M2-INSTALL-007` | 写入成功提示「如果不能输入请按 Win + 空格 切换输入法」 | P0 | `dlg/dict/phrase.aardio:100` | |
| `M2-INSTALL-008` | 写入失败展示具体错误信息 | P0 | `dlg/dict/phrase.aardio:96-98` | |
| `M2-INSTALL-009` | 供 [M1](./M1-lex-table.md) 调用：安装码表时把分离出的短语一并写入 | P0 | `tsfUtil.aardio:181-189` | |
| `M2-INSTALL-010` | **【新增】写入前备份现有短语库，失败时自动回滚** | P0 | 原项目**无此能力** | 与 `M1-INSTALL-013` 同构 |
| `M2-INSTALL-011` | **【新增】写入后校验：重新读取并比对条目数** | P1 | 原项目**无此能力** | |

### ⚠️ 数据丢失风险

原项目的写入流程会**先删除**两个 EUDP 文件再写新内容（`phrase.aardio:194-214`）。若在删除后写入前失败（磁盘满、权限变化、进程崩溃），用户的全部自定义短语**永久丢失且无提示**。

考虑到短语库承载的是用户手工积累的个性化内容（而码表通常可重新下载），此处的数据价值实际**高于**码表，备份需求优先级为 **P0**。

---

## 5. 造词（`COIN`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M2-COIN-001` | 对编辑器选中文本，用**系统码表**按形码规则造词，结果替换选区 | P1 | `dlg/dict/phrase.aardio:156-217` | 与 `M1-COIN-001` 共用底层规则 |
| `M2-COIN-002` | 造词前校验系统码表非空，为空则提示并中止 | P1 | `dlg/dict/phrase.aardio:165-169` | |
| `M2-COIN-003` | 郑码造词时校验是否选定专用构词码表 | P2 | `dlg/dict/phrase.aardio:176-181` | |

### ⚠️ 原项目缺陷：分支条件错误导致郑码造词结果重复

`dlg/dict/phrase.aardio:176-208`：

```aardio
if(imeVer=="zhengma"){ ...郑码规则... }
if(imeVer=="bxm"){ ...表形码规则... }      // ← 应为 elseif
else { ...五笔规则... }
```

第二个 `if` 缺少 `else` 前缀，导致 `imeVer == "zhengma"` 时：先执行郑码分支产出一批结果，随后 `bxm` 判断失败又落入 `else` 分支，用五笔规则**再产出一批**，最终输出重复且错误。

对照 `dlg/dict/lex.aardio:775` 使用的是正确的 `elseif` —— 说明这是 M2 界面独有的复制粘贴缺陷。

**新实现要求**：三种规则互斥分派，共用一个造词服务，避免两处实现漂移。

---

## 6. 默认短语库

原项目内置 36 条默认短语（`phrase.aardio:393-428`），是产品开箱体验的重要组成。新项目应保留同等规模。

| 编码 | 类别 | 内容概要 |
|---|---|---|
| `aar` | 示例 | `aardio` |
| `zzrq` | 日期 | `%yyyy%年%MM%月%dd%日` |
| `zzsj` | 日期时间 | `%yyyy%年%MM%月%dd%日 %HH%:%mm%:%ss%` |
| `z` | 常用符号 | `『』「」🤝✊👍👋💪🙏└──` |
| `zzkh` | 括号 | `「」『』〖〗《》〈〉〔〕【】≮≯≤≥` |
| `zzbd` | 标点 | `．· …… ～ ── ‖ ∵ ∴ ☆ ★ ○ ● …` |
| `zzbq` | 表情 | `🤝👍💪👋🙏😄😂🤣😍☹😭😇` |
| `zzfh` | 符号 | `🆗☑❎✔❌🉑🈲🈴💯ℹ💬✍♀♂✉☎☯☝✈✂📈✨` |
| `zzsx` | 数学 | `.%‰√×÷＋－＜＝＞±∈∏∑…` |
| `zzjt` | 箭头 | `→↑←↓↖↗↘↔↕…⇦⇧⇨⇩⇪` |
| `zzys` `zzsz` `zzds` `zzzs` | 数字序号 | 圆圈/实心/带点/括号数字 |
| `zzdl` `zzxl` | 罗马数字 | 大写 / 小写 |
| `zzzm` | 带圈字母 | `ⓐ`–`ⓩ` |
| `zzqx` | 天气 | `☀☁☂☃☼♨☄๑` |
| `zzts` | 特殊 | `℃°♂♀§№＃＆＠＼＾＿￣` |
| `zzxx` | 希腊字母 | `αβγδεζηθ…ω` |
| `zzzy` | 注音符号 | `ㄅㄆㄇㄈ…ㄦ` |
| `zzry` `zzrz` | 日文平假名 | 清音 / 浊音拗音 |
| `zzpj` `zzpz` | 日文片假名 | 清音 / 浊音拗音 |
| `zzpy` | 拼音声调 | `āáǎàōóǒò…ǔù` |
| `zzey` `zzxe` | 西里尔字母 | 大写 / 小写 |
| `zzfy` | 法文字母 | `ÀÉÈÊÎÏÙÛÇàâäéèêë…` |
| `zzdz` | 地支 | `子丑寅卯辰巳午未申酉戌亥` |
| `zzbg` | 八卦 | `乾☰ 兑☱ 离☲ 震☳ 巽☴ 坎☵ 艮☶ 坤☷ ⚊ ⚋` |
| `zzxq` | 国际象棋 | `♔♕♖♗♘♙♚♛♜♝♞♟` |
| `zzpk` | 扑克 | `♠♡♢♣♤♥♦♧` |
| `zzwx` | 音乐 | `♩♪♫♬♭♮♯` |

**命名约定**：全部以 `zz` 开头（`z` 是五笔通配键，正常输入不会占用），后两位为拼音首字母缩写。新项目应保持此约定。

---

## 7. 数据依赖

| 依赖 | 来源模块 | 说明 |
|---|---|---|
| 系统码表 | [M1](./M1-lex-table.md) | `M2-COIN-001` 需读取以获取单字编码 |
| 造词规则 | [M1](./M1-lex-table.md) | 三种形码规则共用 |
| 注册表写入 | [M4](./M4-ime-control.md) | `M2-INSTALL-004` |
| TSF 重启流程 | [M4](./M4-ime-control.md) | `M2-INSTALL-005` |
| 系统版本探测 | [M7](./M7-app-shell.md) | `M2-IO-004` |
| 配置持久化 | [M7](./M7-app-shell.md) | `M2-EDIT-004` |
| 事件广播 | [M7](./M7-app-shell.md) | `M2-INSTALL-006` |

**反向被依赖**：

| 消费方 | 用途 |
|---|---|
| [M1](./M1-lex-table.md) `M1-SPLIT-003` | 读取系统短语映射表，检测编码冲突 |
| [M1](./M1-lex-table.md) `M1-INSTALL-005` | 安装码表时写入分离出的短语 |
| [M3](./M3-reverse-lookup.md) | 码表中查不到编码时，回退到短语库反查 |

---

## 8. 对外接口草案

### Tauri Commands

| Command | 用途 |
|---|---|
| `phrase_load_system` | 读取系统短语库 → 文本 |
| `phrase_load_file` | 从文件加载（自动识别二进制/文本） |
| `phrase_save_file` | 保存为 `.phrase.txt` |
| `phrase_install` | 解析文本并写入系统（含备份） |
| `phrase_restore` | 还原备份 |
| `phrase_default` | 返回内置默认短语库文本 |
| `phrase_coin_words` | 造词 |
| `phrase_map` | 返回编码→短语映射（供 M1/M3 使用） |
| `phrase_validate` | 仅解析不写入，返回条目数与错误行号 |

> `phrase_validate` 为**新增能力**：原项目没有「先校验再安装」，用户只能通过安装失败得知格式错误，且此时短语库已被删除。

### Events

| Event | 载荷 |
|---|---|
| `phrase://system-changed` | `{ count }` |
| `phrase://progress` | `{ task_id, phase, message }` |

---

## 9. 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 写入中途失败导致用户短语永久丢失 | **高** | 备份 + 回滚，见 `M2-INSTALL-010` |
| 拖放二进制短语文件显示乱码（原项目缺陷） | 中 | 修复分支逻辑 + 测试 |
| 郑码造词结果重复（原项目缺陷） | 中 | 三规则互斥分派，与 M1 共用服务 |
| 短语文本方言解析歧义（6 种格式优先级） | 中 | 严格按原顺序实现 + 逐格式单测 |
| `$[...]` 数组分割规则（空格 vs 逐字符）易误用 | 低 | UI 提供实时预览：输入即展示展开后的候选列表 |
| 低版本 Windows 无 EUDP 能力 | 低 | 启动即探测，不支持时整个模块置灰并说明原因 |

---

## 10. 源码索引

完整反向索引见 [`03-source-index.md`](../03-source-index.md)。

| 域 | 核心逻辑 | 界面入口 |
|---|---|---|
| `IO` | `lib/wubi/phrase.aardio:11-104`（**EUDP 解析**）、`125-176`（EUDP→文本）、`368-391`（v1/v2 择新） | `dlg/dict/phrase.aardio:18-58`（读取）、`108-137`（文件加载）、`248-253`（拖放） |
| `PARSE` | `lib/wubi/phrase.aardio:234-288`（**6 种行格式**）、`244-254`（时间变量）、`292-303`（`$[...]` + 序号） | — |
| `EDIT` | `lib/wubi/phrase.aardio:393-428`（**默认短语库 36 条**） | `dlg/dict/phrase.aardio:139-150, 153-283`；`dlg/dict/phraseHelp.aardio`（格式说明） |
| `INSTALL` | `lib/wubi/phrase.aardio:186-366`（**文本→EUDP 写入**）、`lib/tsfUtil.aardio:208-220`（停机窗口编排） | `dlg/dict/phrase.aardio:60-106` |
| `COIN` | 复用 `lib/wubi/lexFile.aardio:1102-1197` | `dlg/dict/phrase.aardio:156-217` |

### ⚠️ 不要照抄的位置

| 位置 | 问题 | 对应需求 |
|---|---|---|
| `dlg/dict/phrase.aardio:130-135` | 二进制分支后缺 `else`，结果被文本读取覆盖 | `M2-IO-007` |
| `dlg/dict/phrase.aardio:191` | `if(imeVer=="bxm")` 应为 `elseif`，郑码造词输出重复 | `M2-COIN-001` |
| `lib/wubi/phrase.aardio:194-214` | 先删两个文件再写入，无备份 | `M2-INSTALL-010` |
| `lib/wubi/phrase.aardio:106-115` | `findCodeFromMap` 线性扫描 | `M2-IO-008` |

---

## 需求统计

| 域 | 条目数 | P0 | P1 | P2 |
|---|---:|---:|---:|---:|
| `IO` | 9 | 5 | 4 | 0 |
| `PARSE` | 11 | 7 | 4 | 0 |
| `EDIT` | 9 | 3 | 6 | 0 |
| `INSTALL` | 11 | 10 | 1 | 0 |
| `COIN` | 3 | 0 | 2 | 1 |
| **合计** | **43** | **25** | **17** | **1** |
