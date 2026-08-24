# Journal - ye (Part 1)

> AI development session journal
> Started: 2026-08-20

---



## Session 1: WubiLexTool 架构、目录结构、技术选型定案 + 目录骨架

**Date**: 2026-08-21
**Task**: WubiLexTool 架构、目录结构、技术选型定案 + 目录骨架
**Branch**: `main`

### Summary

把 docs/02-architecture.md 从需求期「架构映射」收敛为定案版，并按定案创建 109 个目录的骨架（不可 build，配置归 S0）。

用户定案三个岔路：前端 React 19、Tailwind 升 v4、直接修订 02 不新增 3x 系列。会话中期用户补充环境约束（Volta 管 Node、pnpm、Rust 已装），追加 D17 工具链决策。

新增决策 D8-D17：React 19 / Tailwind v4 CSS-first / src-tauri 进 workspace 但 learn 暂不进 / tauri-specta 生成 TS 类型 / 配置自建 TOML / 简繁内置表 / LZMA 只读导出用 zstd / SystemOps trait 双实现 / 占位开关单一源 / Volta pin 工具链。风险登记册加 R53-R55，回写 R33、R37。

补齐原文档三处空白：§2.7 crate 间五道接口缝、§3.5 IPC 类型契约同步、§8.5 构建与工具链流水线。§9 目录树改定案，前端路由按 UX-IA-001 的 7 领域重组（原树缺 overview/radicals、残留已移除的 help）。

关键判断：UX-TOKEN-011 改描述保留 ID（语义未实质变化），需求总数仍 630。emptyWorkingSet 的「待评估」没强行定案，改为标注 R41 追踪。

计划外发现：全库首次锚点扫描查出 7 处既有断链（章节改名/改号后未回改引用），已修并固化为 .trellis/scripts/check_anchors.py。写该脚本时首轮 39 条报告中 17 条是脚本自身假阳性——GitHub slugger 不 trim 首尾空白、相对路径要 normpath，教训已写进 requirement-id-conventions.md §6。

校验：计数 414/101/115/630 不变、悬空引用 0、占位符 0、锚点 0 失效、目录树 109/109 逐行一致、D1-D17 连续、无可编译文件混入、包管理器口径全库统一 pnpm。

### Git Commits

| Hash | Message |
|------|---------|
| `cb0ad41` | (see git log) |
| `45c5087` | (see git log) |
| `bf3880a` | (see git log) |

### Status

[OK] **Completed**


## Session 2: S0-00 路线图与开发规范校正

**Date**: 2026-08-22
**Task**: S0-00 路线图与开发规范校正
**Branch**: `main`

### Summary

校正 P0 总数、六项缺陷的 S0/S4 阶段归属与 S1 技术预研门槛；建立 backend/frontend 六份英文规范基线并同步 crate README。验证需求计数 414/101/115/630、P0 208、重复与悬空引用 0、锚点 0。

### Git Commits

| Hash | Message |
|------|---------|
| `3cfbd94` | (see git log) |

### Status

[OK] **Completed**


## Session 3: 完成 S0 workspace 与工具链

**Date**: 2026-08-22
**Task**: 完成 S0 workspace 与工具链
**Branch**: `main`

### Summary

建立六成员 Rust workspace、compile-only Tauri 壳和前端工具链；固定 Rust 1.97.1、Node 24.18.1，并直接使用全局 pnpm 11.18.0；同步架构与 Trellis 工具链规范，归档 S0-01 并更新父任务进度。

### Git Commits

| Hash | Message |
|------|---------|
| `c69a745` | (see git log) |
| `58493e8` | (see git log) |
| `a1767f9` | (see git log) |

### Status

[OK] **Completed**


## Session 4: 完成 S0-02 codec 公共模型

**Date**: 2026-08-22
**Task**: 完成 S0-02 codec 公共模型
**Branch**: `main`

### Summary

冻结 wubilex-codec 公共模型、结构化错误与输入限制；补齐 19 项集成测试和 backend 规范证据；归档子任务。

### Git Commits

| Hash | Message |
|------|---------|
| `3b554bc` | (see git log) |
| `619c690` | (see git log) |

### Status

[OK] **Completed**


## Session 5: 完成 S0-03 .lex 二进制编解码

**Date**: 2026-08-22
**Task**: 完成 S0-03 .lex 二进制编解码
**Branch**: `main`

### Summary

实现 raw .lex 安全双向编解码与 20 项格式测试；真实微软五笔 86 样本 207,055 条记录逐字节往返一致；同步 backend 规范并归档子任务。

### Git Commits

| Hash | Message |
|------|---------|
| `f07a112` | (see git log) |
| `cdd7f9c` | (see git log) |

### Status

[OK] **Completed**


## Session 6: S0 EUDP 二进制编解码

**Date**: 2026-08-23
**Task**: S0 EUDP 二进制编解码
**Branch**: `main`

### Summary

完成 EUDP 原始字节的严格解码、规范编码、结构化错误、测试覆盖与后端实现合同，并归档 s0-eudp 子任务。

### Git Commits

| Hash | Message |
|------|---------|
| `84f9ce7` | (see git log) |
| `30d432e` | (see git log) |
| `ce35e5b` | (see git log) |

### Status

[OK] **Completed**


## Session 7: S0 码表文本编解码

**Date**: 2026-08-23
**Task**: S0 码表文本编解码
**Branch**: `main`

### Summary

实现严格文本编码探测、社区码表方言解析、可见警告、空白转义和七种规范输出，并完成测试与 backend 规范同步

### Git Commits

| Hash | Message |
|------|---------|
| `e5393fd` | (see git log) |
| `3d9e110` | (see git log) |
| `1f966c1` | (see git log) |

### Status

[OK] **Completed**


## Session 8: S0 短语与辅助文本编解码

**Date**: 2026-08-24
**Task**: S0 短语与辅助文本编解码
**Branch**: `main`

### Summary

实现短语文本 P1-P6、多行、数组、时间别名与可见警告，新增词频和拆字表严格编解码及八方案探测；完成独立 Trellis 检查、规范同步和全量门禁。

### Git Commits

| Hash | Message |
|------|---------|
| `dc173ea` | (see git log) |
| `2db3138` | (see git log) |
| `8f22d89` | (see git log) |

### Status

[OK] **Completed**


## Session 9: 完成 S0 真实夹具与回归

**Date**: 2026-08-24
**Task**: 完成 S0 真实夹具与回归
**Branch**: `main`

### Summary

实现八方案可复现 fixture 获取与离线校验，补齐真实、属性和跨 codec 回归，实测行覆盖率 90.12%，完成独立检查与 backend 规范同步。

### Git Commits

| Hash | Message |
|------|---------|
| `7cc09c6` | (see git log) |
| `b9db57c` | (see git log) |
| `d5b7cf3` | (see git log) |

### Status

[OK] **Completed**
