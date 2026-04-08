## Why

The Meter Results dialog uses `min-width: 480px` which overrides `max-width: 95vw` on mobile viewports, causing the dialog to overflow the screen and break the layout for users on narrow devices.

## What Changes

- Replace `min-width: 480px; max-width: 95vw` with `width: 95vw; max-width: 480px` on the `q-card` in the Meter Results dialog so it fills 95% of the viewport on mobile and caps at 480px on desktop.
- Add `overflow-x: auto` to the `q-card-section` wrapping the `q-markup-table` so the 5-column table scrolls horizontally on very narrow screens instead of breaking layout.

## Capabilities

### New Capabilities

- `meter-dialog-mobile`: Responsive Meter Results dialog that fits within the viewport on mobile screens.

### Modified Capabilities

<!-- No existing spec-level behavior changes. -->

## Impact

- `src/pages/PublicUsagePage.vue` — lines 273 and 278 only. No backend changes, no new dependencies, no new files.
