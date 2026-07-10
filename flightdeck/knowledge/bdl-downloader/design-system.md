# BDL design-system checklist
SUMMARY: Always shape BDL as a quiet desktop control room: warm mineral OKLCH neutrals, restrained green signal color, clear command hierarchy, adaptive navigation, stable transfer rows, and progressive disclosure.
READ WHEN: before any BDL UI, interaction, visual-token, responsive-layout, or user-facing copy change.

---

## Direction

BDL is a compact operations tool for Bilibili power users. It should feel calm, mechanical, and decisive—not like a marketing page, generic admin dashboard, or neon developer tool.

## Standing rules

- Keep Parse, Transfer, and Settings as the only primary pages; account remains global.
- Put current status and the recommended action before logs or technical detail.
- Use OKLCH semantic tokens and neutrals tinted toward the restrained green brand hue.
- Use the bundled Chinese UI face for product text and a separate mechanical display face for the brand/compact labels.
- Use small radii, crisp outlines, selective elevation, and varied spacing rhythm. Do not wrap every region in a floating card.
- Preserve stable transfer columns and state-specific row actions.
- Adapt navigation and toolbars for narrow desktop windows instead of hiding critical actions.
- Every custom interactive surface needs focus-visible styling, accessible naming, keyboard behavior, and a reduced-motion path.
- Do not use gradient text, decorative glassmorphism, generic glow, or thick colored side stripes.

## Product boundary

Visual work must not move download truth into Vue. Rust continues to own resolution, planning, task state, fetching, persistence, recovery, diagnostics, and account storage.

