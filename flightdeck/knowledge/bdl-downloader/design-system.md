# BDL design-system checklist
SUMMARY: Always shape BDL as a quiet desktop control room: achromatic cool-neutral chrome, restrained Bilibili-pink signal color, clear command hierarchy, adaptive navigation, stable transfer rows, and progressive disclosure.
READ WHEN: before any BDL UI, interaction, visual-token, responsive-layout, or user-facing copy change.

---

## Direction

BDL is a compact operations tool for Bilibili power users. It should feel calm, mechanical, and decisive—not like a marketing page, generic admin dashboard, or neon developer tool.

## Standing rules

- Keep Parse, Account Library, Transfer, and Settings as the only primary pages; account identity and login remain global in the top bar.
- Put current status and the recommended action before logs or technical detail.
- Use OKLCH semantic tokens. Structural surfaces stay achromatic/cool-neutral in both themes: clean whites and grays in light mode, a charcoal surface ladder in dark mode. Pink is reserved for primary actions, selection, focus, and small status signals; red, amber, and green appear only for their semantic states.
- The custom 42px desktop title bar owns appearance, account, and native window controls; page content must not add a second global header.
- Support `system`, `light`, and `dark` appearance preferences. Persist the preference locally and apply it before Vue mounts to avoid a theme flash.
- Dark mode uses five distinct layers: page, region, card, inset/control, and overlay. It never uses pure black or relies on shadows alone for separation.
- Hard-coded light status fills are not allowed; selected, success, warning, danger, info, progress, backdrop, and grid treatments must resolve through semantic tokens.
- Use the bundled Chinese UI face for product text and a separate mechanical display face for the brand/compact labels.
- Use small radii, crisp outlines, selective elevation, and varied spacing rhythm. Do not wrap every region in a floating card.
- Preserve stable transfer columns and state-specific row actions.
- Show queue totals in one place only. Transfer filter tabs own status counts; the sidebar footer may show aggregate speed and a short overall state, but the top bar and page must not repeat the same metrics.
- Keep form labels and controls on shared vertical tokens so native fields and Nuxt UI selects align exactly.
- Use the shared dropdown primitive for transient menus; native details are reserved for persistent inline disclosure content.
- Adapt navigation and toolbars for narrow desktop windows instead of hiding critical actions.
- Every custom interactive surface needs focus-visible styling, accessible naming, keyboard behavior, and a reduced-motion path.
- Focus treatments inside clipped or scrollable regions must render inward; adjacent task-row icon actions must not create a stack of competing outlines.
- Do not use gradient text, decorative glassmorphism, generic glow, or thick colored side stripes.

## Product boundary

Visual work must not move download truth into Vue. Rust continues to own resolution, planning, task state, fetching, persistence, recovery, diagnostics, and account storage.
