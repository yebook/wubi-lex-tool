# Component Guidelines

> Approved UI boundaries before the first product component implementation.

---

## Current Status

**Pending implementation evidence.** The repository has no runnable frontend
entry or product component. S0 contains only toolchain infrastructure and an
isolated virtual-scroll spike, so no component file, export, props or
composition pattern is established yet.

## Approved Boundary

- Product UI uses React 19 and TypeScript. Project-owned shadcn/ui source belongs
  under `src/components/ui/`; reusable product components belong under their
  named directory in `src/components/`; route-specific UI stays with its route.
- Styling uses Tailwind CSS v4 with `src/styles/theme.css` as the token source.
  Components do not introduce literal color, spacing, radius, typography,
  shadow or motion values that bypass the token contract.
- Icons use Lucide components rather than icon fonts or hard-coded glyphs.
  Domain-specific radicals/key caps may use the approved PUA font or product
  rendering instead of an unrelated generic icon.
- Components do not parse files or perform codec/domain transformations. IPC
  payloads use the committed generated bindings.
- Accessibility requirements include keyboard access, visual-order tab flow,
  visible focus, required contrast, non-color-only meaning, accessible names
  and roles, reduced-motion support and resilient system font scaling.
- Large lexicon views must keep DOM and frontend ownership bounded. The S0
  virtual-scroll spike is performance evidence, not a component template.

## Decisions Not Yet Established

The project has not established:

- component filename, export or colocated-test conventions;
- the standard props declaration, ref forwarding or composition API;
- variant/class composition helpers or a component-generation command;
- loading, empty, failure, disabled and feature-placeholder component APIs;
- the product component test renderer, accessibility test helper or snapshot
  policy.

## Forbidden Premature Assumptions

- Do not present a scaffold directory or the spike's `app.tsx` as a product
  component example.
- Do not copy shadcn/ui source before a product task needs and reviews that
  component.
- Do not invent props, variant or barrel-export rules from generic React
  practice.
- Do not put business/codec logic, raw IPC casts or a complete lexicon array in
  a component.
- Do not create a visible component that bypasses the theme, accessibility or
  feature-availability contracts.

## Update Trigger

Update this guide when S1 adds the first reviewed product components. Record
only patterns demonstrated by those components and their tests, including
their actual file/export, props, composition, styling, state and accessibility
contracts.

## Sources

- [`docs/02-architecture.md` frontend stack, D8, D9 and D16](../../../docs/02-architecture.md)
- [`docs/21-ui-ux.md` tokens and component inventory](../../../docs/21-ui-ux.md)
- [`docs/20-nonfunctional.md` NFR-A11Y-001..007](../../../docs/20-nonfunctional.md)
- [Frontend directory structure](./directory-structure.md)
- [Virtualization and performance](./virtualization-performance.md)
