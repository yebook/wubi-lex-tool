# Tailwind CSS v4 令牌约定

> **适用范围**：`src/styles/theme.css` 与任何写 Tailwind 类名的前端代码。
> **为什么单列一份**：v4 是 CSS-first，与网上绝大多数 v3 时代的资料写法不兼容。照抄 v3 写法不会报错，只会静默失效。

---

## 1. 唯一事实来源

令牌**只在 `src/styles/theme.css` 定义一次**。

- 项目中**不存在 `tailwind.config.ts`**
- 构建接入是 Vite 插件 `@tailwindcss/vite`，**没有 `postcss.config.js`**
- 组件里不写颜色、间距、圆角、字号、阴影、动效时长的字面值

需求出处：`UX-TOKEN-001` / `UX-TOKEN-011`，完整约定见 `docs/21-ui-ux.md` §4.6。

---

## 2. `@theme inline` 不是可选的

这是本项目最容易写错、且**最难从现象反推原因**的一处。

```css
/* ✅ 正确：工具类在「使用处」解析变量，切 .dark 才会改色 */
@theme inline {
  --color-primary: var(--wl-primary);
}
:root { --wl-primary: #1E3A5F; }
.dark { --wl-primary: #7DA7D9; }
```

```css
/* ❌ 错误：值被烘死进生成的工具类，主题切换对已渲染元素无效 */
@theme {
  --color-primary: #1E3A5F;
}
```

**错误的表现**：主题切换按钮点了「没反应」，或只有部分元素变色。查这个问题时人的第一反应通常是去看主题切换的 JS 逻辑、`<html>` 上的 class 有没有加上——那些都是对的，真正的原因在 CSS 层，隔得很远。

**判据**：任何需要随主题变化的令牌，必须走 `@theme inline` + `var()` 间接层。不随主题变化的（字体栈、断点）可以直接写在 `@theme` 里。

---

## 3. v3 写法对照

看到左列一律换右列。左列在 v4 下**不报错，只是不生效**。

| v3 写法 | v4 写法 |
|---|---|
| `tailwind.config.ts` + `theme.extend.colors` | `@theme inline { --color-*: var(--wl-*) }` |
| `darkMode: 'class'` | `@custom-variant dark (&:where(.dark, .dark *))` |
| `rgb(var(--x) / <alpha-value>)` | 直接写颜色值；透明度用 `bg-primary/50` |
| PostCSS 插件链 | `@tailwindcss/vite` |
| `data-density="compact"` 属性选择器 | `@custom-variant compact (&:where([data-density="compact"], …))` |
| `prettier-plugin-tailwindcss` 的 `tailwindConfig` | 同插件的 `tailwindStylesheet` 选项 |

---

## 4. 本项目的具名令牌

| 前缀 | 用途 | 备注 |
|---|---|---|
| `--color-primary` / `--color-danger` / `--color-success` / `--color-warning` / `--color-info` | 语义色 | `UX-TOKEN-002` |
| `--color-surface-1..3` | 表面分层 | 用**表面色 + 边框**分层，阴影只给真正的浮层（`UX-TOKEN-004`） |
| `--color-wubi-zone-1..5` | 五笔五区配色 | 深浅色各一组（`UX-TOKEN-014`）。这是产品里真正需要用颜色说话的地方 |
| `--font-mono` | 编码、码表、日志 | 编码需按位对齐才能一眼看出长度差异 |
| `--font-etymon` | 字根 PUA 字体 | 首项必须命中，否则显示缺字符号（`M3-FONT-003`） |

---

## 5. 检查清单

- [ ] 新令牌走 `@theme inline` + `var()`，不是裸 `@theme`
- [ ] 深浅两套值都给了（`:root` 与 `.dark`）
- [ ] 组件里没有颜色/间距字面值
- [ ] 对比度按 `NFR-A11Y-003` 在**两种主题下各验一次**
- [ ] 没有新增 `tailwind.config.ts` 或 `postcss.config.js`
