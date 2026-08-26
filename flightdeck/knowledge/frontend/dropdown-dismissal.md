# BDL dropdown dismissal checklist

## Rules

- Use `UDropdownMenu` for transient action or selection menus.
- Use native `<details>` only for persistent inline disclosure sections whose expanded content participates in document layout.
- Do not build dropdowns with `v-if` plus absolutely positioned markup unless the interaction cannot be represented by an established primitive.
- Outside click and Escape must close the menu, and focus must return to its trigger.
- Icon-only triggers need an accessible label; menu icons are decorative when the item already has text.

## Why

The previous account menu used a custom `v-if` popover and the Transfer "More" menu used native `<details>`. Neither had the correct transient-menu dismissal lifecycle, so clicking elsewhere left menus open over the work surface.
