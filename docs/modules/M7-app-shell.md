# M7 — 应用外壳

> **模块职责**：进程与窗口生命周期、导航容器、系统托盘、全局热键、配置持久化、模块间事件分发、后台任务调度。
>
> **不含任何领域逻辑**：本模块是承载 M1–M6 的骨架。

## 来源文件

| 文件 | 行数 | 角色 |
|---|---:|---|
| `main.aardio` | 321 | 单实例、主窗口、Tab 容器、托盘菜单、全局热键、更新入口 |
| `lib/config.aardio` | 25 | 配置持久化 |
| `dlg/dict/dict.aardio` | 80 | 词库页二级 Tab 容器 |
| `dlg/help/help.aardio` | 45 | 帮助页二级 Tab 容器 |
| `lib/style.aardio` | 134 | UI 皮肤令牌（内容归 [`21-ui-ux.md`](../21-ui-ux.md)） |
| `default.aproj` | — | 应用元数据（名称、图标、版本、公司） |

---

## 1. 进程与实例（`INST`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-INST-001` | 单实例运行：以全局唯一标识检测已有实例 | P0 | `main.aardio:19-25` | 原项目用 `win.ui.atom` + GUID |
| `M7-INST-002` | 检测到已有实例时，将其窗口置前并退出本次启动 | P0 | `main.aardio:21-25` | |
| `M7-INST-003` | 以**管理员权限**运行 | P0 | `main.aardio:1`（`//RUNAS//`）、`default.aproj` | 系统码表替换与 HKLM 写入的前提 |
| `M7-INST-004` | 支持 `/tray` 启动参数：静默启动到托盘，不显示主窗口 | P1 | `main.aardio:303-313` | 开机自启使用此参数 |
| `M7-INST-005` | 应用元数据：名称、图标、版本号、公司、版权 | P1 | `default.aproj:2` | 版本 `12.1.0.0`，输出 `WubiLex.exe` |
| `M7-INST-006` | **【新增】崩溃/异常退出检测与状态恢复** | P0 | 原项目**无** | 配合 [`M4-TSF-010`](./M4-ime-control.md#详述m4-tsf-010-错误处理新增需求) |

### `M7-INST-003` 管理员权限的取舍

原项目**整个进程**都以管理员运行。代价：

- 拖放文件需额外放行 UIPI（`process.admin.enableDropMsg`，`lex.aardio:77`）
- 无法通过普通启动项自启，须用计划任务（[`M4-SYS-001`](./M4-ime-control.md)）
- 应用内的浏览器控件/网络请求都在高权限下运行，攻击面扩大

**新实现可选方案**：

| 方案 | 说明 | 建议 |
|---|---|---|
| A. 整进程提权（同原项目） | 实现最简单 | 一期可行 |
| **B. 主进程普通权限 + 提权辅助进程** | 仅在执行 TSF/ACL/HKLM 操作时按需拉起提权子进程 | 更安全，但需设计 IPC 与提权时机；建议作为演进目标 |

选 A 时必须在 [`20-nonfunctional.md`](../20-nonfunctional.md) 中记录其安全影响。

---

## 2. 窗口与导航（`WIN`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-WIN-001` | 无边框主窗口 + 自定义标题栏，含应用图标与版本号 | P0 | `main.aardio:6-16, 37-38` | |
| `M7-WIN-002` | 顶部一级导航：**词库 / 反查 / 设置 / 帮助** | P0 | `main.aardio:42-68` | |
| `M7-WIN-003` | 词库页左侧二级导航：**码表 / 短语** | P0 | `dict.aardio:14-37` | |
| `M7-WIN-004` | 帮助页左侧二级导航：**帮助 / 字根 / 技巧** | P1 | `help.aardio:14-39` | |
| `M7-WIN-005` | 提供跳转到 ImTip 的入口：已运行则激活，未安装则打开其官网 | P2 | `dict.aardio:57-78` | 见下方说明 |
| `M7-WIN-006` | 窗口内控件随窗口尺寸自适应布局 | P0 | 各窗体的 `dl/dt/dr/db` 锚定属性 | |
| `M7-WIN-007` | 点击关闭按钮**最小化到托盘**而非退出 | P0 | `main.aardio:82-87` | |
| `M7-WIN-008` | `Ctrl + W` 最小化到托盘（主窗口与反查页均生效） | P1 | `main.aardio:315-319`、`spelling.aardio:587-590` | |
| `M7-WIN-009` | 最小化时释放进程工作集内存 | P2 | `main.aardio:76-77` | Tauri/WebView2 下需另行评估效果 |
| `M7-WIN-010` | 最小化时把窗口从任务栏移除（切换为工具窗口样式） | P1 | `main.aardio:73` | |
| `M7-WIN-011` | **【新增】记忆窗口尺寸与位置** | P1 | 原项目**无** | |
| `M7-WIN-012` | **【新增】关闭行为可配置：最小化到托盘 / 直接退出** | P1 | 原项目硬编码为最小化 | 部分用户不希望常驻 |

### `M7-WIN-005` ImTip 集成

原项目在词库页导航栏放了一个 ImTip 入口（`dict.aardio:8, 57-78`）：

1. 用固定 atom `E474890D-1DFA-4575-B456-7B46C15665DC.imtip` 查找已运行的 ImTip 窗口
2. 找到则发送 `WM_COPYDATA` 消息 `"main"` 并置前
3. 未运行则在本机缓存中查找 `imtip.exe` 并启动
4. 都失败则打开 <https://imtip.aardio.com>

这是同作者产品间的交叉推广。ImTip 承接了本项目移除的「超级热键」功能（见 [§7](#7-已废弃模块p3)）。

**新项目建议**：降为 P2 或移除。若保留，应改为「相关工具」推荐位而非一级导航项。

---

## 3. 系统托盘（`TRAY`）

托盘菜单是原项目**功能密度最高**的界面 —— 24 个菜单项，覆盖了绝大部分日常操作，用户无需打开主窗口。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-TRAY-001` | 最小化时创建托盘图标 | P0 | `main.aardio:71-80` | |
| `M7-TRAY-002` | 左键单击托盘图标还原并置前主窗口；若当前在反查页则聚焦输入框 | P0 | `main.aardio:260-267` | |
| `M7-TRAY-003` | 右键弹出完整功能菜单 | P0 | `main.aardio:100-259` | 见下方菜单结构 |
| `M7-TRAY-004` | 菜单项实时反映当前状态（输入法开关、鼠标键、默认输入法均带勾选） | P0 | `main.aardio:163, 176, 180, 198, 212, 243` | 每次弹出时重新查询状态 |
| `M7-TRAY-005` | 提供退出应用入口 | P0 | `main.aardio:252-254` | |
| `M7-TRAY-006` | 退出时清理托盘图标 | P0 | `main.aardio:89-93` | |
| `M7-TRAY-007` | `/tray` 启动时延迟 3 秒创建托盘图标 | P2 | `main.aardio:304-309` | 规避开机时托盘尚未就绪 |
| `M7-TRAY-008` | **【新增】托盘悬停提示展示当前系统码表方案** | P2 | 原项目无 | |

### 托盘菜单结构

```
设置输入法                                    → 设置页
─────────────────────────────
管理码表                                      → 词库页 / 码表
管理短语                                      → 词库页 / 短语
─────────────────────────────
字根反查                        Ctrl + F2     → 反查页 + 聚焦
└──  字根键位图                                → 帮助页 / 字根
─────────────────────────────
启用微软五笔 ( 码表：五笔86 )        ☑         → M4-TIP-001
└──  系统设置 / 五笔                           → ms-settings:regionlanguage-chsime-wubi
启用英文键盘                        ☑         → M4-TIP-003
─────────────────────────────
启用微软拼音                        ☑         → M4-TIP-002
└──  双拼方案                       ▸         → M4-DPY-006 子菜单
─────────────────────────────
打开系统输入法设置                             → ms-settings:keyboard
设置默认输入法                      ▸         → 子菜单（见下）
─────────────────────────────
打开屏幕键盘                                   → osk.exe
反转鼠标左右按键                    ☑         → M4-SYS-006
─────────────────────────────
官方网站                                       → 浏览器打开
─────────────────────────────
退出
```

**「设置默认输入法」子菜单**（`main.aardio:194-231`）：

```
所有应用使用同一输入法              ☑         → M4-TIP-007
─────────────────────────────
（键盘布局列表，逐项可选，当前默认项勾选）
（输入处理器列表，逐项可选，当前默认项勾选）
```

> 菜单项动态生成：先列出全部 `LOTP_KEYBOARDLAYOUT` 类型，再列出全部 `LOTP_INPUTPROCESSOR` 类型，各自取显示名（`main.aardio:203-228`）。

### 反查热键的动态显示

「字根反查」菜单项会拼接当前配置的热键组合（`main.aardio:124-145`）：解析修饰键位掩码为 `Ctrl` / `Alt` / `Shift`，加上主键名，用 ` + ` 连接，以 Tab 分隔追加到菜单文本后。

**Tauri 实现注意**：`tauri-plugin-tray` 的菜单 API 支持勾选项与子菜单，但**动态重建整个菜单**（每次右键都重新查询系统状态）需要在 `on_tray_icon_event` 中处理，或改用「弹出时刷新」策略。

---

## 4. 全局热键与快捷键自定义（`HOTKEY` / `KEYMAP`）

### 4.1 热键注册底层（`HOTKEY`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-HOTKEY-001` | 提供全局热键注册 / 注销能力，支持运行时变更 | P0 | `main.aardio:271-294` | 变更时先注销旧的再注册新的 |
| `M7-HOTKEY-002` | 热键触发时按前台窗口归属决定行为：本应用则最小化，否则唤起并聚焦 | P0 | `main.aardio:279-289` | 实现「按同一热键来回切换」 |
| `M7-HOTKEY-003` | 注册应用内快捷键 | P1 | `main.aardio:315-319` | 原项目仅有 `Ctrl + W` |
| `M7-HOTKEY-004` | **【新增】注册失败（热键被占用）时上报，不静默失败** | P1 | 原项目不检查返回值 | |

### 4.2 快捷键自定义（`KEYMAP`）

> **原项目仅支持自定义一个热键**（反查唤起，`dlg/spelling.aardio:545-565`），其余全部硬编码。新项目提供**统一的快捷键映射系统**。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-KEYMAP-001` | 建立统一的**动作注册表**：所有可绑定动作集中声明（ID、名称、分组、默认绑定、作用域） | P1 | 【新增】 | 单一事实来源，UI 与注册逻辑都从它生成 |
| `M7-KEYMAP-002` | 区分两类作用域：**全局热键**（系统级，应用未聚焦也生效）与**应用内快捷键** | P1 | 【新增】 | 二者的注册机制与冲突域不同 |
| `M7-KEYMAP-003` | 提供快捷键设置界面：按分组列出全部动作、当前绑定、是否为默认值 | P1 | 【新增】 | 见 [`UX-SCREEN-045`](../21-ui-ux.md#57-设置) |
| `M7-KEYMAP-004` | 支持录制组合键（修饰键 + 主键），录制中实时显示按下的组合 | P1 | `spelling.aardio:8, 557` | 原项目用 Win32 `hotkey` 控件 |
| `M7-KEYMAP-005` | **应用内冲突检测**：与其他动作绑定重复时高亮并提示冲突对象 | P1 | 【新增】 | 保存前拦截 |
| `M7-KEYMAP-006` | **系统占用检测**：全局热键注册失败时明确提示「该组合已被其他程序占用」，并保留原绑定 | P1 | 【新增】 | 解决 `M7-HOTKEY-004` 的用户可见性 |
| `M7-KEYMAP-007` | 支持**恢复单项默认值**与**全部恢复默认** | P1 | 【新增】 | |
| `M7-KEYMAP-008` | 支持清除绑定（置空），置空后该动作仅能通过菜单/命令面板触发 | P2 | 【新增】 | |
| `M7-KEYMAP-009` | 快捷键配置持久化 | P1 | 【新增】 | 配置键 `keymap.bindings` |
| `M7-KEYMAP-010` | 变更**即时生效**，无需重启 | P1 | `spelling.aardio:561`（`publish("spellingHotkeyChange")`） | 复用事件驱动重注册机制 |
| `M7-KEYMAP-011` | 默认绑定按**可用性**设定，不沿袭原项目的选择 | P1 | 【新增】 | 见下方默认绑定表与理由 |
| `M7-KEYMAP-012` | 支持导出 / 导入快捷键方案 | P2 | 【新增】 | 便于换机迁移 |
| `M7-KEYMAP-013` | 拒绝绑定系统保留组合（`Ctrl+Alt+Del`、`Win+L` 等），给出明确提示 | P1 | 【新增】 | 这些组合系统不允许应用捕获 |

### 默认绑定表

按可用性原则重新设定。`≠` 标记为**刻意不同于原项目**的选择。

| 动作 ID | 名称 | 默认绑定 | 作用域 | 理由 |
|---|---|---|---|---|
| `spelling.toggle` | 反查唤起 / 返回原窗口 | `Ctrl + F2` | 全局 | 全局热键须避开应用内高频组合；F 键组合冲突率最低 |
| `window.hide` | 最小化到托盘 | `Ctrl + Shift + H` **≠** | 应用内 | 原项目用 `Ctrl + W`，但该组合在所有平台都意味「关闭」，挪用会误导用户 |
| `window.toggle` | 显示 / 隐藏主窗口 | *(未绑定)* | 全局 | 与 `spelling.toggle` 场景重叠，留给用户自选 |
| `command.palette` | 命令面板 | `Ctrl + K` | 应用内 | 现代工具的事实标准 |
| `search.global` | 全局搜索 | `Ctrl + F` | 应用内 | 应用栏搜索；编辑器聚焦时由 `editor.find` 接管 |
| `editor.find` | 编辑器内查找 | `Ctrl + F` | 编辑器上下文 | 上下文相关绑定，与上一条不冲突 |
| `editor.findNext` | 查找下一个 | `F3` | 编辑器上下文 | Windows 通用 |
| `lex.save` | 保存码表 | `Ctrl + S` | 码表页 | 通用 |
| `lex.install` | 安装到系统码表 | `Ctrl + Shift + I` | 码表页 | Install |
| `nav.overview` | 切换到概览 | `Ctrl + 1` | 应用内 | 7 个一级入口按顺序绑 `Ctrl+1..7` |
| `nav.lexicons` | 切换到码表 | `Ctrl + 2` | 应用内 | |
| `nav.phrases` | 切换到短语 | `Ctrl + 3` | 应用内 | |
| `nav.lookup` | 切换到反查 | `Ctrl + 4` | 应用内 | |
| `nav.radicals` | 切换到字根 | `Ctrl + 5` | 应用内 | |
| `nav.learning` | 切换到学习 | `Ctrl + 6` | 应用内 | |
| `nav.settings` | 切换到设置 | `Ctrl + 7` | 应用内 | |
| `nav.back` | 返回上一级 | `Esc` / `Alt + ←` | 应用内 | 对应 [`UX-IA-009`](../21-ui-ux.md) |
| `theme.toggle` | 切换深浅色主题 | *(未绑定)* | 应用内 | 低频，留给用户 |
| `learn.pause` | 暂停自学习 | *(未绑定)* | 全局 | 见 [`M8-PRIV-002`](./M8-self-learning.md) |

> **原项目仅有两个绑定**（`Ctrl+F2` 反查、`Ctrl+W` 最小化），其中后者已按上表理由改掉。其余全部为新设计。

### 动作注册表结构

```rust
struct ActionDef {
    id: &'static str,          // "spelling.toggle"
    name_key: &'static str,    // i18n 键
    group: ActionGroup,        // 导航 / 词库 / 编辑 / 反查 / 应用 / 自学习
    scope: Scope,              // Global | InApp
    default_binding: Option<KeyCombo>,
    reserved: bool,            // 是否禁止用户改绑
}
```

**这张表是 UI、注册逻辑、命令面板三者的共同数据源** —— 新增一个动作只需在此登记一次，快捷键设置页与命令面板自动包含它。

具体的反查热键行为见 [`M3-HOTKEY-*`](./M3-reverse-lookup.md#7-全局热键hotkey)。

---

## 5. 配置持久化（`CONF`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-CONF-001` | 提供键值型配置存储，支持分组、序列化嵌套结构、进程退出时自动保存 | P0 | `config.aardio:1-3` | 原项目落盘 `%APPDATA%/aardio/std/wubi-lex-tool` |
| `M7-CONF-002` | 首次运行时为缺失项写入默认值 | P0 | `config.aardio:5-15` | |
| `M7-CONF-003` | 支持分组即时保存（不必等到退出） | P0 | `lexContents.aardio:51, 67`（`config.lex.save()`） | |
| `M7-CONF-004` | **【新增】配置文件损坏时回退到默认值并备份损坏文件** | P1 | 原项目**无** | |
| `M7-CONF-005` | **【新增】配置 schema 版本化与升级迁移** | P1 | 原项目**无** | |
| `M7-CONF-006` | **【新增】配置路径迁移到应用自有目录** | P0 | 原项目复用 aardio 路径 | 见 [`M6-CACHE-006`](./M6-resource-sync.md) |
| `M7-CONF-007` | **【新增】提供配置导出/导入，便于迁移与备份** | P2 | 原项目无 | |

### 配置项清单

| 键 | 类型 | 默认值 | 归属 | 来源 |
|---|---|---|---|---|
| `lex.names` | 有序字符串数组 | `[]` | [M1](./M1-lex-table.md) | `config.aardio:9-11` |
| `lex.files` | 映射（名称 → 路径 或 URL） | `{}` | [M1](./M1-lex-table.md) | `config.aardio:5-7` |
| `hotkey.spelling` | `[修饰键掩码, 虚拟键码]` | `[MOD_CONTROL(2), VK_F2(0x71)]` | [M3](./M3-reverse-lookup.md) | `config.aardio:13-15` |
| `settings.systemStartup` | 布尔 | `false` | [M4](./M4-ime-control.md) | `setting.aardio:199, 212` |
| `phrase.editorText` | 字符串 | 内置默认短语库 | [M2](./M2-phrase.md) | `dlg/dict/phrase.aardio:61, 260` |

### 新增配置项建议

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `schemaVersion` | 整数 | `1` | 配置迁移用 |
| `keymap.bindings` | 映射（动作 ID → 组合键） | 见 [默认绑定表](#默认绑定表) | `M7-KEYMAP-009` |
| `window.bounds` | 矩形 | 居中 | `M7-WIN-011` |
| `window.closeAction` | 枚举 | `minimize` | `M7-WIN-012` |
| `ui.theme` | 枚举 | `system` | 深/浅色，见 [`21-ui-ux.md`](../21-ui-ux.md) |
| `ui.locale` | 字符串 | `zh-CN` | i18n |
| `network.mirror` | URL | 官方源 | [`M6-DOWN-013`](./M6-resource-sync.md) |
| `network.autoUpdate` | 布尔 | `true` | [`M6-UPDATE-007`](./M6-resource-sync.md) |
| `network.onlineSuggest` | 布尔 | **`false`** | [`M3-INPUT-003`](./M3-reverse-lookup.md) 在线联想，默认关闭 |
| `backup.retain` | 整数 | `5` | 保留的系统码表备份份数 |

---

## 6. 事件总线（`BUS`）

原项目用全局 `publish` / `subscribe` 在各窗体间广播状态变更。这是其模块解耦的唯一手段，新项目需要等价机制（Tauri 的 `emit` / `listen`）。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-BUS-001` | 提供跨模块的发布/订阅事件机制 | P0 | `main.aardio:271, 296` 等 | |
| `M7-BUS-002` | 实现下表全部事件，语义与触发时机对齐 | P0 | 见下表 | |
| `M7-BUS-003` | **【新增】事件负载强类型化** | P1 | 原项目为无类型可变参数 | Rust 侧定义 `serde` 结构 |

### 事件清单

| 事件 | 触发方 | 订阅方 | 语义 |
|---|---|---|---|
| `wubi.system.lex.changed` | [M1](./M1-lex-table.md) 安装完成 | M1（刷新版本）、M3（重载码表）、M4（刷新设置） | 系统码表已变更 |
| `wubi.system.lex.version.changed` | M1 版本探测完成 | [M5](./M5-etymon-help.md)（切换字根图） | 携带方案代号与显示名 |
| `wubi.system.phrase.changed` | [M2](./M2-phrase.md) 安装完成 | M3（重载短语表）、M4（刷新状态） | 系统短语库已变更 |
| `wubi.setting.eudp.changed` | M1 / M2 自动开启 EUDP 时 | M4（同步复选框） | 携带布尔值 |
| `spellingHotkeyChange` | M3 热键设置变更 | M7（重新注册热键） | |
| `spellingFocus` | M7 托盘/热键唤起 | M3（聚焦输入框） | |
| `showEtymon` | M3 按钮 / M7 托盘菜单 | M5（切到字根图页） | |
| `beforeAddLexFile` | M1 开始添加码表 | M1 UI（禁用按钮 + 进度指示） | |
| `afterLexListUpdated` | M1 添加完成 | M1 UI（刷新列表；失败则回滚条目） | 携带成功标志与名称 |

### 新增事件建议

| 事件 | 语义 |
|---|---|
| `task://progress` | 统一的长任务进度（`{ task_id, module, phase, percent, message }`） |
| `task://done` | 任务结束（成功/失败/取消） |
| `tsf://recovery-needed` | 检测到上次未正常退出，见 [`M4-TSF-010`](./M4-ime-control.md) |
| `resource://error` | 资源获取失败 |

---

## 7. 后台任务（`TASK`）

原项目的所有耗时操作都通过 `thread.invoke`（后台线程）或 `win.invoke`（消息泵内执行）避免阻塞 UI。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M7-TASK-001` | 全部耗时操作在后台执行，UI 线程不阻塞 | P0 | 全项目大量使用 | |
| `M7-TASK-002` | 任务执行期间：展示进度指示、禁用相关控件、置编辑器只读；结束后恢复 | P0 | `lex.aardio:38-74`、`dlg/dict/phrase.aardio:139-150` | 原项目用 5 帧图标动画 |
| `M7-TASK-003` | **【新增】统一任务管理器：任务 ID、状态查询、取消** | P1 | 原项目仅下载可取消，其余不可中断 | 空码造词耗时数分钟且无法取消 |
| `M7-TASK-004` | **【新增】并发控制：互斥任务不可同时运行** | P1 | 原项目靠禁用按钮，防护不严 | 例如安装码表期间禁止再次安装 |
| `M7-TASK-005` | **【新增】统一进度上报格式** | P1 | 原项目各处文案自定义 | 事件 `task://progress` |

### 需要可取消的长任务

| 任务 | 来源 | 典型耗时 |
|---|---|---|
| 空码智能造词 | [`M1-COIN-005`](./M1-lex-table.md) | 数分钟 |
| 安装到系统码表 | [`M1-INSTALL-001`](./M1-lex-table.md) | 数分钟 |
| 加载大码表到编辑器 | [`M1-EDIT-001`](./M1-lex-table.md) | 数十秒至数分钟 |
| 词频重算 | [`M1-WEIGHT-001`](./M1-lex-table.md) | 数十秒 |
| 格式转换 / 精简 | [`M1-XFORM-*`](./M1-lex-table.md) | 数秒至数十秒 |
| 整包下载解压 | [`M6-DOWN-010`](./M6-resource-sync.md) | 数分钟 |

> **注意**：安装到系统码表**不应**在停机窗口内被取消（会留下不一致状态）。应在进入停机窗口**前**提供取消点，之后置为不可取消并明确告知用户。

---

## 8. 已废弃模块（P3）

以下代码存在于原仓库但**不进新项目**，此处仅作记录以保证覆盖完整性。

| ID | 项 | 文件 | 行数 | 判据 |
|---|---|---|---:|---|
| `M7-DEPR-001` | 超级热键配置模板 | `lib/app/hotkey.aardio` | 303 | 全文为注释；未被任何文件 `import`；`README.md:27` 明示「新版不再提供超级热键功能」，推荐改用 ImTip |
| `M7-DEPR-002` | 候选窗口句柄探测 | `lib/winex/msCandidate.aardio` | 29 | 仅被 `M7-DEPR-001` 的注释代码与 `M7-DEPR-003` 引用，实际未启用 |
| `M7-DEPR-003` | `candidateWindow` 别名 | `lib/wubi/candidateWindow.aardio` | 7 | 源码标注 `//Deprecated` |
| `M7-DEPR-004` | `ui.chineseNumber` 别名 | `lib/ui/chineseNumber.aardio` | 4 | 源码标注 `@Deprecated` |
| `M7-DEPR-005` | `wubi.chineseNumber` 别名 | `lib/wubi/chineseNumber.aardio` | 4 | 源码标注 `@Deprecated`；与 `M7-DEPR-004` 内容重复 |

### 超级热键的历史用途

`hotkey.aardio` 的注释模板展示了原本支持的能力（`hotkey.aardio:4-303`）：

- 用 aardio 代码定义任意按键组合的回调
- 大写金额 / 日期时间大写面板（`Ctrl + $`）
- 改键、叠字键、引号括号自动配对
- CapsLock 切英文、左右 Shift 分别切中英
- 按 `0` 选第 10 个重码、取消次选键
- 音量控制、启动程序
- 切换拼音混输（`Ctrl + ,`）

**替代品**：[ImTip](https://imtip.aardio.com)（同作者，约 800 KB）。

> 若新项目未来要恢复此能力，**不应**复刻「用脚本语言写配置」的方案（安全性差、门槛高）。建议做成结构化的可视化规则编辑器。

---

## 9. 数据依赖

本模块是**被依赖方**，为所有其他模块提供：

| 能力 | 消费方 |
|---|---|
| 配置持久化 | M1、M2、M3、M4、M6 |
| 事件总线 | 全部 |
| 后台任务与进度 | M1、M2、M3、M6 |
| 全局热键注册 | M3 |
| 页面导航 | M3（跳字根图）、M5、托盘菜单 |
| 系统版本探测 | M2（EUDP 版本门槛）、M5（关于页） |

---

## 10. 对外接口草案

### Tauri Commands

| Command | 用途 |
|---|---|
| `app_info` | 版本、构建信息、系统版本、是否管理员 |
| `config_get` / `config_set` | 配置读写（按键路径） |
| `config_export` / `config_import` | 配置导出导入 |
| `window_minimize_to_tray` / `window_restore` | 窗口控制 |
| `navigate` | 跳转到指定页面（供托盘菜单与跨模块跳转使用） |
| `hotkey_register` / `hotkey_unregister` | 全局热键 |
| `task_list` / `task_cancel` | 后台任务管理 |
| `recovery_check` / `recovery_run` | 崩溃恢复 |

### Events

见 [§6 事件清单](#事件清单) 与新增事件建议。

---

## 11. 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 整进程管理员权限扩大攻击面 | **高** | 一期接受并在非功能需求中记录；演进到「提权辅助进程」方案 |
| 崩溃时 TSF/服务/ACL 停在中间态 | **高** | 启动时恢复检查，见 `M7-INST-006` |
| 长任务不可取消，用户只能杀进程 | 中 | 统一任务管理器，见 `M7-TASK-003` |
| 托盘菜单动态状态在 Tauri 下的重建成本 | 中 | 弹出时刷新，或用状态订阅增量更新 |
| 配置文件损坏导致启动失败 | 中 | 校验 + 回退默认 + 备份，见 `M7-CONF-004` |
| 复用 aardio 的 `%APPDATA%` 路径造成冲突 | 中 | 迁移到自有路径，见 `M7-CONF-006` |
| WebView2 运行时缺失（Win10 早期版本） | 中 | 安装包内置 WebView2 Bootstrapper，或用固定版本分发 |
| `emptyWorkingSet` 在 WebView2 架构下无效甚至有害 | 低 | 评估后决定是否保留 `M7-WIN-009` |

---

## 12. 源码索引

完整反向索引见 [`03-source-index.md`](../03-source-index.md)。

| 域 | 位置 | 说明 |
|---|---|---|
| `INST` | `main.aardio:1`（RUNAS）、`19-25`（**单实例 atom**）、`303-313`（`/tray` 参数） | |
| `WIN` | `main.aardio:6-16`（窗体）、`37-68`（**一级 Tab**）、`71-93`（最小化/关闭/销毁）；`dlg/dict/dict.aardio:14-45`（词库二级 Tab）；`dlg/help/help.aardio:14-43`（帮助二级 Tab） | ImTip 入口在 `dict.aardio:47-78` |
| `TRAY` | `main.aardio:71-80`（创建）、`99-269`（**托盘菜单 24 项**）、`260-267`（左键还原） | 菜单结构见 [§3](#托盘菜单结构) |
| `HOTKEY` | `main.aardio:271-294`（**注册/注销/前台判断**）、`315-319`（`Ctrl+W`） | |
| `KEYMAP` | `dlg/spelling.aardio:8, 545-565`（**唯一的热键录制 UI**）；`lib/config.aardio:13-15`（默认值） | 原项目仅支持自定义 1 个热键 |
| `CONF` | `lib/config.aardio:1-15`（**全部配置定义**）；`lib/app/lexContents.aardio:51, 67`（分组即时保存） | 配置项仅 5 个 |
| `BUS` | 分散：`main.aardio:271, 296`、`dlg/dict/lex.aardio:85-98, 126`、`dlg/spelling.aardio:440-444, 583`、`dlg/setting.aardio:70, 275-285`、`lib/tsfUtil.aardio:123, 177, 187` | 见 [§6 事件清单](#事件清单) |
| `TASK` | 分散：全项目的 `thread.invoke` / `win.invoke` 调用；`dlg/dict/lex.aardio:38-74` 的 `busy()` | 无统一任务管理 |
| `DEPR` | `lib/app/hotkey.aardio`、`lib/winex/msCandidate.aardio`、`lib/wubi/candidateWindow.aardio`、`lib/ui/chineseNumber.aardio`、`lib/wubi/chineseNumber.aardio` | 全部 P3 |
| 应用元数据 | `default.aproj:2` | 名称 / 图标 / 版本 `12.1.0.0` / 输出 `WubiLex.exe` |
| 皮肤令牌 | `lib/style.aardio:1-134` | 新项目改用 Tailwind，仅作配色参考 |

### ⚠️ 不要照抄的位置

| 位置 | 问题 | 对应需求 |
|---|---|---|
| `main.aardio:271-294` | `reghotkey` 返回值未检查，热键被占用时静默失败 | `M7-HOTKEY-004`、`M7-KEYMAP-006` |
| `main.aardio:76-77` | `process.emptyWorkingSet` 在 WebView2 架构下效果存疑 | `M7-WIN-009` |
| `lib/config.aardio:3` | 配置落盘在 aardio 共享路径 `%APPDATA%/aardio/std/` | `M7-CONF-006` |
| 全项目 `thread.invoke` | 无任务 ID、无取消、无统一进度 | `M7-TASK-003/005` |

---

## 需求统计

| 域 | 条目数 | P0 | P1 | P2 | P3 |
|---|---:|---:|---:|---:|---:|
| `INST` | 6 | 4 | 2 | 0 | 0 |
| `WIN` | 12 | 5 | 5 | 2 | 0 |
| `TRAY` | 8 | 6 | 0 | 2 | 0 |
| `HOTKEY` | 4 | 2 | 2 | 0 | 0 |
| `KEYMAP` | 13 | 0 | 11 | 2 | 0 |
| `CONF` | 7 | 4 | 2 | 1 | 0 |
| `BUS` | 3 | 2 | 1 | 0 | 0 |
| `TASK` | 5 | 2 | 3 | 0 | 0 |
| `DEPR` | 5 | 0 | 0 | 0 | 5 |
| **合计** | **63** | **25** | **26** | **7** | **5** |
