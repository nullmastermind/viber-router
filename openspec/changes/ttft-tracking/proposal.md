## Why

The viber-router proxy forwards LLM API requests through a failover waterfall of upstream servers, but has no visibility into Time to First Token (TTFT) — the delay between sending a request and receiving the first SSE streaming chunk. Without TTFT metrics, operators cannot identify slow upstream servers or automatically route around them. Additionally, slow TTFT wastes money because the upstream provider may be processing (and billing) a request that the user perceives as hung.

## What Changes

- Measure TTFT on every SSE streaming request: time from receiving upstream HTTP 200 headers to the first SSE data chunk arriving
- Persist TTFT measurements to a new partitioned `ttft_logs` database table (one row per SSE request, per server attempted)
- Add `ttft_timeout_ms` configuration to groups: when set, the proxy will abort an SSE connection if the first chunk does not arrive within the threshold and try the next server in the failover waterfall (closing the old connection to stop upstream billing)
- Auto-switch safety: skip TTFT timeout logic when the group has only one server or when attempting the last server in the waterfall (always wait indefinitely on the last server)
- New admin API endpoint to query TTFT statistics per server within a time window
- Chart.js visualization in the Group Detail UI showing per-server TTFT over the last hour
- Generalize the partition management system to support multiple partitioned tables

## Capabilities

### New Capabilities
- `ttft-measurement`: Measuring and recording Time to First Token for SSE streaming requests, including the auto-switch timeout mechanism
- `ttft-stats-api`: Admin API endpoint for querying aggregated TTFT statistics per server
- `ttft-ui-chart`: Chart.js visualization of TTFT data in the Group Detail page

### Modified Capabilities
- `proxy-engine`: The proxy failover loop gains TTFT-aware auto-switch behavior for SSE streams (new timeout + drop + continue logic before `build_response`)

## Impact

- **Backend**: `proxy.rs` (SSE detection moves partially into proxy_handler, TTFT timeout logic added to failover loop), `partition.rs` (generalized for multiple tables), new `ttft_buffer.rs`, new `routes/admin/ttft.rs`, `models/group.rs` and `models/group_server.rs` (new field), `cache.rs` (GroupConfig gains ttft_timeout_ms), `main.rs` (second mpsc channel + flush task)
- **Database**: New migration adding `groups.ttft_timeout_ms` column and `ttft_logs` partitioned table
- **Frontend**: New Chart.js + vue-chartjs dependencies, new TTFT chart card in GroupDetailPage.vue, groups store updated with ttft_timeout_ms field and fetchTtftStats action
- **APIs**: New `GET /api/admin/ttft-stats`, updated `PUT /api/admin/groups/:id` accepts ttft_timeout_ms, updated `GET /api/admin/groups/:id` returns ttft_timeout_ms
