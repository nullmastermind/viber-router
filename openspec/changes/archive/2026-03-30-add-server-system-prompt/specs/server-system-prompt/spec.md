## ADDED Requirements

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
