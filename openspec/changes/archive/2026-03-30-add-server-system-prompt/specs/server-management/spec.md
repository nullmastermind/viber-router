## MODIFIED Requirements

### Requirement: Create a server
The system SHALL allow creating a new upstream server with name and base_url as required fields, api_key and system_prompt as optional fields. The system SHALL auto-assign a unique short_id (auto-increment integer).

#### Scenario: Successful server creation with api_key
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with `{"name": "Primary", "base_url": "https://api.example.com", "api_key": "sk-ant-xxx"}`
- **THEN** the system SHALL create the server with the provided api_key and return HTTP 201 with the created server object including its generated UUID and auto-assigned short_id

#### Scenario: Successful server creation without api_key
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with `{"name": "Dynamic Only", "base_url": "https://api.example.com"}`
- **THEN** the system SHALL create the server with null api_key and return HTTP 201 with the created server object including its generated UUID and auto-assigned short_id

#### Scenario: Successful server creation with system_prompt
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with `{"name": "Primary", "base_url": "https://api.example.com", "api_key": "sk-ant-xxx", "system_prompt": "Always respond in Vietnamese"}`
- **THEN** the system SHALL create the server with the provided system_prompt and return HTTP 201 with the created server object including `system_prompt: "Always respond in Vietnamese"`

#### Scenario: Missing required fields
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with missing name or base_url
- **THEN** the system SHALL return HTTP 400 with a validation error message

## ADDED Requirements

### Requirement: Server short_id is visible and copyable in admin UI
The system SHALL display the server's short_id everywhere servers are shown in the admin UI (ServersPage, GroupDetailPage server list, GroupsPage bulk assign). Each short_id display SHALL include a copy button.

#### Scenario: Short ID shown on servers list page
- **WHEN** an admin views the servers list page
- **THEN** each server row SHALL display its short_id in a dedicated column with a copy-to-clipboard button

#### Scenario: Short ID shown on group detail page
- **WHEN** an admin views a group's detail page with assigned servers
- **THEN** each server in the server list SHALL display its short_id with a copy-to-clipboard button

#### Scenario: Server response includes short_id
- **WHEN** the admin API returns a server object (create, get, list, update)
- **THEN** the server object SHALL include the `short_id` integer field
