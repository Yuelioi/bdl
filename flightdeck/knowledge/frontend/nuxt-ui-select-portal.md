# ⚠ Inline Nuxt UI select content shifts BDL forms

## Symptom

Opening a select such as audio quality moves the controls below it downward. The menu behaves like expanding form content instead of a floating popup.

## Root cause

`apps/desktop/src/ui/Select.vue` explicitly used `:portal="false"`. The select content therefore remained inside the grid-based field and settings scroll container.

## Fix

- Keep `portal` enabled.
- Use popper positioning with start alignment, collision padding, and a small side offset.
- Give the trigger a fixed token height through the Nuxt UI `base` slot.
- Give every local field wrapper the same label-row height and label/control gap.

Verify by capturing the settings layout before and after opening the menu: fields below the trigger must keep identical coordinates.
