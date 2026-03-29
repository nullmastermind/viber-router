## ADDED Requirements

### Requirement: Per-server password protection
The system SHALL allow an optional password to be set on any upstream server. When a password is set, the server's `base_url` and `api_key` SHALL be hidden from admin users until the correct password is provided.

> **Implementation note:** Credential masking is enforced server-side via the API — the frontend never receives real credentials for locked servers. Unlock state is maintained in-memory per API process and resets on server restart.

#### Scenario: Server without password
- **WHEN** an admin views a server that has no password set
- **THEN** the server's `base_url` and `api_key` SHALL be displayed in full

#### Scenario: Server with password — list view
- **WHEN** an admin views the servers list for a server that has a password set
- **THEN** the `base_url` column SHALL display the server's name with a lock icon (e.g. `🔒 OpenAI`)
- **THEN** the `api_key` column SHALL display the server's name with a lock icon
- **THEN** clicking the lock icon SHALL open a password prompt dialog

#### Scenario: Server with password — edit form
- **WHEN** an admin clicks the edit button for a protected server
- **THEN** the system SHALL open a password prompt before revealing the edit form
- **WHEN** the admin enters the correct password
- **THEN** the edit form SHALL open pre-filled with the server's real `base_url` and `api_key`
- **WHEN** the admin enters an incorrect password
- **THEN** the system SHALL show an error message "Incorrect password" and keep the prompt open

#### Scenario: Server with password — creating a protected server
- **WHEN** an admin creates a new server and enters a value in the "Protect password" field
- **THEN** the system SHALL hash the password using SHA-256 and store it in the `password_hash` column
- **THEN** the server SHALL be treated as protected immediately after creation

#### Scenario: Server with password — updating the password
- **WHEN** an admin edits a protected server and enters a new value in the "Protect password" field
- **THEN** the system SHALL update the stored password hash
- **WHEN** the admin clears the password field (leaves it empty)
- **THEN** the system SHALL clear the password hash, removing protection from the server

### Requirement: Unlock verification endpoint
The system SHALL provide a verification endpoint that returns a server's real credentials when given the correct password.

#### Scenario: Correct password
- **WHEN** a verified admin sends `POST /api/admin/servers/{id}/verify-password` with body `{"password": "correct_password"}`
- **THEN** the system SHALL return HTTP 200 with `{"base_url": "...", "api_key": "..."}`

#### Scenario: Incorrect password
- **WHEN** a verified admin sends `POST /api/admin/servers/{id}/verify-password` with body `{"password": "wrong_password"}`
- **THEN** the system SHALL return HTTP 401 with `{"error": "Incorrect password"}`

#### Scenario: Server not found
- **WHEN** a verified admin sends `POST /api/admin/servers/{non-existent-id}/verify-password`
- **THEN** the system SHALL return HTTP 404 with `{"error": "Server not found"}`

#### Scenario: Server has no password
- **WHEN** a verified admin sends `POST /api/admin/servers/{id}/verify-password` for a server with no password set
- **THEN** the system SHALL return HTTP 200 with the server's real `base_url` and `api_key`

### Requirement: Session-level unlock state
After a successful password verification, the system SHALL keep the server unlocked for the remainder of the browser session. Refreshing the page SHALL reset all unlock states.

> **Implementation note:** Unlock state is stored in an in-memory `HashSet<Uuid>` in `AppState`. All API endpoints that return server data (list, get, create, update, delete, groups) check this set and mask credentials for servers not in it. The set is process-local — it resets on API server restart. Unlock state is NOT persisted across restarts.

#### Scenario: Unlock persists within session
- **WHEN** an admin successfully unlocks a protected server in the servers list
- **THEN** the real `base_url` and `api_key` SHALL be displayed without re-entering the password
- **WHEN** the admin navigates to another page within the same browser session
- **THEN** the server SHALL remain unlocked

#### Scenario: Unlock resets on page refresh
- **WHEN** an admin refreshes the browser page
- **THEN** all previously unlocked servers SHALL return to the locked state
- **AND** the admin SHALL need to re-enter the password to view real credentials

### Requirement: Group detail server list protection
The system SHALL apply password protection to server information displayed in the group detail page.

#### Scenario: Protected server in group server list
- **WHEN** an admin views the Servers tab of a group that includes a password-protected server
- **THEN** the server's `base_url` SHALL be replaced with the server's name
- **THEN** a lock icon SHALL be displayed next to the server name

#### Scenario: Edit protected server from group page
- **WHEN** an admin clicks the edit button for a protected server in the group server list
- **THEN** the system SHALL prompt for the server password before opening the edit dialog
- **WHEN** the correct password is provided
- **THEN** the edit dialog SHALL open with real values pre-filled

### Requirement: Log failover chain protection
The system SHALL protect upstream URLs shown in log rows for protected servers.

#### Scenario: Protected server in failover chain
- **WHEN** an admin views an expanded log row whose `failover_chain` includes a protected server
- **THEN** the `upstream_url` field SHALL display the server's name instead of the real URL
- **AND** a lock icon SHALL indicate the field is protected

#### Scenario: Download cURL for protected server
- **WHEN** an admin clicks "Download cURL" for a protected server in a log row
- **THEN** the system SHALL prompt for the server password
- **WHEN** the correct password is provided
- **THEN** the cURL file SHALL be generated with the real upstream URL and downloaded
- **WHEN** an incorrect password is provided
- **THEN** the system SHALL show an error and not generate the file
