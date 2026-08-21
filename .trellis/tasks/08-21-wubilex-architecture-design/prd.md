# WubiLexTool 架构设计与项目骨架

## Goal

把 `docs/02-architecture.md` 从「需求期架构**映射**」（含大量「建议」与未定案岔路）收敛为**定案版架构设计**，覆盖三件事：**架构设计**、**项目目录结构**、**技术选型**；并按定案结构在仓库中创建**空目录骨架**（不可 build），使 S0 阶段开工时无需再做结构决策。

现状：需求文档集 16 份 / 7,163 行 / 630 条需求已完成并归档（`08-20-wubilex-requirements-analysis`）。`docs/02-architecture.md` 已有分层雏形、crate 划分、command/event 契约、依赖选型表、D1–D7 设计决策、测试策略、风险登记册 R1–R52。缺的是**定案**与**可落地的结构**。

## Scope

### In

1. 修订 `docs/02-architecture.md` 为定案版（用户选择「直接修订 02」，不新增 3x 系列文档）。
2. 修订 `docs/21-ui-ux.md` §4.6 与 `UX-TOKEN-011`，从 Tailwind v3 写法改为 v4 CSS-first。
3. 同步受影响的交叉引用（`00-overview.md`、`README.md`、`22-roadmap.md` S0/S1 条目）。
4. 按定案结构创建目录树 + 占位文件。

### Out

- 任何可编译产物：**不生成** `Cargo.toml` / `package.json` / `tauri.conf.json` / `vite.config.ts` / 任何 `.rs` 或 `.ts` 源码。
- 需求条目的增删（ID 集合与 630 计数不变；仅允许在语义不变前提下改写 `UX-TOKEN-011` 描述）。
- UI 视觉稿、组件设计、API 函数签名（属实现期决策）。
- 原项目 `wubi-lex/` 的任何改动（只读历史快照）。

## Requirements

### R-A 架构定案

| # | 需求 | 验收方式 |
|---|---|---|
| A1 | 六个 crate 的职责、**允许/禁止依赖**、对应需求域保持并细化为可执行的分层约束 | `02` §1 分层原则表覆盖 6 个 crate，每个都有「禁止」列 |
| A2 | 补齐 **crate 间接口缝（trait 边界）**：网络客户端、系统操作 dry-run、learn→core 入库出口 | `02` 新增章节列出 ≥3 个 trait 缝，每个注明所在 crate、被谁实现、为什么要抽象 |
| A3 | 明确 `src-tauri` 与 workspace 的关系、`wubilex-learn`（P2/S8）是否进默认 members | `02` §9 有明确结论与理由 |
| A4 | 补齐**构建与打包流水线**、`xtask` 职责边界 | `02` 有 xtask 子命令清单与「什么进 xtask、什么不进」 |
| A5 | 保留 D1–D7 并追加本轮新决策（编号连续 D8+），每条含背景/决策/代价 | `02` §7 决策数 ≥ 12，编号无跳号无复用 |

### R-B 技术选型定案

| # | 需求 | 验收方式 |
|---|---|---|
| B1 | 前端框架定案为 **React 19**，去掉「Vue / Svelte 同样可行」的未定案表述 | `02` §6 无「建议」字样，框架行标注为已定案 |
| B2 | 样式定案为 **Tailwind CSS v4（CSS-first）**：`@tailwindcss/vite` + `@theme` + `@custom-variant dark`；不再有 `tailwind.config.ts` | `02` §6、§9 与 `21` §4.6 三处一致，全库无 `theme.extend` / `<alpha-value>` 残留 |
| B3 | 依赖表补 **版本基线列**，版本取自 2026-08-21 实测 registry | `02` §5.1、§6.1 每行有版本；表头注明基线日期与「实测」来源 |
| B4 | 关闭文档中遗留的选型岔路：LZMA 压缩侧、简繁转换、配置存储、路由、图标 | 每条岔路在 `02` 有唯一结论 + 备选与触发切换的条件 |
| B5 | 选型变更须回写风险登记册（至少 R33 `opencc-rust`、R37 LZMA 状态更新） | `02` §10 对应风险行的「缓解」列反映新决策 |

### R-C 目录结构定案

| # | 需求 | 验收方式 |
|---|---|---|
| C1 | `02` §9 从「建议」改为「定案」，并与实际创建的目录树**逐行一致** | 文档树与 `git ls-files` 结果比对无差异 |
| C2 | 前端目录按 `UX-IA-001` 的 **7 个一级领域**（概览/码表/短语/反查/字根/学习/设置）组织，而非按旧模块号 | `src/routes/` 下有且仅有 7 个领域目录 |
| C3 | 后端 `commands/` 按 `02` §3.2 的 command 前缀分文件 | 目录树中 commands 子项与前缀表一一对应 |
| C4 | 每个 crate 根有 `README.md` 声明：职责 / 允许依赖 / 禁止依赖 / 对应需求域 / 所属阶段 | 6 个 crate README 齐全，内容与 `02` §1 分层表一致 |
| C5 | 叶子目录用 `.gitkeep` 占位，确保空目录可入库 | `git status` 能看到全部目录 |

### R-E 工具链定案

开发机实测（2026-08-21）：Volta 2.0.2 · Node 24.18.1（default）· pnpm 11.18.0（**以 volta 全局 package 形式安装**）· Rust 1.97.1 stable-x86_64-pc-windows-msvc。

| # | 需求 | 验收方式 |
|---|---|---|
| E1 | Node 版本由 **Volta 项目级 pin** 管理（`package.json` 的 `volta` 字段），不用 `.nvmrc` / corepack / `packageManager` 字段 | `02` 有工具链章节写明 pin 机制与「为什么不并用 corepack」 |
| E2 | 包管理器定案为 **pnpm**，版本同样由 Volta pin | `02` 全文无 `npm install` / `yarn` 指令；CI 用 pnpm |
| E3 | Rust 工具链由 `rust-toolchain.toml` pin（channel + `rustfmt`/`clippy` + msvc target） | `02` 工具链章节列出该文件内容形状 |
| E4 | CI 从 `package.json` 的 `volta` 字段取 Node/pnpm 版本，与本地同源 | `02` §CI 明确 CI 的版本来源是 volta 字段而非硬编码 |
| E5 | 记录「pnpm 同时以 volta package 与 package-manager 两种身份存在」的解析风险 | `02` §10 有对应风险行 |

> 工具链配置文件（`package.json`、`rust-toolchain.toml` 等）**本任务不创建**——它们属可编译配置，归 S0。本任务只把决策写进文档并在目录树中标注。

### R-D 文档一致性（继承既有契约）

遵循 `.trellis/spec/guides/requirement-id-conventions.md`：

| # | 需求 | 验收方式 |
|---|---|---|
| D1 | 需求 ID 计数不变：模块 414 / 非功能 101 / UI-UX 115 / 总计 630 | 该文档 §3 计数命令输出不变 |
| D2 | 无悬空引用 | 该文档 §4 校验命令无输出 |
| D3 | 无 `TBD` / `待补充` 占位符 | 该文档 §5 校验命令无输出 |
| D4 | `UX-TOKEN-011` 语义不变（Tailwind + class 驱动深色 + 令牌不写字面值），仅机制描述更新，**ID 不作废** | 改后仍为 P0，仍指向 §4.6 |

## Acceptance Criteria

- [ ] `docs/02-architecture.md` 全文无「建议」性质的未定案表述（§5.2 的「选型注意」与备选说明除外，且须明确标为备选）
- [ ] `docs/02-architecture.md` §9 目录树与仓库实际目录**逐行一致**
- [ ] `docs/21-ui-ux.md` §4.6 为 Tailwind v4 写法；`UX-TOKEN-011` 描述同步；全库 grep 无 `theme.extend`、`<alpha-value>`、`tailwind.config` 残留
- [ ] `docs/02-architecture.md` §7 决策编号连续（D1..Dn），新增决策各含背景/决策/代价
- [ ] `docs/02-architecture.md` §5.1 / §6.1 每个依赖有版本基线，表头注明「2026-08-21 实测」
- [ ] 目录骨架已创建：6 个 `crates/*`（含 README.md）、`src-tauri/`、`src/`（7 个领域路由）、`xtask/`
- [ ] `docs/02-architecture.md` 有工具链章节，覆盖 Volta pin / pnpm / `rust-toolchain.toml` / CI 版本来源四项
- [ ] 仓库中**无** `Cargo.toml` / `package.json` / `tauri.conf.json` / `*.rs` / `*.ts`（骨架不可 build，符合本次范围）
- [ ] `requirement-id-conventions.md` §3 计数输出为 `模块=414 非功能=101 UI/UX=115 总计=630`
- [ ] `requirement-id-conventions.md` §4 悬空引用校验无输出
- [ ] `requirement-id-conventions.md` §5 占位符校验无输出
- [ ] `docs/22-roadmap.md` 的 S0 条目「workspace 与 6 个 crate 骨架」标注为本任务已完成目录部分

## Constraints

- **`docs/02-architecture.md` §0 的首要原则不可动摇**：旧项目是行为规格不是实现范本；§0.1 的 12 条行为契约（C1–C12）不因架构调整而变化。
- 目标平台 Windows 独占（Win10 1703+ / Win11），不做跨平台抽象。
- 本任务**只做结构与选型**，不写业务逻辑；任何「顺手实现一点」都超范围。
- 版本基线是**基线不是锁定**：S0 开工时需重新核对并写入真实的 `Cargo.toml` / `package.json`。

## Open Questions

无。三个岔路（前端框架 / Tailwind 版本 / 文档落点）已于 2026-08-21 由用户定案：React 19、Tailwind v4、直接修订 `02`。
