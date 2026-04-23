## 1. Backend: Extend Public Uptime API

- [x] 1.1 Add `ModelUptime` struct with fields: `model` (String), `status` (String), `uptime_percent` (f64), `buckets` (Vec<ChainBucket>)
- [x] 1.2 Add `models: Vec<ModelUptime>` field to `PublicUptimeResponse`
- [x] 1.3 Query `group_allowed_models` joined with `models` table to get the list of allowed model names for the group
- [x] 1.4 Query `proxy_logs` grouped by `request_model` and 30-min bucket for the 45-hour window, filtering by `group_id` and `created_at >= cutoff`, counting total rows and rows with `status_code BETWEEN 200 AND 299`
- [x] 1.5 Build per-model bucket arrays: for each allowed model, create 90 buckets populated from the query results (zero-fill missing buckets)
- [x] 1.6 Derive per-model status and uptime_percent from the last bucket using the same threshold logic as overall status (>95% operational, >=50% degraded, <50% down, 0 requests no_data)
- [x] 1.7 Populate the `models` array in the response, including models with no traffic as `no_data` with zeroed buckets <- (verify: response includes all allowed models, status derivation matches spec thresholds, models not in allowed list are excluded)

## 2. Frontend: Render Per-Model Status

- [x] 2.1 Extend `UptimeApiResponse` TypeScript interface to include `models?: { model: string; status: string; uptime_percent: number; buckets: UptimeBucketRaw[] }[]`
- [x] 2.2 Add template section below overall status bars: a `q-separator` followed by per-model rows, each with model name, status badge (using `statusBadgeColor`/`statusBadgeLabel`), and `<UptimeBars>` component
- [x] 2.3 Add computed property to map each model's buckets to `Bucket[]` format (`{ timestamp, total, success }`) for the `UptimeBars` component
- [x] 2.4 Conditionally render the per-model section only when `uptimeData.models` exists and has length > 0 <- (verify: per-model rows render correctly with all badge states, UptimeBars shows correct data, empty models array hides section cleanly)

## 3. Validation

- [x] 3.1 Run `just check` to ensure no type errors or lint failures <- (verify: both frontend and backend pass type-check and lint with zero errors)
