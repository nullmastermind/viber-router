## ADDED Requirements

### Requirement: TTFT chart in Group Detail page
The Group Detail page SHALL display a Chart.js line chart showing TTFT measurements per server over the last hour. Each server SHALL be rendered as a separate line/series with a distinct color.

#### Scenario: Chart displays TTFT data
- **WHEN** an admin views the Group Detail page for a group that has TTFT data in the last hour
- **THEN** the page SHALL show a line chart with time on the X-axis and TTFT (ms) on the Y-axis, with one line per server

#### Scenario: Chart shows timeout markers
- **WHEN** a TTFT measurement has `timed_out: true`
- **THEN** the chart SHALL display that point distinctly (e.g., different marker or color) to indicate a timeout event

#### Scenario: No TTFT data available
- **WHEN** the group has no TTFT data in the last hour
- **THEN** the chart card SHALL display an empty state message instead of an empty chart

#### Scenario: TTFT timeout configuration in group settings
- **WHEN** an admin views the Group Detail page
- **THEN** the Properties card SHALL include a field to view and edit the `ttft_timeout_ms` setting (empty/null means disabled)

### Requirement: Chart auto-refresh
The TTFT chart SHALL refresh its data periodically to show near-real-time TTFT measurements.

#### Scenario: Chart refreshes automatically
- **WHEN** the Group Detail page is open
- **THEN** the TTFT chart SHALL refresh data every 30 seconds
