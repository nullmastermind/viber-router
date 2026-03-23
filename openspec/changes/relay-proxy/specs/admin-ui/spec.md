## ADDED Requirements

### Requirement: Login page
The admin UI SHALL present a login page that accepts the admin token. On successful login, the token SHALL be stored in localStorage and the user redirected to the dashboard.

#### Scenario: Successful login
- **WHEN** the user enters a valid admin token and clicks Login
- **THEN** the token is stored in localStorage and the user is redirected to the Servers page

#### Scenario: Invalid login
- **WHEN** the user enters an invalid token and clicks Login
- **THEN** an error message "Invalid admin token" is displayed inline

#### Scenario: No token in storage
- **WHEN** the user navigates to any admin page without a token in localStorage
- **THEN** the user is redirected to the login page

### Requirement: Servers management page
The admin UI SHALL display a list of all servers with name, base_url, and api_key visible. The page SHALL support creating, editing, and deleting servers.

#### Scenario: List servers
- **WHEN** the user navigates to the Servers page
- **THEN** a table displays all servers with columns: Name, Base URL, API Key, Actions (Edit, Delete)

#### Scenario: Create server
- **WHEN** the user clicks "Add Server" and fills in name, base_url, api_key, then clicks Save
- **THEN** the server is created and appears in the table

#### Scenario: Edit server
- **WHEN** the user clicks Edit on a server row, modifies fields, and clicks Save
- **THEN** the server is updated in the table

#### Scenario: Delete server
- **WHEN** the user clicks Delete on a server row and confirms the action
- **THEN** the server is removed from the table

#### Scenario: Delete server in use
- **WHEN** the user clicks Delete on a server that is assigned to groups
- **THEN** an error message is displayed listing the groups that reference this server

### Requirement: Groups management page with server-side pagination
The admin UI SHALL display a paginated table of groups with search, filter, and bulk operations. Pagination, search, and filtering SHALL be server-side.

#### Scenario: Paginated group list
- **WHEN** the user navigates to the Groups page
- **THEN** a table displays groups with columns: Name, API Key (with copy button), Status (active/inactive), Servers count, Created At, and a checkbox for selection

#### Scenario: Search groups
- **WHEN** the user types in the search field
- **THEN** the table filters to show groups whose name matches the search term (server-side)

#### Scenario: Filter by active status
- **WHEN** the user selects "Active" or "Inactive" from the status filter
- **THEN** the table shows only groups matching that status

#### Scenario: Filter by server
- **WHEN** the user selects a server from the server filter dropdown
- **THEN** the table shows only groups that have that server assigned

#### Scenario: Bulk activate
- **WHEN** the user selects multiple groups via checkboxes and clicks "Activate"
- **THEN** all selected groups are set to active

#### Scenario: Bulk deactivate
- **WHEN** the user selects multiple groups via checkboxes and clicks "Deactivate"
- **THEN** all selected groups are set to inactive

#### Scenario: Bulk delete
- **WHEN** the user selects multiple groups via checkboxes and clicks "Delete" and confirms
- **THEN** all selected groups are deleted

#### Scenario: Bulk assign server
- **WHEN** the user selects multiple groups, clicks "Assign Server", selects a server and priority
- **THEN** the selected server is added to all selected groups with the specified priority

### Requirement: Group detail page
The admin UI SHALL display a group's full configuration including its servers with priorities and model mappings. The page SHALL support editing group properties, managing server assignments, and reordering priorities.

#### Scenario: View group detail
- **WHEN** the user clicks on a group row in the groups table
- **THEN** the group detail page shows: name, API key (with copy and regenerate buttons), status toggle, failover status codes editor, and a list of assigned servers ordered by priority

#### Scenario: Edit group properties
- **WHEN** the user modifies the group name or failover codes and clicks Save
- **THEN** the group is updated

#### Scenario: Copy API key
- **WHEN** the user clicks the Copy button next to the API key
- **THEN** the API key is copied to the clipboard

#### Scenario: Regenerate API key
- **WHEN** the user clicks Regenerate and confirms
- **THEN** a new API key is generated and displayed

#### Scenario: Add server to group
- **WHEN** the user clicks "Add Server", selects a server from a dropdown, sets priority, and optionally adds model mappings
- **THEN** the server is added to the group's server list

#### Scenario: Edit model mappings
- **WHEN** the user clicks on a server's model mappings section and adds/edits/removes mapping entries
- **THEN** the model mappings are updated

#### Scenario: Reorder servers by drag
- **WHEN** the user drags a server row to a new position in the server list
- **THEN** the server priorities are updated to reflect the new order

#### Scenario: Remove server from group
- **WHEN** the user clicks Remove on a server in the group's server list and confirms
- **THEN** the server is removed from the group

### Requirement: Navigation
The admin UI SHALL have a left sidebar navigation with links to Servers and Groups pages. The current page SHALL be highlighted.

#### Scenario: Navigate between pages
- **WHEN** the user clicks "Servers" or "Groups" in the sidebar
- **THEN** the corresponding page is displayed and the sidebar link is highlighted

#### Scenario: Logout
- **WHEN** the user clicks Logout
- **THEN** the token is removed from localStorage and the user is redirected to the login page
