## Implemented

- [x] GET /api/admin/logs/purge-preview?keep_days=N endpoint
- [x] POST /api/admin/logs/purge endpoint
- [x] Partition DROP TABLE logic for fully-expired partitions
- [x] DELETE FROM logic for partial partitions straddling the cutoff
- [x] partition.rs functions made pub
- [x] SettingsPage "Database Maintenance" section
- [x] Keep-days dropdown
- [x] Preview row count display
- [x] Confirmation dialog before purge
- [x] Success/error notifications
