## Context

The Viber Router proxy forwards requests to upstream servers. Currently, there's no mechanism to inject or merge server-level instructions into requests. The Anthropic API supports two system prompt formats:
1. Simple string: `"system": "You are helpful"`
2. Array of text blocks: `"system": [{"type": "text", "text": "...", "cache_control": {...}}]`

The array format with `cache_control` enables prompt caching optimization. We need to preserve this optimization when merging server prompts.

## Goals / Non-Goals

**Goals:**
- Allow admins to configure a system prompt per upstream server
- Merge server system prompt with client-provided system prompt intelligently
- Preserve prompt caching metadata (cache_control) when present in client request
- Apply merge logic only to `/v1/messages` endpoint (only endpoint with system field)
- Maintain backward compatibility (system_prompt is optional)

**Non-Goals:**
- Support system prompts at group-server assignment level (per-server only)
- Implement system prompt versioning or templates
- Add UI for bulk system prompt management
- Support system prompts on non-messages endpoints

## Decisions

### 1. Hybrid Merge Strategy

**Decision**: Use hybrid approach based on client system format:
- If client sends array with `cache_control` → preserve array format, merge server prompt into last block's text
- If client sends string or array without `cache_control` → normalize to string, concatenate with server prompt
- If only server has prompt → use server prompt as-is
- If only client has prompt → passthrough unchanged

**Rationale**: Preserves prompt caching optimization when client uses it, while keeping simple cases simple. Merging into last block avoids creating new blocks and losing cache_control metadata.

**Alternatives considered**:
- Always append as new block: Simpler but loses cache_control on server prompt
- Always normalize to string: Simpler but breaks prompt caching for clients using it
- Per-group-server system prompts: More flexible but adds complexity; per-server is sufficient for most use cases

### 2. Data Model: system_prompt in servers table

**Decision**: Add `system_prompt TEXT NULL` column to `servers` table. Store as plain text, no JSON.

**Rationale**: System prompts are typically single strings. Storing as text is simpler than JSON. If future needs require structured data, can migrate to JSON later.

**Alternatives considered**:
- Store in group_servers table: Adds complexity; per-server is the primary use case
- Store as JSON with metadata: Premature; text is sufficient now

### 3. Merge Function Location

**Decision**: Create `merge_system_prompts()` function in `proxy.rs`. Call it before transforming model names and forwarding request.

**Rationale**: Proxy engine already transforms request bodies (model mapping, thinking block stripping). System prompt merge fits naturally in this pipeline.

**Alternatives considered**:
- Separate module: Overkill for a single function
- In models layer: Proxy is the right place since it has access to both client and server data

### 4. Merge Format

**Decision**: When concatenating strings, use `"\n\n"` as separator (two newlines).

**Rationale**: Clear visual separation in merged prompt. Matches common practice in prompt engineering.

### 5. Frontend: Textarea for system_prompt

**Decision**: Add textarea field (multi-line) in server create/edit forms. No character limit enforced in UI (let backend/API handle).

**Rationale**: System prompts can be long and multi-line. Textarea is appropriate. No artificial limit in UI; let users decide.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Merged prompt becomes very long | No truncation; let upstream API handle. Document best practices for system prompt length. |
| Cache_control metadata lost if client uses string format | By design (hybrid strategy). Acceptable trade-off: string format doesn't use caching anyway. |
| Server prompt overrides client intent | Server prompt is appended (lower priority). Document that server prompts should be complementary, not contradictory. |
| Merge logic complexity | Mitigated by clear test cases covering all scenarios (string+string, array+string, array with/without cache_control, etc.). |

## Migration Plan

1. **Database**: Run migration adding `system_prompt` column (nullable, no default)
2. **Backend**: Deploy updated models and API endpoints
3. **Proxy**: Deploy merge logic (no-op if system_prompt is null)
4. **Frontend**: Deploy updated server forms
5. **Rollback**: Remove system_prompt column; merge logic is no-op if column doesn't exist

No data migration needed; existing servers have null system_prompt.

## Open Questions

None — all decisions finalized during exploration phase.
