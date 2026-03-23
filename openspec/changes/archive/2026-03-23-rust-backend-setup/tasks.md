## 1. Project Scaffold

- [x] 1.1 Initialize Rust project with `cargo init viber-router-api`
- [x] 1.2 Configure Cargo.toml with all dependencies (axum 0.8, sqlx 0.8, tokio 1, redis 1.1, deadpool-redis 0.23, dotenvy 0.15, tracing 0.1, tracing-subscriber 0.3, anyhow 1, serde 1, serde_json 1)
- [x] 1.3 Create `.env.example` with all config variables (HOST, PORT, DATABASE_URL, DATABASE_MAX_CONNECTIONS, REDIS_URL, REDIS_MAX_CONNECTIONS, RUST_LOG)
- [x] 1.4 Create `.env` with local development defaults ← (verify: .env is gitignored)

## 2. Configuration

- [x] 2.1 Create `src/config.rs` — Config struct that loads and validates all env vars via dotenvy, with defaults for HOST (0.0.0.0), PORT (3000), DATABASE_MAX_CONNECTIONS (50), REDIS_MAX_CONNECTIONS (30), RUST_LOG (info) ← (verify: missing DATABASE_URL or REDIS_URL causes clear startup failure, optional vars have correct defaults)

## 3. Database Connection

- [x] 3.1 Create `src/db.rs` — function to create PgPool from Config (max_connections from config, connect with DATABASE_URL) ← (verify: pool respects DATABASE_MAX_CONNECTIONS, invalid DATABASE_URL fails clearly)

## 4. Redis Connection

- [x] 4.1 Create `src/redis.rs` — function to create deadpool_redis::Pool from Config (max_size from config, connect with REDIS_URL) ← (verify: pool respects REDIS_MAX_CONNECTIONS, invalid REDIS_URL fails clearly)

## 5. Application State & Routes

- [x] 5.1 Create `src/routes/mod.rs` — define router with AppState (PgPool + Redis Pool)
- [x] 5.2 Create `src/routes/health.rs` — GET /health handler that pings DB (sqlx::query SELECT 1) and Redis (PING), returns JSON with status/db/redis fields, 200 when all ok, 503 when any fails ← (verify: all 4 health-check spec scenarios — both ok, db down, redis down, both down)

## 6. Server Entrypoint

- [x] 6.1 Create `src/main.rs` — load config, init tracing-subscriber with env-filter, create DB pool, create Redis pool, build router, bind to HOST:PORT, graceful shutdown on ctrl+c ← (verify: server starts with valid config, structured logs appear, graceful shutdown works)
