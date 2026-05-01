## Purpose
TBD

## Requirements
### Requirement: Meter Results dialog fits viewport on mobile
The Meter Results dialog card SHALL use `width: 95vw; max-width: 480px` so it occupies 95% of the viewport width on narrow screens and is capped at 480px on wider screens.

#### Scenario: Dialog on a narrow mobile viewport
- **WHEN** the user opens the Meter Results dialog on a viewport narrower than 480px
- **THEN** the dialog card width SHALL be 95% of the viewport width and SHALL NOT overflow the screen

#### Scenario: Dialog on a desktop viewport
- **WHEN** the user opens the Meter Results dialog on a viewport wider than 480px
- **THEN** the dialog card width SHALL be at most 480px

### Requirement: Meter Results table scrolls horizontally on very narrow screens
The `q-card-section` containing the `q-markup-table` SHALL have `overflow-x: auto` so the 5-column table can scroll horizontally when the dialog width is insufficient to display all columns without truncation.

#### Scenario: Table overflow on very narrow viewport
- **WHEN** the dialog is displayed on a viewport where the table columns exceed the card width
- **THEN** the table section SHALL be horizontally scrollable and SHALL NOT break the card layout
