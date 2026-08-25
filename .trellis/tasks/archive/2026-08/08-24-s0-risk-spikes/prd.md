# S0 技术预研

## Goal

在进入 S1 前，用最小、可重复、可恢复的原型验证四项尚未落地的底层方案：TSF Profile 控制、TrustedInstaller ACL 所有权往返、Task Scheduler COM 控制 `MsCtfMonitor`，以及数十万行虚拟滚动性能。任何失败都必须形成明确证据并触发架构复评，不能以未验证假设进入后续阶段。

## Background

- `docs/22-roadmap.md` 将四项预研定义为 S1 硬入口门槛；父任务 `s0-foundation` 要求每项都有环境、步骤、结果、判定和清理说明。
- Windows 原型归 `wubilex-winime` 边界，必须使用 `windows` crate 的 Win32/COM API，不得依赖 `takeown`、`icacls`、`schtasks` 等本地化文本输出。
- 虚拟滚动预研验证 TanStack Virtual 或等价成熟方案的最小性能，不实现 S1/S2 产品页面。
- 用户已批准在实现阶段做短暂、可恢复的 live 验证，范围严格限于 TSF Profile 切换/恢复、`MsCtfMonitor` End/Run/恢复和任务自建临时文件的 ACL 所有权往返；默认命令与自动测试仍须只读或 dry-run。
- 当前只读预检确认开发机为 Windows 11 Pro 64-bit build 26200，当前进程未提权，`MsCtfMonitor` 为 Ready、`ctfmon.exe` 正在运行，用户语言列表含三个中文输入法 TIP，系统安装了 Edge；Rust 1.97.1、Node 24.18.1、全局 pnpm 11.18.0 与仓库基线一致。

## Requirements

| ID | Requirement |
|---|---|
| SPIKE-R01 | 四个原型必须彼此隔离、可单独运行和判定；共享代码仅限确有复用价值的 Win32 错误、COM 初始化或证据记录辅助，不提前实现完整 `SystemOps` 产品层。 |
| SPIKE-R02 | TSF 原型使用 `ITfInputProcessorProfileMgr` 对微软五笔 Profile 做真实 ACTIVE/current-profile 切换验证，不改 ENABLED 配置，记录调用前后系统状态，并在成功、失败和异常路径恢复原始活动语言/TIP 状态。 |
| SPIKE-R03 | ACL 原型只操作任务自建的临时文件，通过 Win32 Security API 完成 TrustedInstaller -> Administrators -> TrustedInstaller 所有权往返，并验证最终 owner/DACL 与基线语义一致；不得接触系统码表或用户文件。 |
| SPIKE-R04 | Task Scheduler 原型只通过 Task Scheduler 2.0 COM 对 `\Microsoft\Windows\TextServicesFramework\MsCtfMonitor` 执行 End/Run，记录任务调用、实例/状态和 `ctfmon.exe` presence/PID 时间线（包括单例 PID 未变化的结果），并无条件恢复进入原型前的逻辑状态。 |
| SPIKE-R05 | Windows 原型默认只做预检；真实状态变更必须使用显式 live 开关、管理员权限检查、前置快照和恢复守卫。任何恢复失败都必须高可见地失败并给出人工恢复证据。 |
| SPIKE-R06 | 虚拟滚动原型使用合成的至少 300,000 行数据、只物化视口及 overscan 行，不在前端持有完整词条对象数组；在本机 Edge 中按固定脚本和采样窗口测量滚动帧率。 |
| SPIKE-R07 | 虚拟滚动通过标准为连续滚动有效采样不低于 55 fps；报告同时记录测试数据量、可见 DOM 行数、overscan、预热/采样时长、三次结果和内存观察，禁止只凭肉眼判定。 |
| SPIKE-R08 | 每项原型输出可重复命令、环境、前置条件、原始测量/状态、恢复结果、通过或失败结论及限制；结果存入任务 research，不把机器专属值硬编码进产品配置。 |
| SPIKE-R09 | 任一项失败时，必须在本任务内记录阻塞原因和候选替代方案，并在创建 S1 任务前更新架构决策或风险处置；不得降低路线图判定标准后宣称通过。 |
| SPIKE-R10 | 本任务不读取根目录 `resource/`，不替换真实码表/短语文件，不停止 `TabletInputService`，不终止 `ChsIME.exe`，不实现完整停机窗口、产品 UI、发布或签名。 |

## Acceptance Criteria

- [ ] TSF Profile 原型在显式 live 模式下产生可审计的 ACTIVE/current-profile 状态变化，保持原有 ENABLED 配置不变，并验证原始活动输入法/TIP 已恢复。
- [ ] ACL 原型仅对任务临时文件完成 TrustedInstaller -> Administrators -> TrustedInstaller 往返，最终 owner/DACL 与基线语义一致且临时目标可清理。
- [ ] Task Scheduler COM 原型能 End/Run `MsCtfMonitor`，观察到预期任务/`ctfmon.exe` 行为，并恢复进入前状态。
- [ ] 300,000 行虚拟滚动原型在 Edge 中按固定脚本运行三次，所有有效采样均达到不低于 55 fps，且 DOM 行数保持视口级有界。
- [ ] 四份结果都包含环境、命令、原始证据、判定和清理/恢复说明；任何失败项附带架构复评结论。
- [ ] Windows API 实现不调用或解析 `takeown`、`icacls`、`schtasks`，生产路径无 `unwrap()`/`expect()`，错误包含 HRESULT/Win32 代码与阶段。
- [ ] 默认测试和 CI 不改变系统状态；真实 live 验证只能显式执行，结束后独立只读检查确认任务、进程、TIP 和临时 ACL 无残留。
- [ ] Rust/前端现有质量门、bindings/docs、Trellis 校验和 `git diff --check` 全部通过，锁文件变化经过审查。

## Out Of Scope

- 完整 `SystemOps` trait、TSF 停机窗口、服务控制、进程终止、持久化崩溃恢复和系统码表写入。
- S1 应用外壳以及 S2 的正式虚拟表格、分页 command、排序、筛选、编辑和产品视觉设计。
- 以命令行工具作为 Win32/COM API 的后备实现，或为了通过预研而解析本地化文本输出。
- 对根目录 `resource/`、真实码表、真实短语库或其他用户数据做读写验证。
