## MODIFIED Requirements

### Requirement: Pre-request subscription budget check
The proxy SHALL check subscription budgets before forwarding a request when the sub-key has any subscriptions and the request path is a billing endpoint (`/v1/messages` or `/v1/chat/completions`). The check SHALL use Redis counters for performance. If the check returns `Allowed`, it SHALL include the selected subscription's optional `tpm_limit` so the proxy can perform TPM wait enforcement before forwarding. If the check returns `BonusServers`, the proxy SHALL NOT block the request and SHALL NOT apply TPM wait enforcement — instead it SHALL attempt bonus servers before the group server waterfall (see proxy-engine spec).

#### Scenario: Sub-key with no subscriptions
- **WHEN** a proxy request uses a sub-key that has never had any subscriptions
- **THEN** the system SHALL allow the request without any budget check (unlimited)

#### Scenario: Sub-key with active subscription and available budget
- **WHEN** a proxy request uses a sub-key with an active non-bonus subscription that has remaining budget
- **THEN** the system SHALL allow the request and return the selected subscription's optional TPM limit with the allowed result

#### Scenario: Sub-key with active bonus subscription
- **WHEN** a proxy request uses a sub-key that has at least one active bonus subscription
- **THEN** the subscription check SHALL return `BonusServers` instead of `Allowed` or `Blocked`, and TPM enforcement SHALL NOT apply to the bonus subscription

#### Scenario: All subscriptions exhausted or expired — Anthropic path
- **WHEN** a proxy request to `/v1/messages` uses a sub-key where all non-bonus subscriptions are in terminal states and there are no bonus subscriptions
- **THEN** the system SHALL return 429 with Anthropic-format error: `{"type":"error","error":{"type":"rate_limit_error","message":"Subscription limit exceeded"}}`

#### Scenario: All subscriptions exhausted or expired — OpenAI path
- **WHEN** a proxy request to `/v1/chat/completions` uses a sub-key where all subscriptions are in terminal states and there are no bonus subscriptions
- **THEN** the system SHALL return 429 with OpenAI-format error: `{"error":{"message":"Subscription limit exceeded","type":"rate_limit_error","param":null,"code":null}}`

#### Scenario: Non-billing endpoint bypasses subscription check
- **WHEN** a proxy request uses a sub-key but the path is not `/v1/messages` or `/v1/chat/completions`
- **THEN** the system SHALL NOT perform a subscription check

### Requirement: Subscription selection priority
The proxy SHALL select the charging subscription using this priority: hourly_reset subscriptions first, then pay_per_request subscriptions, then fixed subscriptions. Within the same type, FIFO (oldest `created_at` first). Bonus subscriptions SHALL be excluded from this priority ordering and handled separately via the `BonusServers` result variant. TPM limits SHALL NOT cause the selection loop to skip a selected non-bonus subscription; TPM wait enforcement occurs after selection.

#### Scenario: Hourly_reset preferred over fixed
- **WHEN** a sub-key has both an active hourly_reset subscription with window budget and an active fixed subscription with budget
- **THEN** the system SHALL charge the hourly_reset subscription

#### Scenario: Pay_per_request preferred over fixed
- **WHEN** a sub-key has both an active pay_per_request subscription with budget and an active fixed subscription with budget
- **THEN** the system SHALL charge the pay_per_request subscription

#### Scenario: Hourly window full, fall through to fixed
- **WHEN** a sub-key has an hourly_reset subscription with current window exhausted and a fixed subscription with budget
- **THEN** the system SHALL skip the hourly_reset and charge the fixed subscription

#### Scenario: FIFO within same type
- **WHEN** a sub-key has two active fixed subscriptions, one created before the other
- **THEN** the system SHALL charge the older subscription first

#### Scenario: Selected subscription waits instead of falling through on TPM
- **WHEN** the selected non-bonus subscription is at its TPM limit and a lower-priority subscription has available budget
- **THEN** the proxy SHALL wait for the selected subscription's TPM window rather than skipping to the lower-priority subscription
