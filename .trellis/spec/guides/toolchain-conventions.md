# 工具链口径

> **适用范围**：任何涉及 Node、pnpm、Rust、依赖安装、构建脚本或 CI 的配置与命令。
> **定案来源**：`docs/02-architecture.md` D17 与 §8.5；pnpm 直接使用全局命令，不接入 Volta 的项目级 pnpm pin 或 corepack。

## 1. Scope / Trigger

新增或修改下列任一内容前都必须应用本合同：

- `package.json`、`pnpm-workspace.yaml`、`pnpm-lock.yaml`
- `Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`
- Node/pnpm/Rust 安装说明、脚本或 CI workflow
- 前端依赖版本和 Rust workspace member

版本合同各有一个仓库来源：Node 来自 `package.json.volta.node`，全局 pnpm 的期望版本来自 `package.json.engines.pnpm`，Rust 来自 `rust-toolchain.toml`。Cargo manifest 和 workflow 不重复硬编码这些版本。

## 2. Signatures

项目使用以下命令，不为 pnpm 设置 Volta feature flag：

```powershell
volta pin node@24.18.1

node --version
pnpm --version
cargo --version
pnpm install --frozen-lockfile
```

`pnpm` 必须直接解析到机器已有的全局安装。`Get-Command pnpm` 可以指向 Volta 已安装的 global-package shim；这仍是全局命令，不等同于项目 pin。仓库不执行 `volta pin pnpm`、不写 `volta.pnpm`、不设置 `VOLTA_FEATURE_PNPM`，也不使用 corepack。

## 3. Contracts

| Item | Contract |
|---|---|
| `package.json.volta.node` | `24.18.1`；由 `volta pin node@24.18.1` 写入 |
| `package.json.engines.pnpm` | `11.18.0`；声明全局 pnpm 的唯一期望版本，不负责安装它 |
| `rust-toolchain.toml` | `1.97.1`、minimal profile、`rustfmt`、`clippy`、`x86_64-pc-windows-msvc` |
| `pnpm-workspace.yaml` | pnpm 11 项目设置入口；根目录是唯一前端包时用空 mapping `{}`，省略 `packages` |
| `.npmrc` | 只用于 authentication 与 registry；没有这两类设置时不创建 |
| Lockfiles | `Cargo.lock` 与 `pnpm-lock.yaml` 入库；frozen install 后内容不得变化 |

明确禁止：`.nvmrc`、`packageManager` + corepack、`package.json.volta.pnpm`、`VOLTA_FEATURE_PNPM`、npm/yarn/npx 项目命令、Cargo/workflow 中的重复工具链版本。

pnpm 依赖发布时间保护拒绝刚发布的 patch 时，优先选择仍兼容且已过保护窗口的版本。不得只为追最新 patch 永久加入 `minimumReleaseAgeExclude`；若业务或安全修复必须使用该精确版本，例外必须精确到 package + version，并在升级后重新评估。

## 4. Validation & Error Matrix

| Condition | Required response |
|---|---|
| `pnpm --version` 与 `engines.pnpm` 不同 | 停止生成 lockfile；报告全局 pnpm 路径和版本，由用户决定是否调整全局安装 |
| `pnpm` 路径位于 Volta 的 global-package 目录 | 若仓库没有 `volta.pnpm` 且未设置 `VOLTA_FEATURE_PNPM`，按全局 pnpm 接受；不要因此改动用户环境 |
| 项目配置出现 `volta.pnpm`、`VOLTA_FEATURE_PNPM` 或 corepack | 停止；删除项目级 pnpm 接管机制，恢复为直接使用全局 pnpm |
| `pnpm install --frozen-lockfile` 报 release-age 限制 | 选择已过窗口的兼容版本；只有精确版本不可替代时才评审临时 exclude |
| frozen install 改写 lockfile | 视为失败；用符合 `engines.pnpm` 的全局 pnpm 重新生成并复验 |
| `cargo --version` 与 toolchain 不同 | 检查当前目录和 `rust-toolchain.toml`，不在 Cargo manifest 或 CI 再加版本 |
| 新机器缺少 pnpm | 报告缺失前置条件；不得擅自添加 Volta 项目 pin、corepack、npm 或 yarn |

工具不得自动安装、卸载或改写用户的全局 pnpm。确需调整全局工具时，先把证据和影响交给用户确认。

## 5. Good / Base / Bad Cases

- **Good**：全局 `pnpm --version` 等于 `engines.pnpm`，frozen install 不改锁文件。
- **Base**：仓库只有根前端包，`pnpm-workspace.yaml` 内容为 `{}`，没有 `.npmrc`。
- **Bad**：添加 `volta.pnpm`、`VOLTA_FEATURE_PNPM`、`packageManager` 或 corepack；忽略全局 pnpm 版本漂移；为刚发布依赖留下无期限 release-age 例外；自动修改用户全局 pnpm。

## 6. Tests Required

每次工具链或依赖变更至少断言：

1. `node --version`、`pnpm --version`、`cargo --version` 等于批准版本。
2. `volta which node` 能解析；记录 `Get-Command pnpm` / `command -v pnpm` 的全局命令路径，允许现有 Volta global-package shim。
3. `package.json` 只有 `volta.node` 与 `engines.pnpm`，没有 `volta.pnpm` 或 `packageManager`。
4. `pnpm install --frozen-lockfile` 返回 0，且安装前后 `pnpm-lock.yaml` SHA-256 相同。
5. `cargo metadata --no-deps` 的 workspace member 集合符合 D10。
6. Rust fmt/Clippy/tests 与 TypeScript/ESLint/Vitest 全部通过。
7. 搜索确认没有 `.nvmrc`、corepack、npm/yarn/npx 项目命令或重复 Rust 版本源。

## 7. Wrong vs Correct

错误：为项目添加 Volta pnpm pin 或 corepack，制造第二套 shim。

```powershell
# Wrong
$env:VOLTA_FEATURE_PNPM = "1"
volta pin pnpm@11.18.0
corepack enable
```

正确：直接验证全局 pnpm，并从仓库合同读取期望值。

```powershell
# Correct
pnpm --version
node -e "console.log(require('./package.json').engines.pnpm)"
```
