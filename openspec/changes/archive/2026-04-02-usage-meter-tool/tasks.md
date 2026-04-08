## 1. State and Types

- [x] 1.1 Add meter state refs: `meterRunning`, `meterSnapshot` (copy of `ModelUsage[]`), `meterStartTime`, `meterElapsed` (seconds), `meterIntervalId`
- [x] 1.2 Add dialog state refs: `showMeterDialog`, `meterDeltaRows` (computed delta array)
- [x] 1.3 Define `MeterDeltaRow` interface: `{ model: string | null, input: number, output: number, requests: number, cost: number }`

## 2. Core Logic

- [x] 2.1 Implement `startMeter()`: copy `data.value.usage` to snapshot, record `Date.now()` as start time, set `meterRunning = true`, start 1-second `setInterval` that increments `meterElapsed`
- [x] 2.2 Implement `stopMeter()`: clear the interval, set `meterRunning = false`, call `api.get('/api/public/usage', { params: { key } })` silently, compute per-model deltas against snapshot, populate `meterDeltaRows`, open dialog; on fetch error show `q-notify` and reset state
- [x] 2.3 Implement `formatElapsed(seconds: number): string` that returns zero-padded HH:MM:SS
- [x] 2.4 Implement delta computation: for each model in fresh data find matching snapshot entry by `model` field, subtract `effective_input_tokens`, `total_output_tokens`, `request_count`, `cost_usd` (treat null as 0) ← (verify: delta values are correct for models present only in fresh data, only in snapshot, and in both; null cost handled without NaN)
- [x] 2.5 Extend `onUnmounted` to clear the meter interval if running

## 3. Template — Meter Card

- [x] 3.1 Add `q-card` section between the usage `q-table` and the "Status" subtitle, with title "Usage Meter"
- [x] 3.2 Add "Start" `q-btn` (shown when `!meterRunning`) that calls `startMeter()`
- [x] 3.3 Add elapsed time display and pulsing indicator (shown when `meterRunning`): render `formatElapsed(meterElapsed)` and a colored dot with CSS `animation: pulse`
- [x] 3.4 Add "Stop" `q-btn` (shown when `meterRunning`) that calls `stopMeter()` ← (verify: start/stop toggle renders correctly, elapsed counter increments visibly, pulsing indicator is visible while running)

## 4. Template — Results Dialog

- [x] 4.1 Add `q-dialog` bound to `showMeterDialog` with a `q-card` containing a subtitle showing total elapsed time
- [x] 4.2 Add `q-table` inside the dialog with `meterDeltaRows` as rows and columns: Model, Input Tokens, Output Tokens, Requests, Cost ($)
- [x] 4.3 Format delta columns: tokens use `formatCompact`, cost uses `$x.xxxx` (matching `usageColumns` style), model null renders as em-dash ← (verify: dialog opens after stop, table shows correct per-model deltas, elapsed time subtitle is accurate)

## 5. Quality Check

- [x] 5.1 Run `just check` and fix all type errors and lint warnings ← (verify: `just check` exits with code 0, no TypeScript errors, no Biome warnings)
