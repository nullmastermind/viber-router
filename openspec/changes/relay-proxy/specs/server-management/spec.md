## ADDED Requirements

### Requirement: Create a server
The system SHALL allow creating a new upstream server with name, base_url, and api_key fields.

#### Scenario: Successful server creation
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with `{"name": "Primary", "base_url": "https://api.example.com", "api_key": "sk-ant-xxx"}`
- **THEN** the system SHALL create the server and return HTTP 201 with the created server object including its generated UUID

#### Scenario: Missing required fields
- **WHEN** an authenticated admin sends POST `/api/admin/servers` with missing name, base_url, or api_key
- **THEN** the system SHALL return HTTP 400 with a validation error message

### Requirement: List servers
The system SHALL return a paginated list of servers with optional search by name.

#### Scenario: List all servers
- **WHEN** an authenticated admin sends GET `/api/admin/servers`
- **THEN** the system SHALL return a paginated list of all servers with total count

#### Scenario: Search servers by name
- **WHEN** an authenticated admin sends GET `/api/admin/servers?search=primary`
- **THEN** the system SHALL return servers whose name contains "primary" (case-insensitive)

### Requirement: Get a server by ID
The system SHALL return a single server's details by its UUID.

#### Scenario: Server exists
- **WHEN** an authenticated admin sends GET `/api/admin/servers/{id}` with a valid server UUID
- **THEN** the system SHALL return HTTP 200 with the server object

#### Scenario: Server not found
- **WHEN** an authenticated admin sends GET `/api/admin/servers/{id}` with a non-existent UUID
- **THEN** the system SHALL return HTTP 404

### Requirement: Update a server
The system SHALL allow updating a server's name, base_url, and api_key. When a server is updated, all groups referencing this server SHALL have their Redis cache invalidated.

#### Scenario: Successful update
- **WHEN** an authenticated admin sends PUT `/api/admin/servers/{id}` with updated fields
- **THEN** the system SHALL update the server and return HTTP 200 with the updated server object

#### Scenario: Cache invalidation on update
- **WHEN** a server is updated and it is referenced by groups A and B
- **THEN** the Redis cache entries for group A and group B SHALL be invalidated

### Requirement: Delete a server
The system SHALL allow deleting a server. Deleting a server that is still assigned to groups SHALL be rejected.

#### Scenario: Delete unassigned server
- **WHEN** an authenticated admin sends DELETE `/api/admin/servers/{id}` for a server not assigned to any group
- **THEN** the system SHALL delete the server and return HTTP 204

#### Scenario: Delete assigned server
- **WHEN** an authenticated admin sends DELETE `/api/admin/servers/{id}` for a server assigned to one or more groups
- **THEN** the system SHALL return HTTP 409 with an error message listing the groups that reference this server
