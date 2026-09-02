# Implementation Plan - S1 UI 基础

## Entry Gate

- [x] 用户审阅本任务最终规划摘要，并在后续消息中明确批准实施。
- [x] 获批后运行 `python ./.trellis/scripts/task.py start .trellis/tasks/09-02-s1-ui-foundation`；未 start 前不修改产品代码或安装依赖。
- [x] 实施前加载 `trellis-before-dev`，读取 frontend directory/component/hook/state/type/quality/tailwind 规范、backend directory/error/quality 规范及 cross-layer/toolchain guides。
- [x] 核对工作树和项目 Volta pnpm `11.18.0`；不使用 packageManager、Corepack、npm、yarn、npx 或全局 pnpm。
- [x] 不读取或修改根目录 `resource/`；全库不增加 ImTip surface。

## 1. Freeze Minimal Dependencies And Formatting

- [x] 精确增加 `i18next@26.4.0`、`react-i18next@17.0.13`。
- [x] 精确增加 `@radix-ui/react-dialog@1.1.23`、`@radix-ui/react-dropdown-menu@2.1.24`、`@radix-ui/react-tooltip@1.2.16`、`@radix-ui/react-slot@1.3.3`。
- [x] 精确增加 `class-variance-authority@0.7.1`、`clsx@2.1.1`、`tailwind-merge@3.6.0`。
- [x] 精确增加 dev dependencies `prettier@3.9.6`、`prettier-plugin-tailwindcss@0.8.1`，建立 format/format:check 和 generated bindings exclusion。
- [x] 不增加 React Router、radix-ui 全家桶、shadcn CLI dependency、GSAP、Google Fonts、theme plugin 或 Tailwind v3 package/config。
- [x] 审查 lockfile 只包含上述依赖的必要闭包，pnpm version source 不漂移。

建议命令：

```powershell
pnpm add --save-exact i18next@26.4.0 react-i18next@17.0.13 `
  @radix-ui/react-dialog@1.1.23 @radix-ui/react-dropdown-menu@2.1.24 `
  @radix-ui/react-tooltip@1.2.16 @radix-ui/react-slot@1.3.3 `
  class-variance-authority@0.7.1 clsx@2.1.1 tailwind-merge@3.6.0
pnpm add --save-dev --save-exact prettier@3.9.6 prettier-plugin-tailwindcss@0.8.1
```

## 2. Establish Tailwind v4 Theme Tokens

- [x] 新建 `src/styles/theme.css`，接入 `@import "tailwindcss"`、`@theme inline`、dark 与 compact variants。
- [x] 定义 docs 既定浅/深语义色、surface/border/text、状态色、五区色、三字体栈、spacing/radius/control/shadow/z-index/motion/focus tokens。
- [x] 建立 global box sizing、body/root、color-scheme、font synthesis、letter spacing、focus-visible 和 reduced-motion 基线。
- [x] 为普通 Vite browser 提供 system-theme fallback，但不持久化或覆盖 native bootstrap。
- [x] 迁移 `runtime-status.css` 与 WindowTitleBar，删除临时 root 色板和组件内字面主题值；保持 native 交互尺寸与布局。
- [x] 添加 token 完整性和 sRGB contrast 自动测试，覆盖 light/dark 必要组合。

聚焦门禁：

```powershell
pnpm run test -- src/styles
pnpm run build
```

## 3. Add Native First-Frame UI Bootstrap

- [x] 新建 `src-tauri/src/ui_bootstrap.rs`，纯映射 `UiConfig` 到固定 document-start script、native theme 与 surface background。
- [x] 脚本限制 top-level，处理 documentElement 尚未创建，设置 `.dark`、data-theme、data-density、lang 和 color-scheme 后立即停止 observer。
- [x] `lib.rs` 一次 config snapshot 同时提取 window/ui；失败统一回退默认并沿用现有脱敏 notice/log。
- [x] 在 WebviewWindowBuilder build 前接入 initialization script/theme/background，不改变 hidden startup、placement 或 coordinator 顺序。
- [x] 添加 Rust 单测覆盖全部 enum、默认值、固定脚本、无动态用户数据和 builder helper。

聚焦门禁：

```powershell
cargo test -p wubilex-app ui_bootstrap --all-features --locked
cargo check -p wubilex-app --all-targets --all-features --locked
```

回滚点：native bootstrap 独立于 frontend provider；失败时回退该模块，不引入 Web Storage。

## 4. Implement UI Preferences Provider

- [x] 建立 `src/lib/config-client.ts`，只包装生成 snapshot/update/event。
- [x] 建立 DOM appearance projection helper，system 模式监听 matchMedia，cleanup 可注入测试。
- [x] 建立 `src/app/providers/ui-preferences-provider.tsx` 和 `useUiPreferences`，初值来自 bootstrap attributes。
- [x] 实现 listener-first、revision merge、event-before-snapshot 防旧覆盖和异步 unlisten cleanup。
- [x] 实现单一 patch queue：同步 optimistic DOM、full UiConfig 串行保存、command/event revision 合并、失败回滚和可见 bounded warning。
- [x] 把 provider warning 接到当前 runtime notice surface；不创建临时外观控件或第二 store/persistence。
- [x] 测试 StrictMode、system media change、并发 theme/density patch、失败回滚和 cleanup。

## 5. Establish Bundled i18n

- [x] 建立 `src/i18n/resources/zh-CN.ts`、typed resource registry 和同步 i18next initialization。
- [x] 建立 app-level I18nextProvider；locale 取 generated AppLocale/root bootstrap，不访问网络。
- [x] 外置 RuntimeApp、WindowTitleBar、runtime-view、window Hook 和 provider 的 frontend-owned 文案、aria labels 与 fallback warnings。
- [x] 纯 view helper 注入窄 translator；backend message/detail、brand/version/code/path 保持数据。
- [x] 添加初始化、key/interpolation、fallback 和静态中文分布测试；不创建空 en/zh-TW 资源。

## 6. Implement Minimal UI Primitives

- [x] 建立唯一 `src/lib/cn.ts`。
- [x] 在 `src/components/ui/` 实现 Button、Input、Kbd，固定语义、variants、disabled/read-only/invalid/busy、稳定尺寸和 focus。
- [x] 实现 app-level Overlay/Tooltip provider 与固定 `#overlay-root`。
- [x] 实现 Dialog wrapper：title/description/close、portal、scrim、Escape、focus trap/restore。
- [x] 实现 Dropdown Menu wrapper：group/separator、方向键、Enter、Escape、disabled item。
- [x] 实现 Tooltip wrapper：app provider、hover/focus、supplementary description，不用 title 替代。
- [x] 只重导出真实使用的 Lucide icons；不使用 emoji、icon font 或手写结构 SVG。
- [x] colocated Testing Library/user-event tests 覆盖 pointer/keyboard/focus/aria/portal/states；不添加 showcase 页面。

## 7. Compose Providers And Migrate Current Surface

- [x] 把 `main.tsx` 收窄为 styles/i18n/provider/root composition；将 RuntimeApp 移到可测试 owner（如需要）。
- [x] provider 顺序固定为 i18n -> UI preferences -> overlay/tooltip -> current app。
- [x] 当前 retry/button/input 等真实位置使用新 primitive；WindowTitleBar 保留 native-specific component，不强行改成 generic Button。
- [x] 保留 feature store、runtime listener、window listener、visible warnings 和 load/retry behavior。
- [x] 确认页面没有新增 sidebar/routes/settings/marketing copy、嵌套卡片或功能说明面板。

## 8. Browser And Windows Verification

- [x] Vitest 全量覆盖新增 provider/i18n/primitives 和既有 frontend regression。
- [x] 使用 Playwright 对 light/dark x standard/compact 在 `1024x640`、`1440x900` 采样；检查非空像素、横向溢出、标题栏/警告/overlay 遮挡。
- [x] 对 200% root font、reduced-motion、keyboard-only、长翻译/warning 做布局和焦点检查。
- [x] 真实 Tauri smoke 使用 isolated debug data 写入不同 UI config，确认首帧、普通启动、`/tray`、第二实例、close-to-tray 和 clean exit。
- [x] 截图只留在 ignored `target/`，不提交临时 browser profile、transcript 或 build artifact。

## 9. Full Quality Gate And Static Review

- [x] 运行 Rust fmt/check/clippy/test/doc；接受现有 admin-manifest all-target bin harness 的 Windows 740 环境限制，但标准 workspace/library tests 必须全绿。
- [x] 运行 bindings freshness、docs、task validation、dependency audit、frontend format/typecheck/lint/test/build 和 runtime smoke。
- [x] 使用 `ui-ux-pro-max` 做结束检查：focus/keyboard、two-theme contrast、overlay、density、text scaling、reduced-motion 和 stable states。
- [x] 静态搜索 Tailwind v3/PostCSS/Web Storage/Google Fonts/GSAP/extra icons/Router/root resource/ImTip/frontend hardcoded Chinese，逐条解释允许命中。
- [x] 审查 git diff：无 package metadata 漂移、生成 bindings 手改、build output、用户配置、smoke data 或无关格式化。

```powershell
pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run format:check
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run build

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --all-features --no-deps --locked
cargo xtask bindings --check
cargo xtask check-docs

pnpm run smoke:runtime
python ./.trellis/scripts/task.py validate .trellis/tasks/09-02-s1-ui-foundation
git diff --check
git status --short
```

## 10. Phase 3 Closure

- [x] 使用 `trellis-check` 做全任务独立审查并直接修复 verified findings。
- [x] 使用 `trellis-update-spec` 回填真实形成的 token、provider/i18n、primitive、overlay、format/test 规范，不把规划文本当实现证据。
- [x] 更新本任务 acceptance 与验证记录。
- [x] 按规划/实现/spec coherent units 提交；未经用户明确要求不 push。
- [ ] 归档子任务并记录 journal，回到父任务创建 `s1-routing-shell`。

## Verification Record

- Frontend: format, typecheck, lint, production build, and 12 Vitest files / 66 tests passed.
- Rust: fmt, workspace check, strict Clippy, Rustdoc, and 220 workspace tests passed; `wubilex-app` contributed 69 tests.
- Contracts: generated bindings are current; documentation counts are `414/101/115/630` with zero dangling IDs, placeholders, or broken anchors; all 8 codec fixtures and both task validation passes succeeded.
- Dependencies: frozen install passed without changing `pnpm-lock.yaml`; SHA-256 remained `33DB24671329638B4B72DD6C3D70D616373DDC77F58F8BF65A84353DB74A37F8`; official npm audit found no known vulnerability; `cargo deny check` passed with only existing duplicate-version warnings.
- Runtime and browser: all 4 Windows smoke stages passed; 12 theme/density/viewport combinations and 2 overlay combinations passed, including system theme, 200 percent text, reduced motion, focus, portal, Escape, and focus restoration.
- Environment gaps: local `cargo-llvm-cov` and `actionlint` executables were unavailable. CI installs `cargo-llvm-cov`; workflow structure remains covered by repository tests. The admin-manifest binary harness under `cargo test --all-targets` still encounters the known Windows elevation error 740, while the standard workspace test gate passes.

## Risky Files And Rollback Points

| Boundary | Risky files | Rollback condition |
|---|---|---|
| First frame | `src-tauri/src/lib.rs`, `ui_bootstrap.rs`, `theme.css` | explicit theme flashes/wrong frame, `/tray` flashes, CSP regression |
| Config sync | config client/provider | stale revision wins, sibling field overwrite, failure stays falsely applied |
| Tokens | `theme.css`, runtime/titlebar styles | contrast failure, compact breaks target, old palette remains second source |
| i18n | `src/i18n/**`, current views | missing key, untranslated frontend literal, backend payload incorrectly translated |
| Overlay | Dialog/Menu/Tooltip/provider/index | focus trap/restore failure, overlay clipped or covers controls incoherently |
| Tooling | package/lock/Prettier/CI | generated binding churn, competing pnpm source, whole-repo metadata formatting |

任何局部失败优先回退所属 boundary；不得通过 localStorage、CSP 放宽、手写 focus trap、统一全家桶依赖、删除失败证据或扩大本任务到 routing/feedback 来绕过。
