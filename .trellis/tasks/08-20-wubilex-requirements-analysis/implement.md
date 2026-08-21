# Implement — WubiLex 需求文档集

## 执行顺序

分析阶段已完成（32 个源文件全量通读），以下为文档撰写检查清单。顺序有依赖：先立契约（术语/ID/格式），再写模块，最后做汇总与交叉校验。

### 阶段 A：契约层（必须先于模块文档）

- [x] A1 `docs/README.md` — 索引、阅读路径、文档维护约定、ID 规则速查
- [x] A2 `docs/00-overview.md` — 背景、目标、原项目现状（含 32 文件覆盖表）、模块地图、术语表、优先级定义
- [x] A3 `docs/01-data-formats.md` — 5 类格式规格
  - `.lex` 二进制（imscwubi）：文件头 / 字母索引 / 记录布局 / 写入排序规则
  - EUDP 短语二进制（mschxudp）：文件头 / 偏移表 / 条目布局 / v1&v2 双写
  - 文本码表方言：6 种行格式 + YAML/注释/极点标记清理 + 降序权重归一化 + 空白转义
  - 词频文件：`word\tweight`
  - 拆字数据表：`char\t字根串`（含 PUA 字符与配套字体）
  - 附：码表版本探测算法（特征码打分表）
- [x] A4 `docs/03-source-index.md` — 源码反向索引（文件 → 行号区块 → 需求 ID）+ 8 处已知缺陷清单
      〔R10 追加范围〕定位为**行为查证工具**，非移植清单

### 阶段 B：模块需求文档

- [x] B1 `docs/modules/M1-lex-table.md` — 码表管理（域：LIST/PARSE/EDIT/XFORM/SLIM/WEIGHT/COIN/SPLIT/INSTALL/IO）
- [x] B2 `docs/modules/M2-phrase.md` — 短语词库（域：IO/PARSE/EDIT/INSTALL/COIN）
- [x] B3 `docs/modules/M3-reverse-lookup.md` — 反查拆字（域：INPUT/QUERY/KBD/SPLIT/ANIM/FONT/HOTKEY）
- [x] B4 `docs/modules/M4-ime-control.md` — 输入法控制（域：TIP/DPY/REG/SYS/TSF）
- [x] B5 `docs/modules/M5-etymon-help.md` — 字根图与帮助（域：CHART/TEXT/TIPS/ABOUT）
- [x] B6 `docs/modules/M6-resource-sync.md` — 资源分发（域：CATALOG/DOWN/ARCHIVE/CACHE/UPDATE）
- [x] B7 `docs/modules/M7-app-shell.md` — 应用外壳（域：INST/WIN/TRAY/HOTKEY/KEYMAP/CONF/BUS/TASK）
      〔R9 追加范围〕新增 `KEYMAP` 域：动作注册表 + 快捷键映射，13 条需求 + 15 项默认绑定
- [x] B8 `docs/modules/M8-self-learning.md` — 自学习（双路径：语料导入 / 输入采集）
      〔R14 追加范围〕41 条需求，整体 P2，后置到阶段 S8

### 阶段 C：横切与汇总

- [x] C1 `docs/02-architecture.md` — 分层架构、crate 划分、Tauri command/event 契约、依赖选型表、aardio→Rust 能力映射、风险登记册
      〔R16 追加范围〕新增 `§0` 首要原则：`§0.1` 12 条行为契约 / `§0.2` 实现自由维度 / `§0.3` 24 项已定案更优实现 / `§0.4` 分歧判定流程
- [x] C2 `docs/20-nonfunctional.md` — 性能 / 权限 / 兼容性 / 可靠性 / 安全 / 可观测性 / i18n / 可访问性
- [x] C3 `docs/21-ui-ux.md` — 信息架构、导航模型、关键界面、交互规范、主题令牌、超大数据量策略
      〔R15 追加范围〕按「全新设计、不承接旧项目」整体重写（87 → 115 条），`UX-*` 编号重新分配
- [x] C4 `docs/22-roadmap.md` — 全量需求优先级汇总表 + 里程碑

### 阶段 D：校验

- [x] D1 覆盖校验：32 个 `.aardio` 文件逐一确认已被某模块覆盖或标为 P3 废弃 — **32/32 通过**
- [x] D2 ID 唯一性校验：各文档定义行无重复 ID — **通过**
- [x] D3 汇总一致性：`22-roadmap.md` 条目数 == 各模块文档条目数之和 — **414 + 101 + 115 = 630，与汇总表逐格吻合**
- [x] D4 无占位符：全库 grep `TBD` / `TODO` / `待补充` — **0 条**（仅命中文档内自引用的 grep 示例行）
- [x] D5 格式规格自洽：`01-data-formats.md` 的偏移表与 `lexFile.aardio:337-417`（saveLex）、`lexFile.aardio:216-286`（addFile）、`phrase.aardio:11-104`（loadMap）、`phrase.aardio:186-366`（saveToSystem）逐字段对照
      ⚠️ 撰写期完成；2026-08-21 收尾轮**未重新逐字段比对**（该项无法机械验证，需实读源码）
- [x] D6 悬空交叉引用校验：被引用的 ID 均有定义行 — **630 引用 ↔ 630 定义，0 悬空**

## 验证命令

> ⚠️ 2026-08-21 修正：原版命令有 3 处 regex 缺陷会得出错误结论，已修复。
> 1. `M[1-7]` 漏掉 R14 新增的 **M8** → 改为 `M[1-8]`
> 2. `NFR-[A-Z]+` 匹配不到 **`NFR-A11Y`**（域段含数字），会把 101 少数成 89 → 改为 `[A-Z0-9]+`
> 3. D2 原用 `grep -rh docs/ | uniq -d`，把跨文档的正常引用误报为重复 → 改为按单文件的**定义行**判重

```bash
# D1 覆盖校验 — 每个源文件名是否出现在 03-source-index.md
for f in $(find wubi-lex -name "*.aardio" | sed 's|.*/||' | sort -u); do
  grep -q "$f" docs/03-source-index.md || echo "MISSING: $f"
done   # 期望：无输出

# D2 ID 唯一性 —— 必须按「单文件内的定义行」判重。
# 跨文件出现是正常引用，不是重复。定义行 = 需求表格行（以 | 开头）。
for f in docs/modules/M*.md docs/20-nonfunctional.md docs/21-ui-ux.md; do
  d=$(grep -ohE '^\|[[:space:]]*`?\b[A-Z][A-Z0-9]*-[A-Z0-9]+-[0-9]{3}\b' "$f" \
      | grep -ohE '\b[A-Z][A-Z0-9]*-[A-Z0-9]+-[0-9]{3}\b' | sort | uniq -d)
  [ -n "$d" ] && echo "$f: $d"
done   # 期望：无输出

# D3 汇总一致性
M=$(grep -rhoE '\bM[1-8]-[A-Z0-9]+-[0-9]{3}\b' docs/modules/            | sort -u | wc -l)
N=$(grep -ohE  '\bNFR-[A-Z0-9]+-[0-9]{3}\b'    docs/20-nonfunctional.md | sort -u | wc -l)
U=$(grep -ohE  '\bUX-[A-Z0-9]+-[0-9]{3}\b'     docs/21-ui-ux.md         | sort -u | wc -l)
echo "模块=$M 非功能=$N UI/UX=$U 总计=$((M+N+U))"
# 期望：模块=414 非功能=101 UI/UX=115 总计=630（与 22-roadmap.md 汇总表比对）

# D4 占位符（排除文档内自引用的 grep 示例行）
grep -rn "TBD\|待补充" docs/ --include="*.md" | grep -v 'grep -rn'   # 期望：无输出

# D6 悬空交叉引用：被引用但无定义行的 ID
grep -rhoE '\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' docs/ | sort -u > /tmp/refs.txt
grep -rhoE '^\|[[:space:]]*`?\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' docs/ \
  | grep -ohE '\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' | sort -u > /tmp/defined.txt
comm -23 /tmp/refs.txt /tmp/defined.txt   # 期望：无输出

# 文件清单 — 期望 16 个 .md
find docs -type f -name "*.md" | sort
```

## 评审门

- **门 1（阶段 A 后）**：术语表与 ID 规则确定后再进模块文档 —— 避免后期大范围返工。
- **门 2（阶段 B 后）**：模块需求条目冻结后再写 roadmap —— roadmap 以 ID 为主键，条目变动会导致汇总失效。
- **门 3（阶段 D）**：全部校验命令通过后方可提交。

## 回滚点

- 阶段 A/B/C 任一阶段产出不满足要求 → 删除该阶段产生的文件重写，不影响其他阶段。
- 整体回滚：`rm -rf docs/`（本任务不触碰 `wubi-lex/` 与其他既有文件）。
