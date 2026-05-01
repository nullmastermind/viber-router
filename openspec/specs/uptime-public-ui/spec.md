## Purpose
TBD

## Requirements
### Requirement: Chain-level status display on public page
The PublicUsagePage SHALL display a "Status" section showing the current chain-level status text and 90 uptime bars.

#### Scenario: Status section layout
- **WHEN** the public usage page loads with a valid key
- **THEN** the page SHALL display a "Status" heading, a status badge ("Operational"/"Degraded"/"Down"/"No data") with appropriate color, and a row of 90 status bars below

#### Scenario: Operational status badge
- **WHEN** the chain status is "operational"
- **THEN** the badge SHALL be green with text "Operational"

#### Scenario: Degraded status badge
- **WHEN** the chain status is "degraded"
- **THEN** the badge SHALL be amber/yellow with text "Degraded"

#### Scenario: Down status badge
- **WHEN** the chain status is "down"
- **THEN** the badge SHALL be red with text "Down"

#### Scenario: No data status badge
- **WHEN** the chain status is "no_data"
- **THEN** the badge SHALL be gray with text "No data"

#### Scenario: Status bars use same component
- **WHEN** the public page renders uptime bars
- **THEN** it SHALL use the same UptimeBars component as the admin page

#### Scenario: Uptime data loading failure
- **WHEN** the uptime API request fails
- **THEN** the system SHALL display "Unable to load status" with a retry button, without blocking the rest of the page

#### Scenario: Auto-refresh includes uptime
- **WHEN** the existing 60-second auto-refresh fires
- **THEN** the uptime data SHALL also be refreshed
