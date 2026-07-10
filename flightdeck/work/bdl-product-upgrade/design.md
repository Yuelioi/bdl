# BDL Product Upgrade — Audit and Design Direction

## Research Read

Question: how should BDL evolve from a capable internal-looking downloader into a polished open-source desktop product without weakening its Rust-owned download pipeline?

Target surfaces: application shell, parse workspace, transfer command center, settings, and the public repository surface.

Primary user job: paste one or many Bilibili sources, make a large selection confidently, understand what the downloader is doing, recover failures, and find the output without reading logs.

Constraints:

- Vue 3 + Nuxt UI inside Tauri 2; downloader state and durable behavior remain Rust-owned.
- Chinese-first product UI, offline-capable desktop assets, keyboard and narrow-window support.
- Dense enough for power users without looking like an admin template.
- Existing backend test coverage and normalized DTO boundary must remain intact.

## Current Product Audit

### What is already strong

- Broad resolver and download coverage: video, uploader, favorites, collections, series, bangumi, courses, paging, resume, segmented fetch, URL refresh, muxing, archive assets, startup recovery, diagnostics, and account persistence.
- The product model is coherent: Parse, Transfer, and Settings are distinct workflows; account is global.
- Transfer actions are state-aware and errors are classified into recommended recovery actions.
- Rust has substantial domain tests and the production frontend build passes.

### Product and interaction gaps

- The app opens as three generic panels rather than a recognizable downloader workspace. Global activity, queue health, and the current source are not legible at shell level.
- Parse is a 1,100-line component combining input, selection mapping, filtering, paging, batch settings, and layout. The primary "paste and parse" action competes with source history and dense result controls.
- Transfer is operationally capable but lacks a compact health summary and row-level context-menu parity.
- Settings is one long form with weak location awareness; save state is visible only at the top.
- Narrow windows mostly stack controls. Navigation does not adapt between expanded and compact desktop modes.
- Empty states report absence but do not teach accepted input or the next action.
- There are no frontend tests, no keyboard shortcuts, and no project-level accessibility checks.

### Visual-system gaps

- The palette is a set of unrelated hex values rather than a perceptual semantic system.
- System typography, identical white panels, repeated borders, and uniform spacing make the app feel generic.
- The brand mark is plain text and there is no distinctive visual motif tied to parsing or transfer flow.
- Motion states and `prefers-reduced-motion` handling are absent.
- The current 590 KB JavaScript entry chunk triggers Vite's large-chunk warning.

### Open-source project gaps

- No README, screenshots, license, contribution guide, security policy, changelog, issue templates, or CI workflow are present.
- The repository therefore hides substantial existing functionality and is not contributor-ready.
- License selection is a maintainer decision and must not be guessed during this effort.

## Source Matrix

- Microsoft NavigationView: adaptive expanded, compact, and minimal navigation; page headers remain stable while content gets 12–24 px margins. https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/navigationview
- GitHub Primer DataTable: stable row headers, explicit density, sorting, loading, and named row actions. https://primer.style/product/components/data-table/
- WAI-ARIA Authoring Practices: custom widgets need semantic roles, states, accessible names, and keyboard behavior together. https://www.w3.org/WAI/ARIA/apg/
- MDN reduced motion: non-essential movement must yield to the operating-system preference. https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/%40media/prefers-reduced-motion
- Motrix Next and AB Download Manager: mature open-source downloaders expose queue management, polished theming, release artifacts, contribution paths, and security/build transparency as part of the product—not as afterthoughts. https://github.com/AnInsomniacy/motrix-next and https://github.com/amir1376/ab-download-manager

## Settled Direction

### Concept: Quiet Control Room

BDL should feel calm, mechanical, and decisive: a compact operations console for media transfers, not a marketing dashboard and not a generic CRUD admin.

The memorable motif is a "signal path": source enters on the left, is normalized in the workspace, then moves into a visible transfer rail. Thin tracks, stepped status markers, and clipped geometric surfaces reinforce the pipeline without decorative gradients or glow.

### Product hierarchy

1. The shell communicates where the user is and whether any transfer needs attention.
2. Each page has one primary job: add/select, control/recover, or configure.
3. Status and recommended action appear before technical detail.
4. Advanced controls stay available through progressive disclosure, never hidden from keyboard users.
5. Empty, loading, failure, selected, and narrow-window states are first-class designs.

### Visual language

- Light-first, warm mineral neutrals tinted toward the restrained green brand hue.
- OKLCH tokens for surfaces, text, borders, focus, success, warning, and danger.
- Archivo for the compact mechanical display voice; Noto Sans SC for Chinese product text, bundled for offline use.
- Small radii, crisp outlines, restrained elevation, and selective dark surfaces for the navigation rail—not generic floating cards.
- Motion is limited to navigation selection, page entrance, progress/state changes, and overlays, with a reduced-motion override.

## Local Application

Keep:

- The three primary pages and account placement.
- The normalized tree, backend-owned tasks, existing queue semantics, and task inspector information architecture.
- Local wrappers around Nuxt UI and Tabler icons.

Change first:

- Rebuild the shell and design tokens.
- Make Parse start with a clear command surface and instructional empty state.
- Add a transfer health strip and context-aware operations without destabilizing queue semantics.
- Add settings location/navigation and persistent dirty-state feedback.
- Publish a truthful README and contributor entry point.

Defer until the foundation is verified:

- Backend duplicate-task policy and download-directory health command.
- Full dark theme, updater, scheduler, and account library entry points.
- License file until the maintainer chooses a license.

