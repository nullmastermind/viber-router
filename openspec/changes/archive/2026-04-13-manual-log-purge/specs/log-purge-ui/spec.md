## Overview

A "Database Maintenance" section added to SettingsPage.vue that lets admins purge old proxy_logs partitions.

## UI Elements

- Section header: "Database Maintenance"
- Keep-days dropdown: options for 7, 14, 30, 60, 90 days
- Preview button: calls GET /api/admin/logs/purge-preview and displays the row count
- Row count display: shows "X rows will be deleted" after preview
- Purge button: opens a confirmation dialog before executing
- Confirmation dialog: warns that the action is irreversible, requires explicit confirm
- Success/error notifications via Quasar Notify

## Behavior

1. Admin selects keep_days from dropdown
2. Admin clicks "Preview" — row count is fetched and displayed
3. Admin clicks "Purge" — confirmation dialog appears
4. On confirm — POST /api/admin/logs/purge is called, success notification shown with dropped partition names and deleted row count
5. On cancel — dialog closes, no action taken
