## Why

The viber-router frontend (Quasar/Vue 3) needs a high-performance backend to serve as an LLM API proxy and load balancer. The backend must handle very high throughput with rate limiting, requiring both PostgreSQL for persistent data and Redis for fast in-memory operations. Setting up the foundation now enables all subsequent API development.

## What Changes

- Create a new Rust project `viber-router-api/` at the repository root as the backend service
- Set up Axum web server with production-ready configuration (graceful shutdown, structured logging)
- Configure PostgreSQL connection via SQLx with connection pooling (up to 50 connections)
- Configure Redis connection via deadpool-redis with connection pooling (up to 30 connections)
- Implement `GET /health` endpoint that verifies both DB and Redis connectivity
- All configuration loaded from `.env` via dotenvy with `.env.example` template

## Capabilities

### New Capabilities
- `server-setup`: Axum server initialization, graceful shutdown, structured logging via tracing
- `database-connection`: PostgreSQL connection pool via SQLx, configured through DATABASE_URL
- `redis-connection`: Redis connection pool via deadpool-redis, configured through REDIS_URL
- `health-check`: GET /health endpoint that pings both PostgreSQL and Redis, returns JSON status

### Modified Capabilities
<!-- None — this is a greenfield backend project -->

## Impact

- New directory `viber-router-api/` added to repository root
- New dependencies: axum 0.8, sqlx 0.8, tokio 1, redis 1.1, deadpool-redis 0.23, dotenvy 0.15, tracing, anyhow, serde
- Requires PostgreSQL and Redis running for the health endpoint to report healthy
- New `.env` file with DATABASE_URL, REDIS_URL, HOST, PORT configuration
