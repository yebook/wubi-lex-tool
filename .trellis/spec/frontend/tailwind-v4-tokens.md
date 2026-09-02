# Tailwind CSS v4 Token Convention

> Executable styling contract for `src/styles/theme.css` and every frontend
> consumer of Tailwind utilities or product CSS variables.

---

## Scenario: Product Theme Tokens

### 1. Scope / Trigger

Apply this contract when changing colors, typography, spacing, radii, control
dimensions, elevation, z-index, focus, motion, density, or Tailwind classes.
`src/styles/theme.css` is the only product token source.

### 2. Signatures

```css
@import "tailwindcss" source("../");
@custom-variant dark (&:where(.dark, .dark *));
@custom-variant compact
  (&:where([data-density="compact"], [data-density="compact"] *));

@theme inline {
  --color-primary: var(--wl-primary);
  --font-sans: var(--wl-font-ui);
  --spacing-control: var(--wl-control-size);
}
```

There is no `tailwind.config.*` or `postcss.config.*`. Vite integrates Tailwind
through `@tailwindcss/vite`; Prettier resolves class order through
`tailwindStylesheet: "./src/styles/theme.css"`.

### 3. Contracts

- Theme-dependent public tokens use `@theme inline` and resolve a `--wl-*`
  custom property at the utility use site. A plain `@theme` value would bake
  one theme into generated utilities and is forbidden.
- Semantic colors are `primary`, `primary-hover`, `primary-subtle`,
  `on-primary`, `surface-1..3`, `border`, `border-strong`, `text-1..3`,
  `success`, `warning`, `danger`, `on-danger`, `info`, `focus`, `scrim`, and
  `wubi-zone-1..5`. Light values live in `:root`; dark overrides live in
  `:root.dark` and the system browser fallback.
- The system fallback applies dark values only when no explicit light/dark
  `data-theme` exists. Native bootstrap and the runtime appearance provider
  own explicit preference projection.
- Offline font stacks are `--wl-font-ui`, `--wl-font-mono`, and
  `--wl-font-etymon`. The optional `WubiLexEtymon` face is named first, but this
  layer does not install, download, or read a font resource.
- Spacing follows the 4 px scale; radii are 4/8/12 px; control and native title
  bar sizes remain stable at 44/48 px. Density changes spacing tokens, not the
  required interactive target size.
- Surface separation uses color and borders. `--wl-shadow-overlay` is reserved
  for real menus, tooltips, and dialogs, with named dropdown/tooltip/dialog
  z-index tokens.
- Product components consume semantic utilities or `var(--wl-*)`. They do not
  introduce a second palette, font stack, radius system, shadow system, motion
  duration, or negative letter spacing.
- Global focus uses the named focus width/color/offset. Reduced-motion media
  rules reduce nonessential transition and animation duration without removing
  focus, state, or loading meaning.
- `@import ... source("../")` keeps Tailwind source discovery rooted at `src/`;
  do not broaden scanning to generated bindings, `target/`, or root resources.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| A color changes with theme | Map it through `@theme inline` to a light and dark `--wl-*` value |
| Explicit `data-theme="light"` under dark OS preference | Keep light values; system fallback must not override them |
| `data-density="compact"` is selected | Reduce density spacing while retaining the control hit area |
| Essential text or focus contrast misses its threshold | Treat as a failing token test and adjust the semantic pair |
| Reduced motion is requested | Collapse nonessential durations to `0.01ms`; preserve visible state changes |
| A v3 config, PostCSS config, online font, or literal component palette appears | Reject the change and move the value into the CSS-first token contract |

### 5. Good / Base / Bad Cases

- Good: `bg-surface-2 text-text-1 border-border-strong` responds to light and
  dark values without component code changing.
- Base: browser rendering without native bootstrap follows system preference,
  while native rendering applies saved attributes before React.
- Bad: hard-coding `bg-[#1e3a5f]`, defining a component-local `--primary`,
  shrinking compact buttons below 44 px, or configuring v4 through a v3 file.

### 6. Tests Required

- Read the real `src/styles/theme.css` with `node:fs`; do not import it through
  Vite `?raw`, because the Tailwind transform can return an empty string in
  Vitest and create false-positive source assertions.
- Assert Tailwind import/source, inline theme mappings, dark/compact variants,
  required token sets, system fallback guards, and reduced-motion rules.
- Parse required sRGB colors and assert normal text at 4.5:1 and focus or large
  UI indicators at 3:1 in both explicit themes.
- Run a production build and browser checks for light/dark/system,
  standard/compact, 1024x640, 1440x900, 200 percent root text, and reduced
  motion; assert no horizontal overflow or incoherent overlap.

### 7. Wrong vs Correct

```css
/* Wrong: generated utilities keep one baked color. */
@theme { --color-primary: #1e3a5f; }

/* Correct: utilities resolve the active product token. */
@theme inline { --color-primary: var(--wl-primary); }
:root { --wl-primary: #1e3a5f; }
:root.dark { --wl-primary: #7da7d9; }
```

## Sources

- [Canonical theme stylesheet](../../../src/styles/theme.css)
- [Theme source tests](../../../src/styles/theme.test.ts)
- [`docs/21-ui-ux.md` token requirements](../../../docs/21-ui-ux.md)
- [`docs/20-nonfunctional.md` accessibility requirements](../../../docs/20-nonfunctional.md)
