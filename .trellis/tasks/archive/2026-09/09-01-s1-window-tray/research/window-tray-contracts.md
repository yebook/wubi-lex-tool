# S1 窗口与托盘技术契约

## 研究范围

本文只收敛 `s1-window-tray` 的必要实现：无边框主窗口、正常态 bounds 恢复与保存、关闭策略、最小托盘菜单、`/tray` 延迟启动、第二实例唤起、生成 IPC、可见失败和对应测试。完整动作目录、路由菜单、领域状态、全局快捷键、设置页面、主动工作集清理、ImTip 和根目录 `resource/` 均不在研究范围。

## 仓库基线

- `src-tauri/src/lib.rs` 当前在 Rust `setup` 中创建唯一 `main` WebView，尺寸为 `960x680`、最小 `720x520`，并直接实现第二实例的 `unminimize -> show -> focus`。
- `src-tauri/src/runtime/mod.rs` 已提供窗口就绪前的 activation queue 和最多 8 条 runtime notice；新 coordinator 必须复用这个早到请求保护，不能建立第二套启动队列。
- `src-tauri/src/config/model.rs` 已冻结 `WindowConfig { bounds, maximized, closeAction }`。`WindowBounds` 是逻辑矩形并带采样时 scale factor；默认关闭策略是 `minimizeToTray`。
- `src-tauri/src/config/mod.rs` 的事务保存会串行化、校验、备份、原子替换并递增 revision。窗口高频事件必须调用一个只更新 placement 的内部方法，不能用陈旧的完整 `WindowConfig` 覆盖并发变更的 `closeAction`。
- `src-tauri/src/bindings/mod.rs` 是 command/event/type 的唯一 registry；前端只消费生成 bindings 和 `src/lib/` 的薄 wrapper。
- `scripts/smoke-runtime.ps1` 当前假定关闭主窗口会退出。默认 close-to-tray 生效后该假设失效，必须在本任务中调整。

## Tauri 2.11.5 API 结论

研究使用本机锁定源码 `E:/env/rust/cargo/registry/src/rsproxy.cn-e3de039b2554c837/tauri-2.11.5`，没有选择新插件。

### 托盘

- `tauri` 的 `tray-icon` feature 直接提供 `TrayIconBuilder`、`Menu`、`MenuItem`、`TrayIconEvent`、`tray_by_id` 和 `remove_tray_by_id`。
- `TrayIconBuilder::with_id` 可固定归属 ID；`show_menu_on_left_click(false)` 可把左键释放留给 restore，同时保留右键原生菜单。
- `TrayIconBuilder::build` 会把图标注册到 app manager；`tray_by_id` 是重复创建前的权威检查，退出只按固定 ID 移除本进程拥有的图标。
- 左键恢复匹配 `TrayIconEvent::Click { button: Left, button_state: Up, .. }`。菜单只使用固定 item ID `tray.show` 与 `tray.exit`。
- 图标复用 `app.default_window_icon()`，tooltip 固定为 `WubiLex`。普通可见启动不调用 builder。

### 窗口与显示器

- `WindowEvent` 提供物理像素的 `Moved`、`Resized`、`ScaleFactorChanged` 和可 `prevent_close` 的 `CloseRequested`。
- `Monitor::work_area()` 返回当前显示器物理工作区，且坐标允许为负；`scale_factor()` 用于逻辑/物理转换。
- `WebviewWindow` 提供 `is_minimized`、`is_maximized`、`set_position(PhysicalPosition)`、`set_size(PhysicalSize)`、`set_skip_taskbar`、`unminimize`、`show` 和 `set_focus`。
- 为避免恢复位置闪动，窗口统一以不可见状态构建，先应用校正后的物理 placement 和最大化状态，再决定显示或保持 `/tray` 隐藏。

### 官方拖动区与 capability

- Tauri 注入的 `data-tauri-drag-region` 脚本会自动排除 `button` 等可点击元素；`deep` 值允许品牌子元素参与拖动而不会吞掉旁边按钮。
- 单击拖动区调用 `plugin:window|start_dragging`，双击调用 `plugin:window|internal_toggle_maximize`。
- 当前 capability 没有启用 `core:window:default`，所以必须显式增加且只增加：
  - `core:window:allow-start-dragging`
  - `core:window:allow-internal-toggle-maximize`
- 最小化、最大化按钮和关闭按钮走自有生成 command，不开放 `allow-minimize`、`allow-toggle-maximize`、`allow-close`、`allow-hide`、`allow-show` 或 `allow-set-skip-taskbar`。

## Bounds 保存与恢复算法

### 保存

1. 只在窗口既非最小化也非最大化时采样 `outer_position`、`inner_size` 和当前 scale factor。
2. 用采样 scale 把物理位置和尺寸除回逻辑值，有限检查后四舍五入到 `WindowBounds` 的整数范围。
3. 最大化事件只更新 `maximized=true`，沿用最后一个正常态 bounds；最小化/隐藏事件不改 bounds，也不把 `0x0` 或最大化矩形写入配置。
4. 从最大化恢复到正常态后，下一次 Moved/Resized 重新采样正常 bounds 并保存 `maximized=false`。

### 恢复

恢复算法在纯 Rust 矩形类型上运行，所有边界和面积计算使用 `i64`，最后才做有界转换。

1. 用 `savedLogical * savedScale` 重建保存时的物理矩形。
2. 在当前 monitor work areas 中选择与该矩形正交面积最大的工作区；相交面积相同时优先 primary，再按稳定输入顺序。
3. 若与所有工作区都无正面积交集，选择 primary；没有 primary 时选择第一个可用工作区。没有任何 monitor 或 monitor 查询失败时，回退到 Tauri 居中默认值并产生可见 notice。
4. 目标物理尺寸用保存的逻辑宽高乘目标 monitor 当前 scale，先提升到 `1024x640` 逻辑最小值。工作区能容纳最小值时再把过大尺寸限到工作区。
5. 有交集时保留重建后的物理左上角并限幅；无交集时在目标工作区居中。负坐标按同一公式处理。
6. 若目标工作区本身小于 `1024x640` 逻辑最小值，不违反 native minimum：保留最小尺寸并把左上角钉在工作区起点，保证完整标题栏和可操作区域可见，允许右侧/底部超出这个物理上不可能同时满足的工作区。

该算法不保存 monitor 名称或设备 ID，避免把易变硬件标识写入 schema；显示器拔插时由交集/primary 回退恢复。

## 生命周期与竞态

Rust `WindowCoordinator` 是原生状态转换的唯一所有者。内部纯状态至少区分 `visible`、`hidden`、`exiting`，并记录固定 tray 是否存在、延迟 token、最后正常 placement 和单调 window revision。

| 触发 | 必要效果 | 失败处理 |
|---|---|---|
| 标题栏最小化、任务栏原生最小化、默认 CloseRequested | 先确保唯一 tray，再隐藏并跳过任务栏 | tray 创建失败时保持/恢复窗口可见；其他失败尽量完成剩余动作并发 notice |
| `closeAction=exit` | 阻止默认销毁，进入显式退出 | 保存/清理失败不撤销退出意图 |
| 托盘 `tray.exit` | 无条件绕过 closeAction，进入显式退出 | 同上 |
| 托盘左键、`tray.show`、第二实例 | 取消延迟，`skip_taskbar(false) -> unminimize -> show -> focus` | 逐项尝试，失败入有界 notice；不得创建第二窗口 |
| `/tray` primary | 从构建首帧保持隐藏/skip-taskbar，启动 3 秒延迟 | 延迟 tray 创建失败时 fail-open 恢复主窗口 |
| 第二实例在 3 秒内到达 | 原子失效延迟 token 并通知等待线程，立即 restore | 超时边界已先成功建 tray 时允许保留该唯一 tray |
| 重复 hide/restore/tray event | 返回当前稳定状态或重试未完成 native 效果 | 不增加 tray、窗口或后台 worker |

### 可取消延迟

- 使用标准库 channel 的 `recv_timeout(3s)`，不新增 Tokio 或 timer 依赖。
- coordinator 保存 cancel sender 与 generation token。restore/exit 先使 token 失效，再发送取消信号。
- 超时线程回到 coordinator 后必须再次原子校验 token、`hidden`、非 `exiting` 和 tray absent，不能只依赖“已经睡满 3 秒”。
- 该双重检查处理“第二实例与超时同时到达”的竞态；最多出现一个已被 manager 注册的 tray。

### 显式退出

- 第一次 exit intent 原子切换到 `exiting`；后续退出请求幂等返回。
- 配置 flush、延迟取消和 owned tray 移除在后台准备路径执行，完成或达到有界等待后调用 `app.exit(0)`；失败只记录 notice/log，不把状态切回 visible/hidden。
- `RunEvent::Exit` 继续负责现有 session marker 清理；清理 owned tray 仍做幂等兜底。

## 高频保存与退出 flush

- 一个应用拥有的标准库 worker 线程接收 placement，不在窗口事件/UI 线程执行配置事务。
- trailing quiet period 为 250ms；连续拖动的最大合并窗口为 2s。到任一边界就提交最新 placement，避免 I/O 风暴，也避免持续拖动永远不保存。
- worker 调用 `ConfigService::update_window_placement(bounds, maximized)`，在 service 锁内读取最新配置并只替换 placement 字段，保留当前 `closeAction`。
- 成功继续发完整 `config://changed` snapshot；失败保留原生窗口状态，发送脱敏 runtime notice。
- exit 发 `FlushAndStop` 并最多等待 1s；无论成功、失败或超时都继续退出。进程退出不是无限等待磁盘的恢复机制。

## 生成 IPC 与可见状态

最小共享合同为：

```text
WindowControlIntent = minimizeToTray | toggleMaximize | close
WindowVisibility = visible | hidden | exiting
WindowStateSnapshot = { revision, visibility, maximized }

window_state() -> WindowStateSnapshot
window_control(intent) -> Result<WindowStateSnapshot, AppError>
window://state-changed -> WindowStateChangedEvent { snapshot }
app://runtime-notice -> RuntimeNoticeEvent { notice }
```

- revision 是 JavaScript-safe number，用于前端在“先 listen、后 snapshot”时拒绝旧状态。
- `window_control(close)` 与 native `CloseRequested` 调用同一 coordinator 分支，不通过 `window.close()` 形成递归关闭事件。
- 后台 tray/persistence/native 失败先写入权威 `RuntimeState`，再发 `RuntimeNoticeEvent`；事件丢失时 `app_runtime_snapshot` 仍可重建。
- `AppErrorCode` 只增加必要的 `windowUnavailable` 与 `windowOperationFailed`，沿用现有 `AppError` 形状、M7 归属、中文 message 和 1,024 字符 detail 上限。

## 前端边界与依赖

- 新增唯一 runtime dependency：`lucide-react@1.38.0`，由 `src/icons/` 窄出口提供 `Minus`、`Square`、`Copy`、`X`。
- 为真实交互测试新增 dev dependencies：`@testing-library/react@16.3.3`、`@testing-library/user-event@14.6.6`、`jsdom@30.0.1`。
- `src/lib/window-client.ts` 包装生成 commands/events；Hook 负责 listener-first bootstrap、revision 合并、命令失败转可见 warning；标题栏组件只渲染状态并提交 intent。
- 标题栏使用原生 `button`、44x44 固定命中区、中文 `aria-label`、`title` tooltip、可见 focus 和 Lucide 图标。restore 使用重叠方框图标，图标状态跟随生成 snapshot。
- 品牌拖动区与窗口按钮是同级区域；按钮不带 drag attribute。官方脚本也会把 button 判定为 clickable barrier。
- 本任务只调整现有 runtime surface，不建立 app bar、sidebar、route、theme provider 或设置页。

## 验证边界

### Rust 自动化

- 纯矩形：单屏、多屏、负坐标、DPI 改变、移除显示器、完全离屏、过大/过小、无 monitor、极值溢出保护。
- 生命周期：默认关闭、exit 关闭、显式 tray exit、重复 intent、隐藏前 tray 失败、恢复部分失败、延迟取消、超时竞态、一次性 tray。
- 保存 worker：250ms quiet、2s max、最新值覆盖、保留 closeAction、失败 notice、flush 成功/失败/超时不阻断退出。
- 序列化：window intent/state/event、runtime notice 与新增 AppError code 通过 canonical bindings。

### 前端自动化

- listener-first snapshot merge、revision 拒旧、命令错误、runtime notice 合并。
- 三个窗口按钮的 accessible name、tooltip、键盘 Enter/Space、最大化/还原图标、44x44 class/样式契约和 drag region 隔离。
- 现有 runtime loading/error/notice 流程不回归。

### Windows smoke

- 自动部分：普通启动无 tray 前置证据、`/tray` 无闪窗、3 秒前第二实例取消延迟并恢复、CloseMainWindow 隐藏且进程存活、再次第二实例恢复、只有本次进程/marker 被清理。
- 可见部分：标题栏拖动/双击/三个按钮、任务栏隐藏/恢复、tray 左键、右键仅两项、唯一 tray、显式退出后进程/session marker/tray 清理、多显示器与 DPI 恢复。
- 自动化不得枚举后删除任意同名历史 marker、其它 WubiLex 进程或系统托盘资源；无法按 PID 安全归属的 tray UI 检查保持可见 smoke，不加入脆弱的全局删除脚本。

## 明确不选

- 不使用 tray plugin、store plugin、window-state plugin 或直接 Win32 托盘实现。
- 不新增 monitor ID、tray menu descriptor、action projection、route ID 或 disabled placeholder。
- 不新增 Tokio direct dependency；标准库 channel 足以完成一个延迟和一个保存 worker。
- 不调用 `EmptyWorkingSet`，不做系统写入，不读取根目录 `resource/`，不出现 ImTip。
