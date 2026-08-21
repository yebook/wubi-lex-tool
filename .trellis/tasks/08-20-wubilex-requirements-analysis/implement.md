# Implement — WubiLex 需求文档集

## 执行顺序

分析阶段已完成（32 个源文件全量通读），以下为文档撰写检查清单。顺序有依赖：先立契约（术语/ID/格式），再写模块，最后做汇总与交叉校验。

### 阶段 A：契约层（必须先于模块文档）

- [ ] A1 `docs/README.md` — 索引、阅读路径、文档维护约定、ID 规则速查
- [ ] A2 `docs/00-overview.md` — 背景、目标、原项目现状（含 32 文件覆盖表）、模块地图、术语表、优先级定义
- [ ] A3 `docs/01-data-formats.md` — 5 类格式规格
  - `.lex` 二进制（imscwubi）：文件头 / 字母索引 / 记录布局 / 写入排序规则
  - EUDP 短语二进制（mschxudp）：文件头 / 偏移表 / 条目布局 / v1&v2 双写
  - 文本码表方言：6 种行格式 + YAML/注释/极点标记清理 + 降序权重归一化 + 空白转义
  - 词频文件：`word\tweight`
  - 拆字数据表：`char\t字根串`（含 PUA 字符与配套字体）
  - 附：码表版本探测算法（特征码打分表）

### 阶段 B：模块需求文档

- [ ] B1 `docs/modules/M1-lex-table.md` — 码表管理（域：LIST/PARSE/EDIT/XFORM/SLIM/WEIGHT/COIN/SPLIT/INSTALL/IO）
- [ ] B2 `docs/modules/M2-phrase.md` — 短语词库（域：IO/PARSE/EDIT/INSTALL/COIN）
- [ ] B3 `docs/modules/M3-reverse-lookup.md` — 反查拆字（域：INPUT/QUERY/KBD/SPLIT/ANIM/FONT/HOTKEY）
- [ ] B4 `docs/modules/M4-ime-control.md` — 输入法控制（域：TIP/DPY/REG/SYS/TSF）
- [ ] B5 `docs/modules/M5-etymon-help.md` — 字根图与帮助（域：CHART/TEXT/TIPS/ABOUT）
- [ ] B6 `docs/modules/M6-resource-sync.md` — 资源分发（域：CATALOG/DOWN/ARCHIVE/CACHE/UPDATE）
- [ ] B7 `docs/modules/M7-app-shell.md` — 应用外壳（域：INST/WIN/TRAY/HOTKEY/CONF/BUS/TASK）

### 阶段 C：横切与汇总

- [ ] C1 `docs/02-architecture.md` — 分层架构、crate 划分、Tauri command/event 契约、依赖选型表、aardio→Rust 能力映射、风险登记册
- [ ] C2 `docs/20-nonfunctional.md` — 性能 / 权限 / 兼容性 / 可靠性 / 安全 / 可观测性 / i18n / 可访问性
- [ ] C3 `docs/21-ui-ux.md` — 信息架构、导航模型、关键界面、交互规范、主题令牌、超大数据量策略
- [ ] C4 `docs/22-roadmap.md` — 全量需求优先级汇总表 + 里程碑

### 阶段 D：校验

- [ ] D1 覆盖校验：32 个 `.aardio` 文件逐一确认已被某模块覆盖或标为 P3 废弃
- [ ] D2 ID 唯一性校验：全库 grep 需求 ID，无重复
- [ ] D3 汇总一致性：`22-roadmap.md` 条目数 == 各模块文档条目数之和
- [ ] D4 无占位符：全库 grep `TBD` / `TODO` / `待补充`，应为 0（除非显式标注为「原项目未实现」）
- [ ] D5 格式规格自洽：`01-data-formats.md` 的偏移表与 `lexFile.aardio:337-417`（saveLex）、`lexFile.aardio:216-286`（addFile）、`phrase.aardio:11-104`（loadMap）、`phrase.aardio:186-366`（saveToSystem）逐字段对照

## 验证命令

```bash
# D1 覆盖校验 — 列出全部源文件，人工比对覆盖表
find wubi-lex -name "*.aardio" | sort

# D2 ID 唯一性
grep -rhoE '\bM[1-7]-[A-Z]+-[0-9]{3}\b' docs/ | sort | uniq -d

# D3 汇总一致性
grep -rcoE '\bM[1-7]-[A-Z]+-[0-9]{3}\b' docs/modules/
grep -coE '\bM[1-7]-[A-Z]+-[0-9]{3}\b' docs/22-roadmap.md

# D4 占位符
grep -rn "TBD\|待补充" docs/

# D5 文件清单
find docs -type f -name "*.md" | sort
```

## 评审门

- **门 1（阶段 A 后）**：术语表与 ID 规则确定后再进模块文档 —— 避免后期大范围返工。
- **门 2（阶段 B 后）**：模块需求条目冻结后再写 roadmap —— roadmap 以 ID 为主键，条目变动会导致汇总失效。
- **门 3（阶段 D）**：全部校验命令通过后方可提交。

## 回滚点

- 阶段 A/B/C 任一阶段产出不满足要求 → 删除该阶段产生的文件重写，不影响其他阶段。
- 整体回滚：`rm -rf docs/`（本任务不触碰 `wubi-lex/` 与其他既有文件）。
