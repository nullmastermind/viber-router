## Context

The Meter Results dialog in `src/pages/PublicUsagePage.vue` is a Quasar `q-dialog` containing a `q-card` with a 5-column `q-markup-table`. The card currently has `min-width: 480px; max-width: 95vw`. On viewports narrower than 480px (most phones), `min-width` wins and the dialog overflows the screen.

## Goals / Non-Goals

**Goals:**
- Dialog fits within the viewport on all screen sizes.
- On desktop (>= 480px wide), dialog is capped at 480px.
- On mobile (< 480px wide), dialog fills 95% of the viewport.
- Table content remains readable; horizontal scroll is available if columns overflow.

**Non-Goals:**
- Redesigning the table layout or column structure.
- Responsive column hiding or stacking.
- Any backend or state management changes.

## Decisions

**CSS sizing strategy: `width: 95vw; max-width: 480px`**

Using `width` as the base value with `max-width` as the cap is the standard responsive pattern for dialogs. It replaces the conflicting `min-width` + `max-width` pair where `min-width` was overriding `max-width` on narrow viewports. No alternatives were considered — this is the canonical fix.

**Horizontal scroll on the table section: `overflow-x: auto`**

Applied to the `q-card-section` wrapping the `q-markup-table`. This allows the 5-column table to scroll horizontally on very narrow screens (e.g., < 360px) rather than truncating or wrapping content in a way that breaks readability. The alternative (wrapping columns) would require restructuring the table markup and is out of scope.

## Risks / Trade-offs

- [Minimal visual change on desktop] The dialog width behavior on desktop is unchanged (capped at 480px). On mobile it now fits the screen instead of overflowing. No visual regression expected.

## Migration Plan

Single-file CSS attribute change. No migration needed. Rollback is reverting the two attribute edits.
