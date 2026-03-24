## MODIFIED Requirements

### Requirement: Configuration loaded from .env file
The system SHALL load environment variables from a `.env` file at startup using dotenvy. If the `.env` file does not exist, the system SHALL continue using existing environment variables without error. The following optional environment variable is added: LOG_RETENTION_DAYS (default: 30).

#### Scenario: .env file exists
- **WHEN** a `.env` file exists in the working directory
- **THEN** its variables are loaded into the environment before config parsing

#### Scenario: .env file missing
- **WHEN** no `.env` file exists
- **THEN** the server starts normally using existing environment variables

#### Scenario: LOG_RETENTION_DAYS configured
- **WHEN** LOG_RETENTION_DAYS is set to 7
- **THEN** the system SHALL use 7 days as the log retention period

#### Scenario: LOG_RETENTION_DAYS not set
- **WHEN** LOG_RETENTION_DAYS is not set
- **THEN** the system SHALL default to 30 days for log retention
