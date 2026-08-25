# Thinking Guides

> **Purpose**: Expand your thinking to catch things you might not have considered.

---

## Why Thinking Guides?

**Most bugs and tech debt come from "didn't think of that"**, not from lack of skill:

- Didn't think about what happens at layer boundaries → cross-layer bugs
- Didn't think about code patterns repeating → duplicated code everywhere
- Didn't think about edge cases → runtime errors
- Didn't think about future maintainers → unreadable code

These guides help you **ask the right questions before coding**.

---

## Available Guides

| Guide | Purpose | When to Use |
|-------|---------|-------------|
| [Code Reuse Thinking Guide](./code-reuse-thinking-guide.md) | Identify patterns and reduce duplication | When you notice repeated patterns |
| [Cross-Layer Thinking Guide](./cross-layer-thinking-guide.md) | Think through data flow across layers | Features spanning multiple layers |
| [需求 ID 契约与文档集校验](./requirement-id-conventions.md) | `docs/` 需求 ID 语法、计数不变量、锚点校验、校验命令 | 增删改任何 `docs/` 需求条目**或章节标题**前 |
| [工具链口径](./toolchain-conventions.md) | Volta 项目级 pin Node/pnpm、显式 pnpm feature flag、不碰 corepack、Rust toolchain | 写任何命令、脚本、CI 配置前 |

---

## Quick Reference: Thinking Triggers

### When to Think About Cross-Layer Issues

- [ ] Feature touches 3+ layers (API, Service, Component, Database)
- [ ] Data format changes between layers
- [ ] Multiple consumers need the same data
- [ ] You're not sure where to put some logic
- [ ] You are adding an event kind, JSONL record, RPC payload, or config field
- [ ] UI / command code starts casting raw payload fields directly

→ Read [Cross-Layer Thinking Guide](./cross-layer-thinking-guide.md)

### When to Think About Code Reuse

- [ ] You're writing similar code to something that exists
- [ ] You see the same pattern repeated 3+ times
- [ ] You're adding a new field to multiple places
- [ ] **You're modifying any constant or config**
- [ ] **You're creating a new utility/helper function** ← Search first!
- [ ] Two files read the same untyped payload field with local casts
- [ ] Multiple branches update the same derived state from `kind` / `action`

→ Read [Code Reuse Thinking Guide](./code-reuse-thinking-guide.md)

### When Touching `docs/` Requirement Entries

- [ ] Adding, deleting, or renumbering any `M*` / `NFR-*` / `UX-*` requirement
- [ ] **Renaming or renumbering any section heading** ← 会静默打断所有指向它的 `](#...)` 链接，跑 `check_anchors.py`
- [ ] Adding a new module (→ every `M[1-N]` range in validation scripts must be widened)
- [ ] Writing any script that greps requirement IDs ← **域段可能含数字，`[A-Z]+` 会静默漏计**
- [ ] Justifying a requirement with "原项目就是这么做的" ← not a valid reason on its own

→ Read [需求 ID 契约与文档集校验](./requirement-id-conventions.md)

### When Writing Any Command, Script, or CI Config

- [ ] 要写 `npm` / `yarn` / `npx` ← 本项目只用 pnpm
- [ ] 要在 workflow 里硬编码 Node / pnpm / Rust 版本 ← 版本源是 `volta.node`、`volta.pnpm` 与 `rust-toolchain.toml`
- [ ] 想加 `.nvmrc`、`engines.pnpm` 或 `packageManager` 字段 ← 本项目由 `package.json.volta` 同时固定 Node 与 pnpm

→ Read [工具链口径](./toolchain-conventions.md)

### When Verifying AI Cross-Review Results

- [ ] Reviewer claims "user input can be malicious" → Check the actual data source (internal manifest? user config? external API?)
- [ ] Reviewer flags "missing validation" → Is the data from a trusted internal source?
- [ ] Reviewer says "behavior change" → Read the code comments — is it intentional design?
- [ ] Reviewer identifies a "bug" in test → Mentally delete the feature being tested — does the test still pass? If yes → tautological test

**Common AI reviewer false-positive patterns**:
1. **Trust boundary confusion**: Treating internal data (bundled JSON manifests) as untrusted external input
2. **Ignoring design comments**: Flagging intentional behavior documented in code comments as bugs
3. **Variable misreading**: Not tracing a variable to its actual definition (e.g., Map keyed by path vs name)

**Verification rule**: Every CRITICAL/WARNING finding must be verified against the actual code before prioritizing. Budget ~35% false-positive rate for AI reviews.

---

## Pre-Modification Rule (CRITICAL)

> **Before changing ANY value, ALWAYS search first!**

```bash
# Search for the value you're about to change
grep -r "value_to_change" .
```

This single habit prevents most "forgot to update X" bugs.

---

## How to Use This Directory

1. **Before coding**: Skim the relevant thinking guide
2. **During coding**: If something feels repetitive or complex, check the guides
3. **After bugs**: Add new insights to the relevant guide (learn from mistakes)

---

## Contributing

Found a new "didn't think of that" moment? Add it to the relevant guide.

---

**Core Principle**: 30 minutes of thinking saves 3 hours of debugging.
