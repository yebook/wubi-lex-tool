# 设计 — WubiLexTool 架构定案与目录骨架

> 本文是**定案内容的规格**：`implement.md` 按此逐节写入 `docs/`，并按 §4 创建目录。
> 本任务不写业务代码，所以这里没有函数签名；到 **trait 缝**与**目录**为止。

## 1. 本轮新增设计决策（接续 `02` §7 的 D1–D7）

### D8 — 前端框架定案：React 19 + TypeScript

**背景**：`02` §6.1 把 React 列为「建议」，并留了「Vue / Svelte 同样可行」的口子。留着口子等于每个前端 PR 都要重新论证一次。

**决策**：**React 19 + TypeScript**，不再保留备选表述。

**理由**（按本项目的真实约束排序）：

1. 唯一的硬约束是**虚拟化数十万行**（`R4`、`UX-BIGDATA-*`）。TanStack Virtual 的 React 适配是其一等公民实现。
2. 「文本模式」编辑器需要 CodeMirror 6，其 React 绑定最成熟。
3. shadcn/ui 的 React 版本是上游本体，Vue/Svelte 移植滞后于上游。

**代价**：React 的重渲染心智负担高于 Svelte。对冲：状态用 Zustand 的 selector 订阅，表格行组件强制 `memo`。

### D9 — 样式定案：Tailwind CSS v4（CSS-first）

**背景**：`21` §4.6 与 `UX-TOKEN-011` 写的是 v3 机制（`tailwind.config.ts` + `theme.extend` + `rgb(var(--x) / <alpha-value>)` + `darkMode: 'class'`）。实测当前 `tailwindcss@4.3.3`，v4 已改为 CSS-first，上述三项语法均不再适用。

**决策**：升级到 **v4**，令牌单一事实来源为 `src/styles/theme.css`，**不存在 `tailwind.config.ts`**。

**机制映射**：

| v3 机制（文档原写法） | v4 定案写法 |
|---|---|
| `tailwind.config.ts` + `theme.extend.colors` | `@theme inline { --color-*: var(--wl-*) }` |
| `darkMode: 'class'` | `@custom-variant dark (&:where(.dark, .dark *))` |
| `rgb(var(--x) / <alpha-value>)` | 直接写颜色值；透明度用原生 `bg-primary/50` |
| PostCSS 插件链 | `@tailwindcss/vite` 插件，无 `postcss.config.js` |
| `data-density="compact"` 属性选择器 | `@custom-variant compact (&:where([data-density="compact"], [data-density="compact"] *))` |
| `prettier-plugin-tailwindcss` 的 `tailwindConfig` 选项 | 改为 `tailwindStylesheet: "./src/styles/theme.css"` |

**深浅色令牌的正确写法**（`@theme inline` 是关键 —— 它让工具类在**使用处**解析变量，而非在定义时把值烘死，深浅色才能靠切换 `.dark` 生效）：

```css
/* src/styles/theme.css */
@import "tailwindcss";

@custom-variant dark    (&:where(.dark, .dark *));
@custom-variant compact (&:where([data-density="compact"], [data-density="compact"] *));

@theme inline {
  --color-primary:      var(--wl-primary);
  --color-surface-1:    var(--wl-surface-1);
  --color-wubi-zone-1:  var(--wl-zone-1);
  --font-mono:          "Cascadia Code", "JetBrains Mono", Consolas, monospace;
  --font-etymon:        "WubiLexEtymon", "Cascadia Code", monospace;
}

:root  { --wl-primary: #1E3A5F; --wl-surface-1: #FFFFFF; --wl-zone-1: …; }
.dark  { --wl-primary: #7DA7D9; --wl-surface-1: #16191D; --wl-zone-1: …; }
```

**语义不变，所以 `UX-TOKEN-011` 保留 ID 只改描述**（依 `requirement-id-conventions.md` §6：语义实质变化才作废 ID）。该需求断言的三件事——基于 Tailwind、class 驱动深色、令牌不写字面值——在 v4 下全部成立。

**代价**：v4 的生态插件覆盖略逊于 v3；shadcn/ui 需用其 v4 分支的组件源码。

### D10 — workspace 布局：`src-tauri` 是 member，`wubilex-learn` 暂不进默认 members

**决策**：

- 根 `Cargo.toml` 为 virtual workspace，members = `crates/wubilex-{codec,core,winime,resource}` + `src-tauri` + `xtask`。
- 保留 Tauri 约定的 `src-tauri/` 路径（不改名为 `crates/wubilex-app`），换取 `tauri` CLI 与全部模板/文档的零摩擦。
- `crates/wubilex-learn/`（M8 · P2 · 阶段 S8）**建目录、写 README，但 S8 前不列入 members**。

**理由**：`wubilex-learn` 会拖入 `jieba-rs`（内置词典约 5 MB）与 UIA 绑定，这是 S0–S7 全程用不到的编译负担。目录先占位是为了让分层边界从第一天就可见。

**代价**：S8 开工时需改一行 members。可接受。

### D11 — IPC 类型单一事实来源：Rust 生成 TypeScript

**背景**：`02` §3 定义了 command / event 契约，但没说前端的 TS 类型从哪来。手写两份必然漂移——这是 `AppError`、`TaskProgress` 这类跨越 60+ command 的共享类型上最容易出的错。

**决策**：用 **`tauri-specta`** 从 Rust 侧的 command / event 定义生成 TS，产物落 `src/types/generated/`，**入库**（便于 review 时看到契约 diff）。CI 校验产物是否过期。

**备选**：若 `tauri-specta` 落后于 `tauri` 版本，降级为 `ts-rs`（只生成类型，command 签名手写）+ 一组契约测试。切换触发条件：S1 开工时 `tauri-specta` 不支持当时的 `tauri` 2.x 小版本。

### D12 — 配置存储：自建 `serde` + TOML，不用 `tauri-plugin-store`

**背景**：`02` §5.1 写的是「`tauri-plugin-store` 或自建 `serde` + TOML」。

**决策**：**自建**。

**理由**：`M7-CONF-002/004/007` 要求 schema 版本号、损坏检测与回退、写入前备份。`tauri-plugin-store` 是无 schema 的 JSON KV，这三项都要在它之上再糊一层，不如直接用强类型 TOML。TOML 还让用户可手工修复损坏的配置——这正是 `R31` 的兜底路径。

**代价**：自己实现原子写（临时文件 + rename）与迁移链。

### D13 — 简繁转换：内置映射表，不引 `opencc-rust`

**背景**：`R33` 已登记 `opencc-rust` 的构建复杂度与体积风险。

**决策**：内置字符级简繁映射表（对应 `M1-XFORM-008/009`），OpenCC 级别的词组感知转换列为 P2 增强。

**理由**：原项目用的 `string.conv` 也是表驱动的字符映射，行为契约层面本就只要求到字符级；引入 C++ 依赖换来的词组转换能力超出了需求。

**代价**：「乾坤」这类需要词组上下文的转换会不准。用 `R36` 的对照测试量化差异，超阈值再上 OpenCC。

**回写**：`02` §10 的 `R33` 缓解列改为「已决策：内置转换表，OpenCC 降为 P2 增强」。

### D14 — 压缩：LZMA 只读不写，导出统一 zstd

**背景**：`02` §5.2 指出 `lzma-rs` 只支持解压。实测 `lzma-rs@0.3.0` 仍然只解压。

**决策**：

- **读**：`lzma-rs` 解 LZMA alone（兼容上游既有的 `spelling.tar.lzma` 等历史资源）。
- **写**：一律 **zstd**。`M1-IO-004` 的「导出压缩码表」产出 `.lex.zst`，不再产出 `.lex.lzma`。
- 不引入 `xz2`（绑定 C 的 liblzma，与 D13 同类理由）。

**代价**：导出的压缩包不能被原版 WubiLex 读取。可接受——导出是给本工具和现代解压器用的，且未压缩的 `.lex` / 文本格式仍是主要交换格式。

**回写**：`02` §10 的 `R37` 缓解列更新为本决策。

### D15 — 系统副作用：`SystemOps` trait 双实现，而非运行期分支

**背景**：`02` §2.3 要求 `wubilex-winime` 提供 dry-run 模式，但没说怎么实现。若用 `if dry_run { … } else { … }` 散布在每个调用点，dry-run 与真实路径会各自演化，测试就失去意义。

**决策**：把「有副作用的系统调用」收拢为一个 `SystemOps` trait（停/起服务、结束进程、改文件所有权与 ACL、写注册表、启停 TIP）。两个实现：

| 实现 | 行为 |
|---|---|
| `Win32SystemOps` | 真实调用 Win32 / COM |
| `RecordingSystemOps` | 不改系统，把调用序列记进 `Vec<Op>` |

**编排逻辑（停机窗口、RAII 恢复守卫）只写一次**，泛型于 `SystemOps`。测试对 `RecordingSystemOps` 断言操作序列，包括**恐慌路径下守卫是否产生了完整的恢复序列**——这正是 `R1`（最高风险）唯一能在 CI 里验证的方式。

### D16 — 占位状态由构建期开关双端驱动

**背景**：`UX-INTERACT-013` 要求未实现功能显示规范占位；`R44` 是「占位残留到已实现功能」。

**决策**：单一开关源为 `src-tauri` 的 Cargo feature（每个模块一个，如 `feat-m1-install`），构建时导出为：

- Rust 侧：`#[cfg(feature = …)]` 决定 command 是否注册
- 前端侧：由 `app_features` command 在启动时一次性拉取，写入 Zustand 的 `features` store

前端**不读 Vite `define` 常量**——否则开关有两个源，又会漂移。占位组件只认 store。

**代价**：占位判断是运行期而非编译期，未实现功能的前端代码不会被 tree-shake。相对于「占位状态不一致」的用户可见缺陷，这个体积代价可接受。

### D17 — 工具链：Volta pin Node 与 pnpm，Rust 走 `rust-toolchain.toml`

**背景**：开发机已用 Volta 管理 Node（实测 Volta 2.0.2 / Node 24.18.1 / pnpm 11.18.0 / Rust 1.97.1 stable-msvc）。工具链版本若不固定，Tauri 构建产物会随机器漂移，而这类问题往往在发布前才暴露。

**决策**：

| 维度 | 定案 |
|---|---|
| Node 版本源 | `package.json` 的 `volta.node` 字段（`volta pin node@24.18.1`） |
| 包管理器 | **pnpm**，版本同由 `volta.pnpm` 固定（`volta pin pnpm@11.18.0`） |
| Rust 版本源 | `rust-toolchain.toml`：`channel = "1.97"`、`components = ["rustfmt", "clippy"]`、`targets = ["x86_64-pc-windows-msvc"]` |
| CI 版本来源 | `volta-cli/action` 读同一个 `volta` 字段 —— 本地与 CI **同源**，不在 workflow 里硬编码版本 |

**明确不用**：

- `.nvmrc` —— 与 Volta 重复，两处版本必然漂移
- `packageManager` 字段 + **corepack** —— corepack 与 Volta 都会接管 `pnpm` shim，同时启用等于给自己造一个「哪个版本在跑」的谜题。**选 Volta，就不写 `packageManager`**
- `npm` / `yarn` 的任何命令形态（文档、脚本、CI 一律 pnpm）

**Node 版本选 24.18.1 而非已装的 24.19.0**：与当前 Volta default 一致，开工日零摩擦。升级路径是重跑 `volta pin`，`package.json` 的 diff 即变更记录。

**代价 / 已知风险**：实测 `volta list all` 显示 pnpm 当前是以 **volta 全局 package**（`volta install pnpm`）身份存在（shim: `E:\env\Volta\pnpm`），而项目 pin 会在 **package-manager 槽位**再注册一份。两者并存时生效的是哪一个取决于 Volta 的解析顺序——这类问题只在换机器或新同事拉仓库时暴露。

**处置**：不在本任务动全局环境。在 `02` §10 登记为 `R55`，缓解措施是 S0 开工第一步验证：项目目录内 `pnpm --version` 必须等于 `volta.pnpm` 的 pin 值；不等则 `volta uninstall pnpm` 让项目 pin 成为唯一来源。

**待 S0 核对**：pnpm 11 的配置落点（`.npmrc` vs `pnpm-workspace.yaml`）—— pnpm 10 起部分设置迁往 `pnpm-workspace.yaml`，11 的具体边界需开工时以实际版本文档为准，本文不臆断。

## 2. crate 间接口缝（补 `02` 的空白）

分层表规定了「谁不能依赖谁」，但没说**跨层的必要交互怎么走**。以下五道缝是分层能否成立的关键。

| # | 缝 | 定义于 | 实现者 | 为什么必须抽象 |
|---:|---|---|---|---|
| S1 | `HttpClient` | `wubilex-resource` | `reqwest` 实现 + 测试用 mock | 离线跑单测；`R11`/`R12` 的恶意响应要能构造出来 |
| S2 | `SystemOps` | `wubilex-winime` | `Win32SystemOps` / `RecordingSystemOps` | 见 D15；`R1` 的恢复序列只能这样验证 |
| S3 | `PhraseSink` | `wubilex-core` | `wubilex-app` 注入 | `wubilex-learn` **禁止直写系统文件**（`M8-APPLY-004`）。core 定义出口，learn 只调用，物理写入由 app 编排 |
| S4 | `ResourceProvider` | `wubilex-core` | `wubilex-app`（转调 `wubilex-resource`） | core 需要拆字表/词频文件，但**不能依赖 resource**（分层表禁止 core 碰网络）。core 声明「我要一份拆字表」，app 决定从内置资源还是缓存还是下载拿 |
| S5 | 进度与取消 | 各 crate 各自定义 | `wubilex-app` 适配为 Tauri 事件 | 见下 |

**S5 的具体形态**（这里不用 trait）：

- **进度**：下层函数收 `&mut dyn FnMut(P)`，`P` 是该 crate 自己的进度枚举。不为「共享进度类型」新开一个 crate——那会把一个纯粹的适配问题变成结构问题。
- **取消**：下层收 `&AtomicBool`（或 `&dyn Fn() -> bool`）。`wubilex-codec` / `wubilex-core` **禁止依赖 `tokio`**，所以不能用 `CancellationToken`；由 `wubilex-app` 在 `CancellationToken` 与 `AtomicBool` 之间桥接。

> 这条约束的实际后果：`wubilex-codec` 与 `wubilex-core` 是纯同步的。长任务的异步化由 `wubilex-app` 用 `spawn_blocking` 承担。这是刻意的——纯同步的库层才能被 `rayon` 直接并行（`更优实现 #18`）。

## 3. 构建与打包流水线

### 3.1 `xtask` 职责

| 子命令 | 做什么 | 为什么在 xtask 而非脚本 |
|---|---|---|
| `xtask resources` | 拉取、校验、打包内置资源（86/98 字根图、拆字表、字根字体）到 `src-tauri/resources/` | 与 `D5`「核心资源内置」直接绑定，需要跨平台一致的 SHA-256 校验 |
| `xtask fixtures` | 拉取 8 方案真实码表回归集到 `crates/wubilex-codec/tests/fixtures/` | 二进制不入库，但测试必须可复现 |
| `xtask licenses` | 跑 `cargo-about` 生成许可声明页 | `M5-ABOUT-008` / `R35` |
| `xtask bindings` | 触发 `tauri-specta` 导出并比对 `src/types/generated/` 是否过期 | D11 的 CI 闸门 |
| `xtask check-docs` | 跑 `requirement-id-conventions.md` §3/§4/§5 的三组校验 | 文档不变量没有编译器守护 |

**不进 xtask**：日常构建（`cargo` / `tauri` CLI / `vite` 各司其职）、发布签名与产物上传（CI 的职责，涉及密钥）。

### 3.2 CI 闸门（`NFR-MAINT-006`）

Node 与 pnpm 版本由 `volta-cli/action` 从 `package.json` 的 `volta` 字段读取；Rust 由 `rust-toolchain.toml` 决定。**workflow 里不出现任何硬编码版本号**（D17）。

`fmt` → `clippy -D warnings` → `test` → `cargo deny`（许可与漏洞）→ `xtask bindings --check` → `xtask check-docs` → 前端 `pnpm install --frozen-lockfile` + `tsc --noEmit` + `eslint` + `vitest`。

## 4. 目录结构定案

> 这棵树同时是 `02` §9 的定案内容与本任务实际创建的目录。两者**必须逐行一致**（验收项 C1）。
> 本任务只建目录与 `README.md` / `.gitkeep`；标 `※` 的文件到 S0/S1 才产生，此处不创建。

```
wubi-lex-tool/
├── docs/                             需求文档集（已存在，本任务修订 02 与 21）
├── wubi-lex/                         原项目只读历史快照（已存在，不动）
├── Cargo.toml                     ※  virtual workspace（D10）
├── rust-toolchain.toml            ※  channel 1.97 + rustfmt/clippy + msvc target（D17）
├── package.json                   ※  含 volta.node / volta.pnpm pin；**无** packageManager 字段（D17）
├── pnpm-lock.yaml                 ※  入库
├── vite.config.ts                 ※  含 @tailwindcss/vite 插件（D9）
├── tsconfig.json                  ※
├── crates/
│   ├── wubilex-codec/                字节 ↔ 内存模型 · 纯逻辑 · 覆盖率目标 90%+
│   │   ├── README.md
│   │   ├── src/
│   │   │   ├── lex/                  .lex 二进制读写（契约 C1）
│   │   │   ├── eudp/                 EUDP 二进制读写（契约 C2）
│   │   │   ├── text/                 文本码表 6 方言 + 短语方言（契约 C5/C6）
│   │   │   ├── weight/               词频文件（契约 C12）
│   │   │   ├── split_table/          拆字数据表（契约 C12）
│   │   │   ├── detect/               码表版本探测（契约 C9，含 XFXY 缺陷修复）
│   │   │   └── escape/               空白字符转义（契约 C10）
│   │   └── tests/
│   │       └── fixtures/             8 方案真实码表回归集（由 xtask fixtures 拉取）
│   ├── wubilex-core/                 领域模型与变换 · 纯逻辑 · 同步
│   │   ├── README.md
│   │   └── src/
│   │       ├── table/                码表模型 + 倒排索引（M1-PARSE-017 / R5）
│   │       ├── phrase/               短语模型（M2）
│   │       ├── transform/            格式转换（M1-XFORM-*）
│   │       ├── slim/                 精简（M1-SLIM-*）
│   │       ├── weight/               词频与权重优化（M1-WEIGHT-*）
│   │       ├── coin/                 造词三规则 + 空码造词（M1-COIN-* / 契约 C7）
│   │       ├── split/                短语分离 + 键名占用判定（M1-SPLIT-* / 契约 C8）
│   │       ├── lookup/               编码反查与拆字组合（M3-QUERY / M3-SPLIT）
│   │       ├── convert/              简繁 / 拼音 / GB2312 判定（D13）
│   │       └── ports/                S3 PhraseSink · S4 ResourceProvider
│   ├── wubilex-winime/               Windows 输入法集成 · 唯一可调 Win32/COM 的业务 crate
│   │   ├── README.md
│   │   └── src/
│   │       ├── sysops/               S2 SystemOps trait + Win32 实现 + Recording 实现（D15）
│   │       ├── tip/                  输入法启停与状态（M4-TIP-*）
│   │       ├── double_pinyin/        双拼方案（M4-DPY-* / R25）
│   │       ├── settings/             注册表设置（M4-REG-* / 契约 C3）
│   │       ├── tsf/                  停机窗口编排 + RAII 恢复守卫（M4-TSF-* / R1）
│   │       ├── service/              服务控制（Win32 SCM，非命令行 / R10）
│   │       ├── schtask/              计划任务（Task Scheduler COM）
│   │       └── acl/                  所有权与 ACL 接管（SetNamedSecurityInfo / R9）
│   ├── wubilex-resource/             网络 · 解压 · 校验 · 缓存
│   │   ├── README.md
│   │   └── src/
│   │       ├── http/                 S1 HttpClient trait + reqwest 实现 + mock
│   │       ├── catalog/              在线码表目录（M6-CATALOG-* / R11 严格反序列化）
│   │       ├── download/             下载/进度/取消/续传（M6-DOWN-*）
│   │       ├── archive/              LZMA 解 · zstd 读写 · TAR + 路径穿越防护（D14 / R12）
│   │       ├── cache/                缓存目录与清理（M6-CACHE-* / R27）
│   │       └── verify/               SHA-256 校验（M6-DOWN-011）
│   └── wubilex-learn/                M8 自学习 · P2 · 阶段 S8 · 暂不入 members（D10）
│       ├── README.md
│       └── src/
│           ├── corpus/               路径 A：语料导入（M8-CORPUS-*）
│           ├── capture/              路径 B：UIA 输入采集（M8-CAPTURE-* / R45/R46）
│           ├── segment/              分词 + N-gram 新词发现（M8-LEARN-001）
│           ├── store/                候选池与频次（M8-MANAGE-*）
│           └── apply/                造码与入库决策（M8-APPLY-*，经 S3 PhraseSink）
├── src-tauri/                        wubilex-app · 薄适配层，禁止写领域逻辑
│   ├── README.md
│   ├── Cargo.toml                 ※
│   ├── tauri.conf.json            ※
│   ├── build.rs                   ※
│   ├── capabilities/                 Tauri v2 权限声明
│   ├── icons/                        应用图标
│   ├── resources/                    随包内置资源（D5）
│   │   ├── etymon/                   86/98 字根图
│   │   ├── split-table/              86/98 拆字数据表
│   │   └── fonts/                    字根 PUA 字体
│   └── src/
│       ├── commands/                 按 02 §3.2 的 command 前缀分目录
│       │   ├── lex/                  lex_*                        （M1）
│       │   ├── phrase/               phrase_*                     （M2）
│       │   ├── spelling/             spelling_*                   （M3）
│       │   ├── ime/                  ime_* dpy_* wubi_settings_* tsf_*（M4）
│       │   ├── etymon/               etymon_* help_* about_*      （M5）
│       │   ├── resource/             resource_* update_*          （M6）
│       │   ├── app/                  app_* config_* window_* hotkey_* keymap_* task_*（M7）
│       │   └── learn/                learn_*                      （M8）
│       ├── state/                    AppState（含 lex_sessions，D1）
│       ├── events/                   事件总线 `<域>://<事件名>`（M7-BUS-*）
│       ├── task/                     任务注册表 + 取消 + 进度桥接（S5 / M7-TASK-*）
│       ├── keymap/                   动作注册表 + 绑定解析（M7-KEYMAP-*）
│       ├── config/                   TOML 配置 · schema 版本 · 损坏回退（D12 / R31）
│       ├── error/                    AppError 统一错误模型（02 §3.3）
│       ├── features/                 模块特性开关（D16 / UX-INTERACT-013）
│       ├── recovery/                 崩溃后自恢复（R1 / M7-INST-006）
│       └── bindings/                 tauri-specta 导出入口（D11）
├── src/                              前端 · React 19 + TypeScript
│   ├── app/
│   │   ├── layout/                   应用栏 / 侧栏 / 状态栏（UX-IA-005/006）
│   │   ├── providers/                主题 / i18n / 查询 / 快捷键
│   │   └── router/                   路由表与深链接（UX-IA-010）
│   ├── routes/                       7 个一级领域，对应 UX-IA-001
│   │   ├── overview/                 概览
│   │   ├── lexicons/                 码表（库与编辑器是同页两态，UX-IA-004）
│   │   ├── phrases/                  短语
│   │   ├── lookup/                   反查
│   │   ├── radicals/                 字根
│   │   ├── learning/                 学习（S8 前为占位）
│   │   └── settings/                 设置（含快捷键子页）
│   ├── components/
│   │   ├── ui/                       shadcn/ui（Tailwind v4 分支）
│   │   ├── virtual-table/            UX-COMP-001 · TanStack Virtual
│   │   ├── wubi-keyboard/            UX-COMP-002
│   │   ├── key-cap/                  UX-COMP-003
│   │   ├── hotkey-recorder/          UX-COMP-004
│   │   ├── scheme-badge/             UX-COMP-005
│   │   ├── command-palette/          UX-IA-007
│   │   ├── task-progress/            UX-INTERACT-001
│   │   └── feature-placeholder/      UX-COMP-015 / D16
│   ├── stores/                       Zustand（含 features store）
│   ├── lib/                          IPC 封装 · 工具函数
│   ├── styles/
│   │   └── theme.css              ※  Tailwind v4 令牌唯一来源（D9）
│   ├── types/
│   │   └── generated/                tauri-specta 产物，入库（D11）
│   ├── i18n/                         i18next 资源
│   └── icons/                        Lucide 统一出口（UX-COMP-016）
├── xtask/                            构建工具（§3.1）
│   └── src/
└── .github/
    └── workflows/                    CI（§3.2）
```

### 4.1 与原「建议」结构的差异及理由

| 变更 | 原 `02` §9 | 定案 | 理由 |
|---|---|---|---|
| 前端路由 | 按模块号 `lex/ phrase/ spelling/ settings/ help/ learn/` | 按 7 个领域 `overview/ lexicons/ phrases/ lookup/ radicals/ learning/ settings/` | `UX-IA-001` 定的是领域导航，目录必须与之对齐；原结构缺 `overview`、缺 `radicals`，且残留了已被移除的 `help` |
| Tailwind | `tailwind.config.ts` + `styles/tokens.css` | 仅 `styles/theme.css` | D9 |
| 端口/缝 | 无 | `core/src/ports/`、`winime/src/sysops/`、`resource/src/http/` | §2 的五道缝需要有物理落点 |
| 类型契约 | 无 | `src/types/generated/`、`src-tauri/src/bindings/` | D11 |
| 配置 | 未在树中出现 | `src-tauri/src/config/` | D12 定案自建 |
| Tauri v2 必需项 | 无 | `capabilities/`、`build.rs`、`icons/` | Tauri v2 权限模型的强制目录 |
| 单文件模块 | `weight.rs`、`escape.rs`、`detect.rs` 等 | 统一为目录 | 这几个都要带各自的测试与 fixture，单文件必然要拆；一开始就用目录省一次重构 |
| 工具链 | 未在树中出现 | `rust-toolchain.toml`、`package.json`（volta pin）、`pnpm-lock.yaml` | D17 |

### 4.2 占位文件规则

- 每个 crate 根：`README.md`，内容为**职责 / 允许依赖 / 禁止依赖 / 对应需求域 / 所属阶段**五段（验收项 C4）。
- 所有叶子目录：`.gitkeep`。
- **不创建**任何 `Cargo.toml` / `package.json` / `tauri.conf.json` / `*.rs` / `*.ts`——本次交付明确为「不可 build 的骨架」。

## 5. 文档改动映射

| 文件 | 改什么 |
|---|---|
| `docs/02-architecture.md` §1 | 架构总览图的前端层标注改为 `React 19 + TypeScript + Tailwind v4 + Lucide` |
| `docs/02-architecture.md` §2 | 各 crate 小节末尾补 §2 的 trait 缝落点 |
| `docs/02-architecture.md` **新增 §3.5** | IPC 类型生成与契约同步（D11） |
| `docs/02-architecture.md` §5.1 | 加版本基线列；按 D12/D13/D14 收敛岔路 |
| `docs/02-architecture.md` §5.2 | 「选型注意」重写为「已关闭的岔路 + 备选与切换条件」 |
| `docs/02-architecture.md` §6 | 标题改「前端技术栈（定案）」；删除「建议」与「Vue / Svelte 同样可行」；加版本基线 |
| `docs/02-architecture.md` §7 | 追加 D8–D17 |
| `docs/02-architecture.md` **新增 §8.5** | 构建与打包流水线 + xtask 职责 + **工具链固定（D17）**（§3） |
| `docs/02-architecture.md` §9 | 「目录结构建议」→「目录结构（定案）」，替换为 §4 的树 + §4.1 差异表 |
| `docs/02-architecture.md` §10 | `R33` / `R37` 缓解列回写；新增 `R53`（Tailwind v4 生态成熟度）、`R54`（tauri-specta 版本跟随）、`R55`（pnpm 双身份的 shim 解析歧义） |
| `docs/02-architecture.md` 目录 | 同步新增章节 |
| `docs/21-ui-ux.md` §4.6 | 整节改写为 v4 CSS-first（D9 的映射表 + `@theme inline` 示例） |
| `docs/21-ui-ux.md` `UX-TOKEN-011` | 描述改为 v4 机制，**ID 与 P0 不变** |
| `docs/00-overview.md` | 技术栈行补 React 19 / Tailwind v4 |
| `docs/README.md` | 首行技术栈补版本 |
| `docs/22-roadmap.md` S0 | 「workspace 与 6 个 crate 骨架」补注：目录结构已由本任务定案并创建 |

## 6. 风险

| # | 风险 | 缓解 |
|---:|---|---|
| T1 | 目录树写进文档后与实际创建的不一致（验收项 C1 靠人眼比对易漏） | 用 `git ls-files` + 从文档提取树，脚本化比对（见 `implement.md` 校验步） |
| T2 | 改 `21` §4.6 触碰需求表，误伤 630 计数 | §4.6 内是「约定表」不是需求表；唯一被改的需求行是 `UX-TOKEN-011`，改描述不改 ID。改后立即重跑三组校验 |
| T3 | 版本基线写成「锁定」，S0 照抄导致依赖冲突 | 表头明写「2026-08-21 实测基线，S0 开工须重新核对」；未实测项显式标注 |
| T4 | D8–D16 编号与既有 D1–D7 冲突或跳号 | 写入前 grep `^### D[0-9]` 确认现存最大编号（已实测：止于 D7） |
| T5 | 新增 `R53`/`R54`/`R55` 与既有 R1–R52 冲突 | 同上，grep `^| R[0-9]` 确认（已实测：止于 R52） |
| T6 | 工具链版本写死在文档里，S0 开工时已过期 | 与 `T3` 同处置：标为「2026-08-21 实测基线」；D17 的 pin 值是**起点**，`volta pin` 的 diff 才是权威变更记录 |
