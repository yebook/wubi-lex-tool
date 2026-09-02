# Design - S1 窗口与托盘

## 1. 交付边界

本任务在现有 runtime 与配置服务上增加一个 Rust 原生窗口协调边界和一个紧凑 React 标题栏。MVP 只解决“窗口不会丢、状态能恢复、托盘能显示/退出、失败可见”四件事。

托盘右键菜单固定为：

1. `显示 WubiLex`
2. `退出`

本任务不为后续领域菜单建立 descriptor、placeholder、route/action adapter 或临时扩展接口。统一动作目录和路由交付后再重构菜单投影，避免当前产生第二事实源。

## 2. 架构与所有权

```text
React WindowTitleBar
  -> src/lib/window-client.ts
  -> generated window_state/window_control + events
  -> commands/app/window.rs (thin adapter)
  -> WindowCoordinator
       |- LifecycleState          pure intent/race decisions
       |- bounds                  pure monitor correction
       |- PlacementWorker         coalesced config persistence
       |- tray                    fixed owned tray + two menu items
       `- Tauri main window       native effects

secondary-instance callback ------^ restore
WindowEvent callback -------------^ close/move/resize/scale/minimize
RunEvent::Exit -------------------^ final owned cleanup
```

所有 native transition 由 `src-tauri/src/window/` 所有。React 不直接调用 `@tauri-apps/api/window` 的 minimize/maximize/close/hide/show API；唯一 Tauri core 直通行为是官方 `data-tauri-drag-region` 脚本的拖动和双击。

建议文件归属：

- `src-tauri/src/window/mod.rs`：coordinator、窗口事件入口、typed state。
- `src-tauri/src/window/bounds.rs`：逻辑/物理矩形、monitor 选择和限幅纯函数。
- `src-tauri/src/window/persistence.rs`：250ms/2s 合并 worker 与 1s exit flush。
- `src-tauri/src/window/tray.rs`：固定 tray/menu ID、创建、事件过滤和归属清理。
- `src-tauri/src/commands/app/window.rs`：`window_state`、`window_control` 薄适配。
- `src/lib/window-client.ts`：生成 IPC 的单一前端 wrapper。
- `src/hooks/use-window-controls.ts`：listener-first 状态同步和错误投影。
- `src/components/window-title-bar/`：可访问标题栏及组件测试。
- `src/icons/window-controls.ts`：Lucide 窄出口。

## 3. 共享合同

Rust 定义并通过 canonical registry 生成：

```rust
pub enum WindowControlIntent {
    MinimizeToTray,
    ToggleMaximize,
    Close,
}

pub enum WindowVisibility {
    Visible,
    Hidden,
    Exiting,
}

pub struct WindowStateSnapshot {
    pub revision: u64, // specta number
    pub visibility: WindowVisibility,
    pub maximized: bool,
}

pub struct WindowStateChangedEvent {
    pub snapshot: WindowStateSnapshot,
}

pub struct RuntimeNoticeEvent {
    pub notice: RuntimeNotice,
}
```

Command：

- `window_state() -> WindowStateSnapshot`
- `window_control(intent) -> Result<WindowStateSnapshot, AppError>`

Event：

- `window://state-changed`
- `app://runtime-notice`

`WindowStateSnapshot.revision` 在 coordinator 每次公开状态变化时单调增加。前端先注册两个 listener，再读取 snapshot；event 与 snapshot 冲突时只接受较高 revision，不能让慢 snapshot 覆盖刚发生的最大化状态。

## 4. 启动顺序

1. 解析 primary launch，保留现有 `RuntimeState` activation queue。
2. 加载 `ConfigService`，读取 `WindowConfig`；读取失败时使用默认窗口配置并加入脱敏 notice，不阻止启动。
3. 构建不可见、不可聚焦、无边框的唯一 `main` 窗口：默认 `1024x680`，minimum `1024x640`，`/tray` 时从构建起 skip taskbar。
4. 查询 monitor work areas。存在 bounds 时运行纯校正后设置 physical position/size；无 bounds 时居中。恢复 `maximized`，但不把最大化矩形当 normal bounds。
5. 创建并 manage coordinator，挂接 window events，再标记 runtime window ready。
6. 若 setup 期间已有 secondary activation，立即走 coordinator restore；否则普通启动 show/focus，`/tray` 保持隐藏并安排 3 秒 delay。
7. 普通启动绝不调用 tray builder。

窗口始终先隐藏构建，因此校正、最大化和 `/tray` 都不会暴露错误首帧。

## 5. 生命周期状态机

### 公开状态

- `Visible`：窗口应显示并出现在任务栏。
- `Hidden`：窗口应隐藏并跳过任务栏；除 `/tray` 的 3 秒兼容窗口外，必须已有唯一 tray。
- `Exiting`：退出意图已锁定；任何 hide/restore/control 都不能撤销退出。

### Hide/MinimizeToTray

标题栏最小化、原生最小化和默认关闭都进入同一 hide 分支：

1. 幂等检查 `Exiting` / `Hidden`。
2. `ensure_tray`，先用 `tray_by_id` 检查固定 ID，再构建两项菜单。
3. tray 创建失败则 fail-open：unminimize/show/focus，保持入口可见并报告 notice。
4. tray 可用后执行 hide 与 `set_skip_taskbar(true)`；逐项记录失败，不删除已建立的 tray。
5. 发布 `Hidden` snapshot。

原生 `CloseRequested` 总是先 `prevent_close`，再读取最新 config：

- `minimizeToTray`：调用 hide。
- `exit`：调用显式退出。
- 配置读取失败：保守选择 hide 并报告 notice，用户仍可从 tray 显式退出。

### Restore

tray 左键、`tray.show`、第二实例和以后内部消费者都调用同一 restore：

1. 取消并失效 delayed tray token。
2. 依次尝试 `set_skip_taskbar(false)`、`unminimize`、`show`、`set_focus`，不因前一步失败跳过后续恢复机会。
3. 至少一个 native 失败时记录 `WindowOperationFailed` notice 和脱敏 stage；activation request 不创建新窗口。
4. tray 已存在时保持到进程退出；restore 不删除它。
5. 查询真实 maximize 状态，发布 `Visible` snapshot。

### 显式退出

`closeAction=exit` 与 `tray.exit` 共用 exit intent，后者不读取 closeAction。第一次请求原子进入 `Exiting` 并启动一次后台准备：取消 delayed tray、请求 placement `FlushAndStop`、按固定 ID 移除 tray，然后调用 `app.exit(0)`。flush 最多等待 1 秒，任何失败或超时只进入 notice/log，不阻断退出。

`RunEvent::Exit` 保持现有 session marker clean-exit，并对 tray 移除做幂等兜底。不得删除其它进程或历史资源。

## 6. `/tray` 延迟与第二实例竞态

延迟由一次性标准库等待线程实现：`recv_timeout(Duration::from_secs(3))`。coordinator 保存 cancel sender 与 generation token。

- restore/exit 在状态锁内先使 token 失效，再通知 channel。
- timeout 后必须回 coordinator 再次检查：token 仍匹配、状态仍 Hidden、未 Exiting、tray 仍 absent。
- 检查成立才创建 tray；创建失败立即 fail-open restore。
- 第二实例在 timeout 前取到状态锁时，延迟不会补建 tray。
- timeout 已先成功注册 tray 的边界竞态中，第二实例仍 restore，且该唯一 tray 按“首次创建后保留”规则继续存在。

不增加 Tokio direct dependency，也不使用不可取消的 `sleep(3s)` 后无条件建 tray。

## 7. Bounds 与 DPI

`WindowBounds` 继续表示逻辑 outer position + inner size，并记录采样 scale。保存时从 Tauri 物理值除回逻辑值；恢复时先用 saved scale 重建旧物理矩形，再投影到当前 monitor。

目标 monitor 选择和校正遵循 `research/window-tray-contracts.md`：最大相交工作区，完全离屏则 primary/first 居中；目标尺寸按当前 scale 计算；正常工作区内限幅；工作区物理上小于 native minimum 时保留 `1024x640` 最小值并固定左上角，保证标题栏可达。

所有几何计算使用纯类型和 `i64` 中间值，覆盖负坐标、乘法/加法溢出和浮点有限性。monitor 查询失败只回退居中并告警，不使窗口创建失败。

## 8. Placement Persistence

窗口事件回调只做轻量采样与发送：

- minimized：触发 hide，不保存瞬时矩形。
- maximized：沿用 last-normal bounds，只排队 `maximized=true`。
- normal Moved/Resized/ScaleFactorChanged：采样完整 normal bounds，更新 last-normal，排队 `maximized=false`。

worker 使用两个时间界限：最后事件安静 250ms 或第一条 pending 后 2s，任一到达即保存最新 placement。配置服务新增内部 `update_window_placement`，在同一事务锁内只改 bounds/maximized，保留最新 closeAction；成功继续 emit 完整 `config://changed`，失败保留 native state 并 emit runtime notice。

一个应用进程只建一个 placement worker。channel 断开、重复 flush 和退出准备均幂等。

## 9. Tray 设计

固定 ID：

- tray：`wubilex-main-tray`
- menu：`tray.show`、`tray.exit`

构建内容严格只有 `显示 WubiLex` 与 `退出` 两项。左键释放 restore，右键由 Tauri 原生菜单处理；`show_menu_on_left_click(false)` 防止左键同时打开菜单。

普通启动 tray absent；首次 hide 或 `/tray` delay 成功后 present；restore 不移除；exit 按固定 ID 移除。不存在 plugin capability、前端 tray API、动态领域项、disabled item、码表 tooltip 或动作/路由判断。

## 10. 前端标题栏

`WindowTitleBar` 取代当前 runtime header 的品牌区域，不新增第二层 app bar。布局是同级的拖动品牌区与控制区：

```text
[icon WubiLex v0.1.0 ........ drag region][minimize][maximize/restore][close]
```

- 控制使用 `button type="button"`，DOM 顺序与视觉顺序一致。
- icon 分别使用 Lucide `Minus`、`Square`/`Copy`、`X`；没有自绘 SVG、emoji 或文本方块。
- 每个按钮有中文 `aria-label` 和 `title`；focus-visible 清晰，命中区稳定为 44x44。
- `data-tauri-drag-region="deep"` 只放在品牌区。控制区和按钮显式不属于拖动区。
- 双击品牌区交给 Tauri internal toggle；window event 发布的新 snapshot 切换 maximize/restore icon。
- Hook 中 command 失败转换为现有“可见警告”区的持久 warning；不静默吞错。
- 使用 CSS 自适应高度和文本截断/换行约束，保证 200% 系统字号下品牌与按钮不重叠。

组件不负责 theme、route、feature、tray 或 config；后续 shell 可复用该窗口控制边界。

## 11. 失败与可见性

扩展现有统一错误，不创建窗口专用 payload：

- `AppErrorCode::WindowUnavailable`
- `AppErrorCode::WindowOperationFailed`
- `RuntimeNoticeCode::WindowOperationFailed`
- `RuntimeNoticeCode::WindowPersistenceFailed`
- `RuntimeNoticeCode::TrayUnavailable`

Command 失败同时返回 `AppError` 并写入 runtime notice。后台 persistence/tray 失败没有 caller，因此写入权威 `RuntimeState` 后 emit `app://runtime-notice`。detail 只含固定 stage 和系统错误显示，不含 argv、配置内容或用户数据；队列保持 8 条上限并避免相同 notice 风暴。

Event emit 失败只记录结构化 warning，不能把已经成功的 native/config 操作伪报为失败。

## 12. Manifest、Capability 与依赖

- `src-tauri/Cargo.toml` 的 `desktop` feature 增加 `tauri/tray-icon`；不添加 tray plugin 或 Tokio direct dependency。
- `src-tauri/capabilities/main.json` 保留 event listen/unlisten，只增加两个官方 drag-region 权限，不开放其它 window/menu/tray command。
- `package.json` 精确增加 `lucide-react@1.38.0`。
- 组件交互测试精确增加 `@testing-library/react@16.3.3`、`@testing-library/user-event@14.6.6`、`jsdom@30.0.1`。
- Node/pnpm 继续只读取 `package.json.volta`；实施命令先设置 `VOLTA_FEATURE_PNPM=1`，不添加 `packageManager`。

## 13. 测试、兼容与回滚

Rust 纯测试覆盖 bounds、state transition、delay generation 和 persistence scheduling；MockRuntime/serialization 测试覆盖注册与 wire contract；前端组件测试覆盖真实键盘与 accessible queries。

Windows smoke 分两层：脚本自动验证安全可归属的进程、窗口可见性、第二实例、close-to-tray 和 marker；可见 smoke 验证系统托盘 UI、任务栏、拖动/双击、DPI/多屏与显式 tray exit。不能安全按 PID 归属的系统托盘节点不使用全局删除或脆弱 Toolbar hack 自动化。

回滚按三个边界组织：

1. Rust coordinator/bounds/tray/config worker 与 bindings。
2. React 标题栏、Lucide/Test dependencies 与样式。
3. Windows smoke 和规划/spec 记录。

若 tray 建立失败，运行时回滚形态是显示主窗口；若配置保存失败，保留当前 native placement；若事件失败，以 snapshot command 重建。任何回滚都不放宽 capability、不加入 plugin，也不扩大到 S2/S3、ImTip 或 `resource/`。
