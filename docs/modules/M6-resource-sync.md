# M6 — 资源分发与网络同步

> **模块职责**：在线码表目录、各类资源的下载与解压、本地缓存管理、软件自更新。
>
> **不做**：不解析下载内容的语义（交给 [M1](./M1-lex-table.md) / [M2](./M2-phrase.md) / [M3](./M3-reverse-lookup.md)）。

## 来源文件

| 文件 | 行数 | 角色 |
|---|---:|---|
| `lib/app/lexNetContents.aardio` | 227 | 内置在线码表目录 + `index.json` 更新 |
| `lib/app/lexContents.aardio` | 365 | 码表下载与解压分支（与 M1 共享） |
| `lib/wubi/spellingTable.aardio` | 64 | 拆字数据表下载（与 M3 共享） |
| `lib/wubi/fonts.aardio` | 45 | 字根字体下载（与 M3 共享） |
| `lib/wubi/weightData.aardio` | 22 | 词频文件下载（与 M1 共享） |
| `dlg/spelling.aardio` | 590 | 笔顺 GIF 与字根图下载（与 M3 共享） |
| `dlg/help/etymon.aardio` | 257 | 字根整图下载（与 M5 共享） |
| `dlg/help/wubi.aardio` | 95 | 软件更新 UI（与 M5 共享） |
| `main.aardio` | 321 | 启动时更新检查（与 M7 共享） |
| `sepllingData/build.aardio` | 145 | **开发者工具**：拆字数据编辑与 LZMA 打包 |

---

## 1. ⚠️ 核心风险：单点、明文、无校验

原项目**全部**在线资源指向单一主机 `wubi.aardio.com`，且：

| 问题 | 现状 | 影响 |
|---|---|---|
| **明文 HTTP** | 所有 URL 均为 `http://` | 可被中间人篡改。码表/字体/**软件更新包**被替换即等同任意代码执行 |
| **无完整性校验** | 仅对码表检查 `imscwubi` 魔数（8 字节） | 魔数极易伪造，不构成防护 |
| **单点依赖** | 无镜像、无 CDN 回退 | 服务器不可用则拆字、笔顺、字根图、码表下载、词频全部失效 |
| **无签名** | 软件自更新无任何签名验证 | 最高危：可被投递恶意可执行文件 |

**新项目的硬性要求**（详见 [`20-nonfunctional.md`](../20-nonfunctional.md)）：

1. 全部资源走 **HTTPS**
2. 每个资源附 **SHA-256** 校验和，下载后强制校验
3. 软件更新包必须**代码签名**并验证
4. 支持配置**自定义镜像源**
5. 核心资源（86/98 拆字表、字根图、字体）**内置于安装包**，网络仅用于扩展

---

## 2. 在线码表目录（`CATALOG`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M6-CATALOG-001` | 内置默认码表目录：8 个分类、40+ 条目 | P1 | `lexNetContents.aardio:36-221` | 见 [附录 A](#附录-a内置码表目录) |
| `M6-CATALOG-002` | 从服务器拉取 `index.json` 更新目录 | P1 | `lexNetContents.aardio:22-34` | |
| `M6-CATALOG-003` | 目录缓存到本地文件，下次启动直接使用 | P1 | `lexNetContents.aardio:4-20` | `<AppData>/lex-default-v2.table` |
| `M6-CATALOG-004` | 缓存解析失败时静默回退到内置目录 | P1 | `lexNetContents.aardio:10-19` | 原项目用 `try` 包裹 |
| `M6-CATALOG-005` | 目录以「分类 → 条目」二级菜单呈现 | P1 | `dlg/dict/lex.aardio:1429-1447` | |
| `M6-CATALOG-006` | 更新时机：应用启动 3 秒后、以及每次点击「添加码表」时 | P1 | `dlg/dict/lex.aardio:1400-1402, 1450-1453` | 异步，不阻塞 UI |
| `M6-CATALOG-007` | **【新增】目录 schema 版本化，字段校验失败拒绝加载** | P1 | 原项目直接 `eval` 反序列化 | 见下 |

### ⚠️ `M6-CATALOG-007` 原项目用 `eval` 反序列化缓存

`lexNetContents.aardio:12`：

```aardio
result = eval(..string.load(dataPath))
```

缓存文件内容被当作 **aardio 代码执行**。虽然文件来自本地 `%APPDATA%`（需先有写入权限才能利用），但配合 HTTP 明文传输的 `index.json`（`lexNetContents.aardio:28-31` 直接把响应序列化后落盘），构成一条**远程代码执行**链路。

**新实现要求**：用 `serde_json` 严格反序列化到强类型结构，未知字段拒绝或忽略，绝不执行数据。

---

## 3. 下载（`DOWN`）

| ID | 需求 | P | 来源 | 资源 |
|---|---|---|---|---|
| `M6-DOWN-001` | 提供带进度条的下载 UI，展示当前任务名称 | P1 | `lexContents.aardio:122` | 通用 |
| `M6-DOWN-002` | 下载可取消，取消后清理临时文件 | P1 | `lexContents.aardio:135-143` | 通用 |
| `M6-DOWN-003` | 下载码表（`.lex` / `.lex.lzma`） | P1 | `lexContents.aardio:118-175` | `download/lex/*.lex.lzma` |
| `M6-DOWN-004` | 下载拆字数据表（按方案） | P1 | `spellingTable.aardio:8-17` | `download/spelling/data-table/{版本}.lzma` |
| `M6-DOWN-005` | 下载单字笔顺 GIF（文件名为字符的十六进制编码） | P1 | `spelling.aardio:314-336` | `download/spelling/{版本}/{hex}.gif` |
| `M6-DOWN-006` | 下载字根整图 | P1 | `etymon.aardio:49-54` | `download/etymon/{版本}.jpg` |
| `M6-DOWN-007` | 下载单键字根图 | P1 | `spelling.aardio:59-68` | `download/etymon/{版本}/{键}.gif` |
| `M6-DOWN-008` | 下载字根字体（2 个变体） | P1 | `fonts.aardio:22-36` | `download/spelling/fonts/wubi-lex-etymon-v5.lzma`、`-092.lzma` |
| `M6-DOWN-009` | 下载微软词频文件 | P1 | `weightData.aardio:6-16` | `download/word-weight.lzma` |
| `M6-DOWN-010` | 下载拆分图解字库整包（tar + lzma） | P1 | `spelling.aardio:530-543` | `download/spelling.tar.lzma` |
| `M6-DOWN-011` | **【新增】全部资源走 HTTPS，并校验 SHA-256** | P0 | 原项目**无** | |
| `M6-DOWN-012` | **【新增】失败自动重试（指数退避），支持断点续传** | P1 | 原项目**无** | |
| `M6-DOWN-013` | **【新增】支持配置自定义镜像源** | P1 | 原项目硬编码域名 | |
| `M6-DOWN-014` | 所有下载在后台线程执行，不阻塞 UI | P0 | 原项目用 `thread.invoke` | |
| `M6-DOWN-015` | **【新增】下载失败时给出可读错误（网络 / 404 / 校验失败 / 磁盘），而非静默失败** | P1 | 原项目大量静默失败 | |

### ⚠️ 原项目的静默失败

多处下载在失败时**什么都不做**：

| 位置 | 行为 |
|---|---|
| `spelling.aardio:64-68` | 单键字根图下载失败 → 图片区保持空白，无提示 |
| `spellingTable.aardio:8-17` | 拆字表下载失败 → `find()` 返回 `null`，拆字区空白 |
| `lexNetContents.aardio:26-31` | 目录更新失败 → 静默用旧缓存 |
| `etymon.aardio:52-54` | 字根图下载失败 → `background` 设为不存在的路径 |

对「装饰性资源」（单键字根图）可接受，对「功能性资源」（拆字表、码表）必须上报。

---

## 4. 归档与解压（`ARCHIVE`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M6-ARCHIVE-001` | LZMA 解压，带进度回调 | P1 | `lexContents.aardio:126-152` | **LZMA alone** 格式，非 xz |
| `M6-ARCHIVE-002` | TAR 解包（用于 `spelling.tar.lzma` 整包） | P1 | `spelling.aardio:538-539` | 先 LZMA 解压再 TAR 解包 |
| `M6-ARCHIVE-003` | 解压后校验目标文件魔数（码表校验 `imscwubi`） | P1 | `lexContents.aardio:156-169` | |
| `M6-ARCHIVE-004` | 解压过程可取消 | P1 | `lexContents.aardio:134-144` | |
| `M6-ARCHIVE-005` | 解压进度按输入字节数估算并映射到 0–1000 刻度 | P2 | `lexContents.aardio:130-140` | |
| `M6-ARCHIVE-006` | **【新增】TAR 解包时防御路径穿越（`../`、绝对路径）** | P0 | 原项目无防护 | Zip Slip 类漏洞 |

### LZMA 格式说明

上游产物由 aardio 的 `sevenZip.lzma` 生成，是 **LZMA1 alone**（`.lzma`）格式 —— 13 字节头（1 字节属性 + 4 字节字典大小 + 8 字节未压缩长度）后接原始 LZMA 流。

**Rust 侧**：用 `lzma-rs` 的 `lzma_decompress`（**不是** `xz_decompress`）。若新项目自建资源分发，建议改用 **zstd**（解压快 3–5 倍，`zstd` crate 成熟）。

---

## 5. 缓存管理（`CACHE`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M6-CACHE-001` | 按资源类型分目录组织本地缓存 | P0 | 各模块散落 | 见下方目录结构 |
| `M6-CACHE-002` | 请求资源时先查本地缓存，命中则直接返回 | P0 | 各模块 | |
| `M6-CACHE-003` | **【新增】提供缓存清理入口，按类型选择性清除** | P1 | 原项目**无** | |
| `M6-CACHE-004` | **【新增】统计各类缓存占用空间并在设置页展示** | P2 | 原项目**无** | 笔顺 GIF 整包解压后可达数十 MB |
| `M6-CACHE-005` | **【新增】提供离线资源包，一次性导入全部资源** | P1 | 原项目**无** | 内网/断网环境的刚需 |
| `M6-CACHE-006` | **【新增】缓存目录迁移到应用自有路径，不再复用 aardio 路径** | P0 | 原项目用 `%APPDATA%/aardio/std/wubi/` | |

### 原项目缓存目录结构

```
%APPDATA%\aardio\std\wubi\
├── lex\                              用户码表 (*.lex)
├── etymon\v3\
│   ├── {版本}.jpg                     字根整图
│   └── {版本}\{键}.gif                单键字根图
├── spelling\{版本}\{字}.gif           笔顺动画
├── spelling-data-table\v2\{版本}.txt  拆字数据表
├── fonts\
│   ├── wubi-lex-etymon-v5.otf
│   └── wubi-lex-etymon-092.otf
├── download\                          下载暂存
├── word-weight2.txt                   微软词频
├── lex-default-v2.table               在线码表目录缓存
└── app\update\                        软件更新暂存

%APPDATA%\aardio\std\wubi-lex-tool\    应用配置
```

**新项目建议**：

```
%LOCALAPPDATA%\<AppName>\
├── cache\
│   ├── lex\
│   ├── etymon\
│   ├── stroke\
│   ├── split-table\
│   └── fonts\
├── data\
│   ├── lex\                           用户码表（非缓存，不应被清理）
│   └── backup\                        系统码表/短语备份
└── config\
```

> **关键区分**：用户添加的码表是**数据**不是缓存，必须与可清理的缓存分离存放。原项目把二者都放在 `wubi/lex/`，清理缓存会误删用户数据。

---

## 6. 软件自更新（`UPDATE`）

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M6-UPDATE-001` | 应用启动时检查更新；若已就绪则直接进入更新流程并退出主流程 | P1 | `main.aardio:27-34` | |
| `M6-UPDATE-002` | 帮助页提供手动检查更新入口 | P1 | `wubi.aardio:82-89` | |
| `M6-UPDATE-003` | 更新状态回调（`ready` / `complete` / `latest` / `failed`），驱动 UI 展示 | P1 | `wubi.aardio:59-80` | 见 [M5](./M5-etymon-help.md#更新状态) |
| `M6-UPDATE-004` | 更新就绪时重启应用完成安装 | P1 | `wubi.aardio:86-88` | |
| `M6-UPDATE-005` | **【新增】更新包必须验证代码签名，验证失败拒绝安装** | P0 | 原项目**无任何验证** | **最高危** |
| `M6-UPDATE-006` | **【新增】更新走 HTTPS + 校验和** | P0 | 原项目为 `http://wubi.aardio.com/update/` | |
| `M6-UPDATE-007` | **【新增】提供「自动检查更新」开关** | P1 | 原项目强制启动检查，无法关闭 | |

> **Tauri 侧建议**：直接使用官方 [`tauri-plugin-updater`](https://v2.tauri.app/plugin/updater/)，它内置了签名验证（minisign）与 HTTPS 要求，无需自研。

---

## 7. 开发者工具：拆字数据构建

`sepllingData/build.aardio` 是**不进产品**的内部工具，但新项目需要等价的数据构建流程。

| ID | 需求 | P | 来源 | 备注 |
|---|---|---|---|---|
| `M6-BUILD-001` | 提供拆字数据表的编辑与校验工具 | P2 | `build.aardio:1-95` | 加载 / 编辑 / 查找 / 保存 |
| `M6-BUILD-002` | 批量把 7 个方案的拆字数据打包为压缩产物 | P2 | `build.aardio:96-111` | |
| `M6-BUILD-003` | 编辑时加载字根字体以正确显示 PUA 字符 | P2 | `build.aardio:34-53` | |
| `M6-BUILD-004` | 提供字根字符参考面板 | P2 | `build.aardio:116-143` | |
| `M6-BUILD-005` | **【新增】构建时生成资源清单（含 SHA-256）供客户端校验** | P1 | 原项目**无** | 配合 `M6-DOWN-011` |

> 目录名 `sepllingData` 为拼写错误（应为 `spellingData`），新项目应更正。
> 新实现建议做成 CLI（`cargo xtask build-resources`）而非 GUI，纳入 CI 流程。

---

## 附录 A：内置码表目录

来源 `lexNetContents.aardio:36-221`。共 8 分类 40 条。

### 五笔86

| 名称 | 文件 |
|---|---|
| 微软五笔86( 单字 ) | `ChsWubi86.min.lex.lzma` |
| 微软五笔86( 完整 ) | `ChsWubi86.lex.lzma` |
| QQ五笔86( 单字 ) | `qq86.min.lex.lzma` |
| QQ五笔86( 完整 ) | `qq86.lex.lzma` |
| 极点五笔86( 单字 ) | `freeime86.min.lex.lzma` |
| 极点五笔86( 完整 ) | `freeime86.lex.lzma` |

### 五笔98

| 名称 | 文件 |
|---|---|
| 五笔98( 单字 ) | `ChsWubi98.min.lex.lzma` |
| 五笔98( 完整 ) | `ChsWubi98.lex.lzma` |
| QQ五笔98( 完整 ) | `qq98.lex.lzma` |
| 昱琼词库98( 完整 ) | `yuqiong98.lex.lzma` |
| 海峰词库98( 完整 ) | `sun98.lex.lzma` |

### 五笔新世纪

| 名称 | 文件 |
|---|---|
| 新世纪GBK( 完整 ) | `06.lex.lzma` |
| 新世纪GBK( 单字 ) | `06.min.lex.lzma` |
| 新世纪GB2312( 完整 ) | `06.gb.lex.lzma` |
| 新世纪GB2312( 单字 ) | `06.gb.min.lex.lzma` |

### 五笔092

| 名称 | 文件 |
|---|---|
| 五笔092( 常规 ) | `092/092wb.lex.lzma` |
| 五笔092( 超集 ) | `092/092wbp.lex.lzma` |
| 五笔092K | `092/092Kwb.lex.lzma` |

### 五笔09

| 名称 | 文件 |
|---|---|
| 点儿091( 完整 ) | `091.lex.lzma` |

### 郑码

| 名称 | 文件 |
|---|---|
| 郑码( 构词表 ) | `zhengma/zhengma.chars.lex.lzma` |
| 郑码( 6.6 ) | `zhengma/zhengma6.6.lex.lzma` |
| 郑码( 5.0 ) | `zhengma/zhengma5.0.lex.lzma` |
| 郑码( 小指 ) | `zhengma/zhengma.xiaozhi.lex.lzma` |
| 郑码( 小泉 ) | `zhengma/zhengma.xiaoquan.lex.lzma` |
| 郑码( 过客一剑 ) | `zhengma/zhengma.guoke.lex.lzma` |
| 郑码( 云在天外 ) | `zhengma/zhengma.yztw.lex.lzma` |
| 郑码( 超强 ) | `zhengma/zhengma.chaoqiang.lex.lzma` |
| 郑码( 超集 ) | `zhengma/zhengma.ex.lex.lzma` |
| 郑码( 繁体 ) | `zhengma/zhengma.cht.lex.lzma` |
| 郑码( 繁体超集 ) | `zhengma/zhengma.cht.ex.lex.lzma` |

> 「郑码( 构词表 )」即 [M1 造词](./M1-lex-table.md#7-造词coin) 所需的 `formation` 变体。

### 小鹤音形

| 名称 | 文件 |
|---|---|
| 小鹤音形( 小词库 ) | `xhyx.lex.lzma` |
| 小鹤音形( 大词库 ) | `xhyx.ext.lex.lzma` |
| 小鹤音形( 单字 ) | `xhyx.min.lex.lzma` |

### 表形码

| 名称 | 文件 |
|---|---|
| 表形码威版 | `bxm.wei.lex.lzma` |
| 表形码 Win95 版 | `bxm.win95.lex.lzma` |
| 表形码老夫子版 | `bxm.lfz.lex.lzma` |

**基址**：`http://wubi.aardio.com/download/lex/`
**默认加载规则**：首次运行时，每个分类取**前 2 条**加入用户列表（`lexContents.aardio:195-206`）。

### 目录 JSON schema

```jsonc
[
  {
    "name": "五笔86",           // 分类名
    "items": [
      { "name": "微软五笔86( 单字 )", "url": "http://.../ChsWubi86.min.lex.lzma" }
    ]
  }
]
```

**新版本应扩展为**：

```jsonc
{
  "schemaVersion": 2,
  "categories": [
    {
      "name": "五笔86",
      "items": [
        {
          "name": "微软五笔86( 单字 )",
          "url": "https://.../ChsWubi86.min.lex.zst",
          "sha256": "…",
          "size": 123456,
          "scheme": "86",          // 方案代号，便于按当前系统码表过滤
          "variant": "min",        // min | full
          "license": "…",
          "source": "…"            // 词库来源署名
        }
      ]
    }
  ]
}
```

---

## 8. 数据依赖

| 依赖 | 来源 | 说明 |
|---|---|---|
| 配置持久化 | [M7](./M7-app-shell.md) | 镜像源设置、自动更新开关 |
| 后台任务调度 | [M7](./M7-app-shell.md) | 全部下载任务 |

**反向被依赖**：M1（码表、词频、目录）、M3（拆字表、GIF、字体）、M5（字根图、软件更新）。

---

## 9. 对外接口草案

### Tauri Commands

| Command | 用途 |
|---|---|
| `resource_catalog` | 返回在线码表目录（本地缓存优先） |
| `resource_catalog_refresh` | 强制刷新目录 |
| `resource_fetch` | 通用资源获取：`{ kind, version?, key? }` → 本地路径；缺失则下载 |
| `resource_download_pack` | 整包下载（笔顺图库 / 离线资源包） |
| `resource_cache_stat` | 各类缓存占用统计 |
| `resource_cache_clear` | 按类型清理缓存 |
| `resource_import_offline` | 导入离线资源包 |
| `resource_mirror_get` / `resource_mirror_set` | 镜像源配置 |
| `update_check` / `update_apply` | 软件更新 |

### Events

| Event | 载荷 |
|---|---|
| `resource://progress` | `{ task_id, kind, downloaded, total, phase }` |
| `resource://ready` | `{ kind, version, path }` |
| `resource://error` | `{ kind, reason }` |

---

## 10. 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 软件更新无签名验证 | **极高** | 用 `tauri-plugin-updater`（内置 minisign），见 `M6-UPDATE-005` |
| 全链路明文 HTTP + 无校验 | **极高** | HTTPS + SHA-256，见 `M6-DOWN-011` |
| 目录缓存用 `eval` 反序列化 | **高** | 严格 JSON 反序列化，见 `M6-CATALOG-007` |
| TAR 解包路径穿越 | **高** | 路径规范化 + 拒绝 `..` 与绝对路径，见 `M6-ARCHIVE-006` |
| 上游服务器单点，不可用则多模块失效 | **高** | 核心资源内置 + 镜像源 + 离线包 |
| 缓存与用户数据混放，清理会误删 | 中 | 目录分离，见 `M6-CACHE-006` |
| 大量静默失败让用户无从排查 | 中 | 功能性资源必须上报错误，见 `M6-DOWN-015` |
| LZMA alone 格式易与 xz 混淆 | 低 | 明确用 `lzma_decompress`；新资源改用 zstd |

---

## 11. 源码索引

本模块的实现在原项目中是**分散**的 —— 每个消费方各自写下载逻辑，没有统一的资源层。新项目要把它们收敛到 `wubilex-resource`。

完整反向索引见 [`03-source-index.md`](../03-source-index.md)。

| 域 | 分散在 | 说明 |
|---|---|---|
| `CATALOG` | `lib/app/lexNetContents.aardio:4-221` | 唯一集中的部分。`36-221` 是**内置目录 8 分类 40 条** |
| `DOWN`（码表） | `lib/app/lexContents.aardio:113-175` | 带进度条与解压 |
| `DOWN`（拆字表） | `lib/wubi/spellingTable.aardio:8-17` | 构造即下载 |
| `DOWN`（笔顺 GIF） | `dlg/spelling.aardio:314-336` | 文件名为字符十六进制 |
| `DOWN`（字根整图） | `dlg/help/etymon.aardio:49-54` | |
| `DOWN`（单键字根图） | `dlg/spelling.aardio:59-68` | |
| `DOWN`（字根字体） | `lib/wubi/fonts.aardio:22-36` | |
| `DOWN`（词频文件） | `lib/wubi/weightData.aardio:6-16` | |
| `DOWN`（整包图库） | `dlg/spelling.aardio:530-543` | tar + lzma |
| `ARCHIVE` | `lib/app/lexContents.aardio:126-152` | LZMA 解压 + 进度映射 |
| `CACHE` | 各文件的 `io.appData(...)` 调用 | 无统一管理，见 [§5 目录结构](#原项目缓存目录结构) |
| `UPDATE` | `main.aardio:27-34`（启动检查）、`dlg/help/wubi.aardio:55-89`（UI 与状态） | |
| `BUILD` | `sepllingData/build.aardio:1-145` | 开发者工具，不进产品 |

### 上游 URL 汇总

基址 `http://wubi.aardio.com`（新项目须改 HTTPS 并支持镜像）：

| 资源 | 路径 | 出处 |
|---|---|---|
| 码表目录 | `/download/lex/index.json` | `lexNetContents.aardio:28` |
| 码表文件 | `/download/lex/*.lex.lzma` | `lexNetContents.aardio:36-221` |
| 拆字数据表 | `/download/spelling/data-table/{版本}.lzma` | `spellingTable.aardio:13` |
| 单字笔顺 GIF | `/download/spelling/{版本}/{hex}.gif` | `spelling.aardio:317` |
| 整包图库 | `/download/spelling.tar.lzma` | `spelling.aardio:538` |
| 字根整图 | `/download/etymon/{版本}.jpg` | `etymon.aardio:50` |
| 单键字根图 | `/download/etymon/{版本}/{键}.gif` | `spelling.aardio:62` |
| 字根字体 | `/download/spelling/fonts/wubi-lex-etymon-v5.lzma`、`-092.lzma` | `fonts.aardio:25, 27` |
| 微软词频 | `/download/word-weight.lzma` | `weightData.aardio:9` |
| 软件更新 | `/update/` | `main.aardio:30` |

### ⚠️ 不要照抄的位置

| 位置 | 问题 | 对应需求 |
|---|---|---|
| `lib/app/lexNetContents.aardio:12` | `eval()` 反序列化缓存 —— **RCE 链路** | `M6-CATALOG-007` |
| 全部 URL | 明文 HTTP，无校验和 | `M6-DOWN-011` |
| `main.aardio:27-34` | 更新无签名验证 | `M6-UPDATE-005` |
| `dlg/spelling.aardio:538-539` | TAR 解包无路径穿越防护 | `M6-ARCHIVE-006` |
| `spellingTable.aardio:8-17`、`etymon.aardio:52-54`、`spelling.aardio:64-68` | 下载失败静默 | `M6-DOWN-015` |

---

## 需求统计

| 域 | 条目数 | P0 | P1 | P2 |
|---|---:|---:|---:|---:|
| `CATALOG` | 7 | 0 | 7 | 0 |
| `DOWN` | 15 | 2 | 13 | 0 |
| `ARCHIVE` | 6 | 1 | 4 | 1 |
| `CACHE` | 6 | 3 | 2 | 1 |
| `UPDATE` | 7 | 2 | 5 | 0 |
| `BUILD` | 5 | 0 | 1 | 4 |
| **合计** | **46** | **8** | **32** | **6** |
