# S1 Planning Baseline

## Evidence Reviewed

- `docs/22-roadmap.md` S1 scope and exit criteria.
- `docs/modules/M7-app-shell.md` lifecycle, window, tray, hotkey, keymap, config, bus and task requirements.
- `docs/21-ui-ux.md` information architecture, tokens, components, interaction rules and settings screens.
- `docs/20-nonfunctional.md` startup, security, compatibility, reliability, observability, i18n and accessibility constraints.
- `docs/02-architecture.md` Tauri state, IPC generation, D8/D9/D11/D12/D16 and approved directory layout.
- Current `src/`, `src-tauri/`, root manifests and relevant `.trellis/spec/` guides.
- Archived S0 foundation, risk-spike and integration task evidence.
- Project-scoped `trellis mem` results for S1 and the earlier shell/UI decision.

## Current Repository Baseline

- The Rust workspace and locked frontend toolchain are healthy S0 deliverables.
- `src-tauri` has a reusable bindings builder and mock export path, but no desktop `main.rs`, runtime command set, window or Tauri build wiring.
- `src/` contains only committed generated bindings and `vite-env.d.ts`; there is no `index.html`, React entry, route, provider, store, component or stylesheet.
- React and the Tailwind Vite plugin are installed. React Router, Zustand, i18next, Lucide and product component dependencies are not yet installed.
- The frontend component, Hook and state-management specs intentionally remain pending until S1 produces reviewed examples.
- The root `resource/` directory is user-provided and excluded from this task.

## Requirement Reconciliation

1. `UX-IA-001` replaces the legacy four-item top navigation in `M7-WIN-002` with the approved seven-domain sidebar. Legacy help content is redistributed to Radicals and Settings.
2. `M7-KEYMAP-011` and its approved default table replace the legacy `Ctrl+W` behavior in `M7-WIN-008`; `window.hide` defaults to `Ctrl+Shift+H`.
3. S1 satisfies `M7-INST-006` by detecting abnormal sessions, exposing recovery state and warning the user. Actual TSF/service/ACL repair belongs to S3 and must not be simulated.
4. S1 preserves the complete tray information architecture, but actions owned by later milestones are disabled placeholders. Dynamic system state is not fabricated.
5. M7 domain events are registered as typed contracts in S1. Producers and consumers owned by later modules are attached when those modules are implemented.
6. `M7-WIN-005` is permanently deprecated as P3. ImTip has no route, action, tray item, setting, feature flag, process integration, URL or dependency in S1 or any later milestone.
7. Full Win10 1703, ARM64, installer, WebView2 bootstrapper and signing validation are release concerns. S1 avoids incompatible choices and validates its first runnable shell on Windows 11 x64.

## UI/UX Research Result

The available `ui-ux-pro-max` skill was loaded and applied for accessibility, interaction, theme, typography, icon and navigation rules. Its installation contains path-pointer files for `scripts/` and `data/`, but the referenced local search script is absent, so database search could not run.

The repository UI specification remains the authoritative product source. The synthesized direction is a quiet Windows productivity tool: stable side navigation, restrained ink-blue/vermillion/cool-gray semantic tokens, bounded status cards, shallow settings groups, strong focus states, consistent Lucide icons, 120/200 ms meaningful motion and no decorative marketing composition.

## Decisions With No Remaining User Question

- Product scope and S1 exit criteria are already approved in the roadmap.
- Security posture, frontend stack, configuration ownership, feature-flag source and IPC generation are already architecture decisions.
- The user has asked to continue planning and has not requested an alternate visual direction or compatibility scope.
- No unresolved product, UX, risk-tolerance or acceptance decision blocks the final planning review.
