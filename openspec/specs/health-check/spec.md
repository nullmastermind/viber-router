### Requirement: Health endpoint returns system status
The system SHALL expose a `GET /health` endpoint that checks connectivity to both PostgreSQL and Redis, returning a JSON response with individual component statuses.

#### Scenario: All services healthy
- **WHEN** a GET request is made to `/health` and both PostgreSQL and Redis are reachable
- **THEN** the response status is 200 and the body is `{"status":"ok","db":"ok","redis":"ok"}`

#### Scenario: Database unreachable
- **WHEN** a GET request is made to `/health` and PostgreSQL is unreachable
- **THEN** the response status is 503 and the body is `{"status":"error","db":"error","redis":"ok"}`

#### Scenario: Redis unreachable
- **WHEN** a GET request is made to `/health` and Redis is unreachable
- **THEN** the response status is 503 and the body is `{"status":"error","db":"ok","redis":"error"}`

#### Scenario: Both services unreachable
- **WHEN** a GET request is made to `/health` and both PostgreSQL and Redis are unreachable
- **THEN** the response status is 503 and the body is `{"status":"error","db":"error","redis":"error"}`

