## Purpose
TBD

## Requirements
### Requirement: Per-server uptime bars in group detail
The GroupDetailPage servers tab SHALL display a row of 90 colored bars under each server, representing uptime over the last 45 hours (30-minute buckets).

#### Scenario: Bar colors
- **WHEN** a bucket has >95% success rate
- **THEN** the bar SHALL be green

#### Scenario: Degraded bar
- **WHEN** a bucket has 50-95% success rate
- **THEN** the bar SHALL be yellow/amber

#### Scenario: Failed bar
- **WHEN** a bucket has <50% success rate
- **THEN** the bar SHALL be red

#### Scenario: No data bar
- **WHEN** a bucket has 0 requests
- **THEN** the bar SHALL be gray

#### Scenario: Bar tooltip
- **WHEN** a user hovers over a bar
- **THEN** a tooltip SHALL display the time range, total requests, successful requests, and success percentage

#### Scenario: Uptime data loading failure
- **WHEN** the uptime API request fails
- **THEN** the system SHALL display "Unable to load status" with a retry button, without blocking the rest of the page

#### Scenario: Uptime data loaded on mount
- **WHEN** the GroupDetailPage loads
- **THEN** the system SHALL fetch uptime data and display bars under each server in the servers tab

### Requirement: Uptime bars accessibility
The uptime bars component SHALL include appropriate ARIA attributes for screen reader accessibility.

#### Scenario: Screen reader support
- **WHEN** a screen reader encounters the uptime bars
- **THEN** each bar SHALL have an aria-label describing the time range and status (e.g., "Mar 28 10:00-10:30: 98% uptime, 100 requests")
