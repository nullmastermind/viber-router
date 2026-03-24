## Context

The viber-router proxy forwards LLM API requests through a priority-based failover waterfall. Currently it measures `latency_ms` (total failover loop time) and `server_latency` (time to receive HTTP headers from a single upstream), but neither captures Time to First Token (TTFT) — the delay between receiving upstream HTTP 200 headers and the first SSE data chunk. This gap matters because LLM providers often return headers quickly but take hundreds of milliseconds to seconds before generating the first token, and this "thinking time" varies significantly between providers and models.

The proxy already has: a partitioned `proxy_logs` table with batch-flush via mpsc channel, a Redis-cached `GroupConfig` struct, and a `build_response` function that detects SSE streams and pipes them to the client.

## Goals / Non-Goals

**Goals:**
- Measure TTFT (headers-received → first-SSE-chunk) on every SSE streaming request
- Persist TTFT data for per-server analysis over time
- Allow groups to configure a TTFT timeout that triggers automatic failover to the next server
- Provide a UI chart showing per-server TTFT in the Group Detail page
- Stop upstream billing when TTFT timeout triggers (by closing the connection)

**Non-Goals:**
- Measuring TTFT for non-streaming requests (not applicable)
- Per-server TTFT timeout thresholds (config is per-group only)
- Historical TTFT trend analysis beyond 1-hour window in UI (data is persisted but UI shows 1h)
- Alerting or notifications based on TTFT thresholds
- Changing the existing `latency_ms` metric in proxy_logs

## Decisions

### 1. TTFT measurement point: after send().await returns, before first stream chunk

**Choice**: Measure from `Instant::now()` right after `upstream_req.send().await` returns (HTTP headers received) to when `stream.next().await` yields the first chunk.

**Why**: This isolates pure LLM thinking time from connection overhead (DNS, TLS, TCP). Connection time is already captured in `server_latency`. Separating the two gives operators clearer signal about which upstream is slow at generation vs. slow to connect.

**Alternative considered**: Measuring from before `send().await` (includes connect time). Rejected because it conflates two different failure modes.

### 2. Separate ttft_logs table (not a column on proxy_logs)

**Choice**: New `ttft_logs` partitioned table with its own mpsc channel and flush task.

**Why**: TTFT is recorded for every SSE request (including successful ones), while proxy_logs only records errors and failovers. Mixing them would either require logging all successful requests to proxy_logs (noisy, schema mismatch) or adding nullable TTFT columns that are mostly NULL. A separate table keeps concerns clean and allows independent retention policies.

**Alternative considered**: Adding `ttft_ms` column to proxy_logs. Rejected because proxy_logs is for error/failover events, not per-request metrics.

### 3. SSE detection moves into proxy_handler for TTFT path

**Choice**: When TTFT timeout is enabled and there are multiple servers, detect SSE in `proxy_handler` before calling `build_response`. Peek the first chunk with `tokio::time::timeout`. If timeout fires, drop the stream and continue the failover loop. If chunk arrives, chain it with the rest of the stream and return.

**Why**: `build_response` currently owns SSE detection and stream consumption. But TTFT auto-switch needs to abort mid-stream and return to the failover loop — which is impossible if `build_response` has already taken ownership. Moving SSE detection up allows the timeout decision before committing to a response.

**Existing path preserved**: When TTFT is disabled (NULL), single server, or last server — the existing `build_response` path is used unchanged. Zero risk to current behavior.

### 4. TTFT timeout skip conditions

**Choice**: Skip TTFT timeout logic when ANY of:
- `ttft_timeout_ms` is NULL on the group (feature disabled)
- Group has only 1 server (nowhere to failover to)
- Current server is the last in the waterfall (must wait — no alternative)

**Why**: The last-server rule prevents infinite timeout loops. The single-server rule is a special case of last-server but makes intent explicit.

### 5. Generalize partition.rs instead of duplicating

**Choice**: Parameterize `ensure_partitions`, `create_partition`, `drop_expired_partitions`, and `parse_partition_end_date` to accept a table name prefix.

**Why**: The file is 77 lines with 4 functions that all hardcode `proxy_logs`. Adding `ttft_logs` would double the code with near-identical logic. Parameterizing is a clean 4-function refactor.

### 6. Chart.js via vue-chartjs for UI

**Choice**: Add `chart.js` and `vue-chartjs` as frontend dependencies. Render a line chart in GroupDetailPage showing TTFT per server over the last hour.

**Why**: Chart.js is lightweight, well-maintained, and the user's explicit choice. vue-chartjs provides Vue 3 component wrappers.

## Risks / Trade-offs

- **[TTFT measurement adds per-chunk overhead]** → Minimal: one `Instant::now()` + one `AtomicBool` check per chunk (~nanoseconds). Only the first chunk triggers the timestamp capture; subsequent chunks pass through untouched.

- **[Auto-switch drops a connection that may have been about to respond]** → Mitigated by making the timeout configurable per group. Operators set thresholds based on their expected TTFT. The last server always waits indefinitely as a safety net.

- **[Dropping stream may not immediately stop upstream billing]** → `drop(reqwest::Response)` sends TCP RST/FIN. Most LLM providers (Anthropic, OpenAI) stop generation on connection close. This is standard behavior but not guaranteed by all providers.

- **[Empty SSE stream after 200 (edge case)]** → If `stream.next()` returns `None` immediately (stream closed without data), treat as connection error and try next server. This handles the rare case of an upstream returning 200 + SSE headers but no data.

- **[Redis cache backward compatibility]** → Adding `ttft_timeout_ms: Option<i32>` to `GroupConfig`. Existing cached entries without this field will fail `serde_json::from_str` deserialization and fall through to DB lookup, which returns the new field. Cache self-heals on next write.
