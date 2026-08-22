# S0 地基

## Goal

建立可重复构建、可持续验证的工程地基与 `wubilex-codec` 编解码内核，并在进入 S1 前关闭会导致架构返工的四项技术可行性风险。

## Background

- `docs/22-roadmap.md` 将下一阶段定义为 S0：工程基础设施、编解码、真实回归集与技术预研。
- 架构、目录和工具链已经在 `08-21-wubilex-architecture-design` 中定案；当前仓库只有目录骨架，没有可编译配置或源码。
- 当前开发机实测 Volta `2.0.2`、Node `24.18.1`、全局 pnpm `11.18.0`、Rust/Cargo `1.97.1`，与 D17 基线一致。
- `00-bootstrap-guidelines` 仍在进行中；S0 首批代码形成后才能补入真实代码示例并完成归档。

## In Scope

1. 校正 S0 路线图和 Trellis 规范的现有口径冲突。
2. 建立 Rust workspace、前端/Tauri 最小构建壳、固定工具链、xtask 与 CI。
3. 实现 `.lex`、EUDP、码表文本、短语文本、词频与拆字表的底层编解码。
4. 实现编码探测、空白转义和 8 种码表方案探测。
5. 建立真实 fixture、损坏输入、往返、属性和已知缺陷回归测试。
6. 并行完成 TSF、ACL、Task Scheduler COM 和虚拟滚动四项最小技术预研。
7. 用 S0 真实代码补齐相关 Trellis 规范与示例。

## Out Of Scope

- S1 的完整应用外壳、主题、导航、托盘、快捷键与占位 UI。
- S2 及以后的码表浏览、系统读取、系统写入、编辑与变换功能。
- `wubilex-learn` 的实现或加入 workspace members。
- 产品发布、签名、自动更新和真实系统文件替换。

## Requirements

| ID | Requirement |
|---|---|
| S0-R01 | 根 workspace 必须遵守 D10/D17：`src-tauri` 与 `xtask` 是 member，`wubilex-learn` 不是 member；Volta 只管理 Node，直接使用全局 pnpm，不使用 Volta 项目级 pnpm pin、corepack、npm、yarn 或 npx。 |
| S0-R02 | `wubilex-codec` 保持纯同步、无 Tauri/Win32/网络依赖；二进制解析不得用 `repr(C)`/transmute 映射外部字节。 |
| S0-R03 | `.lex` 和 EUDP 编解码必须遵守 `docs/01-data-formats.md` 的字节布局、排序、UTF-16 和损坏输入契约。 |
| S0-R04 | 文本解析必须覆盖码表 6 种方言 + 微软分支、7 种输出，以及短语 6 种方言、`$[...]`、多行与时间变量。 |
| S0-R05 | 编码探测必须 BOM 优先并覆盖 UTF-8、UTF-16LE/BE、GBK；方案探测覆盖 8 种方案并修复 `XFXY` 缺陷。 |
| S0-R06 | 真实回归集必须可复现获取并校验完整性；仓库不依赖开发机上的隐式 fixture。 |
| S0-R07 | `wubilex-codec` 覆盖率达到 90% 以上；CI 执行 fmt、clippy、test、覆盖率、依赖审计、bindings/docs 检查和前端静态检查。 |
| S0-R08 | 四项技术预研必须各自产出可重复运行的最小原型、步骤、结果和通过/失败判据；失败项在 S1 前触发架构复评。 |
| S0-R09 | Trellis 规范只记录已定案合同和真实实现；不得用假想代码示例提前关闭 bootstrap。 |

## Child Task Map

| Order | Child | Deliverable |
|---:|---|---|
| 0 | `s0-docs-spec-alignment` | 路线图口径和首批规范基线 |
| 1 | `s0-workspace-toolchain` | workspace、工具链和最小构建壳 |
| 2 | `s0-codec-model` | 公共模型、错误与输入限制 |
| 3 | `s0-lex-binary` | `.lex` 二进制编解码 |
| 4 | `s0-eudp` | EUDP 二进制编解码 |
| 5 | `s0-lex-text` | 码表文本方言与输出 |
| 6 | `s0-phrase-aux` | 短语文本、词频、拆字、转义和探测 |
| 7 | `s0-fixtures-regressions` | fixture、属性测试与缺陷回归 |
| 8 | `s0-xtask-ci` | xtask 与 CI 闸门 |
| 9 | `s0-risk-spikes` | 四项 S1 前技术预研 |
| 10 | `s0-integration` | 全量验收、规范回填与 bootstrap 收口 |

子任务按需创建。父任务只维护总需求、依赖和最终集成，不作为代码实现目标。

## Acceptance Criteria

- [x] workspace 中 6 个成员和前端最小壳在固定工具链下通过构建与静态检查。
- [ ] `.lex` 往返在 8 种真实方案上字节级一致。
- [ ] 7 种码表文本输出与行为规格逐字节一致。
- [ ] EUDP 往返覆盖 emoji 代理对、多行短语、`$[...]` 和损坏输入。
- [ ] S0 所属的 codec 缺陷回归均经历失败到通过；S4 所属缺陷有明确阶段归属。
- [ ] `wubilex-codec` 测试覆盖率不低于 90%。
- [ ] CI 与本地验证命令使用同一版本来源并全部通过。
- [ ] 四项技术预研均达到判定标准，或在进入 S1 前完成架构复评并更新设计。
- [ ] 需求计数保持模块 414、NFR 101、UX 115、总计 630；内部锚点 0 失效。
- [ ] 相关 Trellis 规范包含真实 S0 代码示例，`00-bootstrap-guidelines` 满足完成条件。

## Risks And Deferred Items

- `tauri-specta` 与当前 Tauri 小版本兼容性在 workspace 子任务中验证，失败时按 R54 切换为 `ts-rs` + 契约测试。
- pnpm 11 项目设置使用 `pnpm-workspace.yaml`；没有 authentication/registry 项目设置时不创建 `.npmrc`。
- 真实 Windows 技术预研可能需要管理员权限和隔离环境；原型不得操作用户实际输入法数据文件。
- 完整前端体验和业务 command 延后到 S1 及后续阶段。
