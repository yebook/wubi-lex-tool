# Implementation Plan - S1 窗口与托盘

## Entry Gate

- [ ] 用户审阅本任务最终规划摘要，并在后续消息中明确批准实施。
- [ ] 获批后运行 `python ./.trellis/scripts/task.py start .trellis/tasks/09-01-s1-window-tray`；未 start 前不修改产品代码。
- [ ] 实施前加载 `trellis-before-dev`，读取 backend/frontend index、目录、错误、日志、质量、组件、Hook、类型和工具链规范。
- [ ] 检查工作树，只保护并延续本任务规划改动；不覆盖用户并行改动。
- [ ] 设置 `VOLTA_FEATURE_PNPM=1`，核对 `pnpm --version` 等于 `package.json.volta.pnpm`；不使用 packageManager、Corepack、npm、yarn、npx 或全局 pnpm。
- [ ] 不读取或修改根目录 `resource/`，全库不增加 ImTip surface。

## 1. 冻结依赖与安全面

- [ ] 在 `src-tauri/Cargo.toml` 的 `desktop` feature 增加 `tauri/tray-icon`，不增加 tray plugin、window-state plugin、store plugin 或 Tokio direct dependency。
- [ ] 用项目 pnpm 精确增加 `lucide-react@1.38.0`。
- [ ] 精确增加 dev dependencies：`@testing-library/react@16.3.3`、`@testing-library/user-event@14.6.6`、`jsdom@30.0.1`，只用于真实标题栏交互测试。
- [ ] capability 只增加 `core:window:allow-start-dragging` 和 `core:window:allow-internal-toggle-maximize`；确认没有其它 window/menu/tray 前端写权限。
- [ ] 检查 lockfile 只有上述必要依赖变化，没有 package-manager metadata 漂移。

建议命令：

```powershell
$env:VOLTA_FEATURE_PNPM = '1'
pnpm add --save-exact lucide-react@1.38.0
pnpm add --save-dev --save-exact @testing-library/react@16.3.3 @testing-library/user-event@14.6.6 jsdom@30.0.1
```

验证点：`cargo metadata --locked` 能解析新 feature；capability schema 校验通过；`package.json` 仍只有 `volta.node`/`volta.pnpm` 版本来源。

## 2. 实现纯 bounds 校正

- [ ] 新建 `src-tauri/src/window/bounds.rs`，定义与 Tauri 解耦的物理矩形、work area 和校正结果。
- [ ] 实现保存转换：物理 outer position/inner size 按采样 scale 转为有限、有界 logical `WindowBounds`。
- [ ] 实现恢复投影：saved logical + saved scale 重建旧物理矩形，按最大相交面积选择当前 work area，完全离屏时 primary/first 居中。
- [ ] 按目标 monitor scale 恢复尺寸，执行 `1024x640` 最小值、可容纳时的最大限幅与负坐标安全 position clamp。
- [ ] 明确处理无 monitor、查询失败、极小 work area、NaN/无穷、整数极值和面积溢出，不 panic。
- [ ] 添加纯单测：单屏、多屏、负坐标、DPI 100%->150%/200%、显示器移除、完全离屏、过大、过小、工作区小于 minimum、无 monitor、极值。

聚焦门禁：

```powershell
cargo test -p wubilex-app window::bounds --all-features --locked
```

## 3. 建立生命周期状态机与 tray

- [ ] 新建 `src-tauri/src/window/mod.rs` 与 `tray.rs`，实现 `Visible/Hidden/Exiting`、window revision、last-normal placement、tray/delay ownership。
- [ ] 把固定 tray ID、两个 menu item ID 与两条中文 label 集中在 `tray.rs`；普通 visible startup 不创建 tray。
- [ ] 使用 Tauri core `TrayIconBuilder`、默认图标、tooltip、`show_menu_on_left_click(false)`、左键释放 restore 和右键原生菜单。
- [ ] hide 前 `ensure_tray`；创建失败 fail-open 保持/恢复可见。restore 保留已创建 tray，exit 只按固定 ID 删除。
- [ ] 使用 channel `recv_timeout(3s)` + generation token 实现 `/tray` delay；restore/exit 立即通知取消，timeout 后二次原子校验。
- [ ] 第二实例、tray show/left click 和内部 restore 复用同一 `set_skip_taskbar(false) -> unminimize -> show -> focus` 分支。
- [ ] 标题栏最小化、原生最小化和默认关闭复用 hide；原生 CloseRequested 阻止默认销毁并读取最新 closeAction。
- [ ] `closeAction=exit` 与 `tray.exit` 进入同一幂等显式退出；保存失败/超时不撤销退出。
- [ ] 添加状态机/adapter 单测：重复 hide/restore/exit、tray 创建失败、部分 native 失败、3 秒取消、timeout 竞态、唯一 tray、close policy 和显式退出绕过策略。

回滚点：此步骤完成后，纯状态机和 tray builder 可独立回退；不得用“启动时总建 tray”规避竞态测试。

## 4. 接入有界 placement worker

- [ ] 新建 `src-tauri/src/window/persistence.rs`，一个进程只启动一个 standard-thread worker。
- [ ] 合并规则实现为 250ms trailing quiet 或从第一条 pending 起 2s max，提交最新 placement。
- [ ] 在 `ConfigService` 增加内部 `update_window_placement(bounds, maximized)`，在同一配置事务锁内保留最新 closeAction。
- [ ] normal Moved/Resized/ScaleFactorChanged 更新 last-normal 并排队；maximized 只保存 flag + last-normal；minimized/hidden 不保存瞬时矩形。
- [ ] 成功 emit 完整 `config://changed`；失败保持 native state，写结构化 log 和 runtime notice。
- [ ] `FlushAndStop` 最多等待 1 秒；成功、错误、timeout、channel disconnect 都让 exit 继续。
- [ ] 添加确定性 worker 测试，尽量通过注入 clock/wait boundary 避免真实 2 秒 sleep；覆盖最新值、最大等待、并发 closeAction、失败和退出。

回滚点：配置 schema 不变。若 worker 不稳定，回退 worker 接入并保留纯 bounds/coordinator；不能退为在 WindowEvent 线程同步事务保存。

## 5. 接入 Tauri 启动与退出

- [ ] `lib.rs` 声明 window module，并把当前散落的 `activate_main_window` 逻辑迁入 coordinator。
- [ ] 创建窗口时使用 decorations=false、visible=false、focused=false、默认 `1024x680`、minimum `1024x640`，`/tray` 从构建起 skip taskbar。
- [ ] 应用 bounds/center/maximized 后才 show normal startup；`/tray` 保持隐藏并安排 delay。
- [ ] manage coordinator、注册 WindowEvent 后再 `mark_window_ready`，保留 setup 期间 secondary request 的现有队列语义。
- [ ] secondary callback 在 coordinator 可用且 runtime claim 成功时调用统一 restore；失败恢复 activation request 并发可见 notice。
- [ ] 显式退出后台 flush/remove tray 后调用 `app.exit(0)`；`RunEvent::Exit` 保持 session cleanup 并做 owned tray 幂等兜底。
- [ ] 结构化日志包含固定 event/stage、PID、版本和安全错误证据，不记录 argv/config 内容。

聚焦门禁：

```powershell
cargo test -p wubilex-app --all-features --locked
cargo check -p wubilex-app --all-targets --all-features --locked
```

## 6. 扩展 generated IPC 与可见 warning

- [ ] Rust 定义 `WindowControlIntent`、`WindowVisibility`、`WindowStateSnapshot`、`WindowStateChangedEvent`、`RuntimeNoticeEvent`。
- [ ] 实现 `window_state` 与 `window_control` 薄 command，所有 control 分发给 coordinator。
- [ ] 在现有 `AppErrorCode` 增加 `windowUnavailable`、`windowOperationFailed`，沿用统一 `AppError` 与 M7 归属。
- [ ] 在 `RuntimeNoticeCode` 增加 window operation/persistence/tray 三类有界 notice；相同后台错误去重，队列保持 8 条上限。
- [ ] native state change 发 `window://state-changed`；后台失败写权威 RuntimeState 后发 `app://runtime-notice`。emit 失败只写 log。
- [ ] 将 commands/events 加入唯一 bindings registry，运行生成并审查 TypeScript diff，不手改生成文件。
- [ ] 添加 Rust 序列化与 registry 测试，确认 revision 是 JS number、enum camelCase、event name 稳定、错误 detail 有界。

生成门禁：

```powershell
cargo xtask bindings
cargo xtask bindings --check
```

## 7. 实现可访问标题栏

- [ ] 建立 `src/icons/window-controls.ts`，只重导出需要的 Lucide icons。
- [ ] 建立 `src/lib/window-client.ts`，只包装生成 command/event，不复制 wire type 或 native state machine。
- [ ] 建立 `src/hooks/use-window-controls.ts`：先 listen 后 snapshot、按 revision 合并、卸载 unlisten、command failure 进入可见 warning。
- [ ] 建立 `src/components/window-title-bar/WindowTitleBar.tsx` 与测试，渲染 icon/name/version、最小化、最大化/还原、关闭。
- [ ] 品牌区使用 `data-tauri-drag-region="deep"`；按钮/控制区不带 drag attribute，双击由 Tauri 官方脚本处理。
- [ ] 使用原生 button、中文 aria-label/title、44x44 命中区、可见 focus；maximized snapshot 切换 Square/Copy 与 label。
- [ ] 在 `main.tsx` 用标题栏替换当前 runtime brand header，并把 runtime notice event 合入现有可见警告；不引入 route/provider/sidebar/theme 系统。
- [ ] 调整现有 stylesheet，保证 minimum、200% 系统字号、深浅 color scheme、reduced-motion、长 warning 和控制区均不重叠。
- [ ] jsdom/Testing Library 测试覆盖 listener race、旧 revision、Enter/Space、name/tooltip、icon state、drag isolation、command error 和 unlisten。

聚焦门禁：

```powershell
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run build
```

## 8. 调整 Windows Smoke

- [ ] 更新 `scripts/smoke-runtime.ps1` 对默认 close-to-tray 的旧退出假设，保留 marker/process 严格归属和 finally cleanup。
- [ ] 自动检查：普通启动主窗口可见、`/tray` 无闪窗、3 秒内 secondary 恢复、CloseMainWindow 后隐藏且进程存活、再次 secondary 恢复、重复操作不产生第二窗口。
- [ ] 通过固定日志事件或安全 native 证据检查 normal startup 未提前建 tray、delay cancel、hide/restore、owned cleanup；不使用任意 tray/marker 全局删除。
- [ ] 写入并执行可见 smoke checklist：拖动、双击、三个按钮、任务栏切换、tray 左键、右键仅两项、唯一 tray、显式退出、DPI 和多屏恢复。
- [ ] 显式退出后核对本次 PID、session marker 和固定 tray 消失；不把强制 Stop-Process 当作显式退出通过证据。
- [ ] smoke 失败时 finally 只终止本脚本创建且路径等于当前 debug executable 的进程，并只清理本次记录的 marker。

建议入库命令：

```powershell
pnpm run smoke:runtime
```

若自动与可见 smoke 分开，新增脚本名必须由 `package.json` 稳定 script 调用，最终门禁不依赖临时命令。

## 9. 全量质量检查

- [ ] 运行 Rust format/check/clippy/test/doc，修复 warning，不降低 lint。
- [ ] 运行 bindings freshness、docs、dependency policy 和 frontend frozen install/audit/typecheck/lint/test/build。
- [ ] 运行 task validation 与大小写不敏感静态搜索，确认没有额外 package manager、tray plugin、S2/S3、root resource 读取或 ImTip surface。
- [ ] 审查 git diff，确认没有 target/build artifact、用户配置、smoke transcript、截图临时文件或无关 metadata。

```powershell
$env:VOLTA_FEATURE_PNPM = '1'
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --all-features --no-deps --locked
cargo xtask bindings --check
cargo xtask check-docs

pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run build
pnpm run smoke:runtime

python ./.trellis/scripts/task.py validate .trellis/tasks/09-01-s1-window-tray
git status --short
```

静态搜索至少覆盖：

```powershell
rg -ni "imtip|tauri-plugin-(tray|store|window-state)|emptyworkingset|resource[/\\]" src src-tauri package.json pnpm-lock.yaml Cargo.toml Cargo.lock scripts
rg -n 'packageManager|engines.*pnpm|corepack|\bnpm\b|\byarn\b|\bnpx\b' package.json pnpm-lock.yaml .github scripts
```

搜索命中既有文档/锁文件元数据时逐条解释，不能用放宽模式或删除历史证据制造空输出。

## 10. Phase 3 收口

- [ ] 质量检查通过后使用 `trellis-update-spec`，只把真实实现形成的窗口 coordinator、错误、Hook、组件和测试模式回填相关 backend/frontend spec；不把规划文本当实现证据。
- [ ] 再运行受影响规范与全量门禁，更新本任务 acceptance 状态。
- [ ] 按 coherent change unit 准备提交：建议一个产品实现提交和一个 spec/task 收口提交；最终以实际 diff 为准。
- [ ] 未经用户明确要求不 push。
- [ ] 子任务通过并提交后 archive，回到父任务 `08-25-s1-shell-ui` 继续下一子项。

## Risky Files And Rollback Points

| 边界 | 风险文件 | 回滚条件 |
|---|---|---|
| Runtime lifecycle | `src-tauri/src/lib.rs`, `src-tauri/src/window/**` | 第二实例丢请求、普通启动闪窗、exit 被阻断 |
| Config transaction | `src-tauri/src/config/mod.rs`, placement worker | 高频 I/O、closeAction 被陈旧快照覆盖、退出死等 |
| IPC contract | bindings/events/errors/generated TS | 双 registry、手写 wire type、event/snapshot race |
| Security | Cargo feature、capability | 引入 plugin 或开放非必要 native write command |
| Frontend | titlebar/hook/main/style/package lock | 控件不可键盘使用、拖动吞按钮、200% 字号重叠 |
| Smoke | runtime smoke/package scripts | 删除非 owned 进程、marker、tray 或用户配置 |

任何局部失败优先回退所属边界；不得通过总是创建 tray、同步磁盘 I/O、扩大 capability、强杀任意进程或删除用户状态来“修复”测试。
