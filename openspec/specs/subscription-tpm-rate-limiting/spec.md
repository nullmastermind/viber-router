## Purpose
TBD

## Requirements
### Requirement: TPM limit storage
The system SHALL support an optional `tpm_limit` on subscription plans and key subscriptions. A null `tpm_limit` SHALL mean no tokens-per-minute limit applies.

#### Scenario: Migration adds TPM columns
- **WHEN** database migration 044 runs
- **THEN** `subscription_plans` and `key_subscriptions` SHALL each include nullable `tpm_limit FLOAT8` columns

#### Scenario: Existing records remain unlimited
- **WHEN** an existing plan or subscription has `tpm_limit = NULL`
- **THEN** the system SHALL treat it as having no TPM limit

### Requirement: TPM Redis fixed-window counter
The system SHALL track subscription token usage in Redis with key `sub_tpm:{subscription_id}` using a 60-second fixed-window integer counter.

#### Scenario: First increment starts window
- **WHEN** the system increments `sub_tpm:{subscription_id}` by a token count and the returned total equals the added token count
- **THEN** the system SHALL set the key TTL to 60 seconds

#### Scenario: Existing window increment preserves reset time
- **WHEN** the system increments `sub_tpm:{subscription_id}` and the key already existed
- **THEN** the system SHALL NOT extend the existing TTL

### Requirement: TPM wait enforcement
The proxy SHALL enforce TPM after a non-bonus subscription has been selected for a billing request. If the selected subscription has a TPM limit and its current Redis counter is greater than or equal to that limit, the proxy SHALL wait for the Redis key TTL to elapse and retry the TPM check up to five times before returning 429.

#### Scenario: TPM under limit proceeds
- **WHEN** a selected non-bonus subscription has `tpm_limit = 100000` and `sub_tpm:{subscription_id}` is below `100000`
- **THEN** the proxy SHALL proceed to the upstream server waterfall without waiting

#### Scenario: TPM at limit waits for reset
- **WHEN** a selected non-bonus subscription has `tpm_limit = 100000`, `sub_tpm:{subscription_id}` is at least `100000`, and the Redis key TTL is positive
- **THEN** the proxy SHALL asynchronously sleep until the TTL elapses and then retry the TPM check

#### Scenario: TPM remains limited after retries
- **WHEN** the selected subscription remains at or above its TPM limit after five wait retries
- **THEN** the proxy SHALL return HTTP 429 with message `TPM limit exceeded, please retry later`

### Requirement: TPM token accounting
The proxy SHALL add actual input plus output tokens to the selected subscription's TPM counter after a billing response completes and token usage has been parsed.

#### Scenario: Non-streaming response increments TPM
- **WHEN** a non-streaming billing response completes with `input_tokens = 1000` and `output_tokens = 250`
- **THEN** the system SHALL increment `sub_tpm:{subscription_id}` by `1250`

#### Scenario: Streaming response increments after completion
- **WHEN** an SSE billing response completes and parsed usage totals are `input_tokens = 1000` and `output_tokens = 250`
- **THEN** the system SHALL increment `sub_tpm:{subscription_id}` by `1250` after the stream completes

#### Scenario: Missing token usage is not estimated
- **WHEN** a completed billing response does not provide parseable input and output token counts
- **THEN** the system SHALL NOT estimate tokens for TPM accounting

### Requirement: TPM fail-open behavior
The system SHALL fail open for TPM when Redis operations are unavailable or return errors.

#### Scenario: Redis unavailable during TPM check
- **WHEN** Redis cannot read `sub_tpm:{subscription_id}` or its TTL during TPM enforcement
- **THEN** the proxy SHALL allow the request to proceed

#### Scenario: Redis unavailable during TPM increment
- **WHEN** Redis cannot increment `sub_tpm:{subscription_id}` after a response completes
- **THEN** the proxy SHALL not fail the completed request because of the TPM increment error

### Requirement: TPM scope excludes bonus subscriptions
TPM limits SHALL apply only to non-bonus subscriptions of type `fixed`, `hourly_reset`, and `pay_per_request`. Bonus subscriptions SHALL NOT be TPM limited.

#### Scenario: Bonus subscription is selected for bonus routing
- **WHEN** a sub-key has a bonus subscription and the subscription check returns bonus routing information
- **THEN** the proxy SHALL NOT apply TPM wait enforcement to that bonus subscription
