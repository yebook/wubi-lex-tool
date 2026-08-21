# M4 — 输入法控制与系统设置

> **模块职责**：微软输入法的启停、双拼方案管理、五笔行为设置（注册表）、以及**替换系统码表所必需的 TSF 重启与文件 ACL 接管流程**。
>
> 这是全项目**风险最高**的模块：涉及系统服务停启、计划任务控制、进程终止、TrustedInstaller 所有权接管。任一环节失败都可能让用户的输入法完全不可用。

## 来源文件

| 文件 | 行数 | 角色 |
|---|---:|---|
| `lib/tsfInput.aardio` | 146 | 输入法启停、状态查询、双拼方案枚举与切换 |
| `lib/tsfUtil.aardio` | 235 | TSF 重启流程、ACL 接管、安装编排 |
| `dlg/setting.aardio` | 329 | 设置面板：注册表读写与 UI 绑定 |
| `lib/ui/doublePinyinMenu.aardio` | 54 | 双拼方案弹出菜单 + 小鹤双拼一键安装 |

## 关键标识符

| 对象 | 标识 |
|---|---|
| 微软五笔 TIP | `0804:{6A498709-E00B-4C45-A018-8F9E4081AE40}{82590C13-F4DD-44F4-BA1D-8667246FDF8E}` |
| 微软拼音 TIP | `0804:{81D4E9C9-1D3B-41BC-9E6C-4B40BF79E35E}{FA550B04-5AD7-411F-A5AC-CA038EC515D7}` |
| 英文键盘布局 | `0409:00000409`（另需处理 `0804:00000409`、`0804:00000804`） |
| 五笔 CLSID | `{6a498709-e00b-4c45-a018-8f9e4081ae40}` |
| 五笔 Profile GUID | `{82590C13-F4DD-44f4-BA1D-8667246FDF8E}` |
| 语言 ID | `0x804`（简体中文） |
| HKL（英文） | `0x04090409` |
| HKL（中文） | `0x08040804` |

---

## 1. 输入法启停（`TIP`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M4-TIP-001` | 启用 / 禁用微软五笔输入法 | P0 | `tsfInput.aardio:12-15` | |
| `M4-TIP-002` | 启用 / 禁用微软拼音输入法 | P1 | `tsfInput.aardio:8-10` | |
| `M4-TIP-003` | 启用 / 禁用英文键盘；启用时同时禁用中文语言下的两个英文布局 | P1 | `tsfInput.aardio:17-22` | 避免同一语言下出现重复的英文布局 |
| `M4-TIP-004` | 查询三者当前启用状态，供 UI 与托盘菜单展示勾选 | P0 | `tsfInput.aardio:26-38` | |
| `M4-TIP-005` | 枚举全部已启用的键盘布局与 TIP，附带其显示名称 | P1 | `main.aardio:201-229` | 分「键盘布局」与「输入处理器」两组展示 |
| `M4-TIP-006` | 读取 / 设置**默认输入法**（`InputMethodOverride`）；再次选中当前项则清空覆盖 | P1 | `tsfInput.aardio:50-58`、`main.aardio:206-212` | 注册表 `HKCU\Control Panel\International\User Profile` |
| `M4-TIP-007` | 读取 / 设置「所有应用使用同一输入法」开关 | P1 | `tsfInput.aardio:40-48`、`main.aardio:195-198` | `SystemParametersInfo` + `SPI_GET/SET_THREADLOCALINPUTSETTINGS`（`0x104E` / `0x104F`） |
| `M4-TIP-008` | 通过 `ITfInputProcessorProfileMgr` 激活 / 停用五笔 Profile | P1 | `tsfUtil.aardio:54-70` | COM 互操作；标志位 `0x20000000` |
| `M4-TIP-009` | 安装完成后请求输入法切换以使新码表立即生效 | P0 | `lex.aardio:128`、`dlg/dict/phrase.aardio:101` | `changeRequest(0x04090409)` |
| `M4-TIP-010` | 展示当前 Windows 显示语言名称，并在输入法开关变更后刷新 | P2 | `setting.aardio:287-298` | 启用/禁用输入法可能改变系统语言列表 |

### ⚠️ `M4-TIP-003` 的副作用

`enableUsKeyboard` 无论启用与否，都会**无条件禁用** `0804:00000409` 与 `0804:00000804` 两个布局（`tsfInput.aardio:19-20`）。这是有意为之（清理中文语言下冗余的英文布局），但对已有自定义布局配置的用户是破坏性操作。

**新实现要求**：在 UI 上明确说明该行为，或提供「仅切换 0409:00000409」的保守模式。

---

## 2. 双拼方案（`DPY`）

微软拼音支持双拼输入。WubiLex 提供方案枚举、切换、以及**小鹤双拼一键安装**（小鹤是社区最流行的第三方方案，Windows 未内置）。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M4-DPY-001` | 枚举全部双拼方案：3 个内置 + 注册表中的用户自定义方案 | P1 | `tsfInput.aardio:60-107` | 内置：微软双拼(0) / 智能ABC(1) / 自然码(3) |
| `M4-DPY-002` | 从自定义方案中识别小鹤双拼：按名称含「小鹤」**或**方案串精确匹配 | P1 | `tsfInput.aardio:89-93` | 双重判据提高识别率 |
| `M4-DPY-003` | 小鹤双拼未安装时，提供一键写入注册表并启用 | P1 | `doublePinyinMenu.aardio:25-40` | README 宣传的亮点功能 |
| `M4-DPY-004` | 启用 / 禁用双拼（写 `Enable Double Pinyin` 与 `DoublePinyinScheme`） | P1 | `tsfInput.aardio:109-125` | |
| `M4-DPY-005` | 切换到指定方案；再次点选当前方案则关闭双拼（toggle 语义） | P1 | `doublePinyinMenu.aardio:19-23` | |
| `M4-DPY-006` | 提供方案菜单，当前生效方案显示勾选；含「禁用双拼」项 | P1 | `doublePinyinMenu.aardio:12-23` | 设置页与托盘菜单共用 |
| `M4-DPY-007` | 提供「打开系统拼音设置」入口 | P2 | `doublePinyinMenu.aardio:43-46` | `ms-settings:regionlanguage-chsime-pinyin` |

### 详述：自定义方案的注册表编码

**位置**：`HKCU\Software\Microsoft\InputMethod\Settings\CHS`
**键名**：`UserDefinedDoublePinyinScheme{N}`，`N` 为 0 起的序号
**方案 ID**：`10 + N`

**值格式**（星号分隔）：

```
小鹤双拼*2*^*iuvdjhcwfg^xmlnpbksqszxkrltvyovt
│         │ │ │
│         │ │ └── 韵母映射表
│         │ └──── 分隔标记
│         └────── 版本 / 类型
└──────────────── 方案显示名（取第一个 * 之前的部分）
```

**空闲 ID 分配**（`tsfInput.aardio:88-100`）：扫描现有方案，取 `max(已用 index) + 1`，起始值 10。

> **风险**：方案串格式为微软内部约定，无公开文档。原项目硬编码了小鹤的完整串。新实现应把方案串作为**数据**（配置文件）而非代码常量，便于后续补充其他方案（自然码变体、搜狗双拼等）。

---

## 3. 五笔行为设置（`REG`）

图形化封装微软五笔的注册表设置项。

### 3.1 输入行为

**注册表位置**：`HKCU\Software\Microsoft\InputMethod\Settings\CHS`

| ID | 需求 | P | 注册表键 | 来源 |
|---|---|---|---|---|
| `M4-REG-001` | 「纯形码输入」/「形码拼音混输」二选一 | P1 | `PinyinMixEnable` | `setting.aardio:26-27, 64-65, 152` |
| `M4-REG-002` | 「默认英文模式」开关（可短按 Shift 切换） | P1 | `Default Mode` | `setting.aardio:9, 66, 153` |
| `M4-REG-003` | 「四码唯一时自动上屏」开关 | P1 | 混输模式下写 `WubiAutoFinalizeForMixEnable`，纯形码模式下写 `Wubi Auto-finalize Enable` | `setting.aardio:17, 76-81, 159-164` |
| `M4-REG-004` | 「允许自定义短语」开关 | P0 | `Enable Wubi EUDP` | `setting.aardio:13, 67, 154` |
| `M4-REG-005` | 「英文标点默认使用半角输入模式」开关 | P1 | `HalfWidthInputModeByDefault` | `setting.aardio:14, 69, 156` |
| `M4-REG-006` | 「中文输入时使用英文标点」开关 | P1 | `UseEnglishPunctuationsInChineseInputMode` | `setting.aardio:16, 68, 157` |
| `M4-REG-007` | 保存时固定写入 `Floating Icon Time Key = 0` | P2 | `Floating Icon Time Key` | `setting.aardio:151` |

> **`M4-REG-003` 的双键设计不可简化**：微软用两个不同的注册表键分别控制混输模式与纯形码模式下的自动上屏行为。UI 上是同一个复选框，但落盘的键取决于当前的混输模式。切换混输模式后必须重新读取对应键的值。

### 3.2 候选窗口

**注册表位置**：`HKCU\Software\Microsoft\InputMethod\CandidateWindow\CHS\2`

| ID | 需求 | P | 说明 | 来源 |
|---|---|---|---|---|
| `M4-REG-008` | 设置候选词每页显示数，有效范围 3–9；`0` 表示固定宽度模式 | P1 | 同时写 `MaxCandidates` 与 `EnableFixedCandidateCountMode` | `setting.aardio:105-111, 136-148` |
| `M4-REG-009` | 输入框支持上下方向键增减数值，并做范围钳制 | P2 | 上键：>9 归 9，<3 归 3；下键：>9 归 9，<3 归 **0** | `setting.aardio:231-259` |
| `M4-REG-010` | 保存时校验范围，越界给出警告气泡并中止保存 | P1 | 「候选词数目不能小于 3 / 大于 9」 | `setting.aardio:120-132` |

**取值语义**：

| `MaxCandidates` | `EnableFixedCandidateCountMode` | 效果 |
|---:|---:|---|
| 3–9 | 1 | 每页固定显示 N 个候选 |
| 0 | 0 | 候选窗口固定宽度，个数随内容浮动 |

### 3.3 输入法外观

**注册表位置**：`HKLM\SOFTWARE\Microsoft\CTF\TIP\{6a498709-...}\LanguageProfile\0x00000804\{82590C13-...}`

⚠️ **HKLM 写入需要管理员权限。**

| ID | 需求 | P | 说明 | 来源 |
|---|---|---|---|---|
| `M4-REG-011` | 自定义输入法在语言栏中的**显示名称**；留空则还原为系统默认 | P2 | 默认值 `@%SystemRoot%\SYSTEM32\input.dll,-5302` | `setting.aardio:83-91, 167-173` |
| `M4-REG-012` | 自定义输入法**显示图标**（`路径,索引` 格式）；留空则还原为系统默认 | P2 | 默认 `%SystemRoot%\system32\InputMethod\Shared\ResourceDll.dll` + 索引 `1` | `setting.aardio:93-95, 175-189` |
| `M4-REG-013` | 提供图标选择对话框，支持 `.ico` / `.exe` / `.dll`；非 `.ico` 自动追加 `,1` | P2 | | `setting.aardio:220-229` |

### 3.4 读取与反馈

| ID | 需求 | P | 说明 | 来源 |
|---|---|---|---|---|
| `M4-REG-014` | 打开设置页时批量读取全部设置项，未设置的键取内置默认值 | P1 | 默认值表见下 | `setting.aardio:53-69` |
| `M4-REG-015` | 系统码表变更后重新读取 `PinyinMixEnable`（安装小鹤音形会改写它） | P1 | | `setting.aardio:70-74` |
| `M4-REG-016` | 保存成功后提示「请按 Alt + Shift 来回切换一下输入法即可」 | P1 | | `setting.aardio:209` |
| `M4-REG-017` | 若勾选了「启用微软五笔」，保存时同时执行一次 TSF 重启 | P1 | | `setting.aardio:192-195` |

**默认值表**（`setting.aardio:54-61`）：

| 键 | 默认 |
|---|---:|
| `Default Mode` | 1 |
| `Enable Wubi EUDP` | 1 |
| `WubiAutoFinalizeForMixEnable` | 1 |
| `Wubi Auto-finalize Enable` | 1 |
| `HalfWidthInputModeByDefault` | 1 |
| `UseEnglishPunctuationsInChineseInputMode` | 0 |
| `MaxCandidates` | 7 |

---

## 4. 系统集成（`SYS`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M4-SYS-001` | 「允许开机启动」开关：注册为**计划任务**（而非启动项），使开机启动时免 UAC 确认 | P1 | `setting.aardio:197-207` | 任务名「WubiLex( 五笔助手 ) 启动任务」，参数 `/tray` |
| `M4-SYS-002` | 打开系统五笔设置页 | P1 | `setting.aardio:215-217`、`main.aardio:165-168` | `ms-settings:regionlanguage-chsime-wubi` |
| `M4-SYS-003` | 打开系统输入法设置页 | P2 | `main.aardio:188-191` | `ms-settings:keyboard` |
| `M4-SYS-004` | 打开系统拼音设置页 | P2 | `doublePinyinMenu.aardio:43-46` | `ms-settings:regionlanguage-chsime-pinyin` |
| `M4-SYS-005` | 打开系统屏幕键盘 | P2 | `main.aardio:235-238` | `osk.exe`，需 WOW64 重定向处理 |
| `M4-SYS-006` | 反转鼠标左右键（托盘菜单项，带勾选状态） | P2 | `main.aardio:240-243` | `SwapMouseButton` / `GetSystemMetrics(23)` |

> `M4-SYS-001` 的**计划任务方案是关键设计**：本应用需要管理员权限运行，普通启动项会在每次开机弹 UAC。注册为「以最高权限运行」的计划任务可绕过。新实现必须保留此方案，不可退化为注册表 `Run` 项。

---

## 5. TSF 重启与 ACL 接管（`TSF`）

**全项目最高风险的代码路径。** 替换 `%WINDIR%\InputMethod\CHS\ChsWubi*.lex` 需要：

1. 让所有持有该文件句柄的进程释放它（输入法宿主、平板服务）
2. 从 TrustedInstaller 手中夺取文件所有权
3. 替换文件
4. 把所有权还回去
5. 恢复被停掉的服务与任务

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M4-TSF-001` | 提供「安全操作输入法文件」的编排流程，接受一个回调在停机窗口内执行实际操作 | P0 | `tsfUtil.aardio:21-52` | 见下方流程图 |
| `M4-TSF-002` | 停止 `TabletInputService`，操作完成后恢复为手动启动并重新启动 | P0 | `tsfUtil.aardio:25-27, 41-42` | 停止前先置为「禁用」防止自动重启 |
| `M4-TSF-003` | 结束计划任务 `\Microsoft\Windows\TextServicesFramework\MsCtfMonitor`，完成后重新运行 | P0 | `tsfUtil.aardio:29-31, 44-46` | 该任务负责拉起 `ctfmon.exe` |
| `M4-TSF-004` | 终止 `ctfmon.exe` 与 `ChsIME.exe` 进程 | P0 | `tsfUtil.aardio:33-34` | |
| `M4-TSF-005` | 流程开始前禁用微软五笔，结束后重新启用 | P0 | `tsfUtil.aardio:23, 50` | |
| `M4-TSF-006` | 流程开始前保存用户语言列表，结束后还原 | P0 | `tsfUtil.aardio:22, 51` | 禁用/启用 TIP 会改动语言列表 |
| `M4-TSF-007` | 文件 ACL 接管：夺取所有权 → 授予 Administrators 完全控制 → 执行操作 → 归还所有权给 `NT SERVICE\TrustedInstaller` | P0 | `tsfUtil.aardio:72-88` | 用 `takeown` + `icacls` |
| `M4-TSF-008` | 删除目标文件时循环重试最多 10 次（每次间隔 1 秒），期间反复终止占用进程 | P0 | `tsfUtil.aardio:138-147` | |
| `M4-TSF-009` | 文件不存在时跳过 ACL 接管直接执行操作 | P0 | `tsfUtil.aardio:73` | |
| `M4-TSF-010` | **【新增】流程各阶段失败时的显式错误上报与状态恢复** | P0 | 原项目**无此能力** | 见下方详述 |
| `M4-TSF-011` | **【新增】启动时检测是否具备管理员权限，不足时明确提示** | P0 | 原项目靠 `//RUNAS//` 强制提权，无检查 | |
| `M4-TSF-012` | **【新增】流程执行期间向 UI 上报阶段进度** | P1 | 原项目只有笼统的「请耐心等待」 | 7 个阶段可分别上报 |

### 完整流程

```
┌─ 前置 ──────────────────────────────────────────────┐
│ 1. 保存用户语言列表                                   │
│ 2. 禁用微软五笔 TIP                                   │
└──────────────────────────┬──────────────────────────┘
                           ▼
┌─ 停机窗口 ──────────────────────────────────────────┐
│ 3. TabletInputService: 置为禁用 → 停止                │
│ 4. schtasks /End MsCtfMonitor                        │
│ 5. kill ctfmon.exe, ChsIME.exe                       │
│                                                      │
│ 6. ┌─ ACL 接管 (ChsWubiNew.lex) ─────────────┐       │
│    │  takeown → icacls /grant Admins:(F)     │       │
│    │  ┌─ ACL 接管 (ChsWubi.lex) ──────────┐  │       │
│    │  │  takeown → icacls /grant          │  │       │
│    │  │  ┌─ 实际操作 ─────────────────┐   │  │       │
│    │  │  │ • 循环 10 次: kill + 删除   │   │  │       │
│    │  │  │ • 写入新码表               │   │  │       │
│    │  │  │ • 双文件同步               │   │  │       │
│    │  │  │ • (可选) 写入短语库         │   │  │       │
│    │  │  └────────────────────────────┘   │  │       │
│    │  │  icacls /setowner TrustedInstaller │  │       │
│    │  └───────────────────────────────────┘  │       │
│    │  icacls /setowner TrustedInstaller       │       │
│    └─────────────────────────────────────────┘       │
│                                                      │
│ 7. TabletInputService: 置为手动 → 启动                │
│ 8. schtasks /Run MsCtfMonitor                        │
│ 9. sleep 1s                                          │
└──────────────────────────┬──────────────────────────┘
                           ▼
┌─ 后置 ──────────────────────────────────────────────┐
│ 10. 启用微软五笔 TIP                                  │
│ 11. 还原用户语言列表                                  │
│ 12. changeRequest 触发输入法切换                      │
└─────────────────────────────────────────────────────┘
```

### 详述：`M4-TSF-010` 错误处理（新增需求）

**原项目的问题**：

| 环节 | 原项目行为 | 后果 |
|---|---|---|
| 服务停止失败 | 不检查返回值 | 文件被占用，后续删除必然失败 |
| `schtasks` 失败 | 捕获 `ok,out,err` 但**不判断** | ctfmon 随时可能被拉起重新占用文件 |
| 删除文件失败 | 重试 10 次后**不检查结果**，直接继续 | 写入失败或写入到旧文件上 |
| `takeown` 失败 | 返回 `null,err`，但上层只做 `if(!out) return null,err` | 错误信息未展示给用户，UI 只说「安装失败请重试一次即可」 |
| 服务恢复失败 | 不检查 | **用户的触摸键盘/手写面板永久不可用** |
| 进程被中途杀死 | 无任何保护 | 服务停在禁用状态、文件所有权停在 Administrators |

**新实现要求**：

1. **每一步都检查结果**，失败即中止并向上返回结构化错误（阶段 + 系统错误码 + 可读描述）
2. **实现 RAII / drop guard**：无论成功失败或 panic，析构时都必须
   - 把 `TabletInputService` 恢复为「手动启动」并尝试启动
   - 重新运行 `MsCtfMonitor` 计划任务
   - 把已接管的文件所有权归还 `TrustedInstaller`
   - 还原用户语言列表
   - 重新启用五笔 TIP
3. **写入前备份**（见 [`M1-INSTALL-013`](./M1-lex-table.md)），写入后校验，校验失败自动回滚
4. **进程崩溃保护**：把「进入停机窗口」的事实写入配置文件，下次启动时若发现未正常退出，主动执行恢复流程并提示用户

### Rust 实现要点

| 能力 | 建议方案 |
|---|---|
| 服务控制 | `windows` crate 的 `Win32::System::Services`（`OpenSCManager` / `ControlService` / `ChangeServiceConfig`），**不要**调 `sc.exe` |
| 计划任务 | Task Scheduler 2.0 COM（`ITaskService`），或退而求其次调 `schtasks.exe` |
| 进程终止 | `Win32::System::Threading::OpenProcess` + `TerminateProcess`，配合 `ToolHelp32` 枚举 |
| 所有权与 ACL | `Win32::Security::Authorization`（`SetNamedSecurityInfo`），比调 `takeown`/`icacls` 更可靠且无子进程开销 |
| TSF Profile | `windows` crate 的 `ITfInputProcessorProfileMgr`（需启用 `Win32_UI_TextServices` feature） |
| 注册表 | `windows` crate 的 `Win32::System::Registry`，或 `winreg` |
| 提权 | Tauri 应用清单声明 `requireAdministrator` |

> **强烈建议用 API 而非命令行工具**：`takeown` / `icacls` / `schtasks` 会拉起子进程、输出需解析、错误码语义模糊，且在非中文/非英文系统上输出文本不同。原项目大量依赖这些工具正是其错误处理薄弱的根因之一。

---

## 6. 数据依赖

| 依赖 | 来源 | 说明 |
|---|---|---|
| 码表文件写入 | [M1](./M1-lex-table.md) | `M4-TSF-001` 的回调内容 |
| 短语文件写入 | [M2](./M2-phrase.md) | 同上 |
| 配置持久化 | [M7](./M7-app-shell.md) | `settings.systemStartup` |
| 事件广播 | [M7](./M7-app-shell.md) | `system.lex.changed` / `system.phrase.changed` / `setting.eudp.changed` |
| 系统版本探测 | [M7](./M7-app-shell.md) | 部分设置项仅在特定 Windows 版本可用 |

**反向被依赖**：M1 与 M2 的全部安装需求均经由本模块执行。

---

## 7. 对外接口草案

### Tauri Commands

| Command | 用途 |
|---|---|
| `ime_status` | 五笔 / 拼音 / 英文键盘的启用状态 |
| `ime_enable` | 启停指定输入法 |
| `ime_list_profiles` | 枚举已启用的布局与 TIP |
| `ime_default_get` / `ime_default_set` | 默认输入法 |
| `ime_thread_local_get` / `ime_thread_local_set` | 所有应用同一输入法 |
| `dpy_schemes` | 枚举双拼方案 |
| `dpy_set` | 启用/切换/禁用双拼 |
| `dpy_install_xhup` | 安装小鹤双拼 |
| `wubi_settings_get` / `wubi_settings_set` | 五笔行为设置批量读写 |
| `ime_appearance_get` / `ime_appearance_set` | 显示名称与图标 |
| `autostart_get` / `autostart_set` | 开机启动 |
| `open_system_settings` | 打开系统设置页（参数为页面标识） |
| `tsf_guarded_write` | **内部**：在停机窗口内执行文件写入（M1/M2 调用） |
| `tsf_recover` | 手动触发状态恢复 |

### Events

| Event | 载荷 |
|---|---|
| `ime://status-changed` | `{ wubi, pinyin, en }` |
| `ime://settings-changed` | 变更的设置项 |
| `tsf://phase` | `{ phase: 1..12, message }` — 停机窗口内的阶段进度 |
| `tsf://recovery-needed` | `{ reason }` — 检测到上次未正常退出 |

---

## 8. 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 停机窗口中途失败导致输入法不可用 | **极高** | RAII 恢复守卫 + 备份回滚 + 崩溃后自恢复，见 `M4-TSF-010` |
| `TabletInputService` 未恢复导致触摸键盘永久失效 | **高** | 恢复守卫中无条件恢复服务状态 |
| 文件所有权停留在 Administrators | **高** | 恢复守卫中无条件归还 TrustedInstaller |
| 依赖 `takeown`/`icacls`/`schtasks` 的输出解析在非中文系统失效 | **高** | 改用 Win32 API |
| HKLM 写入需管理员权限，权限不足时静默失败 | 中 | 启动时检查权限，`M4-TSF-011` |
| `ITfInputProcessorProfileMgr` COM 互操作复杂度 | 中 | `windows` crate 已有绑定；封装为薄适配层并单独验证 |
| 双拼方案串为微软内部未文档化格式 | 中 | 作为配置数据管理，不硬编码 |
| Windows 版本差异导致注册表键语义变化 | 中 | 按版本分档；写入前读取原值，未知键不覆盖 |
| `enableUsKeyboard` 的无条件禁用副作用 | 低 | UI 明示 + 提供保守模式 |

---

## 9. 源码索引

完整反向索引见 [`03-source-index.md`](../03-source-index.md)。

| 域 | 核心逻辑 | 界面入口 |
|---|---|---|
| `TIP` | `lib/tsfInput.aardio:8-58`（启停 / 状态 / 默认输入法 / 线程本地）；`lib/tsfUtil.aardio:54-70`（**Profile 激活**） | `dlg/setting.aardio:266-314`；`main.aardio:154-231`（托盘菜单） |
| `DPY` | `lib/tsfInput.aardio:60-132`（**方案枚举与切换**） | `lib/ui/doublePinyinMenu.aardio:8-49`（菜单 + **小鹤安装** `25-40`） |
| `REG` | — | `dlg/setting.aardio:52-111`（读取）、`119-211`（**保存全部**）、`220-259`（图标与候选数） |
| `SYS` | — | `dlg/setting.aardio:197-207`（**开机自启计划任务**）、`215-217`；`main.aardio:188-243` |
| `TSF` | `lib/tsfUtil.aardio:21-52`（**停机窗口编排**）、`72-88`（**ACL 接管**）、`138-147`（删除重试） | `dlg/dict/lex.aardio:102-142`（调用方） |

### 关键常量位置

| 常量 | 位置 |
|---|---|
| 微软五笔 TIP ID | `lib/tsfInput.aardio:14` |
| 微软拼音 TIP ID | `lib/tsfInput.aardio:9` |
| 五笔 CLSID / Profile GUID | `lib/tsfUtil.aardio:58-59`、`dlg/setting.aardio:83` |
| 小鹤双拼方案串 | `lib/ui/doublePinyinMenu.aardio:30` |
| 注册表默认值表 | `dlg/setting.aardio:54-61` |
| 输入法显示名/图标默认值 | `dlg/setting.aardio:89, 93, 172, 187-188` |

### ⚠️ 不要照抄的位置

| 位置 | 问题 | 对应需求 |
|---|---|---|
| `lib/tsfUtil.aardio:29-31, 44-48` | `schtasks` / 服务操作**不检查返回值** | `M4-TSF-010` |
| `lib/tsfUtil.aardio:75-87` | 依赖 `takeown` / `icacls` 的文本输出 | `M4-TSF-007`、`NFR-COMPAT-009` |
| `lib/tsfUtil.aardio:138-147` | 删除重试后不校验是否成功 | `M4-TSF-008` |
| `lib/tsfInput.aardio:19-20` | 无条件禁用两个中文语言下的英文布局 | `M4-TIP-003` |

---

## 需求统计

| 域 | 条目数 | P0 | P1 | P2 |
|---|---:|---:|---:|---:|
| `TIP` | 10 | 3 | 6 | 1 |
| `DPY` | 7 | 0 | 6 | 1 |
| `REG` | 17 | 1 | 11 | 5 |
| `SYS` | 6 | 0 | 2 | 4 |
| `TSF` | 12 | 11 | 1 | 0 |
| **合计** | **52** | **15** | **26** | **11** |
