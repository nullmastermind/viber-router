## MODIFIED Requirements

### Requirement: Configuration loaded from .env file
The system SHALL load environment variables from a `.env` file at startup using dotenvy. If the `.env` file does not exist, the system SHALL continue using existing environment variables without error. The ADMIN_TOKEN environment variable SHALL be required — the application SHALL fail to start if it is not set.

#### Scenario: .env file exists
- **WHEN** a `.env` file exists in the working directory
- **THEN** its variables are loaded into the environment before config parsing

#### Scenario: .env file missing
- **WHEN** no `.env` file exists
- **THEN** the server starts normally using existing environment variables

#### Scenario: ADMIN_TOKEN not set
- **WHEN** ADMIN_TOKEN is not set in the environment
- **THEN** the application SHALL fail to start with a clear error message indicating ADMIN_TOKEN is required

## ADDED Requirements

### Requirement: Embedded database migrations run on startup
The system SHALL run embedded SQLx migrations automatically on startup before binding the HTTP listener. If migrations fail, the application SHALL fail to start with a clear error message.

#### Scenario: Migrations succeed
- **WHEN** the application starts and the database is reachable
- **THEN** all pending migrations are applied before the server begins accepting requests

#### Scenario: Migrations fail
- **WHEN** a migration fails (e.g., syntax error, constraint violation)
- **THEN** the application SHALL fail to start with the migration error message

#### Scenario: No pending migrations
- **WHEN** all migrations have already been applied
- **THEN** the application starts normally without re-running migrations
