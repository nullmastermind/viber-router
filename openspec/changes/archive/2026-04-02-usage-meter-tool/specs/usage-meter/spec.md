## ADDED Requirements

### Requirement: Start usage meter
The system SHALL allow a user to start the usage meter, which captures a snapshot of the current `data.usage` array and begins tracking elapsed time.

#### Scenario: Start meter with usage data present
- **WHEN** the user clicks the "Start" button on the Usage Meter card
- **THEN** the system SHALL record a copy of the current `data.usage` array as the snapshot
- **THEN** the system SHALL record the current timestamp as the start time
- **THEN** the system SHALL display an elapsed time counter in HH:MM:SS format that increments every second
- **THEN** the system SHALL display a pulsing colored indicator while the meter is running
- **THEN** the system SHALL replace the "Start" button with a "Stop" button

### Requirement: Stop usage meter and show delta
The system SHALL allow a user to stop the usage meter, which re-fetches live usage data, computes per-model deltas against the snapshot, and displays the results in a dialog.

#### Scenario: Stop meter and display results
- **WHEN** the user clicks the "Stop" button while the meter is running
- **THEN** the system SHALL stop the elapsed time counter
- **THEN** the system SHALL silently re-fetch usage data from `/api/public/usage` using the current sub-key
- **THEN** the system SHALL compute per-model deltas: for each model in the fresh data, subtract the snapshot values for `effective_input_tokens`, `total_output_tokens`, `request_count`, and `cost_usd`
- **THEN** the system SHALL open a results dialog showing a table of per-model deltas and the total elapsed time
- **THEN** the system SHALL reset the meter to its initial (stopped) state

#### Scenario: Stop meter with no new usage
- **WHEN** the user clicks "Stop" and the re-fetched data is identical to the snapshot
- **THEN** the system SHALL still display the results dialog with all-zero deltas

### Requirement: Elapsed time display
The system SHALL display elapsed time in HH:MM:SS format while the meter is running.

#### Scenario: Elapsed time increments each second
- **WHEN** the meter is running
- **THEN** the elapsed time counter SHALL update every second
- **THEN** the format SHALL be HH:MM:SS (zero-padded, e.g., "00:01:05")

### Requirement: Usage Meter card placement
The system SHALL render the Usage Meter as a card section positioned between the "Usage (Last 30 Days)" table and the "Status" section.

#### Scenario: Card is visible when usage data is loaded
- **WHEN** `data` is non-null (usage data has been loaded for a sub-key)
- **THEN** the Usage Meter card SHALL be visible between the usage table and the status section

### Requirement: Timer cleanup on unmount
The system SHALL clean up the elapsed-time interval when the component is unmounted.

#### Scenario: Component unmounts while meter is running
- **WHEN** the component is unmounted while the meter is running
- **THEN** the setInterval for the elapsed time counter SHALL be cleared
