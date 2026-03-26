## [2026-03-26] Round 1 (from spx-apply auto-verify)

### spx-verifier
- Fixed: Changed migration 017 pricing columns from NUMERIC to FLOAT8 to match Rust f64 type and avoid runtime decode errors

### spx-uiux-verifier
- Fixed: Rate badge in GroupDetailPage server rows now keyboard-accessible with tabindex, role="button", aria-label, and keydown handlers
- Fixed: Icon-only buttons (edit, tune, delete) in GroupDetailPage server rows now have descriptive aria-labels
- Fixed: Icon-only buttons (edit, delete) in ModelsPage actions column now have descriptive aria-labels

### spx-arch-verifier
- Fixed: Migration 017 uses FLOAT8 instead of NUMERIC, consistent with migration 018 rate columns and Rust f64 mapping
- Fixed: Price formatting in ModelsPage uses toFixed(4) for consistent decimal display
