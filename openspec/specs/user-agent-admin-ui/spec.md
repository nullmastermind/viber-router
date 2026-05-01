## Purpose
TBD

## Requirements
### Requirement: User Agents tab on group detail page
The GroupDetailPage SHALL include a "User Agents" tab after the "Spam" tab. The tab panel SHALL contain a "Blocked User Agents" section with:
- A `q-select` with `use-input` and `use-chips` props that filters from the list of recorded UAs for the group. The admin SHALL also be able to type a custom UA string not in the recorded list.
- An "Add" button to block the selected/typed UA.
- A list of currently blocked UAs displayed as `q-chip` elements with a delete (x) button to unblock.
- A loading state while data is being fetched.
- An empty state message when no UAs are blocked.

#### Scenario: Tab visible on group detail page
- **WHEN** an admin navigates to a group's detail page
- **THEN** a "User Agents" tab SHALL be visible after the "Spam" tab

#### Scenario: Data loaded on tab activation
- **WHEN** the admin clicks the "User Agents" tab
- **THEN** the system SHALL fetch both recorded UAs and blocked UAs for the group

#### Scenario: Autocomplete from recorded UAs
- **WHEN** the admin types in the UA input field
- **THEN** the dropdown SHALL show matching recorded UAs filtered by the typed text

#### Scenario: Block a user-agent
- **WHEN** the admin selects or types a UA and clicks "Add"
- **THEN** the system SHALL call POST `/api/admin/groups/{id}/user-agents/blocked`, show a success notification, and refresh both lists

#### Scenario: Unblock a user-agent
- **WHEN** the admin clicks the delete (x) button on a blocked UA chip
- **THEN** the system SHALL call DELETE `/api/admin/groups/{id}/user-agents/blocked`, show a success notification, and refresh both lists

#### Scenario: Loading state
- **WHEN** the tab data is being fetched
- **THEN** the UI SHALL show a loading indicator

#### Scenario: Empty blocked list
- **WHEN** the group has no blocked UAs
- **THEN** the UI SHALL show an empty state message (e.g., "No blocked user agents")

#### Scenario: Error notification
- **WHEN** an add or remove operation fails
- **THEN** the UI SHALL show an error notification via q-notify

### Requirement: User-agent store functions
The groups Pinia store SHALL expose four functions for user-agent management:
- `fetchGroupUserAgents(groupId)` — GET `/api/admin/groups/{id}/user-agents`
- `fetchGroupBlockedUserAgents(groupId)` — GET `/api/admin/groups/{id}/user-agents/blocked`
- `addGroupBlockedUserAgent(groupId, userAgent)` — POST `/api/admin/groups/{id}/user-agents/blocked`
- `removeGroupBlockedUserAgent(groupId, userAgent)` — DELETE `/api/admin/groups/{id}/user-agents/blocked`

#### Scenario: Fetch recorded UAs
- **WHEN** `fetchGroupUserAgents(groupId)` is called
- **THEN** the store SHALL return the array of recorded UA objects from the API

#### Scenario: Add blocked UA
- **WHEN** `addGroupBlockedUserAgent(groupId, "BadBot/2.0")` is called
- **THEN** the store SHALL POST `{"user_agent": "BadBot/2.0"}` to the blocked endpoint

#### Scenario: Remove blocked UA
- **WHEN** `removeGroupBlockedUserAgent(groupId, "BadBot/2.0")` is called
- **THEN** the store SHALL DELETE with body `{"user_agent": "BadBot/2.0"}` to the blocked endpoint
