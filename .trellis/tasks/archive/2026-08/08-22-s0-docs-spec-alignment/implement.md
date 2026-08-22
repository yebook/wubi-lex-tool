# Implementation Plan - S0-00 文档与规范校正

## 1. Baseline

- [x] 记录模块/NFR/UX 唯一需求数，应为 414/101/115，总计 630。
- [x] 运行 `python ./.trellis/scripts/check_anchors.py`，应为 0 处失效。
- [x] 搜索 P0 数量、缺陷数量和预研门槛的所有表述，确认修改面完整。

## 2. Correct `docs/22-roadmap.md`

- [x] 把 P0 清单说明从 200 改为 208。
- [x] 把 S0 的“8 个缺陷”改为 3 个 codec 所属回归，并列出具体缺陷。
- [x] 在 S4 增加 `M2-IO-005..007` 和三个 S4 缺陷回归的归属说明。
- [x] 把 S0 与技术预研章节统一为“S0 并行、阻塞 S1；失败先复评”。
- [x] 保留 `NFR-MAINT-004` 的 v1.0 六项总要求，不改变需求定义。
- [x] 同步 codec/winime README 中的缺陷范围和技术预研门槛。

## 3. Establish Initial Specs

- [x] 更新 backend `directory-structure.md`、`error-handling.md`、`quality-guidelines.md`。
- [x] 更新 frontend `directory-structure.md`、`type-safety.md`、`quality-guidelines.md`。
- [x] 更新 backend/frontend `index.md` 状态，区分 baseline 与尚待真实实现的规范。
- [x] 搜索每条拟写规则的现有来源；无来源的规则不写入。
- [x] 不添加虚构代码示例，不归档 `00-bootstrap-guidelines`。

## 4. Validation

```powershell
# Unique requirement counts
$module = rg -o '\bM[1-8]-[A-Z0-9]+-[0-9]{3}\b' docs/modules | ForEach-Object { ($_ -split ':')[-1] } | Sort-Object -Unique
$nfr = rg -o '\bNFR-[A-Z0-9]+-[0-9]{3}\b' docs/20-nonfunctional.md | ForEach-Object { ($_ -split ':')[-1] } | Sort-Object -Unique
$ux = rg -o '\bUX-[A-Z0-9]+-[0-9]{3}\b' docs/21-ui-ux.md | ForEach-Object { ($_ -split ':')[-1] } | Sort-Object -Unique
"module=$($module.Count) nfr=$($nfr.Count) ux=$($ux.Count) total=$($module.Count + $nfr.Count + $ux.Count)"

python ./.trellis/scripts/check_anchors.py
rg -n '200 条 P0|8 个缺陷|不阻塞 S1|须在 S3 前完成|再进入 S3' docs/22-roadmap.md crates/wubilex-codec/README.md crates/wubilex-winime/README.md
rg -n 'TBD|待补充' docs -g '*.md'
git diff --check
git status --short
```

- [x] 计数输出为 `module=414 nfr=101 ux=115 total=630`。
- [x] 锚点输出为 `0 处失效`。
- [x] 冲突表述搜索无输出。
- [x] `docs/` 占位符搜索只有自检命令本身，或经排除后无输出。
- [x] diff 中无产品源码、工程配置或 `wubi-lex/` 改动。

## 5. Review And Rollback

- [x] 对照 `prd.md` 逐项复核，不扩大到 S0-01。
- [x] 校验失败时修正对应文档；不回退用户已有改动，不降低验证标准。
- [x] 质量检查通过后再提交并归档本子任务。
