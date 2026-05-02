# per-model-uptime-api Specification

## Purpose
TBD - created by archiving change per-model-status-breakdown. Update Purpose after archive.
## Requirements
### Requirement: Per-model uptime data in public uptime response
The `GET /api/public/uptime` endpoint SHALL include a `models` array in the response, where each element contains per-model uptime data derived from the `proxy_logs` table.

#### Scenario: Response includes models array
- **WHEN** a user sends `GET /api/public/uptime?key=sk-vibervn-abc123` with a valid active sub-key
- **THEN** the response SHALL include a `models` array alongside the existing `status`, `uptime_percent`, and `buckets` fields

#### Scenario: Per-model entry structure
- **WHEN** the response includes a model entry
- **THEN** each entry SHALL contain `model` (string), `status` (string), `uptime_percent` (float), and `buckets` (array of 90 `{ timestamp, total_requests, successful_requests }` objects)

### Requirement: Per-model data sourced from proxy_logs
The per-model uptime data SHALL be derived from the `proxy_logs` table, grouping by `request_model` and using the same 90 x 30-minute bucket structure as the overall uptime.

#### Scenario: Successful request definition
- **WHEN** a `proxy_logs` row has `status_code` between 200 and 299
- **THEN** that request SHALL be counted as successful for per-model aggregation

#### Scenario: Failed request definition
- **WHEN** a `proxy_logs` row has `status_code` outside the 200-299 range
- **THEN** that request SHALL be counted as failed for per-model aggregation

#### Scenario: Bucket time window
- **WHEN** per-model buckets are generated
- **THEN** the system SHALL use 90 buckets of 30 minutes each (same as overall uptime), covering the most recent 45 hours

### Requirement: Only allowed models included
The per-model response SHALL only include models that are in the group's allowed models list, determined by joining `group_allowed_models` and `models` tables using the group's `group_id`.

#### Scenario: Model in allowed list with traffic
- **WHEN** the group allows model "claude-sonnet-4-20250514" and `proxy_logs` contains requests for that model
- **THEN** the model SHALL appear in the `models` array with its computed status and buckets

#### Scenario: Model in allowed list without traffic
- **WHEN** the group allows model "claude-opus-4-20250514" but `proxy_logs` contains no requests for that model in the 45-hour window
- **THEN** the model SHALL appear in the `models` array with `status: "no_data"`, `uptime_percent: 0.0`, and all buckets with `total_requests: 0` and `successful_requests: 0`

#### Scenario: Model not in allowed list excluded
- **WHEN** `proxy_logs` contains requests for model "gpt-4" but this model is NOT in the group's allowed models
- **THEN** "gpt-4" SHALL NOT appear in the `models` array

### Requirement: Per-model status derivation
Per-model status SHALL be derived from the most recent 30-minute bucket using the same thresholds as overall status.

#### Scenario: Operational per-model status
- **WHEN** the most recent bucket for a model has more than 95% successful requests
- **THEN** the model's `status` SHALL be "operational"

#### Scenario: Degraded per-model status
- **WHEN** the most recent bucket for a model has 50% to 95% successful requests (inclusive of 50%)
- **THEN** the model's `status` SHALL be "degraded"

#### Scenario: Down per-model status
- **WHEN** the most recent bucket for a model has less than 50% successful requests
- **THEN** the model's `status` SHALL be "down"

#### Scenario: No data per-model status
- **WHEN** the most recent bucket for a model has 0 requests
- **THEN** the model's `status` SHALL be "no_data"

