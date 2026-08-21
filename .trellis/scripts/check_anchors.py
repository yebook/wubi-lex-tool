#!/usr/bin/env python3
"""校验 docs/ 内所有 markdown 内部锚点链接是否有效。

用法：  python .trellis/scripts/check_anchors.py      # 期望输出「0 处失效」
契约：  .trellis/spec/guides/requirement-id-conventions.md §6

注意两个已踩过的坑（改这个脚本前先读那一节）：
  1. GitHub 的 slugger 不 trim 过滤后残留的首尾空白 —— 不要加 .strip()
  2. 相对路径必须 os.path.normpath，否则 ../ 形式全部误报
退出码：有失效链接时为 1，便于接进 CI。
"""
import re, pathlib, os, sys
def anchor(text):
    t = re.sub(r'`([^`]*)`', r'\1', text)
    t = re.sub(r'\*\*?([^*]*)\*\*?', r'\1', t)
    t = re.sub(r'\[([^\]]*)\]\([^)]*\)', r'\1', t)
    t = t.strip().lower()
    # GitHub slugger 不 trim 过滤后残留的首尾空白：`### ⚠️ 性能红线` -> `-性能红线`
    return ''.join(c for c in t if c.isalnum() or c in '_- ').replace(' ', '-')

docs = sorted(pathlib.Path('docs').rglob('*.md'))
anchors = {}
for p in docs:
    s = set()
    for line in p.read_text(encoding='utf-8').splitlines():
        m = re.match(r'^(#{1,6})\s+(.*)$', line)
        if m:
            s.add(anchor(m.group(2)))
    anchors[p.as_posix()] = s

bad = []
for p in docs:
    for i, line in enumerate(p.read_text(encoding='utf-8').splitlines(), 1):
        for m in re.finditer(r'\]\(([^)]+)\)', line):
            href = m.group(1)
            if href.startswith('http') or '#' not in href:
                continue
            fp, _, frag = href.partition('#')
            if not frag:
                continue
            joined = os.path.join(str(p.parent), fp if fp else p.name)
            target = os.path.normpath(joined).replace(os.sep, '/')
            if target not in anchors:
                bad.append((p.as_posix(), i, href, 'FILE ' + target)); continue
            if frag not in anchors[target]:
                bad.append((p.as_posix(), i, href, 'ANCHOR'))
print(f"{len(bad)} 处失效：")
for f, i, h, w in bad:
    print(f"  {f}:{i}  {h}  [{w}]")
sys.exit(1 if bad else 0)
