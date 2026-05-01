## MODIFIED Requirements

### Requirement: Subscription cards display
The page SHALL display subscription information as cards, with active subscriptions first and inactive ones (expired/cancelled/exhausted) visually dimmed. Non-bonus subscription cards SHALL show lifetime cost progress and, when `weekly_cost_limit_usd` is present, weekly cost usage, weekly limit, and weekly reset time. Bonus subscriptions SHALL be rendered with a distinct card style separate from budget-based subscriptions.

#### Scenario: Active subscription card
- **WHEN** a subscription has status `active` and `sub_type` is not `bonus`
- **THEN** the card shows: subscription type, cost_used / cost_limit_usd (as progress), weekly_cost_used / weekly_cost_limit_usd when present, weekly_reset_at when present, status badge, and expires_at (if set)

#### Scenario: Weekly limit hidden when unlimited
- **WHEN** a non-bonus subscription has `weekly_cost_limit_usd: null`
- **THEN** the card SHALL NOT display a weekly limit progress indicator

#### Scenario: Hourly reset subscription shows countdown
- **WHEN** a subscription has type `hourly_reset` and `window_reset_at` is set
- **THEN** the card displays the remaining time until quota reset (e.g., "Resets in 2h 15m")

#### Scenario: Weekly reset shown
- **WHEN** a non-bonus subscription has `weekly_reset_at` set
- **THEN** the card displays the remaining time or formatted time until the weekly cost limit resets

#### Scenario: Inactive subscription card
- **WHEN** a subscription has status `expired`, `cancelled`, or `exhausted`
- **THEN** the card is visually dimmed (reduced opacity) and shows the status badge

#### Scenario: No subscriptions
- **WHEN** the sub-key has no subscriptions
- **THEN** the page shows "No subscriptions" in the subscriptions section

#### Scenario: Bonus subscription card — with quota data
- **WHEN** a subscription has `sub_type = 'bonus'` and `bonus_quotas` is a non-empty array
- **THEN** the card shows `bonus_name` as title with a lightning bolt icon, one q-linear-progress bar per quota entry with utilization %, quota name label, and a reset countdown if `reset_at` is present, and a list of `bonus_usage` model/count pairs

#### Scenario: Bonus subscription card — empty quotas (fetch failed)
- **WHEN** a subscription has `sub_type = 'bonus'` and `bonus_quotas` is an empty array
- **THEN** the card shows "Quota info unavailable" in the quota section but still renders `bonus_usage`

#### Scenario: Bonus subscription card — null quotas (no URL configured)
- **WHEN** a subscription has `sub_type = 'bonus'` and `bonus_quotas` is null
- **THEN** the quota section of the bonus card is not rendered
