# Design - S0-01 workspace 与工具链

## 1. Boundary

本任务建立两个可独立检查、共享固定版本源的工程入口：

```text
rust-toolchain.toml -> virtual Cargo workspace -> 4 library shells + src-tauri + xtask
global pnpm + package.json.engines.pnpm -> pnpm lockfile -> TS / ESLint / Vitest / Vite config
```

两条链在 S0-01 只需编译和静态检查，不连接为可启动产品。没有 `src-tauri/src/main.rs`、`index.html` 或 `src/main.tsx`，因此不会抢占 S1 的“第一个可运行应用”里程碑。

## 2. Rust Workspace

根 `Cargo.toml` 使用 resolver `3` 和 edition `2024` 的 workspace 级基线。members 显式列出六项，不使用会把 `wubilex-learn` 意外纳入的 glob；manifest 不重复声明 `rust-version`，工具链版本只由 `rust-toolchain.toml` 固定。

| Member | S0-01 shape | Later owner |
|---|---|---|
| `wubilex-codec` | manifest + empty documented library root | S0 codec children |
| `wubilex-core` | manifest + empty documented library root | S2 / S4 |
| `wubilex-winime` | manifest + empty documented library root | S0 spikes / S3 |
| `wubilex-resource` | manifest + empty documented library root | S5 |
| `wubilex-app` (`src-tauri`) | Tauri v2 build/config + compile-only library root | S1 onward |
| `xtask` | manifest + no-op binary entry | `s0-xtask-ci` |

`rust-toolchain.toml` 固定 `channel = "1.97.1"`、`profile = "minimal"`、`components = ["rustfmt", "clippy"]`、`targets = ["x86_64-pc-windows-msvc"]`。crate 壳不添加业务依赖；Tauri shell 只声明自身编译所需的 `tauri` 与 `tauri-build`。

`src-tauri/build.rs` 调用 Tauri build helper，`tauri.conf.json` 只提供 schema、产品名、版本和 identifier 等合法最小元数据。Windows 上的 `tauri-build` 即使不启用 bundle 也强制生成 executable resource，因此工程在标准 `src-tauri/icons/icon.ico` 路径保留任务自有的编译占位图标；配置不再跨目录引用只读历史快照，S1 可在产品视觉定案后替换该资源。配置不引用尚不存在的 dev/build 脚本或 frontend distribution。library root 不创建 `Builder`、command 或 run 函数。

## 3. Frontend Tooling

`package.json` 是私有根包：`volta.node = "24.18.1"` 固定 Node，`engines.pnpm = "11.18.0"` 校验全局 pnpm；不写 `volta.pnpm` 或 `packageManager`。脚本只提供：

- `typecheck`：严格 TypeScript 无输出检查；
- `lint`：ESLint flat config；
- `test`：Vitest，允许当前零测试但不掩盖未来失败测试。

依赖采用已核对的兼容基线：Tauri `2.11.x`、React `19.2.x`、Vite `8.2.x`、Tailwind `4.3.x`、Vitest `4.1.x`、ESLint `10.x`、`typescript-eslint` `8.67.x`、TypeScript `6.0.3`。具体 patch 版本在 lockfile 中固定，依赖声明使用任务实施时已核对的版本。

`pnpm-workspace.yaml` 声明 pnpm workspace root，并使用空映射省略不需要的 `packages` 与其他项目策略；不创建 `.npmrc`。ESLint 选择与现有依赖兼容、且已过 pnpm 发布时间保护窗口的 `10.8.1`，不为新发布版本增加长期供应链例外。`vite.config.ts` 只接入 `@tailwindcss/vite`，不创建 Tailwind v3 config 或 PostCSS 链。TypeScript 至少包含一个非产品声明文件，使严格检查有真实输入；不建立 HTML、React mount 或 CSS token 实现。

## 4. Validation Contract

Rust 验证按 fmt -> clippy (`-D warnings`) -> test 执行。前端先由符合 `engines.pnpm` 的全局 pnpm 生成 lockfile，再用 frozen install 复验，最后执行 typecheck -> lint -> Vitest。`cargo metadata` 单独验证成员集合，文本搜索验证禁用版本源与被推迟的产品入口没有混入。

`cargo-deny`、`cargo-llvm-cov` 和真实 `cargo xtask` 命令当前机器未安装或尚未实现，不在本子任务伪造绿色结果；它们由 `s0-xtask-ci` 在已有稳定本地命令后接入。

## 5. Compatibility And Trade-Offs

- TypeScript 不选 registry 最新的 `7.0.2`，因为当前 `typescript-eslint@8.67.0` peer range 要求 `<6.1.0`；选择 `6.0.3` 保持官方 Tauri React 模板方向和 lint 兼容性。
- pnpm 直接使用机器上的全局命令；该命令由既有 Volta global package 提供也可接受。仓库用 `engines.pnpm` 校验期望版本，不启用 Volta 的项目级 pnpm pin，也不自动修改用户全局环境。
- pnpm 的发布时间策略会拒绝刚发布的依赖版本；本任务选择已过保护窗口的兼容版本，不在仓库中留下 `minimumReleaseAgeExclude` 例外。
- 前端依赖可以在无 UI 源码时先落锁，以固定后续 S1 的工具链兼容面；不借此建立组件或目录内代码模式。
- compile-only Tauri shell 不能验证窗口或 WebView，但能提前暴露 manifest、build script、MSVC target 和 Tauri 配置错误；运行期合同由 S1 验证。
- Windows resource 占位图标只满足 `tauri-build` 的编译约束，不作为产品图标定案；S1 建立可运行应用时负责确认或替换品牌资源。
- 零测试通过只适用于当前空壳。任何后续测试文件失败时 Vitest 仍必须返回失败，不能用 blanket ignore 规则。

## 6. Rollback

工程壳文件彼此无数据迁移。若 Rust 或前端依赖组合不能通过检查，优先调整本任务新增的 manifest/config 并重新生成 lockfile；不改全局工具，不删除既有目录/README，不回退用户已有改动。Tauri 与前端配置可按文件组独立回退，不影响父任务和已归档的 S0-00 文档成果。
