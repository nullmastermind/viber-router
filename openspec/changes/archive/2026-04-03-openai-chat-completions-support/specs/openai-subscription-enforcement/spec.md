## ADDED Requirements

### Requirement: Subscription budget check applies to OpenAI chat completions
The proxy SHALL apply the same subscription budget check to `/v1/chat/completions` requests from sub-keys as it does to `/v1/messages`. The check SHALL use `is_billing_endpoint(path)` to determine whether to run the subscription check.

#### Scenario: Sub-key on OpenAI path with active subscription and budget
- **WHEN** a sub-key sends a request to `/v1/chat/completions` and has an active subscription with remaining budget
- **THEN** the system SHALL allow the request and select the subscription for cost tracking

#### Scenario: Sub-key on OpenAI path with exhausted subscription
- **WHEN** a sub-key sends a request to `/v1/chat/completions` and all subscriptions are exhausted or expired
- **THEN** the system SHALL return HTTP 429 with OpenAI-format error: `{"error":{"message":"Subscription limit exceeded","type":"rate_limit_error","param":null,"code":null}}`

#### Scenario: Master key on OpenAI path bypasses subscription check
- **WHEN** a master key (no `group_key_id`) sends a request to `/v1/chat/completions`
- **THEN** the system SHALL NOT perform a subscription check (same as for `/v1/messages`)

### Requirement: Telegram alerts fire for OpenAI billing endpoint errors
The proxy SHALL send Telegram error alerts for failed requests to `/v1/chat/completions` using the same conditions as for `/v1/messages` (non-zero final status after all servers exhausted, or 400 upstream error).

#### Scenario: All servers exhausted on OpenAI path
- **WHEN** all servers in the failover chain fail for a `/v1/chat/completions` request
- **THEN** the system SHALL spawn a Telegram alert with the same fields as for a `/v1/messages` failure

#### Scenario: 400 upstream error on OpenAI path
- **WHEN** an upstream server returns 400 for a `/v1/chat/completions` request
- **THEN** the system SHALL spawn a Telegram alert (same as for `/v1/messages`)
