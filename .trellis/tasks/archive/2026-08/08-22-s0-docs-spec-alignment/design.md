# Design - S0-00 文档与规范校正

## 1. Source Of Truth

采用以下优先级消解冲突：

1. 需求定义表决定需求数量与优先级。
2. 模块需求中的功能归属决定缺陷所属阶段。
3. `docs/02-architecture.md` 的缺陷清单决定已知缺陷总数。
4. `docs/22-roadmap.md` 只汇总和排期，不创造新的需求或缺陷。

因此 P0 为 208、缺陷为 6。路线图中的 200/8 是汇总漂移，应修正汇总而不改需求定义。

## 2. Defect Stage Mapping

| Defect | Owning requirement | Stage | Reason |
|---|---|---|---|
| `XFXY` 大写 | `M1-PARSE-013` | S0 | 版本探测属于 codec。 |
| 空白转义不对称 | `M1-PARSE-008`, `M2-PARSE-007` | S0 | 文本编解码属于 codec。 |
| `codeWeight[0]` 死代码 | 码表文本解析合同 | S0 | 新解析器省略死分支并固化行为。 |
| EUDP 拖放被文本覆盖 | `M2-IO-007` | S4 | 文件载入编辑器属于完整编辑体验；S4 同时补 `M2-IO-005..007` 的缺失排期。 |
| 郑码造词重复 | `M2-COIN-001` | S4 | 造词明确在 S4。 |
| `unique()` 不递增 | `M1-SLIM-007` | S4 | 精简明确在 S4。 |

`NFR-MAINT-004` 仍表示 v1.0 前六项全部有回归测试；这里只修正各阶段出口，不降低最终要求。

## 3. Research Gate Wording

统一表述为：四项预研在 S0 与主线并行，结果是 S1 的入口条件。通过则进入 S1；不通过则必须先完成架构复评。它们不阻塞 S0 内其他 codec 工作，但阻塞 S1。

## 4. Initial Spec Baseline

本任务更新六份最接近 S0 的规范：

- `backend/directory-structure.md`：实际 workspace/目录与依赖边界。
- `backend/error-handling.md`：库错误、应用错误和禁止 panic/unwrap 的边界。
- `backend/quality-guidelines.md`：fmt、clippy、测试、覆盖率与二进制解析禁区。
- `frontend/directory-structure.md`：实际目录、S0 最小壳和 S1 领域路由边界。
- `frontend/type-safety.md`：Rust 生成 TS、禁止手写共享 IPC 类型。
- `frontend/quality-guidelines.md`：pnpm、TypeScript、ESLint、Vitest 和生成物检查。

这些文件按现有 index 约定使用英文，记录已定案合同，并引用真实目录/文档。没有实际源代码时不写伪代码示例；index 使用“baseline / examples pending”状态。其余五份模板保持待后续实际实现补充。

## 5. Compatibility

不新增、删除、改号或改优先级；只修正文案和阶段映射。因此需求 ID 集合与 630 总数必须完全不变。修改章节正文后仍执行全量锚点校验。

## 6. Rollback

改动只涉及 `docs/22-roadmap.md`、六份 `.trellis/spec/`、两个 spec index，以及仍引用旧口径的 `crates/wubilex-codec/README.md` 与 `crates/wubilex-winime/README.md`。任何校验失败时，逐文件修正文案；不得通过改需求 ID 或降低校验规则来消除失败。
