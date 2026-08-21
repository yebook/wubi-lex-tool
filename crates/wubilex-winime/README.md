# wubilex-winime

> Windows 输入法子系统集成。**全项目唯一**允许直接调用 Win32 / COM 的业务 crate。
> 也是**唯一可能让用户系统进入不可用状态**的 crate —— 风险 `R1` 就在这里。

## 职责

| 目录 | 内容 | 需求 / 风险 |
|---|---|---|
| `src/sysops/` | `SystemOps` trait + `Win32SystemOps` + `RecordingSystemOps` | 见下 |
| `src/tip/` | TIP 启停与状态查询 | `M4-TIP-*` · `R24` |
| `src/double_pinyin/` | 双拼方案枚举与切换（方案串作为**配置数据**管理，不硬编码） | `M4-DPY-*` · `R25` |
| `src/settings/` | 注册表设置读写 | `M4-REG-*` · **契约 C3** · `R26` |
| `src/tsf/` | 停机窗口编排 + **RAII 恢复守卫** | `M4-TSF-*` · **契约 C4** · **`R1`** |
| `src/service/` | 服务控制（Win32 SCM API） | `M4-TSF-002` · `R8` |
| `src/schtask/` | 计划任务（Task Scheduler COM） | `M4-TSF-003` |
| `src/acl/` | 文件所有权与 ACL 接管（`SetNamedSecurityInfo`） | `M4-TSF-007` · `R9` |

## 允许依赖

`windows` crate（**按需启用 feature**，全量启用会显著拖慢编译）。

需要的 feature 至少包括：`Win32_UI_TextServices`、`Win32_System_Services`、`Win32_Security_Authorization`、`Win32_System_Registry`、`Win32_System_Diagnostics_ToolHelp`。

## 禁止依赖

Tauri、业务逻辑（本 crate 不理解码表内容，只负责「安全地替换文件并重启输入法」）。

**禁止调用命令行工具**：不使用 `takeown` / `icacls` / `schtasks` 并解析其文本输出 —— 那会在非中/英文系统上失效（`R10`），且错误码不明确。一律走 Win32 / COM API。

## `SystemOps` 是这个 crate 的中枢

所有有副作用的系统调用（停/起服务、结束进程、改所有权与 ACL、写注册表、启停 TIP）收拢到 `src/sysops/` 的一个 trait，两个实现：

| 实现 | 行为 |
|---|---|
| `Win32SystemOps` | 真实调用 |
| `RecordingSystemOps` | 不改系统，把调用序列记进 `Vec<Op>` |

**编排逻辑（停机窗口、恢复守卫）只写一次**，泛型于 `SystemOps`。

不要写 `if dry_run { … } else { … }`：那样 dry-run 路径与真实路径会各自演化，测试就失去意义。

**这是 `R1` 唯一能在 CI 里验证的方式** —— 对 `RecordingSystemOps` 断言操作序列，特别是**恐慌路径下守卫是否产生了完整的恢复序列**。

## 对应需求域

`M4-TIP` · `M4-DPY` · `M4-REG` · `M4-TSF` · `M4-SYS`

## 所属阶段

**S3 — 写入闭环**（全项目最高风险阶段）。

**S0 期间需并行完成技术预研**（见 `docs/22-roadmap.md` §5）：在真机上验证服务控制、计划任务、ACL 接管、TIP 启停四项底层能力可行，再进入 S3。别等做到一半才发现方案不可行。

## 不可协商的三条

1. **恢复守卫在 `Drop` 时无条件执行恢复**，包括恐慌路径。`TabletInputService` 必须恢复（`R8`），文件所有权必须归还 TrustedInstaller（`R9`）。
2. **进入停机窗口前落持久化标记**，启动时检测到残留标记即自恢复（`M7-INST-006`）。
3. **任何 `unwrap()` / `expect()` 在本 crate 的生产路径上都是缺陷**。系统 API 失败必须携带 `GetLastError()` 的错误码与文本。
