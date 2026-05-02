# per-model-uptime-ui Specification

## Purpose
TBD - created by archiving change per-model-status-breakdown. Update Purpose after archive.
## Requirements
### Requirement: Per-model status rows displayed below overall status
The public usage page SHALL display per-model status rows below the existing overall status section within the same Status card.

#### Scenario: Per-model rows layout
- **WHEN** the uptime API returns a `models` array with entries
- **THEN** the page SHALL display one row per model, each containing the model name, a status badge, and an `UptimeBars` component

#### Scenario: Model name display
- **WHEN** a per-model row is rendered
- **THEN** the model name SHALL be displayed as text to the left of the status badge

#### Scenario: Status badge reuses existing helpers
- **WHEN** a per-model status badge is rendered
- **THEN** the badge color SHALL use the existing `statusBadgeColor()` function and the label SHALL use the existing `statusBadgeLabel()` function

#### Scenario: Uptime bars reuse existing component
- **WHEN** per-model uptime bars are rendered
- **THEN** each model row SHALL use the existing `UptimeBars` component with the model's buckets mapped to `{ timestamp, total, success }` format

### Requirement: Per-model section always visible
The per-model status section SHALL always be visible (not collapsible, no toggle) when uptime data is available.

#### Scenario: No collapse toggle
- **WHEN** the page loads with uptime data containing per-model entries
- **THEN** all per-model rows SHALL be visible without requiring user interaction to expand or show them

### Requirement: Per-model section with visual separator
The per-model rows SHALL be visually separated from the overall status with a horizontal divider.

#### Scenario: Divider between overall and per-model
- **WHEN** both overall status and per-model status are displayed
- **THEN** a `q-separator` SHALL appear between the overall uptime bars and the first per-model row

### Requirement: Per-model handles empty models array
The per-model section SHALL gracefully handle when the `models` array is empty or absent.

#### Scenario: No models in response
- **WHEN** the uptime API returns an empty `models` array or the field is absent
- **THEN** the per-model section SHALL not render (no empty state, no error -- just the overall status as before)

