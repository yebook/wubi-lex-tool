# wubilex-core

> 码表与短语的内存模型，及其上的全部领域操作。纯逻辑、同步、可脱离 UI 单测。

## 职责

承接 `wubilex-codec` 解出的数据，提供领域模型与变换。**这一层的存在本身就是对原项目最大结构缺陷的修正** —— 原项目把造词、格式转换、精简、词频优化全部内嵌在 `dlg/dict/lex.aardio` 的 1,455 行菜单回调里，既不能测也不能复用。

| 目录 | 内容 | 需求 / 契约 |
|---|---|---|
| `src/table/` | 码表模型 + **倒排索引** | `M1-PARSE-017` · 缓解 `R5` |
| `src/phrase/` | 短语模型 | `M2-*` |
| `src/transform/` | 格式转换（9 项） | `M1-XFORM-*` |
| `src/slim/` | 精简（7 项） | `M1-SLIM-*` |
| `src/weight/` | 词频与权重优化（9 项） | `M1-WEIGHT-*` |
| `src/coin/` | 造词三规则 + 空码造词 | `M1-COIN-*` · **契约 C7** |
| `src/split/` | 短语分离 + 键名占用判定 | `M1-SPLIT-*` · **契约 C8** |
| `src/lookup/` | 编码反查与拆字组合规则 | `M3-QUERY-*` · `M3-SPLIT-002` |
| `src/convert/` | 简繁 / 拼音 / GB2312 判定 | `M1-XFORM-008/009` · `M1-SLIM-004` · `M3-QUERY-005` |
| `src/ports/` | 出站端口 trait：`PhraseSink`（S3）、`ResourceProvider`（S4） | 见下 |

## 允许依赖

`wubilex-codec`。

## 禁止依赖

Windows API、Tauri、网络。

**「禁止网络」有个具体后果**：本 crate 需要拆字数据表与词频文件，但**不能**去拿。解法是 `src/ports/ResourceProvider` —— core 声明「我要一份 86 版拆字表」，由 `wubilex-app` 决定从内置资源、缓存还是下载满足它。

同样地，`src/ports/PhraseSink` 是 `wubilex-learn` 的入库出口：learn **禁止直写系统文件**（`M8-APPLY-004`），只能把结果交给这个 trait，物理写入由 app 编排。

与 `wubilex-codec` 相同的 S5 约束：**禁止依赖 `tokio`**，纯同步；取消收 `&AtomicBool`，进度收 `&mut dyn FnMut(P)`。

## 对应需求域

`M1-XFORM` · `M1-SLIM` · `M1-WEIGHT` · `M1-COIN` · `M1-SPLIT` · `M1-PARSE`（模型层）· `M2`（模型层）· `M3-QUERY` / `M3-SPLIT`

## 所属阶段

模型与索引在 **S2**（只读闭环），变换类操作在 **S4**（编辑与变换）。

关键出口条件：完整码表（数十万条）加载 ≤ 5 s（`NFR-PERF-002`）、单次反查 ≤ 50 ms（`NFR-PERF-004`）。

## 两条契约提醒

**C7 造词规则**与 **C8 键名占用判定**必须与原项目行为一致 —— 不是因为原实现好（C8 是拍脑袋的经验规则），而是因为它们的**输出定义了行为**：造词编码要与其他五笔工具一致，否则用户换工具就乱套。做法是原样实现判定逻辑 + 修掉确认的 bug + 写测试固化，并在代码注释里标明其经验性质。

其余一切（数据结构、算法、并发）自由，见 [`docs/02-architecture.md`](../../docs/02-architecture.md) §0.2。
