# 工具链口径

> **适用范围**：任何涉及 Node、pnpm、Rust、依赖安装、构建脚本或 CI 的配置与命令。
> **定案来源**：`docs/02-architecture.md` D17 与 §8.5；Node 与 pnpm 都由 `package.json.volta` 固定，不接入 corepack 或第二份版本源。

## 1. Scope / Trigger

新增或修改下列任一内容前都必须应用本合同：

- `package.json`、`pnpm-workspace.yaml`、`pnpm-lock.yaml`
- `Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`
- Node/pnpm/Rust 安装说明、脚本或 CI workflow
- 前端依赖版本和 Rust workspace member

版本合同各有一个仓库来源：Node 来自 `package.json.volta.node`，pnpm 来自 `package.json.volta.pnpm`，Rust 来自 `rust-toolchain.toml`。Cargo manifest 和 workflow 不重复硬编码这些版本。

## 2. Signatures

项目使用以下命令。Volta 2.0.2 的 pnpm 支持仍属实验功能，因此 Windows 用户环境与 CI 必须显式设置 feature flag：

```powershell
$env:VOLTA_FEATURE_PNPM = [Environment]::GetEnvironmentVariable('VOLTA_FEATURE_PNPM', 'User')
if ($env:VOLTA_FEATURE_PNPM -ne '1') { throw 'VOLTA_FEATURE_PNPM must be 1' }

volta pin node@24.18.1
volta pin pnpm@11.18.0
node --version
pnpm --version
cargo --version
pnpm install --frozen-lockfile
```

日常命令直接调用 `pnpm`；在项目目录中，Volta 根据 `package.json.volta.pnpm` 解析固定版本。更改 pin 前必须从持久化用户环境把 `VOLTA_FEATURE_PNPM` 载入当前进程并校验为 `1`。CI 在 job 环境中显式设置同一值，让 `volta-cli/action` 同时准备 Node 与 pnpm。仓库不使用 corepack 或独立的 pnpm setup action。

## 3. Contracts

| Item | Contract |
|---|---|
| `package.json.volta.node` | `24.18.1`；由 `volta pin node@24.18.1` 写入 |
| `package.json.volta.pnpm` | `11.18.0`；由 `volta pin pnpm@11.18.0` 写入，是唯一 pnpm 版本源 |
| `VOLTA_FEATURE_PNPM` | Windows 用户环境与 CI 均为 `1`；只启用 Volta pnpm 解析能力，不构成版本源 |
| `rust-toolchain.toml` | `1.97.1`、minimal profile、`rustfmt`、`clippy`、`x86_64-pc-windows-msvc` |
| `pnpm-workspace.yaml` | pnpm 11 项目设置入口；根目录是唯一前端包时用空 mapping `{}`，省略 `packages` |
| `.npmrc` | 只用于 authentication 与 registry；没有这两类设置时不创建 |
| Lockfiles | `Cargo.lock` 与 `pnpm-lock.yaml` 入库；frozen install 后内容不得变化 |

明确禁止：`.nvmrc`、`engines.pnpm`、`packageManager` + corepack、npm/yarn/npx 项目命令、独立 pnpm setup action、Cargo/workflow 中的重复工具链版本。

pnpm 依赖发布时间保护拒绝刚发布的 patch 时，优先选择仍兼容且已过保护窗口的版本。不得只为追最新 patch 永久加入 `minimumReleaseAgeExclude`；若业务或安全修复必须使用该精确版本，例外必须精确到 package + version，并在升级后重新评估。

## 4. Validation & Error Matrix

| Condition | Required response |
|---|---|
| `VOLTA_FEATURE_PNPM` 缺失或不等于 `1` | 停止 Volta/pnpm 命令；报告必须持久化的用户环境前置，不改用其他包管理器 |
| `pnpm --version` 与 `volta.pnpm` 不同 | 停止生成 lockfile；检查当前进程 feature flag 与项目 pin，不修改用户级包管理器安装 |
| 项目配置出现 `engines.pnpm`、`packageManager` 或 corepack | 停止；删除竞争版本源或 shim，恢复为 `package.json.volta.pnpm` 单一来源 |
| `pnpm install --frozen-lockfile` 报 release-age 限制 | 选择已过窗口的兼容版本；只有精确版本不可替代时才评审临时 exclude |
| frozen install 改写 lockfile | 视为失败；确认项目解析的 pnpm 等于 `volta.pnpm` 后重新生成并复验 |
| `cargo --version` 与 toolchain 不同 | 检查当前目录和 `rust-toolchain.toml`，不在 Cargo manifest 或 CI 再加版本 |
| 新机器无法解析 pnpm | 检查 Volta、feature flag 与 manifest pin；不得擅自添加 corepack、npm、yarn 或第二份版本源 |

工具不得自动安装、卸载或改写用户级 pnpm。`volta pin` 可以按项目 manifest 补齐 Volta 工具缓存，但不得变更 Node pin、lockfile 或用户级包管理器配置。

## 5. Good / Base / Bad Cases

- **Good**：feature flag 为 `1`，项目目录中的 `pnpm --version` 等于 `volta.pnpm`，frozen install 不改锁文件。
- **Base**：仓库只有根前端包，`pnpm-workspace.yaml` 内容为 `{}`，没有 `.npmrc`。
- **Bad**：添加 `engines.pnpm`、`packageManager` 或 corepack；漏设 feature flag；忽略项目 pin 与实际 pnpm 的差异；为刚发布依赖留下无期限 release-age 例外；自动修改用户级安装。

## 6. Tests Required

每次工具链或依赖变更至少断言：

1. 从 Windows 用户环境加载的 `VOLTA_FEATURE_PNPM` 等于 `1`；CI 也显式设置该值。
2. `node --version`、`pnpm --version`、`cargo --version` 等于批准版本，`volta which node` 与 `volta which pnpm` 都能解析。
3. `package.json` 只有 `volta.node` 与 `volta.pnpm`，没有 `engines.pnpm` 或 `packageManager`。
4. `pnpm install --frozen-lockfile --force` 返回 0，且安装前后 `pnpm-lock.yaml` SHA-256 相同。
5. `cargo metadata --no-deps` 的 workspace member 集合符合 D10。
6. Rust fmt/Clippy/tests 与 TypeScript/ESLint/Vitest 全部通过。
7. 搜索确认没有 `.nvmrc`、corepack、npm/yarn/npx 项目命令或重复 Rust 版本源。

## 7. Wrong vs Correct

错误：用第二份字段或 corepack 重复接管 pnpm。

```powershell
# Wrong
node -e "console.log(require('./package.json').engines.pnpm)"
corepack enable
```

正确：加载已持久化的 feature flag，并验证项目 pin 与实际命令一致。

```powershell
# Correct
$env:VOLTA_FEATURE_PNPM = [Environment]::GetEnvironmentVariable('VOLTA_FEATURE_PNPM', 'User')
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ($env:VOLTA_FEATURE_PNPM -ne '1') { throw 'VOLTA_FEATURE_PNPM must be 1' }
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }
```
