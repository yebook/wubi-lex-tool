# Implementation Plan - S1 外壳与 UI 骨架

## Entry Gate

- [ ] 用户在本轮最终规划摘要之后明确批准进入实施。
- [ ] 仅在获批后运行 `python ./.trellis/scripts/task.py start .trellis/tasks/08-25-s1-shell-ui`。
- [ ] 开始每个子任务前使用 `trellis-before-dev` 加载目标层规范，并为该子任务创建独立 Trellis 任务。
- [ ] 验证 `VOLTA_FEATURE_PNPM=1`，且 `node` / `pnpm` / `cargo` 与项目 pin 一致；不安装或切换全局 pnpm。
- [ ] 确认工作树状态，保护用户已有改动；不读取根目录 `resource/`。

## Ordered Child Work

- [ ] 1. `s1-runtime-lifecycle`
  - 建立 `index.html`、React entry、Tauri main/library 双入口和 dev/build wiring。
  - 接入最小日志、panic/异常会话标记、参数 parser、单实例转交和管理员权限检测。
  - 配置产品元数据、严格 CSP 和最小 capability。
  - 验收：普通启动、重复启动、未知参数、`/tray` 参数解析、正常/异常退出单测和 Windows smoke。

- [ ] 2. `s1-config-features`
  - 冻结 S1 config schema、feature ID、command/event 和 `AppError` 契约并生成 bindings。
  - 实现默认值、分组更新、原子保存、写前备份、逐版本迁移、损坏回退和导入导出。
  - 实现 `app_features` 与 frontend Zustand feature store；不引入 Vite feature flags。
  - 验收：临时目录中的成功、空、旧版、损坏、I/O 失败、导入失败 rollback 和 feature 一致性测试。

- [ ] 3. `s1-window-tray`
  - 实现无边框标题栏和 native window coordinator。
  - 实现多显示器 bounds 校正、最大化状态、关闭策略和任务栏/托盘切换。
  - 从动作目录构建托盘菜单，接入还原、置前、退出和后续阶段禁用项。
  - 验收：`/tray` 无闪窗、托盘清理、离屏恢复、不同 DPI 和退出拦截 Windows smoke；不存在 ImTip 或通用“相关工具”入口。

- [ ] 4. `s1-ui-foundation`
  - 安装经批准且尚未由前序任务引入的 React Router、i18next、Lucide、shadcn 所需依赖，并更新 lockfile；Zustand 由首个真实消费者 `s1-config-features` 引入。
  - 建立 `theme.css`、首帧主题引导、密度、字体、五区令牌和简体中文资源。
  - 实现经过评审的基础按钮、输入、菜单、对话框、tooltip、kbd 和 overlay host。
  - 验收：Tailwind v4 `@theme inline`、无字面令牌、深浅对比度、reduced-motion、200% 字号和 icon accessible name。

- [ ] 5. `s1-routing-shell`
  - 建立 provider tree、七领域 route table、稳定 route IDs、深链接 parser 和返回栈。
  - 实现 sidebar、app bar、status bar、overview skeleton、settings groups 和三类 feature placeholder。
  - 接入侧栏折叠、主题/密度和 route focus management。
  - 验收：侧栏/快捷路由/深链接/返回一致，七入口可达，未实现 feature 不调用缺失 command。

- [ ] 6. `s1-actions-keymap`
  - 实现 Rust 动作目录、typed descriptor、frontend dispatcher 和托盘/命令面板共用投影。
  - 实现应用内快捷键、全局热键 adapter、事务式改绑、冲突/保留键检测和配置持久化。
  - 实现命令面板、键帽、热键录制器和快捷键设置完整流程。
  - 验收：所有来源提交同一 action ID；占用失败保留旧绑定；默认表、清除、恢复和导入导出测试通过。

- [ ] 7. `s1-task-feedback`
  - 实现 typed event catalog、task registry、revisioned progress、互斥和 cancellation。
  - 实现状态栏任务入口、进度详情/取消、12 阶段不可中断 overlay、错误详情、toast、空状态、确认、未保存拦截、拖放和首次知情说明。
  - 用 synthetic/test adapter 覆盖长任务状态，不添加假业务数据或真实系统操作。
  - 验收：事件丢失后可用 snapshot 重建；取消/失败恢复控件；错误可复制且脱敏；全部弹层键盘可操作。

- [ ] 8. `s1-integration`
  - 联调第二实例/托盘/深链接/route/action/配置/热键/任务/错误的完整数据流。
  - 执行 Windows 可见 smoke、异常进程终止重启、全局热键占用和 release 启动性能测量。
  - 对 `1024x640`、`1440x900` 和高 DPI 截图做深浅主题、文本溢出、遮挡和空白检查。
  - 回填 frontend/backend Trellis spec 的真实模式，运行独立 `trellis-check`，修复后再做最终验收。

## Cross-Task Gates

- [ ] G1：真实 frontend/runtime entry 通过后，才能让其他子任务依赖可运行壳。
- [ ] G2：config schema、feature IDs、route IDs、action IDs、event names 和 error codes 在首个消费者前冻结。
- [ ] G3：任何 Rust IPC 变更同提交包含生成 bindings 和 freshness check。
- [ ] G4：托盘、命令面板、快捷键和按钮不得各自复制动作名称、可用性或执行逻辑。
- [ ] G5：S1 的 placeholder 绝不触发 S2/S3 command；没有用假数据伪装系统状态。
- [ ] G6：超过 100 ms 的测试操作放后台；互斥和取消由 Rust registry 强制，UI disabled 只做反馈。
- [ ] G7：视觉子任务在开始和结束时分别使用 `ui-ux-pro-max` 规则检查；若本机脚本仍缺失，记录降级并执行仓库令牌、对比度、键盘和截图验收。
- [ ] G8：每个子任务通过聚焦测试和 `task.py validate` 后独立提交；父任务只在 S1 integration 全部通过后收口。
- [ ] G9：生产根目录、manifest、capability、route、action、tray、settings 和 feature catalog 的大小写不敏感搜索与快照检查均确认没有 ImTip 集成。

## Final Validation

```powershell
$env:VOLTA_FEATURE_PNPM = [Environment]::GetEnvironmentVariable('VOLTA_FEATURE_PNPM', 'User')
if ($env:VOLTA_FEATURE_PNPM -ne '1') { throw 'VOLTA_FEATURE_PNPM must be 1' }
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
$env:RUSTDOCFLAGS='-D warnings'; cargo doc --workspace --all-features --no-deps --locked
cargo xtask bindings --check
cargo xtask check-docs

pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run typecheck
pnpm run lint
pnpm run test --run

python ./.trellis/scripts/task.py validate .trellis/tasks/08-25-s1-shell-ui
git status --short
```

S1 子任务可在 `package.json` / `xtask` 中加入稳定的 browser 与 Windows smoke 命令；最终门禁必须调用入库命令，不以临时命令替代。

## Rollback Points

- runtime/plugin、config schema、window/tray、UI tokens/routes、action/keymap、task/feedback 分别提交。
- 新插件导致 capability、CSP 或启动回归时，回退所属子任务，不放宽安全边界。
- 配置迁移一旦发布给测试用户，只能新增前向 migration；不得静默改写旧 schema 含义。
- 全局热键改绑失败必须自动恢复旧注册和旧配置，不能要求用户重启修复。
- 集成阶段发现 S2/S3 领域逻辑渗入时，移除该逻辑并恢复 feature placeholder，而不是扩大 S1 范围。
