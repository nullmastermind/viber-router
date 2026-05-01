## Purpose
TBD

## Requirements
### Requirement: PostgreSQL connection pool is established at startup
The system SHALL create a SQLx PgPool using the DATABASE_URL environment variable. The pool SHALL be configured with a maximum number of connections specified by DATABASE_MAX_CONNECTIONS, defaulting to 50.

#### Scenario: Successful database connection
- **WHEN** DATABASE_URL is set to a valid PostgreSQL connection string
- **THEN** a connection pool is created and the application starts successfully

#### Scenario: Missing DATABASE_URL
- **WHEN** DATABASE_URL is not set
- **THEN** the application SHALL fail to start with a clear error message indicating DATABASE_URL is required

#### Scenario: Invalid DATABASE_URL
- **WHEN** DATABASE_URL is set to an invalid connection string
- **THEN** the application SHALL fail to start with a clear error message

### Requirement: Database pool size is configurable
The system SHALL read DATABASE_MAX_CONNECTIONS from the environment to set the maximum pool size. If not set, it SHALL default to 50.

#### Scenario: Custom pool size
- **WHEN** DATABASE_MAX_CONNECTIONS is set to `20`
- **THEN** the pool is created with a maximum of 20 connections

#### Scenario: Default pool size
- **WHEN** DATABASE_MAX_CONNECTIONS is not set
- **THEN** the pool is created with a maximum of 50 connections

