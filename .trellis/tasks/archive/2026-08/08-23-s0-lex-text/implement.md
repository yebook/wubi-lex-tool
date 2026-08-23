# Implementation Plan - S0-05 码表文本编解码

## 1. Baseline And Failure Tests

- [x] 记录 codec 公开 API、依赖树、现有测试数和 workspace 门禁基线。
- [x] 新增公开解码结果、warning、format 枚举和空白 API 的编译合同，先确认目标测试在实现前失败。
- [x] 用手写输入为四编码、A..F、微软/极点、warnings、limits 和七格式建立独立期望值。
- [x] 先添加 `%0B/%0C` 旧不对称回归，确认它能在新 escape 实现前失败。

## 2. Public Types And Dependencies

- [x] 精确 pin `chardetng 1.0.0`、`encoding_rs 0.8.35`，更新 lockfile 并审计 license/features/逆向依赖；行语法使用结构化 token parser，不新增 regex 直接依赖。
- [x] 建立 `text` / `escape` 模块，在 `lib.rs` 暴露必要 API，不把 format-specific warning 放进格式中立 `model/`。
- [x] 实现 `DecodedLexiconText`、`LexiconTextWarning`、warning kind 和 7 分支 `LexiconTextFormat` 的只读访问器及 Rustdoc。

## 3. Encoding And Escape

- [x] 实现 input limit、BOM 优先、无 BOM strict UTF-8 优先和 chardetng `cn` -> GBK 选择。
- [x] 实现 UTF-8/UTF-16LE/BE/GBK without-replacement 解码和原始字节偏移错误。
- [x] 实现 6 种 ASCII 空白的对称编解码，未知/小写/不完整 `%xx` 保持字面值。
- [x] 实现带 BOM 的 deterministic UTF-16LE 输出，覆盖空字符串、BMP 和 emoji。

## 4. Preprocessing And Decoder

- [x] 建立保留原始 line/column 的 line records，实现 YAML、注释、`[Text]` 和极点/微软检测顺序。
- [x] 实现微软专用分支和提前返回，对每个展开 code 验证模型和 limits。
- [x] 实现 A/B 显式权重、C/E 多词拆分、D 降序权重及 F 多码展开，严格保持优先级。
- [x] 实现 D 的全局最小值/5000 平移和极点权重清空/词条标志清理。
- [x] 实现未知行的有序 visible warnings，preview 限定 160 Unicode scalar，并与 entry 共用产出预算；已识别损坏字段和非空零 entry 严格失败。
- [x] 对每个 parser failure/limit/warning 断言结构化 kind 与原始 line/column，不返回部分 document。

## 5. Canonical Formatter

- [x] 实现不修改 document 的 code 稳定分组、effective-weight 补齐和组内稳定权重排序。
- [x] 实现码前词后、码前聚合、码前升序权重三种格式。
- [x] 实现词前码后、词前聚合、词前降序权重三种格式。
- [x] 实现短语升序号格式，每 code 按 canonical weight 排序后重编号 1..N。
- [x] 用手写完整预期字符串验证分隔符、CRLF、空白转义、相邻重复折叠、等权稳定性和缺省权重溢出。

## 6. Robustness And Scope Review

- [x] 覆盖空 document/body、只有预处理内容、只有 warnings、混合有效/warning、最大权重和展开上限。
- [x] 对各编码的截断/非法序列和重要 parser 损坏输入执行无 panic 测试。
- [x] 确认 production source 无 `unwrap()` / `expect()` / unsafe / lossy decode，token parser 不依赖不可观测的静态 pattern 初始化 panic。
- [x] 确认没有 CSV/JSON、压缩、路径 I/O、短语语法、方案探测、core 索引、Tauri/Windows/Tokio/网络逻辑。
- [x] 确认代码/测试/命令不读取、修改或暂存根目录 `resource/`。

## 7. Validation

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo test -p wubilex-codec --all-features
cargo test --workspace --all-features
cargo doc -p wubilex-codec --all-features --no-deps
cargo tree -p wubilex-codec
pnpm lint
pnpm typecheck
pnpm test
python ./.trellis/scripts/task.py validate 08-23-s0-lex-text
python ./.trellis/scripts/check_anchors.py
rg -n 'unwrap\(|expect\(|unsafe|from_utf.*lossy|resource/' crates/wubilex-codec/src crates/wubilex-codec/tests
git diff --check
git status --short
```

- [x] codec 定向测试、workspace 全量 Rust 门禁和全局 pnpm 静态门禁通过。
- [x] Rustdoc 在 warnings denied 下通过，新公开类型/方法文档完整。
- [x] 依赖树只新增已审批的 chardetng/encoding_rs 链，不含 platform/async/network 依赖。
- [x] Trellis check 逐条核对 LT-R01..R15 与 Acceptance Criteria。

## 8. Finish

- [x] 用真实 text/escape parser/formatter 证据更新 backend 规范，但不提前关闭 `00-bootstrap-guidelines`。
- [ ] 提交并归档子任务，在父任务中把 `s0-lex-text` 标为完成。

## Rollback Points

- encoding/escape、decoder 和 formatter 可分别调试，但必须作为同一文本合同通过全量测试后提交。
- 若 chardetng 对真实 GBK fixture 误判，使用可复现样本收窄策略，不回退到系统 locale 或 lossy fallback。
- 若 warning 列表在真实文件中过大，先设计显式 diagnostics limit/truncation 合同再改 API，不恢复静默丢行。
- 若旧产物证明某个排序/重复细节与文档不同，先把对照样本加入 `s0-fixtures-regressions`，再调整 canonical formatter。
