## Why

Users of the public usage page have no way to measure token consumption or cost for a specific activity window. A usage meter lets them snapshot current usage, perform some work, then stop and see exactly what was consumed — useful for estimating costs per task or session.

## What Changes

- Add a "Usage Meter" card section to `PublicUsagePage.vue` between the "Usage (Last 30 Days)" table and the "Status" section.
- New Start/Stop control: clicking Start captures a snapshot of current usage data and begins an elapsed-time counter; clicking Stop re-fetches live usage and shows a delta results dialog.
- New results dialog: displays per-model deltas (input tokens, output tokens, requests, cost) and total elapsed time.
- All state is local to the component — no new store, no backend changes.

## Capabilities

### New Capabilities

- `usage-meter`: Interactive tool on the public usage page that measures token/cost delta between a start snapshot and a stop re-fetch, with a live elapsed-time display while running.

### Modified Capabilities

<!-- No existing spec-level requirements are changing -->

## Impact

- `src/pages/PublicUsagePage.vue` — only file modified.
- No new dependencies; uses existing Quasar components (`q-card`, `q-btn`, `q-dialog`, `q-table`) and the existing `api.get('/api/public/usage')` call pattern.
- No backend changes.
