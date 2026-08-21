# src-tauri — wubilex-app

> Tauri 应用层。把下层能力暴露为 command / event，管理全局状态与任务生命周期。

> ## ⚠️ 一条压倒性约束
>
> **禁止在本 crate 写领域逻辑。**
>
> command 函数应当是薄适配层：参数反序列化 → 调用下层 → 结果序列化。
>
> 原项目最大的结构缺陷就是把造词、格式转换、精简、词频优化全部写进 `dlg/dict/lex.aardio` 的菜单回调（1,455 行），导致既不能测也不能复用。这里是那个错误最容易复发的地方。

## 目录

| 目录 | 内容 | 需求 |
|---|---|---|
| `src/commands/` | 按 command 前缀分目录，见下表 | 各模块 |
| `src/state/` | `AppState`（含 `lex_sessions` 句柄表） | D1 |
| `src/events/` | 事件总线 `<域>://<事件名>` | `M7-BUS-*` |
| `src/task/` | 任务注册表 + 取消 + 进度桥接 | `M7-TASK-*` · `R29` |
| `src/keymap/` | 动作注册表 + 绑定解析 | `M7-KEYMAP-*` |
| `src/config/` | TOML 配置 · schema 版本 · 损坏回退 · 写前备份 | `M7-CONF-*` · `R31` |
| `src/error/` | `AppError` 统一错误模型 | `02` §3.3 |
| `src/features/` | 模块特性开关（驱动占位状态） | D16 · `UX-INTERACT-013` · `R44` |
| `src/recovery/` | 崩溃后自恢复 | **`R1`** · `M7-INST-006` |
| `src/bindings/` | `tauri-specta` 导出入口 | D11 |

### command 前缀映射

| 目录 | 前缀 | 模块 |
|---|---|---|
| `commands/lex/` | `lex_` | M1 码表 |
| `commands/phrase/` | `phrase_` | M2 短语 |
| `commands/spelling/` | `spelling_` | M3 反查 |
| `commands/ime/` | `ime_` `dpy_` `wubi_settings_` `tsf_` | M4 输入法 |
| `commands/etymon/` | `etymon_` `help_` `about_` | M5 字根与帮助 |
| `commands/resource/` | `resource_` `update_` | M6 资源 |
| `commands/app/` | `app_` `config_` `window_` `hotkey_` `keymap_` `task_` | M7 外壳 |
| `commands/learn/` | `learn_` | M8 自学习 |

## 允许依赖

全部下层 crate（`wubilex-learn` 自 S8 起）。

## 禁止

把领域逻辑写在 command 里。

## 本层独有的四件事

下层 crate 刻意不做、必须由这里承担的：

### 1. 异步化与并行

`wubilex-codec` / `wubilex-core` 是**纯同步**的（它们禁止依赖 `tokio`）。长任务由本层用 `spawn_blocking` 挪出异步运行时，批量操作由本层用 `rayon` 并行分片。

### 2. 取消与进度的桥接

下层收 `&AtomicBool`（取消）与 `&mut dyn FnMut(P)`（进度）。本层负责在 `CancellationToken` ↔ `AtomicBool`、下层进度枚举 ↔ Tauri 事件之间转换。

### 3. 端口注入

- `wubilex-core` 的 `ResourceProvider` —— core 声明「我要一份拆字表」，本层决定从内置资源 / 缓存 / 下载满足
- `wubilex-core` 的 `PhraseSink` —— `wubilex-learn` 的入库出口，物理写入由本层编排

### 4. 错误的最后一道关

```rust
struct AppError {
    kind: ErrorKind,        // Io | Parse | Network | Permission | System | Validation | Cancelled
    module: &'static str,   // "M1" | "M4" | ...
    message: String,        // 面向用户的可读描述（中文）
    detail: Option<String>, // 技术细节：系统错误码、行号、路径
    recoverable: bool,
}
```

每个 command 返回 `Result<T, AppError>`。**任何 `unwrap()` / `expect()` 在生产路径上都是缺陷。**

原项目的模式是 `return null, "错误信息"`，且大量调用点直接丢弃错误信息，最终用户只能看到「安装失败请重试一次即可」。不要复现这个。

## `lex_sessions` 是性能的关键

原项目每次操作都做「文本 → 解析 → 变换 → 序列化 → 文本」全量往返，是它卡顿的根因。

新设计（D1）：`lex_open` 返回 session 句柄，码表模型（含倒排索引）常驻 `AppState`。所有变换在模型上**原地**进行，前端通过 `lex_query_page` 按需拉取视口数据。

代价是内存占用（数十万条目约 50–150 MB），可接受。

## 所属阶段

**S1 — 外壳与 UI 骨架**（应用可跑起来），此后每个阶段都会往 `commands/` 里加内容。

## Tauri v2 必需目录

- `capabilities/` — 权限声明（v2 权限模型强制）
- `icons/` — 应用图标
- `resources/` — 随包内置资源：`etymon/`（86/98 字根图）、`split-table/`（86/98 拆字表）、`fonts/`（字根 PUA 字体）。由 `xtask resources` 打包，缓解 `R13`
