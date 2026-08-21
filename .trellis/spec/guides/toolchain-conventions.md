# 工具链口径

> **适用范围**：任何涉及 Node、包管理器、Rust 版本的命令、脚本、CI 配置与文档。
> **为什么单列一份**：工具链有两个来源就等于没有来源。这类问题不在本机复现，只在换机器、新同事拉仓库、或 CI 上暴露。

定案见 `docs/02-architecture.md` §8.5 与决策 D17。

---

## 1. 版本从哪来

| 项 | 来源 | 命令 |
|---|---|---|
| Node | `package.json` 的 `volta.node` | `volta pin node@<ver>` |
| 包管理器 | `package.json` 的 `volta.pnpm` | `volta pin pnpm@<ver>` |
| Rust | `rust-toolchain.toml` | 直接编辑 |
| CI | **同上两处**，由 `volta-cli/action` 与 `rust-toolchain.toml` 读取 | workflow 里不写版本号 |

基线（2026-08-21 实测开发机）：Volta 2.0.2 · Node 24.18.1 · pnpm 11.18.0 · Rust 1.97.1 stable-x86_64-pc-windows-msvc。

---

## 2. 三条「不用」

| 不用 | 为什么 |
|---|---|
| `.nvmrc` | 与 Volta 的 `volta.node` 重复。两处版本必然漂移，且没有任何机制会提醒你 |
| `packageManager` 字段 + corepack | corepack 与 Volta **都会接管 `pnpm` shim**。同时启用后无法确定实际跑的是哪个版本 —— 选了 Volta，就不写 `packageManager` |
| `npm` / `yarn` | 包管理器只有 pnpm 一个。文档、脚本、CI、README 一律用 `pnpm` |

---

## 3. 已知陷阱：pnpm 的双身份

pnpm 在 Volta 下有两种可能的身份：

| 身份 | 装法 | 表现 |
|---|---|---|
| 全局 package | `volta install pnpm` | 生成一个全局 `pnpm` shim |
| 项目 package-manager | `volta pin pnpm@x` | 写进 `package.json` 的 `volta.pnpm` |

**两者可以并存，此时哪个生效取决于 Volta 的解析顺序**（已登记为 `R55`）。当前开发机上 pnpm 是**全局 package** 身份（shim 在 `E:\env\Volta\pnpm`）。

**开工前必查**：

```bash
cd <项目根>
pnpm --version                                   # 必须等于 package.json 的 volta.pnpm
node -e "console.log(require('./package.json').volta)"
```

不相等 → `volta uninstall pnpm`，让项目 pin 成为唯一来源。

---

## 4. 未定项

**pnpm 11 的配置落点**（`.npmrc` vs `pnpm-workspace.yaml`）在 S0 开工时以实际版本的官方文档为准。pnpm 10 起部分设置迁往 `pnpm-workspace.yaml`，11 的边界不要靠记忆或旧资料判断。

---

## 5. 检查清单

- [ ] 改了 Node / pnpm 版本 → 用 `volta pin`，不手改 `package.json`
- [ ] 写脚本或 CI → 命令用 `pnpm`，版本不硬编码
- [ ] 新增 workflow → 版本从 `volta` 字段与 `rust-toolchain.toml` 取
- [ ] 没有引入 `.nvmrc` / `packageManager` / corepack
- [ ] 换机器或新同事上手 → 先跑 §3 的双身份检查
