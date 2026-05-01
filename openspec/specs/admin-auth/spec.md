## Purpose
TBD

## Requirements
### Requirement: Admin endpoints require ADMIN_TOKEN authentication
All `/api/admin/*` endpoints SHALL require a valid `Authorization: Bearer <token>` header where `<token>` matches the `ADMIN_TOKEN` environment variable.

#### Scenario: Valid token
- **WHEN** a request to `/api/admin/*` includes `Authorization: Bearer <valid-token>`
- **THEN** the system SHALL process the request normally

#### Scenario: Missing authorization header
- **WHEN** a request to `/api/admin/*` has no `Authorization` header
- **THEN** the system SHALL return HTTP 401 with `{"error": "Authorization header required"}`

#### Scenario: Invalid token
- **WHEN** a request to `/api/admin/*` includes `Authorization: Bearer <wrong-token>`
- **THEN** the system SHALL return HTTP 401 with `{"error": "Invalid admin token"}`

#### Scenario: ADMIN_TOKEN not configured
- **WHEN** the `ADMIN_TOKEN` environment variable is not set
- **THEN** the application SHALL fail to start with a clear error message indicating ADMIN_TOKEN is required

### Requirement: Login endpoint for UI
The system SHALL provide a POST `/api/admin/login` endpoint that validates the admin token and returns a success response. This endpoint does NOT require the Authorization header — it accepts the token in the request body.

#### Scenario: Valid login
- **WHEN** a POST request to `/api/admin/login` with `{"token": "<valid-admin-token>"}`
- **THEN** the system SHALL return HTTP 200 with `{"success": true}`

#### Scenario: Invalid login
- **WHEN** a POST request to `/api/admin/login` with `{"token": "<wrong-token>"}`
- **THEN** the system SHALL return HTTP 401 with `{"error": "Invalid admin token"}`
