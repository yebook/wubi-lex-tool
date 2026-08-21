# wubilex-codec

> 字节 ↔ 内存模型的双向转换。无状态、纯函数、零平台依赖。

## 职责

`.lex` / EUDP 二进制与各类文本格式的解析与序列化。**全项目行为契约最集中的一层** —— `docs/02-architecture.md` §0.1 的 12 条契约中有 8 条落在这里。

| 目录 | 内容 | 行为契约 |
|---|---|---|
| `src/lex/` | `.lex` 微软五笔码表二进制读写（魔数 `imscwubi`） | **C1** |
| `src/eudp/` | EUDP 用户短语库二进制读写（魔数 `mschxudp`） | **C2** |
| `src/text/` | 文本码表 6 种方言 + 微软码表分支 + 7 种输出格式；短语文本方言与 `$[...]` 语义 | **C5** · **C6** |
| `src/weight/` | 词频文件读写 | **C12** |
| `src/split_table/` | 拆字数据表解析 | **C12** |
| `src/detect/` | 码表版本探测（8 方案；**含 `XFXY` 大写缺陷修复**） | **C9** |
| `src/escape/` | 空白字符转义 `%20` / `%09` / `%0D` / `%0A` | **C10** |

`tests/fixtures/` 存 8 方案真实码表回归集，由 `xtask fixtures` 拉取，**不入库**。

## 允许依赖

无平台依赖，仅标准库 + 解析/编码 crate。

## 禁止依赖

文件系统之外的 I/O、Windows API、Tauri。

额外约束（`docs/02-architecture.md` §2 的 S5 缝）：

- **禁止依赖 `tokio`** —— 本 crate 是纯同步的。取消信号收 `&AtomicBool`，不是 `CancellationToken`。
- 进度上报收 `&mut dyn FnMut(P)`，`P` 是本 crate 自己定义的进度枚举，不共享跨 crate 类型。

> 纯同步是刻意的：只有这样 `wubilex-app` 才能用 `rayon` 直接并行分片（更优实现 #18），以及用 `spawn_blocking` 把长任务挪出异步运行时。

## 对应需求域

`M1-PARSE-*` · `M2-IO-*` / `M2-PARSE-*` · `M1-WEIGHT-*`（文件层）· `M3-SPLIT-001`（数据表层）

数据格式的权威规格在 [`docs/01-data-formats.md`](../../docs/01-data-formats.md)，**不要**从原项目源码反推格式。

## 所属阶段

**S0 — 地基**。本 crate 是 S0 的主要产出，也是唯一有硬性覆盖率要求的 crate。

出口条件（`docs/22-roadmap.md` S0）：

- 测试覆盖率 **≥ 90%**（`NFR-MAINT-002`）
- 8 种方案各 ≥ 1 份真实码表往返测试**字节级一致**
- 7 种文本输出格式与原项目产物逐字节比对通过
- EUDP 往返测试通过（含 emoji 代理对、多行短语、`$[...]` 数组）
- 原项目 8 个缺陷各有对应的「失败 → 通过」测试

## 一条提醒

原项目 `lib/wubi/lexFile.aardio`（1,406 行）是**行为的权威定义，不是实现范本**。查行为用 [`docs/03-source-index.md`](../../docs/03-source-index.md) 反查；写实现按 [`docs/02-architecture.md`](../../docs/02-architecture.md) §0.2 / §0.3 自由取更优解。
