# Implementation Plan - S0-06 短语与辅助文本编解码

## 1. Baseline And Failure Tests

- [x] 记录 codec 公开 API、依赖树、现有测试数和 workspace 门禁基线。
- [x] 为 phrase text、word frequency、spelling table 和 scheme detection 建立公开编译合同，先确认目标测试在实现前失败。
- [x] 用手写输入固定 P1..P6、multiline、`$[...]`、时间变量、辅助表和八方案的独立期望。
- [x] 添加小写 `xfxy` 缺陷回归，并证明测试在探测实现前失败而非自证实现。

## 2. Shared Encoding And Public Models

- [x] 把现有严格 byte decoder 提升为 crate 内共享模块，保持 `text` 公开行为与测试不变。
- [x] 新增 phrase text 的 decoded result、warning kind/warning 访问器及公开模块导出。
- [x] 新增 word-frequency/spelling-table entry/document 值对象、无空白校验和 Rustdoc。
- [x] 确认 Cargo 依赖与 lockfile 不变，crate 仍无 platform/async/network/filesystem 依赖。

## 3. Phrase Text Decoder

- [x] 实现保留行列的跨行注释扫描和未闭合错误。
- [x] 实现 P1..P6 tri-state parser、字段 validator 和固定优先级。
- [x] 实现 P2/P3 multiline 状态及 recognized-line 终止/EOF finalize。
- [x] 实现显式候选、per-code 最大值自动分配、`$[...]` 两种分割和 `1..=255` 边界。
- [x] 实现 P2 `#`、九个时间别名、未知别名保留及共享空白反转义。
- [x] 实现有序有界 visible warnings、非空零 entry 错误和 entry/warning 共用预算。

## 4. Phrase Canonical Formatter

- [x] 实现 code/candidate 稳定排序，不修改 document 并保留重复项。
- [x] 实现多条连续短候选的 `$[...]` 压缩，按 UTF-16 units 正确处理 emoji。
- [x] 实现候选明细回退、共享空白转义和固定 CRLF。
- [x] 用完整手写字符串覆盖候选缺口、重复候选、单条、emoji、两 BMP 字符和超长边界。

## 5. Auxiliary Table Codecs

- [x] 实现无 BOM strict UTF-8、CRLF/LF、空行、精确两列和原始 line/column 辅助函数。
- [x] 实现 word frequency decode/format，覆盖 `1..=65535`、顺序、重复和 LF 输出。
- [x] 实现 spelling table decode/format，覆盖 PUA/non-BMP、顺序、重复和 LF 输出。
- [x] 覆盖 BOM、非法 UTF-8、空字段、多余字段、非法数字及 input/entry limit 边界。

## 6. Scheme Detection

- [x] 以单次特征扫描实现五组按序直接判定，不受 weight/duplicate 影响。
- [x] 实现九项 86/98/06/091 打分和严格赢家/86 fallback。
- [x] 将旧 `XFXY` 改为 `xfxy`，用可区分 06 与 fallback 的最小码表固定回归。
- [x] 覆盖八种显示方案、郑码 formation、直接分支优先级、空文档和平局。

## 7. Robustness And Scope Review

- [x] 覆盖空输入、注释-only、warning-only、最大候选/权重和精确资源边界。
- [x] 对代表性编码、注释、phrase/aux 损坏输入执行 no-panic 测试。
- [x] 扫描 production source，确认无 `unwrap()` / `expect()` / unsafe / lossy decode。
- [x] 确认没有 EUDP wire 重复实现、core 变换、资源下载、路径 I/O、Tauri/Windows/Tokio/网络逻辑。
- [x] 确认代码、测试和验证命令完全不访问根目录 `resource/`。

## 8. Validation

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
python ./.trellis/scripts/task.py validate 08-24-s0-phrase-aux
python ./.trellis/scripts/check_anchors.py
rg -n 'unwrap\(|expect\(|unsafe|from_utf.*lossy|resource/' crates/wubilex-codec/src crates/wubilex-codec/tests
git diff --check
git status --short
```

- [x] codec 定向测试、workspace 全量 Rust 门禁和全局 pnpm 静态门禁通过。
- [x] Rustdoc 在 warnings denied 下通过，公开类型/方法文档完整。
- [x] 依赖树与 lockfile 无新增依赖，production forbidden-pattern scan 通过。
- [x] Trellis check 逐条核对 PA-R01..R16 与 Acceptance Criteria。

## 9. Finish

- [x] 用真实 phrase/aux/scheme 证据更新 backend 规范，但不提前关闭 `00-bootstrap-guidelines`。
- [ ] 分批提交实现、规范与任务记录，归档子任务并同步父任务进度。

## Rollback Points

- shared encoding 重构必须先由现有 text tests 证明行为不变，再叠加新 parser。
- phrase text、auxiliary tables 和 scheme detection 可分块定位；任一失败不要求回退已验证的其他格式合同。
- 若真实 fixture 揭示未记录变体，先保留严格 parser 和失败样本，再经规格评审扩展，不能用静默跳过掩盖损坏。
