# wubilex-resource

> 网络获取、解压、校验、缓存。原项目在这一层的做法是**反面教材**：明文 HTTP、无校验、无镜像、`eval()` 反序列化。

## 职责

| 目录 | 内容 | 需求 / 风险 |
|---|---|---|
| `src/http/` | `HttpClient` trait + `reqwest` 实现 + 测试 mock | 见下 |
| `src/catalog/` | 在线码表目录拉取与缓存 | `M6-CATALOG-*` · **`R11`** |
| `src/download/` | 下载：进度 / 取消 / 重试 / 断点续传 | `M6-DOWN-*` |
| `src/archive/` | LZMA 解压 · zstd 读写 · TAR + **路径穿越防护** | `M6-ARCHIVE-*` · **`R12`** |
| `src/cache/` | 缓存目录管理与清理 | `M6-CACHE-*` · `R27` |
| `src/verify/` | SHA-256 校验 | `M6-DOWN-011` |

## 允许依赖

HTTP / 压缩 crate。`reqwest` 用 **`rustls-tls`** 而非 `native-tls`，避免依赖系统 Schannel 配置。

## 禁止依赖

Tauri、业务逻辑（**不解析下载内容** —— 那是 `wubilex-codec` / `wubilex-core` 的事）。

## `HttpClient` 必须可 mock

网络层抽象为 trait（S1 缝），理由不只是「方便测试」：

`R11`（目录缓存的恶意 payload）与 `R12`（TAR 路径穿越）的防护**只有能构造恶意响应才验证得了**。真实 HTTP 客户端做不到这件事。

## 四条硬性安全约束

| # | 约束 | 缓解的风险 |
|---|---|---|
| 1 | **全部资源走 HTTPS + SHA-256 校验** | `R3` — 原项目全链路明文 HTTP 无校验 |
| 2 | **严格 `serde` 反序列化目录缓存**，绝不 `eval` 等价物 | `R11` — 原项目用 `eval()` 反序列化，是完整的 RCE 链路 |
| 3 | **TAR 解包路径规范化 + 拒绝 `..`** | `R12` — 任意文件写入 |
| 4 | **缓存与用户数据物理分离** | `R27` — 原项目把二者混放在 `wubi/lex/`，清缓存会误删用户手工添加的码表 |

目录分离的定案（`docs/00-overview.md` §6）：

```
%LOCALAPPDATA%\WubiLexTool\cache\   可清理
%LOCALAPPDATA%\WubiLexTool\data\    用户数据，不可清理
```

## 压缩格式：LZMA 只读，写一律 zstd

`lzma-rs` 只支持解压（2026-08-21 实测 `0.3.0` 仍然如此）。定案（`docs/02-architecture.md` D14）：

- **读** LZMA alone —— 兼容上游既有的 `spelling.tar.lzma` 等历史资源
- **写** 一律 zstd —— `M1-IO-004` 的导出产出 `.lex.zst`
- **不引入 `xz2`**（绑定 C 的 liblzma）

## 对应需求域

`M6-CATALOG` · `M6-DOWN` · `M6-ARCHIVE` · `M6-CACHE` · `M6-UPDATE`

## 所属阶段

**S5 — 反查完整 + 资源分发**。

但 `R13`（上游服务器单点）的缓解要更早落地：86/98 的拆字表、字根图、字根字体**内置于安装包**（`src-tauri/resources/`），由 `xtask resources` 在构建期打包。这意味着断网也能用核心功能。
