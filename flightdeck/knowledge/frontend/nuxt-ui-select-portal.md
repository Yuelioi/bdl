# ⚠ Inline Nuxt UI select content shifts BDL forms
SUMMARY: Keep BDL `USelect` content portalled; `portal=false` inserts the popup into the field layout and pushes later settings downward when the menu opens.
READ WHEN: when a select menu opening moves nearby fields, changes scroll position, clips inside a panel, or when editing the local Select wrapper.
RECHECK WHEN: Nuxt UI changes the default USelect portal or popper contract.

---

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

