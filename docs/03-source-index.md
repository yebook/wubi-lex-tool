# 03 — 旧项目源码索引

> ## ⚠️ 先读这一段
>
> 本文档是**行为查证工具**，不是移植清单。
>
> 原项目在文档集中的角色只有一个：**说明「正确的行为是什么」**。它不说明「该怎么写」。
> 用本文档确认「原来是怎么判定的 / 输出格式长什么样 / 边界条件是什么」，然后**用你认为最好的方式实现它**。
>
> 只有 [`02-architecture.md#0.1`](./02-architecture.md#01-什么必须一致行为契约) 列出的 12 条**行为契约**必须逐位一致。其余一切 —— 数据结构、算法、并发、错误处理、UI —— 都自由。
>
> 各区块标注的 ⚠️ 表示该处存在**已知缺陷或反模式**，查行为可以看，写代码别照抄。

**用途**：新项目实现某个需求时，用本文档**反向定位**到原项目中对应的行为定义，直接打开文件对照。

组织方式：按源文件 → 行号区块 → 关键函数/变量 → 对应需求 ID。
与各模块文档的「源码索引」章节互补：模块文档是**正向**（功能 → 源码），本文档是**反向**（源码 → 功能）。

## 使用方式

```
实现 M1-PARSE-002（文本码表方言解析）
  → 查本文档 lib/wubi/lexFile.aardio
  → 定位到 34-213 行 addText()
  → 读懂它「接受哪些行格式、优先级如何、边界怎么处理」
  → 对照 docs/01-data-formats.md#3 的规格
  → 用 Rust 的方式重新实现（预编译正则、rayon 并行、结构化错误）
      —— 不是把 aardio 的控制流翻译一遍
```

所有路径相对于 `wubi-lex/`。行号基于分析时的仓库快照（v12.1，只读，不会变动）。

## 文件速查

| 文件 | 行数 | 模块 | 章节 |
|---|---:|---|---|
| `main.aardio` | 321 | M7 | [→](#mainaardio) |
| `lib/config.aardio` | 25 | M7 | [→](#libconfigaardio) |
| `lib/style.aardio` | 134 | M7 / UX | [→](#libstyleaardio) |
| `lib/tsfInput.aardio` | 146 | M4 | [→](#libtsfinputaardio) |
| `lib/tsfUtil.aardio` | 235 | M4 / M1 | [→](#libtsfutilaardio) |
| `lib/app/hotkey.aardio` | 303 | **P3** | [→](#已废弃文件p3) |
| `lib/app/lexContents.aardio` | 365 | M1 / M6 | [→](#libapplexcontentsaardio) |
| `lib/app/lexNetContents.aardio` | 227 | M6 | [→](#libapplexnetcontentsaardio) |
| `lib/ui/chineseNumber.aardio` | 4 | **P3** | [→](#已废弃文件p3) |
| `lib/ui/doublePinyinMenu.aardio` | 54 | M4 | [→](#libuidoublepinyinmenuaardio) |
| `lib/winex/msCandidate.aardio` | 29 | **P3** | [→](#已废弃文件p3) |
| `lib/wubi/candidateWindow.aardio` | 7 | **P3** | [→](#已废弃文件p3) |
| `lib/wubi/chineseNumber.aardio` | 4 | **P3** | [→](#已废弃文件p3) |
| `lib/wubi/fonts.aardio` | 45 | M3 / M6 | [→](#libwubifontsaardio) |
| `lib/wubi/lexFile.aardio` | 1406 | M1 | [→](#libwubilexfileaardio-) |
| `lib/wubi/phrase.aardio` | 439 | M2 | [→](#libwubiphraseaardio-) |
| `lib/wubi/spellingTable.aardio` | 64 | M3 / M6 | [→](#libwubispellingtableaardio) |
| `lib/wubi/table.aardio` | 273 | M3 / M5 | [→](#libwubitableaardio) |
| `lib/wubi/text.aardio` | 19 | M1 / M2 | [→](#libwubitextaardio) |
| `lib/wubi/weightData.aardio` | 22 | M1 / M6 | [→](#libwubiweightdataaardio) |
| `dlg/setting.aardio` | 329 | M4 | [→](#dlgsettingaardio) |
| `dlg/spelling.aardio` | 590 | M3 | [→](#dlgspellingaardio-) |
| `dlg/dict/dict.aardio` | 80 | M7 | [→](#dlgdictdictaardio) |
| `dlg/dict/lex.aardio` | 1455 | M1 | [→](#dlgdictlexaardio-) |
| `dlg/dict/phrase.aardio` | 286 | M2 | [→](#dlgdictphraseaardio) |
| `dlg/dict/phraseHelp.aardio` | 22 | M2 | [→](#dlgdictphrasehelpaardio) |
| `dlg/dict/wordWeight.aardio` | 98 | M1 | [→](#dlgdictwordweightaardio) |
| `dlg/help/etymon.aardio` | 257 | M5 | [→](#dlghelpetymonaardio) |
| `dlg/help/help.aardio` | 45 | M7 / M5 | [→](#dlghelphelpaardio) |
| `dlg/help/sys.aardio` | 50 | M5 | [→](#dlghelpsysaardio) |
| `dlg/help/wubi.aardio` | 95 | M5 / M6 | [→](#dlghelpwubiaardio) |
| `sepllingData/build.aardio` | 145 | M6 | [→](#sepllingdatabuildaardio) |

⚠️ 标记表示该区块存在**已知缺陷**，实现时不要照抄。

---

## `main.aardio`

应用入口。单实例、主窗口、Tab 容器、托盘菜单、全局热键。

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 1 | `//RUNAS//` | 管理员权限声明 | `M7-INST-003` |
| 6–16 | `win.form` + `mainForm.add` | 主窗口与 4 个一级 Tab strip 定义 | `M7-WIN-001/002` |
| 19–25 | `win.ui.atom` | 单实例检测，已有实例则置前并退出 | `M7-INST-001/002` |
| 27–34 | `fsys.update.simpleMain` | 启动时检查更新，就绪则走更新流程 | `M6-UPDATE-001` |
| 37–38 | `win.ui.simpleWindow` | 无边框窗口 + 自定义标题栏 | `M7-WIN-001` |
| 42–68 | `win.ui.tabs` + `skin` + `loadForm` | 一级导航与子页加载 | `M7-WIN-002` |
| 71–80 | `onMinimize` | 创建托盘、切工具窗口样式、释放工作集 | `M7-TRAY-001`、`M7-WIN-009/010` |
| 82–87 | `onClose` | 关闭 → 最小化到托盘 | `M7-WIN-007` |
| 89–93 | `onDestroy` | 清理托盘图标 | `M7-TRAY-006` |
| 99–269 | `wndproc[_WM_TRAYMESSAGE]` | **托盘菜单主体**，24 个菜单项 | `M7-TRAY-003/004` |
| 104–121 | └ 设置输入法 / 管理码表 / 管理短语 | 页面跳转 | `M7-TRAY-003` |
| 124–139 | └ 热键文本拼接 | 修饰键掩码 → `Ctrl + F2` 文本 | `M3-HOTKEY-005`、`M7-KEYMAP-011` |
| 141–150 | └ 字根反查 / 字根键位图 | | `M3-HOTKEY-005`、`M5-CHART-007` |
| 154–168 | └ 启用微软五笔 + 系统设置跳转 | 含码表方案名拼接 | `M4-TIP-001`、`M4-SYS-002` |
| 170–180 | └ 英文键盘 / 微软拼音 | | `M4-TIP-002/003` |
| 182–183 | └ 双拼方案子菜单 | | `M4-DPY-006` |
| 188–191 | └ 打开系统输入法设置 | `ms-settings:keyboard` | `M4-SYS-003` |
| 194–231 | └ 设置默认输入法子菜单 | 枚举布局与 TIP，勾选当前默认 | `M4-TIP-005/006/007` |
| 235–243 | └ 屏幕键盘 / 反转鼠标键 | | `M4-SYS-005/006` |
| 247–254 | └ 官网 / 退出 | | `M5-ABOUT-001`、`M7-TRAY-005` |
| 260–267 | `_WM_LBUTTONUP` | 左键还原窗口，反查页则聚焦 | `M7-TRAY-002`、`M3-INPUT-008` |
| 271–294 | `subscribe("spellingHotkeyChange")` | 全局热键注销 + 重新注册 | `M7-HOTKEY-001/002`、`M3-HOTKEY-001/002` |
| 279–283 | └ 前台窗口归属判断 | 本应用则最小化，否则唤起 | `M3-HOTKEY-002` |
| 296–299 | `subscribe("showEtymon")` | 跳转字根图页 | `M5-CHART-007` |
| 303–313 | `_ARGV.tray` | `/tray` 静默启动，延迟 3 秒建托盘 | `M7-INST-004`、`M7-TRAY-007` |
| 315–319 | `win.ui.accelerator` | `Ctrl+W` 最小化 | `M7-HOTKEY-003`、`M7-KEYMAP-011` |

---

## `lib/config.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 3 | `fsys.config(...)` | 配置根，落盘 `%APPDATA%/aardio/std/wubi-lex-tool` | `M7-CONF-001/006` |
| 5–11 | `config.lex.files` / `names` | 码表列表默认值 | `M1-LIST-015` |
| 13–15 | `config.hotkey.spelling` | 反查热键默认 `Ctrl + F2` | `M3-HOTKEY-004`、`M7-KEYMAP-011` |

---

## `lib/style.aardio`

UI 皮肤令牌。**新项目改用 Tailwind v4，本文件仅作配色参考**；新色板见 [`21-ui-ux.md#43-色板`](./21-ui-ux.md#43-色板)。

| 行 | 令牌 | 用途 |
|---|---|---|
| 4–10 | `primaryButton` | 主按钮 `#8FB2B0` / hover `#928BB3` |
| 11–17 | `button` | 次级按钮（40% alpha） |
| 18–25 | `transButton` | 透明按钮 |
| 26–40 | `checkBox` | 复选框（FontAwesome 图标切换） |
| 41–50 | `radio` | 单选 |
| 51–64 | `link` | 链接 |
| 65–71 | `plainButton` | 纯文字按钮 |
| 72–83 | `key` | **键帽**（虚拟键盘用）→ `UX-COMP-003` |
| 84–101 | `dropdown` | 下拉菜单 |
| 102–119 | `trackbar` | 滑块（未实际使用） |
| 120–133 | `palette` | 调色板（未实际使用） |

---

## `lib/tsfInput.aardio`

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 8–10 | `enablePinyin` | 微软拼音启停 | `M4-TIP-002` |
| 12–15 | `enableWubi` | 微软五笔启停 | `M4-TIP-001` |
| 17–22 | `enableUsKeyboard` | 英文键盘启停（**无条件禁用 `0804:00000409`/`0804:00000804`**） | `M4-TIP-003` |
| 26–38 | `getStatus` | 三者启用状态查询 | `M4-TIP-004` |
| 40–48 | `getLocal` / `setLocal` | 「所有应用同一输入法」`SPI_*_THREADLOCALINPUTSETTINGS` | `M4-TIP-007` |
| 50–58 | `getInputMethodOverride` | 默认输入法读取 | `M4-TIP-006` |
| 60–107 | `getDoublePinyinSchemes` | 枚举双拼方案（3 内置 + 自定义），识别小鹤 | `M4-DPY-001/002` |
| 88–100 | └ 空闲 ID 分配 | `max(index)+1`，起始 10 | `M4-DPY-003` |
| 109–125 | `enableoublePinyinScheme` | 启停/切换方案 | `M4-DPY-004/005` |
| 127–132 | `enableXhup` | 小鹤双拼快捷开关 | `M4-DPY-003` |

---

## `lib/tsfUtil.aardio`

**最高风险文件**。TSF 停机窗口与安装编排。

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 21–52 | `reset(proc)` | **停机窗口编排**：保存语言 → 禁用五笔 → 停服务 → 结束任务 → kill 进程 → 执行 proc → 恢复 | `M4-TSF-001..006` |
| 25–27 | └ `srvMgr.startDisabled/stop` | `TabletInputService` 停止 | `M4-TSF-002` |
| 29–31 | └ `schtasks /End MsCtfMonitor` | 计划任务结束 | `M4-TSF-003` |
| 33–34 | └ `process.kill` | `ctfmon.exe` / `ChsIME.exe` | `M4-TSF-004` |
| 41–48 | └ 恢复服务与任务 | ⚠️ **不检查返回值** | `M4-TSF-010` |
| 54–61 | `deactivate` | `ITfInputProcessorProfileMgr.DeactivateProfile` | `M4-TIP-008` |
| 63–70 | `activate` | `ActivateProfile` | `M4-TIP-008` |
| 72–88 | `aclGrant` | **所有权接管**：`takeOwn` → `icacls /grant` → proc → `/setowner TrustedInstaller` | `M4-TSF-007/009` |
| 90–198 | `intallLex` | 安装编排主体 | `M1-INSTALL-*` |
| 106–124 | └ 短语检测与确认弹窗 | `hasPhrase()` 位标志解读 | `M1-INSTALL-004`、`M1-SPLIT-001` |
| 119–123 | └ `Enable Wubi EUDP = 1` | | `M1-INSTALL-006`、`M4-REG-004` |
| 126–130 | └ 追加模式 | `sysLex.addLex(lexFile)` | `M1-INSTALL-002`、`M1-PARSE-014` |
| 138–147 | └ 删除重试循环 | ⚠️ 10 次后**不检查是否成功** | `M4-TSF-008` |
| 149–168 | └ 双文件写入 | `ChsWubiNew.lex` + `ChsWubi.lex` | `M1-INSTALL-008` |
| 170–175 | └ 小鹤音形 `PinyinMixEnable = 1` | | `M1-INSTALL-007` |
| 181–189 | └ 短语写入 | 委托 `wubi.phrase.saveToSystem` | `M1-INSTALL-005`、`M2-INSTALL-009` |
| 200–206 | `intallLexFile` / `intallLexText` | 两个入口 | `M1-INSTALL-001/003` |
| 208–220 | `intallPhrase` | 短语安装入口 | `M2-INSTALL-001/005` |

---

## `lib/app/lexContents.aardio`

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 11–54 | `newFile` | 命名循环 + 校验（非空/合法字符/不重名） | `M1-LIST-007`、`M1-INSTALL-016` |
| 56–73 | `remove` / `removeByIndex` | 删除条目 + 本地文件 | `M1-LIST-008` |
| 82–92 | `valid` | 有效性校验，失效自动移除 | `M1-LIST-012` |
| 94–111 | `getPathByIndex` / `getNameByIndex` / `isUrl` | 索引访问 | `M1-LIST-001` |
| 113–175 | `download` | 下载 + LZMA 解压 + 魔数校验 | `M6-DOWN-003`、`M6-ARCHIVE-001/003/004`、`M1-LIST-014` |
| 126–152 | └ `downBox.endProc` | 解压进度（1000 刻度） | `M6-ARCHIVE-005` |
| 177–183 | `clear` | 清空列表 | `M1-LIST-009` |
| 189–210 | `loadDefault` | **每分类取前 2 条**加入列表 | `M1-LIST-011` |
| 212–348 | `addFile` | **多格式导入分派** | `M1-LIST-003/006` |
| 223–226 | └ URL 分支 | | `M1-LIST-004` |
| 227–242 | └ `.txt/.yaml/.ini` | | `M1-PARSE-002` |
| 243–274 | └ `.json` + 首行校验 | | `M1-PARSE-010` |
| 275–308 | └ `.csv` + 首行校验 | | `M1-PARSE-009` |
| 309–327 | └ `.lex.lzma` | | `M1-PARSE-011` |
| 328–347 | └ `.lex` + 魔数校验 | | `M1-PARSE-001/012` |

---

## `lib/app/lexNetContents.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 4 | `dataPath` | 缓存 `lex-default-v2.table` | `M6-CATALOG-003` |
| 6–20 | `getTable` | 读缓存，失败回退内置 | `M6-CATALOG-003/004` |
| 12 | └ `eval(...)` | ⚠️ **把缓存当代码执行 —— 禁止照抄** | `M6-CATALOG-007` |
| 22–34 | `update` | 拉 `index.json` 并序列化落盘 | `M6-CATALOG-002` |
| 36–221 | `defaultData` | **内置目录：8 分类 40 条**，完整清单见 [M6 附录 A](./modules/M6-resource-sync.md#附录-a内置码表目录) | `M6-CATALOG-001` |

---

## `lib/ui/doublePinyinMenu.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 8–49 | `ctor` | 构建双拼弹出菜单 | `M4-DPY-006` |
| 12–16 | └ 禁用双拼项 | | `M4-DPY-004` |
| 18–23 | └ 方案切换（toggle 语义） | 再次点选当前项则关闭 | `M4-DPY-005` |
| 25–40 | └ 小鹤双拼一键安装 | **方案串硬编码在第 30 行** | `M4-DPY-003` |
| 43–46 | └ 系统拼音设置跳转 | | `M4-DPY-007`、`M4-SYS-004` |

---

## `lib/wubi/fonts.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 7–15 | 字体路径选择 | 092 用专用字体 | `M3-FONT-002` |
| 17–20 | `onEtymonFontInstalled` | `fonts.addFamily` 进程级注册 | `M3-FONT-001` |
| 22–36 | 按需下载 | `wubi-lex-etymon-v5.lzma` | `M6-DOWN-008` |
| 43 | 字体来源注释 | 基于 `github.com/yanhuacuo/qingg` | `M5-ABOUT-008` |

---

## `lib/wubi/lexFile.aardio` ★

**码表引擎核心，1,406 行**。新项目 `wubilex-codec` + `wubilex-core` 的主要参考。

### 解析

| 行 | 函数/区块 | 说明 | 需求 |
|---|---|---|---|
| 34–213 | `addText(bin)` | **文本方言解析主体** | `M1-PARSE-002` |
| 41 | └ YAML front matter 剥离 | Rime 兼容 | `M1-PARSE-004` |
| 44 | └ `#` 注释行剥离 | | `M1-PARSE-005` |
| 47–53 | └ `[Text]` 检测 + 极点标记识别 | | `M1-PARSE-006` |
| 56–78 | └ **微软码表专用分支**（词前码后、一行多码，提前 return） | | `M1-PARSE-002` |
| 85–177 | └ **主循环 6 种行格式**（A→F 优先级） | 规格见 [`01-data-formats.md#3.2`](./01-data-formats.md#32-主循环行格式按尝试顺序) | `M1-PARSE-002` |
| 117–133 | └ 降序权重反转 `0xFFFF−w` | | `M1-PARSE-007` |
| 135–137 | └ `if(codeWeight[0])` | ⚠️ **死代码，永不为真** | — |
| 179–187 | └ 降序权重归一化（平移到基线 5000） | | `M1-PARSE-007` |
| 189–212 | └ 极点标记清理（`^$!` 删、`~` 去首字符） | | `M1-PARSE-006` |
| 216–286 | `addFile(path)` | 二进制读取（`imscwubi`）+ 文本回退 | `M1-PARSE-001/003/012` |

### 序列化

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 337–417 | `saveLex(path)` | **`.lex` 二进制写入**，含字母索引生成 | [`01-data-formats.md#1`](./01-data-formats.md#1-lex-微软五笔码表二进制) |
| 356–406 | └ `alphaIndex` 生成 | 26 个字母的起始偏移 | `01-data-formats.md#1.3` |
| 379–390 | └ 权重隐式递增规则 | 无显式权重则前项 +1 | `01-data-formats.md#1.5` |
| 418–442 | `saveTxt(path, reverse)` | 文本导出（UTF-16LE） | `M1-IO-001` |
| 889–1051 | `tostring(reverse, singleLine, sortWeight, toPhrase)` | **7 种输出格式** | `M1-XFORM-001..007` |
| 546–568 | `saveWordWeight(path)` | 词频文件导出 | `M1-IO-005` |
| 718–730 | `getTableData` | 表数据（CSV/JSON 导出用） | `M1-IO-002/003` |
| 731–739 | `setTableData` | 表数据导入 | `M1-PARSE-009/010` |

### 查询与统计

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 443–453 | `find(word, min)` | 反查首个编码 | `M1-PARSE-017` |
| 454–466 | `spelling(word)` | **反查全部编码，按长度降序** ⚠️ **O(n·m) 全表扫描，必须改倒排索引** | `M1-PARSE-017`、`M3-QUERY-001` |
| 491–545 | `sort(f)` | 编码字典序 + 同码按权重升序 | `M1-PARSE-018` |
| 1052–1072 | `statistics()` | 总编码/总词条/重码编码/重码词条 | `M1-PARSE-015` |
| 1073–1078 | `isEmpty` / `getCode` | | `M1-PARSE-016` |
| 1198–1201 | `test(code, str)` | 版本探测的基础断言 | `M1-PARSE-013` |
| 1202–1356 | `getVersion()` | **8 方案探测**：5 组直接判定 + 4 方案打分 | `M1-PARSE-013` |
| 1293 | └ `this.test("XFXY","线")` | ⚠️ **大写导致该测试永久失效 —— 必须改小写** | `M1-PARSE-013` |
| 1361–1369 | `lexFile.path()` | 系统码表路径解析 | `M1-LIST-002` |

### 变换

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 292–307 | `add(code, word, weight)` | 单条添加 | `M1-COIN-005` |
| 308–336 | `addLex(other)` | **码表合并**（追加安装的底层） | `M1-PARSE-014` |
| 467–490 | `filter(f)` | 通用过滤（精简的底层） | `M1-SLIM-001..005` |
| 569–614 | `loadWordWeight(path, reorder, fixedSingleChar)` | **词频重算**，3 种策略 | `M1-WEIGHT-001/003/004` |
| 615–643 | `unique()` | 一码一词 ⚠️ **629 行循环无递增语句** | `M1-SLIM-007` |
| 644–679 | `simplified()` | 出简不出全（两遍扫描） | `M1-SLIM-006` |
| 680–717 | `optShortCodeChar()` | **简码字全码后移**（出简后出全） | `M1-WEIGHT-007` |

### 短语分离

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 740–787 | `hasPhrase()` | 位标志：bit0 = 含 z 键，bit1 = 键名占用 | `M1-SPLIT-001` |
| 754–783 | └ **键名占用启发式** | `gb`/`rare`/`hasWord` 三元判据 | `M1-SPLIT-001` |
| 788–856 | `removePhrase()` | 分离 z 键 + 键名非常用单字 | `M1-SPLIT-002` |
| 857–888 | `removeFromSystemPhrase()` | 分离与系统短语冲突的编码 | `M1-SPLIT-003` |

### 造词

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 1079–1101 | `getCachCharCodes(clear)` | 单字 → 最长编码缓存 | `M1-COIN-006` |
| 1102–1133 | `combine(str)` | **五笔造词规则** | `M1-COIN-001` |
| 1134–1165 | `combineZhengma(str)` | 郑码规则（三字/四字取码不同） | `M1-COIN-002` |
| 1166–1197 | `combineBxm(str)` | 表形码规则 | `M1-COIN-003` |

---

## `lib/wubi/phrase.aardio` ★

**短语引擎核心，439 行**。

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 11–104 | `loadMap(path)` | **EUDP 二进制解析** → 编码映射表 | `M2-IO-002`、[`01-data-formats.md#2`](./01-data-formats.md#2-eudp-用户短语库二进制) |
| 27–30 | └ `mschxudp` 魔数校验 | | `M2-IO-009` |
| 32–57 | └ 文件头 + 偏移表读取 | 尾部追加哨兵 | `01-data-formats.md#2.4` |
| 61–99 | └ 条目遍历（`reduce`） | `cbSize==16` 校验、`deleted` 跳过 | `M2-PARSE-009/010` |
| 106–115 | `findCodeFromMap(map, txt)` | 按文本反查编码（线性扫描） | `M2-IO-008` |
| 125–176 | `load(path)` | **EUDP → 文本** | `M2-IO-003` |
| 146–162 | └ `$[...]` 压缩输出判定 | 序号连续 + 长度 ≤ 2 UTF-16 单元 | `M2-PARSE-008` |
| 186–366 | `saveToSystem(text)` | **文本 → EUDP 二进制** | `M2-INSTALL-001..003` |
| 194–203 | └ kill 进程 + 删文件重试 10 次 | ⚠️ **先删后写，无备份** | `M2-INSTALL-003/010` |
| 224 | └ `/* */` 注释剥离 | | `M2-PARSE-002` |
| 234–288 | └ **6 种行格式解析**（P1→P6） | 规格见 [`01-data-formats.md#4.2`](./01-data-formats.md#42-行格式按尝试顺序) | `M2-PARSE-001` |
| 244–254 | └ `$year` → `%yyyy%` 时间变量映射 | 9 个别名 | `M2-PARSE-006` |
| 258, 271–277 | └ 多行短语累积 | | `M2-PARSE-003` |
| 292–301 | └ `$[...]` 数组展开 | 含空格按空格分，否则逐字符 | `M2-PARSE-004` |
| 302–303 | └ 序号自动分配 | | `M2-PARSE-005` |
| 314 | └ 编码字典序排序 | | `M2-PARSE-011` |
| 316–357 | └ 二进制写入 | 文件头 → 偏移表 → 条目 → 回填 | `01-data-formats.md#2` |
| 368–391 | `getWubiUdpPrimaryPath()` | v1/v2 取较新者 | `M2-IO-001` |
| 393–428 | `default` | **内置默认短语库 36 条** | `M2-EDIT-005` |

---

## `lib/wubi/spellingTable.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 5–18 | `ctor` | 构造即触发下载 `{版本}.lzma` | `M3-SPLIT-005`、`M6-DOWN-004` |
| 19–36 | `find(str)` | 懒加载数据表 → `HashMap` 查询 | `M3-SPLIT-001` |
| 20 | └ 文件不存在直接 `return null` | ⚠️ 静默降级 | `M3-SPLIT-007` |
| 39–55 | `spellingTable.find/init` | 按版本单例缓存 | `M3-SPLIT-006` |

---

## `lib/wubi/table.aardio`

**纯数据文件**。字根静态数据，需**逐字迁移**。

| 行 | 数据 | 说明 | 需求 |
|---|---|---|---|
| 4–34 | `levelOne` | **6 套一级简码**（86/98/06 共用、091、092、郑码、小鹤） | `M3-KBD-003` |
| 36–63 | `self["98"]` | 五笔 98 字根歌诀 | `M5-TEXT-004` |
| 65–92 | `self["86"]` | 五笔 86 字根歌诀 | `M5-TEXT-004` |
| 95–122 | `self["06"]` | 新世纪字根歌诀 | `M5-TEXT-004` |
| 124–150 | `self["091"]` | 091 字根歌诀（**无 Z 键**） | `M5-TEXT-004` |
| 152–179 | `self["zhengma"]` | 郑码字根歌诀 | `M5-TEXT-004` |
| 181–208 | `self["xhyx"]` | 小鹤音形（字根 + 代表字 + 韵母） | `M5-TEXT-004` |
| 210–237 | `self["bxm"]` | 表形码（形态描述） | `M5-TEXT-004` |
| 239–243 | `self["092"]` | 092 代表字（**`C` 键为空**） | `M5-TEXT-004` |
| 245–274 | `keyboard["xhyx"]` | 小鹤韵母键位映射 | `M3-KBD-008`、`M5-TEXT-005` |

> 已在 [M5 附录 A](./modules/M5-etymon-help.md#3-字根歌诀数据附录-a) 完整转录，可直接引用。

---

## `lib/wubi/text.aardio`

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 8–10 | `encode(txt)` | 转义**所有** `\s` 为 `%XX` | `01-data-formats.md#8.1` |
| 12–14 | `decode(txt)` | ⚠️ **仅识别 `%20/%09/%0D/%0A`，与编码端不对称** | `M1-PARSE-008`、`01-data-formats.md#8.2` |

---

## `lib/wubi/weightData.aardio`

| 行 | 函数 | 说明 | 需求 |
|---|---|---|---|
| 4 | `path` | 缓存 `word-weight2.txt` | `M6-CACHE-001` |
| 5–18 | `getPath()` | 缺失则下载 `word-weight.lzma` | `M1-WEIGHT-002`、`M6-DOWN-009` |

---

## `dlg/setting.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 4–32 | 窗体定义 | 15 个设置控件 | `UX-SCREEN-041` |
| 52–62 | `queryValueTable` | **批量读取注册表 + 默认值表** | `M4-REG-014` |
| 64–69 | 控件绑定 | | `M4-REG-001/002/004/005/006` |
| 70–74 | `subscribe("wubi.system.lex.changed")` | 重读 `PinyinMixEnable` | `M4-REG-015` |
| 76–81 | 四码唯一双键读取 | 按混输模式选键 | `M4-REG-003` |
| 83–95 | 显示名称/图标读取 | HKLM | `M4-REG-011/012` |
| 99–111 | 候选词数读取 | `MaxCandidates` + `EnableFixedCandidateCountMode` | `M4-REG-008` |
| 119–211 | `btnSave.oncommand` | **保存全部设置** | `M4-REG-001..013/016/017` |
| 120–132 | └ 候选数范围校验 | 3–9 | `M4-REG-010` |
| 151 | └ `Floating Icon Time Key = 0` | | `M4-REG-007` |
| 192–195 | └ 勾选五笔时触发 TSF 重启 | | `M4-REG-017` |
| 197–207 | └ 开机启动计划任务 | `sys.runAsTask` + `/tray` | `M4-SYS-001` |
| 215–217 | `plusOpenMsSetting` | 系统五笔设置跳转 | `M4-SYS-002` |
| 220–229 | `btnOpenIcon` | 图标选择（非 `.ico` 追加 `,1`） | `M4-REG-013` |
| 231–259 | `translateAccelerator` | 上下键调节候选数 + 钳制 | `M4-REG-009` |
| 266–281 | `updateTsfStatus` | 输入法状态刷新 | `M4-TIP-004` |
| 287–298 | `updateWinLang` | Windows 显示语言 | `M4-TIP-010` |
| 300–314 | 三个输入法开关回调 | | `M4-TIP-001/002/003` |
| 316–326 | `cmbDoublePiny` | 双拼菜单弹出 | `M4-DPY-006` |

---

## `dlg/spelling.aardio` ★

**反查主界面，590 行**。

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 4–48 | 窗体定义 | 含 **26 个键帽 + 五区配色**（`bgcolor`） | `M3-KBD-001/007` |
| 56–74 | `showEtymonImage(ver,k)` | 单键字根图（缓存 + 下载） | `M3-KBD-006`、`M5-CHART-006`、`M6-DOWN-007` |
| 79–340 | `searchSpelling` | **反查主逻辑**（100ms 防抖） | `M3-QUERY-*`、`M3-INPUT-007` |
| 92–127 | └ 一级简码判定与键帽切换 | 缓存未命中则动态构建 | `M3-KBD-003` |
| 129 | └ 拼音展示 | | `M3-QUERY-005` |
| 130–133 | └ 码表反查 + 短语回退 | | `M3-QUERY-001/002` |
| 136–166 | └ 编码展示 + **按键序列动画**（500ms） | `winform.reduce` 定时链 | `M3-QUERY-003`、`M3-KBD-004` |
| 168–170 | └ 「缺少编码」 | | `M3-QUERY-004` |
| 175–236 | └ 拆字（含郑码/表形码/小鹤特判） | | `M3-SPLIT-001/003/004/006` |
| 198–229 | └ **词组拆字组合规则**（2/3/4/>4 字） | | `M3-SPLIT-002` |
| 238–248 | └ `getSpellingGifPath` + 86/98 回退 | | `M3-ANIM-003` |
| 250–305 | └ **词组 GIF 横向拼接**（GDI+ 首帧合成） | | `M3-ANIM-004` |
| 306–337 | └ 单字 GIF（缓存 + 下载） | | `M3-ANIM-001/002` |
| 344–349 | 候选下拉菜单 | | `M3-INPUT-002` |
| 351–373 | `showSuggestion` | 本地拼音 + **百度接口兜底** ⚠️ 隐私 | `M3-INPUT-003` |
| 375–412 | `editBox.onChange` | 输入分派（拼音/汉字/清空/重复归一） | `M3-INPUT-006/009` |
| 415–436 | InkEdit 初始化 | ⚠️ ActiveX，**Tauri 无法复现** | `M3-INPUT-004` |
| 440–442 | `subscribe("wubi.system.phrase.changed")` | 重载短语映射 | `M3-QUERY-008` |
| 444–507 | `subscribe("wubi.system.lex.changed")` | **重载码表 + 探测版本 + 重建键帽** | `M3-QUERY-006`、`M3-KBD-002/005` |
| 454–461 | └ 下载按钮显示条件 | 缺 86/98 图库且方案为 86/98/06 | `M3-ANIM-006` |
| 478–492 | └ 键帽 hover 回调 | 歌诀 + 单键字根图 | `M3-KBD-005/006` |
| 509–525 | 加载指示 + 延迟发布事件 | | `M3-QUERY-009` |
| 530–543 | `plusDownloadSpellingGif` | 整包下载 `spelling.tar.lzma` | `M3-ANIM-005`、`M6-DOWN-010` |
| 545–565 | `plusHotkey` | **热键录制与保存** | `M3-HOTKEY-003/004`、`M7-KEYMAP-004` |
| 567–575 | `plusEtymon` | 字根图跳转 | `M3-HOTKEY-006` |
| 577–581 | `plusSpeech` | `Win + H` 语音输入 | `M3-INPUT-005` |
| 583–585 | `subscribe("spellingFocus")` | 聚焦输入框 | `M3-INPUT-008` |
| 587–590 | `Ctrl+W` 加速键 | | `M7-HOTKEY-003` |

---

## `dlg/dict/dict.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 14–37 | `win.ui.tabs` | 词库页二级导航（码表/短语） | `M7-WIN-003` |
| 39–45 | `showLex` / `showPhrase` | 外部跳转入口（托盘菜单调用） | `M7-TRAY-003` |
| 47–78 | `navImTip` | ImTip 探测/启动/官网回退 | `M7-WIN-005` |

---

## `dlg/dict/lex.aardio` ★

**码表管理主界面，1,455 行**。功能入口最密集的文件。

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 13–35 | 窗体定义 | 列表 + 编辑器 + 下拉按钮组 | `UX-SCREEN-011/012` |
| 38–74 | `busy()` × 3 | 进度指示（5 帧图标动画） | `M1-EDIT-009` |
| 76–80 | `onDropFiles` | 拖放添加（需 UIPI 放行） | `M1-LIST-006` |
| 85–98 | `beforeAddLexFile` / `afterLexListUpdated` | 添加流程事件 | `M7-BUS-002` |
| 102–142 | `installLex(appendLex)` | **安装到系统**（列表项） | `M1-INSTALL-001/002/010/011/012` |
| 152–409 | `listbox.wndproc` | **列表右键菜单** | — |
| 165–177 | └ 网络条目：下载安装 / 删除 | | `M1-LIST-014` |
| 180–196 | └ 删除 / 安装 / 追加 | | `M1-LIST-008`、`M1-INSTALL-001/002` |
| 202–226 | └ 编辑码表（后台线程加载） | | `M1-EDIT-001` |
| 231–245 | └ 词频优化：重算权重 | 打开 `wordWeight.aardio` | `M1-WEIGHT-005` |
| 246–266 | └ 词频优化：出简后出全 | | `M1-WEIGHT-007` |
| 270–283 | └ 导出词频文件 | | `M1-IO-005` |
| 285–298 | └ 导出文本码表 | | `M1-IO-001` |
| 300–325 | └ 导出 CSV（UTF-8 BOM） | | `M1-IO-002` |
| 327–351 | └ 导出 JSON | | `M1-IO-003` |
| 354–370 | └ 导出 lzma | | `M1-IO-004` |
| 372–378 | └ 浏览文件 | | `M1-LIST-013` |
| 383–402 | └ 空白区：清空 / 重置默认 | | `M1-LIST-009/010` |
| 415–425 | `reloadLexList` | 列表刷新（首项系统码表） | `M1-LIST-001` |
| 428–451 | `updateSystemLexVersion` | 后台探测版本 + 广播 | `M1-LIST-002`、`M1-PARSE-013` |
| 442 | └ 版本代号 → 显示名映射 | 8 种 | `01-data-formats.md#7.4` |
| 453–479 | 编辑器初始化 | `limit=0` + 内置格式说明文本 | `M1-EDIT-002` |
| 481–489 | `translateAccelerator` | Tab 插入制表符 | `M1-EDIT-003` |
| 493–709 | 精简下拉菜单 × 7 | | `M1-SLIM-001..007` |
| 711–1246 | `createMenuItems` | **编辑器右键菜单（30+ 项）** | `M1-EDIT-010` |
| 713–728 | └ `wubiTextTableConvert` | 格式转换统一入口 | `M1-XFORM-001..007` |
| 733–801 | └ 选中文本造词（三规则分派） | | `M1-COIN-001..004/007` |
| 803–907 | └ **空码智能造词** | 遍历词频库 + 空码位补词 | `M1-COIN-005` |
| 910–949 | └ 转换格式子菜单 × 7 | | `M1-XFORM-001..007` |
| 951–1009 | └ 分离短语子菜单 × 2 | | `M1-SPLIT-003/004` |
| 1010–1039 | └ 简繁转换 | | `M1-XFORM-008/009` |
| 1041–1152 | └ 词频调整子菜单 × 4 | | `M1-WEIGHT-006/007/008/009` |
| 1154–1196 | └ 精简码表子菜单 × 7 | | `M1-SLIM-001..007` |
| 1199–1225 | └ 另存/打开文本文件 | | `M1-EDIT-006/007` |
| 1226–1242 | └ 统计 | | `M1-EDIT-008` |
| 1249–1279 | `btnSearch` | **查找**（Unicode 感知 + 回绕） | `M1-EDIT-004` |
| 1282–1298 | 保存下拉菜单 | 4 个保存目标 | — |
| 1300–1334 | `ddSaveToCurrent` | 存为当前选中码表 | `M1-INSTALL-015` |
| 1336–1360 | `ddSaveAs` | 存为新码表 | `M1-INSTALL-016` |
| 1362–1396 | `instalLexText` | **从编辑器安装**（替换/追加） | `M1-INSTALL-003` |
| 1400–1448 | `btnAddFile` | 添加菜单（本地/网络/在线目录级联） | `M1-LIST-003/004/005` |
| 1450–1453 | 延迟 3 秒更新目录 | | `M6-CATALOG-006` |

---

## `dlg/dict/phrase.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 6–13 | 窗体定义 | | — |
| 18–58 | `btnLoad` | 读取系统短语 | `M2-IO-001/003` |
| 21–30 | └ Win10 1703 版本检查 | | `M2-IO-004` |
| 60–106 | `btnSave` | 安装到系统短语 | `M2-INSTALL-001/007/008` |
| 86–89 | └ `Enable Wubi EUDP = 1` | | `M2-INSTALL-004` |
| 108–137 | `loadPhraseFromFile` | 文件加载 | `M2-IO-006/007` |
| 130–135 | └ 二进制/文本分支 | ⚠️ **缺 `else`，二进制结果被覆盖** | `M2-IO-007` |
| 139–150 | `busy()` | | `M2-EDIT-009` |
| 153–246 | `enablePopMenu` | 右键菜单 | `M2-EDIT-006` |
| 156–217 | └ 造词 | ⚠️ **191 行缺 `elseif`，郑码结果重复** | `M2-COIN-001/002/003` |
| 220–236 | └ 保存/加载短语文件 | | `M2-IO-005/006` |
| 238–244 | └ 恢复到范例 | | `M2-EDIT-005` |
| 248–253 | `onDropFiles` | 拖放加载 | `M2-IO-007` |
| 259–266 | 内容持久化 | `config.phrase.editorText` | `M2-EDIT-004` |
| 268–276 | Tab 插入制表符 | | `M2-EDIT-002` |
| 278–283 | `btnHelp` | 格式说明弹窗 | `M2-EDIT-007` |

---

## `dlg/dict/phraseHelp.aardio`

纯静态说明文本（3 类写法示例 + 2 条注意事项）。→ `M2-EDIT-007`

---

## `dlg/dict/wordWeight.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 4–17 | 窗体定义 | 2 个策略开关 | `M1-WEIGHT-003/004` |
| 25 | `chkFixedSingleChar.checked = true` | 默认开启 | `M1-WEIGHT-004` |
| 30–32 | `setLexTitle` | | — |
| 35–83 | `btnUpdate` | 重算执行 | `M1-WEIGHT-001/002` |
| 42–52 | └ 词频文件缺失则自动下载 | | `M1-WEIGHT-002` |
| 57–63 | └ 对编辑器文本重算 | | `M1-WEIGHT-006` |
| 64–75 | └ 对码表文件原地重算 | | `M1-WEIGHT-005` |
| 86–95 | `btnOpenFile` | 选择词频文件 | `M1-WEIGHT-001` |

---

## `dlg/help/etymon.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 4–18 | 窗体定义 | 8 个页签（7 图 + 文本） | `M5-CHART-001` |
| 22–39 | `win.ui.tabs` + skin | | `M5-CHART-001` |
| 41–60 | `showImage` | 字根图下载与缓存 | `M5-CHART-002/005`、`M6-DOWN-006` |
| 62–70 | `onSelchange` | 图/文切换 | `M5-CHART-001`、`M5-TEXT-001` |
| 72–79 | `updateEtymonVersion` | 按系统码表方案定位页签 | `M5-CHART-003` |
| 81–234 | `etymonTxt.text` | **文本字根集合 + 5 套字根歌诀全文** | `M5-TEXT-001/002/003` |
| 237–255 | 事件订阅 | `lex.version.changed` / `showEtymon` | `M5-CHART-003/007` |

---

## `dlg/help/help.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 14–39 | `win.ui.tabs` | 帮助页二级导航（帮助/字根/技巧） | `M7-WIN-004` |
| 41–43 | `showEtymon` | 外部跳转入口 | `M5-CHART-007` |

---

## `dlg/help/sys.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 12–39 | `keyboardShortcuts.text` | **Windows 使用技巧 26 条** | `M5-TIPS-002` |
| 6, 43–45 | `btnHelp` | aardio 推广链接 | `M5-TIPS-004`（P3） |

---

## `dlg/help/wubi.aardio`

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 19–39 | `keyboardShortcuts.text` | **微软五笔使用技巧 8 条** | `M5-TIPS-001` |
| 42–53 | 链接皮肤与回调 | 官网 / GitHub | `M5-ABOUT-001/002` |
| 55–89 | 自动更新 UI | 4 种状态回调 + 手动检查 | `M5-ABOUT-004/005/006`、`M6-UPDATE-002/003/004` |
| 91–92 | `win.versionEx.format()` | 系统版本展示 | `M5-ABOUT-003` |

---

## `sepllingData/build.aardio`

**开发者内部工具**，不进产品。新项目需要等价的资源构建 CLI。

| 行 | 区块 | 说明 | 需求 |
|---|---|---|---|
| 4–15 | 窗体定义 | 版本下拉 + 数据编辑器 + 字根参考 | `M6-BUILD-001` |
| 17–27 | 保存与脏标记 | | `M6-BUILD-001` |
| 29–48 | `btnLoad` | 按版本加载 `sepllingData/{版本}.txt` | `M6-BUILD-001` |
| 34–43 | └ 字根字体加载（092 特判） | | `M6-BUILD-003` |
| 58–85 | `btnSearch` | 数据表内查找 | `M6-BUILD-001` |
| 96–111 | `btnLzma` | **批量 LZMA 打包 7 个方案** | `M6-BUILD-002` |
| 116–143 | `editEtymons.text` | 字根字符参考面板（含 PUA） | `M6-BUILD-004` |

> 目录名 `sepllingData` 为拼写错误（应为 `spellingData`），新项目应更正。

---

## 已废弃文件（P3）

以下 5 个文件**不迁移**，仅记录以保证覆盖完整。

| 文件 | 行数 | 说明 | 需求 |
|---|---:|---|---|
| `lib/app/hotkey.aardio` | 303 | 超级热键配置模板，**全文为注释**，未被任何文件 `import`。历史能力清单见 [M7 §8](./modules/M7-app-shell.md#超级热键的历史用途) | `M7-DEPR-001` |
| `lib/winex/msCandidate.aardio` | 29 | 候选窗口句柄探测（`TextInputHost.exe` / `Microsoft.IME.UIManager`），仅被上者的注释代码引用 | `M7-DEPR-002` |
| `lib/wubi/candidateWindow.aardio` | 7 | 上者的别名，源码标注 `//Deprecated` | `M7-DEPR-003` |
| `lib/ui/chineseNumber.aardio` | 4 | `win.dlg.chineseNumber` 别名，标注 `@Deprecated` | `M7-DEPR-004` |
| `lib/wubi/chineseNumber.aardio` | 4 | 同上，另一命名空间下的重复别名 | `M7-DEPR-005` |

---

## 已知缺陷汇总

实现时**不要照抄**这些位置，并为每条编写回归测试（见 [`02-architecture.md#8`](./02-architecture.md#必须覆盖的原项目缺陷回归)）。

| # | 位置 | 缺陷 | 影响 |
|---:|---|---|---|
| 1 | `lexFile.aardio:1293` | `test("XFXY","线")` 用大写，编码键全小写 | 版本探测中该项**永久失效**，新世纪码表易误判 |
| 2 | `dlg/dict/phrase.aardio:130-135` | 二进制分支后缺 `else` | 拖放 EUDP 文件显示乱码 |
| 3 | `dlg/dict/phrase.aardio:191` | `if(imeVer=="bxm")` 应为 `elseif` | 郑码造词同时跑两套规则，输出重复 |
| 4 | `text.aardio:8` vs `12` | `encode` 转义所有 `\s`，`decode` 只认 4 种 | `%0B`/`%0C` 无法还原 |
| 5 | `lexFile.aardio:135-137` | `if(codeWeight[0])` 永不为真 | 死代码 |
| 6 | `lexFile.aardio:629` | `for(i=1;#words)` 缺递增语句 | 依赖 aardio 隐式行为，语义不清 |
| 7 | `lexNetContents.aardio:12` | `eval()` 反序列化缓存 | **远程代码执行**链路 |
| 8 | `tsfUtil.aardio:41-48, 138-147` | 服务恢复、删除重试均不检查结果 | 失败静默，系统可能停在中间态 |
