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
