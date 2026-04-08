## Context

`PublicUsagePage.vue` is a self-contained single-file component that displays sub-key usage data fetched from `/api/public/usage`. It already manages multiple local `ref` states (loading, data, timers) and uses `setInterval` for a countdown clock and a 60-second auto-refresh poll. The page uses Quasar components throughout.

The usage meter is a purely frontend addition. No backend endpoints need to change. The existing `fetchUsage` function already accepts a `silent` flag that suppresses loading state, which is exactly the pattern needed for the stop re-fetch.

## Goals / Non-Goals

**Goals:**
- Add a start/stop usage meter to `PublicUsagePage.vue` as a new card section.
- Capture a snapshot of `data.value.usage` on start.
- Track elapsed time with a 1-second `setInterval`, displayed as HH:MM:SS.
- On stop: re-fetch usage silently, compute per-model deltas, show a `q-dialog` with results.
- Clean up the interval in `onUnmounted`.

**Non-Goals:**
- No backend changes.
- No new Pinia store.
- No new npm dependencies.
- No persistence of meter state across page reloads.
- No support for pausing the meter.

## Decisions

### Local refs only (no store)
All meter state (`meterRunning`, `meterSnapshot`, `meterStartTime`, `meterElapsed`, `meterInterval`, `showMeterDialog`, `meterDelta`) lives as component-local `ref`s. The meter is a UI convenience tool with no need for cross-component state.

Alternatives considered: Pinia store — rejected because the meter is only relevant on this page and has no shared consumers.

### Reuse existing silent re-fetch pattern
On stop, call `api.get('/api/public/usage', { params: { key } })` directly (same as the existing `fetchUsage` silent path) rather than calling `fetchUsage` itself, to avoid side effects on `data.value` before the delta is computed. The snapshot must be compared against fresh data before `data.value` is updated.

Alternatives considered: calling `fetchUsage(key, true)` and watching `data` — rejected because the watcher timing is non-deterministic and would complicate delta computation.

### Delta computation strategy
For each model in the fresh response, look up the matching model in the snapshot by `model` field (string or null). Subtract snapshot values. Models that appear in fresh data but not in the snapshot are treated as having snapshot values of zero (new activity). Models only in the snapshot (disappeared from fresh data) are ignored.

### HH:MM:SS elapsed display
Elapsed seconds are stored as a single `number` ref and formatted with a computed or inline formatter. This is simpler than storing a `Date` and avoids drift from repeated `Date.now()` calls.

### Results dialog
Use `q-dialog` + `q-table` with a `deltaColumns` definition mirroring `usageColumns`. Show total elapsed time as a subtitle above the table. Only show rows where at least one delta value is non-zero, to keep the dialog clean; if all deltas are zero, show all rows (or a "no new usage" message).

## Risks / Trade-offs

- [Clock drift] The 1-second `setInterval` may drift slightly over long sessions. Mitigation: elapsed seconds are incremented by 1 each tick rather than recomputed from wall clock, so display is consistent even if ticks are slightly late. Absolute accuracy is not required for this tool.
- [Re-fetch failure on stop] If the API call on stop fails, the dialog cannot be shown. Mitigation: show a `q-notify` error and reset the meter to stopped state without showing the dialog.
- [Snapshot staleness] The 60-second auto-refresh updates `data.value` while the meter is running, but the snapshot is not updated. This is intentional — the snapshot is fixed at start time. The delta will still be correct because the stop re-fetch is independent.
