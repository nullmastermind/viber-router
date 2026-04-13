## Overview

Two new admin endpoints for managing proxy_logs storage:

- `GET /api/admin/logs/purge-preview?keep_days=N` — returns `{ rows: u64 }` with the count of rows that would be deleted
- `POST /api/admin/logs/purge` with body `{ keep_days: u32 }` — executes the purge and returns `{ dropped_partitions: Vec<String>, deleted_rows: u64 }`

## Purge Logic

1. List all proxy_logs child partitions from pg_inherits
2. For partitions entirely older than keep_days: DROP TABLE
3. For the partition that straddles the cutoff date: DELETE FROM WHERE logged_at < cutoff
4. Preview endpoint runs the same logic in a read-only fashion (counts only, no mutations)

## partition.rs Changes

The following functions were changed from private to pub so logs.rs can call them:

- `list_log_partitions`
- `partition_date_range` (or equivalent date-extraction helper)

## Auth

Both endpoints require admin authentication (same middleware as other /api/admin/* routes).
