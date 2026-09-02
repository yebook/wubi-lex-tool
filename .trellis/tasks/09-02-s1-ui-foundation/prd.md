# S1 UI 基础

## Goal

在已经可运行的 Tauri/React 外壳上建立后续所有页面共同使用的 UI 基础：Tailwind CSS v4 语义令牌、无闪烁的深浅主题、标准/紧凑密度、简体中文资源，以及一组真正可访问且可测试的基础组件。该任务只交付基础设施和当前运行状态页的迁移，不提前实现路由、侧栏、设置页、命令面板或领域页面。

## User Value

- 应用从第一帧起就遵循用户保存的主题和密度，不出现白屏闪烁或先显示错误主题再跳变。
- 深浅主题、系统字号和减少动态效果下，文本、焦点、菜单和对话框仍清晰可用。
- 后续页面直接复用稳定的按钮、输入、菜单、对话框、tooltip 和键帽，不再各自发明样式、焦点行为或文案来源。
- 当前运行状态页与无边框标题栏继续可用，但不再维护一套临时颜色和硬编码中文文案。

## Background And Confirmed Facts

- `s1-runtime-lifecycle`、`s1-config-features` 和 `s1-window-tray` 已完成；当前唯一可见页面是运行状态页，窗口标题栏已建立 Lucide、44x44 控件、键盘语义和拖动区合同。
- Rust 已冻结 `UiConfig { theme, density, locale, sidebarCollapsed, onboardingVersion }`、`config_snapshot`、`config_update_ui` 与 `config://changed`，本任务不需要新增配置 schema 或 IPC 类型。
- 当前 `runtime-status.css` 同时持有临时颜色、字体和组件样式；Tailwind Vite 插件已经安装，但尚无 `src/styles/theme.css` 或产品级 Tailwind 工具类消费者。
- 产品设计事实来源已经定案：墨蓝主色、朱砂危险色、冷灰中性色、三层 surface、五区具名色、4/8 间距节奏、4/8/12 圆角、120/200ms 动效和系统 UI/等宽/字根三字体栈。
- 项目要求 Tailwind v4 CSS-first、`@theme inline`、`.dark` 类、`data-density="compact"`，且禁止 `tailwind.config.ts`、PostCSS 配置和组件内字面令牌。
- `AppLocale` 当前只有 `zh-CN`；首发只交付简体中文，但资源结构必须允许后续增加繁体中文和英文。
- `lucide-react@1.38.0`、Testing Library 和 jsdom 已由前序任务引入；React Router 尚无消费者，routing shell 是它的明确所有者。
- `ui-ux-pro-max` 的有效建议与仓库一致：桌面工具采用低变化、低动效、高密度、Minimal/Swiss 风格；其营销 hero、在线 Google Fonts、青绿色板和 GSAP 建议不适用于本产品，明确不采用。
- 根目录 `resource/` 不是本任务输入；ImTip 永久禁止。

## In Scope

### 1. Dependency And Tooling Boundary

- 使用项目 Volta pnpm 精确增加 `i18next@26.4.0`、兼容 React 19 的 `react-i18next`，以及 Dialog、Dropdown Menu、Tooltip 实际需要的按需 Radix packages。
- 使用 `class-variance-authority`、`clsx` 和 `tailwind-merge` 建立受控 variant/class 合并；只在 Button 等真实 variant 消费者中使用。
- 建立 Prettier 3 + `prettier-plugin-tailwindcss` 的前端格式检查，配置 `tailwindStylesheet: "./src/styles/theme.css"`，排除 Rust 生成的 bindings。
- 不安装 shadcn CLI 运行时依赖、统一 `radix-ui` 全家桶、GSAP、在线字体、主题插件或 Tailwind v3 兼容层。
- 本任务不安装 React Router；它推迟到下一项 `s1-routing-shell`，与第一个真实 route consumer 同提交引入。

### 2. Token And Global Style System

- 新建 `src/styles/theme.css`，用 `@import "tailwindcss"`、`@theme inline`、`@custom-variant dark` 和 `@custom-variant compact` 建立唯一令牌源。
- 完整定义浅色/深色语义色、surface/border/text 层级、success/warning/danger/info、五区色、字体栈、间距、圆角、阴影、z-index、控件尺寸、焦点和动效令牌。
- 只使用系统离线字体；本任务仅定义 `WubiLexEtymon` fallback 栈，不读取、安装或打包字根字体。
- 表面分层使用颜色与边框；阴影只用于菜单、tooltip 和 dialog 等真实浮层。卡片不嵌套，不引入渐变、装饰光斑或营销式版面。
- 把当前 runtime/titlebar 样式迁移到令牌或 Tailwind utilities，删除临时色板；保持 `1024x640`、44x44 窗口控件、200% 字号和长警告不重叠。

### 3. First-Frame Theme, Density And Locale

- Rust 在创建 WebView 前从同一 `ConfigSnapshot` 读取 `UiConfig` 与 `WindowConfig`，避免第二次配置读取和快照竞态。
- 通过 Tauri `WebviewWindowBuilder::initialization_script` 在 HTML 解析前设置根元素的 theme preference、resolved `.dark` class、`data-density` 和 `lang`；脚本只由有限枚举生成，不插入用户字符串。
- 显式 light/dark 同步 native WebView window theme；system 模式使用 `prefers-color-scheme`，前端运行后持续监听系统主题变化。
- 配置不可用时回退 `system + standard + zh-CN`，应用仍启动，并复用现有可见警告路径。
- 不使用 localStorage、sessionStorage、cookie、Vite flag 或第二配置文件保存外观状态。

### 4. UI Preferences Runtime Boundary

- 建立 typed config client 和 `UiPreferencesProvider`，只消费生成的 `UiConfig` / `ConfigSnapshot` / `ConfigChangedEvent`。
- 初始化顺序为先监听 `config://changed`、再读取 snapshot；按单调 revision 合并，旧 snapshot 不覆盖新 event。
- 暴露 theme/density/locale 的受控更新 API，更新时立即投影 DOM，再串行保存完整 UI group；失败回滚到最后确认快照并提供有界可见 warning。
- system theme media listener、Tauri event listener 和未完成异步注册都有可测试 cleanup；React StrictMode 不产生永久重复订阅。
- 本任务只建立 provider/API，不增加临时主题切换器或设置页；真实可见选择器由 routing shell 的外观设置消费。

### 5. Internationalization

- 建立 `src/i18n/` 入口和 `zh-CN` 资源，使用同步内置资源启动 i18next，断网时完整可用。
- 把当前 React 运行状态页、标题栏、重试、状态和前端 warning 文案全部移出组件/Hook/view projection；产品名、版本号和 Rust 返回的权威错误 payload 不伪装为翻译资源。
- view projection 通过窄 translator 依赖生成展示文案，保持纯测试，不在业务 helper 内读取不可替换全局。
- 预留 namespace/locale 注册点，但不添加空的英文或繁体资源，不把字根歌诀等专业内容纳入 UI 翻译。

### 6. Minimal Accessible Primitives

- 在 `src/components/ui/` 建立实际会被后续 S1 消费的 Button、Input、Dropdown Menu、Dialog、Tooltip 和 Kbd，以及统一 Overlay/Tooltip provider。
- Button 使用原生 button、默认 `type="button"`、明确 variants/sizes、disabled/busy 语义和稳定命中区；Input 保留原生 label 关联、invalid/disabled/read-only 语义。
- Menu/Dialog/Tooltip 只包装按需 Radix primitive，保留其键盘、焦点、Escape、portal 和 aria 行为；不复制一套手写 focus trap 或 roving tabindex。
- Dialog 关闭并恢复触发器焦点；Dropdown Menu 支持方向键/Enter/Escape；Tooltip 同时支持 hover 和 keyboard focus，不能只依赖 `title`。
- Portal 统一进入 `#overlay-root` 并使用受控 z-index；overlay 不被当前页面 overflow 裁切，不产生嵌套 provider。
- Kbd 是非交互语义展示；图标一律经 `src/icons/` 的窄 Lucide 出口，组件不使用 emoji、图标字体或手写结构 SVG。

### 7. Testing And Visual Verification

- Rust 单测覆盖三种 theme、两种 density、locale、默认回退、固定脚本内容、无用户数据插值和 WebView builder 接线。
- Vitest/Testing Library 覆盖 provider listener-first/revision/rollback/system media cleanup，以及所有 primitive 的鼠标、键盘、焦点、disabled/invalid/busy 和 portal 行为。
- 自动校验浅色/深色关键前景背景对比度、两套 token 完整性、`@theme inline`、dark/compact variant 和禁止配置文件。
- 浏览器检查覆盖 light/dark、standard/compact、`1024x640`、`1440x900`、200% 字号、reduced-motion、长警告、dialog/menu/tooltip，无横向溢出和遮挡。
- 更新 Windows runtime smoke，确认首帧主题接线没有破坏 `/tray`、第二实例、关闭到托盘和退出清理。

## Out Of Scope

- React Router、七领域 route table、侧栏、应用栏、状态栏、返回栈、深链接消费和任何页面级导航；由 `s1-routing-shell` 实现。
- 外观设置页、可见主题/密度选择器和 sidebar 折叠控件；本任务只提供其 provider/API。
- 命令面板、动作注册表、快捷键录制、全局热键；由 `s1-actions-keymap` 实现。
- toast、错误详情面板、任务进度、确认流程、空状态、拖放和首次知情说明；由 `s1-task-feedback` 实现。
- 虚拟表格、状态卡片、feature placeholder、虚拟键盘、字根图、CodeMirror 和领域组件。
- 新配置字段、schema migration、新 IPC wire type、数据库、网络、在线字体、字根字体资源或根目录 `resource/`。
- ImTip 的任何入口、占位、文案、动作、设置、依赖或资源。

## Acceptance Criteria

- [x] `AC-UIF-01` `src/styles/theme.css` 是唯一产品令牌源，使用 Tailwind v4 `@theme inline` 和 dark/compact custom variants；不存在 `tailwind.config.ts`、PostCSS 配置或组件内字面色板。
- [x] `AC-UIF-02` light/dark/system、standard/compact 和 `zh-CN` 从已保存 `UiConfig` 在 WebView 第一帧前生效；显式主题不先闪系统主题，system 会响应运行时系统变化。
- [x] `AC-UIF-03` UI preferences 先监听再 snapshot、按 revision 拒旧、串行提交、失败回滚并显示 warning；配置失败不阻止应用启动且不产生第二持久化来源。
- [x] `AC-UIF-04` 浅色/深色语义色、三层 surface、两层 border、三层 text、状态色与五区色完整；关键文本、图标、边框和 focus 在两主题满足既定对比度。
- [x] `AC-UIF-05` UI/mono/etymon 三字体栈和 4/8 间距、4/8/12 圆角、120/200ms 动效均为令牌；无 Google Fonts、运行时字体下载或 GSAP。
- [x] `AC-UIF-06` 当前所有 frontend-owned 用户文案位于 `src/i18n/` 简体中文资源中，组件和 view helper 通过 typed/narrow translation boundary 消费；新增 locale 可注册而不改组件结构。
- [x] `AC-UIF-07` Button、Input、Dropdown Menu、Dialog、Tooltip、Kbd 与 Overlay provider 位于 `src/components/ui/`，只依赖必要的按需 Radix/shadcn-style helpers，并具备明确 props/variant 合同。
- [x] `AC-UIF-08` Dialog/Menu/Tooltip 的焦点、方向键、Enter/Space、Escape、触发器恢复、portal、accessible name/state 和 disabled 行为通过真实 Testing Library/user-event 测试。
- [x] `AC-UIF-09` 当前运行状态页和 WindowTitleBar 迁移后功能、拖动区、44x44 窗口按钮、可见 warning、加载/重试及 keyboard 行为不回归。
- [x] `AC-UIF-10` `1024x640` 与 `1440x900`、200% 系统字号、standard/compact、light/dark 和 reduced-motion 下无横向溢出、文本截断、焦点遮挡或 incoherent overlap。
- [x] `AC-UIF-11` 新依赖精确锁定且 audit 通过；Lucide 不重复安装，React Router 延后到 routing consumer，不增加 shadcn CLI/runtime 全家桶、主题插件或其它 package-manager 来源。
- [x] `AC-UIF-12` Rust/frontend 格式、Clippy、tests、bindings/docs、frozen install/audit、typecheck、lint、format check、build、task validation、静态搜索和 Windows runtime smoke 全部通过。
- [x] `AC-UIF-13` 生产代码、依赖、样式、资源、组件和文案均不读取根目录 `resource/`，且不存在 ImTip surface 或 S2/S3 行为。

验收证据记录于 `implement.md` 的“Verification Record”；本地缺少的
`cargo-llvm-cov` 与 `actionlint` 是工具环境缺口，不属于上述 AC 中已声明
通过的门禁，CI 仍安装并执行既定 coverage 工具。

## Key Decisions

- 仓库 `docs/21-ui-ux.md` 的墨蓝/朱砂/冷灰系统优先于搜索工具推荐；搜索工具只验证 Minimal/Swiss、低动效、高密度和可访问性方向。
- first-frame appearance 由 Rust 已加载配置驱动的 Tauri initialization script 完成；不以 localStorage 交换一次闪烁或建立第二事实源。
- i18n 只交付真实 `zh-CN` 内容与扩展入口，不提交空翻译文件。
- primitives 采用按需 Radix packages 和项目自有 source，不安装统一 `radix-ui` 全家桶或 shadcn CLI 运行时依赖。
- React Router 等到 `s1-routing-shell` 首次消费时引入，避免本任务产生未使用依赖；这是对父任务早期安装清单的收紧，不改变 S1 最终技术栈。
- 不建立临时主题选择页面或 component showcase；自动化和当前 runtime surface 提供验收载体。

## Risks And Deferred Items

- Tauri initialization script 在 HTML 解析前执行，必须处理 `documentElement` 尚未创建和 Windows 子 frame 行为；脚本限制为 top-level、固定枚举并有源码级单测。
- `config_update_ui` 接收完整 UI group；并发 field update 若无序列化会覆盖 sibling 字段。本任务必须建立单一更新队列并始终从最新确认/乐观状态构造下一份 UiConfig。
- Radix portal/focus 行为在 jsdom 可验证大部分语义，但最终遮挡和 WebView 焦点仍需真实浏览器与 Windows smoke。
- 字根 PUA 字体资源仍由后续 resource/domain 任务提供；本任务只冻结字体栈和 fallback，不把缺少字体误报为 UI foundation 失败。
