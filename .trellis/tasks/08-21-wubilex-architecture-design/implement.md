# 执行计划 — WubiLexTool 架构定案与目录骨架

> 顺序有意义：**先建目录（步 2），再写 §9 目录树（步 4）**，让文档抄实际结构而不是反过来——这是验收项 C1 唯一不靠人眼的保证方式。

## 前置校验（开工前，建立基线）

```bash
# 1. 现存决策与风险的最大编号（design.md T4/T5）
grep -nE '^### D[0-9]+' docs/02-architecture.md | tail -3      # 期望止于 D7
grep -ohE '^\| R[0-9]+' docs/02-architecture.md | sort -V | tail -3   # 期望止于 R52

# 2. 需求 ID 计数基线（requirement-id-conventions.md §3）
M=$(grep -rhoE '\bM[1-8]-[A-Z0-9]+-[0-9]{3}\b' docs/modules/            | sort -u | wc -l)
N=$(grep -ohE  '\bNFR-[A-Z0-9]+-[0-9]{3}\b'    docs/20-nonfunctional.md | sort -u | wc -l)
U=$(grep -ohE  '\bUX-[A-Z0-9]+-[0-9]{3}\b'     docs/21-ui-ux.md         | sort -u | wc -l)
echo "模块=$M 非功能=$N UI/UX=$U 总计=$((M+N+U))"
# 期望：模块=414 非功能=101 UI/UX=115 总计=630
```

- [ ] P1 记录基线输出；若与期望不符，**停下来先查清楚**再动工（说明归档任务后有人改过文档）

## 步 1 — 建目录骨架

按 `design.md` §4 的树创建。只建目录 + `.gitkeep`，不建 README（README 在步 2 单独写，内容需与 §1 分层表对齐）。

- [ ] 1.1 `crates/wubilex-codec/`（7 个 src 子目录 + `tests/fixtures/`）
- [ ] 1.2 `crates/wubilex-core/`（10 个 src 子目录，含 `ports/`）
- [ ] 1.3 `crates/wubilex-winime/`（8 个 src 子目录，含 `sysops/`）
- [ ] 1.4 `crates/wubilex-resource/`（6 个 src 子目录，含 `http/`）
- [ ] 1.5 `crates/wubilex-learn/`（5 个 src 子目录）
- [ ] 1.6 `src-tauri/`（`capabilities/`、`icons/`、`resources/{etymon,split-table,fonts}/`、`src/` 下 8 个 commands 子目录 + 9 个平级子目录）
- [ ] 1.7 `src/`（`app/{layout,providers,router}/`、`routes/` 7 个领域、`components/` 9 项、`stores/`、`lib/`、`styles/`、`types/generated/`、`i18n/`、`icons/`）
- [ ] 1.8 `xtask/src/`
- [ ] 1.9 `.github/workflows/`

**校验**：

```bash
find crates src-tauri src xtask .github -type d | sort
git status --short | head -50    # .gitkeep 应全部为新增
```

- [ ] 1.10 目录数与 `design.md` §4 树逐项核对；**不得**出现 `Cargo.toml` / `package.json` / `*.rs` / `*.ts`

## 步 2 — 写 6 个 crate README

每份五段：**职责 / 允许依赖 / 禁止依赖 / 对应需求域 / 所属阶段**。「允许/禁止依赖」逐字取自 `docs/02-architecture.md` §1 分层原则表——不要重新措辞，两处漂移就失去了 README 的意义。

- [ ] 2.1 `crates/wubilex-codec/README.md`（含「覆盖率目标 90%+」与契约 C1/C2/C5/C6/C9/C10/C12 清单）
- [ ] 2.2 `crates/wubilex-core/README.md`（含 `ports/` 的 S3/S4 说明）
- [ ] 2.3 `crates/wubilex-winime/README.md`（含 D15 的 `SystemOps` 双实现与 dry-run 要求）
- [ ] 2.4 `crates/wubilex-resource/README.md`（含 S1 `HttpClient` 可 mock 要求）
- [ ] 2.5 `crates/wubilex-learn/README.md`（**显式标注**：P2 / 阶段 S8 / 暂不入 workspace members / 禁止直写系统文件）
- [ ] 2.6 `src-tauri/README.md`（**显式标注**：薄适配层，禁止写领域逻辑）

## 步 3 — 改 `docs/21-ui-ux.md`（Tailwind v4）

**先改 21 再改 02**：02 §6 要引用 21 §4.6，先让被引用方定型。

- [ ] 3.1 §4.6 整节改写：v3→v4 机制映射表 + `@theme inline` 完整示例（取 `design.md` D9）
- [ ] 3.2 `UX-TOKEN-011` 描述改为 v4 机制；**ID 不变、P0 不变、指向 §4.6 的链接不变**
- [ ] 3.3 全库扫描 v3 残留

```bash
grep -rn 'theme\.extend\|<alpha-value>\|tailwind\.config\|darkMode' docs/
# 期望：仅命中 21 §4.6 的「v3 原写法」对照列
```

**⛔ 评审闸门 G1**：跑完整三组校验，确认改需求行没有破坏不变量。

```bash
# 计数（应与 P1 基线一致）
M=$(grep -rhoE '\bM[1-8]-[A-Z0-9]+-[0-9]{3}\b' docs/modules/            | sort -u | wc -l)
N=$(grep -ohE  '\bNFR-[A-Z0-9]+-[0-9]{3}\b'    docs/20-nonfunctional.md | sort -u | wc -l)
U=$(grep -ohE  '\bUX-[A-Z0-9]+-[0-9]{3}\b'     docs/21-ui-ux.md         | sort -u | wc -l)
echo "模块=$M 非功能=$N UI/UX=$U 总计=$((M+N+U))"

# 单文件内定义行唯一性
for f in docs/modules/M*.md docs/20-nonfunctional.md docs/21-ui-ux.md; do
  d=$(grep -ohE '^\|[[:space:]]*`?\b[A-Z][A-Z0-9]*-[A-Z0-9]+-[0-9]{3}\b' "$f" \
      | grep -ohE '\b[A-Z][A-Z0-9]*-[A-Z0-9]+-[0-9]{3}\b' | sort | uniq -d)
  [ -n "$d" ] && echo "$f: $d"
done   # 期望无输出

# 悬空引用
grep -rhoE '\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' docs/ | sort -u > /tmp/refs.txt
grep -rhoE '^\|[[:space:]]*`?\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' docs/ \
  | grep -ohE '\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' | sort -u > /tmp/defined.txt
comm -23 /tmp/refs.txt /tmp/defined.txt   # 期望无输出
```

- [ ] 3.4 G1 三组校验全部通过

## 步 4 — 改 `docs/02-architecture.md`（主体）

按 `design.md` §5 映射表逐节改。**§9 的树从步 1 的实际目录生成，不要手打。**

- [ ] 4.1 §1 架构总览图：前端层标注改为 `React 19 + TypeScript + Tailwind v4 + Lucide`
- [ ] 4.2 §2 各 crate 小节补 trait 缝落点（S1–S5）
- [ ] 4.3 新增 §3.5「IPC 类型契约同步」（D11）
- [ ] 4.4 §5.1 加版本基线列；**实测值与未实测项分别标注**
- [ ] 4.5 §5.2 重写为「已关闭的岔路」：D12 配置存储 / D13 简繁 / D14 压缩，各含备选与切换条件
- [ ] 4.6 §6 标题改「前端技术栈（定案）」；删「建议」表头与「Vue / Svelte 同样可行」段；加版本基线
- [ ] 4.7 §7 追加 D8–D17（各含背景 / 决策 / 代价）
- [ ] 4.8 新增 §8.5「构建、工具链与打包流水线」：xtask 子命令表 + **D17 工具链固定表** + CI 闸门（版本来源为 volta 字段，非硬编码）
- [ ] 4.9 §9 改标题为「目录结构（定案）」，树内容由实际目录生成，附 §4.1 差异表
- [ ] 4.10 §10 回写 `R33`（内置转换表）、`R37`（LZMA 只读 / zstd 写）；新增 `R53` Tailwind v4 生态成熟度、`R54` tauri-specta 版本跟随、`R55` pnpm 双身份 shim 解析歧义
- [ ] 4.11 顶部目录（§17 行起）同步新增的 §3.5 / §8.5

**版本基线表内容**（2026-08-21 实测 registry）：

| 侧 | 实测 | 未实测（标注「S0 核对」） |
|---|---|---|
| Rust | `tauri 2.11.5`、`windows 0.62.2`、`reqwest 0.13.4`、`rayon 1.12.0`、`thiserror 2.0.20`、`lzma-rs 0.3.0`、`jieba-rs 0.10.3`、`encoding_rs 0.8.35` | `serde`、`serde_json`、`regex`、`once_cell`、`chardetng`、`widestring`、`csv`、`tar`、`zstd`、`sha2`、`tokio`、`anyhow`、`pinyin`、`image`、`gif`、`chacha20poly1305`、`tracing`、`specta`/`tauri-specta`、`cargo-about` |
| 前端 | `react 19.2.8`、`vite 8.2.2`、`tailwindcss 4.3.3`、`zustand 5.0.15`、`@tanstack/react-virtual 3.14.10`、`codemirror 6.0.2`、`i18next 26.4.0`、`@tauri-apps/api 2.11.1` | `react-router`（查询失败）、`lucide-react`、`@tailwindcss/vite`、`prettier-plugin-tailwindcss`、`shadcn/ui`（CLI 分发，非版本化依赖） |

- [ ] 4.12 `lzma-rs 0.3.0` 一栏须注明「实测确认：仍仅支持解压」——这是 D14 的依据

**工具链基线**（2026-08-21 开发机实测，写入 §8.5）：

| 项 | 实测值 | 固定方式 |
|---|---|---|
| Volta | 2.0.2 | — |
| Node | 24.18.1（default；另装 24.19.0 / 22 / 20 / 16 / 12） | `volta pin node@24.18.1` |
| pnpm | 11.18.0（shim `E:\env\Volta\pnpm`，**volta 全局 package 身份**） | `volta pin pnpm@11.18.0` → `R55` |
| Rust | 1.97.1 stable-x86_64-pc-windows-msvc | `rust-toolchain.toml` |

- [ ] 4.13 §8.5 须写明三条「不用」：`.nvmrc`、`packageManager` + corepack、`npm`/`yarn` 命令形态
- [ ] 4.14 §8.5 须写明 pnpm 11 配置落点（`.npmrc` vs `pnpm-workspace.yaml`）标为「S0 核对」，不臆断

## 步 5 — 同步交叉引用

- [ ] 5.1 `docs/00-overview.md:39` 技术栈行 → `Rust + Tauri 2 + React 19 + Tailwind CSS v4`
- [ ] 5.2 `docs/00-overview.md:45` 「界面现代化」行同步
- [ ] 5.3 `docs/README.md:3` 首行技术栈同步
- [ ] 5.4 `docs/22-roadmap.md` S0「workspace 与 6 个 crate 骨架」行补注：目录结构已定案并创建，S0 只需补 `Cargo.toml`
- [ ] 5.5 `docs/03-source-index.md:121` 若提到 Tailwind 机制则同步（只提「改用 Tailwind」则不动）

## 步 6 — 最终校验

- [ ] 6.1 **目录树一致性**（design.md T1）：从 `02` §9 提取路径与 `git ls-files` 比对

```bash
git add -A
git ls-files crates src-tauri src xtask .github | sed 's#/[^/]*$##' | sort -u > /tmp/actual.txt
# 从 02 §9 代码块提取路径，与 /tmp/actual.txt 逐行比对
```

- [ ] 6.2 重跑步 3 的 G1 三组校验，输出与 P1 基线一致
- [ ] 6.3 占位符校验

```bash
grep -rn "TBD\|待补充" docs/ --include="*.md" | grep -v 'grep -rn'   # 期望无输出
```

- [ ] 6.4 未定案表述扫描

```bash
grep -n '建议\|同样可行\|或自建\|待评估' docs/02-architecture.md
# 期望：仅命中明确标注为「备选」「切换条件」的行
```

- [ ] 6.5 决策与风险编号连续无跳号

```bash
grep -ohE '^### D[0-9]+' docs/02-architecture.md | grep -oE '[0-9]+' | sort -n | uniq   # 期望 1..17
grep -ohE '^\| R[0-9]+' docs/02-architecture.md | grep -oE '[0-9]+' | sort -n | uniq | tail -3  # 期望止于 55
```

- [ ] 6.6 确认无可编译文件混入

```bash
git ls-files crates src-tauri src xtask | grep -E '\.(rs|ts|tsx|toml|json)$'   # 期望无输出
git ls-files | grep -E '^(package\.json|Cargo\.toml|rust-toolchain\.toml|pnpm-lock\.yaml)$'   # 期望无输出（归 S0）
```

- [ ] 6.7 包管理器口径统一

```bash
grep -rn 'npm install\|npm run\|yarn \|npx ' docs/   # 期望无输出（一律 pnpm）
```

## 步 7 — 收尾

- [ ] 7.1 沉淀 spec：① Tailwind v4 令牌写法（`@theme inline` 为什么必须）进 `.trellis/spec/frontend/`；② 工具链口径（Volta pin / 只用 pnpm / 不启 corepack）进 `.trellis/spec/guides/`
- [ ] 7.2 提交（`docs:` + `chore:` 分两个 commit：文档定案 / 目录骨架）
- [ ] 7.3 归档任务

## 回滚点

| 步 | 回滚方式 |
|---|---|
| 步 1–2（目录） | `git clean -fd crates src-tauri src xtask .github` |
| 步 3–5（文档） | `git checkout -- docs/` |
| 全部 | 本任务无代码产物，`git checkout -- . && git clean -fd` 即完全回退 |
