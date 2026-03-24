## [2026-03-24] Round 1 (from spx-apply auto-verify)

### spx-arch-verifier
- Fixed: Replaced sentinel string pattern (__KEEP__/__NULL__) in update_server handler with three separate SQL query branches to avoid data-integrity collision risk on api_key field
- Fixed: Renamed `any_server_had_key` to `any_server_attempted` in proxy handler for clarity

### spx-verifier
- Acknowledged: GroupsPage short_id copy button — short_id is visible in dropdown labels but copy button is not feasible in q-select options; copy functionality is available on ServersPage and GroupDetailPage where servers are listed
