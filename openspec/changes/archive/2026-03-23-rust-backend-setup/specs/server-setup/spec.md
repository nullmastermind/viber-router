## ADDED Requirements

### Requirement: Server starts and listens on configured address
The system SHALL start an Axum HTTP server on the address specified by HOST and PORT environment variables. If HOST or PORT are not set, the server SHALL default to `0.0.0.0:3000`.

#### Scenario: Server starts with default config
- **WHEN** HOST and PORT are not set in environment
- **THEN** the server starts listening on `0.0.0.0:3000`

#### Scenario: Server starts with custom config
- **WHEN** HOST is set to `127.0.0.1` and PORT is set to `8080`
- **THEN** the server starts listening on `127.0.0.1:8080`

### Requirement: Server shuts down gracefully
The system SHALL listen for SIGINT (ctrl+c) and shut down gracefully, allowing in-flight requests to complete.

#### Scenario: Graceful shutdown on ctrl+c
- **WHEN** the server receives SIGINT
- **THEN** the server stops accepting new connections and shuts down after in-flight requests complete

### Requirement: Structured logging via tracing
The system SHALL initialize tracing-subscriber with env-filter support. The log level SHALL be configurable via the RUST_LOG environment variable, defaulting to `info`.

#### Scenario: Default log level
- **WHEN** RUST_LOG is not set
- **THEN** the server logs at `info` level

#### Scenario: Custom log level
- **WHEN** RUST_LOG is set to `debug`
- **THEN** the server logs at `debug` level

### Requirement: Configuration loaded from .env file
The system SHALL load environment variables from a `.env` file at startup using dotenvy. If the `.env` file does not exist, the system SHALL continue using existing environment variables without error.

#### Scenario: .env file exists
- **WHEN** a `.env` file exists in the working directory
- **THEN** its variables are loaded into the environment before config parsing

#### Scenario: .env file missing
- **WHEN** no `.env` file exists
- **THEN** the server starts normally using existing environment variables
