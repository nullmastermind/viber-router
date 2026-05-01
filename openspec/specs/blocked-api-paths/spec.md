## Purpose
TBD

## Requirements
### Requirement: Blocked paths proxy check
The proxy SHALL check incoming request paths against a globally configured list of blocked paths as the very first operation in `proxy_handler`, before API key extraction. If the request path matches any blocked path (exact match on the path component, excluding query string), the proxy SHALL return HTTP 404 with Anthropic-style error JSON: `{"type":"error","error":{"type":"not_found_error","message":"Not found"}}`.

#### Scenario: Request to blocked path
- **WHEN** a request arrives at `/v1/completions` and `/v1/completions` is in the blocked paths list
- **THEN** the proxy SHALL return HTTP 404 with body `{"type":"error","error":{"type":"not_found_error","message":"Not found"}}` without extracting the API key or performing any further processing

#### Scenario: Request to blocked path with query string
- **WHEN** a request arrives at `/v1/completions?stream=true` and `/v1/completions` is in the blocked paths list
- **THEN** the proxy SHALL return HTTP 404 (path matching uses `original_uri.path()` which excludes query string)

#### Scenario: Request to non-blocked path
- **WHEN** a request arrives at `/v1/messages` and `/v1/messages` is NOT in the blocked paths list
- **THEN** the proxy SHALL proceed with normal processing (API key extraction, group resolution, etc.)

#### Scenario: Empty blocked paths list
- **WHEN** a request arrives and the blocked paths list is empty
- **THEN** the proxy SHALL proceed with normal processing for all paths

### Requirement: Blocked paths Redis cache
The system SHALL cache the blocked paths list in Redis under key `settings:blocked_paths` as a JSON array of strings. The proxy SHALL load this list via a single Redis GET on each request.

#### Scenario: Cache hit
- **WHEN** a proxy request arrives and Redis key `settings:blocked_paths` exists
- **THEN** the proxy SHALL parse the JSON array and use it for path matching

#### Scenario: Cache miss
- **WHEN** a proxy request arrives and Redis key `settings:blocked_paths` does not exist
- **THEN** the proxy SHALL query the `settings` table for `blocked_paths`, cache the result in Redis, and use it for path matching

#### Scenario: Redis unavailable
- **WHEN** a proxy request arrives and Redis is unreachable
- **THEN** the proxy SHALL allow the request through (fail-open) and proceed with normal processing

### Requirement: Admin blocked paths management
The admin UI Settings page SHALL include a "Blocked API Paths" section that allows admins to add and remove paths from the blocked list. Paths are managed as chips (similar to the existing Telegram chat IDs pattern).

#### Scenario: View blocked paths
- **WHEN** the admin navigates to the Settings page
- **THEN** the blocked paths SHALL be displayed as removable chips, or "No blocked paths" if the list is empty

#### Scenario: Add a blocked path
- **WHEN** the admin types `/v1/completions` into the input field and presses Enter or clicks Add
- **THEN** `/v1/completions` SHALL be added as a chip in the blocked paths list (not yet saved to server)

#### Scenario: Remove a blocked path
- **WHEN** the admin clicks the remove button on a blocked path chip
- **THEN** the path SHALL be removed from the list (not yet saved to server)

#### Scenario: Save blocked paths
- **WHEN** the admin clicks "Save Settings" with modified blocked paths
- **THEN** the system SHALL include `blocked_paths` in the PUT `/api/admin/settings` request, and the server SHALL update the database and invalidate the Redis cache
