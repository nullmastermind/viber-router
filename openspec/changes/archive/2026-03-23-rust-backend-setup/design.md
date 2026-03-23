## Context

The viber-router project is a Quasar/Vue 3 frontend that needs a Rust backend service acting as an LLM API proxy and load balancer. The backend must handle very high throughput. PostgreSQL stores persistent data, Redis handles rate limiting and fast in-memory operations.

This is a greenfield Rust project — no existing backend code exists.

## Goals / Non-Goals

**Goals:**
- Production-ready Rust backend scaffold with Axum
- PostgreSQL connection pool via SQLx with tunable pool size
- Redis connection pool via deadpool-redis with tunable pool size
- Health endpoint verifying both DB and Redis connectivity
- All configuration via `.env` file
- Structured logging via tracing
- Graceful shutdown

**Non-Goals:**
- API routes beyond health check (future changes)
- Rate limiting implementation (future — Redis pool is the foundation)
- Authentication/authorization
- Database migrations or schema design
- Docker/deployment configuration

## Decisions

### D1: Axum over Actix-web
Axum is built on Tokio/Tower ecosystem, composable middleware, and has strong community momentum. For a proxy/load balancer that will heavily use Tower layers, Axum is the natural fit.

### D2: SQLx over Diesel/SeaORM
SQLx is async-native, lightweight, and provides compile-time query checking. For a proxy service where the DB layer is secondary to throughput, SQLx's minimal overhead is preferred over a full ORM.

### D3: redis-rs + deadpool-redis over fred
redis-rs (v1.1) has the largest ecosystem and deadpool provides a proven async pool. While fred offers automatic pipelining, redis-rs is sufficient for rate limiting operations and has broader community support.

### D4: rustls over native-tls
Using `tls-rustls` feature for SQLx avoids dependency on system OpenSSL, making builds more portable and reproducible across environments.

### D5: Pool sizing for high throughput
- PostgreSQL: max 50 connections (configurable via `DATABASE_MAX_CONNECTIONS`)
- Redis: max 30 connections (configurable via `REDIS_MAX_CONNECTIONS`)
- These defaults target a proxy handling hundreds of concurrent LLM API requests

### D6: Application state via Axum's State extractor
A shared `AppState` struct holds `PgPool` and `deadpool_redis::Pool`, passed to handlers via Axum's `State` extractor. This is idiomatic Axum and avoids global state.

## Risks / Trade-offs

- [High pool defaults] → May consume excessive resources on small instances. Mitigation: all pool sizes configurable via `.env`.
- [No migrations included] → Database must be manually provisioned. Mitigation: explicit non-goal; migrations will be a separate change.
- [Single health endpoint] → No readiness vs liveness distinction. Mitigation: sufficient for initial setup; can split later if deploying to Kubernetes.
