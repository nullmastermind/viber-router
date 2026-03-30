## 1. Database Migration

- [x] 1.1 Create migration file `025_add_system_prompt_to_servers.sql` adding `system_prompt TEXT NULL` column to `servers` table ← (verify: migration file exists, column added correctly)

## 2. Backend Models (Rust)

- [x] 2.1 Update `Server` struct in `viber-router-api/src/models/server.rs` to add `system_prompt: Option<String>` field
- [x] 2.2 Update `CreateServer` struct to add `system_prompt: Option<String>` field
- [x] 2.3 Update `UpdateServer` struct to add `system_prompt: Option<Option<String>>` field
- [x] 2.4 Update `ServerResponse` struct in `viber-router-api/src/routes/admin/servers.rs` to add `system_prompt: Option<String>` field ← (verify: all structs compile, serialization works)

## 3. Backend API Endpoints

- [x] 3.1 Update `create_server` handler to accept and persist `system_prompt` from request body
- [x] 3.2 Update `update_server` handler to accept and update `system_prompt` field
- [x] 3.3 Update `list_servers` handler to include `system_prompt` in response
- [x] 3.4 Update `get_server` handler to include `system_prompt` in response ← (verify: all endpoints return system_prompt, create/update persist correctly)

## 4. Proxy Engine — System Prompt Merge Logic

- [x] 4.1 Create `merge_system_prompts()` function in `viber-router-api/src/routes/proxy.rs` that:
  - Takes `client_system: Option<serde_json::Value>` and `server_system: Option<String>` as inputs
  - Returns `Option<serde_json::Value>` (merged system or None)
  - Implements hybrid strategy: preserve array with cache_control, normalize to string otherwise
  - Handles all 6 merge scenarios (client string + server string, client array + server string, etc.)
- [x] 4.2 Add helper function to detect if a system array has cache_control metadata
- [x] 4.3 Add helper function to extract text from all blocks in a system array
- [x] 4.4 Integrate `merge_system_prompts()` into `proxy_handler()` function:
  - Call after parsing request body and extracting model
  - Only apply to `/v1/messages` endpoint
  - Merge client system with server system from `GroupServerDetail`
  - Update request body with merged system before forwarding ← (verify: merge logic works for all scenarios, only applies to /v1/messages)

## 5. Frontend Models (TypeScript)

- [x] 5.1 Update `Server` interface in `src/stores/servers.ts` to add `system_prompt: string | null` field
- [x] 5.2 Update `CreateServer` request type to include `system_prompt?: string`
- [x] 5.3 Update `UpdateServer` request type to include `system_prompt?: string | null`

## 6. Frontend — Server Forms

- [x] 6.1 Update create server form component to add textarea field for `system_prompt`
  - Label: "System Prompt (optional)"
  - Placeholder: "e.g., Always respond in Vietnamese"
  - Multi-line textarea, no character limit
- [x] 6.2 Update edit server form component to add textarea field for `system_prompt`
  - Pre-populate with existing value if present
  - Allow clearing (set to null)
- [x] 6.3 Update server detail view to display `system_prompt` in a read-only section ← (verify: forms render correctly, values persist on create/update)

## 7. Testing

- [x] 7.1 Write unit tests for `merge_system_prompts()` function covering:
  - Client string + server string → concatenated string
  - Client array with cache_control + server string → merged into last block
  - Client array without cache_control + server string → normalized to string
  - Only server has system → server system as-is
  - Only client has system → client system passthrough
  - Neither has system → None
  - Empty/null edge cases
- [x] 7.2 Write integration tests for proxy merge logic:
  - Create server with system_prompt
  - Send request to `/v1/messages` with client system
  - Verify merged system in upstream request body
  - Verify merge does NOT apply to `/v1/messages/count_tokens`
- [x] 7.3 Test API endpoints:
  - POST `/api/admin/servers` with system_prompt
  - PUT `/api/admin/servers/{id}` to update system_prompt
  - GET `/api/admin/servers` and `/api/admin/servers/{id}` return system_prompt ← (verify: all tests pass, coverage includes edge cases)

## 8. Full Check and Verification

- [x] 8.1 Run `just check` (type-check + lint for frontend and backend)
- [x] 8.2 Fix any lint or type errors
- [x] 8.3 Run backend tests: `cargo test` in `viber-router-api/`
- [x] 8.4 Run frontend tests (if applicable)
- [x] 8.5 Manual verification:
  - Create a server with system_prompt via admin UI
  - Send a test request to `/v1/messages` with client system
  - Verify merged system in proxy logs or via upstream inspection ← (verify: all checks pass, no errors, manual test successful)
