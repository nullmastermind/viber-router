## ADDED Requirements

### Requirement: Redis connection pool is established at startup
The system SHALL create a deadpool-redis connection pool using the REDIS_URL environment variable. The pool SHALL be configured with a maximum number of connections specified by REDIS_MAX_CONNECTIONS, defaulting to 30.

#### Scenario: Successful Redis connection
- **WHEN** REDIS_URL is set to a valid Redis connection string
- **THEN** a Redis connection pool is created and the application starts successfully

#### Scenario: Missing REDIS_URL
- **WHEN** REDIS_URL is not set
- **THEN** the application SHALL fail to start with a clear error message indicating REDIS_URL is required

#### Scenario: Invalid REDIS_URL
- **WHEN** REDIS_URL is set to an invalid connection string
- **THEN** the application SHALL fail to start with a clear error message

### Requirement: Redis pool size is configurable
The system SHALL read REDIS_MAX_CONNECTIONS from the environment to set the maximum pool size. If not set, it SHALL default to 30.

#### Scenario: Custom pool size
- **WHEN** REDIS_MAX_CONNECTIONS is set to `10`
- **THEN** the Redis pool is created with a maximum of 10 connections

#### Scenario: Default pool size
- **WHEN** REDIS_MAX_CONNECTIONS is not set
- **THEN** the Redis pool is created with a maximum of 30 connections
