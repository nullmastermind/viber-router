## Why

Upstream servers often need to enforce consistent behavior or instructions across all requests routed through them. Currently, there's no way to configure per-server system prompts. This feature enables admins to define server-level system prompts that are automatically merged with client-provided system prompts, allowing for centralized instruction management without requiring client-side changes.

## What Changes

- Add `system_prompt` field to the `servers` table (optional TEXT column)
- Extend server creation and update endpoints to accept and persist system prompts
- Implement hybrid system prompt merge logic in the proxy engine:
  - If client sends array-format system with cache_control, merge server prompt into the last block's text while preserving cache_control
  - If client sends string-format system or array without cache_control, normalize to string and concatenate
  - If only server has system prompt, use it as-is
  - If only client has system prompt, passthrough unchanged
- Apply merge logic only to `/v1/messages` endpoint
- Update admin UI to display and edit server system prompts via textarea

## Capabilities

### New Capabilities
- `server-system-prompt`: Ability to configure and persist a system prompt per upstream server, with automatic merge into client requests

### Modified Capabilities
- `server-management`: Server creation and update now accept optional system_prompt field
- `proxy-engine`: Request forwarding now includes system prompt merge logic for `/v1/messages`

## Impact

- **Database**: New migration adding `system_prompt` column to `servers` table
- **Backend Models**: `Server`, `CreateServer`, `UpdateServer`, `ServerResponse` structs updated
- **Backend API**: POST/PUT `/api/admin/servers` endpoints accept system_prompt; GET endpoints return it
- **Proxy Engine**: `proxy.rs` gains system prompt merge function; request body transformation before upstream forward
- **Frontend**: Server form components add textarea field for system_prompt; server detail view displays it
- **No breaking changes**: system_prompt is optional; existing servers unaffected
