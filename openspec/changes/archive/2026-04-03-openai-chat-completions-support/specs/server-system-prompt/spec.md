## MODIFIED Requirements

### Requirement: Server has optional system_prompt field
The `servers` table SHALL include an optional `system_prompt` TEXT column. Servers created without a system_prompt SHALL have NULL value.

#### Scenario: Create server without system_prompt
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with `{"name": "Primary", "base_url": "https://api.example.com", "api_key": "sk-ant-xxx"}` (no system_prompt field)
- **THEN** the system SHALL create the server with `system_prompt: null`

#### Scenario: Create server with system_prompt
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with `{"name": "Primary", "base_url": "https://api.example.com", "api_key": "sk-ant-xxx", "system_prompt": "Always respond in Vietnamese"}`
- **THEN** the system SHALL create the server with the provided system_prompt and return HTTP 201 with the created server object including `system_prompt: "Always respond in Vietnamese"`

#### Scenario: Retrieve server includes system_prompt
- **WHEN** an authenticated admin sends GET `/api/admin/servers/{id}`
- **THEN** the response SHALL include the `system_prompt` field (null or string value)

#### Scenario: Update server system_prompt
- **WHEN** an authenticated admin sends PUT `/api/admin/servers/{id}` with `{"system_prompt": "New system prompt"}`
- **THEN** the system SHALL update the server's system_prompt and return HTTP 200 with the updated server object

#### Scenario: Clear server system_prompt
- **WHEN** an authenticated admin sends PUT `/api/admin/servers/{id}` with `{"system_prompt": null}`
- **THEN** the system SHALL set the server's system_prompt to null and return HTTP 200

### Requirement: Server system_prompt is returned in list and detail endpoints
The GET `/api/admin/servers` and GET `/api/admin/servers/{id}` endpoints SHALL include the `system_prompt` field in all server objects returned.

#### Scenario: List servers includes system_prompt
- **WHEN** an authenticated admin sends GET `/api/admin/servers?page=1&limit=20`
- **THEN** each server object in the response SHALL include the `system_prompt` field

### Requirement: System prompt merge applies to both Anthropic and OpenAI endpoints
When a server has a non-null `system_prompt`, the proxy SHALL merge it into the outgoing request body. For `/v1/messages`, the merge targets the top-level `system` field using the existing hybrid strategy. For paths starting with `/v1/chat/`, the merge targets the `messages` array: appending to an existing system message's `content` or prepending a new system message.

#### Scenario: Anthropic merge unchanged
- **WHEN** the request path is `/v1/messages` and the server has a system_prompt
- **THEN** the proxy SHALL apply the existing top-level `system` field merge strategy

#### Scenario: OpenAI merge — prepend new system message
- **WHEN** the request path is `/v1/chat/completions`, the server has `system_prompt: "Always respond in Vietnamese"`, and the messages array has no system role entry
- **THEN** the proxy SHALL prepend `{"role":"system","content":"Always respond in Vietnamese"}` to the messages array

#### Scenario: OpenAI merge — append to existing system message
- **WHEN** the request path is `/v1/chat/completions`, the server has `system_prompt: "Always respond in Vietnamese"`, and the first message has `"role":"system","content":"You are helpful"`
- **THEN** the proxy SHALL update the first message to `"content":"You are helpful\n\nAlways respond in Vietnamese"`

#### Scenario: No merge on non-billing paths
- **WHEN** the request path is `/v1/messages/count_tokens` and the server has a system_prompt
- **THEN** the proxy SHALL NOT merge the system prompt (unchanged behavior)
