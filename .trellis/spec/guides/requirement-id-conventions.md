# 需求 ID 契约与文档集校验

> **适用范围**：`docs/` 需求文档集的任何增删改。
> **为什么单列一份**：ID 是 `22-roadmap.md` 汇总表的主键，也是 `03-source-index.md` 反向索引的连接键。ID 语法或计数一旦漂移，两张表会静默失真——没有编译器会报错。

---

## 1. ID 语法

```
<前缀>-<域>-<三位序号>
```

| 段 | 取值 | 示例 |
|---|---|---|
| 前缀 | `M1`..`M8`（模块）、`NFR`（非功能）、`UX`（UI/UX） | `M1` / `NFR` / `UX` |
| 域 | 模块内功能分组缩写，**可能含数字** | `PARSE` / `KEYMAP` / `A11Y` |
| 序号 | 三位，域内**永不复用**；废弃需求保留 ID 并标注「已废弃」 | `004` |

**定义行 vs 引用**：ID 的「定义」是需求表格行（以 `|` 开头的那一行）。同一 ID 在别的文档里出现是正常的**交叉引用**，不是重复。

---

## 2. 三个已踩过的 regex 陷阱

这三条是 2026-08-21 收尾时实际发现的——原 `implement.md` 的校验命令照抄会得出错误结论：

### 陷阱 1：`[A-Z]+` 匹配不到含数字的域段

```bash
grep -oE 'NFR-[A-Z]+-[0-9]{3}'    # ✗ 漏掉 NFR-A11Y-* 与 NFR-I18N-*
grep -oE 'NFR-[A-Z0-9]+-[0-9]{3}' # ✓
```

全库当前有且仅有两个含数字的域段，都在 `20-nonfunctional.md`：

| 域 | 条数 | 由来 |
|---|---|---|
| `A11Y` | 7 | accessibility 的 numeronym |
| `I18N` | 5 | internationalization 的 numeronym |

合计 12 条——正是 `[A-Z]+` 把非功能需求从 **101 少数成 89** 的全部差额。

**危害等级最高**：它不报错，只是少数，而且少得「看起来挺合理」。注意这两个域是 numeronym（首尾字母夹数字）这一构词法的产物，将来再加 `L10N`（localization）之类同样会中招。

### 陷阱 2：模块号范围写死

```bash
grep -oE 'M[1-7]-...'   # ✗ M8 自学习模块加入后全部漏计
grep -oE 'M[1-8]-...'   # ✓ 新增模块时必须同步这个范围
```

**新增模块 = 必须全库搜一遍 `M[1-` 更新所有校验脚本。**

### 陷阱 3：跨文档引用被误判为重复 ID

```bash
grep -rhoE '<ID>' docs/ | sort | uniq -d   # ✗ 把正常引用全报成重复
```

唯一性必须**按单文件、只看定义行**判：

```bash
for f in docs/modules/M*.md docs/20-nonfunctional.md docs/21-ui-ux.md; do
  d=$(grep -ohE '^\|[[:space:]]*`?\b[A-Z][A-Z0-9]*-[A-Z0-9]+-[0-9]{3}\b' "$f" \
      | grep -ohE '\b[A-Z][A-Z0-9]*-[A-Z0-9]+-[0-9]{3}\b' | sort | uniq -d)
  [ -n "$d" ] && echo "$f: $d"
done
```

---

## 3. 计数不变量

`docs/22-roadmap.md` §1.1 汇总表断言：

| 来源 | 条数 |
|---|---|
| 模块（`docs/modules/M1..M8`） | 414 |
| 非功能（`20-nonfunctional.md`） | 101 |
| UI/UX（`21-ui-ux.md`） | 115 |
| **总计** | **630** |

改动需求条目后**必须**重跑并同步汇总表，否则两处会不一致：

```bash
M=$(grep -rhoE '\bM[1-8]-[A-Z0-9]+-[0-9]{3}\b' docs/modules/            | sort -u | wc -l)
N=$(grep -ohE  '\bNFR-[A-Z0-9]+-[0-9]{3}\b'    docs/20-nonfunctional.md | sort -u | wc -l)
U=$(grep -ohE  '\bUX-[A-Z0-9]+-[0-9]{3}\b'     docs/21-ui-ux.md         | sort -u | wc -l)
echo "模块=$M 非功能=$N UI/UX=$U 总计=$((M+N+U))"
```

---

## 4. 悬空引用校验

被引用的 ID 必须存在定义行。删除需求时最容易漏掉别处的引用：

```bash
grep -rhoE '\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' docs/ | sort -u > /tmp/refs.txt
grep -rhoE '^\|[[:space:]]*`?\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' docs/ \
  | grep -ohE '\b(M[1-8]|UX|NFR)-[A-Z0-9]+-[0-9]{3}\b' | sort -u > /tmp/defined.txt
comm -23 /tmp/refs.txt /tmp/defined.txt   # 期望无输出
```

---

## 5. 占位符校验

```bash
grep -rn "TBD\|待补充" docs/ --include="*.md" | grep -v 'grep -rn'
```

> 末尾的 `grep -v` 不可省：`README.md` 与 `22-roadmap.md` 正文里就写着这条命令本身，会自己命中自己。

---

## 6. 改动需求文档的检查清单

- [ ] 新 ID 的域段拼写与该模块既有域段一致（别造 `KEYMAPS` 之类的近义域）
- [ ] 序号未复用已废弃 ID
- [ ] 新增模块 → 全库更新 `M[1-N]` 范围
- [ ] 重跑 §3 计数，同步 `22-roadmap.md` 汇总表
- [ ] 重跑 §4 悬空引用校验
- [ ] 若改的是需求来源/行为 → 同步 `03-source-index.md` 的反向索引

---

## 7. 一条越界原则

`docs/02-architecture.md#0` 定的首要原则是「**旧项目是行为规格，不是实现范本**」。

写需求时的直接推论：**「原项目就是这么做的」不构成需求理由**。只有 `§0.1` 那 12 条行为契约（改了会让用户数据出错或系统不工作）才要求逐位一致；其余维度自由改进。遇到未列出的分歧走 `§0.4` 的判定流程。

> 实例：原项目用 `Ctrl+W` 做「最小化到托盘」，与全平台「关闭」语义冲突——按此原则改为 `Ctrl+Shift+H`，而不是因为「原项目这样」就沿用。
