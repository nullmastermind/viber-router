## [2026-03-23] Round 1 (from spx-apply auto-verify)

### spx-arch-verifier
- Fixed: [CRITICAL] Moved reqwest::Client from per-request creation to shared AppState field (proxy.rs, routes/mod.rs, main.rs)
- Fixed: [CRITICAL] Wrapped reorder_priorities in a database transaction (group_servers.rs)
- Fixed: [WARNING] Hardcoded baseURL in axios.ts replaced with env variable fallback

### spx-verifier
- Fixed: [CRITICAL] Proxy now uses OriginalUri extractor to preserve /v1 prefix in upstream URLs (proxy.rs)
- Fixed: [CRITICAL] Added "Bulk Assign Server" button and dialog to GroupsPage.vue
- Fixed: [CRITICAL] Added servers_count to list_groups response (GroupListItem model + SQL subquery) and frontend column
